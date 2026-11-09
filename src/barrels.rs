// Barrels: powder kegs that blow up, oil barrels that spill slick (and burning) oil,
// acid barrels that spill acid. Each stands on a small block of WOOD cells in the world,
// so fire, explosions and acid reach it through the simulation; blades hit it directly.
// Blasts tear nearby enemies apart and set off other kegs, so one keg can start a chain.
use crate::audio::Sfx;
use crate::body::*;
use crate::game::*;
use crate::rng::*;
use crate::world::*;

#[derive(Clone)]
pub struct Barrel {
    pub x: f32,
    pub y: f32,
    /// 0 powder keg, 1 oil, 2 acid, 3 clay urn, 4 crate
    pub k: u8,
    pub alive: bool,
    /// frames until a lit powder keg goes off (-1: not lit)
    pub fuse: i32,
    /// how many wood cells it stood on when the area was made
    pub cells: i32,
}

impl Barrel {
    pub fn new(x: f32, y: f32, k: u8, w: &World) -> Barrel {
        let mut b = Barrel { x, y, k, alive: true, fuse: -1, cells: 0 };
        b.cells = b.count(w).0.max(1);
        b
    }
    fn rect(&self) -> (i32, i32, i32, i32) {
        (self.x as i32 - 4, self.y as i32 - 3, self.x as i32 + 5, self.y as i32 + 1)
    }
    /// (wood cells left, is fire touching it)
    fn count(&self, w: &World) -> (i32, bool) {
        let (x0, y0, x1, y1) = self.rect();
        let (mut n, mut fire) = (0, false);
        for y in y0 - 1..=y1 {
            for x in x0 - 1..=x1 {
                let m = w.at(x, y);
                if m == WOOD && x >= x0 && x < x1 && y >= y0 && y < y1 {
                    n += 1;
                }
                if m == FIRE {
                    fire = true;
                }
            }
        }
        (n, fire)
    }
}

impl Game {
    /// a swing that lands near a barrel: kegs get lit, oil and acid barrels split open
    pub fn hit_barrels(&mut self, px: f32, py: f32, aim: f32, reach: f32, arc: f32) {
        for i in 0..self.barrels.len() {
            let b = &self.barrels[i];
            if !b.alive {
                continue;
            }
            let (dx, dy) = (b.x - px, b.y - py);
            let d = (dx * dx + dy * dy * 1.6).sqrt();
            let a = dy.atan2(dx) - aim;
            let da = a.sin().atan2(a.cos()).abs();
            if d > reach + 4.0 || (da > arc.min(1.6) && d > 10.0) {
                continue;
            }
            self.strike_barrel(i);
        }
    }

    /// something hit a barrel: a keg's fuse catches, oil and acid barrels split
    pub fn strike_barrel(&mut self, i: usize) {
        let (x, y, bk) = (self.barrels[i].x, self.barrels[i].y, self.barrels[i].k);
        if !self.barrels[i].alive {
            return;
        }
        self.snd(Sfx::Wood, x, y, 0.8);
        if bk == 0 {
            if self.barrels[i].fuse < 0 {
                self.barrels[i].fuse = 70;
                self.float(x, y - 30.0, "FUSE LIT - RUN".into(), [255, 170, 60], true);
            }
        } else if bk >= 3 {
            self.smash(i);
        } else {
            self.break_barrel(i, false);
        }
    }

    /// an urn or a crate breaks open: shards or splinters, and now and then something inside
    pub fn smash(&mut self, i: usize) {
        let (x, y, k) = (self.barrels[i].x, self.barrels[i].y, self.barrels[i].k);
        self.clear_barrel(i);
        let urn = k == 3;
        self.snd(if urn { Sfx::Crumble } else { Sfx::Wood }, x, y, 1.0);
        let col = if urn { [150, 96, 66] } else { [120, 84, 52] };
        for _ in 0..22 {
            self.ptc(x, y, rr(3.0, 12.0), rr(-2.2, 2.2), rr(-1.2, 1.2), rr(1.0, 3.5), 240, PtK::Debris, col);
        }
        if urn {
            // a puff of old dust
            for _ in 0..6 {
                self.ptc(x, y, 6.0, rr(-0.4, 0.4), rr(-0.2, 0.2), 0.3, 60, PtK::Smoke, [110, 100, 90]);
            }
        }
        let lv = self.alvl;
        let r = crate::rng::rnd();
        if r < 0.45 {
            let amt = (3 + ri(6) as i32) * (1 + lv / 2);
            self.pickups.push(Pickup { k: 0, x, y: y + 4.0, z: 8.0, vz: 2.2, amt, t: 0 });
        } else if r < 0.51 {
            self.pickups.push(Pickup { k: 1, x, y: y + 4.0, z: 8.0, vz: 2.2, amt: 1, t: 0 });
        } else if r < 0.525 {
            let item = crate::items::roll(lv, 0.05);
            self.ground.push(crate::gear::GroundItem { item, x, y: y + 6.0, z: 10.0, vz: 2.5, t: 0 });
        }
    }

