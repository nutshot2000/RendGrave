// The simulated world: a top-down grid of ground + top materials.
// Liquids splash and settle into puddles, fire spreads through grass, oil, wood and flesh,
// acid eats whatever it touches. Only chunks with something happening are updated.
use crate::rng::*;

pub const W: usize = 1024;
pub const H: usize = 768;

pub const E: u8 = 0;
pub const STONE: u8 = 1;
pub const BRICK: u8 = 2;
pub const TREE: u8 = 3;
pub const WOOD: u8 = 4;
pub const WATER: u8 = 5;
pub const ACID: u8 = 6;
pub const BLOOD: u8 = 7;
pub const OIL: u8 = 8;
pub const FIRE: u8 = 9;
pub const ICE: u8 = 10;
pub const FLESH: u8 = 11;
pub const BONE: u8 = 12;
pub const RUBBLE: u8 = 13;
pub const METAL: u8 = 14;
pub const CLOTH: u8 = 15;
pub const ASH: u8 = 16;

pub const GRASS: u8 = 0;
pub const DIRT: u8 = 1;
pub const FLOOR: u8 = 2;
pub const MOSS: u8 = 3;
pub const BURNT: u8 = 4;
pub const MUD: u8 = 5;

pub const AUX_TRUNK: u8 = 1;
pub const AUX_INDOOR: u8 = 2;

#[inline]
pub fn is_wall(m: u8) -> bool {
    matches!(m, STONE | BRICK | TREE | WOOD)
}
#[inline]
pub fn is_liquid(m: u8) -> bool {
    matches!(m, WATER | ACID | BLOOD | OIL)
}
#[inline]
pub fn conductive(m: u8) -> bool {
    matches!(m, WATER | ACID | BLOOD | METAL)
}
#[inline]
pub fn wall_h(m: u8) -> i32 {
    match m {
        STONE => 14,
        BRICK => 30,
        TREE => 27,
        WOOD => 8,
        _ => 0,
    }
}
#[inline]
pub fn flammable(m: u8) -> f32 {
    match m {
        OIL => 0.5,
        WOOD => 0.05,
        TREE => 0.02,
        FLESH => 0.02,
        CLOTH => 0.08,
        _ => 0.0,
    }
}

pub const CS: usize = 32;
pub const CW: usize = W / CS;
pub const CH: usize = H / CS;
const N4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const N8: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];

pub enum Ev {
    Steam(i32, i32),
    Fume(i32, i32),
}

pub struct World {
    pub m: Vec<u8>,
    pub life: Vec<u8>,
    pub shade: Vec<u8>,
    pub g: Vec<u8>,
    pub stain: Vec<u8>,
    pub scorch: Vec<u8>,
    pub aux: Vec<u8>,
    pub col: Vec<[u8; 3]>,
    act: Vec<u8>,
    stamp: Vec<u8>,
    pub tick: u8,
    pub zaps: Vec<(u32, u8)>,
    pub events: Vec<Ev>,
}

pub struct Spawns {
    pub player: (f32, f32),
    pub enemies: Vec<(u8, f32, f32)>,
    pub braziers: Vec<(f32, f32)>,
    pub crypt: (i32, i32, i32, i32),
}

