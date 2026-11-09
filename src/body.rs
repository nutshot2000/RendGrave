// Pixel bodies: every character is a hierarchy of parts (torso, head, arms, legs, weapon),
// each part a little grid of layered pixels: an outer layer (skin, cloth, armour) over an
// inner one (flesh, bone). Damage strips the outer layer to reveal the inner, then removes
// the pixel. Anything no longer connected to its joint falls off as a gib.
use crate::rng::*;

pub const PM_FLESH: u8 = 1;
pub const PM_BONE: u8 = 2;
pub const PM_CLOTH: u8 = 3;
pub const PM_METAL: u8 = 4;
pub const PM_HAIR: u8 = 5;
pub const PM_GLOW: u8 = 6;
pub const PM_WOOD: u8 = 7;

#[derive(Clone, Copy, Default)]
pub struct Px {
    pub m: u8,
    pub c: [u8; 3],
    pub hp: u8,
    pub im: u8,
    pub ic: [u8; 3],
    pub burn: u8,
    pub acid: u8,
    pub chr: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PK {
    Torso,
    Head,
    ArmF,
    ArmB,
    LegF,
    LegB,
    Weapon,
}

#[derive(Clone)]
pub struct Part {
    pub k: PK,
    pub w: i32,
    pub h: i32,
    pub px: Vec<Px>,
    pub pivot: (f32, f32),
    pub parent: i32,
    pub attach: (f32, f32),
    pub ang: f32,
    pub init: i32,
    pub on: bool,
}
impl Part {
    pub fn count(&self) -> i32 {
        self.px.iter().filter(|p| p.m != 0).count() as i32
    }
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Option<&Px> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return None;
        }
        let p = &self.px[(y * self.w + x) as usize];
        if p.m == 0 { None } else { Some(p) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Xf {
    pub ox: f32,
    pub oy: f32,
    pub c: f32,
    pub s: f32,
    pub a: f32,
}
impl Xf {
    #[inline]
    pub fn fwd(&self, pv: (f32, f32), lx: f32, ly: f32) -> (f32, f32) {
        let (x, y) = (lx - pv.0, ly - pv.1);
        (self.ox + x * self.c - y * self.s, self.oy + x * self.s + y * self.c)
    }
    #[inline]
    pub fn inv(&self, pv: (f32, f32), x: f32, y: f32) -> (f32, f32) {
        let (dx, dy) = (x - self.ox, y - self.oy);
        (dx * self.c + dy * self.s + pv.0, -dx * self.s + dy * self.c + pv.1)
    }
    pub fn new(ox: f32, oy: f32, a: f32) -> Xf {
        Xf { ox, oy, c: a.cos(), s: a.sin(), a }
    }
}

pub struct Detached {
    pub part: Part,
    pub xf: Xf,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Player,
    Zombie,
    Skeleton,
    Brute,
    /// a zombie that lost its legs long ago
    Crawler,
    /// skeleton with a bow
    Archer,
    /// act I boss: a giant crowned skeleton in rusted armour
    King,
    /// a swollen bog corpse full of acid and gas
    Bloater,
    /// a living outlaw with a sword or an axe
    Bandit,
    /// a bandit with a crossbow
    Crossbow,
    /// boss of the Ravenhold bandits
    Chief,
}
impl Kind {
    /// big, heavy enemies (bigger reach and footprint, resist knockback)
    pub fn big(self) -> bool {
        matches!(self, Kind::Brute | Kind::King | Kind::Chief)
    }
    /// made of bone: no blood, crumbles into bone chips
    pub fn bony(self) -> bool {
        matches!(self, Kind::Skeleton | Kind::Archer | Kind::King)
    }
    /// keeps its distance and shoots
    pub fn ranged(self) -> bool {
        matches!(self, Kind::Archer | Kind::Crossbow)
    }
    /// the shambling, grabbing dead
    pub fn shambler(self) -> bool {
        matches!(self, Kind::Zombie | Kind::Crawler | Kind::Bloater)
    }
    /// area bosses: health bar on top, boss music, no champions
    pub fn boss(self) -> bool {
        matches!(self, Kind::King | Kind::Chief)
    }
    /// living people: they bleed red and run when beaten
    pub fn bandit(self) -> bool {
        matches!(self, Kind::Bandit | Kind::Crossbow | Kind::Chief)
    }
}

pub struct Body {
    pub parts: Vec<Part>,
    pub xf: Vec<Xf>,
    pub mirror: bool,
    pub root: (f32, f32),
    pub dirty: bool,
    pub fx: bool,
    pub order: Vec<usize>,
    pub leg_len: f32,
    pub kind: Kind,
}

pub const T_TORSO: usize = 0;
pub const T_LEGB: usize = 1;
pub const T_LEGF: usize = 2;
pub const T_ARMB: usize = 3;
pub const T_HEAD: usize = 4;
pub const T_ARMF: usize = 5;
pub const T_WEAPON: usize = 6;

impl Body {
    pub fn compute(&mut self) {
        for i in 0..self.parts.len() {
            let p = &self.parts[i];
            if !p.on {
                continue;
            }
            let (ox, oy, pa) = if p.parent < 0 {
                (self.root.0, self.root.1, 0.0)
            } else {
                let pi = p.parent as usize;
                let x = self.xf[pi];
                let (a, b) = x.fwd(self.parts[pi].pivot, p.attach.0, p.attach.1);
                (a, b, x.a)
            };
            self.xf[i] = Xf::new(ox, oy, pa + p.ang);
        }
    }
    #[inline]
    pub fn mx(&self) -> f32 {
        if self.mirror { -1.0 } else { 1.0 }
    }
    /// position of a pixel relative to the feet (mirror applied)
    #[inline]
    pub fn rel(&self, i: usize, lx: f32, ly: f32) -> (f32, f32) {
        let (x, y) = self.xf[i].fwd(self.parts[i].pivot, lx, ly);
        (x * self.mx(), y)
    }
    pub fn pixel_at(&self, rx: f32, ry: f32) -> Option<(usize, i32, i32)> {
        for &i in self.order.iter().rev() {
            let p = &self.parts[i];
            if !p.on {
                continue;
            }
            let (lx, ly) = self.xf[i].inv(p.pivot, rx * self.mx(), ry);
            let (lx, ly) = (lx.floor() as i32, ly.floor() as i32);
            if p.get(lx, ly).is_some() {
                return Some((i, lx, ly));
            }
        }
        None
    }
    pub fn near_pixel(&self, rx: f32, ry: f32, r: i32) -> bool {
        for dy in -r..=r {
            for dx in -r..=r {
                if self.pixel_at(rx + dx as f32, ry + dy as f32).is_some() {
                    return true;
                }
            }
        }
        false
    }
    /// Call `f` on every pixel within r of (cx,cy) (relative coords); f gets the pixel, the distance and its height above the feet.
    pub fn each_in(&mut self, cx: f32, cy: f32, r: f32, mut f: impl FnMut(&mut Px, f32, f32)) {
        let mx = self.mx();
        for i in 0..self.parts.len() {
            if !self.parts[i].on {
                continue;
            }
            let xf = self.xf[i];
            let p = &mut self.parts[i];
            let pv = p.pivot;
            for ly in 0..p.h {
                for lx in 0..p.w {
                    let k = (ly * p.w + lx) as usize;
                    if p.px[k].m == 0 {
                        continue;
                    }
                    let (x, y) = xf.fwd(pv, lx as f32 + 0.5, ly as f32 + 0.5);
                    let x = x * mx;
                    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                    if d < r {
                        f(&mut p.px[k], d, -y);
                    }
                }
            }
        }
        self.fx = true;
    }
    /// Strip one layer from a pixel. Returns true if the pixel is now gone.
    pub fn strip(px: &mut Px) -> bool {
        if px.im != 0 {
            px.m = px.im;
            px.c = px.ic;
            px.im = 0;
            px.hp = 255;
            px.chr /= 2;
            false
        } else {
            px.m = 0;
            true
        }
    }
    pub fn hurt_px(px: &mut Px, amt: u8) -> bool {
        if px.hp > amt {
            px.hp -= amt;
            false
        } else {
            Self::strip(px)
        }
    }

    /// A blade cut along a line (relative coords) from the side nearest the attacker.
    /// Each material takes some of the blow; armour stops it. Returns the removed pixels.
    pub fn cut(&mut self, ax: f32, ay: f32, bx: f32, by: f32, power: f32) -> Vec<(f32, f32, u8)> {
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        let mx = self.mx();
        let mut cands: Vec<(f32, usize, usize, f32, f32)> = vec![];
        for i in 0..self.parts.len() {
            if !self.parts[i].on {
                continue;
            }
            let xf = self.xf[i];
            let p = &self.parts[i];
            for ly in 0..p.h {
                for lx in 0..p.w {
                    let k = (ly * p.w + lx) as usize;
                    if p.px[k].m == 0 {
                        continue;
                    }
                    let (x, y) = xf.fwd(p.pivot, lx as f32 + 0.5, ly as f32 + 0.5);
                    let x = x * mx;
                    let t = (((x - ax) * dx + (y - ay) * dy) / len2).clamp(0.0, 1.0);
                    let (qx, qy) = (ax + dx * t, ay + dy * t);
                    if (x - qx).powi(2) + (y - qy).powi(2) < 1.25 {
                        cands.push((t, i, k, x, y));
                    }
                }
            }
        }
        cands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut pw = power;
        let mut out = vec![];
        for (_, i, k, x, y) in cands {
            if pw <= 0.0 {
                break;
            }
            let px = &mut self.parts[i].px[k];
            let cost = |m: u8| match m {
                PM_FLESH => 1.0,
                PM_CLOTH => 0.6,
                PM_HAIR => 0.3,
                PM_BONE => 2.2,
                PM_WOOD => 2.5,
                PM_METAL => 9.0,
                _ => 0.5,
            };
            if px.m == PM_METAL && pw < cost(PM_METAL) {
                px.hp = px.hp.saturating_sub((pw * 25.0) as u8);
                px.chr = px.chr.saturating_add(40);
                out.push((x, y, PM_METAL));
                if px.hp == 0 {
                    Self::strip(px);
                }
                pw = 0.0;
                break;
            }
            let c = cost(px.m) + if px.im != 0 { cost(px.im) } else { 0.0 };
            pw -= c;
            out.push((x, y, if px.im != 0 { px.im } else { px.m }));
            px.m = 0;
            px.im = 0;
            self.dirty = true;
        }
        out
    }

    /// Burning and acid keep working on pixels over time.
    pub fn update_fx(&mut self, flames: &mut Vec<(f32, f32)>, drips: &mut Vec<(f32, f32, u8)>, immortal: bool) -> i32 {
        if !self.fx {
            return 0;
        }
        let mut any = false;
        let mut burning = 0;
        let mx = self.mx();
        for i in 0..self.parts.len() {
            if !self.parts[i].on {
                continue;
            }
            let xf = self.xf[i];
            let p = &mut self.parts[i];
            let (w, h) = (p.w, p.h);
            for ly in 0..h {
                for lx in 0..w {
                    let k = (ly * w + lx) as usize;
                    let px = p.px[k];
                    if px.m == 0 || (px.burn == 0 && px.acid == 0) {
                        continue;
                    }
                    any = true;
                    let mut q = px;
                    if q.burn > 0 {
                        burning += 1;
                        q.burn -= 1;
                        q.chr = q.chr.saturating_add(2);
                        let rate = match q.m {
                            PM_CLOTH => 0.12,
                            PM_HAIR => 0.3,
                            PM_FLESH => 0.05,
                            PM_WOOD => 0.04,
                            _ => 0.0,
                        };
                        if !immortal && rnd() < rate {
                            if Self::hurt_px(&mut q, 40) {
                                self.dirty = true;
                            }
                        }
                        if q.m == PM_BONE || q.m == PM_METAL {
                            q.burn = q.burn.saturating_sub(3);
                        }
                        if rnd() < 0.05 {
                            let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)][ri(4)];
                            let (nx, ny) = (lx + dx, ly + dy);
                            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                                let n = &mut p.px[(ny * w + nx) as usize];
                                if n.m != 0 && n.burn == 0 && matches!(n.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) {
                                    n.burn = rr(70.0, 150.0) as u8;
                                }
                            }
                        }
                        if rnd() < 0.035 {
                            let (x, y) = xf.fwd(p.pivot, lx as f32 + 0.5, ly as f32 + 0.5);
                            flames.push((x * mx, y));
                        }
                    }
                    if q.acid > 0 && q.m != 0 {
                        q.acid -= 1;
                        let amt = match q.m {
                            PM_FLESH => 9,
                            PM_CLOTH => 12,
                            PM_HAIR => 20,
                            PM_BONE => 4,
                            PM_WOOD => 5,
                            PM_METAL => 1,
                            _ => 6,
                        };
                        if rnd() < 0.5 {
                            let a = q.acid;
                            if immortal {
                                q.hp = q.hp.saturating_sub(amt).max(60);
                            } else if Self::hurt_px(&mut q, amt) {
                                self.dirty = true;
                            } else if q.m != 0 {
                                q.acid = a;
                            }
                        }
                        // acid runs downwards and spreads a little
                        if rnd() < 0.05 && ly + 1 < h {
                            let n = &mut p.px[((ly + 1) * w + lx) as usize];
                            if n.m != 0 && n.acid < 60 {
                                n.acid = 90;
                            }
                        }
                        if rnd() < 0.03 {
                            let (dx, dy) = [(1, 0), (-1, 0), (0, -1), (0, -1)][ri(4)];
                            let (nx, ny) = (lx + dx, ly + dy);
                            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                                let n = &mut p.px[(ny * w + nx) as usize];
                                if n.m != 0 && n.acid < 40 {
                                    n.acid = 110;
                                }
                            }
                        }
                        if rnd() < 0.008 {
                            let (x, y) = xf.fwd(p.pivot, lx as f32 + 0.5, ly as f32 + 0.5);
                            drips.push((x * mx, y, 0));
                        }
                    }
                    p.px[k] = q;
                }
            }
        }
        self.fx = any;
        burning
    }

    /// Find pieces cut off from their joint and turn them into free pieces.
    pub fn split(&mut self) -> Vec<Detached> {
        let mut out = vec![];
        if !self.dirty {
            return out;
        }
        self.dirty = false;
        let n = self.parts.len();
        let mut reach: Vec<Vec<bool>> = vec![vec![]; n];
        for i in 0..n {
            if !self.parts[i].on {
                continue;
            }
            // whole part falls off if its parent's attach point is gone
            if self.parts[i].parent >= 0 {
                let pi = self.parts[i].parent as usize;
                let ok = self.parts[pi].on && {
                    let pp = &self.parts[pi];
                    let (ax, ay) = self.parts[i].attach;
                    let mut found = false;
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let (x, y) = (ax as i32 + dx, ay as i32 + dy);
                            if pp.get(x, y).is_some() && !reach[pi].is_empty() && reach[pi][(y * pp.w + x) as usize] {
                                found = true;
                            }
                        }
                    }
                    found
                };
                if !ok {
                    self.detach_all(i, &mut out);
                    continue;
                }
            }
            let p = &self.parts[i];
            let (w, h) = (p.w, p.h);
            let mut seen = vec![false; (w * h) as usize];
            let mut q = vec![];
            let (pvx, pvy) = (p.pivot.0 as i32, p.pivot.1 as i32);
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let (x, y) = (pvx + dx, pvy + dy);
                    if p.get(x, y).is_some() {
                        let k = (y * w + x) as usize;
                        if !seen[k] {
                            seen[k] = true;
                            q.push((x, y));
                        }
                    }
                }
            }
            if q.is_empty() {
                self.detach_all(i, &mut out);
                continue;
            }
            while let Some((x, y)) = q.pop() {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (nx, ny) = (x + dx, y + dy);
                    if p.get(nx, ny).is_some() {
                        let k = (ny * w + nx) as usize;
                        if !seen[k] {
                            seen[k] = true;
                            q.push((nx, ny));
                        }
                    }
                }
            }
            let loose: Vec<usize> = (0..(w * h) as usize).filter(|&k| p.px[k].m != 0 && !seen[k]).collect();
            if !loose.is_empty() {
                let mut piece = p.clone();
                for k in 0..(w * h) as usize {
                    if seen[k] {
                        piece.px[k].m = 0;
                    }
                }
                for &k in &loose {
                    self.parts[i].px[k].m = 0;
                }
                if loose.len() > 2 {
                    out.push(Detached { part: piece, xf: self.xf[i] });
                }
            }
            reach[i] = seen;
        }
        out
    }
    fn detach_all(&mut self, i: usize, out: &mut Vec<Detached>) {
        if !self.parts[i].on {
            return;
        }
        self.parts[i].on = false;
        if self.parts[i].count() > 2 {
            out.push(Detached { part: self.parts[i].clone(), xf: self.xf[i] });
        }
        for j in 0..self.parts.len() {
            if self.parts[j].parent == i as i32 {
                self.detach_all(j, out);
            }
        }
    }
    /// tear off one part (and whatever hangs from it)
    pub fn detach_part(&mut self, i: usize) -> Vec<Detached> {
        let mut out = vec![];
        self.detach_all(i, &mut out);
        out
    }
    /// pixels currently being eaten by acid
    pub fn count_acid(&self) -> i32 {
        self.parts.iter().filter(|p| p.on).map(|p| p.px.iter().filter(|q| q.m != 0 && q.acid > 0).count() as i32).sum()
    }
    pub fn detach_everything(&mut self) -> Vec<Detached> {
        let mut out = vec![];
        for i in 0..self.parts.len() {
            if self.parts[i].on && self.parts[i].parent < 0 {
                self.detach_all(i, &mut out);
            }
        }
        out
    }
    /// does this part still reach its far end (a foot, a hand)?
    pub fn whole(&self, i: usize) -> bool {
        let p = &self.parts[i];
        if !p.on {
            return false;
        }
        (p.h - 3..p.h).any(|y| (0..p.w).any(|x| p.get(x, y).is_some()))
    }
    pub fn frac(&self, i: usize) -> f32 {
        let p = &self.parts[i];
        if !p.on || p.init == 0 {
            return 0.0;
        }
        p.count() as f32 / p.init as f32
    }
    pub fn tint_all(&mut self, f: impl Fn(&mut Px)) {
        for p in self.parts.iter_mut() {
            for px in p.px.iter_mut() {
                if px.m != 0 {
                    f(px);
                }
            }
        }
    }
}

