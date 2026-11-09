def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/world.rs', [("pub const PEAT: u8 = 9;", "pub const PEAT: u8 = 9;\n/// a trodden road: lighter, with cobbles, so it reads at night\npub const ROAD: u8 = 10;")])

AG = 'src/areagen.rs'
edit(AG, [
    ("                        w.g[i] = if d > r - 1.5 && rnd() < 0.5 { MUD } else { DIRT };", "                        w.g[i] = if d > r - 1.5 { if rnd() < 0.5 { MUD } else { DIRT } } else { ROAD };"),
    ("""    /// where Elder Maren stands
    pub maren: Option<(f32, f32)>,
}""", """    /// where Elder Maren stands
    pub maren: Option<(f32, f32)>,
    /// lantern posts along the roads and at the exits
    pub posts: Vec<(f32, f32)>,
}"""),
    ("barrels: vec![], merchant: None, maren: None }", "barrels: vec![], merchant: None, maren: None, posts: vec![] }"),
    # exits get a pair of lit posts
    ("""    let (x0, x1, ax) = if east { (aw - 28, aw - 6, aw as f32 - 44.0) } else { (6, 28, 44.0) };
    sp.exits.push(Exit { x0, y0: y as i32 - 26, x1, y1: y as i32 + 26, to, label: to.name() });
    sp.entries.push((to, ax, y));""", """    let (x0, x1, ax) = if east { (aw - 28, aw - 6, aw as f32 - 44.0) } else { (6, 28, 44.0) };
    sp.exits.push(Exit { x0, y0: y as i32 - 26, x1, y1: y as i32 + 26, to, label: to.name() });
    sp.entries.push((to, ax, y));
    let px = if east { aw as f32 - 34.0 } else { 34.0 };
    sp.posts.push((px, y - 24.0));
    sp.posts.push((px, y + 26.0));"""),
    ("""/// clear a round patch of ground to `g`""", """/// lantern posts every so often along a road, on alternating sides
fn road_posts(w: &World, sp: &mut Spawns, road: &[(f32, f32)], every: f32) {
    let mut since = every * 0.5;
    let mut side = 1.0;
    for k in 1..road.len().saturating_sub(1) {
        let (a, b) = (road[k - 1], road[k + 1]);
        since += ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() * 0.5;
        if since < every {
            continue;
        }
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(0.01);
        let (x, y) = (road[k].0 - dy / l * 14.0 * side, road[k].1 + dx / l * 14.0 * side);
        if x < 60.0 || y < 60.0 || x > w.aw as f32 - 60.0 || y > w.ah as f32 - 60.0 || !open_spot(w, x, y, 2) {
            continue;
        }
        sp.posts.push((x, y));
        since = 0.0;
        side = -side;
    }
}
/// clear a round patch of ground to `g`"""),
])
s = open(AG, encoding='utf8').read()
# posts along the main roads: after the trees are planted (so the spot checks see them),
# just before each generator's border() call
for fn, road in [('fn forest(', 'road'), ('fn graveyard(', 'road'), ('fn fen(', 'road'), ('fn ravenhold(', 'road'), ('fn town(', 'road')]:
    a = s.index(fn)
    b = s.index('    border(w);\n', a)
    s = s[:b] + f'    road_posts(w, &mut sp, &{road}, 170.0);\n' + s[b:]
open(AG, 'w', encoding='utf8').write(s)

G = 'src/game.rs'
edit(G, [
    ("    pub merchant: Option<(f32, f32)>,\n", "    pub merchant: Option<(f32, f32)>,\n    pub posts: Vec<(f32, f32)>,\n"),
    ("            merchant: None,\n", "            merchant: None,\n            posts: vec![],\n"),
])
edit('src/areas.rs', [("        self.merchant = sp.merchant;\n", "        self.merchant = sp.merchant;\n        self.posts = sp.posts.clone();\n")])

