// Gear in play: how each weapon type hits, on-hit affix effects and legendary powers,
// loot drops and pickups, the inventory screen's actions, and gear showing on the body.
use crate::audio::{Sfx, Snd};
use crate::body::*;
use crate::game::*;
use crate::items::*;
use crate::rng::*;
use crate::skills::*;
use crate::world::*;

pub struct GroundItem {
    pub item: Item,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vz: f32,
    pub t: i32,
}

pub const BAG_SIZE: usize = 32;

impl Game {
    pub fn wkind(&self) -> WKind {
        self.player.equip[0].as_ref().map(|w| w.wk).unwrap_or(WKind::Sword)
    }
    pub fn has_legend(&self, l: Legend) -> bool {
        self.player.equip[0].as_ref().and_then(|w| w.legend) == Some(l)
    }
    /// share of damage that gets through your armour
    pub fn armor_mult(&self) -> f32 {
        let a = self.player.gear.armor;
        (1.0 - a / (a + 60.0)) * self.ward_mult() * (1.0 - 0.04 * self.rank(Sk::Toughness) as f32) * if self.player.ice > 0 { 0.6 } else { 1.0 }
    }
    pub fn swing_frames(&self) -> i32 {
        ((wstats(self.wkind()).frames as f32) / (1.0 + self.player.gear.swift / 100.0) / self.haste_mult() / (1.0 + 0.06 * self.rank(Sk::Momentum) as f32)).round().max(6.0) as i32
    }

