def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


H = 'src/hud.rs'
edit(H, [
    ("use crate::world::*;", "use crate::world::*;\nuse crate::gen::AreaId;"),
    ("        let top = if e.k == Kind::Brute { 66.0 } else { 58.0 };", "        if e.k == Kind::King {\n            continue;\n        }\n        let top = if e.k == Kind::Brute { 66.0 } else { 58.0 };"),
    ("    u.text(8, 8, &format!(\"FOES {}\", alive), [200, 190, 190], 1, true);",
     """    let _ = alive;
    u.text(8, 8, &g.area.name(), [220, 205, 170], 1, true);
    u.text(8, 17, &format!("AREA LEVEL {}", g.alvl), [150, 140, 150], 1, true);
    // boss health
    if let Some(b) = g.enemies.iter().find(|e| e.alive && e.k == Kind::King && e.aggro) {
        let bw = (fw / 2).min(260);
        let bx = fw / 2 - bw / 2;
        u.rect(bx - 2, 30, bw + 4, 8, [10, 6, 10], 230);
        u.rect(bx, 32, (bw as f32 * (b.hp / b.maxhp).clamp(0.0, 1.0)) as i32, 4, [190, 30, 30], 255);
        u.text_c(fw / 2, 21, "THE HOLLOW KING", [240, 200, 120], 1);
    }
    // interaction prompt
    if let Some(pr) = &g.prompt {
        let w = Ui::text_w(pr, 1);
        u.rect(fw / 2 - w / 2 - 4, fh - 62, w + 8, 11, [10, 6, 10], 200);
        u.text(fw / 2 - w / 2, fh - 59, pr, [255, 230, 150], 1, false);
    }"""),
    # first-time hints mention the E key
    ('"AIM AT LEGS TO TAKE THEM OFF"', '"AIM AT LEGS TO TAKE THEM OFF   E INTERACT"'),
    ("""    if g.tree {
        draw_tree(g, &mut u, fw, fh);
    }""", """    if g.tree {
        draw_tree(g, &mut u, fw, fh);
    }
    if g.wp_open {
        draw_waypoints(g, &mut u, fw, fh);
    }
    if g.fade > 0 {
        u.rect(0, 0, fw, fh, [0, 0, 0], (g.fade.min(30) as f32 / 30.0 * 255.0) as u8);
    }"""),
    ("""/// a small map of the whole world, top right""", """/// the waypoint list: (x, y, w, h) of row k
pub fn wp_row(k: usize, fw: i32, fh: i32) -> (i32, i32, i32, i32) {
    let w = 200;
    (fw / 2 - w / 2, fh / 3 + k as i32 * 18, w, 15)
}
pub fn wp_at(m: (f32, f32), view: (f32, f32), n: usize) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    (0..n).find(|&k| {
        let (x, y, w, h) = wp_row(k, fw, fh);
        m.0 as i32 >= x && m.0 as i32 <= x + w && m.1 as i32 >= y && m.1 as i32 <= y + h
    })
}
fn draw_waypoints(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    u.rect(0, 0, fw, fh, [4, 6, 12], 190);
    u.text_c(fw / 2, fh / 3 - 30, "WAYPOINTS", [170, 200, 255], 2);
    let list: Vec<AreaId> = AreaId::WAYPOINTS.iter().copied().filter(|a| g.discovered.contains(a)).collect();
    for (k, a) in list.iter().enumerate() {
        let (x, y, w, h) = wp_row(k, fw, fh);
        let hover = g.wp_hover == Some(k);
        u.rect(x, y, w, h, if hover { [40, 54, 90] } else { [18, 20, 34] }, 240);
        u.rect(x, y, w, 1, [90, 110, 170], 255);
        let here = *a == g.area;
        u.text(x + 6, y + 5, &format!("{}  {}", k + 1, a.name()), if here { [150, 150, 170] } else { [230, 230, 255] }, 1, false);
        let lv = format!("LV {}", a.level());
        u.text(x + w - Ui::text_w(&lv, 1) - 6, y + 5, &lv, [150, 170, 220], 1, false);
    }
    u.text_c(fw / 2, fh / 3 + list.len() as i32 * 18 + 10, "CLICK OR PRESS A NUMBER - ESC CLOSES", [140, 140, 160], 1);
}

/// a small map of the whole world, top right"""),
    ("                TREE => [24, 52, 28],", "                TREE => [24, 52, 28],\n                ROCK => [8, 7, 10],"),
    ("            let c = match m {\n                TREE", "            let g_ = g.world.g[i];\n            let c = if g_ == STAIRS || g_ == STAIRSUP { [240, 220, 140] } else { match m {\n                TREE"),
    ("""                    _ => [36, 60, 34],
                },
            };""", """                    _ => [36, 60, 34],
                },
            } };"""),
    ("if e.k == crate::body::Kind::Brute { [255, 150, 40] }", "if e.k.big() { [255, 150, 40] }"),
])
# waypoint marker on the minimap
edit(H, [("""    let (px, py) = (ox + (g.player.x / sx) as i32, oy + (g.player.y / sy) as i32);""", """    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);
        u.px(x, y, [120, 170, 255], 255);
        u.px(x + 1, y, [120, 170, 255], 255);
        u.px(x, y + 1, [120, 170, 255], 255);
    }
    let (px, py) = (ox + (g.player.x / sx) as i32, oy + (g.player.y / sy) as i32);""")])

