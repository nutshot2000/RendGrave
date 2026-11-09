// Moving between areas: exits and stairs, waypoints, dying back to town,
// enemy scaling by area level, the Hollow King, and the save file.
use crate::body::*;
use crate::game::*;
use crate::areagen::*;
use crate::rng::*;
use crate::skills::*;
use crate::audio::Sfx;

pub const SAVE_FILE: &str = "rendgrave_save.txt";

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Load {
    None,
    Ok,
    /// saved while dead
    Dead,
    /// the save and its backup are both damaged
    Damaged,
}

impl Game {
    /// a fresh game: new seed, start in Ashford (or continue from the save file)
    pub fn start(load: bool) -> Game {
        let seed_ = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        let mut g = Game::blank(seed_);
        g.saving = true;
        let loaded = if load { g.load() } else { Load::None };
        if loaded == Load::Dead {
            // you quit while dead: the same price as waking normally
            g.player.gold = g.player.gold * 9 / 10;
            g.area = AreaId::Ashford;
            g.travel(AreaId::Ashford);
            g.banner = Some(("YOU WAKE IN ASHFORD".into(), "THE DEAD TOOK A TENTH OF YOUR GOLD".into(), 220));
        } else if loaded == Load::Ok {
            // dungeons are rebuilt each visit: you continue at their entrance
            let was = g.area;
            let to = was.surface();
            g.area = AreaId::Ashford;
            g.travel(to);
            g.banner = Some(("WELCOME BACK".into(), to.name(), 200));
            if was.is_dungeon() {
                g.toast = Some(("DUNGEONS CHANGE EACH VISIT - YOU WAIT AT THE ENTRANCE".into(), 300));
            }
            // quit during the last talk with Maren: the act still ends
            if g.has(crate::story::TOLD_CHIEF) && !g.act1_done {
                g.check_act_end();
            }
        } else {
            if loaded == Load::Damaged {
                // keep the damaged file for a look, and don't write over it
                let _ = std::fs::copy(crate::savefile::path(SAVE_FILE), crate::savefile::path("rendgrave_save.damaged.txt"));
            }
            g.apply_gear();
            g.travel(AreaId::Ashford);
            g.banner = Some(("RENDGRAVE".into(), "ASHFORD - THE LAST SAFE HEARTH".into(), 260));
            if loaded == Load::Damaged {
                g.toast = Some(("YOUR SAVE WAS DAMAGED - A COPY IS IN RENDGRAVE_SAVE.DAMAGED.TXT".into(), 600));
            }
        }
        g
    }

    /// enemy stats scale with the area, not with the player
    pub fn scaled_enemy(&self, k: Kind, x: f32, y: f32) -> Enemy {
        let mut e = Game::new_enemy(k, x, y);
        let l = (self.alvl - 1) as f32;
        e.hp *= 1.0 + 0.18 * l;
        e.maxhp = e.hp;
        e.dmg *= 1.0 + 0.1 * l;
        // shield-bearers in the crypt and at Ravenhold
        let shields_here = match k {
            Kind::Skeleton => matches!(self.area, AreaId::Crypt(_)),
            Kind::Bandit => matches!(self.area, AreaId::Ravenhold | AreaId::Cellars(_)),
            _ => false,
        };
        if shields_here && rnd() < 0.25 {
            e.shield_wood = rnd() < 0.5;
            e.shield_max = (if e.shield_wood { 40.0 } else { 60.0 }) * (1.0 + 0.18 * (self.alvl - 1) as f32);
            e.shield = e.shield_max;
            e.speed *= 0.85;
            return e;
        }
        // now and then a champion: tougher, glowing, better loot
        if !k.boss() && self.area != AreaId::Ashford && rnd() < 0.07 {
            e.elite = true;
            e.hp *= 2.6;
            e.maxhp = e.hp;
            e.dmg *= 1.3;
            e.speed *= 1.1;
            e.aff = 1 + ri(5) as u8;
            if e.aff == 4 {
                e.speed *= 1.5;
            }
        }
        e
    }

    pub fn xp_for(&self, k: Kind) -> i32 {
        let base = base_xp(k);
        let area = 0.6 + 0.2 * (self.alvl - 1) as f32;
        // far above an area, its enemies teach you little
        let gap = self.player.sk.level - self.alvl;
        let penalty = if gap > 6 { 0.2 } else if gap > 3 { 0.5 } else { 1.0 };
        (base * area * penalty) as i32
    }
}

