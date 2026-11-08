def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("// Rendgrave: a pixel-simulated action RPG.\n", "// Rendgrave: a pixel-simulated action RPG.\n// Release builds open no console window.\n#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]\n"),
    ("""            if let Some(g) = &self.gfx {
                g.window.set_title(&format!("Rendgrave  -  {:.0} fps", fps));
            }""", """            if cfg!(debug_assertions) {
                if let Some(g) = &self.gfx {
                    g.window.set_title(&format!("Rendgrave  -  {:.0} fps", fps));
                }
            }"""),
    ("    let menus = args.iter().any(|a| a == \"--menus\");\n", "    let menus = args.iter().any(|a| a == \"--menus\");\n    // --press: no control hints, banners or toasts, for store screenshots\n    let press = args.iter().any(|a| a == \"--press\");\n"),
    ("""        audio: if testing { None } else { audio::Audio::new() },
    };""", """        audio: if testing { None } else { audio::Audio::new() },
    };
    app.game.press = press;"""),
])
edit('src/game.rs', [
    ("    pub act1_done: bool,\n", "    pub act1_done: bool,\n    /// screenshots for the store page: hide hints, banners and toasts\n    pub press: bool,\n"),
    ("            act1_done: false,\n", "            act1_done: false,\n            press: false,\n"),
])
edit('src/hud.rs', [
    ("    // controls hint for the first seconds\n    if t < 900 {", "    // controls hint for the first seconds\n    if t < 900 && !g.press {"),
    ("    // banner\n    if let Some((a, b, tt)) = &g.banner {", "    // banner\n    if let Some((a, b, tt)) = g.banner.as_ref().filter(|_| !g.press) {"),
    ("    if let Some((msg, t)) = &g.toast {", "    if let Some((msg, t)) = g.toast.as_ref().filter(|_| !g.press) {"),
])
print("release tweaks ok")
