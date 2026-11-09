// Boss fights. Bosses are only killed by emptying their health: acid and fire don't melt
// them away pixel by pixel (they wear the health down instead), frost can't shatter them and
// blades only scar them. Each has a signature move, calls for help at half health and
// becomes enraged at a quarter:
//   the Hollow King winds up a ground slam (a red ring warns you) that hurls you back;
//   the Bandit Chief charges, and lobs lit powder kegs at you.
use crate::audio::{Sfx, Snd};
use crate::body::Kind;
use crate::game::*;
use crate::rng::*;
use crate::world::*;

pub const SLAM_T: i32 = 50;
pub const SLAM_R: f32 = 78.0;

impl Game {
    /// called every frame for every living enemy; does nothing for ordinary ones
    pub fn boss_moves(&mut self, i: usize) {
        if !self.enemies[i].k.boss() {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let pdead = self.player.dead;
        let e = &mut self.enemies[i];
        let (x, y, bk) = (e.x, e.y, e.k);
        let dist = ((px - x).powi(2) + ((py - y) * 1.4).powi(2)).sqrt();
        // a quarter health left: faster, harder, angrier
        if !e.enraged && e.hp < e.maxhp * 0.25 {
            e.enraged = true;
            e.speed *= 1.3;
            e.dmg *= 1.2;
            e.windup = (e.windup as f32 * 0.75) as i32;
            e.skill_t = e.skill_t.min(60);
            let name = if bk == Kind::King { "THE HOLLOW KING" } else { "THE BANDIT CHIEF" };
            self.banner = Some((format!("{} IS ENRAGED", name), "".into(), 120));
            self.snd(if bk == Kind::King { Sfx::KingRoar } else { Sfx::Roar }, x, y, 1.3);
            self.shake = self.shake.max(8.0);
            return;
        }
        if !e.aggro || e.frozen > 0 || pdead {
            return;
        }
        if e.skill_t > 0 {
            e.skill_t -= 1;
        }
        if e.slam > 0 {
            // rooted while winding up the slam
            e.slam -= 1;
            e.stun = e.stun.max(1);
            e.vx = 0.0;
            e.vy = 0.0;
            if e.slam == 0 {
                self.king_slam(i);
            }
            return;
        }
        if e.skill_t > 0 {
            return;
        }
        let enraged = e.enraged;
        match bk {
            Kind::King if dist < 130.0 => {
                e.slam = SLAM_T;
                e.skill_t = if enraged { 220 } else { 340 };
                self.snd(Sfx::KingRoar, x, y, 1.0);
            }
            Kind::Chief if dist > 70.0 && dist < 280.0 => {
                e.skill_t = if enraged { 170 } else { 280 };
                let n = if enraged { 2 } else { 1 };
                for k in 0..n {
                    // lob a lit keg to land at (or near) where you're standing
                    let (tx, ty) = (px + rr(-14.0, 14.0) + k as f32 * 30.0, py + rr(-8.0, 8.0));
                    let (dx, dy) = (tx - x, ty - y);
                    let d = (dx * dx + dy * dy).sqrt().max(1.0);
                    let sp = 3.0;
                    let t = d / sp;
                    let vz = (0.5 * 0.09 * t * t - 34.0) / t;
                    self.projs.push(Proj { k: 5, x: x + dx / d * 10.0, y: y + dy / d * 4.0, z: 34.0, vx: dx / d * sp, vy: dy / d * sp, vz, life: 400 });
                }
                self.sounds.push(Snd { k: Sfx::Shout, x, y, v: 1.0 });
                self.float(x, y - 80.0, "CATCH!".into(), [255, 170, 60], true);
            }
            _ => {}
        }
    }

    /// the Hollow King's slam lands
    fn king_slam(&mut self, i: usize) {
        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
        let dmg = self.enemies[i].dmg * 1.5;
        self.snd(Sfx::Explosion, x, y, 1.1);
        self.snd(Sfx::Crumble, x, y, 1.0);
        self.shake = self.shake.max(12.0);
        self.hitstop = self.hitstop.max(4);
        self.light(x, y, 140.0, 1.2, [0.5, 0.6, 1.0], 20);
        // a ring of rubble and cracked ground
        for k in 0..60 {
            let a = k as f32 / 60.0 * std::f32::consts::TAU;
            let r = rr(SLAM_R * 0.3, SLAM_R);
            self.ptc(x + a.cos() * r, y + a.sin() * r * 0.7, 2.0, a.cos() * rr(0.5, 2.0), a.sin() * rr(0.3, 1.2), rr(1.0, 3.0), 200, PtK::Debris, [90, 84, 92]);
        }
        let ri_ = SLAM_R as i32;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                let d = ((dx * dx) as f32 + (dy * dy) as f32 / 0.49).sqrt();
                if d < SLAM_R && rnd() < 0.25 * (1.0 - d / SLAM_R) + 0.05 {
                    let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                    if World::inb(cx, cy) {
                        let j = cy as usize * W + cx as usize;
                        self.world.scorch[j] = self.world.scorch[j].saturating_add(70);
                    }
                }
            }
        }
        let (px, py) = (self.player.x, self.player.y);
        let d = ((px - x).powi(2) + ((py - y) / 0.7).powi(2)).sqrt();
        if d < SLAM_R && self.player.inv == 0 && !self.player.dead {
            let f = 1.0 - d / SLAM_R * 0.5;
            let hit = dmg * f * self.armor_mult();
            self.player.hp -= hit;
            self.player.hurt = 1.0;
            self.player.inv = 20;
            // hurled back across the floor
            let l = d.max(1.0);
            self.player.slip = 22;
            self.player.slide = ((px - x) / l * 5.0, (py - y) / l * 3.5);
            self.float(px, py - 60.0, format!("-{}", hit as i32), [255, 80, 80], false);
            self.snd(Sfx::Hurt, px, py, 1.0);
        }
    }

    /// a thrown keg lands: it becomes a lit powder keg on the ground
    pub fn drop_keg(&mut self, x: f32, y: f32) {
        let (bx, by) = (x as i32, y as i32);
        for yy in by - 3..by + 1 {
            for xx in bx - 4..bx + 5 {
                if self.world.at(xx, yy) == E {
                    self.world.set(xx, yy, WOOD);
                }
            }
        }
        let mut b = crate::barrels::Barrel::new(x, y, 0, &self.world);
        b.fuse = 55;
        self.barrels.push(b);
        self.snd(Sfx::Wood, x, y, 0.9);
        self.snd(Sfx::Fuse, x, y, 0.8);
    }

    /// Bosses and champions don't shatter outright: the ice cracks off them for a chunk of
    /// damage instead. Returns false when nothing cracked, so the blow lands as normal.
    pub fn boss_shatter(&mut self, i: usize) -> Option<bool> {
        let e = &self.enemies[i];
        let boss = e.k.boss();
        // a champion with plenty of health left takes a heavy crack, not instant death
        let champ = !boss && e.elite && e.hp > e.maxhp * 0.3;
        if !boss && !champ {
            return None;
        }
        // only the first crack of each freeze hurts; then a boss is immune until it thaws
        let fresh = !boss || e.thaw == 0 || e.thaw > 180;
        if !fresh {
            let e = &mut self.enemies[i];
            e.frozen = 0;
            return Some(false);
        }
        let dmg = if boss { e.maxhp * 0.02 } else { e.maxhp * 0.3 };
        let (x, y) = (e.x, e.y);
        self.snd(Sfx::Shatter, x, y, 0.9);
        for _ in 0..30 {
            self.ptc(x, y, rr(10.0, 60.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.0, 2.0), 60, PtK::Ice, [200, 230, 255]);
        }
        // still frozen while it's hurt, so Deep Freeze counts
        self.hurt(i, dmg, [180, 220, 255], true);
        let e = &mut self.enemies[i];
        e.frozen = 0;
        if boss {
            e.thaw = 180;
        }
        Some(true)
    }
}
