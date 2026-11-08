def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub chief_dead: bool,\n", """    pub chief_dead: bool,
    /// the title screen is up (the town behind it is just scenery)
    pub title: bool,
    pub has_save: bool,
    pub menu_hover: Option<usize>,
    pub confirm_new: bool,
    /// something for main.rs to do (quit, fullscreen, audio, new game)
    pub req: Option<crate::menu::Req>,
    /// (music on, sound on, volume), kept up to date by main.rs for the menus
    pub audio_state: (bool, bool, f32),
    pub act1_done: bool,
    /// counts down to the end-of-act card; -1 while the card is showing
    pub ending: i32,
    /// frames played
    pub played: u64,
"""),
    ("            chief_dead: false,\n", """            chief_dead: false,
            title: false,
            has_save: false,
            menu_hover: None,
            confirm_new: false,
            req: None,
            audio_state: (true, true, 0.7),
            act1_done: false,
            ending: 0,
            played: 0,
"""),
    ("""    pub fn update(&mut self, inp: &Input) {
        if self.fade > 0 {
            self.fade -= 1;
        }
        if inp.tree && !self.prev.tree && !self.player.dead {""", """    pub fn update(&mut self, inp: &Input) {
        if self.fade > 0 {
            self.fade -= 1;
        }
        if self.title {
            // the town drifts by behind the title
            self.frame += 1;
            self.cam.0 = 380.0 + (self.frame as f32 * 0.002).sin() * 120.0;
            self.cam.1 = 250.0 + (self.frame as f32 * 0.0013).cos() * 40.0;
            self.world.update();
            self.update_menu(inp);
            self.prev = inp.clone();
            return;
        }
        if self.ending == -1 {
            if (inp.lmb && !self.prev.lmb) || (inp.close && !self.prev.close) || (inp.interact && !self.prev.interact) {
                self.ending = 0;
                self.hold_block = true;
            }
            self.prev = inp.clone();
            return;
        }
        if self.ending > 0 {
            self.ending -= 1;
            if self.ending == 0 {
                self.ending = -1;
            }
        }
        let menu_up = self.inv_open || self.shop_open || self.wp_open;
        if inp.tree && !self.prev.tree && !self.player.dead {"""),
    ("""            if self.tree {
                self.tree = false;
            } else if !self.player.dead {
                self.paused = !self.paused;
            }""", """            if self.tree {
                self.tree = false;
            } else if !self.player.dead && !menu_up {
                self.paused = !self.paused;
                self.menu_hover = None;
            }"""),
    ("""        if self.paused && !self.tree {
            self.prev = inp.clone();
            return;
        }""", """        if self.paused && !self.tree {
            self.update_menu(inp);
            self.prev = inp.clone();
            return;
        }"""),
    ("        self.frame += 1;\n        if self.hitstop > 0 {", "        self.frame += 1;\n        self.played += 1;\n        if self.hitstop > 0 {"),
])

A = 'src/areas.rs'
edit(A, [
    ("""            format!("chief {}", self.chief_dead as u8),""", """            format!("chief {}", self.chief_dead as u8),
            format!("act1 {}", self.act1_done as u8),
            format!("played {}", self.played),"""),
    ("""                "chief" => self.chief_dead = v == "1",""", """                "chief" => self.chief_dead = v == "1",
                "act1" => self.act1_done = v == "1",
                "played" => self.played = v.parse().unwrap_or(0),"""),
    ("""        self.flash = 0.2;
        self.save();
    }

    pub fn king_slain(&mut self, x: f32, y: f32) {""", """        self.flash = 0.2;
        self.save();
        self.check_act_end();
    }

    pub fn king_slain(&mut self, x: f32, y: f32) {"""),
])
s = open(A, encoding='utf8').read()
a = s.index('    pub fn king_slain(&mut self, x: f32, y: f32) {')
b = s.index('        self.save();\n    }\n', a)
s = s[:b] + '        self.save();\n        self.check_act_end();\n    }\n' + s[b + len('        self.save();\n    }\n'):]
open(A, 'w', encoding='utf8').write(s)

H = 'src/hud.rs'
s = open(H, encoding='utf8').read()
s += open('tools/menu_hud.rs', encoding='utf8').read()
a = """    let (fw, fh) = (f.w as i32, f.h as i32);
    let t = g.frame;
    let mut u = Ui { f };"""
assert s.count(a) == 1
s = s.replace(a, a + """
    if g.title {
        draw_title(g, &mut u, fw, fh);
        let (mx, my) = (inp.mouse.0 as i32, inp.mouse.1 as i32);
        for k in 2..5 {
            for &(dx, dy) in &[(k, 0), (-k, 0), (0, k), (0, -k)] {
                u.px(mx + dx, my + dy, [255, 240, 210], 255);
            }
        }
        if g.fade > 0 {
            u.rect(0, 0, fw, fh, [0, 0, 0], (g.fade.min(60) as f32 / 60.0 * 255.0) as u8);
        }
        return;
    }""")
a = """        u.text_c(fw / 2, fh / 3, "PAUSED", [240, 214, 150], 3);
        u.text_c(fw / 2, fh / 3 + 22, "ESC RESUME    K SKILL TREE    CLOSE THE WINDOW TO QUIT", [200, 190, 190], 1);
        u.text_c(fw / 2, fh / 3 + 34, "M MUSIC ON-OFF    N SOUND ON-OFF    - AND + VOLUME", [150, 140, 150], 1);"""
assert s.count(a) == 1
s = s.replace(a, """        u.text_c(fw / 2, fh / 3 - 10, "PAUSED", [240, 214, 150], 3);
        draw_menu(g, &mut u, fw, fh);
        u.text_c(fw / 2, fh - 20, "ESC RESUME    K SKILLS    I GEAR    M MUSIC    N SOUND", [150, 140, 150], 1);""")
a = """    if g.shop_open {
        draw_shop(g, &mut u, fw, fh);
    }"""
assert s.count(a) == 1
s = s.replace(a, a + """
    if g.ending == -1 {
        draw_ending(g, &mut u, fw, fh);
    }""")
open(H, 'w', encoding='utf8').write(s)
print("menu wired (game, areas, hud)")
