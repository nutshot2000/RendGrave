# Wire in the anatomy enemies (slice step 2).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod anatomy;\n")])

G = 'src/game.rs'
edit(G, [
    ("""    /// the one-armed King's grab: >0 winding up, <0 lunging
    pub grab: i32,
}""", """    /// the one-armed King's grab: >0 winding up, <0 lunging
    pub grab: i32,
    /// see anatomy.rs: F_PANIC, F_DROPPED, F_PUNCTURED
    pub flags: u8,
    /// a punctured bloater: frames of acid left to leak
    pub leak: i32,
    /// shield-bearers: what's left of the shield, its full strength, wood or iron,
    /// and the side it's held on (it turns slower than its bearer)
    pub shield: f32,
    pub shield_max: f32,
    pub shield_wood: bool,
    pub sface: f32,
    pub sturn: i32,
}"""),
    ("        wk_init: [0; 3],\n        grab: 0,\n", "        wk_init: [0; 3],\n        grab: 0,\n        flags: 0,\n        leak: 0,\n        shield: 0.0,\n        shield_max: 0.0,\n        shield_wood: false,\n        sface: 1.0,\n        sturn: 0,\n"),
    ("    pub part_hits: Vec<(usize, f32, f32)>,\n", "    pub part_hits: Vec<(usize, f32, f32)>,\n    /// crossbows dropped by crossbowmen who lost their firing arm, still loaded\n    pub loose_bows: Vec<crate::anatomy::LooseBow>,\n"),
    ("            part_hits: vec![],\n", "            part_hits: vec![],\n            loose_bows: vec![],\n"),
    ("""        for (i, h, d) in std::mem::take(&mut self.part_hits) {""", """        self.bows_tick();
        for (i, h, d) in std::mem::take(&mut self.part_hits) {"""),
    ("            self.champion_tick(i);\n", "            self.champion_tick(i);\n            self.anatomy_tick(i);\n"),
    ("""            let e = &mut self.enemies[i];
            // separation from other enemies""", """            if let Some(w) = self.panic_want(i, spd) {
                want = w;
            }
            let e = &mut self.enemies[i];
            // separation from other enemies"""),
    # a bloater doesn't melt in its own acid
    ("""            if u[1] {
                e.body.each_in(0.0, -1.0, 6.0, |p, _, h| {""", """            if u[1] && e.k != Kind::Bloater {
                e.body.each_in(0.0, -1.0, 6.0, |p, _, h| {"""),
    # what came off, for the crossbow
    ("""            let ds = e.body.split();
            let had = !ds.is_empty();""", """            let mut ds = e.body.split();
            let severed: Vec<PK> = ds.iter().map(|d| d.part.k).collect();
            if e.k == Kind::Crossbow && e.flags & crate::anatomy::F_DROPPED == 0 {
                // the crossbow lands on the ground, loaded, rather than flying off as a gib
                ds.retain(|d| d.part.k != PK::Weapon);
            }
            let had = !severed.is_empty();"""),
    ("""                let txt = "SEVERED".to_string();
                if big_piece {
                    self.float(ex, ey - 70.0, txt, [255, 120, 100], false);
                }
            }""", """                let txt = "SEVERED".to_string();
                if big_piece {
                    self.float(ex, ey - 70.0, txt, [255, 120, 100], false);
                }
                self.on_sever(i, &severed);
            }"""),
    # fireballs splash off a shield
    ("""                                self.hit_h = Some(-z);
                                self.hurt(i, d, [255, 170, 80], false);""", """                                if !self.shield_block(i, d, false) {
                                    self.hit_h = Some(-z);
                                    self.hurt(i, d, [255, 170, 80], false);
                                }"""),
])
s = open(G, encoding='utf8').read()
# explosions set off loose crossbows
a = "    pub fn explode(&mut self, x: f32, y: f32, z: f32, r: f32) {\n"
assert s.count(a) == 1
s = s.replace(a, a + "        self.bows_blast(x, y, r);\n")
open(G, 'w', encoding='utf8').write(s)

