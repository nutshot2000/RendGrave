def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        if n == 0:
            assert s.count(a) >= 1, (path, a[:80])
        else:
            assert s.count(a) == n, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


DONE = True
if not DONE: edit('src/world.rs', [
    ("    #[inline]\n    pub fn inb(x: i32, y: i32) -> bool {", """    /// a world for an area of aw x ah cells; everything outside is solid rock
    pub fn sized(aw: usize, ah: usize) -> World {
        let mut w = World::new();
        w.aw = aw.min(W);
        w.ah = ah.min(H);
        for y in 0..H {
            for x in 0..W {
                if x >= w.aw || y >= w.ah {
                    let i = y * W + x;
                    w.m[i] = ROCK;
                    w.g[i] = FLOOR;
                }
            }
        }
        w
    }
    #[inline]
    pub fn inb(x: i32, y: i32) -> bool {"""),
    ("""    pub fn wake_all(&mut self) {
        for c in self.act.iter_mut() {
            *c = 30;
        }
    }""", """    pub fn wake_all(&mut self) {
        let (cw, ch) = (self.aw.div_ceil(CS).min(CW), self.ah.div_ceil(CS).min(CH));
        for cy in 0..ch {
            for cx in 0..cw {
                self.act[cy * CW + cx] = 30;
            }
        }
    }"""),
])

