// Area generators: each area is built from its biome rules plus a few hand-placed landmarks
// (town buildings, the crypt entrance, boss rooms). Outdoor areas use the save's seed so
// they stay the same; dungeons get a fresh layout on every visit.
use crate::body::Kind;
use crate::rng::*;
use crate::world::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AreaId {
    Ashford,
    Pines,
    Graveyard,
    Crypt(u8),
}

pub const CRYPT_FLOORS: u8 = 3;

impl AreaId {
    pub fn name(&self) -> String {
        match self {
            AreaId::Ashford => "ASHFORD".into(),
            AreaId::Pines => "THE HOLLOW PINES".into(),
            AreaId::Graveyard => "THE OLD GRAVEYARD".into(),
            AreaId::Crypt(f) => format!("SUNKEN CRYPT - FLOOR {}", f),
        }
    }
    pub fn sub(&self) -> &'static str {
        match self {
            AreaId::Ashford => "THE LAST SAFE HEARTH",
            AreaId::Pines => "THE DEAD WALK BETWEEN THE TREES",
            AreaId::Graveyard => "WHERE THE DIGGING NEVER STOPS",
            AreaId::Crypt(f) if *f == CRYPT_FLOORS => "THE HOLLOW KING WAITS BELOW",
            AreaId::Crypt(_) => "THE DEAD DO NOT REST HERE",
        }
    }
    /// area level: what enemies scale to
    pub fn level(&self) -> i32 {
        match self {
            AreaId::Ashford => 1,
            AreaId::Pines => 2,
            AreaId::Graveyard => 5,
            AreaId::Crypt(f) => 2 + *f as i32,
        }
    }
    pub fn is_dungeon(&self) -> bool {
        matches!(self, AreaId::Crypt(_))
    }
    pub fn key(&self) -> String {
        match self {
            AreaId::Ashford => "ashford".into(),
            AreaId::Pines => "pines".into(),
            AreaId::Graveyard => "graveyard".into(),
            AreaId::Crypt(f) => format!("crypt{}", f),
        }
    }
    pub fn from_key(s: &str) -> Option<AreaId> {
        match s {
            "ashford" => Some(AreaId::Ashford),
            "pines" => Some(AreaId::Pines),
            "graveyard" => Some(AreaId::Graveyard),
            _ => s.strip_prefix("crypt").and_then(|f| f.parse().ok()).map(AreaId::Crypt),
        }
    }
    fn hash(&self) -> u64 {
        match self {
            AreaId::Ashford => 0x11,
            AreaId::Pines => 0x2222,
            AreaId::Graveyard => 0x333333,
            AreaId::Crypt(f) => 0x4444_4444 + *f as u64,
        }
    }
    /// areas with a waypoint stone, in menu order
    pub const WAYPOINTS: [AreaId; 3] = [AreaId::Ashford, AreaId::Pines, AreaId::Graveyard];
}

