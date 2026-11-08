def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/game.rs', [("""        let mut g = Game::blank(seed_);
        g.travel(AreaId::Pines);
        g""", """        let mut g = Game::blank(seed_);
        g.apply_gear();
        g.travel(AreaId::Pines);
        g""")])

R = 'src/render.rs'
edit(R, [
    ("use crate::world::*;", "use crate::world::*;\nuse crate::items::{Rarity, Slot};"),
    ("""    // ---- gibs""", """    // ---- loot on the ground: a little shape in its rarity colour, rare and better throw a beam of light
    for gi in &g.ground {
        if gi.t < 0 {
            continue;
        }
        let it = &gi.item;
        let rc = it.rarity.col();
        let (sx, sy) = (gi.x as i32 - cx, (gi.y - gi.z) as i32 - cy);
        let shape: &[(i32, i32)] = match it.slot {
            Slot::Weapon => &[(-4, 0), (-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, -1), (-3, -1), (-3, 1)],
            Slot::Helm => &[(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, -1), (0, -1), (1, -1)],
            Slot::Armor => &[(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, -1), (0, -1), (1, -1), (-1, 1), (0, 1), (1, 1)],
            Slot::Ring => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
        };
        let pulse = 0.6 + (t as f32 * 0.1 + gi.x).sin() * 0.3;
        for &(dx, dy) in shape {
            c.put(sx + dx, sy + dy, gi.y + 0.2, mixc([160, 160, 170], rc, 0.5), scl(rc, 0.35 * pulse), 1.0);
        }
        if it.rarity >= Rarity::Rare {
            for k in 1..40 {
                let a = (1.0 - k as f32 / 40.0) * pulse;
                c.glow(sx, sy - k, scl(rc, a * 0.5));
                if k % 2 == 0 {
                    c.glow(sx - 1, sy - k, scl(rc, a * 0.15));
                    c.glow(sx + 1, sy - k, scl(rc, a * 0.15));
                }
            }
        }
    }
    // ---- gibs"""),
    ("""            windup: e.st == St::Wind && e.k.big(),
            burn: false,""", """            windup: e.st == St::Wind && e.k.big(),
            burn: false,
            elite: e.elite,"""),
    ("""    pub windup: bool,
    pub burn: bool,
}""", """    pub windup: bool,
    pub burn: bool,
    pub elite: bool,
}"""),
    ("""            if tint.burn && h2(lx, ly + (t / 3) as i32, 9) < 0.15 {""", """            if tint.elite {
                let edge = p.get(lx - 1, ly).is_none() || p.get(lx + 1, ly).is_none() || p.get(lx, ly - 1).is_none();
                if edge {
                    let pulse = 0.55 + ((t as f32 * 0.12) + ly as f32 * 0.3).sin() * 0.25;
                    e = [(255.0 * pulse) as u8, (190.0 * pulse) as u8, (60.0 * pulse) as u8];
                }
            }
            if tint.burn && h2(lx, ly + (t / 3) as i32, 9) < 0.15 {"""),
    ("""    for e in &g.enemies {
        if e.alive && !matches!(e.k, Kind::Zombie | Kind::Crawler) && e.frozen == 0 {""", """    for gi in &g.ground {
        if gi.item.rarity >= Rarity::Rare {
            let rc = gi.item.rarity.col();
            add(gi.x, gi.y, 45.0, 0.7, [rc[0] as f32 / 255.0, rc[1] as f32 / 255.0, rc[2] as f32 / 255.0]);
        }
    }
    for e in &g.enemies {
        if e.alive && e.elite {
            add(e.x, e.y, 60.0, 0.6, [1.0, 0.75, 0.3]);
        }
        if e.alive && !matches!(e.k, Kind::Zombie | Kind::Crawler) && e.frozen == 0 {"""),
])

