# Hook sound effects into the game.


def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


def S(k, x, y, v="1.0"):
    return f"self.sounds.push(Snd {{ k: Sfx::{k}, x: {x}, y: {y}, v: {v} }});"


G = 'src/game.rs'
edit(G, [
    ("use crate::areagen::*;", "use crate::areagen::*;\nuse crate::audio::{Sfx, Snd};"),
    ("    pub lamps: Vec<(f32, f32, [f32; 3])>,\n}", "    pub lamps: Vec<(f32, f32, [f32; 3])>,\n    pub sounds: Vec<Snd>,\n    pub toast: Option<(String, i32)>,\n}"),
    ("            lamps: vec![],\n        }", "            lamps: vec![],\n            sounds: vec![],\n            toast: None,\n        }"),
    ("""    pub fn light(&mut self, x: f32, y: f32, r: f32, i: f32, c: [f32; 3], t: i32) {""", """    pub fn snd(&mut self, k: Sfx, x: f32, y: f32, v: f32) {
        self.sounds.push(Snd { k, x, y, v });
    }
    pub fn light(&mut self, x: f32, y: f32, r: f32, i: f32, c: [f32; 3], t: i32) {"""),
    # skill learned
    ("""                    if self.player.sk.learn(i) {""", """                    if self.player.sk.learn(i) {
                        let (x, y) = (self.player.x, self.player.y);
                        self.snd(Sfx::Chime, x, y, 0.7);"""),
    # dodge
    ("""            p.dodge = 16;
            p.dodge_cd = 38;""", """            p.dodge = 16;
            p.dodge_cd = 38;
            """ + S("Dodge", "p.x", "p.y", "0.8")),
    # footsteps
    ("""        if p.moving {
            p.phase += 0.2;
        }""", """        if p.moving {
            let before = (p.phase / std::f32::consts::PI) as i32;
            p.phase += 0.2;
            if (p.phase / std::f32::consts::PI) as i32 != before && p.dodge == 0 {
                """ + S("Step", "p.x", "p.y", "0.5") + """
            }
        }"""),
    ("            p.potions -= 1;", "            p.potions -= 1;\n            " + S("Potion", "p.x", "p.y")),
    ("            p.swing = 16;", "            p.swing = 16;\n            " + S("Swing", "p.x", "p.y", "0.8")),
    ("            p.dead = true;", "            p.dead = true;\n            " + S("Death", "p.x", "p.y")),
    # breaking wood with the sword
    ("""                    self.world.set(x, y, E);
                    self.ptc(x as f32, y as f32, rr(2.0, 8.0), rr(-1.0, 1.0), rr(-1.0, 1.0), rr(0.5, 1.5), 200, PtK::Debris, [120, 86, 52]);""",
     """                    self.world.set(x, y, E);
                    self.ptc(x as f32, y as f32, rr(2.0, 8.0), rr(-1.0, 1.0), rr(-1.0, 1.0), rr(0.5, 1.5), 200, PtK::Debris, [120, 86, 52]);
                    """ + S("Wood", "x as f32", "y as f32", "0.6")),
    # spells
    ("""    pub fn cast_lightning(&mut self, mx: f32, my: f32, _inp: &Input) {""", """    pub fn cast_lightning(&mut self, mx: f32, my: f32, _inp: &Input) {
        """ + S("Zap", "self.player.x", "self.player.y", "0.7")),
    ("""    pub fn cast_fireball(&mut self, mx: f32, my: f32) {""", """    pub fn cast_fireball(&mut self, mx: f32, my: f32) {
        """ + S("Fireball", "self.player.x", "self.player.y", "0.8")),
    ("""    pub fn cast_acid(&mut self, mx: f32, my: f32) {""", """    pub fn cast_acid(&mut self, mx: f32, my: f32) {
        """ + S("Swing", "self.player.x", "self.player.y", "0.5")),
    ("""    pub fn cast_nova(&mut self) {""", """    pub fn cast_nova(&mut self) {
        """ + S("Frost", "self.player.x", "self.player.y", "1.0")),
    ("""    pub fn explode(&mut self, x: f32, y: f32, z: f32, r: f32) {""", """    pub fn explode(&mut self, x: f32, y: f32, z: f32, r: f32) {
        """ + S("Explosion", "x", "y", "(r / 16.0).min(1.3)")),
    ("""    fn acid_hit(&mut self, x: f32, y: f32, z: f32, e: Option<usize>) {""", """    fn acid_hit(&mut self, x: f32, y: f32, z: f32, e: Option<usize>) {
        """ + S("Acid", "x", "y", "0.7")),
    ("""    pub fn shatter(&mut self, i: usize) {""", """    pub fn shatter(&mut self, i: usize) {
        """ + S("Shatter", "self.enemies[i].x", "self.enemies[i].y")),
    # limbs coming off
    ("""            if had {
                e.bleed = 160;""", """            if had {
                if big_piece {
                    """ + S("Sever", "ex", "ey") + """
                }
                e.bleed = 160;"""),
    # enemies notice you
    ("""            if !e.aggro && dist < if e.k.big() { 130.0 } else if e.k == Kind::Archer { 190.0 } else { 160.0 } && self.world.los(e.x, e.y - 1.0, px, py - 1.0) {
                e.aggro = true;
            }""", """            if !e.aggro && dist < if e.k.big() { 130.0 } else if e.k == Kind::Archer { 190.0 } else { 160.0 } && self.world.los(e.x, e.y - 1.0, px, py - 1.0) {
                e.aggro = true;
                let k = match e.k {
                    Kind::Zombie | Kind::Crawler => Sfx::Groan,
                    Kind::Brute => Sfx::Roar,
                    Kind::King => Sfx::KingRoar,
                    _ => Sfx::Rattle,
                };
                let (ex3, ey3) = (e.x, e.y);
                self.sounds.push(Snd { k, x: ex3, y: ey3, v: 0.9 });
            }"""),
    # wind-ups
    ("""                        if e.k != Kind::Archer && dist < e.reach && (armed || matches!(e.k, Kind::Zombie | Kind::Crawler)) && e.hit_cd == 0 {
                            e.st = St::Wind;
                            e.t = e.windup;""", """                        if e.k != Kind::Archer && dist < e.reach && (armed || matches!(e.k, Kind::Zombie | Kind::Crawler)) && e.hit_cd == 0 {
                            e.st = St::Wind;
                            e.t = e.windup;
                            let k = match e.k {
                                Kind::Zombie | Kind::Crawler => Some(Sfx::Groan),
                                Kind::Brute => Some(Sfx::Roar),
                                Kind::King if rnd() < 0.3 => Some(Sfx::KingRoar),
                                _ => None,
                            };
                            if let Some(k) = k {
                                let (ex3, ey3) = (e.x, e.y);
                                self.sounds.push(Snd { k, x: ex3, y: ey3, v: 0.7 });
                            }"""),
    ("""                                self.projs.push(Proj { k: 3, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * 4.2, vy: ady / l * 4.2, vz: 0.0, life: 70 });""",
     """                                self.projs.push(Proj { k: 3, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * 4.2, vy: ady / l * 4.2, vz: 0.0, life: 70 });
                                """ + S("Bow", "ex2", "ey2", "0.8")),
    ("""                            if hit_p && self.player.inv == 0 {""", """                            """ + S("ArrowHit", "x", "y", "0.7") + """
                            if hit_p && self.player.inv == 0 {
                                """ + S("Hurt", "self.player.x", "self.player.y")),
    ("""                            if ok && self.player.inv == 0 && !self.player.dead {""", """                            if k2.big() {
                                """ + S("Crumble", "ex2", "ey2", "0.8") + """
                            }
                            if ok && self.player.inv == 0 && !self.player.dead {
                                """ + S("Hurt", "self.player.x", "self.player.y") + """
                                """ + S("HitFlesh", "self.player.x", "self.player.y", "0.8")),
    # pickups
    ("                self.player.gold += p.amt;", "                self.player.gold += p.amt;\n                " + S("Coin", "px", "py", "0.8")),
    ("                self.player.potions += 1;", "                self.player.potions += 1;\n                " + S("Coin", "px", "py", "0.6")),
    ("                self.player.sk.tomes[i] = true;", "                self.player.sk.tomes[i] = true;\n                " + S("Chime", "px", "py")),
    # toast timer
    ("""    fn update_misc(&mut self) {""", """    fn update_misc(&mut self) {
        if let Some(t) = &mut self.toast {
            t.1 -= 1;
            if t.1 <= 0 {
                self.toast = None;
            }
        }"""),
])