/// XP for a kill before area scaling
pub fn base_xp(k: Kind) -> f32 {
    {
        match k {
            Kind::King => 600.0,
            Kind::Chief => 900.0,
            Kind::Robber => 60.0,
            Kind::Brute => 90.0,
            Kind::Bloater => 40.0,
            Kind::Bandit => 34.0,
            Kind::Crossbow => 36.0,
            Kind::Archer => 30.0,
            Kind::Skeleton => 26.0,
            Kind::Crawler => 16.0,
            _ => 20.0,
        }
    }
}

impl Game {

    /// leave the current area and arrive in another
    pub fn travel(&mut self, to: AreaId) {
        let from = self.area;
        let seed_ = if to.is_dungeon() { self.seed ^ self.frame.wrapping_mul(0x9E37_79B9) ^ next() } else { self.seed };
        let (world, sp) = generate(to, seed_);
        self.world = world;
        self.nav_dirty = true;
        self.area = to;
        self.alvl = to.level();
        let (king_dead, chief_dead) = (self.king_dead, self.chief_dead);
        self.boss_spot = sp.enemies.iter().find(|e| e.0.boss()).map(|e| (e.1, e.2 + 10.0));
        let boss_gone = sp.enemies.iter().any(|e| (e.0 == Kind::King && king_dead) || (e.0 == Kind::Chief && chief_dead));
        self.portal = if boss_gone { self.boss_spot } else { None };
        self.enemies = sp.enemies.iter().filter(|e| !(e.0 == Kind::King && king_dead) && !(e.0 == Kind::Chief && chief_dead)).map(|&(k, x, y)| self.scaled_enemy(k, x, y)).collect();
        // now and then a grave robber is out on the prowl
        if !to.is_dungeon() && to != AreaId::Ashford && rnd() < 0.4 {
            for _ in 0..200 {
                let (rx, ry) = (rr(80.0, self.world.aw as f32 - 80.0), rr(80.0, self.world.ah as f32 - 80.0));
                let (ax, ay) = sp.arrival(from);
                let d = ((rx - ax).powi(2) + (ry - ay).powi(2)).sqrt();
                if d > 260.0 && d < 700.0 && self.world.at(rx as i32, ry as i32) == crate::world::E && !self.world.solid(rx as i32 + 3, ry as i32) && !self.world.solid(rx as i32 - 3, ry as i32) {
                    let mut r = self.scaled_enemy(Kind::Robber, rx, ry);
                    r.elite = false;
                    r.aff = 0;
                    r.skill_t = 25 * 60;
                    self.enemies.push(r);
                    break;
                }
            }
        }
        self.chests = sp.chests.iter().map(|&(x, y, tier)| crate::props::Chest { x, y, tier, open: false, t: 0 }).collect();
        self.barrels = sp.barrels.iter().map(|&(x, y, k)| crate::barrels::Barrel::new(x, y, k, &self.world)).collect();
        self.shrines = sp.shrines.iter().map(|&(x, y, k)| crate::props::Shrine { x, y, k, used: false }).collect();
        self.projs.clear();
        self.ptcs.clear();
        self.gibs.clear();
        self.bolts.clear();
        self.lights.clear();
        self.floats.clear();
        self.pickups.clear();
        self.ground.clear();
        self.slashes.clear();
        self.novas.clear();
        self.fields.clear();
        self.meteors.clear();
        self.spikes.clear();
        self.loose_bows.clear();
        let (x, y) = match sp.waypoint {
            Some((wx, wy)) if self.via_wp => (wx + 6.0, wy + 16.0),
            _ if from == to => sp.start,
            _ => sp.arrival(from),
        };
        self.via_wp = false;
        self.braziers = sp.braziers.clone();
        self.buildings = sp.buildings.clone();
        self.lamps = sp.lamps.clone();
        for s in &self.shrines {
            let c = crate::props::SHRINES[s.k as usize].2;
            self.lamps.push((s.x, s.y - 16.0, [c[0] as f32 / 300.0, c[1] as f32 / 300.0, c[2] as f32 / 300.0]));
        }
        self.exits = sp.exits;
        self.waypoint = sp.waypoint;
        self.merchant = sp.merchant;
        self.posts = sp.posts.clone();
        self.cracks = sp.cracks.clone();
        if to.is_dungeon() {
            self.seen.remove(&to.key());
        }
        self.npcs = match sp.maren {
            Some(m) => crate::story::town_folk(m, &self.world),
            None => vec![],
        };
        self.story_arrive(to);
        self.shop_open = false;
        if to == AreaId::Ashford {
            self.restock();
        }
        self.player.x = x;
        self.player.y = y;
        self.player.burn = 0;
        self.player.wet = 0;
        self.cam = (x - self.view.0 / 2.0, y - 26.0 - self.view.1 / 2.0);
        self.fade = 40;
        self.wp_open = false;
        self.exit_lock = 30;
        self.banner = Some((to.name(), to.sub().to_string(), 200));
        self.snd(Sfx::Travel, x, y, 0.8);
        if to == AreaId::Ashford && !self.discovered.contains(&AreaId::Ashford) {
            self.discovered.push(AreaId::Ashford);
        }
        self.save();
    }

