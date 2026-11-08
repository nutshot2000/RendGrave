def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/areagen.rs', [
    ("""    /// (x, y, kind): 0 powder keg, 1 oil, 2 acid
    pub barrels: Vec<(f32, f32, u8)>,
}""", """    /// (x, y, kind): 0 powder keg, 1 oil, 2 acid
    pub barrels: Vec<(f32, f32, u8)>,
    /// the merchant's stall
    pub merchant: Option<(f32, f32)>,
}"""),
    ("chests: vec![], shrines: vec![], barrels: vec![] }", "chests: vec![], shrines: vec![], barrels: vec![], merchant: None }"),
    ("""    waypoint(w, &mut sp, 552.0, 418.0);""", """    waypoint(w, &mut sp, 552.0, 418.0);
    // the merchant's stall, just off the square on the east road
    let (mx, my) = (600.0, 374.0);
    clearing(w, mx, my, 26.0, 14.0, DIRT);
    w.rect(mx as i32 - 14, my as i32 - 2, mx as i32 + 15, my as i32 + 2, WOOD);
    sp.merchant = Some((mx, my));
    sp.lamps.push((mx, my - 10.0, [1.0, 0.75, 0.4]));"""),
])

G = 'src/game.rs'
edit(G, [
    ("    pub inv_open: bool,\n", "    pub inv_open: bool,\n    pub shop_open: bool,\n    pub shop: Vec<Item>,\n    pub shop_hover: Option<(u8, usize)>,\n    pub merchant: Option<(f32, f32)>,\n"),
    ("            inv_open: false,\n", "            inv_open: false,\n            shop_open: false,\n            shop: vec![],\n            shop_hover: None,\n            merchant: None,\n"),
    ("""        if self.inv_open {
            if inp.close && !self.prev.close {""", """        if self.shop_open {
            if (inp.close && !self.prev.close) || (inp.interact && !self.prev.interact) || (inp.inv && !self.prev.inv) {
                self.shop_open = false;
                self.hold_block = true;
            } else {
                self.update_shop(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.inv_open {
            if inp.close && !self.prev.close {"""),
])

edit('src/areas.rs', [
    ("        self.waypoint = sp.waypoint;\n", "        self.waypoint = sp.waypoint;\n        self.merchant = sp.merchant;\n        self.shop_open = false;\n        if to == AreaId::Ashford {\n            self.restock();\n        }\n"),
    ("""        if self.update_props(press) {
            return;
        }""", """        if let Some((mx, my)) = self.merchant {
            if ((self.player.x - mx).powi(2) + ((self.player.y - my) * 1.4).powi(2)).sqrt() < 34.0 {
                self.prompt = Some("E  TRADE WITH THE MERCHANT".into());
                if press {
                    self.shop_open = true;
                    self.hold_block = true;
                    self.snd(Sfx::Coin, mx, my, 0.7);
                }
                return;
            }
        }
        if self.update_props(press) {
            return;
        }"""),
])

edit('src/main.rs', [
    ("mod areas;\n", "mod areas;\nmod shop;\n"),
    ("let duck = if g.paused || g.tree || g.wp_open || g.inv_open { 0.45 } else { 1.0 };", "let duck = if g.paused || g.tree || g.wp_open || g.inv_open || g.shop_open { 0.45 } else { 1.0 };"),
])