SP = 'src/spells.rs'
edit(SP, [
    ("use crate::world::*;", "use crate::world::*;\nuse crate::audio::Sfx;"),
    # the sound of a blade landing depends on what it hit
    ("""        let metal = cut.iter().filter(|c| c.2 == PM_METAL).count();
        let flesh = cut.len() - metal;
        let mut d = if flesh > 0 { dmg + flesh as f32 * 0.6 } else { dmg * 0.2 };""", """        let metal = cut.iter().filter(|c| c.2 == PM_METAL).count();
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
        self.sounds.push(crate::audio::Snd { k, x: sx, y: sy, v: 1.0 });"""),
    ("""    pub fn zap_enemy(&mut self, i: usize, base: f32) {""", """    pub fn zap_enemy(&mut self, i: usize, base: f32) {
        let (zx, zy) = (self.enemies[i].x, self.enemies[i].y);
        self.snd(Sfx::Zap, zx, zy, 0.6);"""),
    ("""                        self.add_bolt((tx + rr(-30.0, 30.0), ty - 200.0), (tx, ty));""", """                        self.add_bolt((tx + rr(-30.0, 30.0), ty - 200.0), (tx, ty));
                        self.snd(Sfx::Thunder, tx, ty, 0.9);"""),
    ("""            Sk::Thunderstorm => {""", """            Sk::Thunderstorm => {
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Thunder, x, y, 1.0);"""),
    ("""            Sk::AcidRain => self.fields.push(""", """            Sk::AcidRain => self.fields.push("""),
    ("""            Sk::Blizzard => self.fields.push(Field { k: 2, x: mx, y: my, t: 0, max: 360, r: 46.0 }),""", """            Sk::Blizzard => {
                self.snd(Sfx::Frost, mx, my, 1.0);
                self.fields.push(Field { k: 2, x: mx, y: my, t: 0, max: 360, r: 46.0 })
            }"""),
    ("""            Sk::Whirlwind => {
                self.player.whirl = 30;""", """            Sk::Whirlwind => {
                self.player.whirl = 30;
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Whirl, x, y, 1.0);"""),
    ("""        let new: Vec<(f32, f32)> = self.spikes.iter().filter(|s| s.t == 1).map(|s| (s.x, s.y)).collect();
        for (x, y) in new {""", """        let new: Vec<(f32, f32)> = self.spikes.iter().filter(|s| s.t == 1).map(|s| (s.x, s.y)).collect();
        for (x, y) in new {
            self.snd(Sfx::Shatter, x, y, 0.4);"""),
    ("""                self.shake = 12.0;
                self.hitstop = 6;""", """                self.snd(Sfx::Crumble, x, y, 1.0);
                self.shake = 12.0;
                self.hitstop = 6;"""),
    ("""        if ups > 0 {
            let lvl = self.player.sk.level;""", """        if ups > 0 {
            let (lx, ly) = (self.player.x, self.player.y);
            self.snd(Sfx::LevelUp, lx, ly, 1.0);
            let lvl = self.player.sk.level;"""),
])

