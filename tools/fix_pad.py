# Gamepad support: wire pad.rs into input, movement, aiming, text and the controls page.
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)

G = 'src/game.rs'
edit(G, [
    ("    /// Q: throw an oil flask\n    pub throw: bool,\n}", "    /// Q: throw an oil flask\n    pub throw: bool,\n    /// a gamepad's left stick (screen space, down is +y); zero when unused\n    pub stick: (f32, f32),\n    /// the gamepad is auto-aiming: the cursor is on an enemy but not at a chosen height\n    pub auto: bool,\n}"),
    ("""        let mut dx = (inp.right as i32 - inp.left as i32) as f32;
        let mut dy = (inp.down as i32 - inp.up as i32) as f32;
        let l = (dx * dx + dy * dy).sqrt();""", """        let mut dx = (inp.right as i32 - inp.left as i32) as f32;
        let mut dy = (inp.down as i32 - inp.up as i32) as f32;
        if inp.stick != (0.0, 0.0) {
            (dx, dy) = inp.stick;
        }
        let l = (dx * dx + dy * dy).sqrt();"""),
    ("""    pub fn mouse_world(&self, inp: &Input) -> (f32, f32) {""", """    /// a screen that wants a cursor rather than a fight (the gamepad moves a pointer in these)
    pub fn menu_mode(&self) -> bool {
        self.title || self.paused || self.tree || self.inv_open || self.shop_open || self.wp_open || self.controls || self.choice.is_some() || self.talk.is_some() || self.ending == -1
    }
    pub fn mouse_world(&self, inp: &Input) -> (f32, f32) {"""),
])
edit('src/gear.rs', [("            let h = if rel_mx.abs() < 14.0 && rel_my < 2.0 && rel_my > -66.0 {", "            let h = if !inp.auto && rel_mx.abs() < 14.0 && rel_my < 2.0 && rel_my > -66.0 {")])

H = 'src/hud.rs'
edit(H, [
    ("""    pub fn text_w(s: &str, sc: i32) -> i32 {
        s.chars().count() as i32 * 4 * sc - sc
    }
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: [u8; 3], sc: i32, shadow: bool) {
        let mut cx = x;""", """    pub fn text_w(s: &str, sc: i32) -> i32 {
        crate::pad::tr(s).chars().count() as i32 * 4 * sc - sc
    }
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: [u8; 3], sc: i32, shadow: bool) {
        let s = crate::pad::tr(s);
        let mut cx = x;"""),
    ("""        let lines = ["WASD MOVE   LMB ATTACK   RMB CAST", "1-9 SPELLS   SPACE DODGE   F POTION   K SKILLS", "AIM AT LEGS TO TAKE THEM OFF   E INTERACT   I GEAR"];""", """        let lines = if crate::pad::active() {
            ["LEFT STICK MOVE   A ATTACK   X Y B CAST", "RB DODGE   LB POTION   D-PAD UP SKILLS", "RIGHT STICK AIMS - POINT AT LEGS TO TAKE THEM OFF"]
        } else {
            ["WASD MOVE   LMB ATTACK   RMB CAST", "1-9 SPELLS   SPACE DODGE   F POTION   K SKILLS", "AIM AT LEGS TO TAKE THEM OFF   E INTERACT   I GEAR"]
        };"""),
    ("""    let rows: [(&str, &str); 17] = [""", """    const PAD_ROWS: [(&str, &str); 17] = [
        ("LEFT STICK", "MOVE"),
        ("RIGHT STICK", "AIM - OR LET GO TO AUTO-AIM"),
        ("A", "ATTACK, OR INTERACT WHEN IN REACH"),
        ("X / Y / B", "CAST SPELL 1 / 2 / 3"),
        ("LT", "CAST THE SELECTED SPELL"),
        ("D-PAD LEFT RIGHT", "CHOOSE A SPELL"),
        ("RB", "DODGE ROLL"),
        ("LB", "DRINK A POTION"),
        ("RT", "THROW AN OIL FLASK (IF YOU HAVE THEM)"),
        ("D-PAD UP", "SKILL TREE"),
        ("D-PAD DOWN / BACK", "GEAR AND INVENTORY"),
        ("START", "PAUSE"),
        ("IN MENUS", "STICK MOVES THE POINTER, A CLICKS"),
        ("", "X RIGHT-CLICKS, B GOES BACK"),
        ("A", "WAKE IN ASHFORD AFTER DEATH"),
        ("AIM LOW", "TAKE OFF LEGS, HIT CRAWLERS AND BARRELS"),
        ("CRACKED WALLS", "BREAK THEM - SOMETHING IS BEHIND"),
    ];
    let key_rows: [(&str, &str); 17] = ["""),
    ("""    let (w, h) = (300, rows.len() as i32 * 10 + 40);""", """    let rows = if crate::pad::active() { PAD_ROWS } else { key_rows };
    let (w, h) = (300, rows.len() as i32 * 10 + 40);"""),
])
M = 'src/main.rs'
edit(M, [
    ("mod areas;\n", "mod areas;\nmod pad;\n"),
    ("    audio: Option<audio::Audio>,\n}", "    audio: Option<audio::Audio>,\n    pad: pad::Pad,\n}"),
    ("""            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;""", """            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                if down {
                    self.pad.release();
                }"""),
    ("""            WindowEvent::MouseInput { state, button, .. } => {
                let down = state == ElementState::Pressed;""", """            WindowEvent::MouseInput { state, button, .. } => {
                let down = state == ElementState::Pressed;
                if down {
                    self.pad.release();
                }"""),
    ("""            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_px = (position.x, position.y);""", """            WindowEvent::CursorMoved { position, .. } => {
                if (position.x - self.mouse_px.0).abs() + (position.y - self.mouse_px.1).abs() > 6.0 {
                    self.pad.release();
                }
                self.mouse_px = (position.x, position.y);"""),
    ("""            self.input.mouse = (((self.mouse_px.0 - offx as f64) / sc as f64) as f32, ((self.mouse_px.1 - offy as f64) / sc as f64) as f32);
            let now = Instant::now();""", """            self.input.mouse = (((self.mouse_px.0 - offx as f64) / sc as f64) as f32, ((self.mouse_px.1 - offy as f64) / sc as f64) as f32);
            self.pad.poll();
            let now = Instant::now();"""),
    ("""                let mut inp = self.input.clone();
                inp.sel = self.sel_pending.take();
                self.game.update(&inp);""", """                let mut inp = self.input.clone();
                inp.sel = self.sel_pending.take();
                self.pad.apply(&self.game, &mut inp);
                self.game.update(&inp);"""),
    ("""            if n == 4 {
                self.acc = 0.0;
            }""", """            if n == 4 {
                self.acc = 0.0;
            }
            if self.pad.active {
                // the cursor (or aiming reticle) follows the pad
                self.input.mouse = self.pad.cursor;
            }"""),
])
print("pad ok")