pub struct Exit {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub to: AreaId,
    pub label: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BKind {
    House,
    Tomb,
    /// roofless ruin
    Ruin,
    /// the mausoleum over the crypt stairs
    CryptGate,
}

#[derive(Clone, Copy)]
pub struct Building {
    pub x0: i32,
    pub y0: i32,
    pub w: i32,
    pub h: i32,
    pub door: i32,
    pub k: BKind,
    /// 0..1, picks roof colour, windows, chimney side
    pub look: f32,
}
impl Building {
    pub fn inside(&self, x: f32, y: f32) -> bool {
        x >= self.x0 as f32 && x < (self.x0 + self.w) as f32 && y >= self.y0 as f32 && y < (self.y0 + self.h) as f32
    }
    pub fn door_c(&self) -> i32 {
        self.x0 + self.w / 2
    }
    pub fn pitch(&self) -> f32 {
        match self.k {
            BKind::House => 0.72,
            BKind::CryptGate => 0.6,
            _ => 0.5,
        }
    }
    /// height of the roof ridge above the wall tops
    pub fn roof_r(&self) -> f32 {
        self.h as f32 * self.pitch()
    }
    /// ridge height above the wall tops at column x (hipped ends slope down)
    pub fn roof_z(&self, x: i32) -> f32 {
        let ov = 3;
        let r = self.roof_r();
        if self.k == BKind::CryptGate {
            return r;
        }
        let hd = self.h as f32 / 2.0 + ov as f32;
        let d = ((x - (self.x0 - ov)).min(self.x0 + self.w + ov - 1 - x)) as f32 + 0.5;
        r.min(r / hd * d)
    }
    /// chimney position on the roof (houses only)
    pub fn chimney(&self) -> Option<(i32, i32)> {
        if self.k != BKind::House {
            return None;
        }
        let x = if self.look > 0.5 { self.x0 + self.w * 3 / 4 } else { self.x0 + self.w / 5 };
        Some((x, self.y0 + self.h / 2))
    }
}

pub struct Spawns {
    pub buildings: Vec<Building>,
    pub lamps: Vec<(f32, f32, [f32; 3])>,
    pub start: (f32, f32),
    pub entries: Vec<(AreaId, f32, f32)>,
    pub enemies: Vec<(Kind, f32, f32)>,
    pub braziers: Vec<(f32, f32)>,
    pub exits: Vec<Exit>,
    pub waypoint: Option<(f32, f32)>,
}
impl Spawns {
    fn new() -> Spawns {
        Spawns { buildings: vec![], lamps: vec![], start: (100.0, 100.0), entries: vec![], enemies: vec![], braziers: vec![], exits: vec![], waypoint: None }
    }
    pub fn arrival(&self, from: AreaId) -> (f32, f32) {
        self.entries.iter().find(|e| e.0 == from).map(|e| (e.1, e.2)).unwrap_or(self.start)
    }
}

pub fn generate(id: AreaId, seed_: u64) -> (World, Spawns) {
    let s = seed_ ^ id.hash().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    seed(s);
    let mut w = World::new();
    let sp = match id {
        AreaId::Ashford => town(&mut w, s as u32),
        AreaId::Pines => forest(&mut w, s as u32),
        AreaId::Graveyard => graveyard(&mut w, s as u32),
        AreaId::Crypt(f) => dungeon(&mut w, f),
    };
    w.wake_all();
    (w, sp)
}

// ------------------------------------------------------------------ helpers
fn path(w: &mut World, ctrl: &[(f32, f32)], width: f32, seed_: u32) -> Vec<(f32, f32)> {
    let mut pts = vec![];
    for k in 0..ctrl.len() - 1 {
        let (a, b) = (ctrl[k], ctrl[k + 1]);
        let n = ((b.0 - a.0).hypot(b.1 - a.1) / 2.0) as i32;
        for j in 0..n.max(1) {
            let t = j as f32 / n as f32;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let wob = (vnoise(x * 0.03, y * 0.03, seed_ + 3) - 0.5) * 8.0;
            pts.push((x + wob * 0.3, y + wob));
        }
    }
    for &(x, y) in &pts {
        let r = width + vnoise(x * 0.1, y * 0.1, seed_ + 5) * 3.0;
        let ri_ = r.ceil() as i32 + 1;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d < r {
                    let (px, py) = (x as i32 + dx, y as i32 + dy);
                    if World::inb(px, py) {
                        let i = py as usize * W + px as usize;
                        w.g[i] = if d > r - 1.5 && rnd() < 0.5 { MUD } else { DIRT };
                        if w.m[i] == TREE || w.m[i] == STONE {
                            w.m[i] = E;
                        }
                    }
                }
            }
        }
    }
    pts
}
fn near(pts: &[(f32, f32)], x: f32, y: f32, r: f32) -> bool {
    pts.iter().any(|&(a, b)| (a - x).powi(2) + (b - y).powi(2) < r * r)
}
fn ground_noise(w: &mut World, s: u32, moss: f32, dirt: f32) {
    for y in 0..H {
        for x in 0..W {
            let n = fbm(x as f32 * 0.018, y as f32 * 0.018, s);
            w.g[y * W + x] = if n > moss { MOSS } else if n < dirt { DIRT } else { GRASS };
        }
    }
}
fn border(w: &mut World) {
    w.rect(0, 0, W as i32, 6, STONE);
    w.rect(0, H as i32 - 6, W as i32, H as i32, STONE);
    w.rect(0, 0, 6, H as i32, STONE);
    w.rect(W as i32 - 6, 0, W as i32, H as i32, STONE);
}
/// plant trees where `ok` allows; `dead` trees are bare and grey
fn plant_trees(w: &mut World, max: i32, tries: i32, rmin: f32, rmax: f32, s: u32, dead: bool, ok: &dyn Fn(f32, f32, f32) -> bool) {
    let mut placed = 0;
    for _ in 0..tries {
        if placed >= max {
            break;
        }
        let (x, y) = (rr(12.0, W as f32 - 12.0), rr(12.0, H as f32 - 12.0));
        let r = rr(rmin, rmax);
        if !ok(x, y, r) {
            continue;
        }
        let mut clear = true;
        'c: for dy in -(r as i32) - 2..=(r as i32) + 2 {
            for dx in -(r as i32) - 2..=(r as i32) + 2 {
                let m = w.at(x as i32 + dx, y as i32 + dy);
                if m != E || w.g[((y as i32 + dy).clamp(0, H as i32 - 1)) as usize * W + (x as i32 + dx).clamp(0, W as i32 - 1) as usize] == FLOOR {
                    clear = false;
                    break 'c;
                }
            }
        }
        if !clear {
            continue;
        }
        w.blob(x, y, r, TREE, s + 30 + placed as u32, true);
        for dy in -(r as i32) - 6..=(r as i32) + 6 {
            for dx in -(r as i32) - 6..=(r as i32) + 6 {
                let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                if w.at(xx, yy) == TREE {
                    let i = yy as usize * W + xx as usize;
                    if w.col[i] == [0, 0, 0] {
                        w.col[i] = [(dx + 128) as u8, (dy + 128) as u8, r as u8];
                    }
                    if dx * dx + dy * dy <= 5 {
                        w.aux[i] |= AUX_TRUNK;
                    }
                    if dead {
                        w.aux[i] |= AUX_DEAD;
                    }
                }
            }
        }
        placed += 1;
    }
}
fn rocks(w: &mut World, n: i32, s: u32, ok: &dyn Fn(f32, f32) -> bool) {
    for _ in 0..n {
        let (x, y) = (rr(20.0, W as f32 - 20.0), rr(20.0, H as f32 - 20.0));
        if ok(x, y) {
            w.blob(x, y, rr(2.5, 6.0), STONE, s + 21, true);
        }
    }
}
/// a small stone building with a doorway on its south side
/// a building with walls, a south doorway and (unless a ruin) a roof drawn over it
fn building(w: &mut World, sp: &mut Spawns, x0: i32, y0: i32, bw: i32, bh: i32, door: i32, k: BKind) -> Building {
    let floor = if k == BKind::House { PLANK } else { FLOOR };
    w.ground_rect(x0, y0, x0 + bw, y0 + bh, floor, AUX_INDOOR);
    let wall = BRICK;
    let t = 4;
    w.rect(x0, y0, x0 + bw, y0 + t, wall);
    w.rect(x0, y0, x0 + t, y0 + bh, wall);
    w.rect(x0 + bw - t, y0, x0 + bw, y0 + bh, wall);
    let dc = x0 + bw / 2;
    w.rect(x0, y0 + bh - t, dc - door / 2, y0 + bh, wall);
    w.rect(dc + door / 2, y0 + bh - t, x0 + bw, y0 + bh, wall);
    let bit = if k == BKind::House { AUX_HOUSE } else { AUX_TOMB };
    for y in y0..y0 + bh {
        for x in x0..x0 + bw {
            if w.at(x, y) == BRICK {
                w.aux[y as usize * W + x as usize] |= bit;
            }
        }
    }
    let b = Building { x0, y0, w: bw, h: bh, door, k, look: rnd() };
    sp.buildings.push(b);
    if k == BKind::House {
        sp.lamps.push(((x0 + bw / 2) as f32, (y0 + bh / 2) as f32, [1.0, 0.62, 0.3]));
    }
    b
}
fn pack(enemies: &mut Vec<(Kind, f32, f32)>, w: &World, x: f32, y: f32, n: usize, mix: &[(Kind, f32)]) {
    let tot: f32 = mix.iter().map(|m| m.1).sum();
    let mut placed = 0;
    for _ in 0..n * 6 {
        if placed >= n {
            break;
        }
        let (ex, ey) = (x + rr(-22.0, 22.0), y + rr(-14.0, 14.0));
        if w.at(ex as i32, ey as i32) != E || w.solid(ex as i32 + 3, ey as i32) || w.solid(ex as i32 - 3, ey as i32) {
            continue;
        }
        let mut r = rnd() * tot;
        let mut k = mix[0].0;
        for &(kk, p) in mix {
            r -= p;
            if r <= 0.0 {
                k = kk;
                break;
            }
        }
        enemies.push((k, ex, ey));
        placed += 1;
    }
}
fn waypoint(w: &mut World, sp: &mut Spawns, x: f32, y: f32) {
    for dy in -12i32..=12 {
        for dx in -16i32..=16 {
            let (xx, yy) = (x as i32 + dx, y as i32 + dy);
            if World::inb(xx, yy) && ((dx * dx) as f32 / 256.0 + (dy * dy) as f32 / 144.0) < 1.0 {
                let i = yy as usize * W + xx as usize;
                w.m[i] = E;
                w.g[i] = DIRT;
            }
        }
    }
    w.rect(x as i32 - 1, y as i32 - 1, x as i32 + 2, y as i32 + 1, STONE);
    sp.waypoint = Some((x, y));
}

