def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("    loot: bool,\n}\n", "    loot: bool,\n    /// steps taken while the game is paused (the skill tree stops the clock)\n    sub: i32,\n}\n"),
    ("Showcase { dir, log: vec![], lab, spells, tour, loot }", "Showcase { dir, log: vec![], lab, spells, tour, loot, sub: 0 }"),
    ("""            g.player.sk.points = 60;
            g.player.sk.tomes = [true; N];
            for i in 0..N {
                while g.player.sk.learn(i) {}
            }""", """            g.player.sk.points = 400;
            g.player.sk.tomes = [true; N];
            // several passes: some skills need ones later in the list
            for _ in 0..4 {
                for i in 0..N {
                    while g.player.sk.learn(i) {}
                }
            }"""),
    ('shot.as_deref() == Some("s10_tree")', 'shot.as_deref() == Some("s19_fury")'),
    ("""        let at = |g: &game::Game, i: usize| (g.enemies[i].x, g.enemies[i].y);
        let mut cast = |g: &mut game::Game, s: Sk, x: f32, y: f32| {""", """        // with the tree open the clock stops: page through every school
        if g.tree && f == 699 {
            let sh = self.show.as_mut().unwrap();
            sh.sub += 1;
            let k = sh.sub;
            let pages = [(0, "s11_tab_storm"), (2, "s12_tab_venom"), (3, "s13_tab_frost"), (4, "s14_tab_blade"), (5, "s15_tab_body")];
            for (n, (tab, name)) in pages.iter().enumerate() {
                let at = 4 + n as i32 * 4;
                if k == at {
                    g.tree_tab = *tab;
                    g.tree_hover = None;
                }
                if k == at + 2 {
                    return Some(name.to_string());
                }
            }
            if k == 30 {
                g.tree = false;
            }
            return None;
        }
        if f == 705 {
            // a fresh crowd for the new skills
            g.enemies.clear();
            g.fields.clear();
            for k in 0..8 {
                let mut e = game::Game::new_enemy(Kind::Zombie, 170.0 + (k % 4) as f32 * 30.0, 645.0 + (k / 4) as f32 * 26.0);
                e.stun = 100000;
                e.hp = 400.0;
                e.maxhp = 400.0;
                e.burning = 300;
                g.enemies.push(e);
            }
            g.player.x = 90.0;
            g.player.y = 660.0;
        }
        let at = |g: &game::Game, i: usize| (g.enemies[i].x, g.enemies[i].y);
        let mut cast = |g: &mut game::Game, s: Sk, x: f32, y: f32| {"""),
    ("""        match f {
            20 => {
                let (x, y) = at(g, 3);
                cast(g, Sk::Meteor, x, y);
            }""", """        match f {
            712 => cast(g, Sk::Fireball, 220.0, 655.0),
            740 => cast(g, Sk::Combustion, 0.0, 0.0),
            770 => cast(g, Sk::PoisonCloud, 230.0, 660.0),
            820 => {
                cast(g, Sk::StormShield, 0.0, 0.0);
                cast(g, Sk::IceArmor, 0.0, 0.0);
            }
            840 => g.blast(260.0, 670.0, 20.0),
            856 => cast(g, Sk::RotBurst, 260.0, 670.0),
            875 => {
                g.player.x = 200.0;
                g.player.y = 665.0;
                cast(g, Sk::BladeFury, 0.0, 0.0);
            }
            20 => {
                let (x, y) = at(g, 3);
                cast(g, Sk::Meteor, x, y);
            }"""),
    ("""let shots = [(66, "s01_meteor_spikes"),""", """let shots = [(716, "s16_splinter"), (745, "s17_combustion"), (815, "s18_poison"), (826, "s18b_shields"), (860, "s18c_rotburst"), (905, "s19_fury"), (66, "s01_meteor_spikes"),"""),
])
print("talent test ok")
