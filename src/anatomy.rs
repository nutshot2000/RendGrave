// Enemies whose bodies change the fight:
//   burning panic   - a burning living enemy runs for its nearest friend and sets it alight
//   bloater sac     - cut a bloater and it leaks acid as it walks: a trail you can lead
//                     through its friends (their feet melt in it)
//   loaded crossbow - sever a crossbowman's firing arm and he drops his loaded crossbow; it
//                     goes off when struck (in the direction you hit it), blasted or burned,
//                     and the bolt goes through everyone in line, you included
//   shield-bearers  - block blows from the front with a wooden or iron shield. Get behind
//                     them (they turn slowly), break the shield, cut off the shield arm, or
//                     set a wooden one on fire
use crate::audio::Sfx;
use crate::body::*;
use crate::game::*;
use crate::rng::*;
use crate::world::*;

/// Enemy::flags
pub const F_PANIC: u8 = 1;
pub const F_DROPPED: u8 = 2;
pub const F_PUNCTURED: u8 = 4;

#[derive(Clone, Copy)]
pub struct LooseBow {
    pub x: f32,
    pub y: f32,
    /// which way it points while lying there
    pub dir: f32,
    pub t: i32,
}

impl Game {
    // ------------------------------------------------------------ burning panic
    /// a burning, living enemy runs at its nearest unburnt friend
    pub fn panic_want(&mut self, i: usize, spd: f32) -> Option<(f32, f32)> {
        let e = &self.enemies[i];
        if e.k.boss() || e.k.bony() || e.burning < 20 || e.frozen > 0 || e.stun > 0 {
            return None;
        }
        let (x, y) = (e.x, e.y);
        let mut best = None;
        let mut bd = 150.0;
        for (j, f) in self.enemies.iter().enumerate() {
            if j == i || !f.alive || f.k.boss() || f.burning > 5 || f.frozen > 0 {
                continue;
            }
            let d = ((f.x - x).powi(2) + ((f.y - y) * 1.3).powi(2)).sqrt();
            if d < bd {
                bd = d;
                best = Some(j);
            }
        }
        let j = best?;
        if self.enemies[i].flags & F_PANIC == 0 {
            self.enemies[i].flags |= F_PANIC;
            let cry = if self.enemies[i].k.bandit() { "AAARGH!" } else { "!" };
            self.float(x, y - 66.0, cry.into(), [255, 150, 60], false);
        }
        let (fx, fy) = (self.enemies[j].x, self.enemies[j].y);
        if bd < 12.0 {
            // grabs hold of a friend: the fire spreads
            self.enemies[j].body.each_in(0.0, -20.0, 12.0, |p, _, _| {
                if matches!(p.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) && rnd() < 0.5 {
                    p.burn = p.burn.max(130);
                }
            });
            self.enemies[j].aggro = true;
            return None;
        }
        let l = bd.max(1.0);
        Some(((fx - x) / l * spd * 1.3, (fy - y) / l * spd * 1.3))
    }

    // ------------------------------------------------------------ every frame, per enemy
    pub fn anatomy_tick(&mut self, i: usize) {
        let f = self.frame;
        let e = &mut self.enemies[i];
        let (x, y) = (e.x, e.y);
        // a punctured bloater leaks acid as it walks
        if e.leak > 0 {
            e.leak -= 1;
            if f % 5 == 0 {
                self.world.splash(x + rr(-2.0, 2.0), y + 1.0, ACID, 2.2, 1);
                self.ptc(x + rr(-6.0, 6.0), y, rr(10.0, 26.0), rr(-0.3, 0.3), 0.0, -0.4, 30, PtK::Acid, [140, 230, 70]);
            }
        }
        // shields: they turn slowly, burn if wooden, and fall with the arm that holds them
        let e = &mut self.enemies[i];
        if e.shield > 0.0 {
            let want = e.face;
            if e.sface != want {
                e.sturn += 1;
                if e.sturn > 26 {
                    e.sface = want;
                    e.sturn = 0;
                }
            } else {
                e.sturn = 0;
            }
            if e.shield_wood && e.burning > 8 {
                e.shield -= 0.35;
                if f % 3 == 0 {
                    let sx = x + e.sface * 7.0;
                    self.ptc(sx + rr(-2.0, 2.0), y, rr(20.0, 36.0), 0.0, 0.0, 0.7, 20, PtK::Ember, [255, 150, 50]);
                }
                let e = &mut self.enemies[i];
                if e.shield <= 0.0 {
                    self.shield_gone(i, "SHIELD BURNT");
                    return;
                }
            }
            let e = &mut self.enemies[i];
            if !e.body.whole(T_ARMB) {
                self.shield_gone(i, "SHIELD DROPPED");
            }
        }
    }