A = 'src/areas.rs'
edit(A, [
    ("use crate::skills::*;", "use crate::skills::*;\nuse crate::audio::Sfx;"),
    ("""        self.banner = Some((to.name(), to.sub().to_string(), 200));""", """        self.banner = Some((to.name(), to.sub().to_string(), 200));
        self.snd(Sfx::Travel, x, y, 0.8);"""),
    ("""                    self.banner = Some(("WAYPOINT FOUND".into(), "TRAVEL BETWEEN WAYPOINTS WITH E".into(), 160));""", """                    self.banner = Some(("WAYPOINT FOUND".into(), "TRAVEL BETWEEN WAYPOINTS WITH E".into(), 160));
                    self.snd(Sfx::Chime, wx, wy, 1.0);"""),
    ("""                if press {
                    self.wp_open = true;""", """                if press {
                    self.snd(Sfx::Click, wx, wy, 0.8);
                    self.wp_open = true;"""),
    ("""        self.banner = Some(("RISE, MY SERVANTS".into(), "THE HOLLOW KING CALLS THE DEAD".into(), 150));""", """        self.banner = Some(("RISE, MY SERVANTS".into(), "THE HOLLOW KING CALLS THE DEAD".into(), 150));
        self.snd(Sfx::KingRoar, x, y, 1.2);"""),
])
print("hooks ok")
