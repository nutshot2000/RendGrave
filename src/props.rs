// Chests and shrines: things in the world you walk up to and press E on.
// Chests spill gold, potions and gear; shrines give a one-minute blessing.
use crate::audio::Sfx;
use crate::game::*;
use crate::gear::GroundItem;
use crate::items::roll;
use crate::rng::*;

#[derive(Clone)]
pub struct Chest {
    pub x: f32,
    pub y: f32,
    /// 1 = iron-bound: more gold, better gear
    pub tier: u8,
    pub open: bool,
    /// frames since it was opened (the lid swings)
    pub t: i32,
}

#[derive(Clone)]
pub struct Shrine {
    pub x: f32,
    pub y: f32,
    pub k: u8,
    pub used: bool,
}

/// name, effect, colour
pub const SHRINES: [(&str, &str, [u8; 3]); 4] = [
    ("SHRINE OF POWER", "+50% DAMAGE FOR A MINUTE", [255, 120, 50]),
    ("SHRINE OF HASTE", "+35% SPEED FOR A MINUTE", [110, 255, 150]),
    ("SHRINE OF WARDING", "-40% DAMAGE TAKEN FOR A MINUTE", [110, 160, 255]),
    ("SHRINE OF MENDING", "HEALED, AND WOUNDS CLOSE FOR A MINUTE", [255, 80, 120]),
];
pub const BUFF_TIME: i32 = 60 * 60;

impl Game {
    fn buffed(&self, k: u8) -> bool {
        matches!(self.buff, Some((b, t)) if b == k && t > 0)
    }
    pub fn power_mult(&self) -> f32 {
        if self.buffed(0) { 1.5 } else { 1.0 }
    }
    pub fn haste_mult(&self) -> f32 {
        if self.buffed(1) { 1.35 } else { 1.0 }
    }
    pub fn ward_mult(&self) -> f32 {
        if self.buffed(2) { 0.6 } else { 1.0 }
    }

    /// buff timers every frame
    /// another kill: keep the streak going, and call out the big ones
    pub fn streak_kill(&mut self, xp: i32) {
        self.streak = if self.streak_t > 0 { self.streak + 1 } else { 1 };
        self.streak_t = 150;
        let name = match self.streak {
            3 => "TRIPLE KILL",
            5 => "RAMPAGE",
            8 => "MASSACRE",
            12 => "SLAUGHTER",
            20 => "BUTCHERY",
            30 => "GODLIKE",
            _ => return,
        };
        let bonus = (xp.max(5) as f32 * self.streak as f32 * 0.35) as i32;
        self.gain_xp(bonus);
        let (x, y) = (self.player.x, self.player.y);
        self.float(x, y - 84.0, format!("{}  +{} XP", name, bonus), [255, 200, 70], true);
        self.snd(Sfx::Chime, x, y, 0.6);
    }

    pub fn tick_buffs(&mut self) {
        if self.streak_t > 0 {
            self.streak_t -= 1;
            if self.streak_t == 0 {
                self.streak = 0;
            }
        }
        if let Some((k, t)) = self.buff {
            if t <= 1 {
                self.buff = None;
            } else {
                self.buff = Some((k, t - 1));
                if k == 3 && !self.player.dead {
                    let p = &mut self.player;
                    p.hp = (p.hp + 3.0 / 60.0 + p.maxhp * 0.0002).min(p.maxhp);
                }
            }
        }
        for c in self.chests.iter_mut() {
            if c.open {
                c.t += 1;
            }
        }
    }

