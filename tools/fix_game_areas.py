def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


def block(path, start, end, new):
    s = open(path, encoding='utf8').read()
    i = s.index(start)
    j = s.index(end, i)
    s = s[:i] + new + s[j:]
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("use crate::spells::*;", "use crate::spells::*;\nuse crate::gen::*;"),
    ("    pub tree: bool,\n    pub close: bool,\n}", "    pub tree: bool,\n    pub close: bool,\n    pub interact: bool,\n}"),
    ("    pub chill: i32,\n}", "    pub chill: i32,\n    pub summoned: bool,\n}"),
    ("        chill: 0,\n    }\n}", "        chill: 0,\n        summoned: false,\n    }\n}"),
    ("    pub crypt: (i32, i32, i32, i32),\n", ""),
    ("""    pub paused: bool,
    pub hold_block: bool,
}""", """    pub paused: bool,
    pub hold_block: bool,
    pub area: AreaId,
    pub alvl: i32,
    pub exits: Vec<Exit>,
    pub waypoint: Option<(f32, f32)>,
    pub discovered: Vec<AreaId>,
    pub wp_open: bool,
    pub wp_hover: Option<usize>,
    pub king_dead: bool,
    pub fade: i32,
    pub exit_lock: i32,
    pub prompt: Option<String>,
    pub saving: bool,
}"""),
    # enemy stats for the new kinds
    ("""        Kind::Brute => (260.0, 0.5, 22.0, 24.0, 38),
        Kind::Player => (100.0, 1.0, 10.0, 0.0, 10),
    };""", """        Kind::Brute => (260.0, 0.5, 22.0, 24.0, 38),
        Kind::Crawler => (45.0, 1.0, 12.0, 7.0, 18),
        Kind::Archer => (35.0, 0.8, 150.0, 10.0, 34),
        Kind::King => (1500.0, 0.55, 30.0, 30.0, 40),
        Kind::Player => (100.0, 1.0, 10.0, 0.0, 10),
    };"""),
    # construction: a blank game that then travels somewhere
    ("""    pub fn new(seed_: u64) -> Game {
        let (world, sp) = World::generate(seed_);
        let mut body = humanoid(Kind::Player);""", """    /// test modes: straight into the Hollow Pines, no save file
    pub fn new(seed_: u64) -> Game {
        let mut g = Game::blank(seed_);
        g.travel(AreaId::Pines);
        g
    }
    pub fn blank(seed_: u64) -> Game {
        seed(seed_);
        let world = World::new();
        let mut body = humanoid(Kind::Player);"""),
    ("""            x: sp.player.0,
            y: sp.player.1,
            hp: 100.0,""", """            x: 100.0,
            y: 100.0,
            hp: 100.0,"""),
    ("""        let enemies = sp
            .enemies
            .iter()
            .map(|&(k, x, y)| enemy([Kind::Zombie, Kind::Skeleton, Kind::Brute][k as usize], x, y))
            .collect();
        Game {""", """        let enemies = vec![];
        Game {"""),
    ("            braziers: sp.braziers,\n", "            braziers: vec![],\n"),
    ("            cam: (sp.player.0 - 260.0, sp.player.1 - 150.0),\n", "            cam: (0.0, 0.0),\n"),
    ("            banner: Some((\"RENDGRAVE\".into(), \"THE HOLLOW PINES\".into(), 170)),\n", "            banner: None,\n"),
    ("            crypt: sp.crypt,\n", ""),
    ("""            paused: false,
            hold_block: false,
        }""", """            paused: false,
            hold_block: false,
            area: AreaId::Ashford,
            alvl: 1,
            exits: vec![],
            waypoint: None,
            discovered: vec![],
            wp_open: false,
            wp_hover: None,
            king_dead: false,
            fade: 0,
            exit_lock: 0,
            prompt: None,
            saving: false,
        }"""),
    # waypoint menu pauses like the skill tree
    ("""        if self.paused && !self.tree {
            self.prev = inp.clone();
            return;
        }""", """        if self.wp_open {
            if inp.close && !self.prev.close {
                self.wp_open = false;
            } else {
                self.update_waypoints(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.paused && !self.tree {
            self.prev = inp.clone();
            return;
        }"""),
    ("        self.respawn();\n", "        self.update_area(&inp2);\n"),
    # dying sends you back to town
    ("""        if inp.restart && !self.prev.restart && self.player.dead {
            let s = self.seed.wrapping_add(1);
            *self = Game::new(s);
            return;
        }""", """        if inp.restart && !self.prev.restart && self.player.dead {
            self.revive();
            return;
        }"""),
    ("""            self.banner = Some(("YOU DIED".into(), "PRESS R TO RISE AGAIN".into(), 100000));""",
     """            self.banner = Some(("YOU DIED".into(), "PRESS R TO WAKE IN ASHFORD".into(), 100000));"""),
    # gold and xp by kind and area
    ("""        let gold = match k {
            Kind::Brute => 40 + ri(30) as i32,
            Kind::Skeleton => 6 + ri(10) as i32,
            _ => 3 + ri(8) as i32,
        };""", """        let gold = match k {
            Kind::King => 0,
            Kind::Brute => 40 + ri(30) as i32,
            Kind::Skeleton | Kind::Archer => 6 + ri(10) as i32,
            _ => 3 + ri(8) as i32,
        };
        let gold = gold * (1 + self.alvl / 3);"""),
    ("""        self.gain_xp(match k {
            Kind::Brute => 90,
            Kind::Skeleton => 26,
            _ => 20,
        });""", """        let xp = self.xp_for(k);
        self.gain_xp(xp);
        if k == Kind::King {
            self.king_slain(x, y);
        }"""),
    # bones don't bleed
    ("            let kind = if k == Kind::Skeleton { PtK::Debris } else { PtK::Blood };", "            let kind = if k.bony() { PtK::Debris } else { PtK::Blood };"),
    ("            Kind::Skeleton => [200, 196, 180],\n", "            Kind::Skeleton | Kind::Archer | Kind::King => [200, 196, 180],\n            Kind::Crawler => [90u8, 30, 26],\n"),
    ("                if k != Kind::Skeleton {\n                    self.blood(ex, ey, 20.0, 30, dir, k);", "                if !k.bony() {\n                    self.blood(ex, ey, 20.0, 30, dir, k);"),
    ("                if k != Kind::Skeleton && rnd() < 0.25 {", "                if !k.bony() && rnd() < 0.25 {"),
    ("                    let c = if k == Kind::Zombie { [90, 30, 26] } else { [140, 14, 18] };", "                    let c = if matches!(k, Kind::Zombie | Kind::Crawler) { [90, 30, 26] } else { [140, 14, 18] };"),
    ("e.k == Kind::Zombie && burning > 30", "matches!(e.k, Kind::Zombie | Kind::Crawler) && burning > 30"),
    ("let big = if e.k == Kind::Brute { 8.0 } else { 0.0 };", "let big = if e.k.big() { 8.0 } else { 0.0 };"),
    ("][ri(5)] * if e.k == Kind::Brute { 1.15 } else { 1.0 }", "][ri(5)] * if e.k == Kind::King { 1.5 } else if e.k == Kind::Brute { 1.15 } else { 1.0 }"),
    ("dist < if e.k == Kind::Brute { 130.0 } else { 160.0 }", "dist < if e.k.big() { 130.0 } else if e.k == Kind::Archer { 190.0 } else { 160.0 }"),
    ("if e.burning > 25 && e.k != Kind::Skeleton { 1.5 }", "if e.burning > 25 && !e.k.bony() { 1.5 }"),
    ("            let r = if e.k == Kind::Brute { 5.0 } else { 3.0 };", "            let r = if e.k == Kind::King { 7.0 } else if e.k == Kind::Brute { 5.0 } else { 3.0 };"),
    ("                    Kind::Zombie => 0.22,\n                    Kind::Brute => 0.12,", "                    Kind::Zombie | Kind::Crawler => 0.22,\n                    Kind::Brute | Kind::King => 0.12,"),
    ("b.parts[T_HEAD].ang = tw * 2.0 + if e.k == Kind::Zombie { 0.25 } else { 0.0 };", "b.parts[T_HEAD].ang = tw * 2.0 + if matches!(e.k, Kind::Zombie | Kind::Crawler) { 0.25 } else { 0.0 };"),
    ("b.parts[T_ARMB].ang = if e.k == Kind::Zombie { -1.4 - t } else { 0.3 };", "b.parts[T_ARMB].ang = if matches!(e.k, Kind::Zombie | Kind::Crawler) { -1.4 - t } else { 0.3 };"),
    ("b.parts[T_ARMB].ang = if e.k == Kind::Zombie { 0.4 } else { 0.2 };", "b.parts[T_ARMB].ang = if matches!(e.k, Kind::Zombie | Kind::Crawler) { 0.4 } else { 0.2 };"),
    ("                        if e.k == Kind::Zombie {\n", "                        if matches!(e.k, Kind::Zombie | Kind::Crawler) {\n"),
    # AI: archers keep their distance and shoot; the king fights like a brute
    ("""                        if dist > e.reach * 0.8 {
                            let l = dist.max(1.0);
                            want = (dx / l * spd, dy / l * spd);
                        }
                        if dist < e.reach && (armed || e.k == Kind::Zombie) && e.hit_cd == 0 {""", """                        if e.k == Kind::Archer {
                            let l = dist.max(1.0);
                            let sees = self.world.los(e.x, e.y - 1.0, px, py - 1.0);
                            if dist < 70.0 {
                                want = (-dx / l * spd, -dy / l * spd);
                            } else if dist > 125.0 || !sees {
                                want = (dx / l * spd, dy / l * spd);
                            }
                            if dist < e.reach && sees && armed && e.hit_cd == 0 && dist > 40.0 {
                                e.st = St::Wind;
                                e.t = e.windup;
                            }
                        } else if dist > e.reach * 0.8 {
                            let l = dist.max(1.0);
                            want = (dx / l * spd, dy / l * spd);
                        }
                        let e = &mut self.enemies[i];
                        if e.k != Kind::Archer && dist < e.reach && (armed || matches!(e.k, Kind::Zombie | Kind::Crawler)) && e.hit_cd == 0 {"""),
    ("""                            let ok = dist < e.reach + 6.0;
                            let (dmg, k2) = (e.dmg, e.k);
                            let (ex2, ey2) = (e.x, e.y);
                            if k2 == Kind::Brute {""", """                            let ok = dist < e.reach + 6.0 && e.k != Kind::Archer;
                            let (dmg, k2) = (e.dmg, e.k);
                            let (ex2, ey2) = (e.x, e.y);
                            if k2 == Kind::Archer {
                                e.hit_cd = 50;
                                let (adx, ady) = (px - ex2, py - ey2);
                                let l = (adx * adx + ady * ady).sqrt().max(1.0);
                                let f = e.face;
                                self.projs.push(Proj { k: 3, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * 4.2, vy: ady / l * 4.2, vz: 0.0, life: 70 });
                            }
                            if k2.big() {"""),
    # arrows: hit the player, stick in walls
    ("                let hit_e = if kind == 2 { None } else { hit_e };", """                let hit_e = if kind >= 2 { None } else { hit_e };
                let pl = &self.player;
                let hit_p = kind == 3 && !pl.dead && (pl.y - y).abs() < 6.0 && (pl.x - x).abs() < 5.0 && z < 48.0;"""),
    ("                if wall || ground || hit_e.is_some() || p.life <= 0 {", "                if wall || ground || hit_e.is_some() || hit_p || self.projs[k].life <= 0 {"),
    ("""                        1 => self.acid_hit(x, y, z, hit_e),
                        _ => {""", """                        1 => self.acid_hit(x, y, z, hit_e),
                        3 => {
                            if hit_p && self.player.inv == 0 {
                                let d = 10.0 * (1.0 + 0.1 * (self.alvl - 1) as f32) * rr(0.85, 1.15);
                                self.player.hp -= d;
                                self.player.hurt = 1.0;
                                self.player.inv = 10;
                                self.shake = self.shake.max(3.0);
                                let (ppx, ppy) = (self.player.x, self.player.y);
                                self.blood(ppx, ppy, z, 8, (self.projs[k].vx.signum(), 0.0), Kind::Player);
                                self.float(ppx, ppy - 60.0, format!("-{}", d as i32), [255, 80, 80], false);
                            } else {
                                self.ptc(x, y, z, 0.0, 0.0, 0.0, 120, PtK::Debris, [120, 90, 60]);
                            }
                        }
                        _ => {"""),
    ("""            } else {
                let life = self.projs[k].life;""", """            } else if kind == 2 {
                let life = self.projs[k].life;"""),
])

