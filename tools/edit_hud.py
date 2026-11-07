p = 'src/hud.rs'
s = open(p, encoding='utf8').read()


def R(a, b, n=1):
    global s
    assert s.count(a) == n, (a[:80], s.count(a))
    s = s.replace(a, b)


def block(start, end, new):
    global s
    i = s.index(start)
    j = s.index(end, i)
    s = s[:i] + new + s[j:]


R("use crate::rng::*;", "use crate::rng::*;\nuse crate::skills::*;")

block("fn icon(", "pub fn draw(", r'''fn line(u: &mut Ui, x0: i32, y0: i32, x1: i32, y1: i32, c: [u8; 3]) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for j in 0..=n {
        u.px(x0 + (x1 - x0) * j / n, y0 + (y1 - y0) * j / n, c, 255);
    }
}
fn disc(u: &mut Ui, cx: i32, cy: i32, r: f32, c: [u8; 3]) {
    let ri = r.ceil() as i32;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            if ((dx * dx + dy * dy) as f32) <= r * r {
                u.px(cx + dx, cy + dy, c, 255);
            }
        }
    }
}
fn bolt(u: &mut Ui, x: i32, y: i32, s: i32, c: [u8; 3]) {
    let pts: [(i32, i32); 6] = [(4, 0), (1, 5), (4, 6), (0, 12), (6, 5), (3, 4)];
    for w in pts.windows(2) {
        let ((a, b), (c2, d)) = (w[0], w[1]);
        line(u, x + a * s / 2, y + b * s / 2, x + c2 * s / 2, y + d * s / 2, c);
    }
}
fn drop(u: &mut Ui, cx: i32, cy: i32, sz: i32, c: [u8; 3]) {
    for dy in 0..sz * 2 {
        let w = if dy < sz { dy / 2 } else { (sz as f32 * 0.9 - (dy as f32 - sz as f32 * 1.4).abs() * 0.6) as i32 };
        for dx in -w.max(0)..=w.max(0) {
            u.px(cx + dx, cy + dy, c, 255);
        }
    }
}
fn cloud(u: &mut Ui, x: i32, y: i32, c: [u8; 3]) {
    disc(u, x + 5, y + 5, 3.0, c);
    disc(u, x + 9, y + 4, 3.5, c);
    disc(u, x + 12, y + 6, 2.5, c);
    for dx in 3..14 {
        u.px(x + dx, y + 8, c, 255);
    }
}

/// 16x16 pixel icon for each skill
pub fn icon(u: &mut Ui, x: i32, y: i32, s: Sk, t: u64) {
    let fire = |k: i32| if h2(k, (t / 4) as i32, 3) < 0.5 { [255, 160, 40] } else { [230, 90, 30] };
    match s {
        Sk::Lightning => bolt(u, x + 4, y + 1, 2, [255, 245, 170]),
        Sk::ChainMastery => {
            bolt(u, x + 1, y + 2, 2, [255, 245, 170]);
            bolt(u, x + 8, y + 2, 2, [180, 200, 255]);
        }
        Sk::BallLightning => {
            disc(u, x + 8, y + 8, 4.5, [120, 160, 255]);
            disc(u, x + 8, y + 8, 2.5, [230, 240, 255]);
            for k in 0..4 {
                let a = k as f32 * 1.57 + t as f32 * 0.1;
                u.px(x + 8 + (a.cos() * 7.0) as i32, y + 8 + (a.sin() * 7.0) as i32, [200, 220, 255], 255);
            }
        }
        Sk::Thunderstorm => {
            cloud(u, x, y, [110, 110, 130]);
            bolt(u, x + 6, y + 8, 1, [255, 245, 170]);
            bolt(u, x + 10, y + 9, 1, [255, 245, 170]);
        }
        Sk::Fireball => {
            for dy in -5i32..=5 {
                for dx in -5i32..=5 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d < 4.5 {
                        let c = if d < 2.0 { [255, 250, 200] } else { fire(dx * 7 + dy) };
                        u.px(x + 8 + dx, y + 9 + dy, c, 255);
                    }
                }
            }
        }
        Sk::Kindling => {
            for dy in 0..12 {
                let w = (dy as f32 * 0.4) as i32;
                for dx in -w..=w {
                    u.px(x + 8 + dx, y + 2 + dy, if dx.abs() < w - 1 { [255, 220, 120] } else { fire(dx + dy) }, 255);
                }
            }
        }
        Sk::FireWall => {
            for k in 0..4 {
                let h = 6 + ((k * 5 + (t / 5) as i32) % 4);
                for dy in 0..h {
                    u.px(x + 2 + k * 4, y + 14 - dy, fire(k * 3 + dy), 255);
                    u.px(x + 3 + k * 4, y + 14 - dy, fire(k * 5 + dy), 255);
                }
            }
        }
        Sk::Meteor => {
            line(u, x + 1, y + 1, x + 9, y + 9, [255, 140, 40]);
            line(u, x + 2, y + 1, x + 10, y + 9, [255, 200, 90]);
            disc(u, x + 11, y + 11, 3.5, [120, 90, 70]);
            disc(u, x + 10, y + 10, 1.5, [255, 170, 60]);
        }
        Sk::Acid => drop(u, x + 8, y + 2, 6, [110, 210, 50]),
        Sk::Corrosion => {
            drop(u, x + 4, y + 1, 4, [110, 210, 50]);
            drop(u, x + 11, y + 3, 4, [110, 210, 50]);
            drop(u, x + 7, y + 7, 4, [150, 240, 80]);
        }
        Sk::AcidRain => {
            cloud(u, x, y, [60, 100, 40]);
            for k in 0..4 {
                line(u, x + 4 + k * 3, y + 11, x + 3 + k * 3, y + 14, [140, 230, 70]);
            }
        }
        Sk::AcidFlood => {
            for dx in 0..16 {
                let h = 4.0 + ((dx as f32 * 0.6 + t as f32 * 0.1).sin() * 2.0);
                for dy in 0..h as i32 + 3 {
                    u.px(x + dx, y + 15 - dy, if dy as f32 > h { [190, 255, 120] } else { [100, 200, 50] }, 255);
                }
            }
        }
        Sk::Nova | Sk::Blizzard => {
            for k in 0..6 {
                let a = k as f32 * 1.047;
                for r in 0..7 {
                    u.px(x + 8 + (a.cos() * r as f32) as i32, y + 8 + (a.sin() * r as f32) as i32, [190, 235, 255], 255);
                }
            }
            if s == Sk::Blizzard {
                for k in 0..5 {
                    u.px(x + 2 + k * 3, y + 1 + (k * 7) % 13, [255, 255, 255], 255);
                }
            }
        }
        Sk::Brittle => {
            for dy in 0..14i32 {
                let w = 6 - (dy - 7).abs();
                for dx in -w..=w {
                    u.px(x + 8 + dx, y + 1 + dy, [150, 210, 245], 255);
                }
            }
            line(u, x + 6, y + 3, x + 9, y + 8, [40, 60, 90]);
            line(u, x + 9, y + 8, x + 7, y + 13, [40, 60, 90]);
        }
        Sk::IceSpikes => {
            for (k, h) in [(2, 8), (7, 13), (12, 9)] {
                for dy in 0..h {
                    let w = (h - dy) / 4;
                    for dx in -w..=w {
                        u.px(x + k + dx, y + 15 - dy, if dx < 0 { [230, 245, 255] } else { [140, 200, 240] }, 255);
                    }
                }
            }
        }
        Sk::Edge => {
            line(u, x + 3, y + 13, x + 13, y + 3, [230, 236, 248]);
            line(u, x + 4, y + 13, x + 14, y + 3, [160, 170, 190]);
            line(u, x + 2, y + 10, x + 6, y + 14, [214, 180, 80]);
        }
        Sk::Whirlwind => {
            for k in 0..20 {
                let a = k as f32 * 0.3 + t as f32 * 0.2;
                let r = 2.0 + k as f32 * 0.3;
                u.px(x + 8 + (a.cos() * r) as i32, y + 8 + (a.sin() * r) as i32, [230, 236, 248], 255);
            }
        }
        Sk::Bloodthirst => drop(u, x + 8, y + 2, 6, [210, 30, 40]),
        Sk::Executioner => {
            disc(u, x + 8, y + 7, 5.0, [220, 214, 196]);
            u.px(x + 6, y + 7, [30, 20, 20], 255);
            u.px(x + 10, y + 7, [30, 20, 20], 255);
            for dx in -2..=2 {
                u.px(x + 8 + dx, y + 13, [220, 214, 196], 255);
            }
        }
        Sk::Vigor => {
            disc(u, x + 5, y + 6, 3.0, [220, 40, 60]);
            disc(u, x + 11, y + 6, 3.0, [220, 40, 60]);
            for dy in 0..7 {
                for dx in -(6 - dy)..=(6 - dy) {
                    u.px(x + 8 + dx, y + 7 + dy, [220, 40, 60], 255);
                }
            }
        }
        Sk::Wisdom => {
            for dy in -7i32..=7 {
                let w = (7 - dy.abs()) / 2;
                for dx in -w..=w {
                    u.px(x + 8 + dx, y + 8 + dy, [90, 140, 255], 255);
                }
            }
            u.px(x + 7, y + 6, [220, 230, 255], 255);
        }
        Sk::Swiftness => {
            for k in 0..2 {
                line(u, x + 3 + k * 6, y + 3, x + 8 + k * 6, y + 8, [240, 230, 200]);
                line(u, x + 8 + k * 6, y + 8, x + 3 + k * 6, y + 13, [240, 230, 200]);
            }
        }
        Sk::Fortune => {
            disc(u, x + 8, y + 8, 5.5, [220, 170, 50]);
            disc(u, x + 8, y + 8, 3.5, [255, 230, 130]);
        }
    }
}

/// layout of the skill tree screen: (x, y) of each node's top-left corner and the box size
pub fn node_pos(i: usize, fw: i32, _fh: i32) -> (i32, i32, i32) {
    let n = &NODES[i];
    let cw = (fw - 20) / 6;
    let bs = 20;
    let x = 10 + n.col as i32 * cw + cw / 2 - bs / 2;
    let y = 46 + n.row as i32 * 42;
    (x, y, bs)
}
pub fn node_at(m: (f32, f32), view: (f32, f32)) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    for i in 0..N {
        let (x, y, bs) = node_pos(i, fw, fh);
        if m.0 as i32 >= x - 2 && m.0 as i32 <= x + bs + 2 && m.1 as i32 >= y - 2 && m.1 as i32 <= y + bs + 2 {
            return Some(i);
        }
    }
    None
}

fn wrap(s: &str, max: usize) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    for w in s.split(' ') {
        if !cur.is_empty() && cur.len() + 1 + w.len() > max {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn draw_tree(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let t = g.frame;
    let sk = &g.player.sk;
    u.rect(0, 0, fw, fh, [8, 5, 10], 225);
    u.text(12, 10, "SKILLS", [240, 214, 150], 2, true);
    let info = format!("LEVEL {}    SKILL POINTS {}", sk.level, sk.points);
    u.text(fw - Ui::text_w(&info, 1) - 12, 10, &info, if sk.points > 0 { [255, 220, 120] } else { [200, 190, 190] }, 1, true);
    u.text(fw - Ui::text_w("K CLOSE", 1) - 12, 20, "K CLOSE", [150, 140, 150], 1, true);
    let cw = (fw - 20) / 6;
    for (c, (name, col)) in SCHOOLS.iter().enumerate() {
        u.text_c(10 + c as i32 * cw + cw / 2, 34, name, *col, 1);
    }
    // links
    for i in 0..N {
        let n = &NODES[i];
        if n.row == 0 {
            continue;
        }
        let (x, y, bs) = node_pos(i, fw, fh);
        let lit = sk.rank[i] > 0;
        let col = if lit { SCHOOLS[n.col].1 } else { [60, 50, 64] };
        u.rect(x + bs / 2, y - 22 + 4, 1, 20, col, 255);
    }
    for i in 0..N {
        let n = &NODES[i];
        let (x, y, bs) = node_pos(i, fw, fh);
        let r = sk.rank[i];
        let can = sk.can_learn(i).is_ok();
        let hover = g.tree_hover == Some(i);
        let rim = if hover {
            [255, 255, 255]
        } else if r > 0 {
            SCHOOLS[n.col].1
        } else if can {
            if (t / 15) % 2 == 0 { [255, 210, 90] } else { [180, 140, 60] }
        } else {
            [70, 60, 74]
        };
        u.rect(x - 1, y - 1, bs + 2, bs + 2, rim, 255);
        u.rect(x, y, bs, bs, if r > 0 { [34, 26, 36] } else { [18, 14, 20] }, 255);
        icon(u, x + 2, y + 2, n.sk, t);
        if r == 0 {
            u.rect(x, y, bs, bs, [10, 8, 12], if can { 90 } else { 170 });
        }
        // rank pips
        for k in 0..n.max as i32 {
            let px = x + bs / 2 - n.max as i32 * 2 + k * 4;
            u.rect(px, y + bs + 3, 3, 2, if (k as u8) < r { [255, 214, 90] } else { [60, 50, 60] }, 255);
        }
        if n.active {
            u.px(x + bs - 3, y + 2, [255, 255, 255], 200);
        }
    }
    // tooltip
    let i = g.tree_hover.unwrap_or(usize::MAX);
    let (ty, th) = (fh - 62, 56);
    u.rect(10, ty, fw - 20, th, [20, 14, 22], 240);
    u.rect(10, ty, fw - 20, 1, [90, 74, 84], 255);
    if i < N {
        let n = &NODES[i];
        let col = SCHOOLS[n.col].1;
        u.text(18, ty + 6, n.name, col, 2, true);
        let kind = if n.active { "SPELL" } else { "PASSIVE" };
        let rk = format!("{}   RANK {}/{}", kind, sk.rank[i], n.max);
        u.text(fw - Ui::text_w(&rk, 1) - 18, ty + 8, &rk, [200, 190, 190], 1, true);
        let maxc = ((fw - 40) / 4) as usize;
        for (k, l) in wrap(n.desc, maxc).iter().enumerate() {
            u.text(18, ty + 22 + k as i32 * 8, l, [220, 214, 204], 1, false);
        }
        let status = match sk.can_learn(i) {
            Ok(()) => "CLICK TO LEARN".to_string(),
            Err(e) => e.to_string(),
        };
        let extra = if n.active { format!("MANA {}   COOLDOWN {:.1}S", n.mana as i32, n.cd as f32 / 60.0) } else { String::new() };
        u.text(18, ty + th - 10, &status, if sk.can_learn(i).is_ok() { [255, 214, 90] } else { [170, 120, 120] }, 1, true);
        u.text(fw - Ui::text_w(&extra, 1) - 18, ty + th - 10, &extra, [150, 180, 255], 1, true);
    } else {
        u.text(18, ty + 8, "HOVER A SKILL TO SEE WHAT IT DOES. EARN POINTS BY LEVELLING UP.", [200, 190, 190], 1, false);
        u.text(18, ty + 18, "EACH TIER NEEDS THE ONE ABOVE IT AND A MINIMUM LEVEL.", [160, 150, 160], 1, false);
    }
}

''')

