def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/world.rs', [("pub const AUX_TOMB: u8 = 16;", "pub const AUX_TOMB: u8 = 16;\n/// a cracked wall: a blade or a blast breaks it open\npub const AUX_CRACK: u8 = 32;")])

A = 'src/areagen.rs'
edit(A, [("""    // stairs up in the first room, down (or the boss) in the last
    let (ux, uy) = rooms[0].c();""", """    // secret alcoves: a treasure room sealed behind a cracked wall above a room or two
    let mut secrets = 0;
    for k in 1..n.saturating_sub(1) {
        if secrets >= 2 {
            break;
        }
        let r = &rooms[k];
        if r.cave || rnd() < 0.4 {
            continue;
        }
        let cx = r.x + r.w / 2 + rr(-30.0, 30.0) as i32;
        let (ax0, ax1, ay0, ay1) = (cx - 16, cx + 16, r.y - 44, r.y - 11);
        if ay0 < 14 || ax0 < 14 || ax1 > aw - 14 {
            continue;
        }
        let sealed = (ay0 - 6..r.y).all(|y| (ax0 - 6..=ax1 + 6).all(|x| w.m[idx(x, y)] != E));
        if !sealed {
            continue;
        }
        for y in ay0..ay1 {
            for x in ax0..ax1 {
                w.m[idx(x, y)] = E;
            }
        }
        for y in ay0 - 4..ay1 + 4 {
            for x in ax0 - 4..ax1 + 4 {
                if w.m[idx(x, y)] == ROCK {
                    w.m[idx(x, y)] = BRICK;
                }
            }
        }
        // the cracked plug between the room and the alcove
        for y in ay1..r.y {
            for x in cx - 8..cx + 8 {
                let i = idx(x, y);
                w.m[i] = BRICK;
                w.aux[i] |= AUX_CRACK;
            }
        }
        sp.chests.push((cx as f32, (ay0 + 14) as f32, 1));
        barrels(w, &mut sp, (ax0 + 6) as f32, (ay1 - 4) as f32, 2, &[(3, 1.0)]);
        for y in ay0 + 16..ay0 + 30 {
            for x in ax0 + 2..ax1 - 2 {
                if rnd() < 0.04 && w.at(x, y) == E {
                    let i = idx(x, y);
                    w.m[i] = BONE;
                    w.col[i] = [196, 188, 166];
                }
            }
        }
        sp.cracks.push((cx as f32, r.y as f32 - 6.0));
        secrets += 1;
    }
    // stairs up in the first room, down (or the boss) in the last
    let (ux, uy) = rooms[0].c();"""),
    ("""    /// lantern posts along the roads and at the exits
    pub posts: Vec<(f32, f32)>,
}""", """    /// lantern posts along the roads and at the exits
    pub posts: Vec<(f32, f32)>,
    /// cracked walls hiding a secret
    pub cracks: Vec<(f32, f32)>,
}"""),
    ("merchant: None, maren: None, posts: vec![] }", "merchant: None, maren: None, posts: vec![], cracks: vec![] }"),
])

G = 'src/game.rs'
edit(G, [
    ("    pub posts: Vec<(f32, f32)>,\n", "    pub posts: Vec<(f32, f32)>,\n    pub cracks: Vec<(f32, f32)>,\n"),
    ("            posts: vec![],\n", "            posts: vec![],\n            cracks: vec![],\n"),
    ("""                match m {
                    BRICK | STONE if d < r * 0.42 && rnd() < 0.8 => {""", """                if self.world.aux[i] & AUX_CRACK != 0 && is_wall(m) && d < r + 4.0 {
                    // a cracked wall gives way to any blast
                    self.world.set(cx, cy, if rnd() < 0.15 { RUBBLE } else { E });
                    self.world.aux[i] &= !AUX_CRACK;
                    self.secret_found(cx as f32, cy as f32);
                    continue;
                }
                match m {
                    BRICK | STONE if d < r * 0.42 && rnd() < 0.8 => {"""),
])
edit('src/areas.rs', [("        self.posts = sp.posts.clone();\n", "        self.posts = sp.posts.clone();\n        self.cracks = sp.cracks.clone();\n")])
edit('src/gear.rs', [("        self.hit_barrels(px, py, aim, ws.reach, ws.arc);\n", "        self.hit_barrels(px, py, aim, ws.reach, ws.arc);\n        self.hit_cracks(px, py, aim, ws.reach);\n")])

