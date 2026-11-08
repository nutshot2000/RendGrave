def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("    loot: bool,\n    /// steps taken", "    loot: bool,\n    menus: bool,\n    /// steps taken"),
    ("Showcase { dir, log: vec![], lab, spells, tour, loot, sub: 0 }", "Showcase { dir, log: vec![], lab, spells, tour, loot, menus, sub: 0 }"),
    ("    let loot = args.iter().any(|a| a == \"--loot\");\n", "    let loot = args.iter().any(|a| a == \"--loot\");\n    let menus = args.iter().any(|a| a == \"--menus\");\n"),
    ("a == \"--showcase\" || a == \"--lab\" || a == \"--spells\" || a == \"--tour\" || a == \"--loot\")", "a == \"--showcase\" || a == \"--lab\" || a == \"--spells\" || a == \"--tour\" || a == \"--loot\" || a == \"--menus\")"),
    ("""    fn step_showcase(&mut self) -> Option<String> {
        if self.show.as_ref().map(|s| s.loot).unwrap_or(false) {""", """    /// the title screen, the pause menu and the end-of-act card
    fn step_menus(&mut self) -> Option<String> {
        let sh = self.show.as_mut().unwrap();
        sh.sub += 1;
        let k = sh.sub;
        match k {
            1 => self.game = title_game(),
            80 => return Some("m1_title".into()),
            81 => {
                let mut g = game::Game::new(7);
                g.saving = false;
                g.travel(areagen::AreaId::Ashford);
                g.banner = None;
                g.paused = true;
                g.menu_hover = Some(1);
                self.game = g;
            }
            140 => return Some("m2_pause".into()),
            141 => {
                let g = &mut self.game;
                g.paused = false;
                g.ending = -1;
                g.player.kills = 412;
                g.player.gold = 1830;
                g.player.sk.level = 17;
                g.played = 60 * 60 * 60 * 3 + 60 * 60 * 24;
            }
            146 => return Some("m3_ending".into()),
            _ => {}
        }
        None
    }
    fn step_showcase(&mut self) -> Option<String> {
        if self.show.as_ref().map(|s| s.menus).unwrap_or(false) {
            return self.step_menus();
        }
        if self.show.as_ref().map(|s| s.loot).unwrap_or(false) {"""),
    ("""            } else if sh.tour {""", """            } else if sh.menus {
                shot.as_deref() == Some("m3_ending")
            } else if sh.tour {"""),
])
print("menu test ok")