block("    let p = &g.player;\n    // orbs", "    // gold\n", r'''    let p = &g.player;
    // orbs
    let r = 22;
    orb(&mut u, r + 8, fh - r - 6, r, (p.hp / p.maxhp).clamp(0.0, 1.0), [200, 30, 36], t);
    orb(&mut u, fw - r - 8, fh - r - 6, r, (p.mana / p.maxmana).clamp(0.0, 1.0), [50, 90, 230], t + 40);
    u.text_c(r + 8, fh - 2 * r - 17, &format!("{}", p.hp.max(0.0).ceil() as i32), [255, 220, 220], 1);
    u.text_c(fw - r - 8, fh - 2 * r - 17, &format!("{}", p.mana as i32), [210, 220, 255], 1);
    // potion
    let (qx, qy) = (2 * r + 18, fh - 18);
    for dy in 0..8 {
        for dx in -2i32..=2 {
            if dy < 2 && dx != 0 {
                continue;
            }
            u.px(qx + dx, qy + dy, if dy < 2 { [160, 140, 120] } else { [220, 40, 60] }, 255);
        }
    }
    u.text(qx + 5, qy + 2, &format!("{} F", p.potions), [255, 200, 200], 1, true);
    // spell bar: every learned spell
    let bar = p.sk.bar();
    let n = bar.len() as i32;
    let sw = 20;
    let total = n * sw + (n - 1) * 3;
    let bx = fw / 2 - total / 2;
    let by = fh - sw - 6;
    u.rect(bx - 4, by - 4, total + 8, sw + 8, [16, 10, 16], 200);
    for (k, &s) in bar.iter().enumerate() {
        let x = bx + k as i32 * (sw + 3);
        let nd = node(s);
        let sel = p.sel == k;
        let rim = if sel { [240, 196, 80] } else { [90, 74, 84] };
        u.rect(x - 1, by - 1, sw + 2, sw + 2, rim, 255);
        u.rect(x, by, sw, sw, [30, 22, 30], 255);
        icon(&mut u, x + 2, by + 2, s, t);
        let cd = p.cd[s as usize];
        if cd > 0 {
            let fr = cd as f32 / nd.cd as f32;
            u.rect(x, by, sw, (sw as f32 * fr) as i32, [0, 0, 0], 160);
        }
        if p.mana < nd.mana {
            u.rect(x, by, sw, sw, [20, 30, 90], 120);
        }
        if k < 9 {
            u.text(x + 1, by + 1, &format!("{}", k + 1), [230, 220, 200], 1, true);
        }
    }
    if let Some(&s) = bar.get(p.sel) {
        u.text_c(fw / 2, by - 17, node(s).name, [240, 214, 150], 1);
    }
    // experience
    let need = xp_need(p.sk.level);
    let xw = total.max(120);
    let xx = fw / 2 - xw / 2;
    u.rect(xx, by - 8, xw, 3, [24, 18, 26], 230);
    u.rect(xx, by - 8, (xw as f32 * p.sk.xp as f32 / need as f32) as i32, 3, [230, 190, 80], 255);
    u.text(xx - Ui::text_w(&format!("LV {}", p.sk.level), 1) - 4, by - 9, &format!("LV {}", p.sk.level), [240, 214, 150], 1, true);
    if p.sk.points > 0 && (t / 20) % 2 == 0 {
        u.text(xx + xw + 4, by - 9, &format!("+{} K", p.sk.points), [255, 230, 120], 1, true);
    }
''')
R('"1-4 SPELLS   SPACE DODGE   F POTION"', '"1-9 SPELLS   SPACE DODGE   F POTION   K SKILLS"')
R("    // cursor\n", "    if g.tree {\n        draw_tree(g, &mut u, fw, fh);\n    }\n    // cursor\n")
open(p, 'w', encoding='utf8').write(s)
print("hud ok")