H = 'src/hud.rs'
s = open(H, encoding='utf8').read()
s += r'''
// ------------------------------------------------------------------ merchant
fn shop_cell(k: usize) -> (i32, i32, i32) {
    (24 + (k % 4) as i32 * 27, 50 + (k / 4) as i32 * 27, 22)
}
fn potion_button() -> (i32, i32, i32, i32) {
    (24, 112, 103, 16)
}
/// (0 stock, k) / (1 bag, k) / (2 potion, 0)
pub fn shop_at(m: (f32, f32), view: (f32, f32)) -> Option<(u8, usize)> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    let (mx, my) = (m.0 as i32, m.1 as i32);
    for k in 0..crate::shop::STOCK {
        let (x, y, s) = shop_cell(k);
        if mx >= x && mx < x + s && my >= y && my < y + s {
            return Some((0, k));
        }
    }
    let (x, y, w, h) = potion_button();
    if mx >= x && mx < x + w && my >= y && my < y + h {
        return Some((2, 0));
    }
    for k in 0..BAG_SIZE {
        let (x, y, s) = inv_cell(1, k, fw, fh);
        if mx >= x && mx < x + s && my >= y && my < y + s {
            return Some((1, k));
        }
    }
    None
}

fn draw_shop(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let p = &g.player;
    u.rect(0, 0, fw, fh, [10, 6, 6], 228);
    u.text(12, 10, "MERCHANT", [240, 214, 150], 2, true);
    let gs = format!("{} GOLD", p.gold);
    u.text(fw - Ui::text_w(&gs, 1) - 12, 12, &gs, [255, 214, 90], 1, true);
    u.text(fw - Ui::text_w("E CLOSE", 1) - 12, 22, "E CLOSE", [150, 140, 150], 1, true);
    u.text(24, 38, "FOR SALE", [200, 180, 150], 1, false);
    for k in 0..crate::shop::STOCK {
        let (x, y, s) = shop_cell(k);
        let hover = g.shop_hover == Some((0, k));
        let it = g.shop.get(k);
        u.rect(x - 1, y - 1, s + 2, s + 2, if hover { [255, 255, 255] } else if let Some(i) = it { mul_c(i.rarity.col(), 0.8) } else { [50, 42, 54] }, 255);
        u.rect(x, y, s, s, [22, 16, 18], 255);
        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
            if g.is_upgrade(it) {
                for k in 0..4 {
                    u.rect(x + s - 6 - k, y + 2 + k, 1 + k * 2, 1, [90, 240, 90], 255);
                }
                u.rect(x + s - 7, y + 6, 3, 3, [90, 240, 90], 255);
            }
        }
    }
    let (bx, by, bw, bh) = potion_button();
    let hover = g.shop_hover == Some((2, 0));
    u.rect(bx - 1, by - 1, bw + 2, bh + 2, if hover { [255, 255, 255] } else { [160, 40, 50] }, 255);
    u.rect(bx, by, bw, bh, [36, 14, 18], 255);
    disc(u, bx + 8, by + 9, 4.0, [220, 40, 60]);
    u.rect(bx + 6, by + 2, 4, 3, [180, 170, 160], 255);
    u.text(bx + 16, by + 5, &format!("POTION  {} GOLD", g.potion_price()), [240, 220, 200], 1, false);
    u.text(24, 134, &format!("POTIONS CARRIED {}/{}", p.potions, crate::shop::MAX_POTIONS), [170, 150, 150], 1, false);
    u.text(24, 150, "NEW STOCK EACH TIME", [120, 110, 120], 1, false);
    u.text(24, 159, "YOU RETURN TO ASHFORD", [120, 110, 120], 1, false);
    // your bag
    let (gx, _, _) = inv_cell(1, 0, fw, fh);
    u.text(gx, 38, "YOUR BAG", [200, 180, 150], 1, false);
    for k in 0..BAG_SIZE {
        let (x, y, s) = inv_cell(1, k, fw, fh);
        let hover = g.shop_hover == Some((1, k));
        let it = p.bag.get(k);
        u.rect(x - 1, y - 1, s + 2, s + 2, if hover { [255, 255, 255] } else if let Some(i) = it { mul_c(i.rarity.col(), 0.7) } else { [50, 42, 54] }, 255);
        u.rect(x, y, s, s, [18, 13, 20], 255);
        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
        }
    }
    let (_, ly, _) = inv_cell(1, BAG_SIZE - 1, fw, fh);
    u.text(gx, ly + 30, "CLICK TO BUY OR EQUIP   RIGHT CLICK TO SELL", [150, 140, 150], 1, false);
    // tooltip with the price
    let (it, price) = match g.shop_hover {
        Some((0, k)) => (g.shop.get(k), g.shop.get(k).map(|i| (format!("BUY FOR {} GOLD", Game::price(i)), p.gold >= Game::price(i)))),
        Some((1, k)) => (p.bag.get(k), p.bag.get(k).map(|i| (format!("SELLS FOR {} GOLD", i.value()), true))),
        _ => (None, None),
    };
    if let Some(it) = it {
        let mut all = item_lines(it);
        if g.is_upgrade(it) {
            all.push(("BETTER THAN WHAT YOU WEAR".into(), [120, 230, 120]));
        }
        if let Some((t, ok)) = price {
            all.push((t, if ok { [255, 214, 90] } else { [240, 110, 110] }));
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
'''
s = s.replace("""    if g.inv_open {
        draw_inventory(g, &mut u, fw, fh);
    }""", """    if g.inv_open {
        draw_inventory(g, &mut u, fw, fh);
    }
    if g.shop_open {
        draw_shop(g, &mut u, fw, fh);
    }""", 1)
