def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub slide: (f32, f32),\n", "    pub slide: (f32, f32),\n    /// Storm Shield, Ice Armour and Last Stand timers\n    pub shield: i32,\n    pub ice: i32,\n    pub stand_cd: i32,\n"),
    ("            slide: (0.0, 0.0),\n", "            slide: (0.0, 0.0),\n            shield: 0,\n            ice: 0,\n            stand_cd: 0,\n"),
    ("    /// on its back after slipping in oil\n    pub slip: i32,\n", "    /// on its back after slipping in oil\n    pub slip: i32,\n    /// slowed by frost or poison\n    pub slow: i32,\n"),
    ("        elite: false,\n        slip: 0,\n", "        elite: false,\n        slip: 0,\n        slow: 0,\n"),
    ("let spd = e.speed * if u[3] { 0.7 } else { 1.0 } * if legs == 2", "let spd = e.speed * if e.slow > 0 { 0.55 } else { 1.0 } * if u[3] { 0.7 } else { 1.0 } * if legs == 2"),
    ("            if e.slip > 0 {\n                e.slip -= 1;\n", "            if e.slow > 0 {\n                e.slow -= 1;\n            }\n            if e.slip > 0 {\n                e.slip -= 1;\n"),
    ("                self.cast(s, mwx, mwy, inp);\n", """                self.cast(s, mwx, mwy, inp);
                let ov = self.rank(Sk::Overload);
                if ov > 0 && node(s).col == 0 && rnd() < 0.12 * ov as f32 {
                    self.player.mana += node(s).mana;
                    self.player.cd[s as usize] = 0;
                    self.banner_small("OVERLOAD");
                }
"""),
    ("p.hp = (p.hp + 45.0).min(p.maxhp);", "p.hp = (p.hp + 45.0 * (1.0 + 0.25 * p.sk.r(Sk::SecondWind) as f32)).min(p.maxhp);"),
    ("            p.hp -= 0.09;\n", "            p.hp -= 0.09 * (1.0 - 0.3 * p.sk.r(Sk::Fireproof) as f32).max(0.1);\n"),
    ("        b.compute();\n        if p.hp <= 0.0 && !p.dead {", "        b.compute();\n        self.last_stand();\n        let p = &mut self.player;\n        if p.hp <= 0.0 && !p.dead {"),
    ("if l > 0.0 && p.dodge == 0 && p.slip == 0 && rnd() < 0.005 {", "if l > 0.0 && p.dodge == 0 && p.slip == 0 && p.sk.r(Sk::Titan) == 0 && rnd() < 0.005 {"),
    ("                                self.player.hp -= dmg * rr(0.85, 1.15) * am;\n", "                                self.player.hp -= dmg * rr(0.85, 1.15) * am;\n                                retal.push(i);\n"),
    ("        let bm = self.burn_mult();\n        let (px, py) = (self.player.x, self.player.y);\n        if self.frame % 8 == 0", "        let bm = self.burn_mult();\n        let caustic = 1.0 + 0.4 * self.rank(Sk::Caustic) as f32;\n        let mut retal: Vec<usize> = vec![];\n        let (px, py) = (self.player.x, self.player.y);\n        if self.frame % 8 == 0"),
    ("                e.hp -= 0.05;\n            }\n            if u[2] {", "                e.hp -= 0.05 * caustic;\n            }\n            if u[2] {"),
    ("self.spawn_gibs(ex, ey, m, pieces, dir, 1.8, true);", "self.spawn_gibs(ex, ey, m, pieces, dir, 1.8, true);\n        self.shatter_shards(ex, ey);"),
    ("                            self.explode(x, y, z.max(2.0), r);\n", """                            self.explode(x, y, z.max(2.0), r);
                            if self.rank(Sk::Splinter) > 0 {
                                let n1 = self.lights.len();
                                for q in 0..3 {
                                    let a = q as f32 * 2.1 + rr(-0.4, 0.4);
                                    self.explode(x + a.cos() * 16.0, y + a.sin() * 10.0, 2.0, r * 0.55);
                                }
                                self.lights.truncate(n1);
                            }
"""),
    ("self.world.splash(x, y, ACID, 3.5, 3);", "self.world.splash(x, y, ACID, 3.5 + 1.2 * self.rank(Sk::AcidPools) as f32, 3);"),
    ("    pub tree_hover: Option<usize>,\n", "    pub tree_hover: Option<usize>,\n    /// which school the skill tree screen shows\n    pub tree_tab: usize,\n"),
    ("            tree_hover: None,\n", "            tree_hover: None,\n            tree_tab: 0,\n"),
    ("""            self.tree_hover = crate::hud::node_at(inp.mouse, self.view);
            if inp.lmb && !self.prev.lmb {""", """            if inp.left && !self.prev.left {
                self.tree_tab = (self.tree_tab + 5) % 6;
            }
            if inp.right && !self.prev.right {
                self.tree_tab = (self.tree_tab + 1) % 6;
            }
            if inp.lmb && !self.prev.lmb {
                if let Some(t) = crate::hud::tab_at(inp.mouse, self.view) {
                    self.tree_tab = t;
                    let (x, y) = (self.player.x, self.player.y);
                    self.snd(Sfx::Click, x, y, 0.5);
                }
            }
            self.tree_hover = crate::hud::node_at(inp.mouse, self.view, self.tree_tab);
            if inp.lmb && !self.prev.lmb {"""),
])
# retaliation after the enemy loop: the first `for (x, y, z) in flames {` after update_enemies
s = open(G, encoding='utf8').read()
a = s.index('    fn update_enemies(&mut self) {')
b = s.index('        for (x, y, z) in flames {', a)
s = s[:b] + '        for i in retal {\n            self.retaliate(i);\n        }\n' + s[b:]
open(G, 'w', encoding='utf8').write(s)