    // ------------------------------------------------------------ bloater
    /// any wound opens a bloater's sac
    pub fn puncture(&mut self, i: usize) {
        let e = &mut self.enemies[i];
        if e.k != Kind::Bloater || e.flags & F_PUNCTURED != 0 || !e.alive {
            return;
        }
        e.flags |= F_PUNCTURED;
        e.leak = 900;
        let (x, y) = (e.x, e.y);
        self.float(x, y - 66.0, "PUNCTURED".into(), [140, 230, 70], false);
        self.snd(Sfx::Acid, x, y, 0.8);
    }

    // ------------------------------------------------------------ shields
    /// is the player in front of this shield?
    fn shield_faces_player(&self, i: usize) -> bool {
        let e = &self.enemies[i];
        e.shield > 0.0 && ((self.player.x - e.x).signum() == e.sface || (self.player.x - e.x).abs() < 3.0)
    }

    /// a blow from the front lands on the shield instead (returns true if it was blocked)
    pub fn shield_block(&mut self, i: usize, dmg: f32, heavy: bool) -> bool {
        if !self.shield_faces_player(i) {
            return false;
        }
        let e = &mut self.enemies[i];
        let mult = if heavy { 2.0 } else if e.shield_wood { 1.0 } else { 0.6 };
        e.shield -= dmg * mult;
        e.aggro = true;
        e.stun = e.stun.max(6);
        e.vx += e.sface * -0.6;
        let (x, y, wood, side) = (e.x, e.y, e.shield_wood, e.sface);
        for _ in 0..6 {
            self.ptc(x + side * 7.0, y, rr(22.0, 34.0), side * rr(0.3, 1.5), rr(-0.5, 0.5), rr(0.5, 1.5), 20, if wood { PtK::Debris } else { PtK::Spark }, if wood { [130, 90, 54] } else { [255, 230, 160] });
        }
        self.snd(if wood { Sfx::Wood } else { Sfx::HitMetal }, x, y, 0.9);
        if self.enemies[i].shield <= 0.0 {
            self.shield_gone(i, "SHIELD BROKEN");
        } else {
            self.float(x, y - 62.0, "BLOCKED".into(), [200, 200, 210], false);
        }
        true
    }

    fn shield_gone(&mut self, i: usize, why: &str) {
        let e = &mut self.enemies[i];
        e.shield = 0.0;
        e.speed /= 0.85;
        let (x, y, wood, side) = (e.x, e.y, e.shield_wood, e.sface);
        for _ in 0..24 {
            self.ptc(x + side * 7.0, y, rr(16.0, 38.0), side * rr(0.5, 2.5), rr(-1.0, 1.0), rr(0.5, 2.5), 200, PtK::Debris, if wood { [130, 90, 54] } else { [130, 134, 146] });
        }
        self.float(x, y - 70.0, why.into(), [255, 214, 90], true);
        self.snd(Sfx::Crumble, x, y, 0.9);
    }

    // ------------------------------------------------------------ the loaded crossbow
    /// parts came off an enemy: a crossbowman who loses his firing arm drops a loaded bow
    pub fn on_sever(&mut self, i: usize, parts: &[PK]) {
        let e = &mut self.enemies[i];
        if e.k != Kind::Crossbow || e.flags & F_DROPPED != 0 {
            return;
        }
        if !parts.iter().any(|p| matches!(p, PK::ArmF | PK::Weapon)) {
            return;
        }
        e.flags |= F_DROPPED;
        let (x, y, face) = (e.x, e.y, e.face);
        let dir = if face > 0.0 { 0.0 } else { std::f32::consts::PI };
        self.loose_bows.push(LooseBow { x: x + face * 8.0, y: y + 3.0, dir, t: 0 });
        self.float(x, y - 76.0, "LOADED CROSSBOW!".into(), [255, 214, 90], true);
    }

