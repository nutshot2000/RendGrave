def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/render.rs', [
    ("""    // ---- barrels: powder kegs""", """    // ---- townsfolk: Elder Maren with her staff and lantern, villagers in plain clothes
    for n in &g.npcs {
        let (sx, sy) = (n.x as i32 - cx, n.y as i32 - cy);
        shadow(&mut c, n.x, n.y, 6.0);
        let pal: [[u8; 3]; 5] = [[86, 98, 80], [124, 72, 50], [70, 92, 124], [116, 104, 62], [92, 62, 96]];
        let robe = pal[n.k as usize % 5];
        let hair: [u8; 3] = if n.k == 0 { [214, 210, 200] } else { [[60, 40, 30], [150, 110, 60], [40, 30, 26], [110, 70, 40]][n.k as usize % 4] };
        let walk = (n.phase.sin() * 2.0) as i32;
        // legs
        for k in 0..9 {
            c.put(sx - 2 + if k < 5 { walk / 2 } else { 0 }, sy - k, n.y, [56, 46, 40], [0; 3], k as f32);
            c.put(sx + 1 - if k < 5 { walk / 2 } else { 0 }, sy - k, n.y, [48, 40, 36], [0; 3], k as f32);
        }
        // body: a robe for Maren, tunic and belt for villagers
        for k in 9..27 {
            let half = if n.k == 0 { 4 + (27 - k) / 7 } else { 4 };
            for dx in -half..=half {
                let shade = 0.78 + (dx + half) as f32 / (2 * half) as f32 * 0.35;
                let belt = n.k != 0 && k == 15;
                let col = if belt { [60, 40, 28] } else { scl(robe, shade) };
                c.put(sx + dx, sy - k, n.y, col, [0; 3], k as f32);
            }
        }
        // head
        let f = n.face as i32;
        for dy in 0..7 {
            for dx in -3i32..=3 {
                let hairy = dy > 4 || (dx * f < -1 && dy > 1);
                let col = if hairy { hair } else { [214, 172, 138] };
                c.put(sx + dx, sy - 27 - dy, n.y, col, [0; 3], 27.0 + dy as f32);
            }
        }
        c.put(sx + f * 2, sy - 30, n.y + 0.1, [30, 20, 20], [0; 3], 30.0);
        if n.k == 0 {
            // her staff, with a little lantern hung on it
            for k in 0..38 {
                c.put(sx + f * 7, sy - k, n.y + 0.2, [100, 72, 44], [0; 3], k as f32);
            }
            let fl = 0.8 + (t as f32 * 0.21).sin() * 0.15;
            for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
                c.put(sx + f * 7 + dx, sy - 40 + dy, n.y + 0.3, [255, 220, 150], scl([255, 180, 90], fl), 40.0);
            }
        }
    }
    // ---- barrels: powder kegs"""),
])

H = 'src/hud.rs'
edit(H, [
    ("            u.text(8, 27, &txt, c, 1, true);", "            u.text(8, 37, &txt, c, 1, true);"),
    ("""    if let Some((k, left)) = g.buff {""", """    if let Some((obj, _)) = g.objective() {
        let pulse = 0.8 + (t as f32 * 0.08).sin() * 0.2;
        u.text(8, 27, &format!("> {}", obj), [(255.0 * pulse) as u8, (214.0 * pulse) as u8, 90], 1, true);
    }
    if let Some((k, left)) = g.buff {"""),
    ("""    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);""", """    if let Some((gx, gy)) = g.guide() {
        if (g.frame / 15) % 2 == 0 {
            let (x, y) = (ox + (gx / sx) as i32, oy + (gy / sy) as i32);
            for &(dx, dy) in &[(0, -2), (-1, -1), (0, -1), (1, -1), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)] {
                u.px(x + dx, y + dy, [255, 214, 90], 255);
            }
        }
    }
    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);"""),
])

