def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("use crate::audio::{Sfx, Snd};", "use crate::audio::{Sfx, Snd};\nuse crate::gear::GroundItem;\nuse crate::items::*;"),
    ("    pub interact: bool,\n}", "    pub interact: bool,\n    pub inv: bool,\n}"),
    ("    pub summoned: bool,\n}", "    pub summoned: bool,\n    pub elite: bool,\n    pub bleed_dot: i32,\n    pub bleed_amt: f32,\n}"),
    ("        summoned: false,\n    }", "        summoned: false,\n        elite: false,\n        bleed_dot: 0,\n        bleed_amt: 0.0,\n    }"),
    ("    pub whirl: i32,\n}", "    pub whirl: i32,\n    pub swing_max: i32,\n    pub equip: [Option<Item>; 4],\n    pub bag: Vec<Item>,\n    pub gear: GearStats,\n}"),
    ("            whirl: 0,\n", "            whirl: 0,\n            swing_max: 16,\n            equip: [Some(starter_weapon()), None, Some(starter_armor()), None],\n            bag: vec![],\n            gear: GearStats::default(),\n"),
    ("    pub toast: Option<(String, i32)>,\n}", "    pub toast: Option<(String, i32)>,\n    pub ground: Vec<GroundItem>,\n    pub inv_open: bool,\n    pub inv_hover: Option<(u8, usize)>,\n}"),
    ("            toast: None,\n        }", "            toast: None,\n            ground: vec![],\n            inv_open: false,\n            inv_hover: None,\n        }"),
    # the inventory screen pauses the game like the skill tree
    ("""        if self.wp_open {
            if inp.close && !self.prev.close {""", """        if inp.inv && !self.prev.inv && !self.player.dead && !self.wp_open {
            self.inv_open = !self.inv_open;
            self.tree = false;
            self.hold_block = true;
            let (x, y) = (self.player.x, self.player.y);
            self.snd(Sfx::Click, x, y, 0.6);
        }
        if self.inv_open {
            if inp.close && !self.prev.close {
                self.inv_open = false;
                self.hold_block = true;
            } else {
                self.update_inventory(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.wp_open {
            if inp.close && !self.prev.close {"""),
    ("        self.update_area(&inp2);\n", "        self.update_area(&inp2);\n        self.update_ground();\n"),
    # gear in movement, regen and swings
    ("p.mana = (p.mana + 0.16 * (1.0 + 0.2 * p.sk.r(Sk::Wisdom) as f32)).min(p.maxmana);",
     "p.mana = (p.mana + 0.16 * (1.0 + 0.2 * p.sk.r(Sk::Wisdom) as f32) + p.gear.regen / 60.0).min(p.maxmana);"),
    ("        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32);",
     "        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32) * (1.0 + p.gear.speed / 100.0);"),
    ("""        let p = &mut self.player;
        // sword
        if inp.lmb && p.swing == 0 && p.dodge == 0 && p.whirl == 0 {
            p.swing = 16;""", """        let sf = self.swing_frames();
        let wk = self.wkind();
        let p = &mut self.player;
        // weapon
        if inp.lmb && p.swing == 0 && p.dodge == 0 && p.whirl == 0 {
            p.swing = sf;
            p.swing_max = sf;"""),
    ("""            if p.swing == 11 && !p.swung {
                p.swung = true;
                let (sa, side) = (p.swing_a, p.swing_side);
                self.slashes.push(Slash { x: self.player.x, y: self.player.y - 22.0, a0: sa - 1.3 * side, a1: sa + 1.3 * side, t: 10 });""", """            if p.swing <= p.swing_max * 2 / 3 && !p.swung {
                p.swung = true;
                let (sa, side) = (p.swing_a, p.swing_side);
                let arc = wstats(wk).arc.min(1.3);
                self.slashes.push(Slash { x: self.player.x, y: self.player.y - 22.0, a0: sa - arc * side, a1: sa + arc * side, t: 10 });"""),
    ("""            let t = 1.0 - p.swing as f32 / 16.0;""", """            let t = 1.0 - p.swing as f32 / p.swing_max.max(1) as f32;"""),
    ("""            b.parts[T_ARMF].ang = target - 1.6 * side + 3.2 * side * e;
            b.parts[T_TORSO].ang = 0.15 * (e - 0.5) * side;""", """            if wk == WKind::Spear {
                // a straight thrust
                b.parts[T_ARMF].ang = target;
                b.parts[T_TORSO].ang = 0.12 * (1.0 - (e - 0.5).abs() * 2.0);
            } else {
                b.parts[T_ARMF].ang = target - 1.6 * side + 3.2 * side * e;
                b.parts[T_TORSO].ang = 0.15 * (e - 0.5) * side;
            }"""),
    # the sword's enemy loop becomes the per-weapon hit
    ("""        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32;
        let mut any = false;
        let mut crit_any = false;
        for i in 0..self.enemies.len() {""", """        self.weapon_hit(aim, inp);
    }

    #[allow(dead_code)]
    fn old_sword_hit(&mut self, aim: f32, inp: &Input) {
        let (px, py) = (self.player.x, self.player.y);
        let (mwx, mwy) = self.mouse_world(inp);
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32;
        let mut any = false;
        let mut crit_any = false;
        for i in 0..self.enemies.len() {"""),
    # armour soaks hits
    ("                                self.player.hp -= d;", "                                let d = d * self.armor_mult();\n                                self.player.hp -= d;"),
    ("                                self.player.hp -= dmg * rr(0.85, 1.15);", "                                let am = self.armor_mult();\n                                self.player.hp -= dmg * rr(0.85, 1.15) * am;"),
    # loot
    ("""        let xp = self.xp_for(k);
        self.gain_xp(xp);""", """        let elite = self.enemies[i].elite;
        let xp = self.xp_for(k) * if elite { 3 } else { 1 };
        self.gain_xp(xp);
        self.drop_loot(x, y, k, elite);"""),
    # Winter's Grudge: shattering bursts in frost
    ("""    pub fn shatter(&mut self, i: usize) {
        self.sounds.push(Snd { k: Sfx::Shatter, x: self.enemies[i].x, y: self.enemies[i].y, v: 1.0 });""", """    pub fn shatter(&mut self, i: usize) {
        self.sounds.push(Snd { k: Sfx::Shatter, x: self.enemies[i].x, y: self.enemies[i].y, v: 1.0 });
        if self.has_legend(Legend::WintersGrudge) {
            let (x, y) = (self.enemies[i].x, self.enemies[i].y);
            self.novas.push(Nova { x, y, r: 4.0, t: 0 });
        }"""),
    # Gravecleaver: severed limbs explode
    ("""                self.spawn_gibs(ex, ey, m, ds, dir, 1.6, false);
                let txt = "SEVERED".to_string();""", """                self.spawn_gibs(ex, ey, m, ds, dir, 1.6, false);
                if big_piece && self.has_legend(Legend::Gravecleaver) {
                    self.explode(ex + rr(-6.0, 6.0), ey + 1.0, 14.0, 8.0);
                }
                let txt = "SEVERED".to_string();"""),
    # dagger bleed
    ("            if e.bleed > 0 {", """            if e.bleed_dot > 0 {
                e.bleed_dot -= 1;
                e.hp -= e.bleed_amt;
                e.bleed = e.bleed.max(4);
            }
            if e.bleed > 0 {"""),
])

