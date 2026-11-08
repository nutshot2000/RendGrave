// HUD drawn into the unlit UI layer: health and mana orbs, spell bar, gold, potions,
// floating damage numbers, enemy health bars, banners and the cursor.
use crate::body::Kind;
use crate::game::*;
use crate::gfx::Frame;
use crate::rng::*;
use crate::skills::*;
use crate::world::*;
use crate::areagen::AreaId;
use crate::items::*;
use crate::gear::BAG_SIZE;

const FONT: &[(char, &str)] = &[
    ('A', "010101111101101"), ('B', "110101110101110"), ('C', "011100100100011"), ('D', "110101101101110"),
    ('E', "111100110100111"), ('F', "111100110100100"), ('G', "011100101101011"), ('H', "101101111101101"),
    ('I', "111010010010111"), ('J', "001001001101010"), ('K', "101101110101101"), ('L', "100100100100111"),
    ('M', "101111111101101"), ('N', "110101101101101"), ('O', "010101101101010"), ('P', "110101110100100"),
    ('Q', "010101101110011"), ('R', "110101110101101"), ('S', "011100010001110"), ('T', "111010010010010"),
    ('U', "101101101101111"), ('V', "101101101101010"), ('W', "101101111111101"), ('X', "101101010101101"),
    ('Y', "101101010010010"), ('Z', "111001010100111"), ('0', "111101101101111"), ('1', "010110010010111"),
    ('2', "110001010100111"), ('3', "110001010001110"), ('4', "101101111001001"), ('5', "111100110001110"),
    ('6', "011100111101111"), ('7', "111001010010010"), ('8', "111101111101111"), ('9', "111101111001110"),
    ('-', "000000111000000"), ('.', "000000000000010"), ('!', "010010010000010"), (':', "000010000010000"),
    ('+', "000010111010000"), ('/', "001001010100100"), ('%', "101001010100101"), (' ', "000000000000000"),
];

fn glyph(ch: char) -> &'static str {
    let ch = ch.to_ascii_uppercase();
    FONT.iter().find(|g| g.0 == ch).map(|g| g.1).unwrap_or("000000000000000")
}

#[inline]
fn pk(c: [u8; 3], a: u8) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16 | (a as u32) << 24
}

pub struct Ui<'a> {
    pub f: &'a mut Frame,
}
impl<'a> Ui<'a> {
    #[inline]
    pub fn px(&mut self, x: i32, y: i32, c: [u8; 3], a: u8) {
        if x < 0 || y < 0 || x >= self.f.w as i32 || y >= self.f.h as i32 {
            return;
        }
        let i = y as usize * self.f.w + x as usize;
        if a == 255 {
            self.f.ui[i] = pk(c, 255);
            return;
        }
        let o = self.f.ui[i];
        let oa = (o >> 24) as f32 / 255.0;
        let na = a as f32 / 255.0;
        let ra = na + oa * (1.0 - na);
        if ra <= 0.0 {
            return;
        }
        let mix = |s: u8, d: u32| ((s as f32 * na + d as f32 * oa * (1.0 - na)) / ra) as u8;
        self.f.ui[i] = pk([mix(c[0], o & 255), mix(c[1], (o >> 8) & 255), mix(c[2], (o >> 16) & 255)], (ra * 255.0) as u8);
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [u8; 3], a: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.px(xx, yy, c, a);
            }
        }
    }
    pub fn text_w(s: &str, sc: i32) -> i32 {
        s.chars().count() as i32 * 4 * sc - sc
    }
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: [u8; 3], sc: i32, shadow: bool) {
        let mut cx = x;
        for ch in s.chars() {
            let g = glyph(ch).as_bytes();
            for r in 0..5 {
                for col in 0..3 {
                    if g[(r * 3 + col) as usize] == b'1' {
                        if shadow {
                            self.rect(cx + col * sc + 1, y + r * sc + 1, sc, sc, [8, 4, 8], 220);
                        }
                        self.rect(cx + col * sc, y + r * sc, sc, sc, c, 255);
                    }
                }
            }
            cx += 4 * sc;
        }
    }
    pub fn text_c(&mut self, x: i32, y: i32, s: &str, c: [u8; 3], sc: i32) {
        let w = Self::text_w(s, sc);
        self.text(x - w / 2, y, s, c, sc, true);
    }
}

