p = 'src/render.rs'
s = open(p, encoding='utf8').read()


def R(a, b, n=1):
    global s
    assert s.count(a) == n, (a[:80], s.count(a))
    s = s.replace(a, b)


R("use crate::world::*;", "use crate::world::*;\nuse crate::skills::*;\nuse crate::spells::METEOR_T;")
R("            let col = [[0.6, 0.75, 1.0], [1.0, 0.6, 0.3], [0.5, 1.0, 0.3], [0.5, 0.8, 1.0]][g.player.sel];",
  """            let bar = g.player.sk.bar();
            let sc = bar.get(g.player.sel).map(|s| SCHOOLS[node(*s).col].1).unwrap_or([255, 255, 255]);
            let col = [sc[0] as f32 / 255.0, sc[1] as f32 / 255.0, sc[2] as f32 / 255.0];""")
R("""        let r = if p.k == 0 { 3 } else { 2 };""", """        let r = if p.k == 1 { 2 } else { 3 };""")
R("""                let col = if p.k == 0 {
                    if d < 1.2 { [255, 250, 220] } else { fire_col((dx * 7 + dy) as u32, t) }
                } else if d < 1.0 {""", """                let col = if p.k == 0 {
                    if d < 1.2 { [255, 250, 220] } else { fire_col((dx * 7 + dy) as u32, t) }
                } else if p.k == 2 {
                    if d < 1.5 || h2(dx, dy + t as i32, 4) < 0.3 { [235, 245, 255] } else { [110, 150, 255] }
                } else if d < 1.0 {""")
R("""                c.put(sx + dx, sy + dy, p.y + 0.5, col, scl(col, if p.k == 0 { 1.0 } else { 0.5 }), p.z);""",
  """                c.put(sx + dx, sy + dy, p.y + 0.5, col, scl(col, if p.k == 1 { 0.5 } else { 1.0 }), p.z);""")
R("""        if p.k == 0 {
            add(p.x, p.y, 110.0, 1.6, [1.0, 0.55, 0.2]);
        } else {
            add(p.x, p.y, 40.0, 0.5, [0.5, 1.0, 0.3]);
        }""", """        match p.k {
            0 => add(p.x, p.y, 110.0, 1.6, [1.0, 0.55, 0.2]),
            1 => add(p.x, p.y, 40.0, 0.5, [0.5, 1.0, 0.3]),
            _ => add(p.x, p.y, 100.0, 1.4 + (t as f32 * 0.9).sin() * 0.3, [0.55, 0.7, 1.0]),
        }""")
R("""            PtK::Gold => c.glow(sx, sy, [200, 160, 40]),""", """            PtK::Gold => c.glow(sx, sy, [200, 160, 40]),
            PtK::Cloud => {
                let r = 5;
                let a2 = (a * (1.0 - a) * 4.0).min(1.0);
                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = ((dx * dx + dy * dy) as f32).sqrt() / r as f32;
                        if d < 1.0 {
                            c.blend(sx + dx, sy + dy, 1e7, p.c, a2 * 0.22 * (1.0 - d));
                        }
                    }
                }
            }""")
R("""    // ---- lightning bolts""", """    // ---- meteors: a growing shadow and the rock coming down
    for m in &g.meteors {
        let f = m.t as f32 / METEOR_T as f32;
        let (gx, gy) = (m.x as i32 - cx, m.y as i32 - cy);
        let rw = 6.0 + f * 16.0;
        for dy in -6..=6 {
            for dx in -(rw as i32)..=(rw as i32) {
                let d = (dx as f32 / rw).powi(2) + (dy as f32 / (rw * 0.4)).powi(2);
                if d < 1.0 {
                    c.blend(gx + dx, gy + dy, -1e8, [0, 0, 0], 0.5 * f * (1.0 - d));
                }
            }
        }
        for k in 0..60 {
            let a = k as f32 / 60.0 * 6.283;
            let (x, y) = (gx + (a.cos() * rw) as i32, gy + (a.sin() * rw * 0.4) as i32);
            c.glow(x, y, scl([255, 120, 40], f * 0.6));
        }
        let (hx, hy) = ((m.x + (1.0 - f) * 90.0) as i32 - cx, (m.y - (1.0 - f) * 240.0) as i32 - cy);
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d < 4.2 {
                    let col = if d < 2.0 { [255, 240, 180] } else if d < 3.2 { [255, 140, 40] } else { [110, 70, 50] };
                    c.put(hx + dx, hy + dy, 1e7, col, scl(col, 0.9), 50.0);
                }
            }
        }
    }
    // ---- ice spikes
    for sp in &g.spikes {
        if sp.t <= 0 {
            continue;
        }
        let grow = (sp.t as f32 / 4.0).min(1.0) * (1.0 - ((sp.t - 40).max(0) as f32 / 20.0));
        let h = (sp.h * grow) as i32;
        let (sx, sy) = (sp.x as i32 - cx, sp.y as i32 - cy);
        for k in 0..h {
            let w = ((h - k) as f32 / 4.0) as i32;
            for dx in -w..=w {
                let col = if dx < 0 { [225, 245, 255] } else if dx == 0 { [190, 225, 250] } else { [120, 180, 225] };
                c.put(sx + dx, sy - k, sp.y, col, [20, 40, 60], k as f32);
            }
        }
    }
    // ---- lightning bolts""")
open(p, 'w', encoding='utf8').write(s)
print("render ok")