    /// chest and shrine prompts; true when one is in reach (so exits don't also prompt)
    pub fn update_props(&mut self, press: bool) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        let near = |x: f32, y: f32, r: f32| ((x - px).powi(2) + ((y - py) * 1.4).powi(2)).sqrt() < r;
        if let Some(i) = self.chests.iter().position(|c| !c.open && near(c.x, c.y, 22.0)) {
            self.prompt = Some(if self.chests[i].tier > 0 { "E  OPEN IRON CHEST".into() } else { "E  OPEN CHEST".into() });
            if press {
                self.open_chest(i);
            }
            return true;
        }
        if let Some(i) = self.shrines.iter().position(|s| !s.used && near(s.x, s.y, 24.0)) {
            let k = self.shrines[i].k as usize;
            self.prompt = Some(format!("E  {}", SHRINES[k].0));
            if press {
                self.shrines[i].used = true;
                self.buff = Some((k as u8, BUFF_TIME));
                let (sx, sy) = (self.shrines[i].x, self.shrines[i].y);
                let c = SHRINES[k].2;
                let cf = [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0];
                self.light(sx, sy - 20.0, 180.0, 2.5, cf, 50);
                self.snd(Sfx::Shrine, sx, sy, 1.0);
                self.snd(Sfx::Chime, sx, sy, 0.4);
                self.banner = Some((SHRINES[k].0.into(), SHRINES[k].1.into(), 170));
                for _ in 0..60 {
                    self.ptc(px + rr(-10.0, 10.0), py, rr(0.0, 50.0), rr(-0.6, 0.6), rr(-0.4, 0.4), rr(0.5, 2.5), 60, PtK::Spark, c);
                }
                if k == 3 {
                    self.player.hp = self.player.maxhp;
                    self.player.mana = self.player.maxmana;
                }
                // the shrine goes dark
                self.lamps.retain(|l| (l.0 - sx).abs() > 1.0 || (l.1 - (sy - 16.0)).abs() > 1.0);
            }
            return true;
        }
        false
    }

    fn open_chest(&mut self, i: usize) {
        let (x, y, tier) = (self.chests[i].x, self.chests[i].y, self.chests[i].tier);
        self.chests[i].open = true;
        self.snd(Sfx::ChestOpen, x, y, 1.0);
        self.snd(Sfx::Coin, x, y, 0.8);
        let fortune = self.rank(crate::skills::Sk::Fortune) as f32;
        let lv = self.alvl;
        let gold = ((12 + ri(20) as i32) * (1 + lv / 2)) as f32 * if tier > 0 { 2.5 } else { 1.0 } * (1.0 + 0.25 * fortune);
        let piles = 3 + tier as i32 * 2;
        for k in 0..piles {
            self.pickups.push(Pickup { k: 0, x: x + rr(-12.0, 12.0), y: y + rr(4.0, 12.0), z: 8.0, vz: rr(1.8, 3.2), amt: (gold / piles as f32) as i32 + 1, t: k * 4 });
        }
        if rnd() < 0.5 + 0.3 * tier as f32 {
            self.pickups.push(Pickup { k: 1, x: x + rr(-8.0, 8.0), y: y + 8.0, z: 8.0, vz: 2.4, amt: 1, t: 6 });
        }
        // plain chests are mostly gold; iron ones always hold something good
        let items = if tier > 0 { 1 + (rnd() < 0.35) as i32 } else { (rnd() < 0.4) as i32 };
        let luck = if tier > 0 { 0.3 } else { 0.1 };
        for r in 0..items {
            let item = roll(lv, luck);
            self.ground.push(GroundItem { item, x: x + rr(-10.0, 10.0), y: y + rr(6.0, 12.0), z: 10.0, vz: rr(2.0, 3.4), t: -(r * 8) });
        }
        for _ in 0..24 {
            self.ptc(x + rr(-6.0, 6.0), y, 6.0, rr(-0.8, 0.8), rr(-0.4, 0.4), rr(0.8, 2.4), 60, PtK::Spark, [255, 210, 110]);
        }
        self.light(x, y - 6.0, 70.0, 1.6, [1.0, 0.8, 0.4], 30);
    }
}

impl Game {
    /// a swing that meets a cracked wall knocks a hole in it
    pub fn hit_cracks(&mut self, px: f32, py: f32, aim: f32, reach: f32) {
        use crate::world::*;
        let (ux, uy) = (aim.cos(), aim.sin());
        let mut hit = None;
        let mut d = 4.0;
        while d < reach + 6.0 {
            let (x, y) = ((px + ux * d) as i32, (py + uy * d * 0.8) as i32);
            if World::inb(x, y) && self.world.aux[y as usize * W + x as usize] & AUX_CRACK != 0 && is_wall(self.world.at(x, y)) {
                hit = Some((x, y));
                break;
            }
            d += 2.0;
        }
        let Some((hx, hy)) = hit else { return };
        for dy in -5i32..=5 {
            for dx in -7i32..=7 {
                let (x, y) = (hx + dx, hy + dy);
                if dx * dx + dy * dy * 2 > 50 || !World::inb(x, y) {
                    continue;
                }
                let i = y as usize * W + x as usize;
                if self.world.aux[i] & AUX_CRACK != 0 && is_wall(self.world.m[i]) {
                    self.world.set(x, y, if rnd() < 0.15 { RUBBLE } else { E });
                    self.world.aux[i] &= !AUX_CRACK;
                }
            }
        }
        for _ in 0..16 {
            self.ptc(hx as f32, hy as f32, rr(4.0, 30.0), rr(-1.5, 1.5), rr(-0.5, 1.5), rr(0.5, 2.0), 200, PtK::Debris, [92, 82, 96]);
        }
        self.snd(Sfx::Crumble, hx as f32, hy as f32, 0.9);
        self.shake = self.shake.max(2.5);
        self.secret_found(hx as f32, hy as f32);
    }

    /// the first breach in a cracked wall
    pub fn secret_found(&mut self, x: f32, y: f32) {
        if let Some(k) = self.cracks.iter().position(|c| (c.0 - x).abs() < 30.0 && (c.1 - y).abs() < 20.0) {
            self.cracks.swap_remove(k);
            self.float(x, y - 30.0, "A HIDDEN ROOM!".into(), [255, 214, 90], true);
            self.snd(Sfx::Chime, x, y, 0.8);
        }
    }
}