B = 'src/body.rs'
if not DONE: edit(B, [
    ("""    /// act I boss: a giant crowned skeleton in rusted armour
    King,
}""", """    /// act I boss: a giant crowned skeleton in rusted armour
    King,
    /// a swollen bog corpse full of acid and gas
    Bloater,
    /// a living outlaw with a sword or an axe
    Bandit,
    /// a bandit with a crossbow
    Crossbow,
    /// boss of the Ravenhold bandits
    Chief,
}"""),
    ("        matches!(self, Kind::Brute | Kind::King)\n", "        matches!(self, Kind::Brute | Kind::King | Kind::Chief)\n"),
    ("""        matches!(self, Kind::Skeleton | Kind::Archer | Kind::King)
    }
}""", """        matches!(self, Kind::Skeleton | Kind::Archer | Kind::King)
    }
    /// keeps its distance and shoots
    pub fn ranged(self) -> bool {
        matches!(self, Kind::Archer | Kind::Crossbow)
    }
    /// the shambling, grabbing dead
    pub fn shambler(self) -> bool {
        matches!(self, Kind::Zombie | Kind::Crawler | Kind::Bloater)
    }
    /// area bosses: health bar on top, boss music, no champions
    pub fn boss(self) -> bool {
        matches!(self, Kind::King | Kind::Chief)
    }
    /// living people: they bleed red and run when beaten
    pub fn bandit(self) -> bool {
        matches!(self, Kind::Bandit | Kind::Crossbow | Kind::Chief)
    }
}"""),
    ("""pub fn humanoid(orig: Kind) -> Body {
    // variants share a base body plan
""", """pub fn humanoid(orig: Kind) -> Body {
    match orig {
        Kind::Bloater => return bloater(),
        Kind::Bandit | Kind::Crossbow => return bandit(orig),
        Kind::Chief => return chief(),
        _ => {}
    }
    // variants share a base body plan
"""),
])
s = open(B, encoding='utf8').read()
BODY = r'''
/// a zombie swollen with bog water: a wide belly studded with glowing boils
fn bloater() -> Body {
    let mut b = humanoid(Kind::Zombie);
    let s = (next() & 0xffff) as u32;
    let t = b.parts[T_TORSO].clone();
    let nw = t.w * 3 / 2 + 1;
    let k = nw as f32 / t.w as f32;
    let mut px = vec![Px::default(); (nw * t.h) as usize];
    for y in 0..t.h {
        for x in 0..nw {
            let sx = ((x as f32 / k) as i32).min(t.w - 1);
            let mut p = t.px[(y * t.w + sx) as usize];
            if p.m != 0 {
                if p.m == PM_FLESH || p.m == PM_CLOTH {
                    p.m = PM_FLESH;
                    p.c = vary([128, 142, 84], x, y, s, 0.2);
                    p.im = PM_FLESH;
                    p.ic = [110, 150, 60];
                }
                if h2(x, y, s + 3) < 0.1 && y > 2 {
                    p.m = PM_GLOW;
                    p.c = [150, 235, 70];
                }
            }
            px[(y * nw + x) as usize] = p;
        }
    }
    b.parts[T_TORSO].w = nw;
    b.parts[T_TORSO].px = px;
    b.parts[T_TORSO].pivot.0 *= k;
    for p in b.parts.iter_mut() {
        if p.parent == T_TORSO as i32 {
            p.attach.0 *= k;
        }
        for q in p.px.iter_mut() {
            if q.m == PM_FLESH {
                q.c = [(q.c[0] as f32 * 1.05) as u8, (q.c[1] as f32 * 1.08).min(255.0) as u8, (q.c[2] as f32 * 0.8) as u8];
            }
        }
    }
    b.kind = Kind::Bloater;
    b.compute();
    b
}

/// an outlaw in dull mail and leather, with a sword, an axe or a crossbow
fn bandit(orig: Kind) -> Body {
    use crate::items::{Rarity, WKind};
    let mut b = humanoid(Kind::Player);
    let leather = [[112u8, 64, 40], [92, 72, 52], [124, 42, 36], [72, 78, 60]][ri(4)];
    for (pi, p) in b.parts.iter_mut().enumerate() {
        if pi == T_WEAPON {
            continue;
        }
        for q in p.px.iter_mut() {
            if q.m == 0 {
                continue;
            }
            if q.c[2] as i32 > q.c[0] as i32 + 30 {
                // the player's blue becomes worn leather
                let l = (q.c[0] as f32 + q.c[1] as f32 + q.c[2] as f32) / (52.0 + 74.0 + 160.0);
                q.c = mul(leather, l.clamp(0.6, 1.4));
                q.m = PM_CLOTH;
            } else if q.c == [200, 50, 50] {
                q.c = [44, 38, 36];
            } else if q.m == PM_METAL {
                q.c = mul(q.c, 0.62);
            }
        }
    }
    let at = b.parts[T_WEAPON].attach;
    b.parts[T_WEAPON] = if orig == Kind::Crossbow {
        let mut bow = humanoid(Kind::Archer).parts[T_WEAPON].clone();
        bow.attach = at;
        bow
    } else {
        weapon_part(if rnd() < 0.5 { WKind::Sword } else { WKind::Axe }, Rarity::Common, false, at)
    };
    b.kind = orig;
    b.compute();
    b
}

/// the bandit chief: a huge man in blackened plate and a red cloak, with a war axe
fn chief() -> Body {
    use crate::items::{Rarity, WKind};
    let mut b = humanoid(Kind::Brute);
    for (pi, p) in b.parts.iter_mut().enumerate() {
        if pi == T_WEAPON {
            continue;
        }
        for q in p.px.iter_mut() {
            match q.m {
                PM_CLOTH => q.c = mul([140, 30, 28], 0.8 + (q.c[0] as f32 / 255.0) * 0.4),
                PM_METAL => q.c = mul(q.c, 0.55),
                _ => {}
            }
        }
    }
    let at = b.parts[T_WEAPON].attach;
    b.parts[T_WEAPON] = weapon_part(WKind::Axe, Rarity::Rare, false, at);
    b.kind = Kind::Chief;
    b.compute();
    b
}
'''
if not DONE:
    open(B, 'w', encoding='utf8').write(s + BODY)