S = 'src/spells.rs'
edit(S, [
    ("""    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
""", """    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        self.puncture(i);
"""),
    ("""        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }
        // a boss's hide is thick""", """        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }
        if self.shield_block(i, dmg, false) {
            return;
        }
        // a boss's hide is thick"""),
])
edit('src/gear.rs', [
    ("""        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }
        let e = &mut self.enemies[i];
        let bony = e.k.bony();""", """        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }
        if self.shield_block(i, dmg, true) {
            return;
        }
        let e = &mut self.enemies[i];
        let bony = e.k.bony();"""),
    ("        self.hit_cracks(px, py, aim, ws.reach);\n", "        self.hit_cracks(px, py, aim, ws.reach);\n        self.hit_bows(px, py, aim, ws.reach, ws.arc);\n"),
])
edit('src/areas.rs', [
    ("        self.spikes.clear();\n", "        self.spikes.clear();\n        self.loose_bows.clear();\n"),
    # shield-bearers: crypt skeletons and Ravenhold bandits
    ("""        // now and then a champion: tougher, glowing, better loot""", """        // shield-bearers in the crypt and at Ravenhold
        let shields_here = match k {
            Kind::Skeleton => matches!(self.area, AreaId::Crypt(_)),
            Kind::Bandit => matches!(self.area, AreaId::Ravenhold | AreaId::Cellars(_)),
            _ => false,
        };
        if shields_here && rnd() < 0.25 {
            e.shield_wood = rnd() < 0.5;
            e.shield_max = (if e.shield_wood { 40.0 } else { 60.0 }) * (1.0 + 0.18 * (self.alvl - 1) as f32);
            e.shield = e.shield_max;
            e.speed *= 0.85;
            return e;
        }
        // now and then a champion: tougher, glowing, better loot"""),
])

R = 'src/render.rs'
edit(R, [
    ("""        draw_body(&mut c, &e.body, e.x + jitter, e.y, tint);
    }
    // ---- projectiles""", """        draw_body(&mut c, &e.body, e.x + jitter, e.y, tint);
        if e.shield > 0.0 {
            // the shield, held out on the side it faces; cracks show as it weakens
            let frac = e.shield / e.shield_max.max(1.0);
            let (bx, by) = ((e.x + jitter + e.sface * 7.0) as i32 - cx, e.y as i32 - cy);
            for k in 0..16i32 {
                let half = if k == 0 || k == 15 { 2 } else { 3 };
                for dx in -half..=half {
                    let rim = dx.abs() == half || k == 0 || k == 15;
                    let crack = frac < 0.6 && (dx * 7 + k * 3).rem_euclid(5) == 0 && k > 2 && k < 13;
                    let col = if crack {
                        [36, 28, 24]
                    } else if e.shield_wood {
                        if rim { [86, 88, 96] } else if dx == 0 || dx == 2 { [96, 66, 40] } else { [128, 88, 52] }
                    } else if rim {
                        [170, 174, 186]
                    } else if dx == 0 && (k - 8).abs() < 2 {
                        [200, 196, 170]
                    } else {
                        [118, 122, 134]
                    };
                    let sh = 0.8 + (dx + half) as f32 / (2 * half) as f32 * 0.3;
                    c.put(bx + dx, by - 18 - k, e.y + 3.0, scl(col, sh), [0; 3], (18 + k) as f32);
                }
            }
        }
    }
    // ---- loose crossbows: still loaded, glinting
    for b in &g.loose_bows {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);
        let f = if b.dir.cos() >= 0.0 { 1 } else { -1 };
        for k in -5..=4i32 {
            c.put(sx + k * f, sy - 2, b.y + 0.5, [104, 70, 42], [0; 3], 2.0);
        }
        for k in -3..=3i32 {
            c.put(sx + 4 * f, sy - 2 + k, b.y + 0.6, [150, 150, 160], [0; 3], 2.0 + k as f32);
        }
        let glint = ((t / 8) % 6 == 0) as u8 * 120;
        c.put(sx + 6 * f, sy - 2, b.y + 0.7, [220, 220, 230], [glint, glint, glint / 2], 2.0);
    }
    // ---- projectiles"""),
])

edit('src/story.rs', [
    ("""        let king = self.enemies.iter().any(|e| e.alive && e.k == crate::body::Kind::King && e.aggro);
        let tips: [(bool, &str); 5] = [""", """        let king = self.enemies.iter().any(|e| e.alive && e.k == crate::body::Kind::King && e.aggro);
        let seen = |f: &dyn Fn(&Enemy) -> bool| self.enemies.iter().any(|e| e.alive && e.aggro && f(e) && (e.x - p.x).abs() < 220.0 && (e.y - p.y).abs() < 140.0);
        let bloater = seen(&|e| e.k == crate::body::Kind::Bloater);
        let shield = seen(&|e| e.shield > 0.0);
        let tips: [(bool, &str); 7] = ["""),
    ("""            (king, "TIP: AIM AT THE KING'S KNEES, SWORD HAND OR CHEST TO BREAK THEM"),
        ];""", """            (king, "TIP: AIM AT THE KING'S KNEES, SWORD HAND OR CHEST TO BREAK THEM"),
            (bloater, "TIP: CUT A BLOATER AND IT LEAKS ACID - LEAD IT THROUGH ITS FRIENDS"),
            (shield, "TIP: SHIELDS BLOCK FROM THE FRONT - GET BEHIND, CUT THE SHIELD ARM OR BURN WOOD"),
        ];"""),
])
print("anatomy wired")