    /// back to town after dying: keep everything but a tenth of the gold
    pub fn revive(&mut self) {
        let p = &mut self.player;
        p.dead = false;
        p.hp = p.maxhp;
        p.mana = p.maxmana;
        p.gold = p.gold * 9 / 10;
        p.inv = 120;
        self.apply_gear();
        self.area = AreaId::Ashford;
        self.travel(AreaId::Ashford);
        self.banner = Some(("YOU WAKE IN ASHFORD".into(), "THE DEAD TOOK A TENTH OF YOUR GOLD".into(), 220));
    }

    /// exits, waypoints and the E key
    pub fn update_area(&mut self, inp: &Input) {
        if self.exit_lock > 0 {
            self.exit_lock -= 1;
        }
        self.prompt = None;
        if self.player.dead {
            return;
        }
        let (px, py) = (self.player.x as i32, self.player.y as i32);
        let press = inp.interact && !self.prev.interact;
        if let Some((wx, wy)) = self.waypoint {
            let d = ((self.player.x - wx).powi(2) + (self.player.y - wy).powi(2)).sqrt();
            if d < 24.0 {
                if !self.discovered.contains(&self.area) {
                    self.discovered.push(self.area);
                    self.banner = Some(("WAYPOINT FOUND".into(), "TRAVEL BETWEEN WAYPOINTS WITH E".into(), 160));
                    self.snd(Sfx::Chime, wx, wy, 1.0);
                    self.light(wx, wy, 150.0, 2.0, [0.5, 0.7, 1.0], 40);
                    self.save();
                }
                self.prompt = Some("E  WAYPOINT".into());
                if press {
                    self.snd(Sfx::Click, wx, wy, 0.8);
                    self.wp_open = true;
                    self.hold_block = true;
                }
                return;
            }
        }
        if let Some((qx, qy)) = self.portal {
            if ((self.player.x - qx).powi(2) + ((self.player.y - qy) * 1.4).powi(2)).sqrt() < 22.0 {
                self.prompt = Some("E  PORTAL - TRAVEL TO A WAYPOINT".into());
                if press {
                    self.snd(Sfx::Click, qx, qy, 0.8);
                    self.wp_open = true;
                    self.hold_block = true;
                }
                return;
            }
        }
        if let Some(m) = self.npcs.iter().find(|n| n.k == 0) {
            if ((self.player.x - m.x).powi(2) + ((self.player.y - m.y) * 1.4).powi(2)).sqrt() < 28.0 {
                self.prompt = Some("E  TALK TO ELDER MAREN".into());
                if press {
                    self.talk_to_maren();
                }
                return;
            }
        }
        if let Some((mx, my)) = self.merchant {
            if ((self.player.x - mx).powi(2) + ((self.player.y - my) * 1.4).powi(2)).sqrt() < 34.0 {
                self.prompt = Some("E  TRADE WITH THE MERCHANT".into());
                if press {
                    self.shop_open = true;
                    self.hold_block = true;
                    self.snd(Sfx::Coin, mx, my, 0.7);
                }
                return;
            }
        }
        if self.update_props(press) {
            return;
        }
        let mut go = None;
        for e in &self.exits {
            if px >= e.x0 && px <= e.x1 && py >= e.y0 && py <= e.y1 {
                self.prompt = Some(format!("E  TO {}", e.label));
                if press && self.exit_lock == 0 {
                    go = Some(e.to);
                }
            }
        }
        if let Some(to) = go {
            self.travel(to);
        }
    }

    /// the waypoint menu: click or press a number to travel
    pub fn update_waypoints(&mut self, inp: &Input) {
        let list: Vec<AreaId> = AreaId::WAYPOINTS.iter().copied().filter(|a| self.discovered.contains(a)).collect();
        self.wp_hover = crate::hud::wp_at(inp.mouse, self.view, list.len());
        let mut pick = None;
        if inp.lmb && !self.prev.lmb {
            pick = self.wp_hover;
        }
        if let Some(s) = inp.sel {
            if s < list.len() {
                pick = Some(s);
            }
        }
        if let Some(i) = pick {
            let to = list[i];
            self.wp_open = false;
            self.hold_block = true;
            if to != self.area {
                self.via_wp = true;
                self.travel(to);
            }
        }
    }