G = 'src/game.rs'
edit(G, [
    ("""        Kind::King => (1500.0, 0.55, 30.0, 30.0, 40),
        Kind::Player""", """        Kind::King => (1500.0, 0.55, 30.0, 30.0, 40),
        Kind::Bloater => (150.0, 0.34, 14.0, 12.0, 30),
        Kind::Bandit => (95.0, 0.9, 16.0, 13.0, 18),
        Kind::Crossbow => (60.0, 0.8, 175.0, 14.0, 42),
        Kind::Chief => (2400.0, 0.62, 28.0, 34.0, 34),
        Kind::Player"""),
    ("    pub king_dead: bool,\n", "    pub king_dead: bool,\n    pub chief_dead: bool,\n    pub chests: Vec<crate::props::Chest>,\n    pub shrines: Vec<crate::props::Shrine>,\n    /// shrine blessing: (kind, frames left)\n    pub buff: Option<(u8, i32)>,\n"),
    ("            king_dead: false,\n", "            king_dead: false,\n            chief_dead: false,\n            chests: vec![],\n            shrines: vec![],\n            buff: None,\n"),
    ("        self.update_area(&inp2);\n        self.update_ground();\n", "        self.update_area(&inp2);\n        self.update_ground();\n        self.tick_buffs();\n"),
    # an old banner from the single-level days fired when stepping into any house
    ("""        if indoor && !p.indoor {
            self.banner = Some(("THE SUNKEN CRYPT".into(), "THE DEAD DO NOT REST HERE".into(), 220));
        }
""", ""),
    ("""        let u = self.under(self.player.x, self.player.y);
        let p = &mut self.player;
        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32) * (1.0 + p.gear.speed / 100.0);""",
     """        let u = self.under(self.player.x, self.player.y);
        let hm = self.haste_mult();
        let p = &mut self.player;
        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32) * (1.0 + p.gear.speed / 100.0) * hm;"""),
    ("""            Kind::Skeleton | Kind::Archer | Kind::King => [200, 196, 180],
            Kind::Crawler => [90u8, 30, 26],""", """            Kind::Skeleton | Kind::Archer | Kind::King => [200, 196, 180],
            Kind::Crawler => [90u8, 30, 26],
            Kind::Bloater => [110, 150, 50],"""),
    # gold and death effects
    ("""            Kind::King => 0,
            Kind::Brute => 40 + ri(30) as i32,""", """            Kind::King | Kind::Chief => 0,
            Kind::Brute => 40 + ri(30) as i32,
            Kind::Bandit | Kind::Crossbow => 10 + ri(16) as i32,"""),
    ("""        if k == Kind::King {
            self.king_slain(x, y);
        }""", """        if k == Kind::King {
            self.king_slain(x, y);
        }
        if k == Kind::Chief {
            self.chief_slain(x, y);
        }
        if k == Kind::Bloater {
            self.bloater_burst(x, y);
        }"""),
    ("""    fn on_kill(&mut self, i: usize) {""", """    /// a bloater bursts: acid everywhere and a cloud of swamp gas
    fn bloater_burst(&mut self, x: f32, y: f32) {
        self.world.splash(x, y, ACID, 8.0, 3);
        self.world.blob(x, y - 2.0, 9.0, GAS, (self.frame as u32) & 0xffff, true);
        self.snd(Sfx::Acid, x, y, 1.0);
        self.snd(Sfx::Sever, x, y, 0.7);
        for _ in 0..50 {
            self.ptc(x, y, rr(5.0, 30.0), rr(-2.0, 2.0), rr(-1.2, 1.2), rr(0.5, 3.0), 90, PtK::Acid, [130, 220, 60]);
        }
        let d = ((self.player.x - x).powi(2) + ((self.player.y - y) * 1.4).powi(2)).sqrt();
        if d < 34.0 && self.player.inv == 0 && !self.player.dead {
            let dmg = 16.0 * (1.0 + 0.1 * (self.alvl - 1) as f32) * self.armor_mult();
            self.player.hp -= dmg;
            self.player.hurt = 1.0;
            self.player.inv = 14;
            let (px, py) = (self.player.x, self.player.y);
            self.float(px, py - 60.0, format!("-{}", dmg as i32), [150, 255, 90], false);
            self.snd(Sfx::Hurt, px, py, 1.0);
        }
    }

    fn on_kill(&mut self, i: usize) {"""),
    # the dead groan, the big ones roar, bandits shout
    ("""                let k = match e.k {
                    Kind::Zombie | Kind::Crawler => Sfx::Groan,
                    Kind::Brute => Sfx::Roar,""", """                let k = match e.k {
                    Kind::Zombie | Kind::Crawler | Kind::Bloater => Sfx::Groan,
                    Kind::Brute | Kind::Chief => Sfx::Roar,
                    Kind::Bandit | Kind::Crossbow => Sfx::HitMetal,"""),
    ("""                                Kind::Zombie | Kind::Crawler => Some(Sfx::Groan),
                                Kind::Brute => Some(Sfx::Roar),""", """                                Kind::Zombie | Kind::Crawler | Kind::Bloater => Some(Sfx::Groan),
                                Kind::Brute | Kind::Chief => Some(Sfx::Roar),"""),
    # bandits run when beaten; the chief charges
    ("""                        let e = &mut self.enemies[i];
                        if e.k != Kind::Archer && dist < e.reach""", """                        let e = &mut self.enemies[i];
                        if e.k.bandit() && e.k != Kind::Chief && e.hp < e.maxhp * 0.25 && dist < 140.0 {
                            let l = dist.max(1.0);
                            want = (-dx / l * spd * 1.15, -dy / l * spd * 1.15);
                        }
                        if e.k == Kind::Chief && dist > 50.0 && dist < 170.0 && e.hit_cd == 0 && rnd() < 0.012 {
                            let l = dist.max(1.0);
                            e.vx = dx / l * 7.0;
                            e.vy = dy / l * 7.0;
                            e.hit_cd = 50;
                            let (ex3, ey3) = (e.x, e.y);
                            self.sounds.push(Snd { k: Sfx::Roar, x: ex3, y: ey3, v: 1.0 });
                            self.shake = self.shake.max(3.0);
                        }
                        let e = &mut self.enemies[i];
                        if e.k != Kind::Archer && dist < e.reach"""),
    ("""                                self.projs.push(Proj { k: 3, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * 4.2, vy: ady / l * 4.2, vz: 0.0, life: 70 });""",
     """                                let (pk, ps) = if k2 == Kind::Crossbow { (4, 5.8) } else { (3, 4.2) };
                                self.projs.push(Proj { k: pk, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * ps, vy: ady / l * ps, vz: 0.0, life: 70 });"""),
    ("let hit_p = kind == 3 && !pl.dead", "let hit_p = (kind == 3 || kind == 4) && !pl.dead"),
    ("""                        3 => {
                            self.sounds.push(Snd { k: Sfx::ArrowHit, x: x, y: y, v: 0.7 });""", """                        3 | 4 => {
                            self.sounds.push(Snd { k: Sfx::ArrowHit, x: x, y: y, v: 0.7 });"""),
    ("let d = 10.0 * (1.0 + 0.1 * (self.alvl - 1) as f32) * rr(0.85, 1.15);", "let d = if kind == 4 { 15.0 } else { 10.0 } * (1.0 + 0.1 * (self.alvl - 1) as f32) * rr(0.85, 1.15);"),
    ("let r = if e.k == Kind::King { 7.0 } else if e.k == Kind::Brute { 5.0 } else { 3.0 };", "let r = if e.k == Kind::King { 7.0 } else if e.k.big() { 5.0 } else if e.k == Kind::Bloater { 4.0 } else { 3.0 };"),
    ("                    Kind::Brute | Kind::King => 0.12,", "                    Kind::Brute | Kind::King | Kind::Chief => 0.12,"),
    ("e.k == Kind::Archer", "e.k.ranged()", 0),
    ("e.k != Kind::Archer", "!e.k.ranged()", 0),
    ("k2 == Kind::Archer", "k2.ranged()"),
    ("matches!(e.k, Kind::Zombie | Kind::Crawler)", "e.k.shambler()", 0),
    ("matches!(k, Kind::Zombie | Kind::Crawler)", "k.shambler()"),
])