edit('src/spells.rs', [
    ("""        let p = &mut self.player;
        p.maxhp = 100.0 + 15.0 * v;
        p.maxmana = 60.0 + 15.0 * w;""", """        let p = &mut self.player;
        p.maxhp = 100.0 + 15.0 * v + p.gear.hp;
        p.maxmana = 60.0 + 15.0 * w + p.gear.mana;"""),
])

A = 'src/areas.rs'
edit(A, [
    ("""        e.dmg *= 1.0 + 0.1 * l;
        e
    }""", """        e.dmg *= 1.0 + 0.1 * l;
        // now and then a champion: tougher, glowing, better loot
        if k != Kind::King && self.area != AreaId::Ashford && rnd() < 0.07 {
            e.elite = true;
            e.hp *= 2.6;
            e.maxhp = e.hp;
            e.dmg *= 1.3;
            e.speed *= 1.1;
        }
        e
    }"""),
    ("""            format!("king {}", self.king_dead as u8),
        ];""", """            format!("king {}", self.king_dead as u8),
        ];
        let mut lines: Vec<String> = lines.to_vec();
        for (k, it) in self.player.equip.iter().enumerate() {
            if let Some(it) = it {
                lines.push(format!("equip{} {}", k, it.encode()));
            }
        }
        for it in &self.player.bag {
            lines.push(format!("bag {}", it.encode()));
        }"""),
    ("""                "king" => self.king_dead = v == "1",""", """                "king" => self.king_dead = v == "1",
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
                }"""),
    ("""        let Ok(txt) = std::fs::read_to_string(SAVE_FILE) else { return false };""", """        let Ok(txt) = std::fs::read_to_string(SAVE_FILE) else { return false };
        let mut gear_loaded = false;"""),
    ("""        self.recalc();
        self.player.hp = self.player.maxhp;""", """        self.apply_gear();
        self.player.hp = self.player.maxhp;"""),
    # apply gear on a fresh start and after reviving
    ("""        } else {
            g.travel(AreaId::Ashford);""", """        } else {
            g.apply_gear();
            g.travel(AreaId::Ashford);"""),
    ("""        let p = &mut self.player;
        p.body = humanoid(Kind::Player);
        p.body.compute();
        p.dead = false;""", """        let p = &mut self.player;
        p.dead = false;"""),
    ("""        self.area = AreaId::Ashford;
        self.travel(AreaId::Ashford);""", """        self.apply_gear();
        self.area = AreaId::Ashford;
        self.travel(AreaId::Ashford);"""),
    ("        self.pickups.clear();\n", "        self.pickups.clear();\n        self.ground.clear();\n"),
])

M = 'src/main.rs'
edit(M, [
    ("mod areas;\n", "mod areas;\nmod gear;\nmod items;\n"),
    ("                        KeyCode::KeyE => i.interact = down,", "                        KeyCode::KeyE => i.interact = down,\n                        KeyCode::KeyI => i.inv = down,"),
    ("            let duck = if g.paused || g.tree || g.wp_open { 0.45 } else { 1.0 };", "            let duck = if g.paused || g.tree || g.wp_open || g.inv_open { 0.45 } else { 1.0 };"),
])
print("gear wiring ok")