fn orb(u: &mut Ui, cx: i32, cy: i32, r: i32, frac: f32, col: [u8; 3], t: u64) {
    let rf = r as f32;
    for dy in -r - 2..=r + 2 {
        for dx in -r - 2..=r + 2 {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            let (x, y) = (cx + dx, cy + dy);
            if d > rf + 2.0 {
                continue;
            }
            if d > rf {
                // brass rim
                let lit = (dx - dy) as f32 / (rf * 1.4);
                let c = [(150.0 + lit * 60.0) as u8, (116.0 + lit * 46.0) as u8, (60.0 + lit * 20.0) as u8];
                u.px(x, y, c, 255);
                continue;
            }
            let level = cy as f32 + rf - frac * rf * 2.0 + ((dx as f32 * 0.35 + t as f32 * 0.08).sin() * 1.2);
            let inside = y as f32 >= level;
            let mut c = if inside {
                let depth = (y as f32 - level) / (rf * 2.0);
                let sw = ((dx as f32 * 0.4 + dy as f32 * 0.25 + t as f32 * 0.05).sin() * 0.5 + 0.5) * 0.15;
                [(col[0] as f32 * (1.05 - depth * 0.5 + sw)).min(255.0) as u8, (col[1] as f32 * (1.05 - depth * 0.5 + sw)).min(255.0) as u8, (col[2] as f32 * (1.05 - depth * 0.5 + sw)).min(255.0) as u8]
            } else {
                [16, 12, 18]
            };
            if inside && (y as f32 - level).abs() < 1.0 {
                c = [(c[0] as u16 + 60).min(255) as u8, (c[1] as u16 + 60).min(255) as u8, (c[2] as u16 + 60).min(255) as u8];
            }
            // glass highlight
            let hx = dx as f32 + rf * 0.35;
            let hy = dy as f32 + rf * 0.45;
            if (hx * hx + hy * hy).sqrt() < rf * 0.28 {
                c = [(c[0] as u16 / 2 + 110) as u8, (c[1] as u16 / 2 + 110) as u8, (c[2] as u16 / 2 + 115) as u8];
            }
            let shade = 1.0 - (d / rf).powi(3) * 0.45;
            u.px(x, y, [(c[0] as f32 * shade) as u8, (c[1] as f32 * shade) as u8, (c[2] as f32 * shade) as u8], 255);
        }
    }
}