// ---------------- generation ----------------
type Cell = Option<(u8, [u8; 3], u8, [u8; 3])>;

fn mul(c: [u8; 3], f: f32) -> [u8; 3] {
    [(c[0] as f32 * f).min(255.0) as u8, (c[1] as f32 * f).min(255.0) as u8, (c[2] as f32 * f).min(255.0) as u8]
}

fn mk(k: PK, w: i32, h: i32, pivot: (f32, f32), parent: i32, attach: (f32, f32), f: impl Fn(i32, i32) -> Cell) -> Part {
    let mut px = vec![Px::default(); (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if let Some((m, c, im, ic)) = f(x, y) {
                px[(y * w + x) as usize] = Px { m, c, hp: 255, im, ic, burn: 0, acid: 0, chr: 0 };
            }
        }
    }
    let mut p = Part { k, w, h, px, pivot, parent, attach, ang: 0.0, init: 0, on: true };
    shade(&mut p);
    p.init = p.count();
    p
}

/// Bake lighting into the sprite: rounded shading across each row, a top-left key light,
/// darker outlines and a little grain.
fn shade(p: &mut Part) {
    let (w, h) = (p.w, p.h);
    let mut out = p.px.clone();
    for y in 0..h {
        let row: Vec<i32> = (0..w).filter(|&x| p.px[(y * w + x) as usize].m != 0).collect();
        if row.is_empty() {
            continue;
        }
        let (a, b) = (*row.first().unwrap() as f32, *row.last().unwrap() as f32);
        for &x in &row {
            let k = (y * w + x) as usize;
            if p.px[k].m == PM_GLOW {
                continue;
            }
            let nx = if b > a { (x as f32 - a) / (b - a) } else { 0.5 };
            let mut l = 0.74 + 0.42 * (1.0 - ((nx - 0.32).abs() * 1.6).min(1.0));
            l += (1.0 - y as f32 / h as f32) * 0.06;
            l += (h2(x, y, (w * 31 + h) as u32) - 0.5) * 0.12;
            let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| p.get(x + dx, y + dy).is_none());
            if edge {
                l *= 0.68;
            }
            out[k].c = mul(p.px[k].c, l);
            if out[k].im != 0 {
                out[k].ic = mul(p.px[k].ic, l * 0.95);
            }
        }
    }
    p.px = out;
}

