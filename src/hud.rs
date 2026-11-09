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
    ('\'', "010010000000000"), (',', "000000000010100"), ('(', "010100100100010"), (')', "010001001001010"),
    ('<', "001010100010001"), ('>', "100010001010100"), ('?', "110001010000010"), ('=', "000111000111000"),
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
        Sk::Conductor => {
            drop(u, x + 5, y + 4, 5, [80, 140, 230]);
            bolt(u, x + 9, y + 2, 2, [255, 245, 170]);
        }
        Sk::Overload => {
            disc(u, x + 8, y + 8, 6.0, [60, 70, 140]);
            bolt(u, x + 5, y + 1, 2, [255, 255, 255]);
        }
        Sk::StaticField => {
            for k in 0..5 {
                let a = k as f32 * 1.26 + t as f32 * 0.05;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 6.0) as i32, [200, 220, 255], 255);
            }
            bolt(u, x + 5, y + 3, 2, [180, 200, 255]);
        }
        Sk::StormShield => {
            disc(u, x + 8, y + 8, 6.5, [90, 120, 220]);
            disc(u, x + 8, y + 8, 4.5, [24, 20, 40]);
            bolt(u, x + 5, y + 3, 2, [220, 235, 255]);
        }
        Sk::EyeOfStorm => {
            disc(u, x + 8, y + 8, 6.5, [70, 80, 120]);
            disc(u, x + 8, y + 8, 3.0, [255, 245, 170]);
            disc(u, x + 8, y + 8, 1.2, [20, 20, 30]);
        }
        Sk::Searing => {
            for k in 0..5 {
                u.rect(x + 3 + k * 2, y + 12 - (k % 3) * 2, 2, 3 + (k % 3) * 2, fire(k), 255);
            }
            u.rect(x + 4, y + 2, 8, 2, [255, 240, 200], 255);
        }
        Sk::Splinter => {
            disc(u, x + 8, y + 8, 3.0, [255, 200, 90]);
            for &(dx, dy) in &[(-5, -4), (5, -3), (0, 6)] {
                disc(u, x + 8 + dx, y + 8 + dy, 2.0, fire(dx + dy));
            }
        }
        Sk::Fireproof => {
            disc(u, x + 8, y + 8, 6.0, [120, 110, 120]);
            u.rect(x + 5, y + 6, 6, 5, fire(1), 255);
        }
        Sk::Combustion => {
            for k in 0..8 {
                let a = k as f32 * 0.785;
                u.rect(x + 8 + (a.cos() * 5.0) as i32, y + 8 + (a.sin() * 5.0) as i32, 2, 2, fire(k), 255);
            }
            disc(u, x + 8, y + 8, 2.5, [255, 240, 200]);
        }
        Sk::MeteorShower => {
            for &(dx, dy) in &[(3, 4), (9, 2), (6, 10)] {
                disc(u, x + dx + 2, y + dy + 2, 2.0, fire(dx));
                u.px(x + dx, y + dy, [255, 200, 120], 200);
            }
        }
        Sk::Caustic => {
            u.rect(x + 2, y + 10, 12, 4, [110, 210, 50], 255);
            for k in 0..3 {
                u.px(x + 4 + k * 4, y + 7 - (k % 2), [200, 255, 140], 255);
            }
        }
        Sk::PoisonCloud => {
            disc(u, x + 6, y + 9, 4.0, [70, 120, 40]);
            disc(u, x + 10, y + 7, 4.5, [90, 150, 50]);
            disc(u, x + 9, y + 11, 3.0, [110, 180, 60]);
        }
        Sk::AcidPools => {
            disc(u, x + 8, y + 10, 5.5, [70, 150, 40]);
            disc(u, x + 8, y + 10, 3.0, [140, 230, 70]);
            drop(u, x + 8, y + 1, 4, [140, 230, 70]);
        }
        Sk::RotBurst => {
            u.rect(x + 4, y + 9, 8, 3, [190, 180, 160], 255);
            for k in 0..6 {
                let a = k as f32 * 1.05;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 5.0) as i32, [140, 230, 60], 255);
            }
        }
        Sk::Plague => {
            for k in 0..6 {
                drop(u, x + 2 + k * 2, y + 2 + (k % 3) * 4, 3, [120, 220, 60]);
            }
        }
        Sk::Chill => {
            u.rect(x + 7, y + 2, 2, 12, [190, 230, 255], 255);
            u.rect(x + 2, y + 7, 12, 2, [190, 230, 255], 255);
        }
        Sk::IceArmor => {
            u.rect(x + 3, y + 3, 10, 10, [140, 200, 240], 255);
            u.rect(x + 5, y + 5, 6, 6, [210, 240, 255], 255);
        }
        Sk::Shatterstorm => {
            for k in 0..6 {
                let a = k as f32 * 1.05 + 0.3;
                u.rect(x + 8 + (a.cos() * 5.0) as i32, y + 8 + (a.sin() * 5.0) as i32, 2, 1, [210, 240, 255], 255);
            }
            disc(u, x + 8, y + 8, 1.5, [255, 255, 255]);
        }
        Sk::DeepFreeze => {
            disc(u, x + 8, y + 8, 6.0, [80, 140, 200]);
            disc(u, x + 8, y + 8, 3.5, [200, 235, 255]);
        }
        Sk::AbsoluteZero => {
            for k in 0..4 {
                let a = k as f32 * 0.785;
                let (cx, cy) = (a.cos() * 6.0, a.sin() * 6.0);
                for s in -6..=6 {
                    let f = s as f32 / 6.0;
                    u.px(x + 8 + (cx * f) as i32, y + 8 + (cy * f) as i32, [230, 245, 255], 255);
                }
            }
        }
        Sk::Cleave => {
            for k in 0..9 {
                let a = 3.4 + k as f32 * 0.32;
                u.px(x + 8 + (a.cos() * 7.0) as i32, y + 10 + (a.sin() * 7.0) as i32, [240, 240, 250], 255);
            }
        }
        Sk::Dismember => {
            u.rect(x + 3, y + 4, 4, 9, [222, 184, 152], 255);
            u.rect(x + 9, y + 4, 4, 9, [222, 184, 152], 255);
            u.rect(x + 7, y + 2, 2, 12, [200, 30, 40], 255);
        }
        Sk::Momentum => {
            for k in 0..3 {
                u.rect(x + 2 + k * 4, y + 4 + k, 3, 8 - k * 2, [230, 225, 215], 255);
            }
        }
        Sk::BladeFury => {
            for k in 0..12 {
                let a = k as f32 * 0.52 + t as f32 * 0.2;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 6.0) as i32, if k % 3 == 0 { [255, 255, 255] } else { [180, 180, 200] }, 255);
            }
            disc(u, x + 8, y + 8, 1.5, [200, 40, 40]);
        }
        Sk::Warlord => {
            u.rect(x + 3, y + 9, 10, 4, [220, 180, 60], 255);
            for k in 0..3 {
                u.rect(x + 3 + k * 4, y + 5 + (k % 2), 2, 4, [220, 180, 60], 255);
            }
        }
        Sk::Toughness => {
            u.rect(x + 3, y + 3, 10, 9, [150, 150, 165], 255);
            u.rect(x + 5, y + 12, 6, 2, [150, 150, 165], 255);
        }
        Sk::SecondWind => {
            u.rect(x + 6, y + 2, 4, 3, [180, 170, 160], 255);
            disc(u, x + 8, y + 10, 4.5, [220, 40, 60]);
        }
        Sk::Regeneration => {
            u.rect(x + 7, y + 3, 2, 10, [255, 90, 110], 255);
            u.rect(x + 3, y + 7, 10, 2, [255, 90, 110], 255);
        }
        Sk::LastStand => {
            u.rect(x + 7, y + 2, 2, 12, [230, 225, 215], 255);
            disc(u, x + 8, y + 10, 3.0, [200, 30, 40]);
        }
        Sk::Titan => {
            disc(u, x + 8, y + 5, 3.0, [240, 110, 120]);
            u.rect(x + 3, y + 8, 10, 6, [240, 110, 120], 255);
        }
    }
}

