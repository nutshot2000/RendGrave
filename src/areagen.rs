// Area generators: each area is built from its biome rules plus a few hand-placed landmarks
// (town buildings, the crypt entrance, the bandit castle, boss rooms). Outdoor areas use the
// save's seed so they stay the same; dungeons get a fresh layout on every visit.
// Every area has its own size inside the engine's largest map (W x H).
use crate::body::Kind;
use crate::rng::*;
use crate::world::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AreaId {
    Ashford,
    Pines,
    Graveyard,
    Fen,
    Ravenhold,
    Crypt(u8),
    Cellars(u8),
}

pub const CRYPT_FLOORS: u8 = 3;
pub const CELLAR_FLOORS: u8 = 2;

impl AreaId {
    pub fn name(&self) -> String {
        match self {
            AreaId::Ashford => "ASHFORD".into(),
            AreaId::Pines => "THE HOLLOW PINES".into(),
            AreaId::Graveyard => "THE OLD GRAVEYARD".into(),
            AreaId::Fen => "THE BLACKWATER FEN".into(),
            AreaId::Ravenhold => "RAVENHOLD".into(),
            AreaId::Crypt(f) => format!("SUNKEN CRYPT - FLOOR {}", f),
            AreaId::Cellars(f) => format!("RAVENHOLD CELLARS - DEPTH {}", f),
        }
    }
    pub fn sub(&self) -> &'static str {
        match self {
            AreaId::Ashford => "THE LAST SAFE HEARTH",
            AreaId::Pines => "THE DEAD WALK BETWEEN THE TREES",
            AreaId::Graveyard => "WHERE THE DIGGING NEVER STOPS",
            AreaId::Fen => "THE BOG REMEMBERS EVERY BODY",
            AreaId::Ravenhold => "BANDITS HOLD THE BROKEN KEEP",
            AreaId::Crypt(f) if *f == CRYPT_FLOORS => "THE HOLLOW KING WAITS BELOW",
            AreaId::Crypt(_) => "THE DEAD DO NOT REST HERE",
            AreaId::Cellars(f) if *f == CELLAR_FLOORS => "THE CHIEF COUNTS HIS PLUNDER",
            AreaId::Cellars(_) => "STOLEN GOLD AND COLD STEEL",
        }
    }
    /// area level: what enemies scale to
    pub fn level(&self) -> i32 {
        match self {
            AreaId::Ashford => 1,
            AreaId::Pines => 3,
            AreaId::Graveyard => 8,
            AreaId::Fen => 10,
            AreaId::Ravenhold => 12,
            AreaId::Crypt(f) => 4 + *f as i32,
            AreaId::Cellars(f) => 12 + *f as i32,
        }
    }
    pub fn is_dungeon(&self) -> bool {
        matches!(self, AreaId::Crypt(_) | AreaId::Cellars(_))
    }
    /// where you come back up to when you load a save taken underground
    pub fn surface(&self) -> AreaId {
        match self {
            AreaId::Crypt(_) => AreaId::Pines,
            AreaId::Cellars(_) => AreaId::Ravenhold,
            a => *a,
        }
    }
    /// the area's size in cells
    pub fn size(&self) -> (usize, usize) {
        match self {
            AreaId::Ashford => (1024, 768),
            AreaId::Pines => (2048, 1536),
            AreaId::Graveyard => (1792, 1280),
            AreaId::Fen => (2048, 1536),
            AreaId::Ravenhold => (1536, 1152),
            AreaId::Crypt(_) | AreaId::Cellars(_) => (1600, 1200),
        }
    }
    pub fn key(&self) -> String {
        match self {
            AreaId::Ashford => "ashford".into(),
            AreaId::Pines => "pines".into(),
            AreaId::Graveyard => "graveyard".into(),
            AreaId::Fen => "fen".into(),
            AreaId::Ravenhold => "ravenhold".into(),
            AreaId::Crypt(f) => format!("crypt{}", f),
            AreaId::Cellars(f) => format!("cellars{}", f),
        }
    }
    pub fn from_key(s: &str) -> Option<AreaId> {
        match s {
            "ashford" => Some(AreaId::Ashford),
            "pines" => Some(AreaId::Pines),
            "graveyard" => Some(AreaId::Graveyard),
            "fen" => Some(AreaId::Fen),
            "ravenhold" => Some(AreaId::Ravenhold),
            _ => {
                if let Some(f) = s.strip_prefix("cellars") {
                    f.parse().ok().map(AreaId::Cellars)
                } else {
                    s.strip_prefix("crypt").and_then(|f| f.parse().ok()).map(AreaId::Crypt)
                }
            }
        }
    }
    fn hash(&self) -> u64 {
        match self {
            AreaId::Ashford => 0x11,
            AreaId::Pines => 0x2222,
            AreaId::Graveyard => 0x333333,
            AreaId::Fen => 0x5555_5555,
            AreaId::Ravenhold => 0x6666_6666_66,
            AreaId::Crypt(f) => 0x4444_4444 + *f as u64,
            AreaId::Cellars(f) => 0x7777_0000_00 + *f as u64,
        }
    }
    /// areas with a waypoint stone, in menu order
    pub const WAYPOINTS: [AreaId; 5] = [AreaId::Ashford, AreaId::Pines, AreaId::Graveyard, AreaId::Fen, AreaId::Ravenhold];
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
    /// (x, y, tier): tier 1 chests are iron-bound and richer
    pub chests: Vec<(f32, f32, u8)>,
    /// (x, y, kind): see props::SHRINES
    pub shrines: Vec<(f32, f32, u8)>,
    /// (x, y, kind): 0 powder keg, 1 oil, 2 acid
    pub barrels: Vec<(f32, f32, u8)>,
    /// the merchant's stall
    pub merchant: Option<(f32, f32)>,
    /// where Elder Maren stands
    pub maren: Option<(f32, f32)>,
    /// lantern posts along the roads and at the exits
    pub posts: Vec<(f32, f32)>,
}
impl Spawns {
    fn new() -> Spawns {
        Spawns { buildings: vec![], lamps: vec![], start: (100.0, 100.0), entries: vec![], enemies: vec![], braziers: vec![], exits: vec![], waypoint: None, chests: vec![], shrines: vec![], barrels: vec![], merchant: None, maren: None, posts: vec![] }
    }
    pub fn arrival(&self, from: AreaId) -> (f32, f32) {
        self.entries.iter().find(|e| e.0 == from).map(|e| (e.1, e.2)).unwrap_or(self.start)
    }
}

pub fn generate(id: AreaId, seed_: u64) -> (World, Spawns) {
    let s = seed_ ^ id.hash().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    seed(s);
    let (aw, ah) = id.size();
    let mut w = World::sized(aw, ah);
    let sp = match id {
        AreaId::Ashford => town(&mut w, s as u32),
        AreaId::Pines => forest(&mut w, s as u32),
        AreaId::Graveyard => graveyard(&mut w, s as u32),
        AreaId::Fen => fen(&mut w, s as u32),
        AreaId::Ravenhold => ravenhold(&mut w, s as u32),
        AreaId::Crypt(f) => dungeon(&mut w, f, false),
        AreaId::Cellars(f) => dungeon(&mut w, f, true),
    };
    w.wake_all();
    (w, sp)
}