fn vary(c: [u8; 3], x: i32, y: i32, s: u32, amt: f32) -> [u8; 3] {
    mul(c, 1.0 + (h2(x, y, s) - 0.5) * amt)
}

pub fn humanoid(orig: Kind) -> Body {
    match orig {
        Kind::Bloater => return bloater(),
        Kind::Bandit | Kind::Crossbow => return bandit(orig),
        Kind::Chief => return chief(),
        _ => {}
    }
    // variants share a base body plan
    let kind = match orig {
        Kind::Crawler => Kind::Zombie,
        Kind::Archer | Kind::King => Kind::Skeleton,
        k => k,
    };
    let king = orig == Kind::King;
    // sizes: legs, torso, arms, head (pixels)
    let (lw, ll, tw, th, aw, al, hs) = if king {
        (8, 31, 21, 28, 6, 27, 15)
    } else {
        match kind {
            Kind::Player => (6, 20, 14, 19, 5, 17, 11),
            Kind::Zombie => (6, 21, 14, 19, 5, 19, 11),
            Kind::Skeleton => (5, 22, 13, 19, 4, 18, 10),
            _ => (9, 22, 24, 23, 8, 22, 13),
        }
    };
    let s = (next() & 0xffff) as u32;
    let skin: [u8; 3] = match kind {
        Kind::Zombie => [118 + ri(16) as u8, 136 + ri(12) as u8, 96],
        Kind::Brute => [178, 122, 98],
        _ => [222, 184, 152],
    };
    let bone = [214u8, 204, 182];
    let meat = [150u8, 40, 44];
    let shirt: [u8; 3] = match kind {
        Kind::Zombie => [[86u8, 72, 98], [74, 92, 70], [104, 82, 60]][ri(3)],
        Kind::Player => [52, 74, 160],
        _ => [90, 70, 50],
    };
    let pants: [u8; 3] = match kind {
        Kind::Zombie => [60, 58, 74],
        Kind::Player => [70, 64, 80],
        _ => [84, 64, 48],
    };
    let steel = [158u8, 166, 182];
    let iron = [118u8, 120, 128];
    let sk = kind == Kind::Skeleton;

    // torso -------------------------------------------------------------
    let torso = mk(PK::Torso, tw, th, (tw as f32 / 2.0, th as f32 - 0.5), -1, (0.0, 0.0), |x, y| {
        let cx = tw as f32 / 2.0;
        let fx = x as f32 + 0.5 - cx;
        let nx = fx / (tw as f32 / 2.0);
        let half = 1.0 - (y as f32 / th as f32) * 0.18;
        if sk {
            if king && y >= 2 && y < th - 7 && nx.abs() < 0.82 {
                let rust = vary([128, 86, 58], x, y, s + 11, 0.3);
                let rivet = (y == 3 || y == th - 9) && x % 4 == 1;
                return Some((PM_METAL, if rivet { [190, 170, 130] } else { rust }, PM_BONE, bone));
            }
            let spine = fx.abs() < 1.0;
            let ribs = y >= 2 && y < th * 6 / 10 && y % 3 != 1 && nx.abs() < 0.92 - (y as f32 / th as f32) * 0.2;
            let pelvis = y >= th - 4 && nx.abs() < 0.75 && !(y == th - 1 && fx.abs() < 1.5);
            let clav = y == 0 && nx.abs() < 0.95;
            if spine || ribs || pelvis || clav {
                return Some((PM_BONE, vary(bone, x, y, s, 0.12), 0, [0; 3]));
            }
            return None;
        }
        if nx.abs() > half {
            return None;
        }
        let inner_bone = fx.abs() < 1.0 || (y >= 2 && y < th * 6 / 10 && y % 3 == 0 && nx.abs() < half * 0.8);
        let (im, ic) = if inner_bone { (PM_BONE, bone) } else if nx.abs() < half - 0.15 { (PM_FLESH, meat) } else { (0, [0; 3]) };
        match kind {
            Kind::Zombie => {
                let torn = h2(x, y / 2, s + 1) < 0.22 && y > th / 3;
                if !torn && y < th - 3 {
                    Some((PM_CLOTH, vary(shirt, x, y, s, 0.2), PM_FLESH, skin))
                } else if nx < -0.15 && y > 3 && y < th * 6 / 10 && y % 3 == 0 {
                    Some((PM_BONE, bone, 0, [0; 3]))
                } else if h2(x, y, s + 2) < 0.12 {
                    Some((PM_FLESH, meat, im, ic))
                } else {
                    Some((PM_FLESH, vary(skin, x, y, s, 0.15), im, ic))
                }
            }
            Kind::Brute => {
                let plate = y >= 2 && y < th - 5 && nx.abs() < 0.72;
                let rivet = plate && (y == 3 || y == th - 7) && (x % 4 == 1);
                if plate {
                    Some((PM_METAL, if rivet { [214, 214, 222] } else { vary(iron, x, y, s, 0.1) }, PM_FLESH, skin))
                } else if y >= th - 5 {
                    Some((PM_CLOTH, [70, 50, 34], PM_FLESH, skin))
                } else {
                    Some((PM_FLESH, vary(skin, x, y, s, 0.1), im, ic))
                }
            }
            _ => {
                let tabard = fx.abs() < 3.5 && y > 3;
                let belt = y == th - 5 || y == th - 4;
                if belt {
                    Some((PM_CLOTH, if fx.abs() < 1.0 { [210, 176, 80] } else { [70, 46, 30] }, PM_FLESH, skin))
                } else if tabard {
                    Some((PM_CLOTH, vary(shirt, x, y, s, 0.08), PM_METAL, steel))
                } else {
                    Some((PM_METAL, vary(steel, x, y, s, 0.08), PM_FLESH, skin))
                }
            }
        }
    });
    // legs ---------------------------------------------------------------
    let leg = |k: PK, ax: f32| {
        mk(k, lw, ll, (lw as f32 / 2.0, 1.0), 0, (ax, th as f32 - 1.0), move |x, y| {
            let cx = lw as f32 / 2.0;
            let fx = x as f32 + 0.5 - cx;
            if sk {
                let knee = (y - ll / 2).abs() <= 1 && fx.abs() < 1.6;
                let foot = y >= ll - 2 && fx > -1.5;
                if fx.abs() < 1.0 || knee || foot {
                    return Some((PM_BONE, vary(bone, x, y, s + 3, 0.12), 0, [0; 3]));
                }
                return None;
            }
            let foot = y >= ll - 3;
            if foot && fx < -cx + 0.5 && y == ll - 3 {
                return None;
            }
            let (im, ic) = if fx.abs() < 1.0 { (PM_BONE, bone) } else { (PM_FLESH, meat) };
            match kind {
                Kind::Zombie => {
                    if foot {
                        Some((PM_FLESH, mul(skin, 0.8), im, ic))
                    } else if y < ll * 2 / 3 || h2(x, y, s + 4) > 0.3 {
                        Some((PM_CLOTH, vary(pants, x, y, s, 0.2), PM_FLESH, skin))
                    } else {
                        Some((PM_FLESH, vary(skin, x, y, s, 0.15), im, ic))
                    }
                }
                Kind::Brute => {
                    if y > ll / 2 {
                        Some((PM_METAL, vary(iron, x, y, s, 0.1), PM_FLESH, skin))
                    } else {
                        Some((PM_CLOTH, [74, 54, 38], PM_FLESH, skin))
                    }
                }
                _ => {
                    if foot {
                        Some((PM_CLOTH, [60, 42, 32], PM_FLESH, skin))
                    } else if y % 7 < 5 {
                        Some((PM_METAL, vary(steel, x, y, s, 0.08), PM_FLESH, skin))
                    } else {
                        Some((PM_CLOTH, pants, PM_FLESH, skin))
                    }
                }
            }
        })
    };
    let leg_off = lw as f32 / 2.0 - 0.5;
    let leg_b = leg(PK::LegB, tw as f32 / 2.0 + leg_off);
    let leg_f = leg(PK::LegF, tw as f32 / 2.0 - leg_off);
    // arms ---------------------------------------------------------------
    let arm = |k: PK, ax: f32| {
        mk(k, aw, al, (aw as f32 / 2.0, 1.0), 0, (ax, 2.0), move |x, y| {
            let cx = aw as f32 / 2.0;
            let fx = x as f32 + 0.5 - cx;
            if sk {
                let hand = y >= al - 3;
                if fx.abs() < 1.0 || (hand && fx.abs() < 1.6 && (x + y) % 2 == 0) || ((y - al / 2).abs() <= 0 && fx.abs() < 1.6) {
                    return Some((PM_BONE, vary(bone, x, y, s + 5, 0.12), 0, [0; 3]));
                }
                return None;
            }
            let hand = y >= al - 4;
            let (im, ic) = if fx.abs() < 1.0 { (PM_BONE, bone) } else { (PM_FLESH, meat) };
            match kind {
                Kind::Zombie => {
                    if hand {
                        Some((PM_FLESH, mul(skin, 0.9), im, ic))
                    } else if y < al / 3 && h2(x, y, s + 6) > 0.25 {
                        Some((PM_CLOTH, vary(shirt, x, y, s, 0.2), PM_FLESH, skin))
                    } else {
                        Some((PM_FLESH, vary(skin, x, y, s, 0.15), im, ic))
                    }
                }
                Kind::Brute => {
                    if y < 6 {
                        Some((PM_METAL, if y == 0 { [196, 198, 206] } else { vary(iron, x, y, s, 0.1) }, PM_FLESH, skin))
                    } else {
                        Some((PM_FLESH, vary(skin, x, y, s, 0.1), im, ic))
                    }
                }
                _ => {
                    if hand {
                        Some((PM_CLOTH, [80, 56, 40], PM_FLESH, skin))
                    } else {
                        Some((PM_METAL, vary(steel, x, y, s, 0.08), PM_FLESH, skin))
                    }
                }
            }
        })
    };
    let arm_b = arm(PK::ArmB, tw as f32 - 1.5);
    let arm_f = arm(PK::ArmF, 1.5);
    // head ---------------------------------------------------------------
    let head = mk(PK::Head, hs, hs, (hs as f32 / 2.0, hs as f32 - 0.5), 0, (tw as f32 / 2.0, 0.5), |x, y| {
        let c = hs as f32 / 2.0;
        let (fx, fy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c * 0.92);
        let r = (fx * fx + fy * fy * 1.05).sqrt();
        let neck = y >= hs - 2 && fx.abs() < 2.0;
        if r > c && !neck {
            return None;
        }
        let eye_y = (hs as f32 * 0.48) as i32;
        let eye = y == eye_y && (fx - 2.0).abs() < 1.1 || y == eye_y && (fx + 1.0).abs() < 0.9;
        match kind {
            Kind::Skeleton => {
                if king && y < 3 && (x % 3 == 0 || y == 2) {
                    return Some((PM_GLOW, [255, 200, 80], 0, [0; 3]));
                }
                if eye {
                    return Some((PM_GLOW, if king { [140, 200, 255] } else { [255, 70, 40] }, 0, [0; 3]));
                }
                if (y == eye_y || y == eye_y - 1) && (fx - 2.0).abs() < 1.6 || (y == eye_y || y == eye_y - 1) && (fx + 1.0).abs() < 1.4 {
                    return Some((PM_BONE, [40, 30, 30], 0, [0; 3]));
                }
                if y == hs - 3 && fx.abs() < 2.5 && x % 2 == 0 {
                    return Some((PM_BONE, [60, 50, 46], 0, [0; 3]));
                }
                Some((PM_BONE, vary(bone, x, y, s + 7, 0.1), 0, [0; 3]))
            }
            Kind::Zombie => {
                if eye {
                    return Some((PM_GLOW, [214, 220, 120], PM_BONE, bone));
                }
                if y < 3 && h2(x, y, s + 8) < 0.6 {
                    return Some((PM_HAIR, [44, 38, 30], PM_FLESH, skin));
                }
                if y == hs - 3 && fx.abs() < 2.0 {
                    return Some((PM_FLESH, [60, 20, 24], PM_BONE, bone));
                }
                Some((PM_FLESH, vary(skin, x, y, s + 9, 0.15), PM_BONE, bone))
            }
            Kind::Brute => {
                let slit = y == eye_y && fx.abs() < 3.5;
                if slit {
                    return Some((if fx.abs() > 1.5 && fx.abs() < 3.0 { PM_GLOW } else { PM_METAL }, if fx.abs() > 1.5 && fx.abs() < 3.0 { [255, 140, 40] } else { [30, 26, 30] }, PM_FLESH, skin));
                }
                if y < hs - 2 {
                    return Some((PM_METAL, vary(iron, x, y, s + 9, 0.1), PM_FLESH, skin));
                }
                Some((PM_FLESH, skin, PM_BONE, bone))
            }
            Kind::Player => {
                if y == eye_y && fx.abs() < 3.0 && fx > -2.0 {
                    return Some((PM_METAL, [26, 24, 30], PM_FLESH, skin));
                }
                if y == 0 && fx.abs() < 1.5 {
                    return Some((PM_CLOTH, [200, 50, 50], PM_METAL, steel));
                }
                Some((PM_METAL, vary(steel, x, y, s + 9, 0.08), PM_FLESH, skin))
            }
            _ => None,
        }
    });
    // weapon -------------------------------------------------------------
    let weapon = if orig == Kind::Archer {
        // a curved bow, held upright
        mk(PK::Weapon, 5, 20, (2.0, 10.0), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let t = y as f32 / 19.0;
            let bx = (3.6 - (t * std::f32::consts::PI).sin() * 3.4).round() as i32;
            if x == 4 - bx.min(4) || (x == 4 - bx.min(4) + 1 && (y == 0 || y == 19)) {
                return Some((PM_WOOD, vary([120, 84, 50], x, y, 41, 0.2), 0, [0; 3]));
            }
            if x == 4 {
                return Some((PM_CLOTH, [200, 196, 180], 0, [0; 3]));
            }
            None
        })
    } else if king {
        // a huge rusted greatsword
        mk(PK::Weapon, 6, 38, (3.0, 32.5), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let fx = x as f32 - 2.5;
            if y >= 33 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, [70, 46, 34], 0, [0; 3])) } else { None };
            }
            if y == 31 || y == 32 {
                return Some((PM_METAL, [150, 110, 60], 0, [0; 3]));
            }
            if fx.abs() < 2.0 && !(y == 0 && fx.abs() > 0.6) {
                let rust = h2(x, y, 98) < 0.45;
                return Some((PM_METAL, if rust { [130, 84, 54] } else if fx < 0.0 { [176, 172, 168] } else { [130, 128, 130] }, 0, [0; 3]));
            }
            None
        })
    } else {
        match kind {
        Kind::Player => mk(PK::Weapon, 5, 26, (2.5, 21.5), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let fx = x as f32 - 2.0;
            if y >= 22 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, [96, 60, 36], 0, [0; 3])) } else { None };
            }
            if y == 20 || y == 21 {
                return Some((PM_METAL, [214, 180, 80], 0, [0; 3]));
            }
            if fx.abs() < 1.5 && !(y == 0 && fx.abs() > 0.5) {
                return Some((PM_METAL, if fx < 0.0 { [236, 240, 250] } else { [170, 180, 198] }, 0, [0; 3]));
            }
            None
        }),
        Kind::Skeleton => mk(PK::Weapon, 4, 20, (2.0, 16.5), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let fx = x as f32 - 1.5;
            if y >= 17 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, [70, 50, 36], 0, [0; 3])) } else { None };
            }
            if y == 16 {
                return Some((PM_METAL, [110, 96, 80], 0, [0; 3]));
            }
            if fx.abs() < 1.1 {
                let rust = h2(x, y, 99) < 0.4;
                return Some((PM_METAL, if rust { [134, 92, 60] } else { [150, 146, 140] }, 0, [0; 3]));
            }
            None
        }),
        Kind::Brute => mk(PK::Weapon, 8, 28, (4.0, 24.5), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let fx = x as f32 + 0.5 - 4.0;
            let r = if y < 12 { 3.8 } else { 1.6 };
            if fx.abs() > r {
                return None;
            }
            if y < 12 && (x + y * 3) % 5 == 0 {
                return Some((PM_METAL, [180, 180, 190], PM_WOOD, [100, 70, 44]));
            }
            Some((PM_WOOD, vary([106, 74, 46], x, y, 77, 0.2), 0, [0; 3]))
        }),
        _ => mk(PK::Weapon, 1, 1, (0.0, 0.0), T_ARMF as i32, (0.0, 0.0), |_, _| None),
        }
    };
    let parts = vec![torso, leg_b, leg_f, arm_b, head, arm_f, weapon];
    let n = parts.len();
    let mut b = Body {
        parts,
        xf: vec![Xf::default(); n],
        mirror: false,
        root: (0.0, -(ll as f32) + 1.5),
        dirty: false,
        fx: false,
        order: vec![T_ARMB, T_LEGB, T_TORSO, T_LEGF, T_HEAD, T_ARMF, T_WEAPON],
        leg_len: ll as f32,
        kind: orig,
    };
    if kind == Kind::Zombie {
        b.parts[T_WEAPON].on = false;
    }
    if orig == Kind::Crawler {
        b.parts[T_LEGF].on = false;
        b.parts[T_LEGB].on = false;
    }
    b.compute();
    b
}