    pub fn update_barrels(&mut self) {
        for i in 0..self.barrels.len() {
            if !self.barrels[i].alive {
                continue;
            }
            let (left, fire) = self.barrels[i].count(&self.world);
            let b = &mut self.barrels[i];
            let wrecked = left * 10 < b.cells * 7;
            if b.k == 0 {
                if b.fuse < 0 && (fire || wrecked) {
                    // caught in fire: a short fuse; caught in a blast: almost at once
                    b.fuse = if wrecked { 3 + ri(10) as i32 } else { 35 + ri(25) as i32 };
                }
                if b.fuse > 0 {
                    b.fuse -= 1;
                    let (x, y, f) = (b.x, b.y, b.fuse);
                    if f % 2 == 0 {
                        self.ptc(x + rr(-1.0, 1.0), y, 15.0, rr(-0.6, 0.6), rr(-0.3, 0.3), rr(0.6, 1.6), 14, PtK::Spark, [255, 200, 90]);
                    }
                    if f % 16 == 0 {
                        self.snd(Sfx::Fuse, x, y, 0.55);
                    }
                    self.light(x, y - 16.0, 30.0, 0.6, [1.0, 0.6, 0.2], 2);
                } else if b.fuse == 0 {
                    self.keg_boom(i);
                }
            } else if b.k >= 3 {
                // urns only care about being knocked apart; crates also burn
                if wrecked || (fire && b.k == 4) {
                    self.smash(i);
                }
            } else if fire || wrecked {
                self.break_barrel(i, fire);
            }
        }
    }