// ------------------------------------------------------------------ Ashford
fn town(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    ground_noise(w, s, 0.7, 0.28);
    let (tx0, ty0, tx1, ty1) = (300, 230, 720, 550);
    let road_y = 390.0;
    let road = path(w, &[(470.0, road_y), (620.0, road_y + 4.0), (760.0, road_y - 10.0), (900.0, road_y + 12.0), (W as f32 - 4.0, road_y)], 7.0, s);
    let cross = path(w, &[(510.0, 250.0), (512.0, 540.0)], 6.0, s + 1);
    // palisade with an east gate
    for x in tx0..tx1 + 3 {
        for &y in &[ty0, ty1] {
            w.rect(x, y, x + 1, y + 3, WOOD);
        }
    }
    for y in ty0..=ty1 + 2 {
        w.rect(tx0, y, tx0 + 3, y + 1, WOOD);
        if (y as f32 - road_y).abs() > 12.0 {
            w.rect(tx1, y, tx1 + 3, y + 1, WOOD);
        }
    }
    // houses
    let houses = [(318, 252, 84, 56), (414, 246, 74, 54), (552, 250, 80, 56), (644, 258, 66, 52), (322, 448, 80, 58), (570, 452, 84, 58), (428, 466, 68, 52)];
    for &(x, y, bw, bh) in &houses {
        building(w, &mut sp, x, y, bw, bh, 18, BKind::House);
        for _ in 0..2 {
            let (cx, cy) = (x + 8 + ri((bw - 18) as usize) as i32, y + 8 + ri((bh - 20) as usize) as i32);
            w.rect(cx, cy, cx + 6, cy + 5, WOOD);
        }
    }
    // the square: campfire, well, waypoint
    let (sx, sy) = (512.0, 395.0);
    for dy in -40i32..=40 {
        for dx in -55i32..=55 {
            let (x, y) = (sx as i32 + dx, sy as i32 + dy);
            if (dx * dx) as f32 / 3025.0 + (dy * dy) as f32 / 1600.0 < 1.0 {
                w.g[y as usize * W + x as usize] = DIRT;
            }
        }
    }
    sp.braziers.push((sx, sy - 6.0));
    for dy in -6i32..=6 {
        for dx in -6i32..=6 {
            let d = dx * dx + dy * dy;
            if d <= 36 && d > 12 {
                w.set(470 + dx, 415 + dy, STONE);
            } else if d <= 12 {
                w.set(470 + dx, 415 + dy, WATER);
            }
        }
    }
    waypoint(w, &mut sp, 552.0, 418.0);
    for &(x, y) in &[(712.0, 372.0), (712.0, 408.0), (400.0, 330.0), (620.0, 340.0), (400.0, 430.0), (620.0, 436.0)] {
        sp.braziers.push((x, y));
    }
    // the dark forest all around
    let inside = |x: f32, y: f32, r: f32| x > tx0 as f32 - r - 8.0 && x < tx1 as f32 + r + 8.0 && y > ty0 as f32 - r - 8.0 && y < ty1 as f32 + r + 8.0;
    plant_trees(w, 170, 6000, 10.0, 16.0, s, false, &|x, y, r| !inside(x, y, r) && !near(&road, x, y, r * 0.6 + 14.0) && !near(&cross, x, y, r));
    border(w);
    sp.exits.push(Exit { x0: W as i32 - 28, y0: road_y as i32 - 26, x1: W as i32 - 6, y1: road_y as i32 + 26, to: AreaId::Pines, label: "THE HOLLOW PINES".into() });
    sp.start = (sx + 20.0, sy + 16.0);
    sp.entries.push((AreaId::Pines, W as f32 - 44.0, road_y));
    sp
}