/// Break every part into small chunks (used when a frozen enemy shatters).
pub fn shatter(d: &Detached, chunk: i32) -> Vec<Detached> {
    let p = &d.part;
    let mut out = vec![];
    let mut cy = 0;
    while cy < p.h {
        let mut cx = 0;
        while cx < p.w {
            let mut piece = p.clone();
            let mut n = 0;
            for y in 0..p.h {
                for x in 0..p.w {
                    let k = (y * p.w + x) as usize;
                    let inside = x >= cx && x < cx + chunk && y >= cy && y < cy + chunk;
                    if !inside {
                        piece.px[k].m = 0;
                    } else if piece.px[k].m != 0 {
                        n += 1;
                    }
                }
            }
            if n > 0 {
                let c = (cx as f32 + chunk as f32 / 2.0, cy as f32 + chunk as f32 / 2.0);
                let (ox, oy) = d.xf.fwd(p.pivot, c.0, c.1);
                piece.pivot = c;
                out.push(Detached { part: piece, xf: Xf::new(ox, oy, d.xf.a) });
            }
            cx += chunk;
        }
        cy += chunk;
    }
    out
}

/// the weapon in the player's hand, shaped by its type; rarer weapons have richer metal,
/// legendaries carry glowing runes along the blade
pub fn weapon_part(wk: crate::items::WKind, rarity: crate::items::Rarity, glow: bool, attach: (f32, f32)) -> Part {
    use crate::items::{Rarity, WKind};
    let steel: [u8; 3] = match rarity {
        Rarity::Common => [170, 176, 190],
        Rarity::Magic => [160, 180, 235],
        Rarity::Rare => [226, 206, 130],
        Rarity::Legendary => [90, 84, 96],
    };
    let rune: [u8; 3] = [255, 140, 50];
    let lit = |c: [u8; 3], left: bool| if left { mul(c, 1.25) } else { mul(c, 0.85) };
    let wood = [96u8, 62, 38];
    let par = T_ARMF as i32;
    match wk {
        WKind::Sword => mk(PK::Weapon, 5, 26, (2.5, 21.5), par, attach, move |x, y| {
            let fx = x as f32 - 2.0;
            if y >= 22 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, wood, 0, [0; 3])) } else { None };
            }
            if y == 20 || y == 21 {
                return Some((PM_METAL, [214, 180, 80], 0, [0; 3]));
            }
            if fx.abs() < 1.5 && !(y == 0 && fx.abs() > 0.5) {
                if glow && fx.abs() < 0.5 && y % 3 == 1 {
                    return Some((PM_GLOW, rune, 0, [0; 3]));
                }
                return Some((PM_METAL, lit(steel, fx < 0.0), 0, [0; 3]));
            }
            None
        }),
        WKind::Axe => mk(PK::Weapon, 9, 28, (2.0, 24.5), par, attach, move |x, y| {
            let haft = x == 1 || x == 2;
            let head = y < 10 && x >= 3 && {
                let dy = (y as f32 - 4.5).abs();
                (x as f32) < 4.0 + (5.0 - dy * 0.55) + if y < 2 || y > 7 { 0.0 } else { 0.8 }
            };
            if head {
                if glow && x == 4 && y % 2 == 0 {
                    return Some((PM_GLOW, rune, 0, [0; 3]));
                }
                let edge = x >= 7;
                return Some((PM_METAL, if edge { mul(steel, 1.3) } else { lit(steel, y < 5) }, 0, [0; 3]));
            }
            if haft && y >= 1 {
                return Some((PM_WOOD, if x == 1 { mul(wood, 1.15) } else { wood }, 0, [0; 3]));
            }
            None
        }),
        WKind::Hammer => mk(PK::Weapon, 8, 28, (4.0, 24.5), par, attach, move |x, y| {
            let head = y < 7 && x <= 7;
            if head {
                if glow && (x == 2 || x == 5) && y == 3 {
                    return Some((PM_GLOW, [140, 220, 255], 0, [0; 3]));
                }
                let face = x == 0 || x == 7;
                return Some((PM_METAL, if face { mul(steel, 0.75) } else { lit(steel, y < 3) }, 0, [0; 3]));
            }
            if (x == 3 || x == 4) && y >= 7 {
                return Some((PM_WOOD, if x == 3 { mul(wood, 1.15) } else { wood }, 0, [0; 3]));
            }
            None
        }),
        WKind::Spear => mk(PK::Weapon, 5, 44, (2.0, 32.5), par, attach, move |x, y| {
            let fx = x as f32 - 2.0;
            if y < 8 {
                let w = if y < 2 { 0.6 } else if y < 6 { 1.6 } else { 1.0 };
                if fx.abs() <= w {
                    if glow && fx.abs() < 0.5 && y > 1 {
                        return Some((PM_GLOW, rune, 0, [0; 3]));
                    }
                    return Some((PM_METAL, lit(steel, fx < 0.0), 0, [0; 3]));
                }
                return None;
            }
            if fx.abs() < 1.0 {
                return Some((PM_WOOD, if y % 9 == 0 { mul(wood, 0.7) } else { wood }, 0, [0; 3]));
            }
            None
        }),
        WKind::Dagger => mk(PK::Weapon, 5, 14, (2.5, 10.5), par, attach, move |x, y| {
            let fx = x as f32 - 2.0;
            if y >= 11 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, wood, 0, [0; 3])) } else { None };
            }
            if y == 9 || y == 10 {
                return Some((PM_METAL, [200, 170, 90], 0, [0; 3]));
            }
            let w = if y < 2 { 0.5 } else { 1.2 };
            if fx.abs() <= w {
                if glow && fx.abs() < 0.5 && y % 2 == 0 {
                    return Some((PM_GLOW, [140, 255, 100], 0, [0; 3]));
                }
                return Some((PM_METAL, lit(steel, fx < 0.0), 0, [0; 3]));
            }
            None
        }),
    }
}