edit('src/gear.rs', [
    ("        1.0 - a / (a + 60.0)\n", "        (1.0 - a / (a + 60.0)) * self.ward_mult()\n"),
    ("((wstats(self.wkind()).frames as f32) / (1.0 + self.player.gear.swift / 100.0))", "((wstats(self.wkind()).frames as f32) / (1.0 + self.player.gear.swift / 100.0) / self.haste_mult())"),
    ("""            Kind::King => (1.0, 2, 0.35),""", """            Kind::King => (1.0, 2, 0.35),
            Kind::Chief => (1.0, 3, 0.4),
            Kind::Bandit | Kind::Crossbow => (0.14, 1, 0.02),
            Kind::Bloater => (0.12, 1, 0.0),"""),
])
edit('src/spells.rs', [
    ("""    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        let e = &mut self.enemies[i];""", """    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        let dmg = dmg * self.power_mult();
        let e = &mut self.enemies[i];"""),
    ("let (x, y) = (rr(40.0, W as f32 - 40.0), rr(40.0, H as f32 - 40.0));", "let (x, y) = (rr(40.0, self.world.aw as f32 - 40.0), rr(40.0, self.world.ah as f32 - 40.0));"),
])

A = 'src/areas.rs'
edit(A, [
    ("let to = if g.area.is_dungeon() { AreaId::Pines } else { g.area };", "let to = g.area.surface();"),
    ("if k != Kind::King && self.area != AreaId::Ashford && rnd() < 0.07 {", "if !k.boss() && self.area != AreaId::Ashford && rnd() < 0.07 {"),
    ("""            Kind::King => 600.0,
            Kind::Brute => 90.0,""", """            Kind::King => 600.0,
            Kind::Chief => 900.0,
            Kind::Brute => 90.0,
            Kind::Bloater => 40.0,
            Kind::Bandit => 34.0,
            Kind::Crossbow => 36.0,"""),
    ("""        let king_dead = self.king_dead;
        self.enemies = sp.enemies.iter().filter(|e| !(e.0 == Kind::King && king_dead)).map(|&(k, x, y)| self.scaled_enemy(k, x, y)).collect();""",
     """        let (king_dead, chief_dead) = (self.king_dead, self.chief_dead);
        self.enemies = sp.enemies.iter().filter(|e| !(e.0 == Kind::King && king_dead) && !(e.0 == Kind::Chief && chief_dead)).map(|&(k, x, y)| self.scaled_enemy(k, x, y)).collect();
        self.chests = sp.chests.iter().map(|&(x, y, tier)| crate::props::Chest { x, y, tier, open: false, t: 0 }).collect();
        self.shrines = sp.shrines.iter().map(|&(x, y, k)| crate::props::Shrine { x, y, k, used: false }).collect();"""),
    ("""        self.lamps = sp.lamps.clone();""", """        self.lamps = sp.lamps.clone();
        for s in &self.shrines {
            let c = crate::props::SHRINES[s.k as usize].2;
            self.lamps.push((s.x, s.y - 16.0, [c[0] as f32 / 300.0, c[1] as f32 / 300.0, c[2] as f32 / 300.0]));
        }"""),
    ("""        let mut go = None;
        for e in &self.exits {""", """        if self.update_props(press) {
            return;
        }
        let mut go = None;
        for e in &self.exits {"""),
    ("""        if e.k != Kind::King || e.summoned || e.hp > e.maxhp * 0.5 {
            return;
        }
        e.summoned = true;
        let (x, y) = (e.x, e.y);
        self.banner = Some(("RISE, MY SERVANTS".into(), "THE HOLLOW KING CALLS THE DEAD".into(), 150));
        self.snd(Sfx::KingRoar, x, y, 1.2);""", """        if !e.k.boss() || e.summoned || e.hp > e.maxhp * 0.5 {
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
        }"""),
    ("let mut s = self.scaled_enemy(if k % 2 == 0 { Kind::Skeleton } else { Kind::Archer }, sx, sy);",
     "let kind = match (bk, k % 2) { (Kind::Chief, 0) => Kind::Bandit, (Kind::Chief, _) => Kind::Crossbow, (_, 0) => Kind::Skeleton, _ => Kind::Archer };\n            let mut s = self.scaled_enemy(kind, sx, sy);"),
    ("""    pub fn king_slain(&mut self, x: f32, y: f32) {""", """    pub fn chief_slain(&mut self, x: f32, y: f32) {
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

    pub fn king_slain(&mut self, x: f32, y: f32) {"""),
    ("""            format!("king {}", self.king_dead as u8),""", """            format!("king {}", self.king_dead as u8),
            format!("chief {}", self.chief_dead as u8),"""),
    ("""                "king" => self.king_dead = v == "1",""", """                "king" => self.king_dead = v == "1",
                "chief" => self.chief_dead = v == "1","""),
])

