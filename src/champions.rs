// Champion types: every glowing champion has a power of its own, shown by its outline colour
// and its name tag. Burning ones leave a trail of fire and set you alight, frozen ones chill
// you, vampiric ones heal on every hit, swift ones run you down, and volatile ones drop a lit
// powder keg when they die.
use crate::audio::Sfx;
use crate::game::*;
use crate::rng::*;

/// (name, outline colour), indexed by Enemy::aff - 1
pub const CHAMPS: [(&str, [u8; 3]); 5] = [
    ("BURNING", [255, 110, 40]),
    ("FROZEN", [120, 190, 255]),
    ("VAMPIRIC", [230, 40, 60]),
    ("SWIFT", [235, 235, 245]),
    ("VOLATILE", [255, 225, 60]),
];

pub fn champ_col(aff: u8) -> [u8; 3] {
    if aff == 0 { [255, 190, 60] } else { CHAMPS[(aff - 1) as usize].1 }
}
pub fn champ_name(aff: u8) -> String {
    if aff == 0 { "CHAMPION".into() } else { CHAMPS[(aff - 1) as usize].0.to_string() }
}

impl Game {
    /// every frame, for champions
    pub fn champion_tick(&mut self, i: usize) {
        let e = &mut self.enemies[i];
        if !e.elite || e.aff == 0 || e.frozen > 0 {
            return;
        }
        let (x, y, f) = (e.x, e.y, self.frame);
        match e.aff {
            1 => {
                // never burns itself; leaves fire where it walks
                if f % 10 == 0 {
                    e.body.tint_all(|p| p.burn = 0);
                }
                if e.aggro && f % 9 == 0 {
                    self.world.ignite(x as i32 + rr(-3.0, 3.0) as i32, y as i32 + rr(-1.0, 1.0) as i32);
                }
                if f % 4 == 0 {
                    self.ptc(x + rr(-6.0, 6.0), y, rr(10.0, 45.0), 0.0, 0.0, 0.6, 20, PtK::Ember, [255, 140, 50]);
                }
            }
            2 => {
                if f % 5 == 0 {
                    self.ptc(x + rr(-7.0, 7.0), y, rr(5.0, 50.0), 0.0, 0.0, 0.2, 40, PtK::Ice, [200, 235, 255]);
                }
            }
            3 => {
                if f % 8 == 0 {
                    self.ptc(x + rr(-5.0, 5.0), y, rr(10.0, 50.0), 0.0, 0.0, 0.5, 30, PtK::Heal, [200, 20, 40]);
                }
            }
            5 => {
                if f % 6 == 0 {
                    self.ptc(x + rr(-4.0, 4.0), y, rr(20.0, 50.0), rr(-0.4, 0.4), 0.0, 0.8, 16, PtK::Spark, [255, 220, 90]);
                }
            }
            _ => {}
        }
    }

    /// a champion's hit lands on you
    pub fn champion_hit(&mut self, i: usize) {
        let e = &self.enemies[i];
        if !e.elite {
            return;
        }
        let (x, y) = (e.x, e.y);
        match e.aff {
            1 => self.player.burn = self.player.burn.max(120),
            2 => {
                self.player.frost = self.player.frost.max(150);
                self.snd(Sfx::Frost, self.player.x, self.player.y, 0.6);
            }
            3 => {
                let e = &mut self.enemies[i];
                let heal = e.maxhp * 0.08;
                e.hp = (e.hp + heal).min(e.maxhp);
                self.float(x, y - 70.0, format!("+{}", heal as i32), [230, 40, 60], false);
            }
            _ => {}
        }
    }

    /// a champion dies
    pub fn champion_death(&mut self, i: usize) {
        let e = &self.enemies[i];
        if !e.elite {
            return;
        }
        let (x, y) = (e.x, e.y);
        match e.aff {
            5 => {
                // it was carrying powder
                self.drop_keg(x, y + 2.0);
                self.float(x, y - 70.0, "IT'S GOING TO BLOW".into(), [255, 225, 60], true);
            }
            2 => {
                // bursts into freezing shards
                for _ in 0..40 {
                    let a = rnd() * std::f32::consts::TAU;
                    self.ptc(x, y, rr(5.0, 40.0), a.cos() * rr(1.0, 3.0), a.sin() * rr(0.6, 2.0), rr(0.5, 2.0), 60, PtK::Ice, [210, 240, 255]);
                }
                let d = ((self.player.x - x).powi(2) + (self.player.y - y).powi(2)).sqrt();
                if d < 50.0 {
                    self.player.frost = self.player.frost.max(120);
                }
                self.snd(Sfx::Shatter, x, y, 0.9);
            }
            _ => {}
        }
    }
}

impl Game {
    /// the grave robber: once he's seen you he runs, and gets away if you don't catch him
    pub fn robber_tick(&mut self, i: usize) {
        if self.enemies[i].k != crate::body::Kind::Robber || !self.enemies[i].aggro {
            return;
        }
        let e = &mut self.enemies[i];
        let (x, y) = (e.x, e.y);
        if e.skill_t == 25 * 60 {
            self.float(x, y - 70.0, "A GRAVE ROBBER - CATCH HIM!".into(), [255, 214, 90], true);
            self.snd(Sfx::Shout, x, y, 1.0);
        }
        let e = &mut self.enemies[i];
        e.skill_t -= 1;
        if e.skill_t % 3 == 0 {
            // gold trickles from his sack as he runs
            self.ptc(x + rr(-3.0, 3.0), y, rr(20.0, 30.0), rr(-0.5, 0.5), rr(-0.3, 0.3), rr(0.5, 1.5), 40, PtK::Spark, [255, 210, 80]);
        }
        let e = &mut self.enemies[i];
        if e.skill_t <= 0 {
            e.alive = false;
            for _ in 0..30 {
                self.ptc(x, y, rr(0.0, 40.0), rr(-1.0, 1.0), rr(-0.5, 0.5), rr(0.2, 1.0), 80, PtK::Smoke, [80, 76, 70]);
            }
            self.float(x, y - 60.0, "HE GOT AWAY".into(), [200, 180, 160], true);
            self.snd(Sfx::Travel, x, y, 0.6);
        }
    }

    /// caught him: his sack bursts
    pub fn robber_loot(&mut self, x: f32, y: f32) {
        let lv = self.alvl;
        for k in 0..10 {
            let amt = (8 + ri(10) as i32) * (1 + lv / 2);
            self.pickups.push(Pickup { k: 0, x: x + rr(-16.0, 16.0), y: y + rr(-8.0, 8.0), z: 10.0, vz: rr(2.0, 3.5), amt, t: k * 3 });
        }
        let mut item = crate::items::roll(lv, 0.4);
        for _ in 0..20 {
            if item.rarity >= crate::items::Rarity::Rare {
                break;
            }
            item = crate::items::roll(lv, 0.4);
        }
        self.ground.push(crate::gear::GroundItem { item, x, y: y + 6.0, z: 12.0, vz: 3.0, t: 10 });
        self.banner = Some(("CAUGHT THE GRAVE ROBBER".into(), "HIS SACK BURSTS OPEN".into(), 150));
        self.snd(Sfx::Coin, x, y, 1.0);
    }
}
