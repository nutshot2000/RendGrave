
// ------------------------------------------------------------------ menus
pub fn menu_row(k: usize, fw: i32, fh: i32, title: bool) -> (i32, i32, i32, i32) {
    let (w, h) = (180, 13);
    let y0 = if title { fh / 2 - 4 } else { fh / 3 + 18 };
    (fw / 2 - w / 2, y0 + k as i32 * 16, w, h)
}
pub fn menu_at(m: (f32, f32), view: (f32, f32), n: usize, title: bool) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    (0..n).find(|&k| {
        let (x, y, w, h) = menu_row(k, fw, fh, title);
        let (mx, my) = (m.0 as i32, m.1 as i32);
        mx >= x && mx < x + w && my >= y - 1 && my < y + h + 2
    })
}
fn draw_menu(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let items = g.menu_items();
    for (k, m) in items.iter().enumerate() {
        let (x, y, w, h) = menu_row(k, fw, fh, g.title);
        let hover = g.menu_hover == Some(k);
        u.rect(x, y, w, h, if hover { [70, 40, 30] } else { [20, 13, 18] }, 225);
        u.rect(x, y + h - 1, w, 1, if hover { [255, 190, 110] } else { [70, 50, 60] }, 255);
        let txt = g.menu_text(*m);
        let col = if hover { [255, 236, 200] } else if matches!(m, crate::menu::MI::Continue | crate::menu::MI::Resume) { [240, 214, 150] } else { [200, 190, 190] };
        u.text_c(x + w / 2, y + 4, &txt, col, 1);
    }
}
fn draw_title(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let t = g.frame;
    // darken towards the edges so the menu reads over the town
    for y in 0..fh {
        let d = ((y - fh / 2) as f32 / (fh / 2) as f32).abs();
        u.rect(0, y, fw, 1, [4, 2, 6], (90.0 + d * 120.0) as u8);
    }
    let pulse = 0.85 + (t as f32 * 0.03).sin() * 0.15;
    u.text_c(fw / 2 + 2, fh / 5 + 2, "RENDGRAVE", [40, 8, 6], 4);
    u.text_c(fw / 2, fh / 5, "RENDGRAVE", [(240.0 * pulse) as u8, (200.0 * pulse) as u8, (140.0 * pulse) as u8], 4);
    u.text_c(fw / 2, fh / 5 + 30, "A PIXEL-SIMULATED ACTION RPG", [190, 170, 160], 1);
    u.text_c(fw / 2, fh / 5 + 42, "EARLY ACCESS  -  ACT I: THE HOLLOW KING", [150, 130, 130], 1);
    draw_menu(g, u, fw, fh);
    u.text_c(fw / 2, fh - 14, "WASD MOVE   LMB ATTACK   RMB CAST   SPACE DODGE   E INTERACT", [120, 110, 120], 1);
}
fn draw_ending(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    u.rect(0, 0, fw, fh, [4, 2, 6], 215);
    let p = &g.player;
    u.text_c(fw / 2, fh / 6, "ACT I COMPLETE", [240, 214, 150], 3);
    u.text_c(fw / 2, fh / 6 + 26, "THE HOLLOW KING AND THE BANDIT CHIEF ARE DEAD", [210, 190, 180], 1);
    let secs = g.played / 60;
    let lines = [
        format!("LEVEL {}", p.sk.level),
        format!("{} KILLS", p.kills),
        format!("{} GOLD", p.gold),
        format!("PLAYED {}H {:02}M", secs / 3600, (secs / 60) % 60),
    ];
    for (k, l) in lines.iter().enumerate() {
        u.text_c(fw / 2, fh / 6 + 48 + k as i32 * 11, l, [180, 170, 170], 1);
    }
    u.text_c(fw / 2, fh / 2 + 34, "ACT II - THE FROZEN NORTH - IS COMING IN A FUTURE UPDATE", [150, 190, 255], 1);
    u.text_c(fw / 2, fh / 2 + 46, "THANK YOU FOR PLAYING THE EARLY ACCESS", [200, 190, 190], 1);
    u.text_c(fw / 2, fh / 2 + 58, "KEEP EXPLORING, HUNT CHAMPIONS AND LEGENDARIES, OR STOP HERE", [150, 140, 150], 1);
    if (g.frame / 30) % 2 == 0 {
        u.text_c(fw / 2, fh - 30, "CLICK TO CONTINUE", [255, 214, 90], 1);
    }
}