/// layout of the skill tree screen: one school per tab, three paths side by side.
/// Returns the (x, y) of the node's top-left corner and the box size.
pub fn node_pos(i: usize, fw: i32, _fh: i32) -> (i32, i32, i32) {
    let n = &NODES[i];
    let bs = 20;
    let x = fw / 2 + (n.lane as i32 - 1) * 150 - 60;
    let y = 50 + n.row as i32 * 31;
    (x, y, bs)
}
/// the tabs along the top
fn tab_rect(k: usize, fw: i32) -> (i32, i32, i32, i32) {
    let tw = (fw - 20) / 6;
    (10 + k as i32 * tw, 28, tw - 2, 14)
}
pub fn tab_at(m: (f32, f32), view: (f32, f32)) -> Option<usize> {
    let fw = view.0 as i32;
    (0..6).find(|&k| {
        let (x, y, w, h) = tab_rect(k, fw);
        let (mx, my) = (m.0 as i32, m.1 as i32);
        mx >= x && mx < x + w && my >= y && my < y + h
    })
}
pub fn node_at(m: (f32, f32), view: (f32, f32), tab: usize) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    for i in 0..N {
        if NODES[i].col != tab {
            continue;
        }
        let (x, y, bs) = node_pos(i, fw, fh);
        // the name to the right of the box counts too
        if m.0 as i32 >= x - 2 && m.0 as i32 <= x + bs + 70 && m.1 as i32 >= y - 2 && m.1 as i32 <= y + bs + 2 {
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
            if !g.seen_at(x as usize, y as usize) {
                // not explored yet
                u.px(ox + x, oy + y, [12, 10, 16], 220);
                continue;
            }
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
                    ROAD => [150, 124, 84],
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
    let seen_w = |x: f32, y: f32| g.seen_at(((x / sx) as usize).min(83), ((y / sy) as usize).min(62));
    for ch in g.chests.iter().filter(|c| !c.open && seen_w(c.x, c.y)) {
        u.px(ox + (ch.x / sx) as i32, oy + (ch.y / sy) as i32, [255, 210, 60], 255);
    }
    for s in g.shrines.iter().filter(|s| !s.used && seen_w(s.x, s.y)) {
        let c = crate::props::SHRINES[s.k as usize].2;
        let (x, y) = (ox + (s.x / sx) as i32, oy + (s.y / sy) as i32);
        for &(dx, dy) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            u.px(x + dx, y + dy, c, 255);
        }
    }
    for e in g.enemies.iter().filter(|e| g.seen_at(((e.x / sx) as usize).min(83), ((e.y / sy) as usize).min(62))) {
        if e.alive {
            let (x, y) = (ox + (e.x / sx) as i32, oy + (e.y / sy) as i32);
            if x >= ox && y >= oy && x < ox + mw && y < oy + mh {
                u.px(x, y, if e.k.big() { [255, 150, 40] } else { [230, 50, 50] }, 255);
            }
        }
    }
    // exits: a bright notch where each one leaves the map
    for e in &g.exits {
        let (x0, y0) = (ox + (e.x0 as f32 / sx) as i32, oy + (e.y0 as f32 / sy) as i32);
        let (x1, y1) = (ox + (e.x1 as f32 / sx) as i32, oy + (e.y1 as f32 / sy) as i32);
        for y in y0..=y1.max(y0 + 1) {
            for x in x0..=x1.max(x0 + 1) {
                u.px(x, y, [235, 235, 255], 255);
            }
        }
    }
    if let Some((gx, gy)) = g.guide() {
        if (g.frame / 15) % 2 == 0 {
            let (x, y) = (ox + (gx / sx) as i32, oy + (gy / sy) as i32);
            for &(dx, dy) in &[(0, -2), (-1, -1), (0, -1), (1, -1), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)] {
                u.px(x + dx, y + dy, [255, 214, 90], 255);
            }
        }
    }
    if let Some((qx, qy)) = g.portal {
        let (x, y) = (ox + (qx / sx) as i32, oy + (qy / sy) as i32);
        for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
            u.px(x + dx, y + dy, [190, 130, 255], 255);
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

pub fn draw(g: &Game, f: &mut Frame, inp: &Input, cx: i32, cy: i32) {
    for v in f.ui.iter_mut() {
        *v = 0;
    }
    let (fw, fh) = (f.w as i32, f.h as i32);
    let t = g.frame;
    let mut u = Ui { f };
    if g.title {
        draw_title(g, &mut u, fw, fh);
        let (mx, my) = (inp.mouse.0 as i32, inp.mouse.1 as i32);
        for k in 2..5 {
            for &(dx, dy) in &[(k, 0), (-k, 0), (0, k), (0, -k)] {
                u.px(mx + dx, my + dy, [255, 240, 210], 255);
            }
        }
        if g.fade > 0 {
            u.rect(0, 0, fw, fh, [0, 0, 0], (g.fade.min(60) as f32 / 60.0 * 255.0) as u8);
        }
        return;
    }
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
            u.text_c(sx, sy - 8, &crate::champions::champ_name(e.aff), crate::champions::champ_col(e.aff), 1);
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
    if g.pick == 2 {
        // oil flasks
        let fx = qx + 26;
        for dy in 0..6 {
            for dx in -1i32..=1 {
                u.px(fx + dx, qy + 2 + dy, if dy == 0 { [150, 110, 70] } else if dx == 1 { [100, 120, 130] } else { [50, 56, 64] }, 255);
            }
        }
        u.text(fx + 4, qy + 2, &format!("{} Q", g.flasks), [200, 210, 220], 1, true);
    }
    // spell bar: every learned spell
    let full = p.sk.bar();
    // at most 12 slots on screen; the window follows the selected spell
    let vis = 12usize;
    let start = if full.len() > vis { p.sel.saturating_sub(vis / 2).min(full.len() - vis) } else { 0 };
    let bar: Vec<_> = full.iter().copied().skip(start).take(vis).collect();
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
    if start > 0 {
        u.text(bx - 12, by + 6, "<", [240, 214, 150], 1, true);
    }
    if start + bar.len() < full.len() {
        u.text(bx + total + 7, by + 6, ">", [240, 214, 150], 1, true);
    }
    for (k0, &s) in bar.iter().enumerate() {
        let k = k0 + start;
        let x = bx + k0 as i32 * (sw + 3);
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
    if let Some(&s) = full.get(p.sel) {
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
    if let Some((obj, _)) = g.objective() {
        let pulse = 0.8 + (t as f32 * 0.08).sin() * 0.2;
        u.text(8, 27, &obj, [(255.0 * pulse) as u8, (214.0 * pulse) as u8, 90], 1, true);
    }
    if g.streak >= 3 && g.streak_t > 0 {
        // the streak counter, draining as the window closes
        let w = 40;
        let x = fw / 2 - w / 2;
        u.text_c(fw / 2, 64, &format!("{} KILLS", g.streak), [255, 200, 70], 2);
        u.rect(x, 78, w, 2, [40, 30, 20], 200);
        u.rect(x, 78, w * g.streak_t / 150, 2, [255, 200, 70], 255);
    }
    if let Some((k, left)) = g.buff {
        let (name, _, c) = crate::props::SHRINES[k as usize];
        let s = left / 60;
        let txt = format!("{}  {}:{:02}", name.trim_start_matches("SHRINE OF "), s / 60, s % 60);
        let blink = left > 600 || (left / 15) % 2 == 0;
        if blink {
            u.text(8, 37, &txt, c, 1, true);
        }
    }
    // boss health
    if let Some(b) = g.enemies.iter().find(|e| e.alive && e.k.boss() && e.aggro) {
        let bw = (fw / 2).min(260);
        let bx = fw / 2 - bw / 2;
        u.rect(bx - 2, 62, bw + 4, 8, [10, 6, 10], 230);
        u.rect(bx, 64, (bw as f32 * (b.hp / b.maxhp).clamp(0.0, 1.0)) as i32, 4, [190, 30, 30], 255);
        u.text_c(fw / 2, 53, if b.k == Kind::King { "THE HOLLOW KING" } else { "THE BANDIT CHIEF" }, [240, 200, 120], 1);
        if b.k == Kind::King {
            // his weak points: knees, sword arm, chest
            let cw = bw / 3 - 4;
            for (k, name) in crate::kingparts::WEAK_NAMES.iter().enumerate() {
                let x0 = bx + k as i32 * (cw + 6);
                let left = b.wk[k].clamp(0.0, 1.0);
                u.rect(x0 - 1, 71, cw + 2, 4, [10, 6, 10], 230);
                if left > 0.0 {
                    u.rect(x0, 72, (cw as f32 * left) as i32, 2, [230, 170, 70], 255);
                }
                let (txt, col) = if left > 0.0 { (name.to_string(), [200, 180, 150]) } else { (format!("{} - BROKEN", name), [255, 120, 90]) };
                u.text_c(x0 + cw / 2, 77, &txt, col, 1);
            }
        }
    }
    // interaction prompt
    if let Some(pr) = &g.prompt {
        let w = Ui::text_w(pr, 1);
        u.rect(fw / 2 - w / 2 - 4, fh - 62, w + 8, 11, [10, 6, 10], 200);
        u.text(fw / 2 - w / 2, fh - 59, pr, [255, 230, 150], 1, false);
    }
    // controls hint for the first seconds
    if t < 900 && !g.press {
        let a = if t > 780 { ((900 - t) as f32 / 120.0 * 255.0) as u8 } else { 255 };
        let lines = ["WASD MOVE   LMB ATTACK   RMB CAST", "1-9 SPELLS   SPACE DODGE   F POTION   K SKILLS", "AIM AT LEGS TO TAKE THEM OFF   E INTERACT   I GEAR"];
        for (k, l) in lines.iter().enumerate() {
            let w = Ui::text_w(l, 1);
            let y = 26 + k as i32 * 8;
            u.rect(fw / 2 - w / 2 - 3, y - 2, w + 6, 9, [10, 6, 10], (a as f32 * 0.6) as u8);
            u.text(fw / 2 - w / 2, y, l, [230, 220, 200], 1, false);
        }
    }
    // banner
    if let Some((a, b, tt)) = g.banner.as_ref().filter(|_| !g.press) {
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
        if !g.controls {
            u.text_c(fw / 2, fh / 3 - 10, "PAUSED", [240, 214, 150], 3);
        }
        if g.controls {
            draw_controls(&mut u, fw, fh);
        } else {
            draw_menu(g, &mut u, fw, fh);
        }
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
    if g.shop_open {
        draw_shop(g, &mut u, fw, fh);
    }
    draw_guide(g, &mut u, fw, fh, cx, cy);
    draw_exit_labels(g, &mut u, fw, fh, cx, cy);
    draw_talk(g, &mut u, fw, fh);
    draw_choice(g, &mut u, fw, fh);
    if g.ending == -1 {
        draw_ending(g, &mut u, fw, fh);
    }
    if let Some((msg, t)) = g.toast.as_ref().filter(|_| !g.press) {
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

// ------------------------------------------------------------------ menus
pub fn menu_row(k: usize, fw: i32, fh: i32, title: bool) -> (i32, i32, i32, i32) {
    let (w, h) = (180, 13);
    let y0 = if title { fh / 2 - 24 } else { fh / 3 + 14 };
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
    if g.controls {
        draw_controls(u, fw, fh);
        return;
    }
    let pulse = 0.85 + (t as f32 * 0.03).sin() * 0.15;
    u.text_c(fw / 2 + 2, fh / 5 + 2, "RENDGRAVE", [40, 8, 6], 4);
    u.text_c(fw / 2, fh / 5, "RENDGRAVE", [(240.0 * pulse) as u8, (200.0 * pulse) as u8, (140.0 * pulse) as u8], 4);
    u.text_c(fw / 2, fh / 5 + 30, "A PIXEL-SIMULATED ACTION RPG", [190, 170, 160], 1);
    u.text_c(fw / 2, fh / 5 + 42, "EARLY ACCESS  -  ACT I: THE HOLLOW KING", [150, 130, 130], 1);
    draw_menu(g, u, fw, fh);
    u.text_c(fw / 2, fh - 14, "WASD MOVE   LMB ATTACK   RMB CAST   SPACE DODGE   E INTERACT", [120, 110, 120], 1);
}
/// the full list of keys, from the title or the pause menu
fn draw_controls(u: &mut Ui, fw: i32, fh: i32) {
    let rows: [(&str, &str); 17] = [
        ("WASD / ARROWS", "MOVE"),
        ("LEFT MOUSE", "ATTACK WITH YOUR WEAPON"),
        ("RIGHT MOUSE", "CAST THE SELECTED SPELL"),
        ("1 - 9 / WHEEL", "CHOOSE A SPELL"),
        ("SPACE", "DODGE ROLL"),
        ("F", "DRINK A POTION"),
        ("Q", "THROW AN OIL FLASK (IF YOU HAVE THEM)"),
        ("E", "INTERACT, TALK, OPEN, PICK UP"),
        ("I", "GEAR AND INVENTORY"),
        ("K / TAB", "SKILL TREE"),
        ("ESC", "PAUSE, CLOSE A WINDOW"),
        ("R", "WAKE IN ASHFORD AFTER DEATH"),
        ("M / N", "MUSIC / SOUND ON OR OFF"),
        ("- / +", "VOLUME"),
        ("F11", "FULLSCREEN"),
        ("AIM LOW", "TAKE OFF LEGS, HIT CRAWLERS AND BARRELS"),
        ("CRACKED WALLS", "BREAK THEM - SOMETHING IS BEHIND"),
    ];
    let (w, h) = (300, rows.len() as i32 * 10 + 40);
    let (x, y) = (fw / 2 - w / 2, (fh - h) / 2);
    u.rect(x - 2, y - 2, w + 4, h + 4, [70, 50, 60], 255);
    u.rect(x, y, w, h, [16, 10, 14], 245);
    u.text_c(fw / 2, y + 6, "CONTROLS", [240, 214, 150], 1);
    for (k, (a, b)) in rows.iter().enumerate() {
        let ry = y + 20 + k as i32 * 10;
        u.text(x + 12, ry, a, [255, 214, 140], 1, false);
        u.text(x + 118, ry, b, [210, 200, 200], 1, false);
    }
    u.text_c(fw / 2, y + h - 11, "CLICK OR ESC TO GO BACK", [150, 140, 150], 1);
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
    let (ax, ay) = (pcx as f32 + ux * tt, (pcy as f32 + uy * tt).clamp(50.0, fh as f32 - 100.0));
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
/// the boxes of a choice: the starting gift, the respec
pub fn choice_box(k: usize, n: usize, fw: i32, fh: i32) -> (i32, i32, i32, i32) {
    let w = ((fw - 40) / n as i32 - 8).min(170);
    let total = n as i32 * (w + 8) - 8;
    let x0 = fw / 2 - total / 2;
    (x0 + k as i32 * (w + 8), fh / 2 - 30, w, 62)
}
pub fn choice_at(m: (f32, f32), view: (f32, f32), n: usize) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    (0..n).find(|&k| {
        let (x, y, w, h) = choice_box(k, n, fw, fh);
        let (mx, my) = (m.0 as i32, m.1 as i32);
        mx >= x && mx < x + w && my >= y && my < y + h
    })
}
fn draw_choice(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let Some(c) = &g.choice else { return };
    u.rect(0, 0, fw, fh, [6, 3, 8], 150);
    u.text_c(fw / 2, fh / 2 - 50, &c.title, [240, 214, 150], 1);
    let n = c.opts.len();
    for (k, (name, desc)) in c.opts.iter().enumerate() {
        let (x, y, w, h) = choice_box(k, n, fw, fh);
        let hov = c.hover == Some(k);
        u.rect(x, y, w, h, if hov { [60, 36, 28] } else { [20, 13, 18] }, 240);
        u.rect(x, y, w, 1, if hov { [255, 190, 110] } else { [90, 70, 60] }, 255);
        u.text_c(x + w / 2, y + 6, &format!("{}  {}", k + 1, name), if hov { [255, 236, 200] } else { [240, 214, 150] }, 1);
        for (j, l) in wrap(desc, ((w - 10) / 4) as usize).iter().enumerate() {
            u.text(x + 6, y + 20 + j as i32 * 9, l, [200, 190, 190], 1, false);
        }
    }
    let foot = if c.kind == crate::agency::ChoiceKind::Start { "CLICK ONE, OR PRESS 1, 2 OR 3" } else { "CLICK ONE, OR ESC" };
    u.text_c(fw / 2, fh / 2 + 52, foot, [150, 140, 150], 1);
}

fn draw_talk(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let Some((pages, page)) = &g.talk else { return };
    let (x, y, w, h) = (30, fh - 92, fw - 60, 62);
    u.rect(x, y, w, h, [16, 11, 14], 238);
    u.rect(x, y, w, 1, [200, 170, 110], 255);
    u.rect(x, y + h - 1, w, 1, [90, 70, 50], 255);
    if let Some(name) = &g.talk_name {
        // a note, a place, a stranger: just the words
        u.text(x + 12, y + 7, name, [240, 214, 150], 1, true);
        let maxc = ((w - 24) / 4) as usize;
        if let Some(txt) = pages.get(*page) {
            for (k, l) in wrap(txt, maxc).iter().enumerate() {
                u.text(x + 12, y + 20 + k as i32 * 9, l, [230, 222, 210], 1, false);
            }
        }
        let more = if *page + 1 < pages.len() { format!("E  NEXT  ({}/{})", page + 1, pages.len()) } else { "E  CLOSE".into() };
        u.text(x + w - Ui::text_w(&more, 1) - 8, y + h - 10, &more, [255, 214, 90], 1, false);
        return;
    }
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

/// names over nearby exits, so you know where a road or a stair leads
fn draw_exit_labels(g: &Game, u: &mut Ui, fw: i32, fh: i32, cx: i32, cy: i32) {
    if g.player.dead || g.tree || g.inv_open || g.shop_open || g.wp_open || g.paused || g.talk.is_some() {
        return;
    }
    for e in &g.exits {
        let (ex, ey) = ((e.x0 + e.x1) as f32 / 2.0, (e.y0 + e.y1) as f32 / 2.0);
        let d = ((ex - g.player.x).powi(2) + (ey - g.player.y).powi(2)).sqrt();
        if d > 260.0 {
            continue;
        }
        let a = (((260.0 - d) / 80.0).min(1.0) * 230.0) as u8;
        let txt = format!("TO {}", e.label);
        let w = Ui::text_w(&txt, 1);
        let (sx, sy) = (ex as i32 - cx, e.y0 - cy - 14);
        let x = (sx - w / 2).clamp(6, fw - w - 6);
        let y = sy.clamp(44, fh - 80);
        u.rect(x - 3, y - 2, w + 6, 10, [10, 8, 14], a.min(190));
        u.text(x, y, &txt, [210, 220, 255], 1, false);
    }
}