R = 'src/render.rs'
edit(R, [
    # stairs on the ground
    ("""        FLOOR => {
            let (tx, ty) = (x.rem_euclid(10), (y + (x / 10) % 2 * 5).rem_euclid(10));""", """        STAIRS | STAIRSUP => {
            // steps going down into darkness (or up into light)
            let step = (y.rem_euclid(4) == 0) as i32 as f32;
            let depth = (y.rem_euclid(24)) as f32 / 24.0;
            let b = if w.g[i] == STAIRS { 0.75 - depth * 0.5 } else { 0.45 + depth * 0.5 };
            scl([96, 90, 100], b - step * 0.25)
        }
        FLOOR => {
            let (tx, ty) = (x.rem_euclid(10), (y + (x / 10) % 2 * 5).rem_euclid(10));"""),
    # bedrock walls in dungeons
    ("""                BRICK => {
                    let edge = north != BRICK;
                    scl([96, 88, 100], 0.85 + s * 0.15 + if edge { 0.25 } else { 0.0 })
                }
                _ => {""", """                BRICK => {
                    let edge = north != BRICK;
                    scl([96, 88, 100], 0.85 + s * 0.15 + if edge { 0.25 } else { 0.0 })
                }
                ROCK => scl([20, 18, 24], 0.8 + s * 0.4),
                _ => {"""),
    ("""                        STONE => scl([84, 82, 90], 0.95 - k as f32 / h as f32 * 0.35 + s * 0.1),
                        _ => scl([102, 70, 42], 0.9 - k as f32 / h as f32 * 0.3),""", """                        STONE => scl([84, 82, 90], 0.95 - k as f32 / h as f32 * 0.35 + s * 0.1),
                        ROCK => scl([46, 42, 48], 0.9 - k as f32 / h as f32 * 0.5 + h2(wx, k / 3, 7) * 0.2),
                        _ => scl([102, 70, 42], 0.9 - k as f32 / h as f32 * 0.3),"""),
    # dead trees in the graveyard: bare grey canopies you can see through
    ("""    let leaf = mixc([34, 76, 40], [62, 98, 44], seed);""", """    let dead = w.aux[i] & AUX_DEAD != 0;
    let leaf = if dead { mixc([58, 50, 44], [84, 76, 70], seed) } else { mixc([34, 76, 40], [62, 98, 44], seed) };
    if dead && h2(wx, wy, 13) < 0.55 && w.aux[i] & AUX_TRUNK == 0 {
        // bare branches: only a lattice of twigs
        if (wx + wy) % 3 != 0 {
            return;
        }
    }"""),
    # waypoint stone glow + arrows
    ("""    // ---- pickups""", """    // ---- waypoint: a rune circle that glows brighter once found
    if let Some((wx, wy)) = g.waypoint {
        let found = g.discovered.contains(&g.area);
        let (sx, sy) = (wx as i32 - cx, wy as i32 - cy);
        let pulse = 0.6 + (t as f32 * 0.06).sin() * 0.3;
        for k in 0..90 {
            let a = k as f32 / 90.0 * std::f32::consts::TAU;
            let (x, y) = (sx + (a.cos() * 14.0) as i32, sy + (a.sin() * 9.0) as i32);
            c.glow(x, y, scl([90, 140, 255], pulse * if found { 1.0 } else { 0.35 }));
        }
        for k in 0..12 {
            c.put(sx, sy - 14 - k, wy, [130, 150, 190], scl([80, 120, 255], pulse * 0.6), 14.0 + k as f32);
        }
    }
    // ---- pickups"""),
    ("""        let r = if p.k == 1 { 2 } else { 3 };""", """        if p.k == 3 {
            // arrow: shaft along its flight, grey head
            let l = (p.vx * p.vx + p.vy * p.vy).sqrt().max(0.01);
            let (ux, uy) = (p.vx / l, p.vy / l * 0.6);
            for k in 0..7 {
                let (x, y) = ((p.x - ux * k as f32) as i32 - cx, (p.y - p.z - uy * k as f32) as i32 - cy);
                c.put(x, y, p.y + 0.5, if k == 0 { [200, 200, 210] } else if k == 6 { [220, 220, 220] } else { [120, 86, 54] }, [0; 3], p.z);
            }
            continue;
        }
        let r = if p.k == 1 { 2 } else { 3 };"""),
    ("""            _ => add(p.x, p.y, 100.0, 1.4 + (t as f32 * 0.9).sin() * 0.3, [0.55, 0.7, 1.0]),""", """            2 => add(p.x, p.y, 100.0, 1.4 + (t as f32 * 0.9).sin() * 0.3, [0.55, 0.7, 1.0]),
            _ => {}"""),
    ("""        if e.alive && e.k != Kind::Zombie && e.frozen == 0 {
            let col = if e.k == Kind::Skeleton { [1.0, 0.25, 0.15] } else { [1.0, 0.5, 0.15] };""", """        if e.alive && !matches!(e.k, Kind::Zombie | Kind::Crawler) && e.frozen == 0 {
            let col = if e.k == Kind::King { [0.5, 0.7, 1.0] } else if e.k.bony() { [1.0, 0.25, 0.15] } else { [1.0, 0.5, 0.15] };"""),
    ("            shadow(&mut c, e.x, e.y, if e.k == Kind::Brute { 11.0 } else { 7.0 });", "            shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k == Kind::Brute { 11.0 } else { 7.0 });"),
    ("            windup: e.st == St::Wind && e.k == Kind::Brute,", "            windup: e.st == St::Wind && e.k.big(),"),
])
if 'if let Some((wx, wy)) = g.waypoint {\n        let found' in open(R, encoding='utf8').read():
    # the waypoint light
    edit(R, [("""    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;""", """    if let Some((wx, wy)) = g.waypoint {
        add(wx, wy + 2.0, 90.0, 0.9, [0.45, 0.6, 1.0]);
    }
    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;""")])
print("view ok")