open(H, 'w', encoding='utf8').write(s)

edit('src/render.rs', [
    ("""    // ---- barrels: powder kegs""", """    // ---- the merchant's stall: a counter under a striped awning, the merchant behind it
    if let Some((mx, my)) = g.merchant {
        let (sx, sy) = (mx as i32 - cx, my as i32 - cy);
        // posts
        for k in 0..30 {
            c.put(sx - 15, sy - k, my - 6.0, [92, 64, 40], [0; 3], k as f32);
            c.put(sx + 15, sy - k, my - 6.0, [80, 56, 36], [0; 3], k as f32);
        }
        // the merchant: hooded, round, holding up a lantern
        let robe = [70, 52, 96];
        for k in 0..20 {
            let half = if k < 12 { 5 } else { 4 };
            for dx in -half..=half {
                let shade = 0.8 + (dx + half) as f32 / (2 * half) as f32 * 0.35;
                c.put(sx + dx, sy - 6 - k, my - 4.0, scl(robe, shade), [0; 3], 6.0 + k as f32);
            }
        }
        for dy in 0..7 {
            for dx in -3..=3 {
                let face = dy > 1 && dy < 6 && dx.abs() < 3;
                c.put(sx + dx, sy - 26 - dy, my - 4.0, if face { [214, 172, 138] } else { [56, 42, 80] }, [0; 3], 26.0 + dy as f32);
            }
        }
        c.put(sx - 1, sy - 29, my - 3.9, [30, 20, 20], [0; 3], 29.0);
        c.put(sx + 1, sy - 29, my - 3.9, [30, 20, 20], [0; 3], 29.0);
        let flick = 0.85 + (t as f32 * 0.3).sin() * 0.1;
        c.put(sx + 8, sy - 22, my - 3.0, [255, 220, 140], scl([255, 190, 90], flick), 22.0);
        c.put(sx + 8, sy - 21, my - 3.0, [255, 200, 110], scl([255, 170, 70], flick), 21.0);
        // counter with goods on it
        for dx in -14..=14 {
            for k in 0..9 {
                let col = if k == 8 { [140, 100, 64] } else if dx % 5 == 0 { [86, 58, 36] } else { [112, 78, 48] };
                c.put(sx + dx, sy - k, my + 0.2, col, [0; 3], k as f32);
            }
        }
        for &(dx, col) in &[(-10, [220, 40, 60]), (-6, [220, 40, 60]), (2, [190, 190, 205]), (8, [230, 200, 90])] {
            c.put(sx + dx, sy - 9, my + 0.3, col, [0; 3], 9.0);
            c.put(sx + dx, sy - 10, my + 0.3, col, [0; 3], 10.0);
        }
        // striped awning
        for dx in -18..=18 {
            for k in 0..6 {
                let stripe = ((dx + 18) / 4) % 2 == 0;
                let col = if stripe { [170, 40, 38] } else { [222, 206, 170] };
                c.put(sx + dx, sy - 30 - k + (dx.abs() / 6), my - 6.5, scl(col, 0.8 + k as f32 * 0.05), [0; 3], 30.0 + k as f32);
            }
        }
    }
    // ---- barrels: powder kegs"""),
])
print("shop wired")