S = 'src/spells.rs'
edit(S, [
    ("""    pub fn cast(&mut self, s: Sk, mx: f32, my: f32, inp: &Input) {
        match s {""", """    pub fn cast(&mut self, s: Sk, mx: f32, my: f32, inp: &Input) {
        if self.cast_talent(s, mx, my) {
            return;
        }
        match s {"""),
    ("""        let mult = 1.0 + 0.15 * self.rank(Sk::ChainMastery) as f32;
        let e = &mut self.enemies[i];
        let wet = e.wet > 0;
        let dmg = base * mult * if wet { 1.7 } else { 1.0 } * rr(0.85, 1.15);
        e.stun = e.stun.max(26);""", """        let mult = 1.0 + 0.15 * self.rank(Sk::ChainMastery) as f32;
        let (cond, stat) = (self.rank(Sk::Conductor), self.rank(Sk::StaticField));
        let e = &mut self.enemies[i];
        let wet = e.wet > 0 || (cond > 0 && e.bleed > 0);
        let dmg = base * mult * if wet { 1.7 + 0.3 * cond as f32 } else { 1.0 } * rr(0.85, 1.15);
        e.stun = e.stun.max(26 + 13 * stat);"""),
    ("""        let dmg = dmg * self.power_mult();
        let e = &mut self.enemies[i];
        e.hp -= dmg;
        e.aggro = true;
        e.hpbar = 180;
        e.flash = e.flash.max(4);
        let (x, y) = (e.x, e.y);
        self.float(x + rr(-4.0, 4.0), y - 62.0, format!("{}", dmg.round() as i32), col, big);""", """        let mut dmg = dmg * self.power_mult();
        let (sear, deep, chill, war) = (self.rank(Sk::Searing), self.rank(Sk::DeepFreeze), self.rank(Sk::Chill), self.rank(Sk::Warlord));
        let e = &mut self.enemies[i];
        if e.burning > 0 {
            dmg *= 1.0 + 0.15 * sear as f32;
        }
        if e.frozen > 0 {
            dmg *= 1.0 + 0.2 * deep as f32;
        }
        if chill > 0 {
            e.slow = e.slow.max(60 * chill);
        }
        e.hp -= dmg;
        e.aggro = true;
        e.hpbar = 180;
        e.flash = e.flash.max(4);
        let executed = war > 0 && big && !e.k.boss() && e.hp > 0.0 && e.hp < e.maxhp * 0.3;
        if executed {
            e.hp = 0.0;
        }
        let (x, y) = (e.x, e.y);
        self.float(x + rr(-4.0, 4.0), y - 62.0, format!("{}", dmg.round() as i32), col, big);
        if executed {
            self.float(x, y - 74.0, "EXECUTED".into(), [255, 90, 70], true);
        }"""),
    ("self.fields.push(Field { k: 0, x: 0.0, y: 0.0, t: 0, max: 360, r: 150.0 });", "self.fields.push(Field { k: 0, x: 0.0, y: 0.0, t: 0, max: if self.rank(Sk::EyeOfStorm) > 0 { 720 } else { 360 }, r: 150.0 });"),
    ("                    if t % 16 == 0 {", "                    if t % (if self.rank(Sk::EyeOfStorm) > 0 { 9 } else { 16 }) == 0 {"),
    ("Sk::AcidRain => self.fields.push(Field { k: 1, x: mx, y: my, t: 0, max: 300, r: 34.0 }),", "Sk::AcidRain => {\n                let pl = self.rank(Sk::Plague) > 0;\n                self.fields.push(Field { k: 1, x: mx, y: my, t: 0, max: if pl { 420 } else { 300 }, r: if pl { 56.0 } else { 34.0 } })\n            }"),
    ("        self.world.splash(mx, my, ACID, 9.0, 16);", "        let pl = if self.rank(Sk::Plague) > 0 { 1.6 } else { 1.0 };\n        self.world.splash(mx, my, ACID, 9.0 * pl, 16);"),
    ("""                _ => {
                    for _ in 0..4 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd().sqrt() * r;""", """                3 => self.poison_field(x, y, t, r),
                _ => {
                    for _ in 0..4 {
                        let a = rnd() * std::f32::consts::TAU;
                        let d = rnd().sqrt() * r;"""),
    ("""                2 => Some((90.0, 0.7, [0.6, 0.85, 1.0])),""", """                2 => Some((90.0, 0.7, [0.6, 0.85, 1.0])),
                3 => Some((50.0, 0.3, [0.4, 0.8, 0.3])),"""),
    ("""            if self.fields[k].t >= self.fields[k].max {
                self.fields.swap_remove(k);""", """            if kind == 2 && self.fields[k].t == self.fields[k].max - 1 && self.rank(Sk::AbsoluteZero) > 0 {
                // everything in the blizzard freezes solid
                for e in self.enemies.iter_mut() {
                    if e.alive && ((e.x - x).powi(2) + ((e.y - y) / 0.7).powi(2)).sqrt() < r {
                        e.frozen = e.frozen.max(if e.k.boss() { 60 } else { 240 });
                        e.chill = 0;
                    }
                }
                self.snd(Sfx::Frost, x, y, 1.0);
                self.flash = self.flash.max(0.1);
            }
            if self.fields[k].t >= self.fields[k].max {
                self.fields.swap_remove(k);"""),
    ("        if w == 26 || w == 16 || w == 6 {", "        if w == 26 || w == 16 || w == 6 || (w > 30 && w % 10 == 6) {"),
    ("        let w = self.rank(Sk::Wisdom) as f32;\n", "        let w = self.rank(Sk::Wisdom) as f32;\n        let titan = if self.rank(Sk::Titan) > 0 { 1.2 } else { 1.0 };\n"),
    ("p.maxhp = 100.0 + 15.0 * v + p.gear.hp;", "p.maxhp = (100.0 + 15.0 * v + p.gear.hp) * titan;"),
])