s = open(H, encoding='utf8').read()
s += r'''
// ------------------------------------------------------------------ story
/// the gold chevron over your objective, or an arrow at the screen edge pointing to it
fn draw_guide(g: &Game, u: &mut Ui, fw: i32, fh: i32, cx: i32, cy: i32) {
    let Some((gx, gy)) = g.guide() else { return };
    if g.player.dead || g.tree || g.inv_open || g.shop_open || g.wp_open || g.paused {
        return;
    }
    let (sx, sy) = (gx as i32 - cx, gy as i32 - cy - 56);
    let col = [255, 214, 90];
    let m = 18;
    if sx >= m && sx < fw - m && sy >= m + 40 && sy < fh - 70 {
        let bob = ((g.frame as f32 * 0.1).sin() * 3.0) as i32;
        for k in 0..5 {
            u.rect(sx - 4 + k, sy + bob + k, 9 - k * 2, 1, col, 230);
        }
        return;
    }
    // off screen: a small arrow at the edge, pointing the way
    let (pcx, pcy) = (fw / 2, fh / 2);
    let (dx, dy) = ((sx - pcx) as f32, (sy - pcy) as f32);
    let l = (dx * dx + dy * dy).sqrt().max(1.0);
    let (ux, uy) = (dx / l, dy / l);
    let tx = if ux.abs() > 1e-3 { ((fw / 2 - m) as f32 / ux.abs()).min(1e9) } else { 1e9 };
    let ty = if uy.abs() > 1e-3 { ((fh / 2 - m - 30) as f32 / uy.abs()).min(1e9) } else { 1e9 };
    let tt = tx.min(ty);
    let (ax, ay) = (pcx as f32 + ux * tt, pcy as f32 + uy * tt);
    let pulse = (200.0 + (g.frame as f32 * 0.15).sin() * 55.0) as u8;
    for k in 0..8 {
        let w = (8 - k) as f32 * 0.6;
        let (bx, by) = (ax - ux * k as f32, ay - uy * k as f32);
        for s in -(w as i32)..=(w as i32) {
            u.px((bx - uy * s as f32) as i32, (by + ux * s as f32) as i32, col, pulse);
        }
    }
}

/// the dialogue box at the foot of the screen
fn draw_talk(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let Some((pages, page)) = &g.talk else { return };
    let (x, y, w, h) = (30, fh - 92, fw - 60, 62);
    u.rect(x, y, w, h, [16, 11, 14], 238);
    u.rect(x, y, w, 1, [200, 170, 110], 255);
    u.rect(x, y + h - 1, w, 1, [90, 70, 50], 255);
    // a little portrait: Maren's lantern-lit face
    u.rect(x + 8, y + 10, 26, 30, [40, 30, 34], 255);
    u.rect(x + 13, y + 14, 16, 12, [214, 172, 138], 255);
    u.rect(x + 11, y + 12, 20, 4, [214, 210, 200], 255);
    u.rect(x + 11, y + 12, 3, 14, [214, 210, 200], 255);
    u.rect(x + 17, y + 19, 2, 1, [30, 20, 20], 255);
    u.rect(x + 24, y + 19, 2, 1, [30, 20, 20], 255);
    u.rect(x + 11, y + 26, 20, 14, [86, 98, 80], 255);
    u.text(x + 44, y + 7, "ELDER MAREN", [240, 214, 150], 1, true);
    let maxc = ((w - 56) / 4) as usize;
    if let Some(txt) = pages.get(*page) {
        for (k, l) in wrap(txt, maxc).iter().enumerate() {
            u.text(x + 44, y + 20 + k as i32 * 9, l, [230, 222, 210], 1, false);
        }
    }
    let more = if *page + 1 < pages.len() { format!("E  NEXT  ({}/{})", page + 1, pages.len()) } else { "E  CLOSE".into() };
    if (g.frame / 30) % 2 == 0 || *page + 1 >= pages.len() {
        u.text(x + w - Ui::text_w(&more, 1) - 8, y + h - 10, &more, [255, 214, 90], 1, false);
    }
}
'''
a = """    if g.ending == -1 {
        draw_ending(g, &mut u, fw, fh);
    }"""
assert s.count(a) == 1
s = s.replace(a, """    draw_guide(g, &mut u, fw, fh, cx, cy);
    draw_talk(g, &mut u, fw, fh);
""" + a)
open(H, 'w', encoding='utf8').write(s)
print("story draw ok")