fn line(u: &mut Ui, x0: i32, y0: i32, x1: i32, y1: i32, c: [u8; 3]) {
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

/// a 16x16 picture of an item
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
            if g.is_upgrade(it) {
                // a green arrow: better than what you're wearing
                for k in 0..4 {
                    u.rect(x + s - 6 - k, y + 2 + k, 1 + k * 2, 1, [90, 240, 90], 255);
                }
                u.rect(x + s - 7, y + 6, 3, 3, [90, 240, 90], 255);
            }
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

/// the waypoint list: (x, y, w, h) of row k
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

/// a small map of the whole world, top right
fn minimap(g: &Game, u: &mut Ui, fw: i32, cx: i32, cy: i32) {
    let (mw, mh) = (84, 63);
    let (ox, oy) = (fw - mw - 8, 26);
    let sx = g.world.aw as f32 / mw as f32;
    let sy = g.world.ah as f32 / mh as f32;
    u.rect(ox - 2, oy - 2, mw + 4, mh + 4, [70, 58, 66], 220);
    for y in 0..mh {
        for x in 0..mw {
            let (wx, wy) = ((x as f32 * sx) as usize, (y as f32 * sy) as usize);
            let i = wy * W + wx;
            let m = g.world.m[i];
            let g_ = g.world.g[i];
            let c = if g_ == STAIRS || g_ == STAIRSUP { [240, 220, 140] } else { match m {
                TREE => [24, 52, 28],
                ROCK => [8, 7, 10],
                GAS => [90, 120, 60],
                STONE => [86, 84, 92],
                BRICK => [120, 104, 112],
                WOOD => [110, 80, 50],
                WATER | ICE => [50, 90, 150],
                ACID => [100, 190, 50],
                FIRE => [255, 140, 40],
                OIL => [30, 26, 26],
                _ => match g.world.g[i] {
                    FLOOR => [60, 56, 64],
                    DIRT | MUD => [70, 56, 42],
                    BURNT => [30, 26, 26],
                    PEAT => [44, 34, 26],
                    _ => [36, 60, 34],
                },
            } };
            u.px(ox + x, oy + y, c, 210);
        }
    }
    // view rectangle
    let (vw, vh) = g.view;
    let (rx, ry) = (ox + (cx as f32 / sx) as i32, oy + (cy as f32 / sy) as i32);
    let (rw, rh) = ((vw / sx) as i32, (vh / sy) as i32);
    for k in 0..rw {
        u.px(rx + k, ry, [200, 190, 170], 140);
        u.px(rx + k, ry + rh, [200, 190, 170], 140);
    }
    for k in 0..rh {
        u.px(rx, ry + k, [200, 190, 170], 140);
        u.px(rx + rw, ry + k, [200, 190, 170], 140);
    }
    for ch in g.chests.iter().filter(|c| !c.open) {
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
        if e.alive {
            let (x, y) = (ox + (e.x / sx) as i32, oy + (e.y / sy) as i32);
            if x >= ox && y >= oy && x < ox + mw && y < oy + mh {
                u.px(x, y, if e.k.big() { [255, 150, 40] } else { [230, 50, 50] }, 255);
            }
        }
    }
    if let Some((wx, wy)) = g.waypoint {
        let (x, y) = (ox + (wx / sx) as i32, oy + (wy / sy) as i32);
        u.px(x, y, [120, 170, 255], 255);
        u.px(x + 1, y, [120, 170, 255], 255);
        u.px(x, y + 1, [120, 170, 255], 255);
    }
    let (px, py) = (ox + (g.player.x / sx) as i32, oy + (g.player.y / sy) as i32);
    for &(dx, dy) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        u.px(px + dx, py + dy, [255, 255, 255], 255);
    }
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
        u.text(18, ty + 18, "EACH TIER NEEDS THE ONE ABOVE IT AND A MINIMUM LEVEL. ULTIMATES NEED A TOME.", [160, 150, 160], 1, false);
    }
}

