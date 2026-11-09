// The Hollow King comes apart. Three weak points - his knee plates, his sword arm and his
// breastplate - each have a small pool of their own. Damage that lands on that part of his
// body drains it, and the armour visibly cracks and falls away as it does. Breaking one
// changes the fight:
//   knees  - he drops to one knee, can no longer slam, and walks slower
//   arm    - the sword arm tears off; no more swings, he lunges to grab and throw you
//   chest  - the breastplate falls and bares a glowing core: +40% damage taken
// What you've broken by half health decides his second phase (see areas.rs boss_logic).
use crate::audio::Sfx;
use crate::body::*;
use crate::game::*;
use crate::rng::*;

/// weak points: knees, sword arm, chest
pub const KNEES: usize = 0;
pub const ARM: usize = 1;
pub const CHEST: usize = 2;
pub const WEAK_NAMES: [&str; 3] = ["KNEES", "ARM", "CHEST"];
/// each pool as a share of his health
const POOL: [f32; 3] = [0.07, 0.09, 0.10];
/// the grab: wind-up frames, then a lunge
pub const GRAB_T: i32 = 34;

/// which weak point a hit at height `h` (negative = above the feet) lands on
pub fn band(h: f32) -> Option<usize> {
    if h > -26.0 {
        Some(KNEES)
    } else if h > -38.0 {
        Some(ARM)
    } else if h > -64.0 {
        Some(CHEST)
    } else {
        None
    }
}

fn metal_in(b: &Body, parts: &[usize]) -> i32 {
    parts.iter().filter(|&&p| b.parts[p].on).map(|&p| b.parts[p].px.iter().filter(|q| q.m == PM_METAL).count() as i32).sum()
}
fn region(w: usize) -> &'static [usize] {
    match w {
        KNEES => &[T_LEGF, T_LEGB],
        ARM => &[T_ARMF],
        _ => &[T_TORSO],
    }
}

impl Game {
    pub fn is_king(&self, i: usize) -> bool {
        self.enemies[i].k == Kind::King
    }

    /// damage `dmg` landed on the King at height `h`
    pub fn boss_part_hit(&mut self, i: usize, h: f32, dmg: f32) {
        if !self.is_king(i) || !self.enemies[i].alive {
            return;
        }
        let Some(w) = band(h) else { return };
        let e = &mut self.enemies[i];
        if e.wk[w] <= 0.0 {
            return;
        }
        if e.wk_init[w] == 0 {
            e.wk_init[w] = if w == ARM { e.body.parts[T_ARMF].count().max(1) } else { metal_in(&e.body, region(w)).max(1) };
        }
        e.wk[w] -= dmg / (e.maxhp * POOL[w]);
        let left = e.wk[w].max(0.0);
        // the armour falls away as the pool drains
        if w != ARM {
            let target = (e.wk_init[w] as f32 * left).ceil() as i32;
            let parts = region(w);
            let mut n = metal_in(&e.body, parts);
            let mut tries = 0;
            while n > target && tries < 400 {
                tries += 1;
                let p = parts[ri(parts.len())];
                let part = &mut e.body.parts[p];
                if part.px.is_empty() {
                    continue;
                }
                let k = ri(part.px.len());
                if part.px[k].m == PM_METAL {
                    Body::strip(&mut part.px[k]);
                    n -= 1;
                }
            }
            e.body.dirty = true;
        } else {
            // the arm cracks and darkens
            let part = &mut e.body.parts[T_ARMF];
            for _ in 0..6 {
                let k = ri(part.px.len().max(1));
                if let Some(px) = part.px.get_mut(k) {
                    if px.m != 0 {
                        px.chr = px.chr.saturating_add(60);
                    }
                }
            }
        }
        if e.wk[w] <= 0.0 {
            self.break_weak(i, w);
        }
    }