R = 'src/render.rs'
edit(R, [
    ("""        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);""", """        ROAD => {
            // packed earth with cobbles showing through
            let cob = h2(x / 3, y / 2 + (x / 3) % 2, 13);
            let base = if cob > 0.62 { [128, 112, 90] } else { [112, 92, 66] };
            scl(base, 0.84 + n * 0.2 + s * 0.1)
        }
        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);"""),
    ("""    for &(lx, ly, lc) in &g.lamps {
        let fl = 1.0 + (t as f32 * 0.17 + lx).sin() * 0.06;
        add(lx, ly, 90.0, 1.3 * fl, lc);
    }""", """    for &(lx, ly, lc) in &g.lamps {
        let fl = 1.0 + (t as f32 * 0.17 + lx).sin() * 0.06;
        add(lx, ly, 90.0, 1.3 * fl, lc);
    }
    for &(px, py) in &g.posts {
        let fl = 1.0 + (t as f32 * 0.19 + px).sin() * 0.07;
        add(px + 4.0, py + 2.0, 85.0, 1.1 * fl, [1.0, 0.68, 0.32]);
    }"""),
    ("""    // ---- barrels: powder kegs""", """    // ---- lantern posts: a pole with a lantern hung from a short arm
    for &(px, py) in &g.posts {
        let (sx, sy) = (px as i32 - cx, py as i32 - cy);
        shadow(&mut c, px, py, 3.0);
        for k in 0..28 {
            c.put(sx, sy - k, py, [86, 62, 40], [0; 3], k as f32);
            c.put(sx + 1, sy - k, py, [66, 48, 32], [0; 3], k as f32);
        }
        for dx in 0..6 {
            c.put(sx + dx, sy - 28, py, [80, 58, 38], [0; 3], 28.0);
        }
        c.put(sx + 5, sy - 27, py, [60, 60, 66], [0; 3], 27.0);
        let fl = 0.85 + (t as f32 * 0.19 + px).sin() * 0.1;
        for dy in 0..4 {
            for dx in 4..7 {
                let rim = dx == 4 || dx == 6 || dy == 0 || dy == 3;
                c.put(sx + dx, sy - 26 + dy, py + 0.1, if rim { [70, 66, 70] } else { [255, 214, 130] }, if rim { [0; 3] } else { scl([255, 170, 70], fl) }, 26.0 - dy as f32);
            }
        }
    }
    // ---- barrels: powder kegs"""),
])

H = 'src/hud.rs'
edit(H, [
    ("                    PEAT => [44, 34, 26],", "                    PEAT => [44, 34, 26],\n                    ROAD => [150, 124, 84],"),
    ("""    if let Some((gx, gy)) = g.guide() {
        if (g.frame / 15) % 2 == 0 {""", """    // exits: a bright notch where each one leaves the map
    for e in &g.exits {
        let (x0, y0) = (ox + (e.x0 as f32 / sx) as i32, oy + (e.y0 as f32 / sy) as i32);
        let (x1, y1) = (ox + (e.x1 as f32 / sx) as i32, oy + (e.y1 as f32 / sy) as i32);
        for y in y0..=y1.max(y0 + 1) {
            for x in x0..=x1.max(x0 + 1) {
                u.px(x, y, [235, 235, 255], 255);
            }
        }
    }
    if let Some((gx, gy)) = g.guide() {
        if (g.frame / 15) % 2 == 0 {"""),
])
s = open(H, encoding='utf8').read()
s += r'''
/// names over nearby exits, so you know where a road or a stair leads
fn draw_exit_labels(g: &Game, u: &mut Ui, fw: i32, fh: i32, cx: i32, cy: i32) {
    if g.player.dead || g.tree || g.inv_open || g.shop_open || g.wp_open || g.paused || g.talk.is_some() {
        return;
    }
    for e in &g.exits {
        let (ex, ey) = ((e.x0 + e.x1) as f32 / 2.0, (e.y0 + e.y1) as f32 / 2.0);
        let d = ((ex - g.player.x).powi(2) + (ey - g.player.y).powi(2)).sqrt();
        if d > 260.0 {
            continue;
        }
        let a = (((260.0 - d) / 80.0).min(1.0) * 230.0) as u8;
        let txt = format!("TO {}", e.label);
        let w = Ui::text_w(&txt, 1);
        let (sx, sy) = (ex as i32 - cx, e.y0 - cy - 14);
        let x = (sx - w / 2).clamp(6, fw - w - 6);
        let y = sy.clamp(44, fh - 80);
        u.rect(x - 3, y - 2, w + 6, 10, [10, 8, 14], a.min(190));
        u.text(x, y, &txt, [210, 220, 255], 1, false);
    }
}
'''
a = "    draw_guide(g, &mut u, fw, fh, cx, cy);\n"
assert s.count(a) == 1
s = s.replace(a, a + "    draw_exit_labels(g, &mut u, fw, fh, cx, cy);\n")
open(H, 'w', encoding='utf8').write(s)

edit('src/mapdump.rs', [("                            PEAT => [52, 40, 30],", "                            PEAT => [52, 40, 30],\n                            ROAD => [150, 124, 84],")])
print("roads ok")
