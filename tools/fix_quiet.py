# Quiet scenes and a town that remembers (slice step 6).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod quiet;\n")])
edit('src/quiet.rs', [("""                self.lamps.push((552.0 + a.cos() * 120.0, 400.0 + a.sin() * 60.0, cols[k % 4]));""", """                let (lx, ly) = (552.0 + a.cos() * 120.0, 400.0 + a.sin() * 60.0);
                if !self.world.solid(lx as i32, ly as i32) {
                    self.posts.push((lx, ly));
                    self.lamps.push((lx + 4.0, ly - 20.0, cols[k % 4]));
                }""")])

A = 'src/areagen.rs'
edit(A, [
    ("""    /// a hand-built encounter announced when you come near
    pub setpiece: Option<(f32, f32)>,
}""", """    /// a hand-built encounter announced when you come near
    pub setpiece: Option<(f32, f32)>,
    /// quiet places: (x, y, kind) - see quiet.rs
    pub lore: Vec<(f32, f32, u8)>,
}"""),
    ("posts: vec![], cracks: vec![], setpiece: None }", "posts: vec![], cracks: vec![], setpiece: None, lore: vec![] }"),
    ("""    let sp = match id {
        AreaId::Ashford => town(&mut w, s as u32),""", """    let mut sp = match id {
        AreaId::Ashford => town(&mut w, s as u32),"""),
    ("""        AreaId::Cellars(f) => dungeon(&mut w, f, true),
    };
    w.wake_all();""", """        AreaId::Cellars(f) => dungeon(&mut w, f, true),
    };
    quiet_scene(&mut w, &mut sp, id);
    w.wake_all();"""),
    ("""// ------------------------------------------------------------------ helpers
fn idx(""", """/// a quiet place off the beaten track, one per outdoor area that has one (see quiet.rs)
fn quiet_scene(w: &mut World, sp: &mut Spawns, id: AreaId) {
    let kind = match id {
        AreaId::Pines => 0u8,
        AreaId::Fen => 1,
        AreaId::Graveyard => 2,
        _ => return,
    };
    let (aw, ah) = (w.aw as f32, w.ah as f32);
    let mut spot = None;
    for t in 0..1500 {
        let (x, y) = if kind == 1 && t < 900 {
            // the overlook sits near the top or bottom edge
            (rr(160.0, aw - 160.0), if rnd() < 0.5 { rr(110.0, 200.0) } else { rr(ah - 200.0, ah - 110.0) })
        } else {
            (rr(140.0, aw - 140.0), rr(140.0, ah - 140.0))
        };
        let far = |p: (f32, f32), d: f32| ((p.0 - x).powi(2) + (p.1 - y).powi(2)).sqrt() > d;
        if !far(sp.start, 260.0) || !sp.exits.iter().all(|e| far((((e.x0 + e.x1) / 2) as f32, ((e.y0 + e.y1) / 2) as f32), 220.0)) {
            continue;
        }
        if let Some((sx, sy)) = sp.waypoint {
            if !far((sx, sy), 160.0) {
                continue;
            }
        }
        // dry, open ground, off the road, clear of buildings
        let ok = (-36..=36).step_by(4).all(|dy: i32| {
            (-44..=44).step_by(4).all(|dx: i32| {
                let (px, py) = (x as i32 + dx, y as i32 + dy);
                if !inarea(w, px, py) {
                    return false;
                }
                let i = idx(px, py);
                !is_liquid(w.m[i]) && !matches!(w.m[i], BRICK | STONE) && !matches!(w.g[i], ROAD | FLOOR | PLANK | STAIRS | STAIRSUP)
            })
        });
        if ok {
            spot = Some((x as i32, y as i32));
            break;
        }
    }
    let Some((cx, cy)) = spot else { return };
    // clear the trees and rocks
    for dy in -30..=30 {
        for dx in -38..=38 {
            if (dx * dx) as f32 / 1444.0 + (dy * dy) as f32 / 900.0 < 1.0 {
                let i = idx(cx + dx, cy + dy);
                if matches!(w.m[i], TREE | ROCK | WOOD | GAS) {
                    w.m[i] = E;
                    w.aux[i] &= !AUX_TRUNK;
                }
            }
        }
    }
    match kind {
        0 => {
            // a burned camp: scorched ground, ash, a collapsed tent, a dead fire, bones
            for dy in -24..=24 {
                for dx in -30..=30 {
                    let d = (dx * dx) as f32 / 900.0 + (dy * dy) as f32 / 576.0;
                    if d < 1.0 {
                        let i = idx(cx + dx, cy + dy);
                        w.g[i] = BURNT;
                        if rnd() < 0.08 * (1.0 - d) {
                            w.m[i] = ASH;
                            w.col[i] = [70, 66, 62];
                        }
                    }
                }
            }
            for ty in 0..12 {
                for tx in -(ty / 2 + 3)..=(ty / 2 + 3) {
                    if rnd() < 0.7 {
                        let i = idx(cx - 18 + tx, cy - 14 + ty);
                        w.m[i] = CLOTH;
                        w.col[i] = [52, 40, 36];
                    }
                }
            }
            for k in 0..10 {
                let a = k as f32 / 10.0 * std::f32::consts::TAU;
                let i = idx(cx + 10 + (a.cos() * 4.0) as i32, cy + 4 + (a.sin() * 3.0) as i32);
                w.m[i] = RUBBLE;
            }
            for _ in 0..10 {
                let i = idx(cx + rr(-12.0, 12.0) as i32, cy + rr(4.0, 14.0) as i32);
                w.m[i] = BONE;
                w.col[i] = [196, 188, 166];
            }
            barrels(w, sp, cx as f32 + 22.0, cy as f32 - 10.0, 1, &[(4, 1.0)]);
        }
        1 => {
            // an old stone overlook ringed with boulders, a lamp left burning
            for dy in -20..=20 {
                for dx in -28..=28 {
                    let d = (dx * dx) as f32 / 784.0 + (dy * dy) as f32 / 400.0;
                    let i = idx(cx + dx, cy + dy);
                    if d < 1.0 {
                        w.g[i] = FLOOR;
                    } else if d < 1.35 && (dx + dy * 3).rem_euclid(9) < 3 && dy.abs() > 6 {
                        w.m[i] = ROCK;
                    }
                }
            }
            sp.lamps.push((cx as f32, cy as f32 - 4.0, [1.0, 0.75, 0.45]));
            sp.posts.push((cx as f32 + 16.0, cy as f32 - 8.0));
            // nothing dead comes up here
            sp.enemies.retain(|e| ((e.1 - cx as f32).powi(2) + (e.2 - cy as f32).powi(2)).sqrt() > 160.0);
        }
        _ => {
            // a pen of stakes with a villager inside, and the dead nosing about it
            for dy in -14..=14 {
                for dx in -14..=14 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    let i = idx(cx + dx, cy + dy);
                    if d < 10.0 {
                        w.m[i] = E;
                        w.g[i] = DIRT;
                    } else if d < 12.5 {
                        w.m[i] = WOOD;
                        w.col[i] = if (dx + dy) % 3 == 0 { [84, 60, 38] } else { [118, 84, 52] };
                    }
                }
            }
            for k in 0..3 {
                let a = k as f32 * 2.1;
                sp.enemies.push((Kind::Zombie, cx as f32 + a.cos() * 34.0, cy as f32 + a.sin() * 24.0));
            }
        }
    }
    sp.lore.push((cx as f32, cy as f32, kind));
}

// ------------------------------------------------------------------ helpers
fn idx("""),
])

