// Spells unlocked through the skill tree, plus the shared ways of hurting enemies
// (blade cuts, lightning zaps) that the sword and several spells use.
use crate::body::*;
use crate::game::*;
use crate::rng::*;
use crate::skills::*;
use crate::world::*;
use crate::audio::Sfx;

pub struct Field {
    pub k: u8, // 0 thunderstorm (follows you), 1 acid rain, 2 blizzard
    pub x: f32,
    pub y: f32,
    pub t: i32,
    pub max: i32,
    pub r: f32,
}
pub struct Meteor {
    pub x: f32,
    pub y: f32,
    pub t: i32,
}
pub const METEOR_T: i32 = 56;
pub struct Spike {
    pub x: f32,
    pub y: f32,
    pub t: i32,
    pub h: f32,
}

impl Game {
    pub fn rank(&self, s: Sk) -> i32 {
        self.player.sk.r(s)
    }
    pub fn burn_mult(&self) -> f32 {
        1.0 + 0.3 * self.rank(Sk::Kindling) as f32
    }

    /// shared damage bookkeeping for an enemy
    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        let e = &mut self.enemies[i];
        e.hp -= dmg;
        e.aggro = true;
        e.hpbar = 180;
        e.flash = e.flash.max(4);
        let (x, y) = (e.x, e.y);
        self.float(x + rr(-4.0, 4.0), y - 62.0, format!("{}", dmg.round() as i32), col, big);
    }

    /// A blade stroke through an enemy at height `h` (negative = above the feet), coming from `src_x`.
    pub fn cut_enemy(&mut self, i: usize, h: f32, power: f32, dmg: f32, src_x: f32, crit: bool) {
        if self.enemies[i].frozen > 0 {
            self.shatter(i);
            return;
        }
        let e = &mut self.enemies[i];
        let slope = rr(-0.3, 0.3);
        let from_left = src_x < e.x;
        let (ax, bx) = if from_left { (-30.0, 30.0) } else { (30.0, -30.0) };
        let cut = e.body.cut(ax, h + ax * slope, bx, h + bx * slope, power);
        let metal = cut.iter().filter(|c| c.2 == PM_METAL).count();
        let flesh = cut.len() - metal;
        let bone = cut.iter().filter(|c| c.2 == PM_BONE).count();
        let mut d = if flesh > 0 { dmg + flesh as f32 * 0.6 } else { dmg * 0.2 };
        let (sx, sy) = (e.x, e.y);
        let k = if flesh == 0 && metal > 0 {
            Sfx::HitMetal
        } else if bone * 2 > flesh || e.k.bony() {
            Sfx::HitBone
        } else {
            Sfx::HitFlesh
        };
        self.sounds.push(crate::audio::Snd { k, x: sx, y: sy, v: 1.0 });
        if crit {
            d *= 1.6;
        }
        e.stun = e.stun.max(if e.k.big() { 4 } else { 10 });
        let dir = if from_left { 1.0 } else { -1.0 };
        let kb = if e.k == Kind::King { 0.1 } else if e.k.big() { 0.4 } else { 1.6 };
        e.vx += dir * kb;
        if flesh > 0 {
            e.bleed = 160;
        }
        let (ex, ey, k) = (e.x, e.y, e.k);
        for c in cut.iter().step_by(2) {
            if c.2 == PM_METAL {
                for _ in 0..3 {
                    self.ptc(ex + c.0, ey, -c.1, dir * rr(0.5, 2.5) + rr(-1.0, 1.0), rr(-1.0, 1.0), rr(0.5, 2.0), 20, PtK::Spark, [255, 230, 160]);
                }
            } else {
                self.blood(ex + c.0, ey + 1.0, -c.1, 2, (dir, 0.0), k);
            }
        }
        // bloodthirst heals on cuts that draw blood
        let bt = self.rank(Sk::Bloodthirst);
        if bt > 0 && flesh > 0 && !k.bony() {
            let heal = (flesh as f32 * 0.25 * bt as f32).min(3.0 * bt as f32);
            let p = &mut self.player;
            p.hp = (p.hp + heal).min(p.maxhp);
        }
        self.hurt(i, d, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);
    }

    /// A lightning strike on an enemy: damage, stun, a scorched path through the body.
    pub fn zap_enemy(&mut self, i: usize, base: f32) {
        let (zx, zy) = (self.enemies[i].x, self.enemies[i].y);
        self.snd(Sfx::Zap, zx, zy, 0.6);
        let mult = 1.0 + 0.15 * self.rank(Sk::ChainMastery) as f32;
        let e = &mut self.enemies[i];
        let wet = e.wet > 0;
        let dmg = base * mult * if wet { 1.7 } else { 1.0 } * rr(0.85, 1.15);
        e.stun = e.stun.max(26);
        e.twitch = 26;
        if e.frozen > 0 {
            e.frozen = 0;
        }
        let mut x = rr(-3.0, 3.0);
        let mut y = -58.0;
        while y < 0.0 {
            let nx = x + rr(-2.5, 2.5);
            e.body.each_in(nx, y, 1.4, |p, _, _| {
                p.chr = p.chr.saturating_add(160);
                if (p.m == PM_CLOTH || p.m == PM_HAIR) && rnd() < 0.08 {
                    p.burn = p.burn.max(60);
                }
                p.hp = p.hp.saturating_sub(70);
            });
            x = nx.clamp(-6.0, 6.0);
            y += 2.0;
        }
        let (ex, ey) = (e.x, e.y);
        for _ in 0..8 {
            self.ptc(ex, ey, rr(5.0, 50.0), rr(-1.5, 1.5), rr(-1.0, 1.0), rr(0.0, 1.5), 16, PtK::Spark, [200, 220, 255]);
        }
        self.hurt(i, dmg, if wet { [150, 210, 255] } else { [210, 225, 255] }, wet);
    }

    pub fn cast(&mut self, s: Sk, mx: f32, my: f32, inp: &Input) {
        match s {
            Sk::Lightning => self.cast_lightning(mx, my, inp),
            Sk::Fireball => self.cast_fireball(mx, my),
            Sk::Acid => self.cast_acid(mx, my),
            Sk::Nova => self.cast_nova(),
            Sk::BallLightning => {
                let p = &self.player;
                let (dx, dy) = (mx - p.x, my - p.y);
                let l = (dx * dx + dy * dy).sqrt().max(1.0);
                self.projs.push(Proj { k: 2, x: p.x + p.face * 8.0, y: p.y + 1.0, z: 22.0, vx: dx / l * 1.1, vy: dy / l * 1.1, vz: 0.0, life: 330 });
            }
            Sk::Thunderstorm => {
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Thunder, x, y, 1.0);
                self.fields.push(Field { k: 0, x: 0.0, y: 0.0, t: 0, max: 360, r: 150.0 });
                self.flash = self.flash.max(0.2);
                self.banner_small("THE SKY ANSWERS");
            }
            Sk::FireWall => self.cast_firewall(mx, my),
            Sk::Meteor => {
                self.meteors.push(Meteor { x: mx, y: my, t: 0 });
            }
            Sk::AcidRain => self.fields.push(Field { k: 1, x: mx, y: my, t: 0, max: 300, r: 34.0 }),
            Sk::AcidFlood => self.cast_flood(mx, my),
            Sk::IceSpikes => {
                let (px, py) = (self.player.x, self.player.y);
                let (dx, dy) = (mx - px, my - py);
                let l = (dx * dx + dy * dy).sqrt().max(1.0);
                for k in 0..13 {
                    let d = 12.0 + k as f32 * 8.0;
                    self.spikes.push(Spike { x: px + dx / l * d + rr(-2.0, 2.0), y: py + dy / l * d + rr(-1.5, 1.5), t: -(k * 3), h: rr(12.0, 18.0) });
                }
            }
            Sk::Blizzard => {
                self.snd(Sfx::Frost, mx, my, 1.0);
                self.fields.push(Field { k: 2, x: mx, y: my, t: 0, max: 360, r: 46.0 })
            }
            Sk::Whirlwind => {
                self.player.whirl = 30;
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Whirl, x, y, 1.0);
            }
            _ => {}
        }
    }

    pub fn banner_small(&mut self, s: &str) {
        let (x, y) = (self.player.x, self.player.y);
        self.float(x, y - 72.0, s.to_string(), [220, 220, 255], false);
    }

    fn cast_firewall(&mut self, mx: f32, my: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let (dx, dy) = (mx - px, my - py);
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        let (ux, uy) = (dx / l, dy / l);
        for k in 0..90 {
            let d = 14.0 + k as f32;
            for w in -2..=2 {
                let (x, y) = ((px + ux * d - uy * w as f32) as i32, (py + uy * d + ux * w as f32) as i32);
                if !World::inb(x, y) || self.world.solid(x, y) {
                    continue;
                }
                let m = self.world.at(x, y);
                if m == WATER {
                    continue;
                }
                if is_wall(m) || m == OIL {
                    self.world.ignite(x, y);
                } else {
                    self.world.set_life(x, y, FIRE, rr(170.0, 255.0) as u8);
                }
            }
            if k % 3 == 0 {
                self.ptc(px + ux * d, py + uy * d, 2.0, 0.0, 0.0, rr(0.5, 1.2), 30, PtK::Flame, [255, 170, 60]);
            }
        }
        self.light(px + ux * 50.0, py + uy * 50.0, 140.0, 1.5, [1.0, 0.55, 0.2], 40);
    }

    fn cast_flood(&mut self, mx: f32, my: f32) {
        self.world.splash(mx, my, ACID, 9.0, 16);
        for _ in 0..6 {
            let (ox, oy) = (rr(-14.0, 14.0), rr(-9.0, 9.0));
            self.world.splash(mx + ox, my + oy, ACID, 4.0, 12);
        }
        for _ in 0..60 {
            let a = rnd() * std::f32::consts::TAU;
            let sp = rr(0.5, 2.5);
            self.ptc(mx, my, 2.0, a.cos() * sp, a.sin() * sp * 0.7, rr(1.0, 3.0), 200, PtK::Acid, [130, 230, 60]);
        }
        self.light(mx, my, 90.0, 1.2, [0.5, 1.0, 0.3], 40);
        self.shake = self.shake.max(3.0);
    }

    pub fn update_spells(&mut self) {
        // ---- fields: thunderstorm, acid rain, blizzard
        let mut k = 0;
        while k < self.fields.len() {
            self.fields[k].t += 1;
            let f = &self.fields[k];
            let (kind, t, r) = (f.k, f.t, f.r);
            let (x, y) = if kind == 0 { (self.player.x, self.player.y) } else { (f.x, f.y) };
            match kind {
                0 => {
                    if t % 16 == 0 {
                        let cands: Vec<usize> = (0..self.enemies.len())
                            .filter(|&i| {
                                let e = &self.enemies[i];
                                e.alive && ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt() < r
                            })
                            .collect();
                        let (tx, ty, who) = if !cands.is_empty() && rnd() < 0.8 {
                            let i = cands[ri(cands.len())];
                            (self.enemies[i].x, self.enemies[i].y, Some(i))
                        } else {
                            (x + rr(-r, r) * 0.7, y + rr(-r, r) * 0.5, None)
                        };
                        self.add_bolt((tx + rr(-30.0, 30.0), ty - 200.0), (tx, ty));
                        self.snd(Sfx::Thunder, tx, ty, 0.9);
                        self.flash = self.flash.max(0.08);
                        self.shake = self.shake.max(2.0);
                        self.world.ignite(tx as i32, ty as i32);
                        if let Some(i) = who {
                            self.zap_enemy(i, 22.0);
                        }
                        for dy in -2..=2 {
                            for dx in -3..=3 {
                                let (cx, cy) = (tx as i32 + dx, ty as i32 + dy);
                                if conductive(self.world.at(cx, cy)) {
                                    let cells = self.world.conduct(cx, cy, 3000);
                                    let set: std::collections::HashSet<usize> = cells.into_iter().collect();
                                    for j in 0..self.enemies.len() {
                                        let e = &self.enemies[j];
                                        if e.alive && Some(j) != who && World::inb(e.x as i32, e.y as i32) && set.contains(&(e.y as usize * W + e.x as usize)) {
                                            self.zap_enemy(j, 14.0);
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
                1 => {
                    for _ in 0..3 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd().sqrt() * r;
                        self.ptc(x + a.cos() * d, y + a.sin() * d * 0.7, rr(55.0, 70.0), 0.0, 0.0, -1.6, 120, PtK::Acid, [150, 230, 70]);
                    }
                    if t % 3 == 0 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd() * r;
                        self.ptc(x + a.cos() * d, y + a.sin() * d * 0.5, rr(66.0, 76.0), rr(-0.1, 0.1), 0.0, 0.0, 90, PtK::Cloud, [44, 70, 34]);
                    }
                    if t % 6 == 0 {
                        let cor = self.rank(Sk::Corrosion);
                        for i in 0..self.enemies.len() {
                            let e = &mut self.enemies[i];
                            if !e.alive || ((e.x - x).powi(2) + ((e.y - y) / 0.7).powi(2)).sqrt() > r {
                                continue;
                            }
                            let (rx, ry) = (rr(-6.0, 6.0), -rr(10.0, 58.0));
                            e.body.each_in(rx, ry, 3.0 + cor as f32, |p, _, _| p.acid = p.acid.max(200));
                            e.aggro = true;
                            e.hpbar = 180;
                            e.hp -= 1.0;
                        }
                    }
                }
                _ => {
                    for _ in 0..4 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd().sqrt() * r;
                        self.ptc(x + a.cos() * d - 20.0, y + a.sin() * d * 0.7, rr(40.0, 60.0), 0.9, 0.2, -1.4, 60, PtK::Ice, [210, 235, 255]);
                    }
                    if t % 2 == 0 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd() * r;
                        self.ptc(x + a.cos() * d, y + a.sin() * d * 0.6, rr(10.0, 50.0), rr(0.3, 0.8), 0.0, 0.0, 70, PtK::Cloud, [190, 210, 230]);
                    }
                    // freeze puddles, put out fires
                    for _ in 0..30 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd() * r;
                        let (cx, cy) = ((x + a.cos() * d) as i32, (y + a.sin() * d * 0.7) as i32);
                        match self.world.at(cx, cy) {
                            WATER | BLOOD => self.world.set(cx, cy, ICE),
                            FIRE => self.world.set(cx, cy, E),
                            _ => {}
                        }
                    }
                    let br = self.rank(Sk::Brittle);
                    for i in 0..self.enemies.len() {
                        let e = &mut self.enemies[i];
                        if !e.alive || e.frozen > 0 || ((e.x - x).powi(2) + ((e.y - y) / 0.7).powi(2)).sqrt() > r {
                            continue;
                        }
                        e.chill += 2;
                        e.hp -= 0.06;
                        e.aggro = true;
                        e.hpbar = 180;
                        if e.chill > 70 {
                            e.chill = 0;
                            e.frozen = 150 + br * 60;
                            e.body.tint_all(|p| p.burn = 0);
                        }
                    }
                }
            }
            let light = match kind {
                1 => Some((60.0, 0.5, [0.5, 1.0, 0.3])),
                2 => Some((90.0, 0.7, [0.6, 0.85, 1.0])),
                _ => None,
            };
            if let Some((lr, li, lc)) = light {
                self.light(x, y, lr, li, lc, 1);
            }
            if self.fields[k].t >= self.fields[k].max {
                self.fields.swap_remove(k);
            } else {
                k += 1;
            }
        }
        // ---- meteors
        let mut k = 0;
        while k < self.meteors.len() {
            self.meteors[k].t += 1;
            let m = &self.meteors[k];
            let (x, y, t) = (m.x, m.y, m.t);
            let f = t as f32 / METEOR_T as f32;
            let (hx, hz) = (x + (1.0 - f) * 90.0, (1.0 - f) * 240.0);
            for _ in 0..4 {
                self.ptc(hx + rr(-3.0, 3.0), y, hz + rr(-3.0, 3.0), rr(0.2, 0.8), 0.0, rr(0.2, 1.0), rr(15.0, 30.0) as i32, PtK::Flame, [255, 170, 60]);
            }
            if t % 2 == 0 {
                self.ptc(hx, y, hz, 0.4, 0.0, 0.3, 90, PtK::Smoke, [50, 44, 46]);
            }
            self.light(hx, y, 120.0, 1.5, [1.0, 0.55, 0.2], 1);
            if t >= METEOR_T {
                let r = 22.0 + 2.0 * self.rank(Sk::Kindling) as f32;
                self.explode(x, y, 4.0, r);
                let n1 = self.lights.len();
                for _ in 0..3 {
                    self.explode(x + rr(-12.0, 12.0), y + rr(-8.0, 8.0), 2.0, r * 0.5);
                }
                self.lights.truncate(n1);
                // crater: smash walls hard
                let ri_ = (r * 0.6) as i32;
                for dy in -ri_..=ri_ {
                    for dx in -ri_..=ri_ {
                        if dx * dx + dy * dy <= ri_ * ri_ && self.world.solid(x as i32 + dx, y as i32 + dy) && rnd() < 0.8 {
                            self.world.set(x as i32 + dx, y as i32 + dy, RUBBLE);
                        }
                    }
                }
                self.snd(Sfx::Crumble, x, y, 1.0);
                self.shake = 12.0;
                self.hitstop = 6;
                self.flash = self.flash.max(0.12);
                self.meteors.swap_remove(k);
            } else {
                k += 1;
            }
        }
        // ---- ice spikes
        let mut hits: Vec<(usize, f32)> = vec![];
        for s in self.spikes.iter_mut() {
            s.t += 1;
            if s.t == 1 {
                for (i, e) in self.enemies.iter().enumerate() {
                    if e.alive && (e.x - s.x).abs() < 9.0 && (e.y - s.y).abs() < 6.0 && !hits.iter().any(|h| h.0 == i) {
                        hits.push((i, s.x));
                    }
                }
            }
        }
        let new: Vec<(f32, f32)> = self.spikes.iter().filter(|s| s.t == 1).map(|s| (s.x, s.y)).collect();
        for (x, y) in new {
            self.snd(Sfx::Shatter, x, y, 0.4);
            for _ in 0..6 {
                self.ptc(x, y, rr(0.0, 12.0), rr(-1.0, 1.0), rr(-0.6, 0.6), rr(0.5, 2.0), 50, PtK::Ice, [210, 240, 255]);
            }
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                    match self.world.at(cx, cy) {
                        WATER | BLOOD => self.world.set(cx, cy, ICE),
                        FIRE => self.world.set(cx, cy, E),
                        _ => {}
                    }
                }
            }
            self.light(x, y, 40.0, 0.8, [0.6, 0.85, 1.0], 20);
            self.shake = self.shake.max(1.5);
        }
        let br = self.rank(Sk::Brittle);
        for (i, sx) in hits {
            if !self.enemies[i].alive {
                continue;
            }
            if self.enemies[i].frozen > 0 {
                self.shatter(i);
                continue;
            }
            let h = -rr(4.0, 11.0);
            self.cut_enemy(i, h, 26.0, 14.0, sx - 10.0 * (self.enemies[i].x - self.player.x).signum(), false);
            if self.enemies[i].alive {
                self.enemies[i].frozen = 50 + br * 20;
            }
        }
        self.spikes.retain(|s| s.t < 60);
    }

    /// whirlwind: three spinning cuts at everything around you
    pub fn update_whirl(&mut self) {
        let w = self.player.whirl;
        if w <= 0 {
            return;
        }
        self.player.whirl -= 1;
        if w == 26 || w == 16 || w == 6 {
            let (px, py) = (self.player.x, self.player.y);
            let a0 = rnd() * std::f32::consts::TAU;
            self.slashes.push(Slash { x: px, y: py - 22.0, a0, a1: a0 + std::f32::consts::TAU, t: 9 });
            let edge = self.rank(Sk::Edge);
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if !e.alive {
                    continue;
                }
                let d = ((e.x - px).powi(2) + ((e.y - py) * 1.3).powi(2)).sqrt();
                if d > 36.0 + if e.k.big() { 8.0 } else { 0.0 } {
                    continue;
                }
                let h = -[10.0, 18.0, 26.0, 34.0][ri(4)];
                let crit = rnd() < 0.08 * self.rank(Sk::Executioner) as f32 + 0.1;
                let pw = 20.0 + edge as f32 * 8.0 + if crit { 20.0 } else { 0.0 };
                self.cut_enemy(i, h, pw, 9.0 * (1.0 + 0.1 * edge as f32), px, crit);
            }
            self.shake = self.shake.max(2.0);
        }
    }

    /// ball lightning: zaps nearby enemies as it drifts
    pub fn ball_zap(&mut self, x: f32, y: f32, z: f32, life: i32) {
        if life % 9 != 0 {
            return;
        }
        let mut best = None;
        let mut bd = 70.0;
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.alive {
                continue;
            }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt();
            if d < bd {
                bd = d;
                best = Some(i);
            }
        }
        if let Some(i) = best {
            let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
            self.add_bolt((x, y - z), (ex, ey - 28.0));
            self.zap_enemy(i, 9.0);
        }
    }

    /// level up: called whenever XP is gained
    pub fn gain_xp(&mut self, n: i32) {
        let sk = &mut self.player.sk;
        sk.xp += n;
        let mut ups = 0;
        while sk.xp >= xp_need(sk.level) {
            sk.xp -= xp_need(sk.level);
            sk.level += 1;
            sk.points += 1;
            ups += 1;
        }
        if ups > 0 {
            let (lx, ly) = (self.player.x, self.player.y);
            self.snd(Sfx::LevelUp, lx, ly, 1.0);
            let lvl = self.player.sk.level;
            self.recalc();
            let p = &mut self.player;
            p.hp = p.maxhp;
            p.mana = p.maxmana;
            let (x, y) = (p.x, p.y);
            self.banner = Some((format!("LEVEL {}", lvl), "SKILL POINT GAINED  -  PRESS K".into(), 200));
            for _ in 0..80 {
                let a = rnd() * std::f32::consts::TAU;
                self.ptc(x + a.cos() * 8.0, y + a.sin() * 4.0, rr(0.0, 50.0), a.cos() * 0.3, a.sin() * 0.2, rr(0.5, 1.6), 60, PtK::Heal, [255, 220, 120]);
            }
            self.light(x, y, 160.0, 2.0, [1.0, 0.85, 0.5], 50);
        }
    }

    /// derived stats from passives
    pub fn recalc(&mut self) {
        let v = self.rank(Sk::Vigor) as f32;
        let w = self.rank(Sk::Wisdom) as f32;
        let p = &mut self.player;
        p.maxhp = 100.0 + 15.0 * v;
        p.maxmana = 60.0 + 15.0 * w;
        p.hp = p.hp.min(p.maxhp);
        p.mana = p.mana.min(p.maxmana);
    }

    /// keep the land populated: new groups rise away from the player
    pub fn respawn(&mut self) {
        self.spawn_t -= 1;
        if self.spawn_t > 0 {
            return;
        }
        self.spawn_t = 1500;
        let alive = self.enemies.iter().filter(|e| e.alive).count();
        if alive >= 18 {
            return;
        }
        self.enemies.retain(|e| e.alive);
        let lvl = self.player.sk.level;
        for _ in 0..40 {
            let (x, y) = (rr(40.0, W as f32 - 40.0), rr(40.0, H as f32 - 40.0));
            let d = ((x - self.player.x).powi(2) + (y - self.player.y).powi(2)).sqrt();
            if d < 260.0 || self.world.at(x as i32, y as i32) != E || self.world.aux[y as usize * W + x as usize] & AUX_INDOOR != 0 {
                continue;
            }
            let n = 3 + ri(3);
            for j in 0..n {
                let (ex, ey) = (x + rr(-14.0, 14.0), y + rr(-10.0, 10.0));
                if self.world.at(ex as i32, ey as i32) != E {
                    continue;
                }
                let k = if lvl >= 3 && j == 0 && rnd() < 0.3 {
                    Kind::Brute
                } else if rnd() < 0.4 {
                    Kind::Skeleton
                } else {
                    Kind::Zombie
                };
                let mut e = Game::new_enemy(k, ex, ey);
                let s = 1.0 + 0.12 * (lvl - 1) as f32;
                e.hp *= s;
                e.maxhp *= s;
                e.dmg *= 1.0 + 0.08 * (lvl - 1) as f32;
                self.enemies.push(e);
            }
            self.banner = Some(("THE DEAD STIR".into(), "MORE HAVE RISEN".into(), 140));
            break;
        }
    }
}