    /// the Hollow King calls his servants at half health
    pub fn boss_logic(&mut self, i: usize) {
        let e = &mut self.enemies[i];
        if !e.k.boss() || e.summoned || e.hp > e.maxhp * 0.5 {
            return;
        }
        e.summoned = true;
        let (x, y, bk) = (e.x, e.y, e.k);
        // a King who has lost his sword arm calls archers instead, in a ring well back:
        // you have to close in on him while they shoot
        let archers = bk == Kind::King && e.wk[crate::kingparts::ARM] <= 0.0;
        if archers {
            self.banner = Some(("THE ARCHERS OF THE CRYPT".into(), "THE KING CALLS HIS BOWMEN".into(), 150));
            self.snd(Sfx::KingRoar, x, y, 1.2);
            self.shake = 8.0;
            for k in 0..6 {
                // find open floor around him, closer in if the walls are near
                let spot = (0..12).find_map(|t| {
                    let a = k as f32 / 6.0 * std::f32::consts::TAU + 0.3 + t as f32 * 0.4;
                    let r = 120.0 - t as f32 * 6.0;
                    let (sx, sy) = (x + a.cos() * r, y + a.sin() * r * 0.6);
                    let open = !self.world.solid(sx as i32, sy as i32) && !self.world.solid(sx as i32 - 3, sy as i32) && !self.world.solid(sx as i32 + 3, sy as i32);
                    if open { Some((sx, sy)) } else { None }
                });
                let Some((sx, sy)) = spot else { continue };
                let mut s = self.scaled_enemy(Kind::Archer, sx, sy);
                s.aggro = true;
                s.stun = 30;
                self.enemies.push(s);
                for _ in 0..20 {
                    self.ptc(sx, sy, 0.0, rr(-1.0, 1.0), rr(-0.6, 0.6), rr(0.5, 2.0), 200, PtK::Debris, [90, 76, 64]);
                }
            }
            return;
        }
        if bk == Kind::Chief {
            self.banner = Some(("TO ME, BROTHERS".into(), "THE CHIEF CALLS HIS GANG".into(), 150));
            self.snd(Sfx::Roar, x, y, 1.2);
        } else {
            self.banner = Some(("RISE, MY SERVANTS".into(), "THE HOLLOW KING CALLS THE DEAD".into(), 150));
            self.snd(Sfx::KingRoar, x, y, 1.2);
        }
        self.shake = 8.0;
        for k in 0..5 {
            let a = k as f32 / 5.0 * std::f32::consts::TAU;
            let (sx, sy) = (x + a.cos() * 50.0, y + a.sin() * 30.0);
            if self.world.solid(sx as i32, sy as i32) {
                continue;
            }
            let kind = match (bk, k % 2) { (Kind::Chief, 0) => Kind::Bandit, (Kind::Chief, _) => Kind::Crossbow, (_, 0) => Kind::Skeleton, _ => Kind::Archer };
            let mut s = self.scaled_enemy(kind, sx, sy);
            s.aggro = true;
            s.stun = 30;
            self.enemies.push(s);
            for _ in 0..20 {
                self.ptc(sx, sy, 0.0, rr(-1.0, 1.0), rr(-0.6, 0.6), rr(0.5, 2.0), 200, PtK::Debris, [90, 76, 64]);
            }
        }
    }

    pub fn chief_slain(&mut self, x: f32, y: f32) {
        let first = !self.chief_dead;
        self.chief_dead = true;
        if first {
            self.player.sk.points += 1;
        }
        self.banner = Some(("THE BANDIT CHIEF FALLS".into(), if first { "A BONUS SKILL POINT - RAVENHOLD IS FREE".into() } else { "RAVENHOLD IS FREE".into() }, 400));
        let gold: i32 = (0..8).map(|_| 60 + ri(50) as i32).sum();
        self.boss_gold(x, y, gold);
        self.flash = 0.2;
        self.open_portal();
        self.save();
    }

