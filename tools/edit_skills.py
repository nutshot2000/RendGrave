p = 'src/game.rs'
s = open(p, encoding='utf8').read()


def R(a, b, n=1):
    global s
    assert s.count(a) == n, (a[:80], s.count(a))
    s = s.replace(a, b)


def block(start, end, new):
    global s
    i = s.index(start)
    j = s.index(end, i)
    s = s[:i] + new + s[j:]


R("use crate::world::*;", "use crate::world::*;\nuse crate::skills::*;\nuse crate::spells::*;")
R("    pub restart: bool,\n}", "    pub restart: bool,\n    pub tree: bool,\n    pub close: bool,\n}")
R("    pub hit_cd: i32,\n}", "    pub hit_cd: i32,\n    pub chill: i32,\n}")
R("        hit_cd: 0,\n", "        hit_cd: 0,\n        chill: 0,\n")
R("    pub cd: [i32; 4],", "    pub cd: [i32; N],")
R("    pub kills: i32,\n}", "    pub kills: i32,\n    pub sk: Skills,\n    pub maxhp: f32,\n    pub maxmana: f32,\n    pub whirl: i32,\n}")
R("            cd: [0; 4],", "            cd: [0; N],")
R("            kills: 0,\n", "            kills: 0,\n            sk: Skills::new(),\n            maxhp: 100.0,\n            maxmana: 100.0,\n            whirl: 0,\n")
R("    Steam,\n}", "    Steam,\n    Cloud,\n}")
R("    pub prev: Input,\n}", "    pub prev: Input,\n    pub fields: Vec<Field>,\n    pub meteors: Vec<Meteor>,\n    pub spikes: Vec<Spike>,\n    pub spawn_t: i32,\n    pub tree: bool,\n    pub tree_hover: Option<usize>,\n}")
R("            prev: Input::default(),\n", "            prev: Input::default(),\n            fields: vec![],\n            meteors: vec![],\n            spikes: vec![],\n            spawn_t: 1500,\n            tree: false,\n            tree_hover: None,\n")
for f in ["fn float(", "fn blood(", "fn shatter(", "fn add_bolt(", "fn cast_acid(", "fn cast_nova(", "fn mouse_world("]:
    if ("    pub " + f) not in s:
        R("    " + f, "    pub " + f)

R("""    pub fn update(&mut self, inp: &Input) {
        self.frame += 1;""", """    pub fn update(&mut self, inp: &Input) {
        if (inp.tree && !self.prev.tree && !self.player.dead) || (inp.close && !self.prev.close && self.tree) {
            self.tree = !self.tree;
        }
        if self.tree {
            self.tree_hover = crate::hud::node_at(inp.mouse, self.view);
            if inp.lmb && !self.prev.lmb {
                if let Some(i) = self.tree_hover {
                    if self.player.sk.learn(i) {
                        self.recalc();
                        self.flash = self.flash.max(0.05);
                    }
                }
            }
            self.prev = inp.clone();
            return;
        }
        self.frame += 1;""")
R("        self.update_misc();\n        self.prev = inp.clone();", "        self.update_misc();\n        self.update_spells();\n        self.update_whirl();\n        self.respawn();\n        self.prev = inp.clone();")
R("p.mana = (p.mana + 0.16).min(100.0);", "p.mana = (p.mana + 0.16 * (1.0 + 0.2 * p.sk.r(Sk::Wisdom) as f32)).min(p.maxmana);")
R("p.potions > 0 && p.hp < 100.0 {", "p.potions > 0 && p.hp < p.maxhp {")
R("p.hp = (p.hp + 45.0).min(100.0);", "p.hp = (p.hp + 45.0).min(p.maxhp);")
R("        let mut speed = 1.25;", "        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32);\n        if p.whirl > 0 {\n            speed *= 0.75;\n        }")
R("if inp.lmb && p.swing == 0 && p.dodge == 0 {", "if inp.lmb && p.swing == 0 && p.dodge == 0 && p.whirl == 0 {")
R("""        let p = &mut self.player;
        let s = p.sel;
        if inp.rmb && p.cd[s] == 0 && p.dodge == 0 {
            if p.mana >= SPELL_COST[s] {
                p.mana -= SPELL_COST[s];
                p.cd[s] = SPELL_CD[s];
                p.cast_t = 14;
                p.face = if ax < 0.0 { -1.0 } else { 1.0 };
                match s {
                    0 => self.cast_lightning(mwx, mwy, inp),
                    1 => self.cast_fireball(mwx, mwy),
                    2 => self.cast_acid(mwx, mwy),
                    _ => self.cast_nova(),
                }
            } else""", """        let bar = self.player.sk.bar();
        let p = &mut self.player;
        if p.sel >= bar.len() {
            p.sel = 0;
        }
        let s = bar[p.sel];
        let nd = node(s);
        let si = s as usize;
        if inp.rmb && p.cd[si] == 0 && p.dodge == 0 && p.whirl == 0 {
            if p.mana >= nd.mana {
                p.mana -= nd.mana;
                p.cd[si] = nd.cd;
                p.cast_t = 14;
                p.face = if ax < 0.0 { -1.0 } else { 1.0 };
                self.cast(s, mwx, mwy, inp);
            } else""")