    /// a swing that meets a loose crossbow sets it off, aimed the way you hit it
    pub fn hit_bows(&mut self, px: f32, py: f32, aim: f32, reach: f32, arc: f32) {
        for k in (0..self.loose_bows.len()).rev() {
            let b = self.loose_bows[k];
            let (dx, dy) = (b.x - px, b.y - py);
            let d = (dx * dx + dy * dy * 1.6).sqrt();
            let a = dy.atan2(dx) - aim;
            let da = a.sin().atan2(a.cos()).abs();
            if d > reach + 6.0 || (da > arc.min(1.6) && d > 10.0) {
                continue;
            }
            self.loose_bows.swap_remove(k);
            // it flies off the way you struck it
            self.fire_bow(b.x, b.y, dy.atan2(dx), true);
        }
    }

    /// an explosion sets off any crossbow near it, pointing the way it lay
    pub fn bows_blast(&mut self, x: f32, y: f32, r: f32) {
        for k in (0..self.loose_bows.len()).rev() {
            let b = self.loose_bows[k];
            if ((b.x - x).powi(2) + (b.y - y).powi(2)).sqrt() < r * 2.0 + 6.0 {
                self.loose_bows.swap_remove(k);
                self.fire_bow(b.x, b.y, b.dir, false);
            }
        }
    }

    /// blasts and fire set off any crossbow they reach
    pub fn bows_tick(&mut self) {
        let mut go = vec![];
        for (k, b) in self.loose_bows.iter_mut().enumerate() {
            b.t += 1;
            let m = self.world.at(b.x as i32, b.y as i32);
            if m == FIRE || self.world.scorch[(b.y as usize).min(H - 1) * W + (b.x as usize).min(W - 1)] > 200 {
                go.push(k);
            }
        }
        for &k in go.iter().rev() {
            let b = self.loose_bows.swap_remove(k);
            self.fire_bow(b.x, b.y, b.dir, false);
        }
    }

    /// the bolt: a straight line along the ground through everything in its way
    pub fn fire_bow(&mut self, x: f32, y: f32, dir: f32, struck: bool) {
        let (ux, uy) = (dir.cos(), dir.sin() * 0.6);
        self.snd(Sfx::Bow, x, y, 1.0);
        let dmg = 60.0 * (1.0 + 0.12 * self.alvl as f32);
        let mut hit = vec![];
        let mut end = (x, y);
        let mut d = 6.0;
        while d < 320.0 {
            let (bx, by) = (x + ux * d, y + uy * d);
            if self.world.solid(bx as i32, by as i32) {
                break;
            }
            end = (bx, by);
            for (j, e) in self.enemies.iter().enumerate() {
                if e.alive && !hit.contains(&j) && (e.x - bx).abs() < 6.0 && (e.y - by).abs() < 6.0 {
                    hit.push(j);
                }
            }
            let p = &self.player;
            if !struck || d > 30.0 {
                if !p.dead && p.inv == 0 && (p.x - bx).abs() < 5.0 && (p.y - by).abs() < 5.0 {
                    let h = dmg * 0.5 * self.armor_mult();
                    let p = &mut self.player;
                    p.hp -= h;
                    p.hurt = 1.0;
                    p.inv = 20;
                    let (qx, qy) = (p.x, p.y);
                    self.float(qx, qy - 60.0, format!("-{}", h as i32), [255, 80, 80], false);
                }
            }
            d += 3.0;
        }
        // the streak of the bolt
        let n = ((end.0 - x).abs() + (end.1 - y).abs()) as i32 / 4;
        for k in 0..n.max(1) {
            let f = k as f32 / n.max(1) as f32;
            self.ptc(x + (end.0 - x) * f, y + (end.1 - y) * f, 20.0, ux * 0.4, 0.0, 0.0, 14, PtK::Spark, [255, 230, 170]);
        }
        let n_hit = hit.len();
        for j in hit {
            let ex = self.enemies[j].x;
            self.cut_enemy(j, -20.0, 45.0, dmg, ex - ux * 10.0, false);
        }
        if n_hit >= 2 {
            self.float(end.0, end.1 - 60.0, format!("{} IN ONE SHOT", n_hit), [255, 214, 90], true);
        }
        self.snd(Sfx::ArrowHit, end.0, end.1, 0.9);
        self.shake = self.shake.max(3.0);
    }
}