    pub fn king_slain(&mut self, x: f32, y: f32) {
        self.king_dead = true;
        self.player.sk.points += 1;
        self.banner = Some(("THE HOLLOW KING FALLS".into(), "A BONUS SKILL POINT - THE CRYPT FALLS SILENT".into(), 400));
        let gold: i32 = (0..6).map(|_| 50 + ri(40) as i32).sum();
        self.boss_gold(x, y, gold);
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        if !missing.is_empty() {
            let i = missing[ri(missing.len())];
            self.player.sk.tomes[i] = true;
            self.toast = Some((format!("THE KING'S TOME: {} - LEARN IT AT LEVEL 30", NODES[i].name), 400));
        }
        self.flash = 0.25;
        self.open_portal();
        self.save();
    }

    /// a boss's gold goes straight into your purse (it can't be lost by quitting), with a
    /// shower of coins for show
    fn boss_gold(&mut self, x: f32, y: f32, gold: i32) {
        self.player.gold += gold;
        self.float(x, y - 80.0, format!("+{} GOLD", gold), [255, 214, 90], true);
        self.snd(Sfx::Coin, x, y, 1.0);
        for _ in 0..50 {
            self.ptc(x + rr(-14.0, 14.0), y, rr(10.0, 40.0), rr(-1.5, 1.5), rr(-0.8, 0.8), rr(1.0, 3.0), 70, PtK::Spark, [255, 210, 80]);
        }
    }

    /// the boss is dead: a way home opens where it stood
    pub fn open_portal(&mut self) {
        let Some((x, y)) = self.boss_spot else { return };
        self.portal = Some((x, y));
        for _ in 0..60 {
            let a = rnd() * std::f32::consts::TAU;
            self.ptc(x, y, rr(5.0, 40.0), a.cos() * rr(0.5, 2.5), a.sin() * rr(0.3, 1.5), rr(0.5, 2.0), 80, PtK::Spark, [170, 120, 255]);
        }
        self.light(x, y - 20.0, 200.0, 2.5, [0.7, 0.5, 1.0], 60);
        self.toast = Some(("A PORTAL HOME HAS OPENED".into(), 300));
    }

    // ---------------------------------------------------------------- saving
    pub fn save(&self) {
        if !self.saving {
            return;
        }
        let p = &self.player;
        let join = |v: Vec<String>| v.join(",");
        let mut lines = vec![
            "version 2".to_string(),
            format!("seed {}", self.seed),
            format!("area {}", self.area.key()),
            format!("level {}", p.sk.level),
            format!("xp {}", p.sk.xp),
            format!("points {}", p.sk.points),
            format!("gold {}", p.gold),
            format!("potions {}", p.potions),
            format!("kills {}", p.kills),
            format!("dead {}", p.dead as u8),
            format!("waypoints {}", join(self.discovered.iter().map(|a| a.key()).collect())),
            format!("king {}", self.king_dead as u8),
            format!("chief {}", self.chief_dead as u8),
            format!("act1 {}", self.act1_done as u8),
            format!("story {}", self.story),
            format!("played {}", self.played),
            format!("tips {}", self.tips),
        ];
        // skills by name, so adding or reordering skills never scrambles a save
        for (i, n) in NODES.iter().enumerate() {
            if p.sk.rank[i] > 0 || p.sk.tomes[i] {
                lines.push(format!("skill {:?} {} {}", n.sk, p.sk.rank[i], p.sk.tomes[i] as u8));
            }
        }
        // explored minimap, packed into hex, for the outdoor areas
        for (k, v) in &self.seen {
            if AreaId::from_key(k).map(|a| a.is_dungeon()).unwrap_or(true) {
                continue;
            }
            let mut hex = String::new();
            for c in v.chunks(4) {
                let n = c.iter().enumerate().fold(0u8, |a, (i, b)| a | ((*b as u8) << i));
                hex.push(char::from_digit(n as u32, 16).unwrap());
            }
            lines.push(format!("seen_{} {}", k, hex));
        }
        // "equipped" always comes first, so an empty slot loads as empty
        lines.push("equipped".into());
        for (k, it) in self.player.equip.iter().enumerate() {
            if let Some(it) = it {
                lines.push(format!("equip{} {}", k, it.encode()));
            }
        }
        for it in &self.player.bag {
            lines.push(format!("bag {}", it.encode()));
        }
        // a save without this last line was cut short and is never trusted
        lines.push("end".into());
        if let Err(e) = crate::savefile::write_safe(SAVE_FILE, &(lines.join("\n") + "\n")) {
            *self.save_fail.borrow_mut() = Some(format!("COULD NOT SAVE: {}", e).to_uppercase());
        }
    }

