# A portal opens in the boss room when a dungeon boss dies (and stays on later visits):
# step in and press E to pick any waypoint you've found. Waypoint travel lands at the stone.
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub waypoint: Option<(f32, f32)>,\n", "    pub waypoint: Option<(f32, f32)>,\n    /// the way out of a dungeon once its boss is dead\n    pub portal: Option<(f32, f32)>,\n    /// where this floor's boss stands, for the portal\n    pub boss_spot: Option<(f32, f32)>,\n    /// set while travelling by waypoint or portal: arrive at the stone\n    pub via_wp: bool,\n"),
    ("            waypoint: None,\n", "            waypoint: None,\n            portal: None,\n            boss_spot: None,\n            via_wp: false,\n"),
])

A = 'src/areas.rs'
edit(A, [
    ("""        let (king_dead, chief_dead) = (self.king_dead, self.chief_dead);
""", """        let (king_dead, chief_dead) = (self.king_dead, self.chief_dead);
        self.boss_spot = sp.enemies.iter().find(|e| e.0.boss()).map(|e| (e.1, e.2 + 10.0));
        let boss_gone = sp.enemies.iter().any(|e| (e.0 == Kind::King && king_dead) || (e.0 == Kind::Chief && chief_dead));
        self.portal = if boss_gone { self.boss_spot } else { None };
"""),
    ("""        let (x, y) = if from == to { sp.start } else { sp.arrival(from) };""",
     """        let (x, y) = match sp.waypoint {
            Some((wx, wy)) if self.via_wp => (wx + 6.0, wy + 16.0),
            _ if from == to => sp.start,
            _ => sp.arrival(from),
        };
        self.via_wp = false;"""),
    ("""        if let Some(m) = self.npcs.iter().find(|n| n.k == 0) {
            if ((self.player.x""", """        if let Some((qx, qy)) = self.portal {
            if ((self.player.x - qx).powi(2) + ((self.player.y - qy) * 1.4).powi(2)).sqrt() < 22.0 {
                self.prompt = Some("E  PORTAL - TRAVEL TO A WAYPOINT".into());
                if press {
                    self.snd(Sfx::Click, qx, qy, 0.8);
                    self.wp_open = true;
                    self.hold_block = true;
                }
                return;
            }
        }
        if let Some(m) = self.npcs.iter().find(|n| n.k == 0) {
            if ((self.player.x"""),
    ("""            if to != self.area {
                self.travel(to);
            }""", """            if to != self.area {
                self.via_wp = true;
                self.travel(to);
            }"""),
    ("""        self.flash = 0.2;
        self.save();
    }""", """        self.flash = 0.2;
        self.open_portal();
        self.save();
    }"""),
    ("""        self.flash = 0.25;
        self.save();
    }""", """        self.flash = 0.25;
        self.open_portal();
        self.save();
    }

    /// the boss is dead: a way home opens where it stood
    pub fn open_portal(&mut self) {
        let Some((x, y)) = self.boss_spot else { return };
        self.portal = Some((x, y));
        for _ in 0..60 {
            let a = rnd() * std::f32::consts::TAU;
            self.ptc(x, y, rr(5.0, 40.0), a.cos() * rr(0.5, 2.5), a.sin() * rr(0.3, 1.5), rr(0.5, 2.0), 80, PtK::Spark, [170, 120, 255]);
        }
        self.light(x, y - 20.0, 200.0, 2.5, [0.7, 0.5, 1.0], 60);
        self.toast = Some(("A PORTAL HOME HAS OPENED".into(), 300));
    }"""),
])

# the guide points at the portal when your next goal is elsewhere
edit('src/story.rs', [("""        let hop = next_hop(self.area, target)?;""", """        if let Some(p) = self.portal {
            return Some(p);
        }
        let hop = next_hop(self.area, target)?;""")])

R = 'src/render.rs'
edit(R, [
    ("""    // ---- pickups
    for p in &g.pickups {""", """    // ---- portal: a swirling violet gate where the boss fell
    if let Some((qx, qy)) = g.portal {
        let (sx, sy) = (qx as i32 - cx, qy as i32 - cy);
        let tf = t as f32;
        // rune ring on the floor
        for k in 0..70 {
            let a = k as f32 / 70.0 * std::f32::consts::TAU;
            let (x, y) = (sx + (a.cos() * 13.0) as i32, sy + (a.sin() * 6.0) as i32);
            c.glow(x, y, scl([150, 90, 255], 0.6 + (tf * 0.08 + a * 3.0).sin() * 0.3));
        }
        // the standing gate: an upright oval of spinning light
        for k in 0..140 {
            let a = k as f32 / 140.0 * std::f32::consts::TAU + tf * 0.05;
            let r = 0.55 + 0.45 * ((k as f32 * 0.37 + tf * 0.11).sin() * 0.5 + 0.5);
            let (x, y) = (sx + (a.cos() * 9.0 * r) as i32, sy - 18 + (a.sin() * 16.0 * r) as i32);
            let col = if k % 3 == 0 { [230, 200, 255] } else { [140, 80, 255] };
            c.put(x, y, qy + 0.5, scl(col, 0.5), scl(col, 0.9), (sy - y) as f32);
        }
    }
    // ---- pickups
    for p in &g.pickups {"""),
    ("""    if let Some((wx, wy)) = g.waypoint {
        add(wx, wy + 2.0, 90.0, 0.9, [0.45, 0.6, 1.0]);
    }""", """    if let Some((wx, wy)) = g.waypoint {
        add(wx, wy + 2.0, 90.0, 0.9, [0.45, 0.6, 1.0]);
    }
    if let Some((qx, qy)) = g.portal {
        add(qx, qy - 14.0, 130.0, 1.6 + (t as f32 * 0.07).sin() * 0.2, [0.65, 0.45, 1.0]);
    }"""),
])

# minimap: a violet dot for the portal
edit('src/hud.rs', [("""    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);""", """    if let Some((qx, qy)) = g.portal {
        let (x, y) = (ox + (qx / sx) as i32, oy + (qy / sy) as i32);
        for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
            u.px(x + dx, y + dy, [190, 130, 255], 255);
        }
    }
    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);""")])
print("portal ok")