M = 'src/main.rs'
edit(M, [
    ("mod areas;\n", "mod areas;\nmod mapdump;\nmod props;\n"),
    ("""                    AreaId::Pines | AreaId::Graveyard => audio::Track::Pines,
                    AreaId::Crypt(_) => audio::Track::Crypt,""", """                    AreaId::Pines | AreaId::Graveyard | AreaId::Fen | AreaId::Ravenhold => audio::Track::Pines,
                    AreaId::Crypt(_) | AreaId::Cellars(_) => audio::Track::Crypt,"""),
    ("g.enemies.iter().any(|e| e.alive && e.k == body::Kind::King && e.aggro)", "g.enemies.iter().any(|e| e.alive && e.k.boss() && e.aggro)"),
    ("""        let stops = [(AreaId::Ashford, "t1_ashford"), (AreaId::Pines, "t2_pines"), (AreaId::Graveyard, "t3_graveyard"), (AreaId::Crypt(1), "t4_crypt1"), (AreaId::Crypt(2), "t5_crypt2"), (AreaId::Crypt(3), "t6_crypt3_boss")];""",
     """        let stops = [
            (AreaId::Ashford, "t01_ashford"),
            (AreaId::Pines, "t02_pines"),
            (AreaId::Graveyard, "t03_graveyard"),
            (AreaId::Fen, "t04_fen"),
            (AreaId::Ravenhold, "t05_ravenhold"),
            (AreaId::Crypt(1), "t06_crypt1"),
            (AreaId::Crypt(3), "t07_crypt3_boss"),
            (AreaId::Cellars(1), "t08_cellars1"),
            (AreaId::Cellars(2), "t09_cellars2_boss"),
        ];"""),
    ("""                if *a == AreaId::Crypt(3) {
                    if let Some(b) = g.enemies.iter().position(|e| e.k == body::Kind::King) {""", """                if matches!(a, AreaId::Crypt(3) | AreaId::Cellars(2)) {
                    if let Some(b) = g.enemies.iter().position(|e| e.k.boss()) {"""),
    ("""    if let Some(i) = args.iter().position(|a| a == "--sfx-dump") {""", """    if let Some(i) = args.iter().position(|a| a == "--maps") {
        mapdump::dump(&args.get(i + 1).cloned().unwrap_or_else(|| "maps".into()));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--sfx-dump") {"""),
])