pub fn draw(g: &Game, f: &mut Frame, inp: &Input, cx: i32, cy: i32) {
    for v in f.ui.iter_mut() {
        *v = 0;
    }
    let (fw, fh) = (f.w as i32, f.h as i32);
    let t = g.frame;
    let mut u = Ui { f };
    // enemy health bars
    for e in &g.enemies {
        if !e.alive || e.hpbar == 0 {
            continue;
        }
        if e.k.boss() {
            continue;
        }
        let top = if e.k.big() { 66.0 } else { 58.0 };
        let (sx, sy) = (e.x as i32 - cx, (e.y - top) as i32 - cy);
        let w = if e.k == Kind::Brute { 30 } else { 20 };
        let a = (e.hpbar.min(40) as f32 / 40.0 * 255.0) as u8;
        u.rect(sx - w / 2 - 1, sy - 1, w + 2, 4, [10, 6, 10], a);
        let fw_ = ((e.hp / e.maxhp).clamp(0.0, 1.0) * w as f32) as i32;
        u.rect(sx - w / 2, sy, fw_, 2, if e.frozen > 0 { [140, 200, 255] } else if e.elite { [240, 180, 60] } else { [200, 40, 40] }, a);
        if e.elite {
            u.text_c(sx, sy - 8, "CHAMPION", [255, 200, 90], 1);
        }
    }
    // loot names near the player, stacked so they never overlap
    let mut placed: Vec<(i32, i32, i32, i32)> = vec![];
    let mut near: Vec<&crate::gear::GroundItem> = g.ground.iter().filter(|gi| gi.t >= 10 && ((gi.x - g.player.x).powi(2) + (gi.y - g.player.y).powi(2)).sqrt() < 110.0).collect();
    near.sort_by(|a, b| b.item.rarity.cmp(&a.item.rarity));
    for gi in near {
        let (sx, mut sy) = (gi.x as i32 - cx, (gi.y - gi.z) as i32 - cy - 12);
        let w = Ui::text_w(&gi.item.name, 1);
        let x0 = sx - w / 2 - 2;
        for _ in 0..8 {
            if placed.iter().any(|&(a, b, c, d)| x0 < a + c && x0 + w + 4 > a && sy - 2 < b + d && sy + 7 > b) {
                sy -= 10;
            } else {
                break;
            }
        }
        placed.push((x0, sy - 2, w + 4, 9));
        u.rect(x0, sy - 2, w + 4, 9, [8, 5, 10], 190);
        u.text(sx - w / 2, sy, &gi.item.name, gi.item.rarity.col(), 1, false);
    }
    // floating numbers
    for fl in &g.floats {
        let (sx, sy) = (fl.x as i32 - cx, fl.y as i32 - cy);
        let sc = if fl.big { 2 } else { 1 };
        let a = (fl.t.min(20) as f32 / 20.0 * 255.0) as u8;
        let w = Ui::text_w(&fl.txt, sc);
        let mut x = sx - w / 2;
        for ch in fl.txt.chars() {
            let gl = glyph(ch).as_bytes();
            for r in 0..5 {
                for col in 0..3 {
                    if gl[(r * 3 + col) as usize] == b'1' {
                        u.rect(x + col * sc + 1, sy + r * sc + 1, sc, sc, [10, 4, 6], a);
                        u.rect(x + col * sc, sy + r * sc, sc, sc, fl.c, a);
                    }
                }
            }
            x += 4 * sc;
        }
    }
    let p = &g.player;
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
    let total = (n * sw + (n - 1) * 3).max(0);
    let bx = fw / 2 - total / 2;
    let by = fh - sw - 6;
    if n > 0 {
        u.rect(bx - 4, by - 4, total + 8, sw + 8, [16, 10, 16], 200);
    } else {
        u.text_c(fw / 2, by + 6, "NO SPELLS YET - LEVEL UP AND PRESS K", [150, 140, 150], 1);
    }
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
    // gold
    let gs = format!("{}", p.gold);
    let gw = Ui::text_w(&gs, 2);
    for dy in 0..6 {
        for dx in 0..6 {
            let d = ((dx as f32 - 2.5).powi(2) + (dy as f32 - 2.5).powi(2)).sqrt();
            if d < 3.0 {
                u.px(fw - gw - 18 + dx, 9 + dy, if d < 1.5 { [255, 236, 140] } else { [220, 170, 50] }, 255);
            }
        }
    }
    u.text(fw - gw - 8, 8, &gs, [255, 220, 120], 2, true);
    let alive = g.enemies.iter().filter(|e| e.alive).count();
    let _ = alive;
    u.text(8, 8, &g.area.name(), [220, 205, 170], 1, true);
    u.text(8, 17, &format!("AREA LEVEL {}", g.alvl), [150, 140, 150], 1, true);
    if let Some((k, left)) = g.buff {
        let (name, _, c) = crate::props::SHRINES[k as usize];
        let s = left / 60;
        let txt = format!("{}  {}:{:02}", name.trim_start_matches("SHRINE OF "), s / 60, s % 60);
        let blink = left > 600 || (left / 15) % 2 == 0;
        if blink {
            u.text(8, 27, &txt, c, 1, true);
        }
    }
    // boss health
    if let Some(b) = g.enemies.iter().find(|e| e.alive && e.k.boss() && e.aggro) {
        let bw = (fw / 2).min(260);
        let bx = fw / 2 - bw / 2;
        u.rect(bx - 2, 62, bw + 4, 8, [10, 6, 10], 230);
        u.rect(bx, 64, (bw as f32 * (b.hp / b.maxhp).clamp(0.0, 1.0)) as i32, 4, [190, 30, 30], 255);
        u.text_c(fw / 2, 53, if b.k == Kind::King { "THE HOLLOW KING" } else { "THE BANDIT CHIEF" }, [240, 200, 120], 1);
    }
    // interaction prompt
    if let Some(pr) = &g.prompt {
        let w = Ui::text_w(pr, 1);
        u.rect(fw / 2 - w / 2 - 4, fh - 62, w + 8, 11, [10, 6, 10], 200);
        u.text(fw / 2 - w / 2, fh - 59, pr, [255, 230, 150], 1, false);
    }
    // controls hint for the first seconds
    if t < 900 {
        let a = if t > 780 { ((900 - t) as f32 / 120.0 * 255.0) as u8 } else { 255 };
        let lines = ["WASD MOVE   LMB SWORD   RMB CAST", "1-9 SPELLS   SPACE DODGE   F POTION   K SKILLS", "AIM AT LEGS TO TAKE THEM OFF   E INTERACT   I GEAR"];
        for (k, l) in lines.iter().enumerate() {
            let w = Ui::text_w(l, 1);
            let y = 26 + k as i32 * 8;
            u.rect(fw / 2 - w / 2 - 3, y - 2, w + 6, 9, [10, 6, 10], (a as f32 * 0.6) as u8);
            u.text(fw / 2 - w / 2, y, l, [230, 220, 200], 1, false);
        }
    }
    // banner
    if let Some((a, b, tt)) = &g.banner {
        let al = (*tt).min(40) as f32 / 40.0;
        let y = fh / 3;
        let col = if a == "YOU DIED" { [220, 40, 40] } else { [240, 210, 140] };
        let w = Ui::text_w(a, 3);
        u.rect(0, y - 8, fw, 40, [6, 2, 6], (al * 150.0) as u8);
        let c = [(col[0] as f32 * al) as u8, (col[1] as f32 * al) as u8, (col[2] as f32 * al) as u8];
        u.text(fw / 2 - w / 2, y, a, c, 3, true);
        u.text_c(fw / 2, y + 20, b, [(200.0 * al) as u8, (190.0 * al) as u8, (180.0 * al) as u8], 1);
    }
    minimap(g, &mut u, fw, cx, cy);
    if g.paused && !g.tree {
        u.rect(0, 0, fw, fh, [6, 3, 8], 170);
        u.text_c(fw / 2, fh / 3, "PAUSED", [240, 214, 150], 3);
        u.text_c(fw / 2, fh / 3 + 22, "ESC RESUME    K SKILL TREE    CLOSE THE WINDOW TO QUIT", [200, 190, 190], 1);
        u.text_c(fw / 2, fh / 3 + 34, "M MUSIC ON-OFF    N SOUND ON-OFF    - AND + VOLUME", [150, 140, 150], 1);
    }
    if g.tree {
        draw_tree(g, &mut u, fw, fh);
    }
    if g.wp_open {
        draw_waypoints(g, &mut u, fw, fh);
    }
    if g.inv_open {
        draw_inventory(g, &mut u, fw, fh);
    }
    if let Some((msg, t)) = &g.toast {
        let a = ((*t).min(20) as f32 / 20.0 * 255.0) as u8;
        let w = Ui::text_w(msg, 1);
        u.rect(fw / 2 - w / 2 - 5, 74, w + 10, 13, [10, 6, 10], a.min(210));
        u.text(fw / 2 - w / 2, 78, msg, [240, 220, 170], 1, false);
    }
    if g.fade > 0 {
        u.rect(0, 0, fw, fh, [0, 0, 0], (g.fade.min(30) as f32 / 30.0 * 255.0) as u8);
    }
    // cursor
    let (mx, my) = (inp.mouse.0 as i32, inp.mouse.1 as i32);
    for k in 2..5 {
        for &(dx, dy) in &[(k, 0), (-k, 0), (0, k), (0, -k)] {
            u.px(mx + dx + 1, my + dy + 1, [0, 0, 0], 180);
            u.px(mx + dx, my + dy, [255, 240, 210], 255);
        }
    }
    u.px(mx, my, [255, 80, 60], 255);
}
