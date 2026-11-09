// The skills added with the deeper trees: new actives (Storm Shield, Combustion, Poison
// Cloud, Rot Burst, Ice Armour, Blade Fury) and the per-frame upkeep for passives that
// need it (regeneration, Last Stand, shield timers). Passives that only change a number
// live where that number is used (zap_enemy, hurt, armor_mult, weapon_hit, ...).
use crate::audio::Sfx;
use crate::game::*;
use crate::rng::*;
use crate::skills::*;
use crate::spells::{Field, Meteor};
use crate::world::*;

impl Game {
    /// cast one of the new active skills; false if `s` is not one of them
    pub fn cast_talent(&mut self, s: Sk, mx: f32, my: f32) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        match s {
            Sk::StormShield => {
                self.player.shield = 480;
                self.snd(Sfx::Thunder, px, py, 0.6);
                self.light(px, py - 20.0, 120.0, 1.6, [0.6, 0.7, 1.0], 20);
                self.banner_small("STORM SHIELD");
            }
            Sk::Combustion => {
                let mut n = 0;
                let n1 = self.lights.len();
                for i in 0..self.enemies.len() {
                    let e = &self.enemies[i];
                    if !e.alive || e.burning <= 0 || ((e.x - px).powi(2) + (e.y - py).powi(2)).sqrt() > 180.0 || n >= 10 {
                        continue;
                    }
                    let (ex, ey) = (e.x, e.y);
                    self.explode(ex, ey, 20.0, 14.0 + 2.0 * self.rank(Sk::Kindling) as f32);
                    n += 1;
                    // one big light for the lot, or the screen whites out
                    self.lights.truncate(n1 + 1);
                }
                if n == 0 {
                    self.banner_small("NOTHING NEAR YOU IS BURNING");
                    self.player.mana += node(Sk::Combustion).mana;
                    self.player.cd[Sk::Combustion as usize] = 0;
                }
            }
            Sk::PoisonCloud => {
                self.snd(Sfx::Acid, mx, my, 0.8);
                let r = 40.0 * if self.rank(Sk::Plague) > 0 { 1.3 } else { 1.0 };
                self.fields.push(Field { k: 3, x: mx, y: my, t: 0, max: 420, r });
            }
            Sk::RotBurst => {
                // the freshest remains nearest the cursor
                let mut near: Vec<(f32, usize)> = self.gibs.iter().enumerate().map(|(i, g)| (((g.x - mx).powi(2) + (g.y - my).powi(2)).sqrt(), i)).filter(|(d, _)| *d < 140.0).collect();
                near.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                near.truncate(6);
                if near.is_empty() {
                    self.banner_small("NO REMAINS NEAR THE CURSOR");
                    self.player.mana += node(Sk::RotBurst).mana;
                    self.player.cd[Sk::RotBurst as usize] = 0;
                    return true;
                }
                let spots: Vec<(f32, f32)> = near.iter().map(|&(_, i)| (self.gibs[i].x, self.gibs[i].y)).collect();
                let mut idx: Vec<usize> = near.iter().map(|&(_, i)| i).collect();
                idx.sort_unstable_by(|a, b| b.cmp(a));
                for i in idx {
                    self.gibs.swap_remove(i);
                }
                let n1 = self.lights.len();
                for (k, (x, y)) in spots.into_iter().enumerate() {
                    self.world.splash(x, y, ACID, 4.0 + self.rank(Sk::AcidPools) as f32, 4);
                    self.explode(x, y, 3.0, 11.0);
                    for _ in 0..20 {
                        self.ptc(x, y, rr(2.0, 12.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.5, 2.5), 80, PtK::Acid, [140, 230, 60]);
                    }
                    if k > 0 {
                        self.lights.truncate(n1 + 1);
                    }
                }
            }
            Sk::IceArmor => {
                self.player.ice = 600;
                self.snd(Sfx::Frost, px, py, 0.9);
                self.light(px, py - 20.0, 110.0, 1.4, [0.6, 0.85, 1.0], 20);
                self.banner_small("ICE ARMOUR");
            }
            Sk::BladeFury => {
                self.player.whirl = 240;
                self.snd(Sfx::Whirl, px, py, 1.0);
                self.banner_small("BLADE FURY");
            }
            Sk::Meteor if self.rank(Sk::MeteorShower) > 0 => {
                // the main meteor is cast as usual; three more follow it in
                for k in 0..3 {
                    let a = k as f32 * 2.1 + rr(-0.3, 0.3);
                    self.meteors.push(Meteor { x: mx + a.cos() * 46.0, y: my + a.sin() * 30.0, t: -12 - k * 12 });
                }
                return false;
            }
            _ => return false,
        }
        true
    }

    /// every frame: shields, regeneration, Last Stand
    pub fn update_talents(&mut self) {
        let p = &mut self.player;
        if p.stand_cd > 0 {
            p.stand_cd -= 1;
        }
        if p.frost > 0 {
            p.frost -= 1;
            if p.frost % 6 == 0 {
                let (x, y) = (p.x, p.y);
                self.ptc(x + rr(-6.0, 6.0), y, rr(5.0, 45.0), 0.0, 0.0, -0.3, 30, PtK::Ice, [200, 235, 255]);
            }
        }
        let p = &mut self.player;
        if p.dead {
            return;
        }
        let regen = self.player.sk.r(Sk::Regeneration) as f32;
        let p = &mut self.player;
        if regen > 0.0 {
            p.hp = (p.hp + regen / 60.0).min(p.maxhp);
        }
        let (x, y) = (p.x, p.y);
        if p.shield > 0 {
            p.shield -= 1;
            if p.shield % 5 == 0 {
                let a = rnd() * std::f32::consts::TAU;
                self.ptc(x + a.cos() * 9.0, y, rr(10.0, 50.0), 0.0, 0.0, 0.2, 10, PtK::Spark, [190, 210, 255]);
            }
            self.light(x, y - 25.0, 60.0, 0.6, [0.5, 0.6, 1.0], 1);
        }
        let p = &mut self.player;
        if p.ice > 0 {
            p.ice -= 1;
            if p.ice % 4 == 0 {
                self.ptc(x + rr(-8.0, 8.0), y, rr(5.0, 50.0), 0.0, 0.0, 0.3, 30, PtK::Ice, [200, 235, 255]);
            }
        }
    }

    /// a killing blow, unless Last Stand is ready
    pub fn last_stand(&mut self) -> bool {
        let p = &self.player;
        if p.hp > 0.0 || p.dead || p.stand_cd > 0 || self.rank(Sk::LastStand) == 0 {
            return false;
        }
        let p = &mut self.player;
        p.hp = 1.0;
        p.inv = 120;
        p.stand_cd = 7200;
        let (x, y) = (p.x, p.y);
        self.banner = Some(("LAST STAND".into(), "NOT YET".into(), 120));
        self.snd(Sfx::LevelUp, x, y, 0.8);
        self.flash = self.flash.max(0.2);
        true
    }

    /// enemies that hit you while a shield is up get struck back
    pub fn retaliate(&mut self, i: usize) {
        if !self.enemies[i].alive {
            return;
        }
        if self.player.shield > 0 {
            let (px, py) = (self.player.x, self.player.y);
            let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
            self.add_bolt((px, py - 30.0), (ex, ey - 30.0));
            self.zap_enemy(i, 26.0);
        }
        if self.player.ice > 0 {
            self.enemies[i].slow = self.enemies[i].slow.max(150);
        }
    }

    /// the poison cloud field: hurts and slows foes inside it
    pub fn poison_field(&mut self, x: f32, y: f32, t: i32, r: f32) {
        for _ in 0..2 {
            let a = rnd() * std::f32::consts::TAU;
            let d = rnd().sqrt() * r;
            self.ptc(x + a.cos() * d, y + a.sin() * d * 0.6, rr(2.0, 30.0), rr(-0.2, 0.2), rr(-0.1, 0.1), rr(0.0, 0.2), 80, PtK::Cloud, [70, 120, 40]);
        }
        if t % 10 != 0 {
            return;
        }
        let dmg = 4.0 * (1.0 + 0.4 * self.rank(Sk::Caustic) as f32) * self.spell_power();
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive || e.k.bony() && rnd() < 0.5 || ((e.x - x).powi(2) + ((e.y - y) / 0.6).powi(2)).sqrt() > r {
                continue;
            }
            let e = &mut self.enemies[i];
            e.slow = e.slow.max(30);
            e.body.each_in(rr(-5.0, 5.0), -rr(8.0, 50.0), 2.0, |p, _, _| p.acid = p.acid.max(120));
            self.hurt(i, dmg, [140, 230, 70], false);
        }
    }

    /// shards from a shattered enemy
    pub fn shatter_shards(&mut self, x: f32, y: f32) {
        let r = self.rank(Sk::Shatterstorm);
        if r == 0 {
            return;
        }
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive || ((e.x - x).powi(2) + ((e.y - y) * 1.4).powi(2)).sqrt() > 48.0 {
                continue;
            }
            self.enemies[i].slow = self.enemies[i].slow.max(90);
            self.hurt(i, 18.0 * r as f32, [180, 225, 255], false);
        }
        for _ in 0..30 {
            let a = rnd() * std::f32::consts::TAU;
            self.ptc(x, y, rr(5.0, 30.0), a.cos() * rr(1.0, 3.5), a.sin() * rr(0.6, 2.0), rr(0.5, 2.0), 50, PtK::Ice, [210, 240, 255]);
        }
    }
}