H_ = 'src/hud.rs'
edit(H_, [
    ("""        if e.k == Kind::King {
            continue;
        }
        let top = if e.k == Kind::Brute { 66.0 } else { 58.0 };""", """        if e.k.boss() {
            continue;
        }
        let top = if e.k.big() { 66.0 } else { 58.0 };"""),
    ("""    if let Some(b) = g.enemies.iter().find(|e| e.alive && e.k == Kind::King && e.aggro) {""", """    if let Some(b) = g.enemies.iter().find(|e| e.alive && e.k.boss() && e.aggro) {"""),
    ("""        u.text_c(fw / 2, 53, "THE HOLLOW KING", [240, 200, 120], 1);""", """        u.text_c(fw / 2, 53, if b.k == Kind::King { "THE HOLLOW KING" } else { "THE BANDIT CHIEF" }, [240, 200, 120], 1);"""),
    ("""    u.text(8, 17, &format!("AREA LEVEL {}", g.alvl), [150, 140, 150], 1, true);""", """    u.text(8, 17, &format!("AREA LEVEL {}", g.alvl), [150, 140, 150], 1, true);
    if let Some((k, left)) = g.buff {
        let (name, _, c) = crate::props::SHRINES[k as usize];
        let s = left / 60;
        let txt = format!("{}  {}:{:02}", name.trim_start_matches("SHRINE OF "), s / 60, s % 60);
        let blink = left > 600 || (left / 15) % 2 == 0;
        if blink {
            u.text(8, 27, &txt, c, 1, true);
        }
    }"""),
])
print("wire ok")