    fn clear_barrel(&mut self, i: usize) {
        self.barrels[i].alive = false;
        let (x0, y0, x1, y1) = self.barrels[i].rect();
        for y in y0..y1 {
            for x in x0..x1 {
                if matches!(self.world.at(x, y), WOOD | FIRE) {
                    self.world.set(x, y, E);
                }
            }
        }
        let (x, y) = (self.barrels[i].x, self.barrels[i].y);
        for _ in 0..16 {
            self.ptc(x, y, rr(2.0, 12.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(1.0, 3.5), 200, PtK::Debris, [110, 74, 44]);
        }
    }

    /// an oil or acid barrel splits: a wide slick spreads out (burning, if fire did it)
    fn break_barrel(&mut self, i: usize, burning: bool) {
        let (x, y, k) = (self.barrels[i].x, self.barrels[i].y, self.barrels[i].k);
        self.clear_barrel(i);
        self.snd(Sfx::Wood, x, y, 1.0);
        let m = if k == 2 { ACID } else { OIL };
        self.snd(if k == 2 { Sfx::Acid } else { Sfx::Splash }, x, y, 0.8);
        let r = 15.0;
        for dy in -(r as i32)..=r as i32 {
            for dx in -(r as i32) * 2..=(r as i32) * 2 {
                let d = ((dx * dx) as f32 / 3.0 + (dy * dy) as f32).sqrt();
                let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                let edge = r * (0.7 + 0.5 * vnoise(cx as f32 * 0.2, cy as f32 * 0.2, 5));
                if d < edge && self.world.at(cx, cy) == E {
                    self.world.set(cx, cy, m);
                }
            }
        }
        for _ in 0..30 {
            let c = if k == 2 { [130, 220, 60] } else { [40, 34, 30] };
            self.ptc(x, y, rr(4.0, 14.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.5, 2.0), 60, if k == 2 { PtK::Acid } else { PtK::Blood }, c);
        }
        if burning && k == 1 {
            for _ in 0..12 {
                self.world.ignite(x as i32 + rr(-8.0, 8.0) as i32, y as i32 + rr(-4.0, 4.0) as i32);
            }
            self.snd(Sfx::Fireball, x, y, 0.8);
        }
        self.world.wake(x as i32, y as i32);
    }

    fn keg_boom(&mut self, i: usize) {
        let (x, y) = (self.barrels[i].x, self.barrels[i].y);
        self.clear_barrel(i);
        self.blast(x, y - 2.0, 24.0);
    }

    /// a big blast: fire and wreckage, and anything close is torn apart
    pub fn blast(&mut self, x: f32, y: f32, r: f32) {
        self.explode(x, y, 4.0, r * 0.7);
        self.shake = self.shake.max(12.0);
        self.hitstop = self.hitstop.max(5);
        self.flash = self.flash.max(0.18);
        self.light(x, y, 200.0, 1.3, [1.0, 0.55, 0.2], 26);
        for _ in 0..40 {
            let a = rnd() * std::f32::consts::TAU;
            let sp = rr(1.5, 5.0);
            self.ptc(x, y, rr(2.0, 10.0), a.cos() * sp, a.sin() * sp * 0.6, rr(1.0, 4.0), rr(20.0, 45.0) as i32, PtK::Flame, [255, 190, 80]);
        }
        let ring = r * 2.2;
        for i in 0..self.enemies.len() {
            if !self.enemies[i].alive {
                continue;
            }
            let (ex, ey, k) = (self.enemies[i].x, self.enemies[i].y, self.enemies[i].k);
            let (dx, dy) = (ex - x, ey - y);
            let d = (dx * dx + dy * dy * 2.0).sqrt();
            if d > ring {
                continue;
            }
            let f = 1.0 - d / ring;
            let l = d.max(1.0);
            let dir = (dx / l, dy / l);
            let boss = k.boss();
            let dmg = (150.0 * f + 25.0) * if boss { 0.6 } else { 1.0 };
            let e = &mut self.enemies[i];
            e.hp -= dmg;
            e.aggro = true;
            e.hpbar = 180;
            e.flash = 6;
            e.stun = e.stun.max(30);
            e.vx += dir.0 * 6.0 * f;
            e.vy += dir.1 * 3.0 * f;
            let m = e.body.mirror;
            let dead = e.hp <= 0.0;
            let g0 = self.gibs.len();
            if !boss && (dead || f > 0.55) {
                // in pieces
                let e = &mut self.enemies[i];
                e.alive = false;
                e.hp = 0.0;
                let ds = e.body.detach_everything();
                self.spawn_gibs(ex, ey, m, ds, dir, 2.4 + f * 2.5, false);
                if !k.bony() {
                    self.blood(ex, ey, 25.0, 60, dir, k);
                }
                self.float(ex, ey - 64.0, "BLOWN APART".into(), [255, 150, 60], true);
                self.on_kill(i);
            } else if !boss && f > 0.15 {
                // lose a limb or two
                let limbs = [T_ARMF, T_ARMB, T_LEGF, T_LEGB, T_ARMF, T_LEGF];
                let n = 1 + (rnd() < f) as usize;
                let mut ds = vec![];
                for _ in 0..n {
                    let p = limbs[ri(limbs.len())];
                    ds.extend(self.enemies[i].body.detach_part(p));
                }
                self.enemies[i].body.dirty = true;
                if !ds.is_empty() {
                    self.enemies[i].bleed = 200;
                    self.spawn_gibs(ex, ey, m, ds, dir, 2.0 + f * 2.0, false);
                    if !k.bony() {
                        self.blood(ex, ey, 25.0, 30, dir, k);
                    }
                }
            }
            for g in self.gibs[g0..].iter_mut() {
                g.vz += rr(1.5, 4.0) * (0.5 + f);
            }
            let txt = format!("{}", dmg as i32);
            self.float(ex + rr(-4.0, 4.0), ey - 56.0, txt, [255, 170, 80], false);
        }
        // and you, if you stood too close
        let (px, py) = (self.player.x, self.player.y);
        let d = ((px - x).powi(2) + (py - y).powi(2) * 2.0).sqrt();
        if d < r * 1.8 && self.player.inv == 0 && !self.player.dead {
            let f = 1.0 - d / (r * 1.8);
            let dmg = (45.0 * f + 6.0) * self.armor_mult();
            self.player.hp -= dmg;
            self.player.hurt = 1.0;
            self.player.inv = 20;
            self.float(px, py - 60.0, format!("-{}", dmg as i32), [255, 120, 60], false);
            self.snd(Sfx::Hurt, px, py, 1.0);
        }
    }
}
