def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("    menus: bool,\n    /// steps taken", "    menus: bool,\n    extras: bool,\n    /// steps taken"),
    ("Showcase { dir, log: vec![], lab, spells, tour, loot, menus, sub: 0 }", "Showcase { dir, log: vec![], lab, spells, tour, loot, menus, extras, sub: 0 }"),
    ("    let menus = args.iter().any(|a| a == \"--menus\");\n", "    let menus = args.iter().any(|a| a == \"--menus\");\n    let extras = args.iter().any(|a| a == \"--extras\");\n"),
    ("|| a == \"--loot\" || a == \"--menus\")", "|| a == \"--loot\" || a == \"--menus\" || a == \"--extras\")"),
    ("""    fn step_showcase(&mut self) -> Option<String> {
        if self.show.as_ref().map(|s| s.menus).unwrap_or(false) {""", """    /// secret walls, urns, champion types, the grave robber and kill streaks
    fn step_extras(&mut self) -> Option<String> {
        use body::Kind;
        let g = &mut self.game;
        let f = g.frame as i32;
        g.player.hp = g.player.maxhp;
        let cam = |g: &mut game::Game| g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);
        match f {
            1 => {
                for _ in 0..20 {
                    g.travel(areagen::AreaId::Crypt(2));
                    if !g.cracks.is_empty() {
                        break;
                    }
                }
                g.banner = None;
                for e in g.enemies.iter_mut() {
                    e.stun = 100000;
                    e.aggro = false;
                }
                if let Some(&(x, y)) = g.cracks.first() {
                    g.player.x = x;
                    g.player.y = y + 30.0;
                }
                cam(g);
            }
            61 | 75 => {
                let (px, py) = (g.player.x, g.player.y);
                g.hit_cracks(px, py, -std::f32::consts::FRAC_PI_2, 40.0);
            }
            91 => {
                if let Some(b) = g.barrels.iter().position(|b| b.alive && b.k == 3) {
                    let (x, y) = (g.barrels[b].x, g.barrels[b].y);
                    g.player.x = x - 20.0;
                    g.player.y = y + 14.0;
                    cam(g);
                    let near: Vec<usize> = (0..g.barrels.len()).filter(|&i| g.barrels[i].alive && (g.barrels[i].x - x).abs() < 30.0 && (g.barrels[i].y - y).abs() < 12.0).collect();
                    for i in near.into_iter().take(2) {
                        g.strike_barrel(i);
                    }
                }
            }
            121 => {
                g.travel(areagen::AreaId::Pines);
                g.banner = None;
                g.enemies.clear();
                let (px, py) = (g.player.x + 60.0, g.player.y);
                for k in 0..5u8 {
                    let mut e = g.scaled_enemy(Kind::Skeleton, px + k as f32 * 26.0, py + if k % 2 == 0 { 0.0 } else { 14.0 });
                    e.elite = true;
                    e.aff = k + 1;
                    e.stun = 100000;
                    e.hpbar = 200;
                    g.enemies.push(e);
                }
                cam(g);
                g.cam.0 += 60.0;
            }
            181 => {
                g.enemies.clear();
                let (px, py) = (g.player.x, g.player.y);
                let mut r = g.scaled_enemy(Kind::Robber, px + 70.0, py);
                r.skill_t = 25 * 60;
                g.enemies.push(r);
                for k in 0..6 {
                    let mut z = g.scaled_enemy(Kind::Zombie, px - 40.0 + k as f32 * 6.0, py + 30.0);
                    z.stun = 100000;
                    g.enemies.push(z);
                }
            }
            215 => {
                // six quick kills for a streak
                for e in g.enemies.iter_mut().filter(|e| e.k == Kind::Zombie) {
                    e.hp = -1.0;
                }
            }
            _ => {}
        }
        let shots = [(60, "x1_cracked_wall"), (110, "x2_breach_and_urns"), (175, "x3_champions"), (212, "x4_robber"), (222, "x5_streak")];
        shots.iter().find(|s| s.0 == f).map(|s| s.1.to_string())
    }
    fn step_showcase(&mut self) -> Option<String> {
        if self.show.as_ref().map(|s| s.extras).unwrap_or(false) {
            return self.step_extras();
        }
        if self.show.as_ref().map(|s| s.menus).unwrap_or(false) {"""),
    ("""            } else if sh.menus {""", """            } else if sh.extras {
                shot.as_deref() == Some("x5_streak")
            } else if sh.menus {"""),
])
print("extras test ok")