G = 'src/game.rs'
edit(G, [
    ("    pub setpiece: Option<(f32, f32)>,\n", "    pub setpiece: Option<(f32, f32)>,\n    /// quiet places in this area, which ones you've found (bits), and who is talking\n    pub lore: Vec<(f32, f32, u8)>,\n    pub lore_done: u8,\n    pub talk_name: Option<String>,\n"),
    ("            setpiece: None,\n", "            setpiece: None,\n            lore: vec![],\n            lore_done: 0,\n            talk_name: None,\n"),
])
AR = 'src/areas.rs'
edit(AR, [
    ("        self.setpiece = sp.setpiece;\n", "        self.setpiece = sp.setpiece;\n        self.lore = sp.lore.clone();\n"),
    ("""        self.npcs = match sp.maren {
            Some(m) => crate::story::town_folk(m, &self.world),
            None => vec![],
        };""", """        self.npcs = match sp.maren {
            Some(m) => crate::story::town_folk(m, &self.world),
            None => vec![],
        };
        self.place_quiet();"""),
    ("""        if self.update_props(press) {
            return;
        }""", """        if self.update_lore(press) {
            return;
        }
        if self.update_props(press) {
            return;
        }"""),
    ('            format!("pick {}", self.pick),\n', '            format!("pick {}", self.pick),\n            format!("lore {}", self.lore_done),\n'),
    ('                "pick" => self.pick = v.parse().unwrap_or(0),', '                "pick" => self.pick = v.parse().unwrap_or(0),\n                "lore" => self.lore_done = v.parse().unwrap_or(0),'),
])
ST = 'src/story.rs'
edit(ST, [
    ("const BARKS: [&str; 10] = [", "pub const BARKS: [&str; 10] = ["),
    ("""        let told_chief = self.has(TOLD_CHIEF);
        let Some((pages, page)) = &mut self.talk else { return };
        if close || (next && *page + 1 >= pages.len()) {
            let generic = pages.len() == 1;""", """        let told_chief = self.has(TOLD_CHIEF);
        let lore_talk = self.talk_name.is_some();
        let Some((pages, page)) = &mut self.talk else { return };
        if (close || (next && *page + 1 >= pages.len())) && lore_talk {
            self.talk = None;
            self.talk_name = None;
            self.hold_block = true;
            return;
        }
        if close || (next && *page + 1 >= pages.len()) {
            let generic = pages.len() == 1;"""),
    ("""            if n.k == 0 {
                n.face = if px < n.x { -1.0 } else { 1.0 };
                continue;
            }""", """            if n.k == 0 || n.k == crate::quiet::TOMAS {
                n.face = if px < n.x { -1.0 } else { 1.0 };
                if n.k == crate::quiet::TOMAS && near < 34.0 && n.bark == 0 && self.area == AreaId::Ashford {
                    n.bark = 900;
                    let (x, y) = (n.x, n.y);
                    let line = self.bark_for(crate::quiet::TOMAS, i);
                    self.float(x, y - 44.0, line.into(), [220, 210, 190], false);
                }
                continue;
            }"""),
    ("""                let line = BARKS[(i * 3 + (self.frame / 600) as usize) % BARKS.len()];""", """                let nk = self.npcs[i].k;
                let line = self.bark_for(nk, i);"""),
])
H = 'src/hud.rs'
edit(H, [
    ("""    // a little portrait: Maren's lantern-lit face
    u.rect(x + 8, y + 10, 26, 30, [40, 30, 34], 255);""", """    if let Some(name) = &g.talk_name {
        // a note, a place, a stranger: just the words
        u.text(x + 12, y + 7, name, [240, 214, 150], 1, true);
        let maxc = ((w - 24) / 4) as usize;
        if let Some(txt) = pages.get(*page) {
            for (k, l) in wrap(txt, maxc).iter().enumerate() {
                u.text(x + 12, y + 20 + k as i32 * 9, l, [230, 222, 210], 1, false);
            }
        }
        let more = if *page + 1 < pages.len() { format!("E  NEXT  ({}/{})", page + 1, pages.len()) } else { "E  CLOSE".into() };
        u.text(x + w - Ui::text_w(&more, 1) - 8, y + h - 10, &more, [255, 214, 90], 1, false);
        return;
    }
    // a little portrait: Maren's lantern-lit face
    u.rect(x + 8, y + 10, 26, 30, [40, 30, 34], 255);"""),
])
print("quiet ok")