s = open('src/props.rs', encoding='utf8').read()
s += '''
impl Game {
    /// a swing that meets a cracked wall knocks a hole in it
    pub fn hit_cracks(&mut self, px: f32, py: f32, aim: f32, reach: f32) {
        use crate::world::*;
        let (ux, uy) = (aim.cos(), aim.sin());
        let mut hit = None;
        let mut d = 4.0;
        while d < reach + 6.0 {
            let (x, y) = ((px + ux * d) as i32, (py + uy * d * 0.8) as i32);
            if World::inb(x, y) && self.world.aux[y as usize * W + x as usize] & AUX_CRACK != 0 && is_wall(self.world.at(x, y)) {
                hit = Some((x, y));
                break;
            }
            d += 2.0;
        }
        let Some((hx, hy)) = hit else { return };
        for dy in -5i32..=5 {
            for dx in -7i32..=7 {
                let (x, y) = (hx + dx, hy + dy);
                if dx * dx + dy * dy * 2 > 50 || !World::inb(x, y) {
                    continue;
                }
                let i = y as usize * W + x as usize;
                if self.world.aux[i] & AUX_CRACK != 0 && is_wall(self.world.m[i]) {
                    self.world.set(x, y, if rnd() < 0.15 { RUBBLE } else { E });
                    self.world.aux[i] &= !AUX_CRACK;
                }
            }
        }
        for _ in 0..16 {
            self.ptc(hx as f32, hy as f32, rr(4.0, 30.0), rr(-1.5, 1.5), rr(-0.5, 1.5), rr(0.5, 2.0), 200, PtK::Debris, [92, 82, 96]);
        }
        self.snd(Sfx::Crumble, hx as f32, hy as f32, 0.9);
        self.shake = self.shake.max(2.5);
        self.secret_found(hx as f32, hy as f32);
    }

    /// the first breach in a cracked wall
    pub fn secret_found(&mut self, x: f32, y: f32) {
        if let Some(k) = self.cracks.iter().position(|c| (c.0 - x).abs() < 30.0 && (c.1 - y).abs() < 20.0) {
            self.cracks.swap_remove(k);
            self.float(x, y - 30.0, "A HIDDEN ROOM!".into(), [255, 214, 90], true);
            self.snd(Sfx::Chime, x, y, 0.8);
        }
    }
}
'''
open('src/props.rs', 'w', encoding='utf8').write(s)

R = 'src/render.rs'
edit(R, [("""                            if st == 0 {
                                let row = k / 4;
                                let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                                let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                                scl(base, 1.0 - k as f32 / h as f32 * 0.35)""", """                            if st == 0 {
                                let row = k / 4;
                                let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                                let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                                if w.aux[i] & AUX_CRACK != 0 {
                                    // a cracked section: lighter, crumbling, split by dark fissures
                                    let fis = (wx * 3 + k * 2).rem_euclid(11) == 0 || (wx * 5 - k * 3).rem_euclid(13) == 0;
                                    let col = if fis { [24, 20, 26] } else { mixc(base, [128, 112, 100], 0.25) };
                                    scl(col, 1.0 - k as f32 / h as f32 * 0.3)
                                } else {
                                    scl(base, 1.0 - k as f32 / h as f32 * 0.35)
                                }"""),
])
# a little dust falling from cracked walls, as a hint
edit('src/ambient.rs', [("""            AreaId::Crypt(_) | AreaId::Cellars(_) => {""", """            AreaId::Crypt(_) | AreaId::Cellars(_) => {
                if f % 20 == 0 {
                    for &(x, y) in self.cracks.clone().iter() {
                        if (x - cx - vw / 2.0).abs() < vw && (y - cy - vh / 2.0).abs() < vh {
                            self.ptc(x + rr(-8.0, 8.0), y + 2.0, rr(20.0, 40.0), 0.0, 0.0, -0.2, 70, PtK::Debris, [120, 110, 100]);
                        }
                    }
                }"""),
])
print("secrets wired")