// ------------------------------------------------------------------ helpers
fn idx(x: i32, y: i32) -> usize {
    y as usize * W + x as usize
}
fn inarea(w: &World, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && (x as usize) < w.aw && (y as usize) < w.ah
}
/// a winding road; it clears trees and rocks, and becomes a plank bridge over water
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
                    if inarea(w, px, py) {
                        let i = idx(px, py);
                        if w.m[i] == WATER || w.g[i] == PLANK {
                            w.m[i] = E;
                            w.g[i] = PLANK;
                            continue;
                        }
                        w.g[i] = if d > r - 1.5 { if rnd() < 0.5 { MUD } else { DIRT } } else { ROAD };
                        if matches!(w.m[i], TREE | STONE | GAS) {
                            w.m[i] = E;
                        }
                    }
                }
            }
        }
    }
    pts
}
/// a stream of water with muddy banks
fn river(w: &mut World, ctrl: &[(f32, f32)], width: f32, seed_: u32) -> Vec<(f32, f32)> {
    let mut pts = vec![];
    for k in 0..ctrl.len() - 1 {
        let (a, b) = (ctrl[k], ctrl[k + 1]);
        let n = ((b.0 - a.0).hypot(b.1 - a.1) / 2.0) as i32;
        for j in 0..n.max(1) {
            let t = j as f32 / n as f32;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let wob = (vnoise(x * 0.012, y * 0.012, seed_ + 7) - 0.5) * 60.0;
            pts.push((x + wob, y));
        }
    }
    for &(x, y) in &pts {
        let r = width * (0.8 + 0.5 * vnoise(x * 0.04, y * 0.04, seed_ + 8));
        let ri_ = r.ceil() as i32 + 5;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                let (px, py) = (x as i32 + dx, y as i32 + dy);
                if !inarea(w, px, py) {
                    continue;
                }
                let i = idx(px, py);
                if d < r {
                    w.m[i] = WATER;
                } else if d < r + 4.5 && w.m[i] != WATER {
                    w.g[i] = MUD;
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
    for y in 0..w.ah {
        for x in 0..w.aw {
            let n = fbm(x as f32 * 0.018, y as f32 * 0.018, s);
            w.g[y * W + x] = if n > moss { MOSS } else if n < dirt { DIRT } else { GRASS };
        }
    }
}
fn border(w: &mut World) {
    let (aw, ah) = (w.aw as i32, w.ah as i32);
    w.rect(0, 0, aw, 6, STONE);
    w.rect(0, ah - 6, aw, ah, STONE);
    w.rect(0, 0, 6, ah, STONE);
    w.rect(aw - 6, 0, aw, ah, STONE);
}
/// plant trees where `ok` allows; `dead` trees are bare and grey
fn plant_trees(w: &mut World, max: i32, tries: i32, rmin: f32, rmax: f32, s: u32, dead: bool, ok: &dyn Fn(f32, f32, f32) -> bool) {
    let mut placed = 0;
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    for _ in 0..tries {
        if placed >= max {
            break;
        }
        let (x, y) = (rr(12.0, aw - 12.0), rr(12.0, ah - 12.0));
        let r = rr(rmin, rmax);
        if !ok(x, y, r) {
            continue;
        }
        let mut clear = true;
        'c: for dy in -(r as i32) - 2..=(r as i32) + 2 {
            for dx in -(r as i32) - 2..=(r as i32) + 2 {
                let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                if !inarea(w, xx, yy) {
                    clear = false;
                    break 'c;
                }
                let i = idx(xx, yy);
                if w.m[i] != E || matches!(w.g[i], FLOOR | PLANK) {
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
                    let i = idx(xx, yy);
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
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    for _ in 0..n {
        let (x, y) = (rr(20.0, aw - 20.0), rr(20.0, ah - 20.0));
        if ok(x, y) && w.at(x as i32, y as i32) == E {
            w.blob(x, y, rr(2.5, 6.0), STONE, s + 21, true);
        }
    }
}
/// a building with walls, a south doorway and (unless a ruin) a roof drawn over it
fn building(w: &mut World, sp: &mut Spawns, x0: i32, y0: i32, bw: i32, bh: i32, door: i32, k: BKind) -> Building {
    let floor = if k == BKind::House { PLANK } else { FLOOR };
    // clear the plot first
    for y in y0 - 2..y0 + bh + 2 {
        for x in x0 - 2..x0 + bw + 2 {
            if inarea(w, x, y) && matches!(w.m[idx(x, y)], TREE | STONE | WATER | GAS) {
                w.m[idx(x, y)] = E;
            }
        }
    }
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
                w.aux[idx(x, y)] |= bit;
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
/// knock holes in a ruin's walls and leave rubble about
fn crumble(w: &mut World, b: &Building, n: i32) {
    for _ in 0..n {
        let (x, y) = (b.x0 + ri(b.w as usize) as i32, b.y0 + ri(b.h as usize) as i32);
        if w.at(x, y) == BRICK {
            w.set(x, y, RUBBLE);
        }
    }
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
/// scatter packs over the open ground, away from `safe` and from each other
fn packs(sp: &mut Spawns, w: &World, n: i32, gap: f32, mix: &[(Kind, f32)], size: (usize, usize), safe: &dyn Fn(f32, f32) -> bool, pick: &mut dyn FnMut() -> (f32, f32)) {
    let mut placed = 0;
    for _ in 0..n * 30 {
        if placed >= n {
            break;
        }
        let (x, y) = pick();
        if safe(x, y) || w.at(x as i32, y as i32) != E || sp.enemies.iter().any(|e| (e.1 - x).hypot(e.2 - y) < gap) {
            continue;
        }
        pack(&mut sp.enemies, w, x, y, size.0 + ri(size.1 + 1), mix);
        placed += 1;
    }
}
fn open_spot(w: &World, x: f32, y: f32, r: i32) -> bool {
    for dy in -r..=r {
        for dx in -r..=r {
            let (xx, yy) = (x as i32 + dx, y as i32 + dy);
            if !inarea(w, xx, yy) || w.m[idx(xx, yy)] != E {
                return false;
            }
        }
    }
    true
}
/// chests and shrines in quiet corners; `bad` keeps them off roads and out of the start
fn treasures(w: &World, sp: &mut Spawns, chests: i32, shrines: i32, tier: u8, bad: &dyn Fn(f32, f32) -> bool) {
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    let mut tries = 0;
    let mut placed = 0;
    while placed < chests && tries < 4000 {
        tries += 1;
        let (x, y) = (rr(40.0, aw - 40.0), rr(40.0, ah - 40.0));
        if bad(x, y) || !open_spot(w, x, y, 8) || sp.chests.iter().any(|c| (c.0 - x).hypot(c.1 - y) < 220.0) {
            continue;
        }
        sp.chests.push((x, y, if rnd() < 0.2 { 1 } else { tier }));
        placed += 1;
    }
    let mut placed = 0;
    tries = 0;
    while placed < shrines && tries < 4000 {
        tries += 1;
        let (x, y) = (rr(60.0, aw - 60.0), rr(60.0, ah - 60.0));
        if bad(x, y) || !open_spot(w, x, y, 12) || sp.shrines.iter().any(|c| (c.0 - x).hypot(c.1 - y) < 400.0) || sp.chests.iter().any(|c| (c.0 - x).hypot(c.1 - y) < 60.0) {
            continue;
        }
        sp.shrines.push((x, y, ri(4) as u8));
        placed += 1;
    }
}
fn waypoint(w: &mut World, sp: &mut Spawns, x: f32, y: f32) {
    for dy in -12i32..=12 {
        for dx in -16i32..=16 {
            let (xx, yy) = (x as i32 + dx, y as i32 + dy);
            if inarea(w, xx, yy) && ((dx * dx) as f32 / 256.0 + (dy * dy) as f32 / 144.0) < 1.0 {
                let i = idx(xx, yy);
                w.m[i] = E;
                w.g[i] = DIRT;
            }
        }
    }
    w.rect(x as i32 - 1, y as i32 - 1, x as i32 + 2, y as i32 + 1, STONE);
    sp.waypoint = Some((x, y));
}
/// an exit through the west or east edge of the map, with its arrival point
fn edge_exit(w: &World, sp: &mut Spawns, east: bool, y: f32, to: AreaId) {
    let aw = w.aw as i32;
    let (x0, x1, ax) = if east { (aw - 28, aw - 6, aw as f32 - 44.0) } else { (6, 28, 44.0) };
    sp.exits.push(Exit { x0, y0: y as i32 - 26, x1, y1: y as i32 + 26, to, label: to.name() });
    sp.entries.push((to, ax, y));
    let px = if east { aw as f32 - 34.0 } else { 34.0 };
    sp.posts.push((px, y - 24.0));
    sp.posts.push((px, y + 26.0));
}
/// lantern posts every so often along a road, on alternating sides
fn road_posts(w: &World, sp: &mut Spawns, road: &[(f32, f32)], every: f32) {
    let mut since = every * 0.5;
    let mut side = 1.0;
    for k in 1..road.len().saturating_sub(1) {
        let (a, b) = (road[k - 1], road[k + 1]);
        since += ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() * 0.5;
        if since < every {
            continue;
        }
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(0.01);
        let (x, y) = (road[k].0 - dy / l * 14.0 * side, road[k].1 + dx / l * 14.0 * side);
        if x < 60.0 || y < 60.0 || x > w.aw as f32 - 60.0 || y > w.ah as f32 - 60.0 || !open_spot(w, x, y, 2) {
            continue;
        }
        sp.posts.push((x, y));
        since = 0.0;
        side = -side;
    }
}
/// clear a round patch of ground to `g`
fn clearing(w: &mut World, x: f32, y: f32, rx: f32, ry: f32, g: u8) {
    for dy in -(ry as i32)..=ry as i32 {
        for dx in -(rx as i32)..=rx as i32 {
            let (xx, yy) = (x as i32 + dx, y as i32 + dy);
            if inarea(w, xx, yy) && (dx * dx) as f32 / (rx * rx) + (dy * dy) as f32 / (ry * ry) < 1.0 {
                let i = idx(xx, yy);
                if matches!(w.m[i], TREE | STONE | WATER | GAS) {
                    w.m[i] = E;
                }
                if g != 0 && rnd() < 0.85 {
                    w.g[i] = g;
                }
            }
        }
    }
}
/// a still pond with a muddy rim
fn pond(w: &mut World, px: f32, py: f32, size: f32, s: u32) {
    let rx = (size * 1.8) as i32;
    for dy in -rx..=rx {
        for dx in -rx..=rx {
            let (x, y) = (px as i32 + dx, py as i32 + dy);
            if !inarea(w, x, y) {
                continue;
            }
            let d = ((dx * dx) as f32 / 1.3 + (dy * dy) as f32).sqrt();
            let r = size * (0.8 + 0.4 * vnoise(x as f32 * 0.05, y as f32 * 0.05, s + 9));
            let i = idx(x, y);
            if d < r {
                w.m[i] = WATER;
            } else if d < r + 6.0 {
                w.g[i] = MUD;
            }
        }
    }
}
/// a row of barrels; each stands on a small block of wood, so fire, blasts and blades
/// can reach it (see props.rs)
fn barrels(w: &mut World, sp: &mut Spawns, x: f32, y: f32, n: usize, mix: &[(u8, f32)]) {
    let tot: f32 = mix.iter().map(|m| m.1).sum();
    for k in 0..n {
        let (bx, by) = (x + k as f32 * 13.0 + rr(-1.5, 1.5), y + rr(-2.0, 2.0) + if k % 2 == 1 { 5.0 } else { 0.0 });
        if !open_spot(w, bx, by, 6) {
            continue;
        }
        let mut r = rnd() * tot;
        let mut kind = mix[0].0;
        for &(kk, p) in mix {
            r -= p;
            if r <= 0.0 {
                kind = kk;
                break;
            }
        }
        w.rect(bx as i32 - 4, by as i32 - 3, bx as i32 + 5, by as i32 + 1, WOOD);
        sp.barrels.push((bx, by, kind));
    }
}
/// a stack of logs or crates
fn stack(w: &mut World, x: i32, y: i32, bw: i32, bh: i32) {
    w.rect(x, y, x + bw, y + bh, WOOD);
}

// ------------------------------------------------------------------ Ashford
fn town(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    let aw = w.aw as f32;
    ground_noise(w, s, 0.7, 0.28);
    let (tx0, ty0, tx1, ty1) = (300, 230, 720, 550);
    let road_y = 390.0;
    let road = path(w, &[(470.0, road_y), (620.0, road_y + 4.0), (760.0, road_y - 10.0), (900.0, road_y + 12.0), (aw - 4.0, road_y)], 7.0, s);
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
                w.g[idx(x, y)] = DIRT;
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
    // the merchant's stall, just off the square on the east road
    let (mx, my) = (600.0, 374.0);
    clearing(w, mx, my, 26.0, 14.0, DIRT);
    w.rect(mx as i32 - 14, my as i32 - 2, mx as i32 + 15, my as i32 + 2, WOOD);
    sp.merchant = Some((mx, my));
    sp.maren = Some((sx - 30.0, sy - 14.0));
    sp.lamps.push((mx, my - 10.0, [1.0, 0.75, 0.4]));
    for &(x, y) in &[(712.0, 372.0), (712.0, 408.0), (400.0, 330.0), (620.0, 340.0), (400.0, 430.0), (620.0, 436.0)] {
        sp.braziers.push((x, y));
    }
    // the dark forest all around
    let inside = |x: f32, y: f32, r: f32| x > tx0 as f32 - r - 8.0 && x < tx1 as f32 + r + 8.0 && y > ty0 as f32 - r - 8.0 && y < ty1 as f32 + r + 8.0;
    plant_trees(w, 170, 6000, 10.0, 16.0, s, false, &|x, y, r| !inside(x, y, r) && !near(&road, x, y, r * 0.6 + 14.0) && !near(&cross, x, y, r));
    road_posts(w, &mut sp, &road, 170.0);
    border(w);
    edge_exit(w, &mut sp, true, road_y, AreaId::Pines);
    sp.start = (sx + 20.0, sy + 16.0);
    sp
}

// ------------------------------------------------------------------ Hollow Pines
fn forest(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    ground_noise(w, s, 0.64, 0.3);
    let yw = rr(ah * 0.4, ah * 0.6);
    let ye = rr(ah * 0.35, ah * 0.65);
    // a stream runs north to south through the eastern woods
    let rx = rr(aw * 0.62, aw * 0.72);
    let stream = river(w, &[(rx, 0.0), (rx + rr(-80.0, 80.0), ah * 0.5), (rx + rr(-120.0, 120.0), ah)], 7.0, s + 11);
    let mid: Vec<(f32, f32)> = (1..9).map(|k| (k as f32 * aw / 9.0, rr(ah * 0.25, ah * 0.75))).collect();
    let mut ctrl = vec![(4.0, yw)];
    ctrl.extend(mid.iter().copied());
    ctrl.push((aw - 4.0, ye));
    let road = path(w, &ctrl, 7.0, s);
    // the crypt mausoleum, up a side path in the north
    let (mx, my) = (rr(aw * 0.35, aw * 0.5), 170.0);
    let (bx, by, bw, bh) = (mx as i32 - 38, my as i32 - 30, 76, 56);
    let jn = road.iter().copied().min_by(|a, b| (a.0 - mx).abs().partial_cmp(&(b.0 - mx).abs()).unwrap()).unwrap();
    let side = path(w, &[(mx, my + 30.0), (mx + 20.0, (my + jn.1) / 2.0), jn], 5.0, s + 2);
    building(w, &mut sp, bx, by, bw, bh, 22, BKind::CryptGate);
    for y in by + 14..by + 30 {
        for x in mx as i32 - 11..mx as i32 + 11 {
            w.g[idx(x, y)] = STAIRS;
        }
    }
    sp.lamps.push((mx, (by + 22) as f32, [0.45, 0.75, 1.0]));
    sp.exits.push(Exit { x0: mx as i32 - 12, y0: by + 12, x1: mx as i32 + 12, y1: by + 32, to: AreaId::Crypt(1), label: "THE SUNKEN CRYPT".into() });
    sp.entries.push((AreaId::Crypt(1), mx, my + 40.0));
    sp.braziers.push((mx - 24.0, my + 36.0));
    sp.braziers.push((mx + 24.0, my + 36.0));
    // the abandoned logging camp in the south
    let (lx, ly) = (rr(aw * 0.2, aw * 0.45), ah - rr(260.0, 330.0));
    let jl = road.iter().copied().min_by(|a, b| (a.0 - lx).abs().partial_cmp(&(b.0 - lx).abs()).unwrap()).unwrap();
    let camp_road = path(w, &[jl, (lx + 30.0, (ly + jl.1) / 2.0), (lx, ly)], 5.0, s + 3);
    clearing(w, lx, ly, 150.0, 95.0, DIRT);
    for (k, &(ox, oy)) in [(-120, -70), (40, -76), (-40, 30)].iter().enumerate() {
        let b = building(w, &mut sp, lx as i32 + ox, ly as i32 + oy, 64 + k as i32 * 6, 48, 16, BKind::Ruin);
        crumble(w, &b, 50);
    }
    for k in 0..5 {
        stack(w, lx as i32 + 70 + (k % 2) * 14, ly as i32 + 30 + k * 9, 30, 5);
    }
    sp.braziers.push((lx, ly + 4.0));
    sp.chests.push((lx - 85.0, ly - 40.0, 0));
    barrels(w, &mut sp, lx + 60.0, ly - 20.0, 3, &[(1, 0.7), (0, 0.3)]);
    // ponds away from the road
    let mut ponds = vec![];
    for _ in 0..200 {
        if ponds.len() >= 4 {
            break;
        }
        let (px, py) = (rr(150.0, aw - 150.0), rr(150.0, ah - 150.0));
        if near(&road, px, py, 110.0) || near(&stream, px, py, 120.0) || (px - mx).hypot(py - my) < 160.0 || (px - lx).hypot(py - ly) < 220.0 || ponds.iter().any(|&(a, b): &(f32, f32)| (a - px).hypot(b - py) < 300.0) {
            continue;
        }
        pond(w, px, py, rr(26.0, 46.0), s + ponds.len() as u32);
        ponds.push((px, py));
    }
    waypoint(w, &mut sp, 90.0, yw + 34.0);
    let keep = |x: f32, y: f32| (x < 180.0 && (y - yw).abs() < 100.0) || (x - mx).hypot(y - my) < 90.0 || ((x - lx) / 160.0).powi(2) + ((y - ly) / 105.0).powi(2) < 1.0;
    rocks(w, 240, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y));
    plant_trees(w, 560, 26000, 10.0, 16.0, s, false, &|x, y, r| {
        let edge = x < 70.0 || y < 70.0 || x > aw - 70.0 || y > ah - 70.0;
        let dens = fbm(x * 0.008, y * 0.008, s + 40) * 1.25 + if edge { 0.5 } else { 0.0 };
        rnd() < dens && !near(&road, x, y, r * 0.6 + 12.0) && !near(&side, x, y, r * 0.5 + 8.0) && !near(&camp_road, x, y, r * 0.5 + 8.0) && !keep(x, y)
    });
    road_posts(w, &mut sp, &road, 170.0);
    border(w);
    edge_exit(w, &mut sp, false, yw, AreaId::Ashford);
    edge_exit(w, &mut sp, true, ye, AreaId::Graveyard);
    sp.start = (90.0, yw);
    // the dead: packs along the road and in the woods
    let mix = [(Kind::Zombie, 0.65), (Kind::Crawler, 0.2), (Kind::Skeleton, 0.15)];
    let safe = |x: f32, y: f32| x < 260.0 && (y - yw).abs() < 200.0;
    let r2 = road.clone();
    packs(&mut sp, w, 30, 110.0, &mix, (3, 2), &safe, &mut || if rnd() < 0.5 { r2[ri(r2.len())] } else { (rr(60.0, aw - 60.0), rr(60.0, ah - 60.0)) });
    pack(&mut sp.enemies, w, lx, ly - 10.0, 5, &[(Kind::Zombie, 0.7), (Kind::Crawler, 0.3)]);
    sp.enemies.push((Kind::Brute, mx + 30.0, my + 60.0));
    sp.enemies.push((Kind::Brute, lx + 20.0, ly + 10.0));
    let (bx2, by2) = road[road.len() * 3 / 4];
    sp.enemies.push((Kind::Brute, bx2, by2 + 10.0));
    treasures(w, &mut sp, 5, 2, 0, &|x, y| near(&road, x, y, 30.0) || keep(x, y));
    sp
}

// ------------------------------------------------------------------ Old Graveyard
fn graveyard(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    ground_noise(w, s, 0.6, 0.42);
    let yw = rr(ah * 0.4, ah * 0.6);
    let ye = rr(ah * 0.3, ah * 0.7);
    let (cx, cy) = (rr(aw * 0.42, aw * 0.55), rr(ah * 0.38, ah * 0.6));
    let road = path(w, &[(4.0, yw), (aw * 0.15, yw + rr(-60.0, 60.0)), (aw * 0.3, cy + rr(-90.0, 90.0)), (cx - 70.0, cy + 70.0), (cx + 70.0, cy + 70.0), (aw * 0.7, cy + rr(-100.0, 100.0)), (aw * 0.85, ye + rr(-80.0, 80.0)), (aw - 4.0, ye)], 6.0, s);
    // ruined chapel
    let (x0, y0, cw, ch) = (cx as i32 - 80, cy as i32 - 60, 160, 110);
    let chapel = building(w, &mut sp, x0, y0, cw, ch, 22, BKind::Ruin);
    w.rect(x0, y0 + 30, x0 + 5, y0 + 54, E);
    crumble(w, &chapel, 110);
    for k in 0..4 {
        w.rect(x0 + 30 + k * 30, y0 + 30, x0 + 37 + k * 30, y0 + 37, BRICK);
    }
    // the altar
    w.rect(cx as i32 - 14, y0 + 12, cx as i32 + 14, y0 + 18, STONE);
    sp.braziers.push((cx - 50.0, cy - 30.0));
    sp.braziers.push((cx + 50.0, cy - 30.0));
    sp.chests.push((cx, y0 as f32 + 26.0, 1));
    // the plague pit: an open mass grave in a far corner
    let (qx, qy) = (if cx > aw * 0.5 { rr(200.0, 420.0) } else { rr(aw - 420.0, aw - 200.0) }, if rnd() < 0.5 { rr(160.0, 280.0) } else { rr(ah - 280.0, ah - 160.0) });
    for dy in -50i32..=50 {
        for dx in -80i32..=80 {
            let (x, y) = (qx as i32 + dx, qy as i32 + dy);
            let d = (dx * dx) as f32 / 6400.0 + (dy * dy) as f32 / 2500.0;
            if d < 1.0 && inarea(w, x, y) {
                let i = idx(x, y);
                w.m[i] = E;
                w.g[i] = MUD;
                w.stain[i] = (rnd() * 120.0) as u8;
                if rnd() < 0.06 * (1.0 - d) + 0.01 {
                    w.m[i] = BONE;
                    w.col[i] = [200, 192, 170];
                }
            }
        }
    }
    // plots of graves with low fences
    for _ in 0..34 {
        let (px, py) = (rr(160.0, aw - 160.0), rr(80.0, ah - 140.0));
        let (pw, ph) = (rr(70.0, 140.0) as i32, rr(50.0, 100.0) as i32);
        if (px - cx).abs() < 170.0 && (py - cy).abs() < 130.0 || (px - qx).abs() < 160.0 && (py - qy).abs() < 110.0 || px < 200.0 && (py - yw).abs() < 120.0 {
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
                                let i = idx(x + dx, y + dy);
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
    for _ in 0..40 {
        if sp.buildings.iter().filter(|b| b.k == BKind::Tomb).count() >= 12 {
            break;
        }
        let (x, y) = (rr(150.0, aw - 150.0) as i32, rr(80.0, ah - 130.0) as i32);
        if (x as f32 - cx).abs() < 180.0 && (y as f32 - cy).abs() < 150.0 || near(&road, x as f32 + 29.0, y as f32 + 23.0, 46.0) || sp.buildings.iter().any(|b| (b.x0 - x).abs() < 100 && (b.y0 - y).abs() < 90) {
            continue;
        }
        building(w, &mut sp, x, y, 58, 46, 16, BKind::Tomb);
        sp.enemies.push((Kind::Skeleton, x as f32 + 29.0, y as f32 + 24.0));
        if rnd() < 0.3 {
            sp.chests.push((x as f32 + 29.0, y as f32 + 16.0, 0));
        }
    }
    waypoint(w, &mut sp, 80.0, yw + 32.0);
    let keep = |x: f32, y: f32| x < 170.0 && (y - yw).abs() < 100.0;
    plant_trees(w, 170, 9000, 8.0, 13.0, s, true, &|x, y, r| !near(&road, x, y, r * 0.5 + 10.0) && !keep(x, y) && !((x - cx).abs() < 110.0 + r && (y - cy).abs() < 90.0 + r) && !((x - qx).abs() < 90.0 + r && (y - qy).abs() < 60.0 + r));
    rocks(w, 90, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y));
    road_posts(w, &mut sp, &road, 170.0);
    border(w);
    edge_exit(w, &mut sp, false, yw, AreaId::Pines);
    edge_exit(w, &mut sp, true, ye, AreaId::Fen);
    sp.start = (60.0, yw);
    let mix = [(Kind::Skeleton, 0.4), (Kind::Archer, 0.25), (Kind::Zombie, 0.25), (Kind::Crawler, 0.1)];
    let safe = |x: f32, y: f32| x < 260.0 && (y - yw).abs() < 200.0;
    packs(&mut sp, w, 26, 100.0, &mix, (3, 2), &safe, &mut || (rr(60.0, aw - 60.0), rr(60.0, ah - 60.0)));
    pack(&mut sp.enemies, w, qx, qy, 7, &[(Kind::Zombie, 0.5), (Kind::Crawler, 0.5)]);
    sp.enemies.push((Kind::Brute, cx, cy + 10.0));
    sp.enemies.push((Kind::Brute, qx + 30.0, qy));
    sp.enemies.push((Kind::Brute, rr(aw * 0.6, aw - 200.0), rr(150.0, ah - 150.0)));
    treasures(w, &mut sp, 4, 2, 0, &|x, y| near(&road, x, y, 30.0) || keep(x, y));
    sp
}

// ------------------------------------------------------------------ Blackwater Fen
fn fen(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    // peat, moss and mud underfoot; black water wherever the ground dips
    for y in 0..w.ah {
        for x in 0..w.aw {
            let (fx, fy) = (x as f32, y as f32);
            // mostly dark bog moss; peat in broad bands; a few drier grassy rises
            let n = fbm(fx * 0.007, fy * 0.007, s);
            let dry = fbm(fx * 0.009, fy * 0.009, s + 1);
            let i = y * W + x;
            w.g[i] = if n > 0.58 { PEAT } else if dry > 0.64 { GRASS } else { MOSS };
            let wet = fbm(fx * 0.011, fy * 0.011, s + 2);
            // keep the map edges dry so you can always arrive and leave
            let edge = ((x.min(w.aw - x) as f32 - 60.0) / 220.0).clamp(0.0, 1.0);
            if wet > 0.63 + (1.0 - edge) * 0.4 {
                w.m[i] = WATER;
            } else if wet > 0.6 {
                w.g[i] = MUD;
            }
        }
    }
    let yw = rr(ah * 0.4, ah * 0.6);
    let ye = rr(ah * 0.35, ah * 0.65);
    // a raised causeway with boardwalks over the deep water
    let mid: Vec<(f32, f32)> = (1..8).map(|k| (k as f32 * aw / 8.0, rr(ah * 0.25, ah * 0.75))).collect();
    let mut ctrl = vec![(4.0, yw)];
    ctrl.extend(mid.iter().copied());
    ctrl.push((aw - 4.0, ye));
    let road = path(w, &ctrl, 6.0, s);
    // a few side trails to the islands
    let mut trails = vec![];
    for k in 0..3 {
        let a = road[road.len() * (k + 1) / 4];
        let b = (a.0 + rr(-200.0, 200.0), if a.1 > ah * 0.5 { rr(120.0, ah * 0.35) } else { rr(ah * 0.65, ah - 120.0) });
        trails.extend(path(w, &[a, ((a.0 + b.0) / 2.0 + rr(-60.0, 60.0), (a.1 + b.1) / 2.0), b], 4.0, s + 20 + k as u32));
        // a sunken hut at the end of each trail
        clearing(w, b.0, b.1, 70.0, 50.0, PEAT);
        let hut = building(w, &mut sp, b.0 as i32 - 32, b.1 as i32 - 34, 64, 48, 16, BKind::Ruin);
        crumble(w, &hut, 60);
        sp.chests.push((b.0, b.1 - 14.0, if k == 1 { 1 } else { 0 }));
        sp.braziers.push((b.0 + 44.0, b.1 + 10.0));
        pack(&mut sp.enemies, w, b.0, b.1 + 30.0, 4, &[(Kind::Bloater, 0.5), (Kind::Zombie, 0.5)]);
    }
    // the hermit's hut, still lit, near the middle of the causeway
    let (hx, hy) = road[road.len() / 2];
    let hy = if hy > ah * 0.5 { hy - 110.0 } else { hy + 70.0 };
    clearing(w, hx, hy + 20.0, 80.0, 55.0, DIRT);
    building(w, &mut sp, hx as i32 - 36, hy as i32 - 10, 72, 52, 16, BKind::House);
    sp.chests.push((hx - 18.0, hy + 10.0, 0));
    // swamp gas collects in the hollows
    let mut gas = 0;
    for _ in 0..2000 {
        if gas >= 40 {
            break;
        }
        let (x, y) = (rr(200.0, aw - 200.0), rr(60.0, ah - 60.0));
        if near(&road, x, y, 40.0) || w.at(x as i32, y as i32) != E {
            continue;
        }
        let r = rr(6.0, 14.0);
        w.blob(x, y, r, GAS, s + 60 + gas, true);
        gas += 1;
    }
    waypoint(w, &mut sp, 90.0, yw + 34.0);
    let keep = |x: f32, y: f32| x < 180.0 && (y - yw).abs() < 100.0;
    let huts: Vec<(f32, f32)> = sp.buildings.iter().map(|b| ((b.x0 + b.w / 2) as f32, (b.y0 + b.h / 2) as f32)).collect();
    let by_hut = |x: f32, y: f32, r: f32| huts.iter().any(|&(a, b)| (a - x).abs() < 50.0 + r && (b - y).abs() < 40.0 + r);
    plant_trees(w, 160, 9000, 8.0, 13.0, s, true, &|x, y, r| !near(&road, x, y, r * 0.5 + 10.0) && !near(&trails, x, y, r * 0.5 + 6.0) && !keep(x, y) && !by_hut(x, y, r));
    plant_trees(w, 120, 9000, 9.0, 14.0, s + 5, false, &|x, y, r| fbm(x * 0.01, y * 0.01, s + 41) > 0.5 && !near(&road, x, y, r * 0.5 + 10.0) && !near(&trails, x, y, r * 0.5 + 6.0) && !keep(x, y) && !by_hut(x, y, r));
    rocks(w, 50, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y));
    road_posts(w, &mut sp, &road, 170.0);
    border(w);
    edge_exit(w, &mut sp, false, yw, AreaId::Graveyard);
    edge_exit(w, &mut sp, true, ye, AreaId::Ravenhold);
    sp.start = (60.0, yw);
    let mix = [(Kind::Zombie, 0.4), (Kind::Bloater, 0.25), (Kind::Crawler, 0.2), (Kind::Skeleton, 0.15)];
    let safe = |x: f32, y: f32| x < 260.0 && (y - yw).abs() < 200.0;
    let r2 = road.clone();
    packs(&mut sp, w, 30, 110.0, &mix, (3, 2), &safe, &mut || if rnd() < 0.5 { r2[ri(r2.len())] } else { (rr(60.0, aw - 60.0), rr(60.0, ah - 60.0)) });
    let (bx, by) = road[road.len() * 2 / 3];
    sp.enemies.push((Kind::Brute, bx, by + 8.0));
    sp.enemies.push((Kind::Brute, hx + 50.0, hy + 40.0));
    treasures(w, &mut sp, 3, 2, 0, &|x, y| near(&road, x, y, 30.0) || keep(x, y));
    sp
}

// ------------------------------------------------------------------ Ravenhold
fn ravenhold(w: &mut World, s: u32) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    ground_noise(w, s, 0.66, 0.34);
    let yw = rr(ah * 0.42, ah * 0.58);
    // the castle fills the east of the map
    let (c0, r0, c1, r1) = (560, 200, w.aw as i32 - 120, w.ah as i32 - 180);
    let gate_y = ((r0 + r1) / 2) as f32 + rr(-60.0, 60.0);
    let road = path(w, &[(4.0, yw), (aw * 0.15, yw + rr(-50.0, 50.0)), (aw * 0.3, gate_y + rr(-40.0, 40.0)), (c0 as f32 - 40.0, gate_y), (c0 as f32 + 80.0, gate_y)], 7.0, s);
    // the courtyard is bare packed earth
    for y in r0..r1 {
        for x in c0..c1 {
            let i = idx(x, y);
            // trodden earth, muddier in the low spots
            w.g[i] = if fbm(x as f32 * 0.012, y as f32 * 0.012, s + 3) > 0.6 { MUD } else { DIRT };
        }
    }
    // curtain walls, 7 thick, with a gatehouse on the west and breaches
    let t = 7;
    w.rect(c0, r0, c1, r0 + t, BRICK);
    w.rect(c0, r1 - t, c1, r1, BRICK);
    w.rect(c0, r0, c0 + t, r1, BRICK);
    w.rect(c1 - t, r0, c1, r1, BRICK);
    w.rect(c0 - 2, gate_y as i32 - 18, c0 + t + 2, gate_y as i32 + 18, E);
    for &(gx, gy) in &[(c0 - 14, gate_y as i32 - 34), (c0 - 14, gate_y as i32 + 18)] {
        w.rect(gx, gy, gx + 30, gy + 16, BRICK);
    }
    sp.braziers.push((c0 as f32 - 22.0, gate_y - 26.0));
    sp.braziers.push((c0 as f32 - 22.0, gate_y + 26.0));
    // towers at the corners
    for &(tx, ty) in &[(c0, r0), (c1, r0), (c0, r1), (c1, r1)] {
        w.rect(tx - 20, ty - 20, tx + 20, ty + 20, BRICK);
    }
    // breaches in the walls, rubble spilling out
    for _ in 0..4 {
        let side = ri(4);
        let (bx, by) = match side {
            0 => (rr(c0 as f32 + 80.0, c1 as f32 - 80.0) as i32, r0),
            1 => (rr(c0 as f32 + 80.0, c1 as f32 - 80.0) as i32, r1 - t),
            2 => (c1 - t, rr(r0 as f32 + 80.0, r1 as f32 - 80.0) as i32),
            _ => (c0, rr(r0 as f32 + 80.0, gate_y - 80.0).max(r0 as f32 + 60.0) as i32),
        };
        let horiz = side < 2;
        let len = rr(16.0, 28.0) as i32;
        if horiz {
            w.rect(bx, by, bx + len, by + t, E);
        } else {
            w.rect(bx, by, bx + t, by + len, E);
        }
        for _ in 0..40 {
            let (x, y) = (bx + rr(-16.0, len as f32 + 16.0) as i32, by + rr(-16.0, 22.0) as i32);
            if w.at(x, y) == E {
                w.set(x, y, RUBBLE);
            }
        }
    }
    // the keep: a roofless ruin with the cellar stairs inside
    let (kw, kh) = (220, 160);
    let (kx, ky) = (c1 - kw - 70, (r0 + r1) / 2 - kh / 2 - 20);
    let keep_b = building(w, &mut sp, kx, ky, kw, kh, 30, BKind::Ruin);
    crumble(w, &keep_b, 140);
    for k in 0..3 {
        w.rect(kx + 50 + k * 60, ky + 50, kx + 58 + k * 60, ky + 58, BRICK);
        w.rect(kx + 50 + k * 60, ky + 104, kx + 58 + k * 60, ky + 112, BRICK);
    }
    let (sx, sy) = (kx + kw / 2, ky + 26);
    // a cobbled road from the gate across the yard to the keep's door
    let yard_road = path(w, &[(c0 as f32 + 4.0, gate_y), ((c0 + kx) as f32 / 2.0, gate_y + rr(-20.0, 20.0)), ((kx + kw / 2) as f32, (ky + kh) as f32 + 24.0), ((kx + kw / 2) as f32, (ky + kh) as f32 + 2.0)], 6.0, s + 7);
    let _ = yard_road;
    for y in sy..sy + 16 {
        for x in sx - 12..sx + 12 {
            w.g[idx(x, y)] = STAIRS;
        }
    }
    sp.exits.push(Exit { x0: sx - 13, y0: sy - 2, x1: sx + 13, y1: sy + 18, to: AreaId::Cellars(1), label: "THE CELLARS".into() });
    sp.entries.push((AreaId::Cellars(1), sx as f32, sy as f32 + 30.0));
    sp.lamps.push((sx as f32, sy as f32 + 8.0, [1.0, 0.55, 0.25]));
    sp.braziers.push((sx as f32 - 30.0, sy as f32 + 24.0));
    sp.braziers.push((sx as f32 + 30.0, sy as f32 + 24.0));
    sp.chests.push((kx as f32 + 24.0, ky as f32 + 24.0, 1));
    // barracks and stores along the north and south walls
    let mut halls = vec![];
    for k in 0..3 {
        let bx = c0 + 70 + k * 170;
        if bx + 110 > kx - 30 {
            break;
        }
        building(w, &mut sp, bx, r0 + 24, 110, 64, 18, BKind::House);
        halls.push((bx, r0 + 24));
        if rnd() < 0.7 {
            let b = building(w, &mut sp, bx + 10, r1 - 100, 90, 58, 18, BKind::Ruin);
            crumble(w, &b, 40);
        }
    }
    // the bandit camp in the courtyard: tents, cookfires, crates and spilt oil
    let (yx0, yx1) = (c0 + 60, kx - 40);
    for _ in 0..9 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32 - 40.0) as i32, rr(r0 as f32 + 110.0, r1 as f32 - 130.0) as i32);
        if (y as f32 - gate_y).abs() < 30.0 || w.at(x, y) != E || w.at(x + 34, y + 22) != E {
            continue;
        }
        // a tent of cloth over a pole frame
        let tc = [[120u8, 46, 40], [96, 84, 60], [70, 66, 90]][ri(3)];
        for ty in 0..22 {
            let half = 6 + ty * 11 / 22;
            for tx in -half..=half {
                let (xx, yy) = (x + 17 + tx, y + ty);
                if ty == 21 && tx.abs() < 3 {
                    continue;
                }
                if inarea(w, xx, yy) {
                    let i = idx(xx, yy);
                    w.m[i] = CLOTH;
                    w.col[i] = [(tc[0] as f32 * rr(0.85, 1.1)) as u8, (tc[1] as f32 * rr(0.85, 1.1)) as u8, (tc[2] as f32 * rr(0.85, 1.1)) as u8];
                }
            }
        }
        sp.braziers.push((x as f32 + 17.0, y as f32 + 34.0));
    }
    for _ in 0..14 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32) as i32, rr(r0 as f32 + 100.0, r1 as f32 - 100.0) as i32);
        if (y as f32 - gate_y).abs() > 24.0 && w.at(x, y) == E {
            stack(w, x, y, 7 + ri(4) as i32, 6);
        }
    }
    for _ in 0..6 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32), rr(r0 as f32 + 90.0, r1 as f32 - 90.0));
        w.splash(x, y, OIL, rr(4.0, 7.0), 0);
    }
    // the bandits' powder and lamp oil, stacked about the yard
    for _ in 0..7 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32 - 40.0), rr(r0 as f32 + 100.0, r1 as f32 - 100.0));
        if (y - gate_y).abs() > 30.0 {
            barrels(w, &mut sp, x, y, 2 + ri(3), &[(0, 0.6), (1, 0.4)]);
        }
    }
    // a gallows by the gate, as a warning
    w.rect(c0 + 40, gate_y as i32 - 70, c0 + 44, gate_y as i32 - 40, WOOD);
    w.rect(c0 + 40, gate_y as i32 - 72, c0 + 64, gate_y as i32 - 69, WOOD);
    waypoint(w, &mut sp, 90.0, yw + 34.0);
    // woods and boulders outside the walls
    let keep = |x: f32, y: f32| x < 180.0 && (y - yw).abs() < 100.0;
    let castle = |x: f32, y: f32, r: f32| x > c0 as f32 - 50.0 - r && y > r0 as f32 - 50.0 - r && y < r1 as f32 + 50.0 + r;
    plant_trees(w, 240, 12000, 10.0, 15.0, s, false, &|x, y, r| {
        let dens = fbm(x * 0.009, y * 0.009, s + 42) * 1.2;
        rnd() < dens && !near(&road, x, y, r * 0.6 + 12.0) && !keep(x, y) && !castle(x, y, r)
    });
    rocks(w, 120, s, &|x, y| !near(&road, x, y, 14.0) && !keep(x, y) && !castle(x, y, 0.0));
    road_posts(w, &mut sp, &road, 170.0);
    border(w);
    edge_exit(w, &mut sp, false, yw, AreaId::Fen);
    sp.start = (60.0, yw);
    // bandits: a few on the road, most inside the walls
    let mix = [(Kind::Bandit, 0.62), (Kind::Crossbow, 0.38)];
    let safe = |x: f32, y: f32| x < 300.0 && (y - yw).abs() < 220.0;
    let r2 = road.clone();
    packs(&mut sp, w, 5, 120.0, &mix, (2, 2), &safe, &mut || r2[ri(r2.len())]);
    packs(&mut sp, w, 14, 80.0, &mix, (3, 2), &safe, &mut || (rr(c0 as f32 + 30.0, c1 as f32 - 30.0), rr(r0 as f32 + 30.0, r1 as f32 - 30.0)));
    for &(hx, hy) in &halls {
        pack(&mut sp.enemies, w, hx as f32 + 55.0, hy as f32 + 32.0, 3, &mix);
    }
    pack(&mut sp.enemies, w, sx as f32, sy as f32 + 60.0, 5, &mix);
    sp.enemies.push((Kind::Brute, sx as f32, sy as f32 + 80.0));
    pack(&mut sp.enemies, w, c0 as f32 - 60.0, gate_y, 4, &mix);
    treasures(w, &mut sp, 3, 1, 0, &|x, y| near(&road, x, y, 30.0) || keep(x, y));
    sp
}

