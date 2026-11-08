fn draw_tree(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let t = g.frame;
    let sk = &g.player.sk;
    let tab = g.tree_tab.min(5);
    u.rect(0, 0, fw, fh, [8, 5, 10], 225);
    u.text(12, 10, "SKILLS", [240, 214, 150], 2, true);
    let info = format!("LEVEL {}    SKILL POINTS {}", sk.level, sk.points);
    u.text(fw - Ui::text_w(&info, 1) - 12, 10, &info, if sk.points > 0 { [255, 220, 120] } else { [200, 190, 190] }, 1, true);
    u.text(fw - Ui::text_w("A D OR CLICK A TAB    K CLOSE", 1) - 12, 19, "A D OR CLICK A TAB    K CLOSE", [150, 140, 150], 1, true);
    // tabs: school name and points spent; a dot where something can be learned
    for (c, (name, col)) in SCHOOLS.iter().enumerate() {
        let (x, y, w, h) = tab_rect(c, fw);
        let on = c == tab;
        u.rect(x, y, w, h, if on { mul_c(*col, 0.35) } else { [20, 15, 22] }, 255);
        u.rect(x, y + h - 1, w, 1, if on { *col } else { [60, 50, 64] }, 255);
        let spent = sk.spent(c);
        let label = if spent > 0 { format!("{} {}", name, spent) } else { name.to_string() };
        u.text_c(x + w / 2, y + 4, &label, if on { [255, 250, 240] } else { mul_c(*col, 0.8) }, 1);
        let can = (0..N).any(|i| NODES[i].col == c && sk.can_learn(i).is_ok());
        if can && (t / 15) % 2 == 0 {
            u.rect(x + w - 5, y + 2, 3, 3, [255, 214, 90], 255);
        }
    }
    let scol = SCHOOLS[tab].1;
    // links from each skill to the ones it opens
    for i in 0..N {
        let n = &NODES[i];
        if n.col != tab {
            continue;
        }
        let (x, y, bs) = node_pos(i, fw, fh);
        for r in n.req {
            let j = *r as usize;
            let (px, py, _) = node_pos(j, fw, fh);
            let lit = sk.rank[j] > 0;
            let c = if lit { mul_c(scol, if sk.rank[i] > 0 { 1.0 } else { 0.6 }) } else { [55, 46, 58] };
            let (ax, ay) = (px + bs / 2, py + bs + 6);
            let (bx, by) = (x + bs / 2, y - 2);
            let my = (ay + by) / 2;
            u.rect(ax, ay, 1, (my - ay).max(1), c, 255);
            u.rect(ax.min(bx), my, (bx - ax).abs() + 1, 1, c, 255);
            u.rect(bx, my, 1, (by - my).max(1), c, 255);
        }
    }
    for i in 0..N {
        let n = &NODES[i];
        if n.col != tab {
            continue;
        }
        let (x, y, bs) = node_pos(i, fw, fh);
        let r = sk.rank[i];
        let can = sk.can_learn(i).is_ok();
        let hover = g.tree_hover == Some(i);
        let rim = if hover {
            [255, 255, 255]
        } else if r > 0 {
            scol
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
            if needs_tome(i) && !sk.tomes[i] {
                u.text(x + 2, y + bs - 7, "?", [200, 160, 90], 1, true);
            }
        }
        // rank pips
        for k in 0..n.max as i32 {
            let px = x + bs / 2 - n.max as i32 * 2 + k * 4;
            u.rect(px, y + bs + 3, 3, 2, if (k as u8) < r { [255, 214, 90] } else { [60, 50, 60] }, 255);
        }
        if n.active {
            u.px(x + bs - 3, y + 2, [255, 255, 255], 200);
        }
        let nc = if r > 0 { [240, 230, 215] } else if can { [220, 200, 150] } else { [120, 110, 120] };
        u.text(x + bs + 5, y + 3, n.name, nc, 1, false);
        let kind = if n.active { "SPELL" } else { "PASSIVE" };
        u.text(x + bs + 5, y + 12, &format!("{}  LV {}", kind, min_level(i)), [110, 100, 112], 1, false);
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
        let rk = format!("{}   LEVEL {}   RANK {}/{}", kind, min_level(i), sk.rank[i], n.max);
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
        u.text(18, ty + 18, "EACH SCHOOL SPLITS INTO TWO PATHS. EITHER PATH LEADS TO ITS ULTIMATE.", [160, 150, 160], 1, false);
        u.text(18, ty + 28, "ULTIMATE SPELLS ALSO NEED THEIR TOME, FOUND IN THE WORLD.", [160, 150, 160], 1, false);
    }
}