// ------------------------------------------------------------------ Hollow Pines
fn forest(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    ground_noise(w, s, 0.64, 0.3);
    let yw = rr(300.0, 460.0);
    let ye = rr(300.0, 460.0);
    let mid: Vec<(f32, f32)> = (1..6).map(|k| (k as f32 * W as f32 / 6.0, rr(250.0, 520.0))).collect();
    let mut ctrl = vec![(4.0, yw)];
    ctrl.extend(mid.iter().copied());
    ctrl.push((W as f32 - 4.0, ye));
    let road = path(w, &ctrl, 7.0, s);
    // the crypt mausoleum, off the road to the north
    let (mx, my) = (rr(620.0, 760.0), 150.0);
    let (bx, by, bw, bh) = (mx as i32 - 38, my as i32 - 30, 76, 56);
    let side = path(w, &[(mx, my + 30.0), (mx + 10.0, my + 86.0), (road[road.len() * 2 / 3].0, road[road.len() * 2 / 3].1)], 5.0, s + 2);
    building(w, &mut sp, bx, by, bw, bh, 22, BKind::CryptGate);
    for y in by + 14..by + 30 {
        for x in mx as i32 - 11..mx as i32 + 11 {
            w.g[y as usize * W + x as usize] = STAIRS;
        }
    }
    sp.lamps.push((mx, (by + 22) as f32, [0.45, 0.75, 1.0]));
    sp.exits.push(Exit { x0: mx as i32 - 12, y0: by + 12, x1: mx as i32 + 12, y1: by + 32, to: AreaId::Crypt(1), label: "THE SUNKEN CRYPT".into() });
    sp.entries.push((AreaId::Crypt(1), mx, my + 40.0));
    sp.braziers.push((mx - 24.0, my + 36.0));
    sp.braziers.push((mx + 24.0, my + 36.0));
    // a pond somewhere away from the road
    for _ in 0..30 {
        let (px, py) = (rr(150.0, 900.0), rr(120.0, 650.0));
        if near(&road, px, py, 80.0) || (px - mx).hypot(py - my) < 120.0 {
            continue;
        }
        for dy in -60i32..=60 {
            for dx in -70i32..=70 {
                let (x, y) = (px as i32 + dx, py as i32 + dy);
                if !World::inb(x, y) {
                    continue;
                }
                let d = ((dx * dx) as f32 / 1.3 + (dy * dy) as f32).sqrt();
                let r = 38.0 * (0.8 + 0.4 * vnoise(x as f32 * 0.05, y as f32 * 0.05, s + 9));
                let i = y as usize * W + x as usize;
                if d < r {
                    w.m[i] = WATER;
                } else if d < r + 6.0 {
                    w.g[i] = MUD;
                }
            }
        }
        break;
    }
    waypoint(w, &mut sp, 90.0, yw + 34.0);
    let keep = |x: f32, y: f32| (x < 160.0 && (y - yw).abs() < 90.0) || (x - mx).hypot(y - my) < 80.0;
    rocks(w, 60, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y));
    plant_trees(w, 135, 6000, 10.0, 16.0, s, false, &|x, y, r| {
        let edge = x < 70.0 || y < 70.0 || x > W as f32 - 70.0 || y > H as f32 - 70.0;
        let dens = fbm(x * 0.01, y * 0.01, s + 40) * 1.2 + if edge { 0.5 } else { 0.0 };
        rnd() < dens && !near(&road, x, y, r * 0.6 + 12.0) && !near(&side, x, y, r * 0.5 + 8.0) && !keep(x, y)
    });
    border(w);
    sp.exits.push(Exit { x0: 6, y0: yw as i32 - 26, x1: 28, y1: yw as i32 + 26, to: AreaId::Ashford, label: "ASHFORD".into() });
    sp.exits.push(Exit { x0: W as i32 - 28, y0: ye as i32 - 26, x1: W as i32 - 6, y1: ye as i32 + 26, to: AreaId::Graveyard, label: "THE OLD GRAVEYARD".into() });
    sp.entries.push((AreaId::Ashford, 44.0, yw));
    sp.entries.push((AreaId::Graveyard, W as f32 - 44.0, ye));
    sp.start = (90.0, yw);
    // the dead: packs along the road and in the woods
    let mix = [(Kind::Zombie, 0.7), (Kind::Crawler, 0.2), (Kind::Skeleton, 0.1)];
    let mut packs = 0;
    for _ in 0..200 {
        if packs >= 10 {
            break;
        }
        let (x, y) = if rnd() < 0.6 { road[ri(road.len())] } else { (rr(60.0, W as f32 - 60.0), rr(60.0, H as f32 - 60.0)) };
        if x < 220.0 || w.at(x as i32, y as i32) != E || sp.enemies.iter().any(|e| (e.1 - x).hypot(e.2 - y) < 90.0) {
            continue;
        }
        pack(&mut sp.enemies, w, x, y, 3 + ri(3), &mix);
        packs += 1;
    }
    sp.enemies.push((Kind::Brute, mx + 30.0, my + 60.0));
    sp
}