    /// is this a whole save? new saves end with "end"; older ones must have the key lines
    pub fn save_ok(txt: &str) -> bool {
        let has = |k: &str| txt.lines().any(|l| l.split(' ').next() == Some(k));
        if has("version") {
            txt.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.trim() == "end").unwrap_or(false)
        } else {
            ["seed", "level", "xp", "gold", "ranks", "played"].iter().all(|k| has(k))
        }
    }

    /// load the save, or its backup if the save is damaged
    pub fn load(&mut self) -> Load {
        let main = crate::savefile::read(SAVE_FILE);
        let txt = match &main {
            Some(t) if Self::save_ok(t) => t.clone(),
            None => return Load::None,
            Some(_) => match crate::savefile::read(&format!("{}.bak", SAVE_FILE)) {
                Some(b) if Self::save_ok(&b) => {
                    self.toast = Some(("YOUR SAVE WAS DAMAGED - LOADED THE BACKUP".into(), 400));
                    b
                }
                _ => return Load::Damaged,
            },
        };
        let mut was_dead = false;
        for line in txt.lines() {
            let mut it = line.splitn(2, ' ');
            let (k, v) = (it.next().unwrap_or(""), it.next().unwrap_or("").trim());
            let nums = |v: &str| v.split(',').filter_map(|x| x.parse::<i64>().ok()).collect::<Vec<_>>();
            match k {
                "seed" => self.seed = v.parse().unwrap_or(self.seed),
                "area" => self.area = AreaId::from_key(v).unwrap_or(AreaId::Ashford),
                "level" => self.player.sk.level = v.parse().unwrap_or(1),
                "xp" => self.player.sk.xp = v.parse().unwrap_or(0),
                "points" => self.player.sk.points = v.parse().unwrap_or(0),
                // the old format: ranks by position
                "ranks" => {
                    for (i, r) in nums(v).iter().enumerate().take(N) {
                        self.player.sk.rank[i] = *r as u8;
                    }
                }
                "tomes" => {
                    for (i, t) in nums(v).iter().enumerate().take(N) {
                        self.player.sk.tomes[i] = *t != 0;
                    }
                }
                // the new format: by name
                "skill" => {
                    let mut f = v.split(' ');
                    let (name, r, t) = (f.next().unwrap_or(""), f.next().and_then(|x| x.parse::<u8>().ok()).unwrap_or(0), f.next() == Some("1"));
                    if let Some(i) = NODES.iter().position(|n| format!("{:?}", n.sk) == name) {
                        self.player.sk.rank[i] = r.min(NODES[i].max);
                        self.player.sk.tomes[i] = t;
                    }
                }
                "gold" => self.player.gold = v.parse().unwrap_or(0),
                "potions" => self.player.potions = v.parse::<i32>().unwrap_or(3).clamp(0, crate::shop::MAX_POTIONS),
                "kills" => self.player.kills = v.parse().unwrap_or(0),
                "dead" => was_dead = v == "1",
                "waypoints" => self.discovered = v.split(',').filter_map(AreaId::from_key).collect(),
                "king" => self.king_dead = v == "1",
                "chief" => self.chief_dead = v == "1",
                "act1" => self.act1_done = v == "1",
                "story" => self.story = v.parse().unwrap_or(0),
                _ if k.starts_with("seen_") => {
                    let mut out = vec![];
                    for ch in v.chars() {
                        let n = ch.to_digit(16).unwrap_or(0);
                        for i in 0..4 {
                            out.push(n & (1 << i) != 0);
                        }
                    }
                    out.resize(crate::game::MM.0 * crate::game::MM.1, false);
                    self.seen.insert(k[5..].to_string(), out);
                }
                "played" => self.played = v.parse().unwrap_or(0),
                "tips" => self.tips = v.parse().unwrap_or(0),
                "equipped" => self.player.equip = [None, None, None, None],
                "equip0" | "equip1" | "equip2" | "equip3" => {
                    let s: usize = k[5..].parse().unwrap_or(0);
                    self.player.equip[s] = crate::items::Item::decode(v);
                }
                "bag" => {
                    if let Some(it) = crate::items::Item::decode(v) {
                        if self.player.bag.len() < crate::gear::BAG_SIZE {
                            self.player.bag.push(it);
                        }
                    }
                }
                _ => {}
            }
        }
        self.infer_story();
        self.apply_gear();
        self.player.hp = self.player.maxhp;
        self.player.mana = self.player.maxmana;
        if was_dead {
            Load::Dead
        } else {
            Load::Ok
        }
    }
}