    /// the moment the weapon lands
    pub fn weapon_hit(&mut self, aim: f32, inp: &Input) {
        let wk = self.wkind();
        let ws = wstats(wk);
        let g = self.player.gear;
        let (px, py) = (self.player.x, self.player.y);
        let (mwx, mwy) = self.mouse_world(inp);
        self.hit_barrels(px, py, aim, ws.reach, ws.arc);
        self.hit_cracks(px, py, aim, ws.reach);
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32 + g.crit / 100.0;
        let mut targets: Vec<(f32, usize)> = vec![];
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.alive {
                continue;
            }
            let (dx, dy) = (e.x - px, e.y - py);
            let big = if e.k.big() { 8.0 } else { 0.0 };
            let d = (dx * dx + dy * dy * 1.6).sqrt();
            if d > ws.reach + big {
                continue;
            }
            let a = dy.atan2(dx);
            let da = (a - aim).sin().atan2((a - aim).cos()).abs();
            if da > ws.arc * (1.0 + 0.15 * self.rank(Sk::Cleave) as f32) && d > 10.0 {
                continue;
            }
            targets.push((d, i));
        }
        targets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if wk == WKind::Spear {
            targets.truncate(2);
        }
        let mut crit_any = false;
        let any = !targets.is_empty();
        for &(_, i) in &targets {
            let e = &self.enemies[i];
            let (rel_mx, rel_my) = (mwx - e.x, mwy - e.y);
            let h = if rel_mx.abs() < 14.0 && rel_my < 2.0 && rel_my > -66.0 {
                rel_my
            } else {
                -[8.0, 14.0, 24.0, 30.0, 38.0][ri(5)] * if e.k == Kind::King { 1.5 } else if e.k == Kind::Brute { 1.15 } else { 1.0 }
            };
            let crit = rnd() < crit_ch;
            crit_any |= crit;
            let power = (ws.power + edge as f32 * 8.0 + g.keen + if crit { 20.0 + 6.0 * ex_r as f32 } else { 0.0 }) * (1.0 + 0.2 * self.rank(Sk::Dismember) as f32);
            let dmg = g.weapon_dmg * ws.dmg_mult * (1.0 + g.dmg_pct / 100.0) * (1.0 + 0.1 * edge as f32);
            match wk {
                WKind::Hammer => self.crush_enemy(i, h, dmg, crit),
                WKind::Axe => {
                    self.cut_enemy(i, h, power, dmg, px, crit);
                    // the axe bites deep: a second cut right under the first
                    let e = &mut self.enemies[i];
                    if e.alive && e.frozen == 0 {
                        let side = if px < e.x { -30.0 } else { 30.0 };
                        e.body.cut(side, h + 1.6, -side, h + 1.6, power * 0.6);
                    }
                }
                WKind::Spear => {
                    let e = &mut self.enemies[i];
                    if e.frozen == 0 {
                        e.body.each_in(0.0, h, 3.4, |p, _, _| {
                            p.m = 0;
                            p.im = 0;
                        });
                        e.body.dirty = true;
                    }
                    self.cut_enemy(i, h, power * 0.8, dmg, px, crit);
                }
                WKind::Dagger => {
                    self.cut_enemy(i, h, power, dmg, px, crit);
                    let e = &mut self.enemies[i];
                    if !e.k.bony() {
                        e.bleed_dot = 150;
                        e.bleed_amt = dmg * 0.012;
                    }
                }
                WKind::Sword => self.cut_enemy(i, h, power, dmg, px, crit),
            }
            if self.enemies[i].alive {
                self.on_hit(i, dmg, h);
            }
            if g.leech > 0.0 {
                let p = &mut self.player;
                p.hp = (p.hp + g.leech).min(p.maxhp);
            }
        }
        if self.has_legend(Legend::Pyre) {
            self.fire_breath(aim);
        }
        if any {
            let heavy = wk == WKind::Hammer || wk == WKind::Axe;
            self.hitstop = if crit_any || heavy { 5 } else { 3 };
            self.shake = self.shake.max(if heavy { 4.5 } else if crit_any { 4.0 } else { 2.5 });
        }
    }

    /// a hammer blow: smashes a hole, shatters bone and ice, throws them back
    fn crush_enemy(&mut self, i: usize, h: f32, dmg: f32, crit: bool) {
        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }
        let e = &mut self.enemies[i];
        let bony = e.k.bony();
        let mut bones = 0;
        e.body.each_in(0.0, h, if crit { 8.0 } else { 6.5 }, |p, d, _| {
            if rnd() < 0.8 - d * 0.05 {
                if p.m == PM_BONE {
                    bones += 1;
                }
                Body::hurt_px(p, 255);
            }
        });
        e.body.dirty = true;
        e.stun = e.stun.max(if e.k.big() { 12 } else { 32 });
        let dir = if self.player.x < e.x { 1.0 } else { -1.0 };
        e.vx += dir * if e.k.big() { 1.0 } else { 3.2 };
        let (ex, ey, k) = (e.x, e.y, e.k);
        let mult = if bony { 1.5 } else { 1.0 } * if crit { 1.6 } else { 1.0 };
        self.sounds.push(Snd { k: if bony || bones > 3 { Sfx::HitBone } else { Sfx::HitFlesh }, x: ex, y: ey, v: 1.2 });
        self.sounds.push(Snd { k: Sfx::Crumble, x: ex, y: ey, v: 0.5 });
        for _ in 0..12 {
            if bony {
                self.ptc(ex, ey, -h, dir * rr(0.5, 2.5), rr(-0.8, 0.8), rr(0.5, 2.0), 200, PtK::Debris, [210, 200, 180]);
            } else {
                self.blood(ex, ey, -h, 1, (dir, 0.0), k);
            }
        }
        self.hit_h = Some(h);
        self.hurt(i, dmg * mult, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);
    }

    /// affixes and legendary powers that trigger on every hit
    pub(crate) fn on_hit(&mut self, i: usize, dmg: f32, h: f32) {
        let g = self.player.gear;
        let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
        if g.burning || self.has_legend(Legend::Pyre) {
            self.enemies[i].body.each_in(0.0, h, 6.0, |p, _, _| {
                if matches!(p.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) {
                    p.burn = p.burn.max(110);
                }
            });
        }
        if g.corroding || self.has_legend(Legend::MiresKiss) {
            self.enemies[i].body.each_in(0.0, h, 6.0, |p, _, _| p.acid = p.acid.max(200));
            if self.has_legend(Legend::MiresKiss) {
                self.world.splash(ex, ey + 1.0, ACID, 4.0, 3);
            }
        }
        if (g.thundering && rnd() < 0.22) || self.has_legend(Legend::Stormcaller) {
            self.add_bolt((ex + rr(-20.0, 20.0), ey - 190.0), (ex, ey - 30.0));
            self.zap_enemy(i, dmg * 0.7);
        }
        let freeze = (g.freezing && rnd() < 0.15) || self.has_legend(Legend::WintersGrudge);
        let e = &mut self.enemies[i];
        if freeze && e.alive && e.frozen == 0 {
            e.frozen = if e.k == Kind::King { 40 } else { 130 };
            e.body.tint_all(|p| p.burn = 0);
            self.sounds.push(Snd { k: Sfx::Frost, x: ex, y: ey, v: 0.6 });
        }
    }

    /// Pyre: a jet of fire from the spear tip
    fn fire_breath(&mut self, aim: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let (ux, uy) = (aim.cos(), aim.sin() * 0.7);
        for k in 0..30 {
            let d = 12.0 + k as f32 * 1.6;
            let (x, y) = (px + ux * d, py + uy * d);
            if rnd() < 0.5 {
                self.world.ignite(x as i32, y as i32);
            }
            self.ptc(x, y, rr(14.0, 30.0), ux * rr(0.5, 2.0), uy * rr(0.5, 2.0), rr(0.0, 0.6), rr(14.0, 28.0) as i32, PtK::Flame, [255, 160, 60]);
        }
        for e in self.enemies.iter_mut() {
            if !e.alive {
                continue;
            }
            let (dx, dy) = (e.x - px, e.y - py);
            let along = dx * ux + dy * uy;
            let off = (dx * uy - dy * ux).abs();
            if along > 0.0 && along < 62.0 && off < 12.0 {
                e.body.each_in(0.0, -30.0, 30.0, |p, _, _| {
                    if matches!(p.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) && rnd() < 0.3 {
                        p.burn = p.burn.max(120);
                    }
                });
                e.aggro = true;
            }
        }
        self.light(px + ux * 40.0, py + uy * 40.0, 100.0, 1.4, [1.0, 0.55, 0.2], 14);
        self.sounds.push(Snd { k: Sfx::Fireball, x: px, y: py, v: 0.6 });
    }

    // ------------------------------------------------------------ loot
    /// drops are rare: most of the dead carry nothing, champions and brutes often do,
    /// bosses always do. Plain (common) items stop dropping once you are past the start
    /// and become a little gold instead, so what you do find is worth a look
    pub fn drop_loot(&mut self, x: f32, y: f32, k: Kind, elite: bool) {
        let (chance, rolls, luck) = loot_odds(k, elite);
        let mut to_bag = 0;
        for r in 0..rolls {
            if rnd() < chance {
                let mut item = roll(self.alvl, luck);
                if k.boss() {
                    // a boss always gives at least one rare, and its hoard goes straight
                    // into your bag so it can't be lost by dying or quitting
                    for _ in 0..40 {
                        if r > 0 || item.rarity >= Rarity::Rare {
                            break;
                        }
                        item = roll(self.alvl, luck);
                    }
                    if self.player.bag.len() < BAG_SIZE {
                        if item.rarity == Rarity::Legendary {
                            self.banner = Some((item.name.clone(), "LEGENDARY - IN YOUR BAG".into(), 300));
                        }
                        self.player.bag.push(item);
                        to_bag += 1;
                        continue;
                    }
                }
                if item.rarity == Rarity::Common && self.player.sk.level >= 4 {
                    self.pickups.push(crate::game::Pickup { k: 0, x: x + rr(-6.0, 6.0), y: y + rr(-3.0, 3.0), z: 10.0, vz: 1.8, amt: item.value() * 2, t: 0 });
                    continue;
                }
                self.ground.push(GroundItem { item, x: x + rr(-8.0, 8.0), y: y + rr(-4.0, 4.0), z: 12.0, vz: rr(1.8, 3.2), t: -(r as i32) * 8 });
            }
        }
        if to_bag > 0 {
            self.float(x, y - 96.0, format!("{} ITEM{} ADDED TO YOUR BAG", to_bag, if to_bag > 1 { "S" } else { "" }), [120, 220, 255], true);
        }
    }

    /// is this better than what you're wearing in its slot?
    pub fn is_upgrade(&self, it: &Item) -> bool {
        let s = SLOTS.iter().position(|x| *x == it.slot).unwrap_or(0);
        match &self.player.equip[s] {
            Some(cur) => it.score() > cur.score() * 1.05,
            None => true,
        }
    }

    pub fn update_ground(&mut self) {
        let (px, py) = (self.player.x, self.player.y);
        let mut take = None;
        for (k, gi) in self.ground.iter_mut().enumerate() {
            gi.t += 1;
            if gi.t < 0 {
                continue;
            }
            gi.vz -= 0.16;
            gi.z += gi.vz;
            if gi.z < 0.0 {
                gi.z = 0.0;
                gi.vz = if gi.vz < -0.8 { -gi.vz * 0.35 } else { 0.0 };
            }
            let d = ((gi.x - px).powi(2) + (gi.y - py).powi(2)).sqrt();
            if d < 9.0 && gi.t > 30 && take.is_none() && !self.player.dead {
                take = Some(k);
            }
        }
        if let Some(k) = take {
            if self.player.bag.len() >= BAG_SIZE {
                if self.frame % 90 == 0 {
                    self.float(px, py - 60.0, "BAG FULL".into(), [255, 120, 120], false);
                }
                return;
            }
            let gi = self.ground.swap_remove(k);
            let it = gi.item;
            self.float(px, py - 62.0, it.name.clone(), it.rarity.col(), it.rarity >= Rarity::Rare);
            if self.is_upgrade(&it) {
                self.float(px, py - 74.0, "UPGRADE - PRESS I".into(), [120, 255, 120], true);
                self.snd(Sfx::Chime, px, py, 0.6);
            }
            self.sounds.push(Snd { k: if it.rarity >= Rarity::Rare { Sfx::Chime } else { Sfx::Coin }, x: px, y: py, v: 0.8 });
            if it.rarity == Rarity::Legendary {
                self.banner = Some((it.name.clone(), "LEGENDARY - PRESS I TO EQUIP".into(), 220));
            }
            self.player.bag.push(it);
            self.save();
        }
    }

    // ------------------------------------------------------------ inventory
    pub fn update_inventory(&mut self, inp: &Input) {
        self.inv_hover = crate::hud::inv_at(inp.mouse, self.view);
        if self.inv_hover.map(|h| h.0) != Some(1) || self.inv_hover.map(|h| h.1) != self.sell_confirm {
            self.sell_confirm = None;
        }
        let click = inp.lmb && !self.prev.lmb;
        let rclick = inp.rmb && !self.prev.rmb;
        match self.inv_hover {
            Some((0, s)) if click => self.unequip(s),
            Some((1, i)) if click && i < self.player.bag.len() => self.equip_from_bag(i),
            Some((1, i)) if rclick && i < self.player.bag.len() && self.player.bag[i].rarity >= Rarity::Rare && self.sell_confirm != Some(i) => {
                self.sell_confirm = Some(i);
                self.toast = Some((format!("RIGHT-CLICK AGAIN TO SELL {}", self.player.bag[i].name), 150));
            }
            Some((1, i)) if rclick && i < self.player.bag.len() => {
                self.sell_confirm = None;
                let it = self.player.bag.remove(i);
                let v = it.value();
                self.player.gold += v;
                let (x, y) = (self.player.x, self.player.y);
                self.toast = Some((format!("SOLD {} FOR {} GOLD", it.name, v), 120));
                self.sounds.push(Snd { k: Sfx::Coin, x, y, v: 0.9 });
                self.save();
            }
            _ => {}
        }
    }
    pub fn equip_from_bag(&mut self, i: usize) {
        let it = self.player.bag.remove(i);
        let s = SLOTS.iter().position(|x| *x == it.slot).unwrap();
        if let Some(old) = self.player.equip[s].replace(it) {
            self.player.bag.insert(i, old);
        }
        self.apply_gear();
        let (x, y) = (self.player.x, self.player.y);
        self.sounds.push(Snd { k: Sfx::HitMetal, x, y, v: 0.4 });
        self.save();
    }
    pub fn unequip(&mut self, s: usize) {
        if self.player.bag.len() >= BAG_SIZE {
            return;
        }
        if let Some(it) = self.player.equip[s].take() {
            self.player.bag.push(it);
            self.apply_gear();
            let (x, y) = (self.player.x, self.player.y);
            self.sounds.push(Snd { k: Sfx::Click, x, y, v: 0.6 });
            self.save();
        }
    }

    /// recompute stats from gear and rebuild how the character looks
    pub fn apply_gear(&mut self) {
        self.player.gear = gear_stats(&self.player.equip);
        self.recalc();
        if self.player.dead {
            return;
        }
        let mut b = humanoid(Kind::Player);
        match &self.player.equip[0] {
            Some(w) => b.parts[T_WEAPON] = weapon_part(w.wk, w.rarity, w.legend.is_some(), b.parts[T_WEAPON].attach),
            None => b.parts[T_WEAPON].on = false,
        }
        let tint = |r: Option<Rarity>| match r {
            None => [0.75, 0.62, 0.5],
            Some(Rarity::Common) => [1.0, 1.0, 1.0],
            Some(Rarity::Magic) => [0.82, 0.92, 1.25],
            Some(Rarity::Rare) => [1.25, 1.08, 0.62],
            Some(Rarity::Legendary) => [0.7, 0.55, 0.55],
        };
        let at = tint(self.player.equip[2].as_ref().map(|i| i.rarity));
        let ht = tint(self.player.equip[1].as_ref().map(|i| i.rarity));
        for (k, part) in b.parts.iter_mut().enumerate() {
            if k == T_WEAPON {
                continue;
            }
            let t = if k == T_HEAD { ht } else { at };
            for px in part.px.iter_mut() {
                if px.m == PM_METAL {
                    px.c = [(px.c[0] as f32 * t[0]).min(255.0) as u8, (px.c[1] as f32 * t[1]).min(255.0) as u8, (px.c[2] as f32 * t[2]).min(255.0) as u8];
                }
            }
        }
        b.mirror = self.player.face < 0.0;
        b.compute();
        self.player.body = b;
    }
}

/// (chance to drop, how many tries, rarity luck) for a kill
pub fn loot_odds(k: Kind, elite: bool) -> (f32, i32, f32) {
    let (chance, rolls, luck) = match k {
        Kind::King => (1.0, 2, 0.45),
        Kind::Chief => (1.0, 3, 0.5),
        Kind::Brute => (0.25, 1, 0.12),
        Kind::Bandit | Kind::Crossbow => (0.035, 1, 0.02),
        Kind::Bloater | Kind::Skeleton | Kind::Archer => (0.03, 1, 0.0),
        _ => (0.025, 1, 0.0),
    };
    if elite {
        ((chance * 8.0f32).min(0.7), rolls, luck + 0.2)
    } else {
        (chance, rolls, luck)
    }
}
