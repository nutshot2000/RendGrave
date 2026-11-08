def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/body.rs', [
    ("""    pub fn detach_everything(&mut self) -> Vec<Detached> {""", """    /// tear off one part (and whatever hangs from it)
    pub fn detach_part(&mut self, i: usize) -> Vec<Detached> {
        let mut out = vec![];
        self.detach_all(i, &mut out);
        out
    }
    pub fn detach_everything(&mut self) -> Vec<Detached> {"""),
])

G = 'src/game.rs'
edit(G, [
    ("    fn spawn_gibs(&mut self,", "    pub(crate) fn spawn_gibs(&mut self,"),
    ("    fn on_kill(&mut self, i: usize) {", "    pub(crate) fn on_kill(&mut self, i: usize) {"),
    ("    pub shrines: Vec<crate::props::Shrine>,\n", "    pub shrines: Vec<crate::props::Shrine>,\n    pub barrels: Vec<crate::barrels::Barrel>,\n"),
    ("            shrines: vec![],\n", "            shrines: vec![],\n            barrels: vec![],\n"),
    ("        self.tick_buffs();\n", "        self.tick_buffs();\n        self.update_barrels();\n"),
    # slipping in oil
    ("    pub swing_max: i32,\n", "    pub swing_max: i32,\n    /// sliding helplessly on spilt oil\n    pub slip: i32,\n    pub slide: (f32, f32),\n"),
    ("            swing_max: 16,\n", "            swing_max: 16,\n            slip: 0,\n            slide: (0.0, 0.0),\n"),
    ("        summoned: false,\n        elite: false,\n", "        summoned: false,\n        elite: false,\n        slip: 0,\n"),
    ("    pub elite: bool,\n", "    pub elite: bool,\n    /// on its back after slipping in oil\n    pub slip: i32,\n"),
    ("""        if u[2] {
            speed *= 0.7;
            p.wet = 240;
        }""", """        if u[2] {
            speed *= 0.7;
            p.wet = 240;
        }
        if u[3] {
            speed *= 0.78;
            if l > 0.0 && p.dodge == 0 && p.slip == 0 && rnd() < 0.005 {
                p.slip = 34;
                p.slide = (dx * 2.6, dy * 2.6);
                self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.9 });
            }
        }"""),
    ("""        let (mut vx, mut vy) = (dx * speed, dy * speed);""", """        let (mut vx, mut vy) = (dx * speed, dy * speed);
        if p.slip > 0 {
            p.slip -= 1;
            let t = p.slip as f32 / 34.0;
            vx = p.slide.0 * t;
            vy = p.slide.1 * t;
        }"""),
    ("            let crawling = legs == 0;\n", """            if e.slip > 0 {
                e.slip -= 1;
            } else if u[3] && e.aggro && !e.k.big() && e.frozen == 0 && legs == 2 && rnd() < 0.012 {
                // feet go out from under it
                e.slip = 75;
                e.stun = e.stun.max(75);
                e.vx += e.face * 2.5;
                let (sx3, sy3) = (e.x, e.y);
                self.sounds.push(Snd { k: Sfx::Dodge, x: sx3, y: sy3, v: 0.7 });
            }
            let e = &mut self.enemies[i];
            let crawling = legs == 0 || e.slip > 0;
"""),
    ("let spd = e.speed * if legs == 2", "let spd = e.speed * if u[3] { 0.7 } else { 1.0 } * if legs == 2"),
])
edit('src/gear.rs', [
    ("""        let (mwx, mwy) = self.mouse_world(inp);
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32 + g.crit / 100.0;""", """        let (mwx, mwy) = self.mouse_world(inp);
        self.hit_barrels(px, py, aim, ws.reach, ws.arc);
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32 + g.crit / 100.0;"""),
])
edit('src/areas.rs', [
    ("        self.shrines = sp.shrines.iter()", "        self.barrels = sp.barrels.iter().map(|&(x, y, k)| crate::barrels::Barrel::new(x, y, k, &self.world)).collect();\n        self.shrines = sp.shrines.iter()"),
])
edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod barrels;\n")])
edit('src/mapdump.rs', [("        for s in &sp.shrines {", "        for b in &sp.barrels {\n            dot(b.0, b.1, 2, if b.2 == 0 { [255, 120, 0] } else if b.2 == 1 { [150, 100, 60] } else { [120, 255, 60] });\n        }\n        for s in &sp.shrines {")])
edit('src/render.rs', [
    ("""    // ---- waypoint: a rune circle that glows brighter once found""", """    // ---- barrels: powder kegs (painted cross), oil (dark, steel hoops), acid (green, glowing seam)
    for b in g.barrels.iter().filter(|b| b.alive) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);
        let (wood, hoop) = match b.k {
            0 => ([118u8, 74, 42], [40u8, 34, 30]),
            1 => ([62, 54, 50], [130, 130, 138]),
            _ => ([84, 110, 70], [120, 124, 120]),
        };
        let lit = b.fuse > 0 && (b.fuse / 4) % 2 == 0;
        for k in 0..15i32 {
            let bulge = 1.0 - ((k as f32 - 7.0) / 7.5).powi(2);
            let half = (3.5 + bulge * 1.8) as i32;
            for dx in -half..=half {
                let shade = 0.75 + (dx + half) as f32 / (2 * half) as f32 * 0.45;
                let hooped = k == 2 || k == 12 || (b.k != 0 && k == 7);
                let mut col = if hooped { hoop } else if dx.rem_euclid(3) == 0 { scl(wood, 0.8) } else { wood };
                col = scl(col, shade);
                let mut em = [0u8; 3];
                if b.k == 0 && !hooped && (5..=9).contains(&k) && (dx == k - 7 || dx == 7 - k) {
                    col = [200, 40, 30];
                }
                if b.k == 2 && dx == half - 1 && k > 3 && k < 11 {
                    col = [150, 230, 90];
                    em = [20, 60, 10];
                }
                if b.k == 1 && dx == 1 - half && k > 3 && k < 11 {
                    col = [150, 150, 160];
                }
                if lit {
                    em = [70, 26, 0];
                }
                c.put(sx + dx, sy - k, b.y + 0.5, col, em, k as f32);
            }
        }
        for dx in -4..=4 {
            c.put(sx + dx, sy - 15, b.y + 0.5, scl(wood, 1.15), [0; 3], 15.0);
        }
        if b.k == 0 {
            c.put(sx + 2, sy - 16, b.y + 0.5, [70, 60, 46], [0; 3], 16.0);
            c.put(sx + 3, sy - 17, b.y + 0.5, [70, 60, 46], [0; 3], 17.0);
            if b.fuse > 0 {
                c.glow(sx + 3, sy - 18, [255, 200, 80]);
            }
        }
    }
    // ---- waypoint: a rune circle that glows brighter once found"""),
])
print("barrels wire ok")