R("""        if p.dodge > 0 {
            b.parts[T_TORSO].ang = (16 - p.dodge) as f32 / 16.0 * std::f32::consts::TAU;
        }""", """        if p.dodge > 0 {
            b.parts[T_TORSO].ang = (16 - p.dodge) as f32 / 16.0 * std::f32::consts::TAU;
        }
        if p.whirl > 0 {
            b.mirror = (p.whirl / 3) % 2 == 0;
            b.parts[T_ARMF].ang = -1.57;
            b.parts[T_ARMB].ang = 1.4;
            b.parts[T_TORSO].ang = 0.1;
        }""")
block("    fn sword_hit(", "    pub fn cast_lightning(", """    fn sword_hit(&mut self, aim: f32, inp: &Input) {
        let (px, py) = (self.player.x, self.player.y);
        let (mwx, mwy) = self.mouse_world(inp);
        // wooden things in the arc break
        for dy in -26i32..=26 {
            for dx in -34i32..=34 {
                let (x, y) = (px as i32 + dx, py as i32 + dy);
                let d = ((dx * dx) as f32 + (dy * dy * 2) as f32).sqrt();
                if d > 30.0 {
                    continue;
                }
                let a = (dy as f32).atan2(dx as f32);
                let da = (a - aim).sin().atan2((a - aim).cos()).abs();
                if da < 1.2 && self.world.at(x, y) == WOOD && rnd() < 0.5 {
                    self.world.set(x, y, E);
                    self.ptc(x as f32, y as f32, rr(2.0, 8.0), rr(-1.0, 1.0), rr(-1.0, 1.0), rr(0.5, 1.5), 200, PtK::Debris, [120, 86, 52]);
                }
            }
        }
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32;
        let mut any = false;
        let mut crit_any = false;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive {
                continue;
            }
            let (dx, dy) = (e.x - px, e.y - py);
            let big = if e.k == Kind::Brute { 8.0 } else { 0.0 };
            let d = (dx * dx + dy * dy * 1.6).sqrt();
            if d > 34.0 + big {
                continue;
            }
            let a = dy.atan2(dx);
            let da = (a - aim).sin().atan2((a - aim).cos()).abs();
            if da > 1.35 && d > 10.0 {
                continue;
            }
            // cut height: wherever the cursor is on the enemy, otherwise a random spot
            let (rel_mx, rel_my) = (mwx - e.x, mwy - e.y);
            let h = if rel_mx.abs() < 14.0 && rel_my < 2.0 && rel_my > -66.0 {
                rel_my
            } else {
                -[8.0, 14.0, 24.0, 30.0, 38.0][ri(5)] * if e.k == Kind::Brute { 1.15 } else { 1.0 }
            };
            let crit = rnd() < crit_ch;
            let power = 26.0 + edge as f32 * 8.0 + if crit { 20.0 + 6.0 * ex_r as f32 } else { 0.0 };
            let dmg = 14.0 * (1.0 + 0.1 * edge as f32);
            self.cut_enemy(i, h, power, dmg, px, crit);
            any = true;
            crit_any |= crit;
        }
        if any {
            self.hitstop = if crit_any { 5 } else { 3 };
            self.shake = self.shake.max(if crit_any { 4.0 } else { 2.5 });
        }
    }

""")
R("            if jumps > 4 {", "            if jumps > 4 + self.rank(Sk::ChainMastery) {")
block("        for &i in &hit {\n            let e = &mut self.enemies[i];\n            let wet", "        self.hitstop = self.hitstop.max(2);", """        for &i in &hit {
            self.zap_enemy(i, 20.0);
        }
""")
R("                let ground = z <= 0.0;", "                let ground = z <= 0.0 && kind != 2;")
R("""                if wall || ground || hit_e.is_some() || p.life <= 0 {
                    if kind == 0 {
                        self.explode(x, y, z.max(2.0), 13.0);
                    } else {
                        self.acid_hit(x, y, z, hit_e);
                    }""", """                let hit_e = if kind == 2 { None } else { hit_e };
                if wall || ground || hit_e.is_some() || p.life <= 0 {
                    match kind {
                        0 => {
                            let r = 13.0 + 2.0 * self.rank(Sk::Kindling) as f32;
                            self.explode(x, y, z.max(2.0), r);
                        }
                        1 => self.acid_hit(x, y, z, hit_e),
                        _ => {
                            for _ in 0..30 {
                                self.ptc(x, y, z, rr(-2.0, 2.0), rr(-1.5, 1.5), rr(-1.0, 2.0), 20, PtK::Spark, [190, 215, 255]);
                            }
                            self.light(x, y, 120.0, 2.0, [0.6, 0.75, 1.0], 12);
                        }
                    }""")
