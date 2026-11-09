// The simulated world: a top-down grid of ground + top materials.
// Liquids splash and settle into puddles, fire spreads through grass, oil, wood and flesh,
// acid eats whatever it touches. Only chunks with something happening are updated.
use crate::rng::*;

/// the largest map the engine holds; each area uses its own size inside it (aw x ah)
pub const W: usize = 2048;
pub const H: usize = 1536;

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
/// dungeon bedrock: very tall, dark, indestructible
pub const ROCK: u8 = 17;
/// swamp gas: invisible-ish, explodes when fire reaches it
pub const GAS: u8 = 18;

pub const GRASS: u8 = 0;
pub const DIRT: u8 = 1;
pub const FLOOR: u8 = 2;
pub const MOSS: u8 = 3;
pub const BURNT: u8 = 4;
pub const MUD: u8 = 5;
pub const STAIRS: u8 = 6;
pub const STAIRSUP: u8 = 7;
pub const PLANK: u8 = 8;
/// bog peat: once lit it smoulders for a long time and creeps outwards
pub const PEAT: u8 = 9;
/// a trodden road: lighter, with cobbles, so it reads at night
pub const ROAD: u8 = 10;

pub const AUX_TRUNK: u8 = 1;
pub const AUX_INDOOR: u8 = 2;
pub const AUX_DEAD: u8 = 4;
/// wall cells of a timber-framed house
pub const AUX_HOUSE: u8 = 8;
/// wall cells of a stone tomb or chapel
pub const AUX_TOMB: u8 = 16;
/// a cracked wall: a blade or a blast breaks it open
pub const AUX_CRACK: u8 = 32;

#[inline]
pub fn is_wall(m: u8) -> bool {
    matches!(m, STONE | BRICK | TREE | WOOD | ROCK)
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
        BRICK => 44,
        TREE => 84,
        WOOD => 8,
        ROCK => 44,
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
    Boom(i32, i32),
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
    /// this area's size
    pub aw: usize,
    pub ah: usize,
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
            aw: W,
            ah: H,
            tick: 0,
            zaps: vec![],
            events: vec![],
        }
    }
    /// a world for an area of aw x ah cells; everything outside is solid rock
    pub fn sized(aw: usize, ah: usize) -> World {
        let mut w = World::new();
        w.aw = aw.min(W);
        w.ah = ah.min(H);
        for y in 0..H {
            for x in 0..W {
                if x >= w.aw || y >= w.ah {
                    let i = y * W + x;
                    w.m[i] = ROCK;
                    w.g[i] = FLOOR;
                }
            }
        }
        w
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
        if !Self::inb(x, y) {
            return true;
        }
        let i = y as usize * W + x as usize;
        let m = self.m[i];
        is_wall(m) && (m != TREE || self.aux[i] & AUX_TRUNK != 0)
    }
    /// does this cell stop light and line of sight? (canopies don't, trunks do)
    pub fn blocks(&self, i: usize) -> bool {
        let m = self.m[i];
        is_wall(m) && m != WOOD && (m != TREE || self.aux[i] & AUX_TRUNK != 0)
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
    pub fn wake_all(&mut self) {
        let (cw, ch) = (self.aw.div_ceil(CS).min(CW), self.ah.div_ceil(CS).min(CH));
        for cy in 0..ch {
            for cx in 0..cw {
                self.act[cy * CW + cx] = 30;
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
            GAS => {
                // drifts a little
                if rnd() < 0.03 {
                    if let Some((j, nx, ny)) = self.nb(x, y, N4[ri(4)]) {
                        if self.m[j] == E {
                            self.m[j] = GAS;
                            self.m[i] = E;
                            self.stamp[j] = t;
                            self.keep(nx as usize, ny as usize);
                        }
                    }
                }
            }
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
                GAS => {
                    self.m[j] = FIRE;
                    self.life[j] = rr(6.0, 14.0) as u8;
                    self.stamp[j] = t;
                    self.wake(nx, ny);
                    if rnd() < 0.08 {
                        self.events.push(Ev::Boom(nx, ny));
                    }
                }
                E if self.g[j] == PEAT && rnd() < 0.02 => {
                    self.m[j] = FIRE;
                    self.life[j] = rr(150.0, 250.0) as u8;
                    self.stamp[j] = t;
                    self.wake(nx, ny);
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
        let peat = self.g[i] == PEAT;
        if peat && l <= 1 {
            self.g[i] = BURNT;
        }
        if l <= 1 {
            self.m[i] = if rnd() < 0.2 { ASH } else { E };
            self.life[i] = 0;
            self.scorch[i] = self.scorch[i].saturating_add(70);
            return;
        }
        // peat smoulders: it burns slowly
        if !(peat && rnd() < 0.8) {
            self.life[i] = l - 1;
        }
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
            if !Self::inb(x as i32, y as i32) || self.blocks(y as usize * W + x as usize) {
                return false;
            }
        }
        true
    }

    // ---------------- world generation ----------------
    pub fn blob(&mut self, cx: f32, cy: f32, r: f32, m: u8, seed_: u32, only_empty: bool) {
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
    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, m: u8) {
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
    pub fn ground_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, g: u8, aux: u8) {
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
}