// ------------------------------------------------------------------ dungeons
struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cave: bool,
}
impl Room {
    fn c(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

/// a dungeon floor: the Sunken Crypt, or (cellars) the vaults under Ravenhold.
/// Big open halls joined by wide passages, so fights have room and you can see them.
fn dungeon(w: &mut World, floor: u8, cellars: bool) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as i32, w.ah as i32);
    for y in 0..w.ah {
        for x in 0..w.aw {
            let i = y * W + x;
            w.m[i] = ROCK;
            w.g[i] = FLOOR;
            w.aux[i] = AUX_INDOOR;
        }
    }
    let last = if cellars { CELLAR_FLOORS } else { CRYPT_FLOORS };
    let boss = floor == last;
    let mut rooms: Vec<Room> = vec![];
    let target = 10 + ri(3);
    // boss arena first, at the far end
    if boss {
        rooms.push(Room { x: aw - 420 + ri(60) as i32, y: rr(140.0, ah as f32 - 400.0) as i32, w: 330, h: 250, cave: false });
    }
    for _ in 0..3000 {
        if rooms.len() >= target {
            break;
        }
        let (rw, rh) = (rr(150.0, 290.0) as i32, rr(110.0, 200.0) as i32);
        let (x, y) = (rr(40.0, (aw - rw - 40) as f32) as i32, rr(40.0, (ah - rh - 40) as f32) as i32);
        if rooms.iter().any(|r| x < r.x + r.w + 46 && x + rw + 46 > r.x && y < r.y + r.h + 46 && y + rh + 46 > r.y) {
            continue;
        }
        rooms.push(Room { x, y, w: rw, h: rh, cave: !cellars && rnd() < 0.35 });
    }
    // order rooms into a winding chain from the far side to the boss / stairs down
    if boss {
        let b = rooms.remove(0);
        rooms.sort_by_key(|r| r.c().0);
        rooms.push(b);
    } else {
        rooms.sort_by_key(|r| r.c().0 + r.c().1 / 3);
    }
    let ok = |x: i32, y: i32| x >= 8 && y >= 8 && x < aw - 8 && y < ah - 8;
    let carve = |w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32| {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                if ok(x, y) {
                    w.m[idx(x, y)] = E;
                }
            }
        }
    };
    let disc = |w: &mut World, cx: i32, cy: i32, r: i32| {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r && ok(cx + dx, cy + dy) {
                    w.m[idx(cx + dx, cy + dy)] = E;
                }
            }
        }
    };
    let sd = (next() & 0xffff) as u32;
    for r in &rooms {
        if r.cave {
            // a natural cavern: a lumpy oval
            let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
            for y in r.y - 10..r.y + r.h + 10 {
                for x in r.x - 10..r.x + r.w + 10 {
                    let (nx, ny) = ((x as f32 - cx) / (r.w as f32 / 2.0), (y as f32 - cy) / (r.h as f32 / 2.0));
                    let lim = 1.0 - 0.3 * vnoise(x as f32 * 0.04, y as f32 * 0.04, sd);
                    if nx * nx + ny * ny < lim && ok(x, y) {
                        w.m[idx(x, y)] = E;
                    }
                }
            }
        } else {
            carve(w, r.x, r.y, r.x + r.w, r.y + r.h);
        }
    }
    let corridor = |w: &mut World, a: (i32, i32), b: (i32, i32)| {
        let hw = 16 + ri(6) as i32;
        if rnd() < 0.5 {
            carve(w, a.0, a.1 - hw, b.0, a.1 + hw);
            carve(w, b.0 - hw, a.1, b.0 + hw, b.1);
            disc(w, b.0, a.1, hw + 8);
        } else {
            carve(w, a.0 - hw, a.1, a.0 + hw, b.1);
            carve(w, a.0, b.1 - hw, b.0, b.1 + hw);
            disc(w, a.0, b.1, hw + 8);
        }
    };
    for k in 0..rooms.len() - 1 {
        corridor(w, rooms[k].c(), rooms[k + 1].c());
    }
    for _ in 0..3 {
        let (a, b) = (ri(rooms.len() - 1), ri(rooms.len() - 1));
        if a != b {
            corridor(w, rooms[a].c(), rooms[b].c());
        }
    }
    // line the carved spaces with brick: paint a ring round every open cell that touches rock
    let snapshot = w.m.clone();
    for y in 5..ah - 5 {
        for x in 5..aw - 5 {
            let i = idx(x, y);
            if snapshot[i] != E {
                continue;
            }
            if snapshot[i - 1] == E && snapshot[i + 1] == E && snapshot[i - W] == E && snapshot[i + W] == E {
                continue;
            }
            for dy in -4i32..=4 {
                for dx in -4i32..=4 {
                    let j = idx(x + dx, y + dy);
                    if snapshot[j] == ROCK {
                        w.m[j] = BRICK;
                    }
                }
            }
        }
    }
    // dress the rooms
    let n = rooms.len();
    let kegs: &[(u8, f32)] = if cellars { &[(0, 0.5), (1, 0.4), (2, 0.1)] } else { &[(0, 0.25), (1, 0.5), (2, 0.25)] };
    for (k, r) in rooms.iter().enumerate() {
        let (cx, cy) = r.c();
        let ordinary = k > 0 && k < n - 1 || (!boss && k == n - 1);
        if ordinary {
            if cellars && rnd() < 0.5 {
                // a storeroom: plank floor, crates along the walls
                w.ground_rect(r.x, r.y, r.x + r.w, r.y + r.h, PLANK, AUX_INDOOR);
                for _ in 0..(r.w * r.h / 2500) {
                    let side = rnd() < 0.5;
                    let (bx, by) = if side {
                        (r.x + 8 + ri((r.w - 24).max(1) as usize) as i32, if rnd() < 0.5 { r.y + 8 } else { r.y + r.h - 14 })
                    } else {
                        (if rnd() < 0.5 { r.x + 8 } else { r.x + r.w - 20 }, r.y + 8 + ri((r.h - 20).max(1) as usize) as i32)
                    };
                    stack(w, bx, by, 8 + ri(5) as i32, 6 + ri(3) as i32);
                }
            } else if !r.cave && r.w > 190 && r.h > 140 && rnd() < 0.6 {
                // a pillared hall
                let (nx, ny) = (3, 2);
                for gy in 0..ny {
                    for gx in 0..nx {
                        let px = r.x + (gx + 1) * r.w / (nx + 1) - 6;
                        let py = r.y + (gy + 1) * r.h / (ny + 1) - 6;
                        w.rect(px, py, px + 12, py + 12, BRICK);
                    }
                }
            }
            if !cellars && !r.cave && rnd() < 0.3 {
                // coffins along the north wall
                for c in 0..4 {
                    let x = r.x + 14 + c * 20;
                    if x + 8 < r.x + r.w - 10 {
                        w.rect(x, r.y + 8, x + 8, r.y + 24, WOOD);
                    }
                }
            }
            if rnd() < 0.3 {
                w.splash(cx as f32 + rr(-30.0, 30.0), cy as f32 + rr(-20.0, 20.0), OIL, rr(7.0, 12.0), 0);
            }
            if !cellars && rnd() < 0.18 {
                let (ax, ay) = (cx + rr(-30.0, 30.0) as i32, cy + rr(-15.0, 15.0) as i32);
                for dy in -11i32..=11 {
                    for dx in -16i32..=16 {
                        if (dx * dx) as f32 / 256.0 + (dy * dy) as f32 / 121.0 < 1.0 && w.at(ax + dx, ay + dy) == E {
                            w.set(ax + dx, ay + dy, ACID);
                        }
                    }
                }
            }
            for _ in 0..(r.w * r.h / 500) {
                let (bx, by) = (r.x + ri(r.w as usize) as i32, r.y + ri(r.h as usize) as i32);
                if w.at(bx, by) == E {
                    let i = idx(bx, by);
                    if !cellars && rnd() < 0.4 {
                        w.m[i] = BONE;
                        w.col[i] = [196, 188, 166];
                    } else {
                        w.stain[i] = w.stain[i].saturating_add(90);
                    }
                }
            }
            // barrels here and there: powder kegs, oil and acid
            if rnd() < 0.5 {
                let (bx, by) = (r.x as f32 + rr(30.0, (r.w - 60).max(31) as f32), r.y as f32 + if rnd() < 0.5 { 20.0 } else { r.h as f32 - 12.0 });
                barrels(w, &mut sp, bx, by, 2 + ri(3), kegs);
            }
            // treasure in some rooms, a shrine now and then
            if rnd() < 0.25 {
                let (tx, ty) = (r.x as f32 + rr(20.0, r.w as f32 - 20.0), r.y as f32 + 18.0);
                if open_spot(w, tx, ty, 6) {
                    sp.chests.push((tx, ty, if rnd() < 0.3 { 1 } else { 0 }));
                }
            } else if rnd() < 0.1 && sp.shrines.is_empty() && open_spot(w, cx as f32, cy as f32, 12) {
                sp.shrines.push((cx as f32, cy as f32, ri(4) as u8));
            }
        }
        // light: braziers round the room
        let spots = [(r.x + 14, r.y + 16), (r.x + r.w - 14, r.y + 16), (r.x + 14, r.y + r.h - 10), (r.x + r.w - 14, r.y + r.h - 10)];
        let want = if r.w > 180 { 4 } else { 2 };
        for &(bx, by) in spots.iter().take(want) {
            if open_spot(w, bx as f32, by as f32, 3) {
                sp.braziers.push((bx as f32, by as f32));
            }
        }
        // enemies: bigger rooms, bigger packs
        if k > 0 && !(boss && k == n - 1) {
            let cnt = 3 + ri(3) + if floor > 1 { 1 } else { 0 } + (r.w * r.h / 20000) as usize;
            let mix: &[(Kind, f32)] = if cellars {
                &[(Kind::Bandit, 0.55), (Kind::Crossbow, 0.35), (Kind::Zombie, 0.1)]
            } else {
                &[(Kind::Skeleton, 0.45), (Kind::Archer, 0.2), (Kind::Zombie, 0.2), (Kind::Crawler, 0.15)]
            };
            pack(&mut sp.enemies, w, cx as f32 - 30.0, cy as f32, cnt / 2 + 1, mix);
            pack(&mut sp.enemies, w, cx as f32 + 30.0, cy as f32, cnt / 2, mix);
            if rnd() < 0.2 + floor as f32 * 0.08 {
                sp.enemies.push((Kind::Brute, cx as f32 + rr(-10.0, 10.0), cy as f32 + rr(-6.0, 6.0)));
            }
        }
    }
    // stairs up in the first room, down (or the boss) in the last
    let (ux, uy) = rooms[0].c();
    for y in uy - 20..uy - 8 {
        for x in ux - 9..ux + 9 {
            w.m[idx(x, y)] = E;
            w.g[idx(x, y)] = STAIRSUP;
        }
    }
    let up = if floor > 1 {
        if cellars { AreaId::Cellars(floor - 1) } else { AreaId::Crypt(floor - 1) }
    } else if cellars {
        AreaId::Ravenhold
    } else {
        AreaId::Pines
    };
    let up_label = if floor > 1 { if cellars { format!("DEPTH {}", floor - 1) } else { format!("FLOOR {}", floor - 1) } } else { up.name() };
    sp.exits.push(Exit { x0: ux - 10, y0: uy - 22, x1: ux + 10, y1: uy - 6, to: up, label: up_label });
    sp.entries.push((up, ux as f32, uy as f32 + 6.0));
    sp.start = (ux as f32, uy as f32 + 6.0);
    let (dx, dy) = rooms[n - 1].c();
    if boss {
        sp.enemies.push((if cellars { Kind::Chief } else { Kind::King }, dx as f32, dy as f32 - 10.0));
        if cellars {
            // the chief's hoard, and his powder store
            sp.chests.push((dx as f32 - 90.0, dy as f32 - 80.0, 1));
            sp.chests.push((dx as f32 + 90.0, dy as f32 - 80.0, 1));
            barrels(w, &mut sp, dx as f32 - 130.0, dy as f32 + 90.0, 4, &[(0, 1.0)]);
            barrels(w, &mut sp, dx as f32 + 90.0, dy as f32 + 90.0, 3, &[(0, 0.5), (1, 0.5)]);
        } else {
            barrels(w, &mut sp, dx as f32 - 120.0, dy as f32 + 90.0, 3, &[(1, 1.0)]);
        }
        for k in 0..4 {
            sp.braziers.push((dx as f32 + [-110.0, 110.0, -110.0, 110.0][k], dy as f32 + [-80.0, -80.0, 80.0, 80.0][k]));
        }
    } else {
        for y in dy - 6..dy + 6 {
            for x in dx - 9..dx + 9 {
                w.m[idx(x, y)] = E;
                w.g[idx(x, y)] = STAIRS;
            }
        }
        let down = if cellars { AreaId::Cellars(floor + 1) } else { AreaId::Crypt(floor + 1) };
        let label = if cellars { format!("DEPTH {}", floor + 1) } else { format!("FLOOR {}", floor + 1) };
        sp.exits.push(Exit { x0: dx - 10, y0: dy - 8, x1: dx + 10, y1: dy + 8, to: down, label });
        sp.entries.push((down, dx as f32, dy as f32 + 18.0));
    }
    sp
}