R("""            } else if rnd() < 0.5 {
                self.ptc(x, y, z, rr(-0.2, 0.2), rr(-0.2, 0.2), -0.2, 20, PtK::Acid, [140, 230, 70]);
            }""", """            } else if kind == 1 {
                if rnd() < 0.5 {
                    self.ptc(x, y, z, rr(-0.2, 0.2), rr(-0.2, 0.2), -0.2, 20, PtK::Acid, [140, 230, 70]);
                }
            } else {
                let life = self.projs[k].life;
                if rnd() < 0.6 {
                    let a = rnd() * 6.283;
                    self.ptc(x + a.cos() * 4.0, y, z + a.sin() * 4.0, a.cos() * 0.6, 0.0, a.sin() * 0.6, 8, PtK::Spark, [190, 215, 255]);
                }
                self.ball_zap(x, y, z, life);
            }""")
R("""        let p = &self.player;
        let (dx, dy) = (mx - p.x, my - p.y);
        let base = dy.atan2(dx);""", """        let n = 3 + self.rank(Sk::Corrosion);
        let p = &self.player;
        let (dx, dy) = (mx - p.x, my - p.y);
        let base = dy.atan2(dx);""")
R("""        for k in 0..3 {
            let a = base + (k as f32 - 1.0) * 0.1 + rr(-0.03, 0.03);""", """        for k in 0..n {
            let a = base + (k as f32 - (n - 1) as f32 / 2.0) * 0.08 + rr(-0.03, 0.03);""")
R("    fn update_misc(&mut self) {", "    fn update_misc(&mut self) {\n        let br = self.rank(Sk::Brittle);")
R("                    e.frozen = 260;", "                    e.frozen = 260 + 60 * br;")
R("            if r > 84.0 {", "            if r > 84.0 + 10.0 * br as f32 {")
R("    fn update_enemies(&mut self) {", "    fn update_enemies(&mut self) {\n        let bm = self.burn_mult();")
R("e.hp -= (burning.min(60)) as f32 * 0.004;", "e.hp -= (burning.min(60)) as f32 * 0.004 * bm;")
R("""        self.pickups.push(Pickup { k: 0, x, y, z: 10.0, vz: 1.5, amt: gold, t: 0 });
        if rnd() < if k == Kind::Brute { 1.0 } else { 0.2 } {""", """        let fortune = self.rank(Sk::Fortune) as f32;
        let gold = (gold as f32 * (1.0 + 0.25 * fortune)) as i32;
        self.pickups.push(Pickup { k: 0, x, y, z: 10.0, vz: 1.5, amt: gold, t: 0 });
        self.gain_xp(match k {
            Kind::Brute => 90,
            Kind::Skeleton => 26,
            _ => 20,
        });
        if rnd() < if k == Kind::Brute { 1.0 } else { 0.18 + 0.07 * fortune } {""")
R("""        if self.enemies.iter().all(|e| !e.alive) && !self.cleared {
            self.cleared = true;
            self.banner = Some(("ALL FOES SLAIN".into(), format!("{} GOLD  -  PRESS R FOR A NEW LAND", self.player.gold), 100000));
            self.player.dead = false;
        }
""", "")
open(p, 'w', encoding='utf8').write(s)
print("game ok")
