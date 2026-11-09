def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub shake: f32,\n", "    pub shake: f32,\n    /// screen shake setting: 0 full, 1 low, 2 off\n    pub shake_opt: u8,\n    /// the controls page is open over a menu\n    pub controls: bool,\n"),
    ("            shake: 0.0,\n", "            shake: 0.0,\n            shake_opt: crate::menu::load_shake(),\n            controls: false,\n"),
    ("""        } else if inp.close && !self.prev.close {
            if self.tree {
                self.tree = false;
            } else if""", """        } else if inp.close && !self.prev.close {
            if self.tree {
                self.tree = false;
            } else if self.controls {
                self.controls = false;
            } else if"""),
])

M = 'src/menu.rs'
edit(M, [
    ("    Fullscreen,\n    ToTitle,\n    Quit,\n}", "    Fullscreen,\n    Shake,\n    Controls,\n    ToTitle,\n    Quit,\n}"),
    ("v.extend([MI::NewGame, MI::Music, MI::Sound, MI::Volume, MI::Fullscreen, MI::Quit]);",
     "v.extend([MI::NewGame, MI::Controls, MI::Music, MI::Sound, MI::Volume, MI::Shake, MI::Fullscreen, MI::Quit]);"),
    ("vec![MI::Resume, MI::Music, MI::Sound, MI::Volume, MI::Fullscreen, MI::ToTitle, MI::Quit]",
     "vec![MI::Resume, MI::Controls, MI::Music, MI::Sound, MI::Volume, MI::Shake, MI::Fullscreen, MI::ToTitle, MI::Quit]"),
    ("""            MI::Fullscreen => "FULLSCREEN  F11".into(),\n""",
     """            MI::Fullscreen => "FULLSCREEN  F11".into(),
            MI::Shake => format!("SCREEN SHAKE  {}", ["FULL", "LOW", "OFF"][self.shake_opt.min(2) as usize]),
            MI::Controls => "CONTROLS".into(),\n"""),
    ("""    pub fn update_menu(&mut self, inp: &Input) {
        let items""", """    pub fn update_menu(&mut self, inp: &Input) {
        if self.controls {
            // any click or key closes the controls page
            let any = (inp.lmb && !self.prev.lmb) || (inp.close && !self.prev.close) || (inp.interact && !self.prev.interact) || (inp.dodge && !self.prev.dodge);
            if any {
                self.controls = false;
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Click, x, y, 0.7);
            }
            return;
        }
        let items"""),
    ("""            MI::Fullscreen => Some(Req::Fullscreen),\n""", """            MI::Fullscreen => Some(Req::Fullscreen),
            MI::Shake => {
                self.shake_opt = (self.shake_opt + 1) % 3;
                save_shake(self.shake_opt);
                None
            }
            MI::Controls => {
                self.controls = true;
                None
            }\n"""),
])
s = open(M, encoding='utf8').read()
s += '''
/// screen shake is scaled by the player's setting
pub fn shake_scale(opt: u8) -> f32 {
    [1.0, 0.4, 0.0][opt.min(2) as usize]
}
pub fn load_shake() -> u8 {
    std::fs::read_to_string("rendgrave_options.txt")
        .ok()
        .and_then(|t| t.lines().find_map(|l| l.strip_prefix("shake ").and_then(|v| v.trim().parse().ok())))
        .unwrap_or(0)
}
fn save_shake(v: u8) {
    let _ = std::fs::write("rendgrave_options.txt", format!("shake {}\\n", v));
}
'''
open(M, 'w', encoding='utf8').write(s)

edit('src/render.rs', [("    let sh = g.shake;\n", "    let sh = g.shake * crate::menu::shake_scale(g.shake_opt);\n")])

H = 'src/hud.rs'
edit(H, [
    ("let y0 = if title { fh / 2 - 4 } else { fh / 3 + 18 };", "let y0 = if title { fh / 2 - 24 } else { fh / 3 + 14 };"),
    ("""        draw_menu(g, &mut u, fw, fh);
        let (_, ly, _, _) = menu_row(g.menu_items().len(), fw, fh, false);
        u.text_c(fw / 2, ly + 4, "ESC RESUME    K SKILLS    I GEAR    M MUSIC    N SOUND", [150, 140, 150], 1);""",
     """        if g.controls {
            draw_controls(&mut u, fw, fh);
        } else {
            draw_menu(g, &mut u, fw, fh);
            let (_, ly, _, _) = menu_row(g.menu_items().len(), fw, fh, false);
            u.text_c(fw / 2, ly + 4, "ESC RESUME    K SKILLS    I GEAR    M MUSIC    N SOUND", [150, 140, 150], 1);
        }"""),
    ("""    let pulse = 0.85 + (t as f32 * 0.03).sin() * 0.15;""", """    if g.controls {
        draw_controls(u, fw, fh);
        return;
    }
    let pulse = 0.85 + (t as f32 * 0.03).sin() * 0.15;"""),
    ("""        u.text_c(fw / 2, fh / 3 - 10, "PAUSED", [240, 214, 150], 3);
""", """        if !g.controls {
            u.text_c(fw / 2, fh / 3 - 10, "PAUSED", [240, 214, 150], 3);
        }
"""),
    ("""fn draw_ending(g: &Game""", """/// the full list of keys, from the title or the pause menu
fn draw_controls(u: &mut Ui, fw: i32, fh: i32) {
    let rows: [(&str, &str); 16] = [
        ("WASD / ARROWS", "MOVE"),
        ("LEFT MOUSE", "ATTACK WITH YOUR WEAPON"),
        ("RIGHT MOUSE", "CAST THE SELECTED SPELL"),
        ("1 - 9 / WHEEL", "CHOOSE A SPELL"),
        ("SPACE", "DODGE ROLL"),
        ("F", "DRINK A POTION"),
        ("E", "INTERACT, TALK, OPEN, PICK UP"),
        ("I", "GEAR AND INVENTORY"),
        ("K / TAB", "SKILL TREE"),
        ("ESC", "PAUSE, CLOSE A WINDOW"),
        ("R", "WAKE IN ASHFORD AFTER DEATH"),
        ("M / N", "MUSIC / SOUND ON OR OFF"),
        ("- / +", "VOLUME"),
        ("F11", "FULLSCREEN"),
        ("AIM LOW", "TAKE OFF LEGS, HIT CRAWLERS AND BARRELS"),
        ("CRACKED WALLS", "BREAK THEM - SOMETHING IS BEHIND"),
    ];
    let (w, h) = (300, rows.len() as i32 * 10 + 28);
    let (x, y) = (fw / 2 - w / 2, (fh - h) / 2 + 10);
    u.rect(x - 2, y - 2, w + 4, h + 4, [70, 50, 60], 255);
    u.rect(x, y, w, h, [16, 10, 14], 245);
    u.text_c(fw / 2, y + 6, "CONTROLS", [240, 214, 150], 1);
    for (k, (a, b)) in rows.iter().enumerate() {
        let ry = y + 20 + k as i32 * 10;
        u.text(x + 12, ry, a, [255, 214, 140], 1, false);
        u.text(x + 118, ry, b, [210, 200, 200], 1, false);
    }
    u.text_c(fw / 2, y + h + 8, "CLICK OR ESC TO GO BACK", [150, 140, 150], 1);
}

fn draw_ending(g: &Game"""),
])
print("options ok")