impl World {
    pub fn new() -> World {
        let n = W * H;
        World {
            m: vec![E; n],
            life: vec![0; n],
            shade: (0..n).map(|_| (next() & 255) as u8).collect(),
            g: vec![GRASS; n],
            stain: vec![0; n],
            scorch: vec![0; n],
            aux: vec![0; n],
            col: vec![[0; 3]; n],
            act: vec![0; CW * CH],
            stamp: vec![0; n],
            tick: 0,
            zaps: vec![],
            events: vec![],
        }
    }
    #[inline]
    pub fn inb(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H
    }
    #[inline]
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if Self::inb(x, y) { self.m[y as usize * W + x as usize] } else { STONE }
    }
    #[inline]
    pub fn solid(&self, x: i32, y: i32) -> bool {
        is_wall(self.at(x, y))
    }
    pub fn wake(&mut self, x: i32, y: i32) {
        let cx = (x.max(0) as usize / CS).min(CW - 1);
        let cy = (y.max(0) as usize / CS).min(CH - 1);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (a, b) = (cx as i32 + dx, cy as i32 + dy);
                if a >= 0 && b >= 0 && (a as usize) < CW && (b as usize) < CH {
                    self.act[b as usize * CW + a as usize] = 60;
                }
            }
        }
    }
    #[inline]
    fn keep(&mut self, x: usize, y: usize) {
        let c = (y / CS) * CW + x / CS;
        if self.act[c] < 3 {
            self.act[c] = 3;
        }
    }
    pub fn set(&mut self, x: i32, y: i32, m: u8) {
        if !Self::inb(x, y) {
            return;
        }
        let i = y as usize * W + x as usize;
        self.m[i] = m;
        self.life[i] = 0;
        self.wake(x, y);
    }
    pub fn set_life(&mut self, x: i32, y: i32, m: u8, l: u8) {
        if !Self::inb(x, y) {
            return;
        }
        let i = y as usize * W + x as usize;
        self.m[i] = m;
        self.life[i] = l;
        self.wake(x, y);
    }
    /// Light something at a cell: grass catches, oil flares, wood and flesh start burning.
    pub fn ignite(&mut self, x: i32, y: i32) -> bool {
        if !Self::inb(x, y) {
            return false;
        }
        let i = y as usize * W + x as usize;
        let m = self.m[i];
        if m == E {
            let g = self.g[i];
            if g == GRASS || g == MOSS || rnd() < 0.3 {
                self.m[i] = FIRE;
                self.life[i] = rr(16.0, 40.0) as u8;
                if g == GRASS || g == MOSS {
                    self.g[i] = BURNT;
                }
                self.wake(x, y);
                return true;
            }
            return false;
        }
        self.ignite_cell(i, x, y)
    }
    fn ignite_cell(&mut self, i: usize, x: i32, y: i32) -> bool {
        let m = self.m[i];
        match m {
            OIL => {
                self.m[i] = FIRE;
                self.life[i] = rr(40.0, 90.0) as u8;
            }
            TREE | WOOD | FLESH | CLOTH if self.life[i] == 0 => {
                self.life[i] = match m {
                    TREE => rr(180.0, 250.0) as u8,
                    WOOD => rr(140.0, 220.0) as u8,
                    FLESH => rr(90.0, 160.0) as u8,
                    _ => rr(40.0, 80.0) as u8,
                };
            }
            ICE => {
                self.m[i] = WATER;
            }
            _ => return false,
        }
        self.wake(x, y);
        true
    }
    /// Throw a liquid down: it fills empty cells and keeps spreading for `energy` moves.
    pub fn splash(&mut self, cx: f32, cy: f32, m: u8, r: f32, energy: u8) {
        let ri_ = r.ceil() as i32;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                if (dx * dx + dy * dy) as f32 > r * r {
                    continue;
                }
                let (x, y) = (cx as i32 + dx, cy as i32 + dy);
                if !Self::inb(x, y) {
                    continue;
                }
                let i = y as usize * W + x as usize;
                let t = self.m[i];
                if (t == E || (is_liquid(t) && t != m) || t == ASH) && rnd() < 0.85 {
                    if t == FIRE && m == WATER {
                        continue;
                    }
                    self.m[i] = m;
                    self.life[i] = energy;
                    self.wake(x, y);
                } else if t == FIRE && m == WATER {
                    self.m[i] = E;
                }
            }
        }
    }
    /// Electric current spreads through connected puddles and metal.
    pub fn conduct(&mut self, x: i32, y: i32, max: usize) -> Vec<usize> {
        let mut out = vec![];
        if !conductive(self.at(x, y)) {
            return out;
        }
        let mut seen = std::collections::HashSet::new();
        let mut q = std::collections::VecDeque::new();
        let s = y as usize * W + x as usize;
        q.push_back(s);
        seen.insert(s);
        while let Some(i) = q.pop_front() {
            out.push(i);
            if out.len() >= max {
                break;
            }
            let (cx, cy) = ((i % W) as i32, (i / W) as i32);
            for (dx, dy) in N8 {
                let (nx, ny) = (cx + dx, cy + dy);
                if !Self::inb(nx, ny) {
                    continue;
                }
                let j = ny as usize * W + nx as usize;
                if !seen.contains(&j) && conductive(self.m[j]) {
                    seen.insert(j);
                    q.push_back(j);
                }
            }
        }
        for &i in &out {
            self.zaps.push((i as u32, 14 + (next() % 10) as u8));
        }
        out
    }

    pub fn update(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let t = self.tick;
        let flip = t & 1 == 1;
        for k in (0..self.zaps.len()).rev() {
            if self.zaps[k].1 <= 1 {
                self.zaps.swap_remove(k);
            } else {
                self.zaps[k].1 -= 1;
            }
        }
        for cy in (0..CH).rev() {
            for cxn in 0..CW {
                let cx = if flip { cxn } else { CW - 1 - cxn };
                let ci = cy * CW + cx;
                if self.act[ci] == 0 {
                    continue;
                }
                self.act[ci] -= 1;
                for y in cy * CS..(cy + 1) * CS {
                    for xn in 0..CS {
                        let x = if flip { cx * CS + xn } else { cx * CS + CS - 1 - xn };
                        let i = y * W + x;
                        if self.stamp[i] == t {
                            continue;
                        }
                        let m = self.m[i];
                        if m == E {
                            continue;
                        }
                        self.cell(x, y, i, m, t);
                    }
                }
            }
        }
    }

    fn cell(&mut self, x: usize, y: usize, i: usize, m: u8, t: u8) {
        match m {
            WATER | BLOOD | OIL | ACID => self.liquid(x, y, i, m, t),
            FIRE => self.fire(x, y, i, t),
            ICE => {
                if rnd() < 0.00015 {
                    self.m[i] = WATER;
                }
                self.keep(x, y);
            }
            TREE | WOOD | FLESH | CLOTH => {
                if self.life[i] > 0 {
                    self.burning(x, y, i, m, t)
                }
            }
            _ => {}
        }
    }

    fn nb(&self, x: usize, y: usize, d: (i32, i32)) -> Option<(usize, i32, i32)> {
        let (nx, ny) = (x as i32 + d.0, y as i32 + d.1);
        if Self::inb(nx, ny) { Some((ny as usize * W + nx as usize, nx, ny)) } else { None }
    }

    fn liquid(&mut self, x: usize, y: usize, i: usize, m: u8, t: u8) {
        if m == ACID {
            if let Some((j, nx, ny)) = self.nb(x, y, N8[ri(8)]) {
                let tm = self.m[j];
                let eat = match tm {
                    FLESH | CLOTH => 0.08,
                    BONE => 0.03,
                    WOOD | TREE => 0.02,
                    RUBBLE | ASH => 0.05,
                    BRICK | STONE => 0.003,
                    METAL => 0.004,
                    _ => 0.0,
                };
                if eat > 0.0 && rnd() < eat {
                    self.m[j] = E;
                    self.life[j] = 0;
                    self.wake(nx, ny);
                    if rnd() < 0.3 {
                        self.events.push(Ev::Fume(nx, ny));
                    }
                    if rnd() < 0.35 {
                        self.m[i] = E;
                        self.life[i] = 0;
                        return;
                    }
                }
            }
            if (self.g[i] == GRASS || self.g[i] == MOSS) && rnd() < 0.01 {
                self.g[i] = DIRT;
                self.scorch[i] = self.scorch[i].saturating_add(30);
            }
            if rnd() < 0.0005 {
                self.m[i] = E;
                self.events.push(Ev::Fume(x as i32, y as i32));
                return;
            }
            self.keep(x, y);
        }
        let l = self.life[i];
        if l > 0 {
            if let Some((j, nx, ny)) = self.nb(x, y, N4[ri(4)]) {
                if self.m[j] == E {
                    self.m[j] = m;
                    self.life[j] = l - 1;
                    self.m[i] = E;
                    self.life[i] = 0;
                    self.stamp[j] = t;
                    self.keep(nx as usize, ny as usize);
                    return;
                }
            }
            self.life[i] = l - 1;
            self.keep(x, y);
        }
    }

    fn spread_fire(&mut self, x: usize, y: usize, t: u8, chance_floor: f32) -> bool {
        if let Some((j, nx, ny)) = self.nb(x, y, N8[ri(8)]) {
            let tm = self.m[j];
            match tm {
                WATER => return true,
                ICE => {
                    if rnd() < 0.1 {
                        self.m[j] = WATER;
                    }
                }
                E => {
                    let gg = self.g[j];
                    if (gg == GRASS || gg == MOSS) && rnd() < 0.035 {
                        self.m[j] = FIRE;
                        self.life[j] = rr(18.0, 40.0) as u8;
                        self.g[j] = BURNT;
                        self.stamp[j] = t;
                        self.wake(nx, ny);
                    } else if rnd() < chance_floor {
                        self.m[j] = FIRE;
                        self.life[j] = rr(5.0, 12.0) as u8;
                        self.stamp[j] = t;
                    }
                }
                _ => {
                    let f = flammable(tm);
                    if f > 0.0 && rnd() < f {
                        self.ignite_cell(j, nx, ny);
                    }
                }
            }
        }
        false
    }

    fn fire(&mut self, x: usize, y: usize, i: usize, t: u8) {
        let l = self.life[i];
        if l <= 1 {
            self.m[i] = if rnd() < 0.2 { ASH } else { E };
            self.life[i] = 0;
            self.scorch[i] = self.scorch[i].saturating_add(70);
            return;
        }
        self.life[i] = l - 1;
        if (self.g[i] == GRASS || self.g[i] == MOSS) && rnd() < 0.1 {
            self.g[i] = BURNT;
        }
        for _ in 0..2 {
            if self.spread_fire(x, y, t, 0.0) {
                self.m[i] = E;
                self.life[i] = 0;
                self.events.push(Ev::Steam(x as i32, y as i32));
                return;
            }
        }
        self.keep(x, y);
    }

    fn burning(&mut self, x: usize, y: usize, i: usize, m: u8, t: u8) {
        if !(m == TREE && rnd() < 0.5) {
            self.life[i] -= 1;
        }
        if self.life[i] == 0 {
            self.scorch[i] = self.scorch[i].saturating_add(90);
            match m {
                TREE | WOOD => {
                    self.m[i] = RUBBLE;
                    let s = 22 + (self.shade[i] % 14);
                    self.col[i] = [s + 6, s, s - 4];
                }
                FLESH => {
                    self.m[i] = if rnd() < 0.5 { BONE } else { ASH };
                    self.col[i] = [70, 56, 46];
                }
                _ => self.m[i] = ASH,
            }
            self.wake(x as i32, y as i32);
            return;
        }
        if self.spread_fire(x, y, t, 0.03) {
            self.life[i] = 0;
        }
        self.keep(x, y);
    }

    pub fn los(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
        let d = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let n = (d / 2.0) as i32;
        for k in 1..n {
            let t = k as f32 / n as f32;
            let (x, y) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
            let m = self.at(x as i32, y as i32);
            if is_wall(m) && m != WOOD {
                return false;
            }
        }
        true
    }

    // ---------------- world generation ----------------
    fn blob(&mut self, cx: f32, cy: f32, r: f32, m: u8, seed_: u32, only_empty: bool) {
        let ri_ = (r * 1.4).ceil() as i32;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                let (x, y) = (cx as i32 + dx, cy as i32 + dy);
                if !Self::inb(x, y) {
                    continue;
                }
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                let nr = r * (0.8 + 0.4 * vnoise(x as f32 * 0.15, y as f32 * 0.15, seed_));
                if d < nr {
                    let i = y as usize * W + x as usize;
                    if only_empty && self.m[i] != E {
                        continue;
                    }
                    self.m[i] = m;
                    self.life[i] = 0;
                }
            }
        }
    }
    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, m: u8) {
        for y in y0..y1 {
            for x in x0..x1 {
                if Self::inb(x, y) {
                    let i = y as usize * W + x as usize;
                    self.m[i] = m;
                    self.life[i] = 0;
                }
            }
        }
    }
    fn ground_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, g: u8, aux: u8) {
        for y in y0..y1 {
            for x in x0..x1 {
                if Self::inb(x, y) {
                    let i = y as usize * W + x as usize;
                    self.g[i] = g;
                    self.aux[i] |= aux;
                }
            }
        }
    }

    pub fn generate(seed_: u64) -> (World, Spawns) {
        seed(seed_);
        let s = (seed_ & 0xffff) as u32;
        let mut w = World::new();
        // ground: grass with moss and dirt patches
        for y in 0..H {
            for x in 0..W {
                let n = fbm(x as f32 * 0.018, y as f32 * 0.018, s);
                let i = y * W + x;
                w.g[i] = if n > 0.64 { MOSS } else if n < 0.3 { DIRT } else { GRASS };
            }
        }
        // a road from the camp to the crypt door
        let road = [(90.0, 680.0), (220.0, 640.0), (360.0, 600.0), (480.0, 520.0), (620.0, 470.0), (756.0, 440.0), (756.0, 400.0)];
        let mut road_pts = vec![];
        for k in 0..road.len() - 1 {
            let (a, b) = (road[k], road[k + 1]);
            let n = (((b.0 - a.0) as f32).hypot(b.1 - a.1) / 2.0) as i32;
            for j in 0..n {
                let t = j as f32 / n as f32;
                let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                let wob = (vnoise(x * 0.03, y * 0.03, s + 3) - 0.5) * 10.0;
                road_pts.push((x + wob * 0.3, y + wob));
            }
        }
        for &(x, y) in &road_pts {
            let r = 6.0 + vnoise(x * 0.1, y * 0.1, s + 5) * 3.0;
            for dy in -10..=10 {
                for dx in -10..=10 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d < r {
                        let (px, py) = (x as i32 + dx, y as i32 + dy);
                        if Self::inb(px, py) {
                            let i = py as usize * W + px as usize;
                            w.g[i] = if d > r - 1.5 && rnd() < 0.5 { MUD } else { DIRT };
                        }
                    }
                }
            }
        }
        let near_road = |x: f32, y: f32, r: f32| road_pts.iter().any(|&(a, b)| (a - x).powi(2) + (b - y).powi(2) < r * r);
        // the pond
        let pond = (300.0f32, 330.0f32);
        for dy in -70i32..=70 {
            for dx in -80i32..=80 {
                let (x, y) = (pond.0 as i32 + dx, pond.1 as i32 + dy);
                let d = ((dx * dx) as f32 / 1.3 + (dy * dy) as f32).sqrt();
                let r = 46.0 * (0.8 + 0.4 * vnoise(x as f32 * 0.05, y as f32 * 0.05, s + 9));
                let i = y as usize * W + x as usize;
                if d < r {
                    w.m[i] = WATER;
                } else if d < r + 6.0 {
                    w.g[i] = MUD;
                }
            }
        }
        // the crypt
        let (x0, y0, x1, y1) = (600, 110, 910, 405);
        w.ground_rect(x0, y0, x1, y1, FLOOR, AUX_INDOOR);
        for (a, b) in [(x0, x1)] {
            w.rect(a, y0, b, y0 + 7, BRICK);
            w.rect(a, y1 - 7, 740, y1, BRICK);
            w.rect(772, y1 - 7, b, y1, BRICK);
        }
        w.rect(x0, y0, x0 + 7, y1, BRICK);
        w.rect(x1 - 7, y0, x1, y1, BRICK);
        w.rect(706, y0, 713, 196, BRICK);
        w.rect(706, 230, 713, y1, BRICK);
        w.rect(x0, 262, 640, 268, BRICK);
        w.rect(668, 262, 706, 268, BRICK);
        for &px in &[770, 850] {
            for &py in &[165, 245, 325] {
                w.rect(px, py, px + 9, py + 9, BRICK);
            }
        }
        // coffins and spilled oil in the north room
        for k in 0..4 {
            w.rect(622 + k * 21, 140, 630 + k * 21, 158, WOOD);
            w.rect(622 + k * 21, 200, 630 + k * 21, 218, WOOD);
        }
        w.splash(650.0, 240.0, OIL, 9.0, 0);
        w.splash(684.0, 176.0, OIL, 6.0, 0);
        // acid pool in the south room
        for dy in -16i32..=16 {
            for dx in -22i32..=22 {
                let d = ((dx * dx) as f32 / 1.8 + (dy * dy) as f32).sqrt();
                if d < 13.0 + vnoise((650 + dx) as f32 * 0.2, (335 + dy) as f32 * 0.2, s) * 4.0 {
                    let i = (335 + dy) as usize * W + (650 + dx) as usize;
                    w.m[i] = ACID;
                }
            }
        }
        // bones and old blood in the hall
        for _ in 0..40 {
            let (bx, by) = (rr(720.0, 900.0) as i32, rr(120.0, 395.0) as i32);
            if w.at(bx, by) == E {
                let c = rr(170.0, 210.0) as u8;
                for k in 0..(2 + ri(4)) as i32 {
                    let (xx, yy) = (bx + k, by + (k % 2));
                    if w.at(xx, yy) == E {
                        let i = yy as usize * W + xx as usize;
                        w.m[i] = BONE;
                        w.col[i] = [c, c - 8, c - 26];
                    }
                }
            }
        }
        for _ in 0..30 {
            let (bx, by) = (rr(615.0, 900.0), rr(120.0, 395.0));
            for _ in 0..40 {
                let (x, y) = ((bx + rr(-8.0, 8.0)) as i32, (by + rr(-5.0, 5.0)) as i32);
                if Self::inb(x, y) {
                    let i = y as usize * W + x as usize;
                    w.stain[i] = w.stain[i].saturating_add(70);
                }
            }
        }
        let braziers = vec![(728.0, 132.0), (892.0, 132.0), (728.0, 385.0), (892.0, 385.0), (732.0, 418.0), (780.0, 418.0), (625.0, 290.0)];
        for &(bx, by) in &braziers {
            w.rect(bx as i32 - 2, by as i32 - 1, bx as i32 + 3, by as i32 + 2, METAL);
        }
        // graveyard
        for gy in 0..4 {
            for gx in 0..5 {
                let (x, y) = (470 + gx * 22 + ri(5) as i32, 300 + gy * 26 + ri(5) as i32);
                if !near_road(x as f32, y as f32, 16.0) {
                    w.rect(x, y, x + 5, y + 2, STONE);
                    for _ in 0..12 {
                        let (a, b) = (x + ri(7) as i32 - 1, y + 3 + ri(10) as i32);
                        if Self::inb(a, b) {
                            w.g[b as usize * W + a as usize] = DIRT;
                        }
                    }
                }
            }
        }
        // rocks
        for _ in 0..70 {
            let (x, y) = (rr(20.0, W as f32 - 20.0), rr(20.0, H as f32 - 20.0));
            if near_road(x, y, 16.0) || (x > 580.0 && x < 930.0 && y > 90.0 && y < 425.0) || (x - 110.0).hypot(y - 670.0) < 70.0 {
                continue;
            }
            w.blob(x, y, rr(2.5, 6.0), STONE, s + 21, true);
        }
        // forest
        let mut placed = 0;
        for _ in 0..4000 {
            if placed > 300 {
                break;
            }
            let (x, y) = (rr(12.0, W as f32 - 12.0), rr(12.0, H as f32 - 12.0));
            let r = rr(5.5, 10.0);
            let dens = fbm(x * 0.01, y * 0.01, s + 40);
            if rnd() > dens * 1.4 {
                continue;
            }
            if near_road(x, y, r + 22.0)
                || (x > 575.0 && x < 935.0 && y > 85.0 && y < 440.0)
                || (x - pond.0).hypot(y - pond.1) < 66.0
                || (x - 110.0).hypot(y - 670.0) < 80.0
                || (x > 455.0 && x < 590.0 && y > 285.0 && y < 420.0)
            {
                continue;
            }
            let mut clear = true;
            'c: for dy in -(r as i32) - 2..=(r as i32) + 2 {
                for dx in -(r as i32) - 2..=(r as i32) + 2 {
                    if w.at(x as i32 + dx, y as i32 + dy) != E {
                        clear = false;
                        break 'c;
                    }
                }
            }
            if !clear {
                continue;
            }
            w.blob(x, y, r, TREE, s + 30 + placed, true);
            for dy in 0..(r as i32 + 3) {
                for dx in -1..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if w.at(xx, yy) == TREE {
                        w.aux[yy as usize * W + xx as usize] |= AUX_TRUNK;
                    }
                }
            }
            placed += 1;
        }
        // map border
        w.rect(0, 0, W as i32, 6, STONE);
        w.rect(0, H as i32 - 6, W as i32, H as i32, STONE);
        w.rect(0, 0, 6, H as i32, STONE);
        w.rect(W as i32 - 6, 0, W as i32, H as i32, STONE);
        // enemies: zombies along the road, skeletons in the crypt, brutes guarding it
        let mut enemies = vec![];
        for &(gx, gy) in &[(260.0, 600.0), (420.0, 520.0), (560.0, 500.0), (380.0, 640.0)] {
            for k in 0..3 {
                enemies.push((0u8, gx + (k as f32 - 1.0) * 14.0 + rr(-4.0, 4.0), gy + rr(-12.0, 12.0)));
            }
        }
        for &(x, y) in &[(760.0, 200.0), (820.0, 300.0), (880.0, 220.0), (810.0, 140.0), (740.0, 350.0), (650.0, 180.0), (680.0, 230.0), (640.0, 320.0)] {
            enemies.push((1, x, y));
        }
        enemies.push((2, 790.0, 448.0));
        enemies.push((2, 820.0, 260.0));
        for &(_, x, y) in &enemies {
            for dy in -8i32..=8 {
                for dx in -8i32..=8 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if matches!(w.at(xx, yy), TREE | STONE) {
                        w.set(xx, yy, E);
                    }
                }
            }
        }
        for c in w.act.iter_mut() {
            *c = 30;
        }
        (w, Spawns { player: (110.0, 672.0), enemies, braziers, crypt: (x0, y0, x1, y1) })
    }
}