// ------------------------------------------------------------------ Old Graveyard
fn graveyard(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    ground_noise(w, s, 0.6, 0.42);
    let yw = rr(330.0, 430.0);
    let (cx, cy) = (rr(640.0, 760.0), rr(300.0, 440.0));
    let road = path(w, &[(4.0, yw), (200.0, yw + rr(-40.0, 40.0)), (420.0, cy + rr(-60.0, 60.0)), (cx - 70.0, cy + 20.0)], 6.0, s);
    // ruined chapel
    let (x0, y0, cw, ch) = (cx as i32 - 70, cy as i32 - 55, 140, 100);
    building(w, &mut sp, x0, y0, cw, ch, 22, BKind::Ruin);
    w.rect(x0, y0 + 30, x0 + 5, y0 + 50, E);
    for _ in 0..90 {
        let (x, y) = (x0 + ri(cw as usize) as i32, y0 + ri(ch as usize) as i32);
        if w.at(x, y) == BRICK {
            w.set(x, y, RUBBLE);
        }
    }
    for k in 0..3 {
        w.rect(x0 + 30 + k * 32, y0 + 30, x0 + 37 + k * 32, y0 + 37, BRICK);
    }
    sp.braziers.push((cx - 40.0, cy - 30.0));
    sp.braziers.push((cx + 40.0, cy - 30.0));
    // plots of graves with low fences
    for _ in 0..14 {
        let (px, py) = (rr(160.0, 900.0), rr(80.0, 640.0));
        let (pw, ph) = (rr(70.0, 130.0) as i32, rr(50.0, 90.0) as i32);
        if (px - cx).abs() < 140.0 && (py - cy).abs() < 110.0 {
            continue;
        }
        let (gx0, gy0) = (px as i32, py as i32);
        for x in gx0..gx0 + pw {
            if x % 6 != 0 {
                w.set(x, gy0, WOOD);
                w.set(x, gy0 + ph, WOOD);
            }
        }
        for row in 0..(ph / 18) {
            for col in 0..(pw / 14) {
                let (x, y) = (gx0 + 6 + col * 14 + ri(3) as i32, gy0 + 8 + row * 18 + ri(3) as i32);
                if rnd() < 0.8 && !near(&road, x as f32, y as f32, 10.0) {
                    w.rect(x, y, x + 5, y + 2, STONE);
                    // open graves, some with bones
                    if rnd() < 0.25 {
                        for dy in 3..9 {
                            for dx in 0..5 {
                                let i = (y + dy) as usize * W + (x + dx) as usize;
                                w.g[i] = MUD;
                                if rnd() < 0.08 {
                                    w.m[i] = BONE;
                                    w.col[i] = [200, 192, 170];
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // mausoleums
    for _ in 0..5 {
        let (x, y) = (rr(150.0, 900.0) as i32, rr(80.0, 620.0) as i32);
        if (x as f32 - cx).abs() < 160.0 && (y as f32 - cy).abs() < 130.0 || near(&road, x as f32 + 29.0, y as f32 + 23.0, 46.0) || sp.buildings.iter().any(|b| (b.x0 - x).abs() < 90 && (b.y0 - y).abs() < 80) {
            continue;
        }
        building(w, &mut sp, x, y, 58, 46, 16, BKind::Tomb);
        sp.enemies.push((Kind::Skeleton, x as f32 + 29.0, y as f32 + 24.0));
    }
    waypoint(w, &mut sp, 80.0, yw + 32.0);
    let keep = |x: f32, y: f32| x < 150.0 && (y - yw).abs() < 90.0;
    plant_trees(w, 70, 4000, 8.0, 13.0, s, true, &|x, y, r| !near(&road, x, y, r * 0.5 + 10.0) && !keep(x, y) && !((x - cx).abs() < 100.0 + r && (y - cy).abs() < 80.0 + r));
    rocks(w, 30, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y));
    border(w);
    sp.exits.push(Exit { x0: 6, y0: yw as i32 - 26, x1: 28, y1: yw as i32 + 26, to: AreaId::Pines, label: "THE HOLLOW PINES".into() });
    sp.entries.push((AreaId::Pines, 44.0, yw));
    sp.start = (60.0, yw);
    let mix = [(Kind::Skeleton, 0.4), (Kind::Archer, 0.25), (Kind::Zombie, 0.25), (Kind::Crawler, 0.1)];
    let mut packs = 0;
    for _ in 0..300 {
        if packs >= 12 {
            break;
        }
        let (x, y) = (rr(200.0, W as f32 - 60.0), rr(60.0, H as f32 - 60.0));
        if w.at(x as i32, y as i32) != E || sp.enemies.iter().any(|e| (e.1 - x).hypot(e.2 - y) < 80.0) {
            continue;
        }
        pack(&mut sp.enemies, w, x, y, 3 + ri(3), &mix);
        packs += 1;
    }
    sp.enemies.push((Kind::Brute, cx, cy + 10.0));
    sp.enemies.push((Kind::Brute, rr(300.0, 600.0), rr(150.0, 600.0)));
    sp
}

// ------------------------------------------------------------------ dungeons
struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}
impl Room {
    fn c(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

fn dungeon(w: &mut World, floor: u8) -> Spawns {
    let mut sp = Spawns::new();
    for i in 0..W * H {
        w.m[i] = ROCK;
        w.g[i] = FLOOR;
        w.aux[i] = AUX_INDOOR;
    }
    let boss = floor == CRYPT_FLOORS;
    let mut rooms: Vec<Room> = vec![];
    let target = 9 + ri(3);
    // boss arena first, at one end
    if boss {
        rooms.push(Room { x: rr(700.0, 820.0) as i32, y: rr(200.0, 360.0) as i32, w: 170, h: 130 });
    }
    for _ in 0..600 {
        if rooms.len() >= target {
            break;
        }
        let (rw, rh) = (rr(56.0, 120.0) as i32, rr(44.0, 86.0) as i32);
        let (x, y) = (rr(30.0, (W as i32 - rw - 30) as f32) as i32, rr(30.0, (H as i32 - rh - 30) as f32) as i32);
        if rooms.iter().any(|r| x < r.x + r.w + 26 && x + rw + 26 > r.x && y < r.y + r.h + 26 && y + rh + 26 > r.y) {
            continue;
        }
        rooms.push(Room { x, y, w: rw, h: rh });
    }
    // order rooms into a winding chain from the far side to the boss / stairs down
    if boss {
        let b = rooms.remove(0);
        rooms.sort_by_key(|r| r.c().0);
        rooms.push(b);
    } else {
        rooms.sort_by_key(|r| r.c().0 + r.c().1 / 3);
    }
    let carve = |w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32| {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                if World::inb(x, y) {
                    w.m[y as usize * W + x as usize] = E;
                }
            }
        }
    };
    for r in &rooms {
        carve(w, r.x, r.y, r.x + r.w, r.y + r.h);
    }
    let corridor = |w: &mut World, a: (i32, i32), b: (i32, i32)| {
        let hw = 7;
        if rnd() < 0.5 {
            carve(w, a.0, a.1 - hw, b.0, a.1 + hw);
            carve(w, b.0 - hw, a.1, b.0 + hw, b.1);
        } else {
            carve(w, a.0 - hw, a.1, a.0 + hw, b.1);
            carve(w, a.0, b.1 - hw, b.0, b.1 + hw);
        }
    };
    for k in 0..rooms.len() - 1 {
        corridor(w, rooms[k].c(), rooms[k + 1].c());
    }
    for _ in 0..2 {
        let (a, b) = (ri(rooms.len() - 1), ri(rooms.len() - 1));
        if a != b {
            corridor(w, rooms[a].c(), rooms[b].c());
        }
    }
    // line the carved spaces with brick
    let snapshot = w.m.clone();
    for y in 4..H - 4 {
        for x in 4..W - 4 {
            let i = y * W + x;
            if snapshot[i] != ROCK {
                continue;
            }
            let mut near_open = false;
            'n: for dy in -4i32..=4 {
                for dx in -4i32..=4 {
                    if snapshot[(y as i32 + dy) as usize * W + (x as i32 + dx) as usize] == E {
                        near_open = true;
                        break 'n;
                    }
                }
            }
            if near_open {
                w.m[i] = BRICK;
            }
        }
    }
    // dress the rooms
    let n = rooms.len();
    for (k, r) in rooms.iter().enumerate() {
        let (cx, cy) = r.c();
        if k > 0 && k < n - 1 || (!boss && k == n - 1) {
            if r.w > 80 && r.h > 60 && rnd() < 0.6 {
                for &(px, py) in &[(r.x + 18, r.y + 16), (r.x + r.w - 26, r.y + 16), (r.x + 18, r.y + r.h - 22), (r.x + r.w - 26, r.y + r.h - 22)] {
                    w.rect(px, py, px + 8, py + 8, BRICK);
                }
            }
            if rnd() < 0.35 {
                for c in 0..3 {
                    let x = r.x + 10 + c * 18;
                    if x + 8 < r.x + r.w - 6 {
                        w.rect(x, r.y + 8, x + 8, r.y + 24, WOOD);
                    }
                }
            }
            if rnd() < 0.3 {
                w.splash(cx as f32 + rr(-20.0, 20.0), cy as f32 + rr(-10.0, 10.0), OIL, rr(5.0, 9.0), 0);
            }
            if rnd() < 0.18 {
                let (ax, ay) = (cx + rr(-15.0, 15.0) as i32, cy + rr(-8.0, 8.0) as i32);
                for dy in -9i32..=9 {
                    for dx in -13i32..=13 {
                        if (dx * dx) as f32 / 169.0 + (dy * dy) as f32 / 81.0 < 1.0 && w.at(ax + dx, ay + dy) == E {
                            w.set(ax + dx, ay + dy, ACID);
                        }
                    }
                }
            }
            for _ in 0..(r.w * r.h / 400) {
                let (bx, by) = (r.x + ri(r.w as usize) as i32, r.y + ri(r.h as usize) as i32);
                if w.at(bx, by) == E {
                    let i = by as usize * W + bx as usize;
                    if rnd() < 0.4 {
                        w.m[i] = BONE;
                        w.col[i] = [196, 188, 166];
                    } else {
                        w.stain[i] = w.stain[i].saturating_add(90);
                    }
                }
            }
        }
        sp.braziers.push((r.x as f32 + 10.0, r.y as f32 + 12.0));
        if r.w > 70 {
            sp.braziers.push(((r.x + r.w) as f32 - 10.0, r.y as f32 + 12.0));
        }
        // enemies
        if k > 0 && !(boss && k == n - 1) {
            let mix = [(Kind::Skeleton, 0.45), (Kind::Archer, 0.2), (Kind::Zombie, 0.2), (Kind::Crawler, 0.15)];
            let cnt = 2 + ri(3) + if floor > 1 { 1 } else { 0 };
            pack(&mut sp.enemies, w, cx as f32, cy as f32, cnt, &mix);
            if rnd() < 0.2 + floor as f32 * 0.08 {
                sp.enemies.push((Kind::Brute, cx as f32 + rr(-10.0, 10.0), cy as f32 + rr(-6.0, 6.0)));
            }
        }
    }
    // stairs up in the first room, down (or the boss) in the last
    let (ux, uy) = rooms[0].c();
    for y in uy - 20..uy - 8 {
        for x in ux - 9..ux + 9 {
            w.g[y as usize * W + x as usize] = STAIRSUP;
        }
    }
    let up = if floor == 1 { AreaId::Pines } else { AreaId::Crypt(floor - 1) };
    sp.exits.push(Exit { x0: ux - 10, y0: uy - 22, x1: ux + 10, y1: uy - 6, to: up, label: if floor == 1 { "THE HOLLOW PINES".into() } else { format!("FLOOR {}", floor - 1) } });
    sp.entries.push((up, ux as f32, uy as f32 + 6.0));
    sp.start = (ux as f32, uy as f32 + 6.0);
    let (dx, dy) = rooms[n - 1].c();
    if boss {
        sp.enemies.push((Kind::King, dx as f32, dy as f32 - 10.0));
        for k in 0..4 {
            sp.braziers.push((dx as f32 + [-70.0, 70.0, -70.0, 70.0][k], dy as f32 + [-50.0, -50.0, 50.0, 50.0][k]));
        }
    } else {
        for y in dy - 6..dy + 6 {
            for x in dx - 9..dx + 9 {
                w.g[y as usize * W + x as usize] = STAIRS;
            }
        }
        let down = AreaId::Crypt(floor + 1);
        sp.exits.push(Exit { x0: dx - 10, y0: dy - 8, x1: dx + 10, y1: dy + 8, to: down, label: format!("FLOOR {}", floor + 1) });
        sp.entries.push((down, dx as f32, dy as f32 + 18.0));
    }
    sp
}
