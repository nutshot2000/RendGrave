# Buildings with character: roofs, framed doors, windows, chimneys, pillars.


def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


# ---------------- world: new aux bits and plank floors ----------------
edit('src/world.rs', [
    ("pub const AUX_DEAD: u8 = 4;", "pub const AUX_DEAD: u8 = 4;\n/// wall cells of a timber-framed house\npub const AUX_HOUSE: u8 = 8;\n/// wall cells of a stone tomb or chapel\npub const AUX_TOMB: u8 = 16;"),
    ("pub const STAIRSUP: u8 = 7;", "pub const STAIRSUP: u8 = 7;\npub const PLANK: u8 = 8;"),
])

# ---------------- generators ----------------
edit('src/areagen.rs', [
    ("""pub struct Spawns {
    pub start: (f32, f32),""", """#[derive(Clone, Copy, PartialEq, Debug)]
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
    pub start: (f32, f32),"""),
    ("        Spawns { start: (100.0, 100.0), entries: vec![], enemies: vec![], braziers: vec![], exits: vec![], waypoint: None }",
     "        Spawns { buildings: vec![], lamps: vec![], start: (100.0, 100.0), entries: vec![], enemies: vec![], braziers: vec![], exits: vec![], waypoint: None }"),
    ("""fn building(w: &mut World, x0: i32, y0: i32, bw: i32, bh: i32, wall: u8, door: i32) {
    w.ground_rect(x0, y0, x0 + bw, y0 + bh, FLOOR, AUX_INDOOR);
    w.rect(x0, y0, x0 + bw, y0 + 5, wall);
    w.rect(x0, y0, x0 + 5, y0 + bh, wall);
    w.rect(x0 + bw - 5, y0, x0 + bw, y0 + bh, wall);
    let dc = x0 + bw / 2;
    w.rect(x0, y0 + bh - 5, dc - door / 2, y0 + bh, wall);
    w.rect(dc + door / 2, y0 + bh - 5, x0 + bw, y0 + bh, wall);
}""", """/// a building with walls, a south doorway and (unless a ruin) a roof drawn over it
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
}"""),
    # town houses: bigger, timber-framed
    ("""    let houses = [(330, 260, 70, 50), (420, 252, 60, 48), (560, 255, 66, 50), (640, 262, 60, 46), (335, 440, 64, 52), (580, 450, 70, 50), (430, 460, 56, 44)];
    for &(x, y, bw, bh) in &houses {
        building(w, x, y, bw, bh, BRICK, 12);""", """    let houses = [(318, 252, 84, 56), (414, 246, 74, 54), (552, 250, 80, 56), (644, 258, 66, 52), (322, 448, 80, 58), (570, 452, 84, 58), (428, 466, 68, 52)];
    for &(x, y, bw, bh) in &houses {
        building(w, &mut sp, x, y, bw, bh, 18, BKind::House);"""),
    # the crypt gate in the forest
    ("""    let (bx, by, bw, bh) = (mx as i32 - 30, my as i32 - 26, 60, 44);
    let side = path(w, &[(mx, my + 24.0), (mx + 10.0, my + 80.0), (road[road.len() * 2 / 3].0, road[road.len() * 2 / 3].1)], 5.0, s + 2);
    building(w, bx, by, bw, bh, BRICK, 16);
    for y in by + 12..by + 24 {
        for x in mx as i32 - 9..mx as i32 + 9 {
            w.g[y as usize * W + x as usize] = STAIRS;
        }
    }
    sp.exits.push(Exit { x0: mx as i32 - 10, y0: by + 10, x1: mx as i32 + 10, y1: by + 26, to: AreaId::Crypt(1), label: "THE SUNKEN CRYPT".into() });
    sp.entries.push((AreaId::Crypt(1), mx, my + 30.0));
    sp.braziers.push((mx - 16.0, my + 24.0));
    sp.braziers.push((mx + 16.0, my + 24.0));""", """    let (bx, by, bw, bh) = (mx as i32 - 38, my as i32 - 30, 76, 56);
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
    sp.braziers.push((mx + 24.0, my + 36.0));"""),
    ("    building(w, x0, y0, cw, ch, BRICK, 20);", "    building(w, &mut sp, x0, y0, cw, ch, 22, BKind::Ruin);"),
    ("""        building(w, x, y, 44, 36, BRICK, 10);
        sp.enemies.push((Kind::Skeleton, x as f32 + 22.0, y as f32 + 20.0));""", """        building(w, &mut sp, x, y, 58, 46, 16, BKind::Tomb);
        sp.enemies.push((Kind::Skeleton, x as f32 + 29.0, y as f32 + 24.0));"""),
    ("""        if (x as f32 - cx).abs() < 150.0 && (y as f32 - cy).abs() < 120.0 || near(&road, x as f32 + 22.0, y as f32 + 18.0, 40.0) {""",
     """        if (x as f32 - cx).abs() < 160.0 && (y as f32 - cy).abs() < 130.0 || near(&road, x as f32 + 29.0, y as f32 + 23.0, 46.0) || sp.buildings.iter().any(|b| (b.x0 - x).abs() < 90 && (b.y0 - y).abs() < 80) {"""),
])

# ---------------- game: keep the buildings, chimney smoke, interior lamps ----------------
edit('src/game.rs', [
    ("    pub saving: bool,\n}", "    pub saving: bool,\n    pub buildings: Vec<Building>,\n    pub lamps: Vec<(f32, f32, [f32; 3])>,\n}"),
    ("            saving: false,\n        }", "            saving: false,\n            buildings: vec![],\n            lamps: vec![],\n        }"),
    ("""    fn update_misc(&mut self) {
        let br = self.rank(Sk::Brittle);""", """    fn update_misc(&mut self) {
        let br = self.rank(Sk::Brittle);
        // chimney smoke
        if self.frame % 7 == 0 {
            let chims: Vec<(i32, i32, i32)> = self.buildings.iter().filter_map(|b| b.chimney().map(|(x, y)| (x, y, b.h))).collect();
            for (x, y, h) in chims {
                let z = 44.0 + h as f32 * 0.75 + 4.0;
                self.ptc(x as f32 + 2.0, y as f32, z, rr(-0.05, 0.05) + 0.06, 0.0, rr(0.2, 0.35), rr(120.0, 200.0) as i32, PtK::Smoke, [70, 66, 70]);
            }
        }"""),
])
edit('src/areas.rs', [
    ("        self.braziers = sp.braziers.clone();", "        self.braziers = sp.braziers.clone();\n        self.buildings = sp.buildings.clone();\n        self.lamps = sp.lamps.clone();"),
])
print("buildings data ok")
