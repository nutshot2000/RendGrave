def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


M = 'src/main.rs'
edit(M, [
    ("mod body;\n", "mod areas;\nmod body;\nmod gen;\n"),
    ("    lab: bool,\n    spells: bool,\n}", "    lab: bool,\n    spells: bool,\n    tour: bool,\n}"),
    ("            WindowEvent::CloseRequested => el.exit(),", "            WindowEvent::CloseRequested => {\n                self.game.save();\n                el.exit();\n            }"),
    ("                        KeyCode::KeyR => i.restart = down,", "                        KeyCode::KeyR => i.restart = down,\n                        KeyCode::KeyE => i.interact = down,"),
    # showcase: no fixed coordinates any more
    ("""            g.player.x = 214.0;
            g.player.y = 606.0;
            g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);
            // a puddle for the lightning to run through
            g.world.splash(250.0, 604.0, world::WATER, 9.0, 0);
        }
        if f == 600 {
            g.player.x = 745.0;
            g.player.y = 300.0;
            g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);""", """            place_near_enemy(g, 70.0);
            let (x, y) = (g.player.x + 36.0, g.player.y);
            g.world.splash(x, y, world::WATER, 9.0, 0);
        }
        if f == 600 {
            g.travel(gen::AreaId::Crypt(1));
            place_near_enemy(g, 60.0);"""),
    ("""    /// every new spell, cast at a lineup of enemies, plus the skill tree screen""", """    /// visit every area and take a picture of each
    fn step_tour(&mut self) -> Option<String> {
        use gen::AreaId;
        let g = &mut self.game;
        let f = g.frame as i32;
        g.player.hp = g.player.maxhp;
        let stops = [(AreaId::Ashford, "t1_ashford"), (AreaId::Pines, "t2_pines"), (AreaId::Graveyard, "t3_graveyard"), (AreaId::Crypt(1), "t4_crypt1"), (AreaId::Crypt(2), "t5_crypt2"), (AreaId::Crypt(3), "t6_crypt3_boss")];
        for (k, (a, name)) in stops.iter().enumerate() {
            let at = 1 + k as i32 * 70;
            if f == at {
                g.travel(*a);
                if !g.discovered.contains(a) {
                    g.discovered.push(*a);
                }
                for e in g.enemies.iter_mut() {
                    e.stun = 100000;
                }
                if *a == AreaId::Crypt(3) {
                    if let Some(b) = g.enemies.iter().position(|e| e.k == body::Kind::King) {
                        let (bx, by) = (g.enemies[b].x, g.enemies[b].y);
                        g.player.x = bx - 70.0;
                        g.player.y = by + 10.0;
                        g.enemies[b].aggro = true;
                    }
                } else if *a != AreaId::Ashford {
                    place_near_enemy(g, 80.0);
                }
                g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);
            }
            if f == at + 55 {
                return Some(name.to_string());
            }
        }
        if f == 1 + stops.len() as i32 * 70 {
            g.travel(AreaId::Pines);
            if let Some((wx, wy)) = g.waypoint {
                g.player.x = wx + 6.0;
                g.player.y = wy + 8.0;
            }
            g.wp_open = true;
        }
        if f == 1 + stops.len() as i32 * 70 && g.wp_open {
            return Some("t7_waypoints".into());
        }
        None
    }
    /// every new spell, cast at a lineup of enemies, plus the skill tree screen"""),
    ("""        if self.show.as_ref().map(|s| s.spells).unwrap_or(false) {
            return self.step_spells();
        }""", """        if self.show.as_ref().map(|s| s.tour).unwrap_or(false) {
            return self.step_tour();
        }
        if self.show.as_ref().map(|s| s.spells).unwrap_or(false) {
            return self.step_spells();
        }"""),
    ("""            if (sh.spells && shot.as_deref() == Some("s10_tree")) || (!sh.spells && self.game.frame > if sh.lab { 330 } else { 900 }) {""",
     """            let done = if sh.tour {
                shot.as_deref() == Some("t7_waypoints")
            } else if sh.spells {
                shot.as_deref() == Some("s10_tree")
            } else {
                self.game.frame > if sh.lab { 330 } else { 900 }
            };
            if done {"""),
    ("""    let spells = args.iter().any(|a| a == "--spells");
    let show = args.iter().position(|a| a == "--showcase" || a == "--lab" || a == "--spells").map(|i| {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "shots".into());
        std::fs::create_dir_all(&dir).ok();
        Showcase { dir, log: vec![], lab, spells }
    });""", """    let spells = args.iter().any(|a| a == "--spells");
    let tour = args.iter().any(|a| a == "--tour");
    let fresh = args.iter().any(|a| a == "--new");
    let show = args.iter().position(|a| a == "--showcase" || a == "--lab" || a == "--spells" || a == "--tour").map(|i| {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "shots".into());
        std::fs::create_dir_all(&dir).ok();
        Showcase { dir, log: vec![], lab, spells, tour }
    });"""),
    ("""        game: game::Game::new(seed),""", """        game: if show.is_some() { game::Game::new(seed) } else { game::Game::start(!fresh) },"""),
])
s = open(M, encoding='utf8').read()
s += """
/// test helper: stand a little to the west of the first reachable enemy
fn place_near_enemy(g: &mut game::Game, d: f32) {
    for k in 0..g.enemies.len() {
        let (ex, ey) = (g.enemies[k].x, g.enemies[k].y);
        for off in [d, d * 0.7, -d, -d * 0.7] {
            let (x, y) = (ex - off, ey);
            if !g.world.solid(x as i32, y as i32) && !g.world.solid(x as i32 + 3, y as i32) && !g.world.solid(x as i32 - 3, y as i32) && g.world.at(x as i32, y as i32) == world::E {
                g.player.x = x;
                g.player.y = y;
                g.cam = (x - g.view.0 / 2.0, y - 26.0 - g.view.1 / 2.0);
                return;
            }
        }
    }
}
"""
open(M, 'w', encoding='utf8').write(s)
print("main ok")
