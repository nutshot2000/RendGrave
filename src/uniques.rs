// The four uniques that turn the simulation into a weapon (see docs/SLICE_PLAN.md step 3).
use crate::body::*;
use crate::game::*;
use crate::items::*;
use crate::rng::*;
use crate::world::*;

impl Game {
    /// the Splintering Maul: a shattered foe's ice is driven on into those behind it
    pub fn splinter(&mut self, x: f32, y: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let (dx, dy) = (x - px, y - py);
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        let (ux, uy) = (dx / l, dy / l);
        let dmg = 30.0 * (1.0 + 0.1 * self.alvl as f32);
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive {
                continue;
            }
            let (ex, ey) = (e.x - x, e.y - y);
            let along = ex * ux + ey * uy;
            let side = (ex * uy - ey * ux).abs();
            if along > 2.0 && along < 100.0 && side < 22.0 {
                self.hurt(i, dmg, [180, 225, 255], false);
                let e = &mut self.enemies[i];
                if e.alive && !e.k.boss() && e.frozen == 0 {
                    // frozen in turn: the next blow shatters them too
                    e.frozen = 90;
                    e.body.tint_all(|p| p.burn = 0);
                }
            }
        }
        for _ in 0..40 {
            let s = rr(0.0, 1.0);
            self.ptc(x + ux * s * 20.0, y + uy * s * 10.0, rr(5.0, 40.0), ux * rr(2.0, 5.0) + rr(-0.5, 0.5), uy * rr(1.0, 3.0) + rr(-0.4, 0.4), rr(0.0, 1.5), 40, PtK::Ice, [210, 240, 255]);
        }
    }

    /// the Conduit: every hit arcs from the target to severed metal armour on the ground,
    /// and from each piece on to the nearest other foe
    pub fn conduit_arc(&mut self, i: usize, dmg: f32) {
        let (tx, ty) = (self.enemies[i].x, self.enemies[i].y);
        let mut pieces: Vec<(f32, f32, f32)> = self
            .gibs
            .iter()
            .filter(|g| g.part.px.iter().filter(|p| p.m == PM_METAL).count() >= 3)
            .filter(|g| ((g.x - tx).powi(2) + (g.y - ty).powi(2)).sqrt() < 110.0)
            .map(|g| (g.x, g.y, g.z))
            .collect();
        pieces.truncate(3);
        for (gx, gy, gz) in pieces {
            self.add_bolt((tx, ty - 30.0), (gx, gy - gz));
            let mut best = None;
            let mut bd = 90.0;
            for (j, e) in self.enemies.iter().enumerate() {
                if j == i || !e.alive {
                    continue;
                }
                let d = ((e.x - gx).powi(2) + (e.y - gy).powi(2)).sqrt();
                if d < bd {
                    bd = d;
                    best = Some(j);
                }
            }
            if let Some(j) = best {
                let (ex, ey) = (self.enemies[j].x, self.enemies[j].y);
                self.add_bolt((gx, gy - gz), (ex, ey - 30.0));
                self.zap_enemy(j, dmg * 0.6);
            }
            self.light(gx, gy, 60.0, 1.0, [0.6, 0.8, 1.0], 8);
        }
    }

    /// the Ember Ring: fire near you pours mana into you
    pub fn ember_ring(&mut self) {
        self.player.gear_ember = self.has_legend(Legend::EmberRing);
        if !self.player.gear_ember || self.frame % 10 != 0 || self.player.dead {
            return;
        }
        let (px, py) = (self.player.x as i32, self.player.y as i32);
        let mut fires = vec![];
        for dy in -24..=24 {
            for dx in -40..=40 {
                if (dx + dy) % 2 == 0 && self.world.at(px + dx, py + dy) == FIRE {
                    fires.push((px + dx, py + dy));
                }
            }
        }
        if fires.is_empty() {
            return;
        }
        let gain = (fires.len().min(30) as f32) * 0.1;
        let p = &mut self.player;
        p.mana = (p.mana + gain).min(p.maxmana);
        for _ in 0..3 {
            let (fx, fy) = fires[ri(fires.len())];
            let (dx, dy) = (px - fx, py - fy);
            self.ptc(fx as f32, fy as f32, 4.0, dx as f32 * 0.04, dy as f32 * 0.04, 0.4, 25, PtK::Ember, [120, 160, 255]);
        }
    }

    /// a boss gives one of the four uniques you don't have yet
    pub fn give_unique(&mut self) {
        let have = |g: &Game, l: Legend| g.player.bag.iter().chain(g.player.equip.iter().flatten()).any(|it| it.legend == Some(l));
        let missing: Vec<Legend> = UNIQUES.iter().copied().filter(|&l| !have(self, l)).collect();
        if missing.is_empty() || self.player.bag.len() >= crate::gear::BAG_SIZE {
            return;
        }
        let it = legendary(missing[ri(missing.len())], self.alvl);
        self.banner = Some((it.name.clone(), "A UNIQUE - IN YOUR BAG. PRESS I".into(), 320));
        self.player.bag.push(it);
    }
}

/// standing in oil with the Mudwalker's leathers
pub fn mud_bonus(g: &Game) -> f32 {
    if g.has_legend(Legend::Mudwalker) && g.under(g.player.x, g.player.y)[3] { 1.25 } else { 1.0 }
}