edit('src/gear.rs', [
    ("        (1.0 - a / (a + 60.0)) * self.ward_mult()\n", "        (1.0 - a / (a + 60.0)) * self.ward_mult() * (1.0 - 0.04 * self.rank(Sk::Toughness) as f32) * if self.player.ice > 0 { 0.6 } else { 1.0 }\n"),
    ("/ self.haste_mult())", "/ self.haste_mult() / (1.0 + 0.06 * self.rank(Sk::Momentum) as f32))"),
    ("            if da > ws.arc && d > 10.0 {", "            if da > ws.arc * (1.0 + 0.15 * self.rank(Sk::Cleave) as f32) && d > 10.0 {"),
    ("            let power = ws.power + edge as f32 * 8.0 + g.keen + if crit { 20.0 + 6.0 * ex_r as f32 } else { 0.0 };", "            let power = (ws.power + edge as f32 * 8.0 + g.keen + if crit { 20.0 + 6.0 * ex_r as f32 } else { 0.0 }) * (1.0 + 0.2 * self.rank(Sk::Dismember) as f32);"),
])

edit('src/main.rs', [
    ("mod areas;\n", "mod areas;\nmod talents;\n"),
    ("                g.tree_hover = Some(Sk::Meteor as usize);", "                g.tree_tab = 1;\n                g.tree_hover = Some(Sk::Meteor as usize);"),
])
print("talents wired")
