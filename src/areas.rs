// Moving between areas: exits and stairs, waypoints, dying back to town,
// enemy scaling by area level, the Hollow King, and the save file.
use crate::body::*;
use crate::game::*;
use crate::areagen::*;
use crate::rng::*;
use crate::skills::*;
use crate::audio::Sfx;

pub const SAVE_FILE: &str = "rendgrave_save.txt";

impl Game {
    /// a fresh game: new seed, start in Ashford (or continue from the save file)
    pub fn start(load: bool) -> Game {
        let seed_ = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        let mut g = Game::blank(seed_);
        g.saving = true;
        if load && g.load() {
            let to = g.area.surface();
            g.area = AreaId::Ashford;
            g.travel(to);
            g.banner = Some(("WELCOME BACK".into(), to.name(), 200));
        } else {
            g.apply_gear();
            g.travel(AreaId::Ashford);
            g.banner = Some(("RENDGRAVE".into(), "ASHFORD - THE LAST SAFE HEARTH".into(), 260));
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
        // now and then a champion: tougher, glowing, better loot
        if !k.boss() && self.area != AreaId::Ashford && rnd() < 0.07 {
            e.elite = true;
            e.hp *= 2.6;
            e.maxhp = e.hp;
            e.dmg *= 1.3;
            e.speed *= 1.1;
        }
        e
    }

    pub fn xp_for(&self, k: Kind) -> i32 {
        let base = match k {
            Kind::King => 600.0,
            Kind::Chief => 900.0,
            Kind::Brute => 90.0,
            Kind::Bloater => 40.0,
            Kind::Bandit => 34.0,
            Kind::Crossbow => 36.0,
            Kind::Archer => 30.0,
            Kind::Skeleton => 26.0,
            Kind::Crawler => 16.0,
            _ => 20.0,
        };
        let area = 1.0 + 0.25 * (self.alvl - 1) as f32;
        let gap = self.player.sk.level - self.alvl;
        let penalty = if gap > 8 { 0.15 } else if gap > 4 { 0.5 } else { 1.0 };
        (base * area * penalty) as i32
    }

    /// leave the current area and arrive in another
    pub fn travel(&mut self, to: AreaId) {
        let from = self.area;
        let seed_ = if to.is_dungeon() { self.seed ^ self.frame.wrapping_mul(0x9E37_79B9) ^ next() } else { self.seed };
        let (world, sp) = generate(to, seed_);
        self.world = world;
        self.area = to;
        self.alvl = to.level();
        let (king_dead, chief_dead) = (self.king_dead, self.chief_dead);
        self.enemies = sp.enemies.iter().filter(|e| !(e.0 == Kind::King && king_dead) && !(e.0 == Kind::Chief && chief_dead)).map(|&(k, x, y)| self.scaled_enemy(k, x, y)).collect();
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
        let (x, y) = if from == to { sp.start } else { sp.arrival(from) };
        self.braziers = sp.braziers.clone();
        self.buildings = sp.buildings.clone();
        self.lamps = sp.lamps.clone();
        for s in &self.shrines {
            let c = crate::props::SHRINES[s.k as usize].2;
            self.lamps.push((s.x, s.y - 16.0, [c[0] as f32 / 300.0, c[1] as f32 / 300.0, c[2] as f32 / 300.0]));
        }
        self.exits = sp.exits;
        self.waypoint = sp.waypoint;
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
        for k in 0..8 {
            self.pickups.push(Pickup { k: 0, x: x + rr(-16.0, 16.0), y: y + rr(-8.0, 8.0), z: 14.0, vz: rr(1.5, 3.0), amt: 60 + ri(50) as i32, t: k * 5 });
        }
        self.flash = 0.2;
        self.save();
    }

    pub fn king_slain(&mut self, x: f32, y: f32) {
        self.king_dead = true;
        self.player.sk.points += 1;
        self.banner = Some(("THE HOLLOW KING FALLS".into(), "A BONUS SKILL POINT - THE CRYPT FALLS SILENT".into(), 400));
        for k in 0..6 {
            self.pickups.push(Pickup { k: 0, x: x + rr(-14.0, 14.0), y: y + rr(-8.0, 8.0), z: 14.0, vz: rr(1.5, 3.0), amt: 50 + ri(40) as i32, t: k * 6 });
        }
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        if !missing.is_empty() {
            let i = missing[ri(missing.len())];
            self.pickups.push(Pickup { k: 2, x, y: y + 6.0, z: 16.0, vz: 3.0, amt: i as i32, t: 0 });
        }
        self.flash = 0.25;
        self.save();
    }

    // ---------------------------------------------------------------- saving
    pub fn save(&self) {
        if !self.saving {
            return;
        }
        let p = &self.player;
        let join = |v: Vec<String>| v.join(",");
        let lines = [
            format!("seed {}", self.seed),
            format!("area {}", self.area.key()),
            format!("level {}", p.sk.level),
            format!("xp {}", p.sk.xp),
            format!("points {}", p.sk.points),
            format!("ranks {}", join(p.sk.rank.iter().map(|r| r.to_string()).collect())),
            format!("tomes {}", join(p.sk.tomes.iter().map(|t| (*t as u8).to_string()).collect())),
            format!("gold {}", p.gold),
            format!("potions {}", p.potions),
            format!("waypoints {}", join(self.discovered.iter().map(|a| a.key()).collect())),
            format!("king {}", self.king_dead as u8),
            format!("chief {}", self.chief_dead as u8),
        ];
        let mut lines: Vec<String> = lines.to_vec();
        for (k, it) in self.player.equip.iter().enumerate() {
            if let Some(it) = it {
                lines.push(format!("equip{} {}", k, it.encode()));
            }
        }
        for it in &self.player.bag {
            lines.push(format!("bag {}", it.encode()));
        }
        let _ = std::fs::write(SAVE_FILE, lines.join("\n"));
    }
    fn load(&mut self) -> bool {
        let Ok(txt) = std::fs::read_to_string(SAVE_FILE) else { return false };
        let mut gear_loaded = false;
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
                "gold" => self.player.gold = v.parse().unwrap_or(0),
                "potions" => self.player.potions = v.parse().unwrap_or(3),
                "waypoints" => self.discovered = v.split(',').filter_map(AreaId::from_key).collect(),
                "king" => self.king_dead = v == "1",
                "chief" => self.chief_dead = v == "1",
                "equip0" | "equip1" | "equip2" | "equip3" => {
                    if !gear_loaded {
                        self.player.equip = [None, None, None, None];
                        gear_loaded = true;
                    }
                    let s: usize = k[5..].parse().unwrap_or(0);
                    self.player.equip[s] = crate::items::Item::decode(v);
                }
                "bag" => {
                    if let Some(it) = crate::items::Item::decode(v) {
                        self.player.bag.push(it);
                    }
                }
                _ => {}
            }
        }
        self.apply_gear();
        self.player.hp = self.player.maxhp;
        self.player.mana = self.player.maxmana;
        true
    }
}
