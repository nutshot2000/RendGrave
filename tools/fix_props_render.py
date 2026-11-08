def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)

edit('src/render.rs', [
    ("""    // ---- waypoint: a rune circle that glows brighter once found""", """    // ---- chests: wooden boxes, iron-bound for the good ones; the lid swings open
    for ch in &g.chests {
        let (sx, sy) = (ch.x as i32 - cx, ch.y as i32 - cy);
        let iron = ch.tier > 0;
        let wood = if iron { [96, 64, 40] } else { [122, 82, 48] };
        let band = if iron { [150, 150, 160] } else { [80, 56, 34] };
        for dx in -6..=6 {
            for k in 0..7 {
                let edge = dx.abs() == 6 || k == 0;
                let c = if (dx.abs() == 3 || k == 3) && iron { band } else if edge { scl(wood, 0.7) } else { scl(wood, 0.9 + (dx + 6) as f32 * 0.012) };
                c.put(sx + dx, sy - k, ch.y, c, [0; 3], k as f32);
            }
        }
        let open = ch.open;
        let lid = if open { (ch.t as f32 / 10.0).min(1.0) } else { 0.0 };
        for dx in -6..=6 {
            for k in 0..4 {
                // the lid rises and tips back as it opens
                let (yy, zz) = (sy - 7 - k - (lid * 3.0) as i32, 7.0 + k as f32 + lid * 3.0);
                let c = if k == 3 { scl(wood, 1.1) } else if iron && dx.abs() == 3 { band } else { wood };
                if !(open && lid >= 1.0 && k > 1) {
                    c.put(sx + dx, yy - (lid * k as f32) as i32, ch.y - 0.1, c, [0; 3], zz);
                }
            }
        }
        if !open {
            c.put(sx, sy - 6, ch.y + 0.2, [230, 196, 90], [40, 30, 6], 6.0);
            if (t / 40) % 3 == 0 {
                c.glow(sx + ((t / 3) % 10) as i32 - 5, sy - 10, [60, 50, 20]);
            }
        } else {
            for dx in -4..=4 {
                c.put(sx + dx, sy - 7, ch.y + 0.1, [36, 26, 18], [0; 3], 7.0);
            }
        }
    }
    // ---- shrines: a carved stone pillar with a floating light
    for s in &g.shrines {
        let (sx, sy) = (s.x as i32 - cx, s.y as i32 - cy);
        let col = crate::props::SHRINES[s.k as usize].2;
        for dy in -5..=5 {
            for dx in -9..=9 {
                if (dx * dx) as f32 / 81.0 + (dy * dy) as f32 / 25.0 < 1.0 {
                    c.put(sx + dx, sy + dy, s.y - 20.0, [70, 68, 74], [0; 3], 0.5);
                }
            }
        }
        for k in 0..20 {
            let half = if k < 3 { 5 } else if k > 16 { 4 } else { 3 };
            for dx in -half..=half {
                let rune = !s.used && dx == 0 && k % 4 == 1 && k > 3;
                let base = if dx < 0 { [120, 118, 126] } else { [92, 90, 100] };
                c.put(sx + dx, sy - k, s.y, if rune { col } else { base }, if rune { scl(col, 0.8) } else { [0; 3] }, k as f32);
            }
        }
        let bob = ((t as f32 * 0.05).sin() * 2.0) as i32;
        let orb = if s.used { [60, 58, 64] } else { col };
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                if dx * dx + dy * dy <= 5 {
                    c.put(sx + dx, sy - 26 + dy + bob, s.y + 0.2, orb, if s.used { [0; 3] } else { scl(col, 1.0) }, 26.0);
                }
            }
        }
        if !s.used {
            for k in 0..3 {
                let a = t as f32 * 0.04 + k as f32 * 2.1;
                c.glow(sx + (a.cos() * 8.0) as i32, sy - 22 + (a.sin() * 3.0) as i32 + bob, scl(col, 0.7));
            }
        }
    }
    // ---- waypoint: a rune circle that glows brighter once found"""),
    ("shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k == Kind::Brute { 11.0 } else { 7.0 });",
     "shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k.big() { 11.0 } else if e.k == Kind::Bloater { 10.0 } else { 7.0 });"),
    ("        if p.k == 3 {", "        if p.k == 3 || p.k == 4 {"),
])
edit('src/hud.rs', [
    ("""    for e in &g.enemies {
        if e.alive {""", """    for ch in g.chests.iter().filter(|c| !c.open) {
        u.px(ox + (ch.x / sx) as i32, oy + (ch.y / sy) as i32, [255, 210, 60], 255);
    }
    for s in g.shrines.iter().filter(|s| !s.used) {
        let c = crate::props::SHRINES[s.k as usize].2;
        let (x, y) = (ox + (s.x / sx) as i32, oy + (s.y / sy) as i32);
        for &(dx, dy) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            u.px(x + dx, y + dy, c, 255);
        }
    }
    for e in &g.enemies {
        if e.alive {"""),
])
print("props render ok")
