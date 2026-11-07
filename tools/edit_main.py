p = 'src/main.rs'
s = open(p, encoding='utf8').read()


def R(a, b, n=1):
    global s
    assert s.count(a) == n, (a[:80], s.count(a))
    s = s.replace(a, b)


R("mod rng;\n", "mod rng;\nmod skills;\nmod spells;\n")
R("""                        KeyCode::Digit4 if down => self.sel_pending = Some(3),
                        KeyCode::Escape if down => el.exit(),""", """                        KeyCode::Digit4 if down => self.sel_pending = Some(3),
                        KeyCode::Digit5 if down => self.sel_pending = Some(4),
                        KeyCode::Digit6 if down => self.sel_pending = Some(5),
                        KeyCode::Digit7 if down => self.sel_pending = Some(6),
                        KeyCode::Digit8 if down => self.sel_pending = Some(7),
                        KeyCode::Digit9 if down => self.sel_pending = Some(8),
                        KeyCode::KeyK | KeyCode::Tab => i.tree = down,
                        KeyCode::Escape => i.close = down,""")
R("""            WindowEvent::CursorMoved { position, .. } => {""", """            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                let n = self.game.player.sk.bar().len().max(1) as i32;
                let cur = self.game.player.sel as i32;
                let next = if dy < 0.0 { cur + 1 } else if dy > 0.0 { cur - 1 } else { cur };
                self.sel_pending = Some(next.rem_euclid(n) as usize);
            }
            WindowEvent::CursorMoved { position, .. } => {""")
R("""    lab: bool,
}""", """    lab: bool,
    spells: bool,
}""")
R("""        if self.show.as_ref().map(|s| s.lab).unwrap_or(false) {
            return self.step_lab();
        }""", """        if self.show.as_ref().map(|s| s.spells).unwrap_or(false) {
            return self.step_spells();
        }
        if self.show.as_ref().map(|s| s.lab).unwrap_or(false) {
            return self.step_lab();
        }""")
R("""    fn step_showcase(&mut self) -> Option<String> {""", """    /// every new spell, cast at a lineup of enemies, plus the skill tree screen
    fn step_spells(&mut self) -> Option<String> {
        use body::*;
        use skills::*;
        let g = &mut self.game;
        let f = g.frame;
        let inp = &mut self.input;
        inp.lmb = false;
        inp.rmb = false;
        inp.tree = false;
        if f == 1 {
            for y in 560..760 {
                for x in 20..420 {
                    g.world.set(x, y, world::E);
                    g.world.g[y as usize * world::W + x as usize] = world::GRASS;
                }
            }
            g.world.splash(250.0, 640.0, world::WATER, 12.0, 0);
            g.enemies.clear();
            g.player.x = 70.0;
            g.player.y = 660.0;
            g.player.sk.level = 7;
            g.player.sk.points = 30;
            for i in 0..N {
                while g.player.sk.learn(i) {}
            }
            g.recalc();
            let kinds = [Kind::Zombie, Kind::Skeleton, Kind::Zombie, Kind::Brute, Kind::Zombie, Kind::Skeleton, Kind::Zombie, Kind::Zombie];
            for (k, kind) in kinds.iter().enumerate() {
                let mut e = game::Game::new_enemy(*kind, 160.0 + (k % 4) as f32 * 34.0, 640.0 + (k / 4) as f32 * 30.0);
                e.stun = 100000;
                e.hp = 2000.0;
                e.maxhp = 2000.0;
                e.body.mirror = true;
                g.enemies.push(e);
            }
            g.banner = None;
        }
        g.cam = (40.0, 655.0 - 26.0 - g.view.1 / 2.0);
        g.player.hp = g.player.maxhp;
        g.player.mana = 999.0;
        let at = |g: &game::Game, i: usize| (g.enemies[i].x, g.enemies[i].y);
        let mut cast = |g: &mut game::Game, s: Sk, x: f32, y: f32| {
            let i2 = inp.clone();
            g.cast(s, x, y, &i2);
        };
        match f {
            20 => {
                let (x, y) = at(g, 3);
                cast(g, Sk::Meteor, x, y);
            }
            30 => cast(g, Sk::IceSpikes, 260.0, 640.0),
            110 => {
                let (x, y) = at(g, 0);
                cast(g, Sk::AcidRain, x + 10.0, y + 10.0)
            }
            200 => cast(g, Sk::FireWall, 300.0, 700.0),
            260 => cast(g, Sk::Blizzard, 220.0, 668.0),
            330 => cast(g, Sk::BallLightning, 400.0, 650.0),
            420 => cast(g, Sk::Thunderstorm, 0.0, 0.0),
            520 => cast(g, Sk::AcidFlood, 250.0, 690.0),
            600 => {
                g.player.x = 210.0;
                g.player.y = 655.0;
                cast(g, Sk::Whirlwind, 0.0, 0.0);
            }
            700 => {
                g.tree = true;
                g.tree_hover = Some(Sk::Meteor as usize);
            }
            _ => {}
        }
        if f > 700 {
            g.frame += 0;
        }
        let shots = [(66, "s01_meteor_spikes"), (80, "s02_meteor_impact"), (190, "s03_acid_rain"), (240, "s04_firewall"), (320, "s05_blizzard"), (360, "s06_ball"), (470, "s07_storm"), (570, "s08_flood"), (612, "s09_whirl"), (702, "s10_tree")];
        for (fr, name) in shots {
            if f == fr {
                return Some(name.to_string());
            }
        }
        None
    }
    fn step_showcase(&mut self) -> Option<String> {""")
R("""            if self.game.frame > if sh.lab { 330 } else { 900 } {""", """            if self.game.frame > if sh.spells { 702 } else if sh.lab { 330 } else { 900 } {""")
R("""    let lab = args.iter().any(|a| a == "--lab");
    let show = args.iter().position(|a| a == "--showcase" || a == "--lab").map(|i| {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "shots".into());
        std::fs::create_dir_all(&dir).ok();
        Showcase { dir, log: vec![], lab }
    });""", """    let lab = args.iter().any(|a| a == "--lab");
    let spells = args.iter().any(|a| a == "--spells");
    let show = args.iter().position(|a| a == "--showcase" || a == "--lab" || a == "--spells").map(|i| {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "shots".into());
        std::fs::create_dir_all(&dir).ok();
        Showcase { dir, log: vec![], lab, spells }
    });""")
open(p, 'w', encoding='utf8').write(s)
print("main ok")