    fn break_weak(&mut self, i: usize, w: usize) {
        let e = &mut self.enemies[i];
        e.wk[w] = 0.0;
        let (x, y, m) = (e.x, e.y, e.body.mirror);
        e.stun = e.stun.max(if w == KNEES { 120 } else { 80 });
        e.slam = 0;
        e.grab = 0;
        let (text, h) = match w {
            KNEES => {
                // every last plate comes off, and he goes down on one knee
                for &p in region(KNEES) {
                    for px in e.body.parts[p].px.iter_mut() {
                        if px.m == PM_METAL {
                            Body::strip(px);
                        }
                    }
                }
                e.speed *= 0.8;
                ("KNEES SHATTERED", 15.0)
            }
            ARM => ("SWORD ARM SEVERED", 32.0),
            _ => {
                for px in e.body.parts[T_TORSO].px.iter_mut() {
                    if px.m == PM_METAL {
                        Body::strip(px);
                    }
                }
                ("CORE EXPOSED", 45.0)
            }
        };
        e.body.dirty = true;
        if w == ARM {
            let ds = e.body.detach_part(T_ARMF);
            let dir = if self.player.x < x { 1.0 } else { -1.0 };
            self.spawn_gibs(x, y, m, ds, (dir * 1.5, 0.0), 1.2, false);
        }
        for _ in 0..40 {
            let col = if w == ARM { [214, 204, 182] } else { [128, 86, 58] };
            self.ptc(x + rr(-10.0, 10.0), y, h + rr(-6.0, 6.0), rr(-2.5, 2.5), rr(-1.0, 1.0), rr(0.5, 3.0), 200, PtK::Debris, col);
        }
        if w == CHEST {
            self.light(x, y - 45.0, 140.0, 2.0, [1.0, 0.4, 0.3], 60);
        }
        self.float(x, y - 95.0, text.into(), [255, 214, 90], true);
        self.snd(Sfx::Crumble, x, y, 1.2);
        self.snd(Sfx::KingRoar, x, y, 1.0);
        self.shake = self.shake.max(7.0);
        self.hitstop = self.hitstop.max(6);
    }

    /// the one-armed King's grab: wind up, lunge, and throw you if he catches you
    /// (returns true while the move is running)
    pub fn king_grab(&mut self, i: usize) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        let e = &mut self.enemies[i];
        if e.grab == 0 {
            return false;
        }
        let (x, y) = (e.x, e.y);
        if e.grab > 0 {
            // reaching out, rooted: the warning
            e.grab -= 1;
            e.stun = e.stun.max(1);
            e.vx = 0.0;
            e.vy = 0.0;
            if e.grab % 6 == 0 {
                self.ptc(x, y, 40.0, rr(-0.5, 0.5), 0.0, 0.5, 30, PtK::Spark, [255, 80, 60]);
            }
            let e = &mut self.enemies[i];
            if e.grab == 0 {
                let (dx, dy) = (px - x, py - y);
                let l = (dx * dx + dy * dy).sqrt().max(1.0);
                e.vx = dx / l * 7.5;
                e.vy = dy / l * 5.0;
                e.grab = -22;
                self.snd(Sfx::Roar, x, y, 1.0);
            }
            return true;
        }
        // lunging
        e.grab += 1;
        let d = ((px - x).powi(2) + ((py - y) * 1.4).powi(2)).sqrt();
        if d < 26.0 && self.player.inv == 0 && !self.player.dead {
            let dmg = e.dmg * 1.4;
            e.grab = 0;
            let hit = dmg * self.armor_mult();
            let p = &mut self.player;
            p.hp -= hit;
            p.hurt = 1.0;
            p.inv = 30;
            let l = d.max(1.0);
            p.slip = 30;
            p.slide = ((px - x) / l * 6.0, (py - y) / l * 4.0);
            self.float(px, py - 60.0, format!("-{}", hit as i32), [255, 80, 80], false);
            self.float(px, py - 74.0, "THROWN".into(), [255, 140, 90], true);
            self.snd(Sfx::Hurt, px, py, 1.0);
            self.shake = self.shake.max(6.0);
        }
        true
    }
}