# the king's summon check runs each frame he is alive
edit(G, [("""            let e = &mut self.enemies[i];
            // AI
            if e.frozen > 0 {""", """            self.boss_logic(i);
            let e = &mut self.enemies[i];
            // AI
            if e.frozen > 0 {""")])

# spells.rs: big/bony helpers
edit('src/spells.rs', [
    ("e.stun = e.stun.max(if e.k == Kind::Brute { 4 } else { 10 });", "e.stun = e.stun.max(if e.k.big() { 4 } else { 10 });"),
    ("let kb = if e.k == Kind::Brute { 0.4 } else { 1.6 };", "let kb = if e.k == Kind::King { 0.1 } else if e.k.big() { 0.4 } else { 1.6 };"),
    ("if bt > 0 && flesh > 0 && k != Kind::Skeleton {", "if bt > 0 && flesh > 0 && !k.bony() {"),
    ("if d > 36.0 + if e.k == Kind::Brute { 8.0 } else { 0.0 } {", "if d > 36.0 + if e.k.big() { 8.0 } else { 0.0 } {"),
])
# areas.rs: wake at the campfire after dying
edit('src/areas.rs', [("""        let from = self.area;
        self.area = AreaId::Pines;
        self.travel(AreaId::Ashford);
        self.area = AreaId::Ashford;
        let _ = from;""", """        self.area = AreaId::Ashford;
        self.travel(AreaId::Ashford);""")])
print("game areas ok")