H = 'src/hud.rs'
edit(H, [
    ("use crate::gen::AreaId;" if False else "use crate::areagen::AreaId;", "use crate::areagen::AreaId;\nuse crate::items::*;\nuse crate::gear::BAG_SIZE;"),
    # elite names and gold bars
    ("""        u.rect(sx - w / 2, sy, fw_, 2, if e.frozen > 0 { [140, 200, 255] } else { [200, 40, 40] }, a);""", """        u.rect(sx - w / 2, sy, fw_, 2, if e.frozen > 0 { [140, 200, 255] } else if e.elite { [240, 180, 60] } else { [200, 40, 40] }, a);
        if e.elite {
            u.text_c(sx, sy - 8, "CHAMPION", [255, 200, 90], 1);
        }"""),
    # names over loot near you
    ("""    // floating numbers""", """    // loot names near the player
    for gi in &g.ground {
        let d = ((gi.x - g.player.x).powi(2) + (gi.y - g.player.y).powi(2)).sqrt();
        if gi.t < 10 || d > 110.0 {
            continue;
        }
        let (sx, sy) = (gi.x as i32 - cx, (gi.y - gi.z) as i32 - cy - 12);
        let w = Ui::text_w(&gi.item.name, 1);
        u.rect(sx - w / 2 - 2, sy - 2, w + 4, 9, [8, 5, 10], 170);
        u.text(sx - w / 2, sy, &gi.item.name, gi.item.rarity.col(), 1, false);
    }
    // floating numbers"""),
    ("""    if g.wp_open {
        draw_waypoints(g, &mut u, fw, fh);
    }""", """    if g.wp_open {
        draw_waypoints(g, &mut u, fw, fh);
    }
    if g.inv_open {
        draw_inventory(g, &mut u, fw, fh);
    }"""),
    ('"AIM AT LEGS TO TAKE THEM OFF   E INTERACT"', '"AIM AT LEGS TO TAKE THEM OFF   E INTERACT   I GEAR"'),
    ("""/// the waypoint list: (x, y, w, h) of row k""", r'''/// a 16x16 picture of an item
pub fn item_icon(u: &mut Ui, x: i32, y: i32, it: &Item) {
    let rc = it.rarity.col();
    let steel = match it.rarity {
        Rarity::Common => [176, 182, 196],
        Rarity::Magic => [160, 182, 240],
        Rarity::Rare => [230, 210, 130],
        Rarity::Legendary => [255, 150, 70],
    };
    let wood = [110, 72, 44];
    match it.slot {
        Slot::Weapon => match it.wk {
            WKind::Sword => {
                line(u, x + 3, y + 13, x + 13, y + 3, steel);
                line(u, x + 4, y + 13, x + 14, y + 3, [120, 126, 140]);
                line(u, x + 2, y + 10, x + 6, y + 14, [214, 180, 80]);
                line(u, x + 1, y + 15, x + 3, y + 13, wood);
            }
            WKind::Axe => {
                line(u, x + 4, y + 15, x + 11, y + 2, wood);
                for k in 0..6 {
                    line(u, x + 9 + k / 2, y + 1 + k, x + 14, y + 2 + k, steel);
                }
            }
            WKind::Hammer => {
                line(u, x + 4, y + 15, x + 10, y + 5, wood);
                for k in 0..5 {
                    line(u, x + 6 + k, y + 1 + k, x + 12 + k / 2, y + k, steel);
                }
                u.rect(x + 7, y + 1, 7, 5, steel, 255);
            }
            WKind::Spear => {
                line(u, x + 2, y + 15, x + 12, y + 3, wood);
                line(u, x + 12, y + 3, x + 15, y, steel);
                line(u, x + 11, y + 2, x + 14, y + 1, steel);
            }
            WKind::Dagger => {
                line(u, x + 6, y + 11, x + 12, y + 5, steel);
                line(u, x + 4, y + 9, x + 8, y + 13, [200, 170, 90]);
                line(u, x + 3, y + 14, x + 5, y + 12, wood);
            }
        },
        Slot::Helm => {
            for dy in 0..8 {
                let w = 6 - (7 - dy) / 2;
                u.rect(x + 8 - w, y + 4 + dy, w * 2, 1, if dy < 3 { mul_c(steel, 1.15) } else { steel }, 255);
            }
            u.rect(x + 4, y + 9, 8, 1, [30, 26, 34], 255);
        }
        Slot::Armor => {
            u.rect(x + 3, y + 3, 10, 11, steel, 255);
            u.rect(x + 1, y + 3, 3, 4, steel, 255);
            u.rect(x + 12, y + 3, 3, 4, steel, 255);
            u.rect(x + 6, y + 3, 4, 2, [30, 26, 34], 255);
            u.rect(x + 7, y + 6, 2, 7, mul_c(steel, 0.8), 255);
        }
        Slot::Ring => {
            for k in 0..20 {
                let a = k as f32 / 20.0 * std::f32::consts::TAU;
                u.px(x + 8 + (a.cos() * 4.0) as i32, y + 9 + (a.sin() * 4.0) as i32, [220, 180, 80], 255);
            }
            disc(u, x + 8, y + 4, 1.6, rc);
        }
    }
}
fn mul_c(c: [u8; 3], f: f32) -> [u8; 3] {
    [(c[0] as f32 * f).min(255.0) as u8, (c[1] as f32 * f).min(255.0) as u8, (c[2] as f32 * f).min(255.0) as u8]
}

/// inventory layout: equipment slot k / bag cell k -> (x, y, size)
pub fn inv_cell(kind: u8, k: usize, fw: i32, _fh: i32) -> (i32, i32, i32) {
    let s = 22;
    if kind == 0 {
        (24, 44 + k as i32 * 30, s)
    } else {
        let gx = (fw / 2 - 100).max(150);
        (gx + (k % 8) as i32 * (s + 3), 44 + (k / 8) as i32 * (s + 3), s)
    }
}
pub fn inv_at(m: (f32, f32), view: (f32, f32)) -> Option<(u8, usize)> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    let (mx, my) = (m.0 as i32, m.1 as i32);
    for kind in 0..2u8 {
        let n = if kind == 0 { 4 } else { BAG_SIZE };
        for k in 0..n {
            let (x, y, s) = inv_cell(kind, k, fw, fh);
            if mx >= x && mx < x + s && my >= y && my < y + s {
                return Some((kind, k));
            }
        }
    }
    None
}

fn item_lines(it: &Item) -> Vec<(String, [u8; 3])> {
    let mut l = vec![(it.name.clone(), it.rarity.col())];
    l.push((format!("{:?} {}   ITEM LEVEL {}", it.rarity, it.kind_text(), it.ilvl).to_uppercase(), [150, 140, 150]));
    let b = it.base_text();
    if !b.is_empty() {
        l.push((b, [235, 230, 220]));
    }
    if it.slot == Slot::Weapon {
        let ws = wstats(it.wk);
        let feel = match it.wk {
            WKind::Sword => "BALANCED SLICES",
            WKind::Axe => "SLOW DEEP CHOPS",
            WKind::Hammer => "CRUSHES BONE AND ICE",
            WKind::Spear => "LONG THRUSTS THAT PIERCE TWO",
            WKind::Dagger => "FAST CUTS THAT BLEED",
        };
        l.push((format!("{}  -  {:.1} SWINGS A SECOND", feel, 60.0 / ws.frames as f32), [170, 160, 170]));
    }
    for a in &it.affixes {
        l.push((a.text(), [120, 160, 255]));
    }
    if let Some(t) = it.legend_text() {
        l.push((t.to_string(), [255, 150, 70]));
    }
    l.push((format!("SELLS FOR {} GOLD", it.value()), [200, 170, 90]));
    l
}

fn draw_inventory(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let p = &g.player;
    u.rect(0, 0, fw, fh, [8, 5, 10], 228);
    u.text(12, 10, "GEAR", [240, 214, 150], 2, true);
    let gs = format!("{} GOLD", p.gold);
    u.text(fw - Ui::text_w(&gs, 1) - 12, 12, &gs, [255, 214, 90], 1, true);
    u.text(fw - Ui::text_w("I CLOSE", 1) - 12, 22, "I CLOSE", [150, 140, 150], 1, true);
    // equipment
    let names = ["WEAPON", "HELM", "ARMOUR", "RING"];
    for k in 0..4 {
        let (x, y, s) = inv_cell(0, k, fw, fh);
        let hover = g.inv_hover == Some((0, k));
        let it = p.equip[k].as_ref();
        u.rect(x - 1, y - 1, s + 2, s + 2, if hover { [255, 255, 255] } else { it.map(|i| i.rarity.col()).unwrap_or([70, 60, 74]) }, 255);
        u.rect(x, y, s, s, [22, 16, 24], 255);
        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
        }
        u.text(x + s + 6, y + 3, names[k], [150, 140, 150], 1, false);
        if let Some(it) = it {
            let n: String = it.name.chars().take(18).collect();
            u.text(x + s + 6, y + 12, &n, it.rarity.col(), 1, false);
        }
    }
    // your numbers
    let gst = &p.gear;
    let wk = p.equip[0].as_ref().map(|w| w.wk).unwrap_or(WKind::Sword);
    let dmg = gst.weapon_dmg * wstats(wk).dmg_mult * (1.0 + gst.dmg_pct / 100.0);
    let red = (gst.armor / (gst.armor + 60.0) * 100.0) as i32;
    let stats = [
        format!("DAMAGE {}", dmg.round() as i32),
        format!("ARMOUR {}  (-{}% DAMAGE)", gst.armor as i32, red),
        format!("HEALTH {}   MANA {}", p.maxhp as i32, p.maxmana as i32),
        format!("CRIT {}%   ATTACK SPEED +{}%", (15.0 + gst.crit) as i32, gst.swift as i32),
    ];
    for (k, s) in stats.iter().enumerate() {
        u.text(24, 172 + k as i32 * 10, s, [210, 200, 190], 1, false);
    }
    // bag
    for k in 0..BAG_SIZE {
        let (x, y, s) = inv_cell(1, k, fw, fh);
        let hover = g.inv_hover == Some((1, k));
        let it = p.bag.get(k);
        u.rect(x - 1, y - 1, s + 2, s + 2, if hover { [255, 255, 255] } else if let Some(i) = it { mul_c(i.rarity.col(), 0.7) } else { [50, 42, 54] }, 255);
        u.rect(x, y, s, s, [18, 13, 20], 255);
        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
        }
    }
    let (bx, by, _) = inv_cell(1, BAG_SIZE - 1, fw, fh);
    u.text(inv_cell(1, 0, fw, fh).0, by + 30, "CLICK TO EQUIP OR TAKE OFF   RIGHT CLICK TO SELL", [150, 140, 150], 1, false);
    let _ = bx;
    // tooltip
    let hovered = match g.inv_hover {
        Some((0, k)) => p.equip[k].as_ref(),
        Some((1, k)) => p.bag.get(k),
        _ => None,
    };
    if let Some(it) = hovered {
        let lines = item_lines(it);
        let mut all = lines;
        // compare with what you're wearing in that slot
        if matches!(g.inv_hover, Some((1, _))) {
            let s = SLOTS.iter().position(|x| *x == it.slot).unwrap();
            match &p.equip[s] {
                Some(cur) => {
                    let d = it.base - cur.base;
                    let what = if it.slot == Slot::Weapon { "DAMAGE" } else { "ARMOUR" };
                    if it.slot != Slot::Ring {
                        all.push((format!("{}{} {} VS EQUIPPED", if d >= 0 { "+" } else { "" }, d, what), if d >= 0 { [120, 230, 120] } else { [240, 110, 110] }));
                    }
                }
                None => all.push(("SLOT IS EMPTY".into(), [120, 230, 120])),
            }
        }
        let w = all.iter().map(|l| Ui::text_w(&l.0, 1)).max().unwrap_or(60) + 12;
        let h = all.len() as i32 * 9 + 8;
        let (mx, my) = (g.prev.mouse.0 as i32 + 12, g.prev.mouse.1 as i32 + 8);
        let x = mx.min(fw - w - 4);
        let y = my.min(fh - h - 4);
        u.rect(x, y, w, h, [16, 11, 18], 245);
        u.rect(x, y, w, 1, it.rarity.col(), 255);
        for (k, (t, c)) in all.iter().enumerate() {
            u.text(x + 6, y + 5 + k as i32 * 9, t, *c, 1, false);
        }
    }
}

/// the waypoint list: (x, y, w, h) of row k'''),
])
print("gear view ok")