/// a zombie swollen with bog water: a wide belly studded with glowing boils
fn bloater() -> Body {
    let mut b = humanoid(Kind::Zombie);
    let s = (next() & 0xffff) as u32;
    let t = b.parts[T_TORSO].clone();
    let nw = t.w * 3 / 2 + 1;
    let k = nw as f32 / t.w as f32;
    let mut px = vec![Px::default(); (nw * t.h) as usize];
    for y in 0..t.h {
        for x in 0..nw {
            let sx = ((x as f32 / k) as i32).min(t.w - 1);
            let mut p = t.px[(y * t.w + sx) as usize];
            if p.m != 0 {
                if p.m == PM_FLESH || p.m == PM_CLOTH {
                    p.m = PM_FLESH;
                    p.c = vary([92, 124, 70], x, y, s, 0.2);
                    p.im = PM_FLESH;
                    p.ic = [110, 150, 60];
                }
                if h2(x, y, s + 3) < 0.1 && y > 2 {
                    p.m = PM_GLOW;
                    p.c = [150, 235, 70];
                }
            }
            px[(y * nw + x) as usize] = p;
        }
    }
    b.parts[T_TORSO].w = nw;
    b.parts[T_TORSO].px = px;
    b.parts[T_TORSO].pivot.0 *= k;
    for p in b.parts.iter_mut() {
        if p.parent == T_TORSO as i32 {
            p.attach.0 *= k;
        }
        for q in p.px.iter_mut() {
            if q.m == PM_FLESH {
                q.c = [(q.c[0] as f32 * 0.85) as u8, (q.c[1] as f32 * 1.02).min(255.0) as u8, (q.c[2] as f32 * 0.8) as u8];
            }
        }
    }
    b.kind = Kind::Bloater;
    b.compute();
    b
}

/// an outlaw in dull mail and leather, with a sword, an axe or a crossbow
fn bandit(orig: Kind) -> Body {
    use crate::items::{Rarity, WKind};
    let mut b = humanoid(Kind::Player);
    let leather = [[112u8, 64, 40], [92, 72, 52], [124, 42, 36], [72, 78, 60]][ri(4)];
    for (pi, p) in b.parts.iter_mut().enumerate() {
        if pi == T_WEAPON {
            continue;
        }
        for q in p.px.iter_mut() {
            if q.m == 0 {
                continue;
            }
            if q.c[2] as i32 > q.c[0] as i32 + 30 {
                // the player's blue becomes worn leather
                let l = (q.c[0] as f32 + q.c[1] as f32 + q.c[2] as f32) / (52.0 + 74.0 + 160.0);
                q.c = mul(leather, l.clamp(0.6, 1.4));
                q.m = PM_CLOTH;
            } else if q.c == [200, 50, 50] {
                q.c = [44, 38, 36];
            } else if q.m == PM_METAL {
                // a dark hood, leather sleeves and breeches; only the chest keeps its mail
                let l = (q.c[0] as f32 + q.c[1] as f32 + q.c[2] as f32) / 600.0;
                q.c = match pi {
                    T_HEAD => mul([66, 52, 46], 0.7 + l * 0.6),
                    T_ARMF | T_ARMB => mul(leather, 0.55 + l * 0.5),
                    T_LEGF | T_LEGB => mul([74, 62, 54], 0.7 + l * 0.5),
                    _ => mul(q.c, 0.55),
                };
                if pi != T_TORSO {
                    q.m = PM_CLOTH;
                }
                if pi == T_HEAD && l < 0.2 {
                    // the visor slit becomes a face peering out of the hood
                    q.m = PM_FLESH;
                    q.c = [196, 156, 126];
                }
            }
        }
    }
    let at = b.parts[T_WEAPON].attach;
    b.parts[T_WEAPON] = if orig == Kind::Crossbow {
        let mut bow = humanoid(Kind::Archer).parts[T_WEAPON].clone();
        bow.attach = at;
        bow
    } else {
        weapon_part(if rnd() < 0.5 { WKind::Sword } else { WKind::Axe }, Rarity::Common, false, at)
    };
    b.kind = orig;
    b.compute();
    b
}

/// the bandit chief: a huge man in blackened plate and a red cloak, with a war axe
fn chief() -> Body {
    use crate::items::{Rarity, WKind};
    let mut b = humanoid(Kind::Brute);
    for (pi, p) in b.parts.iter_mut().enumerate() {
        if pi == T_WEAPON {
            continue;
        }
        for q in p.px.iter_mut() {
            match q.m {
                PM_CLOTH => q.c = mul([140, 30, 28], 0.8 + (q.c[0] as f32 / 255.0) * 0.4),
                PM_METAL => q.c = mul(q.c, 0.55),
                _ => {}
            }
        }
    }
    let at = b.parts[T_WEAPON].attach;
    b.parts[T_WEAPON] = weapon_part(WKind::Axe, Rarity::Rare, false, at);
    b.kind = Kind::Chief;
    b.compute();
    b
}
