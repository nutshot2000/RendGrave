# Wire the Hollow King's weak points into the game (slice step 1).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod kingparts;\n")])

G = 'src/game.rs'
edit(G, [
    ("""    pub bleed_dot: i32,
    pub bleed_amt: f32,
}

pub struct Player {""", """    pub bleed_dot: i32,
    pub bleed_amt: f32,
    /// the Hollow King's weak points (knees, sword arm, chest): what's left of each, 1 to 0
    pub wk: [f32; 3],
    pub wk_init: [i32; 3],
    /// the one-armed King's grab: >0 winding up, <0 lunging
    pub grab: i32,
}

pub struct Player {"""),
    ("        aff: 0,\n        slip: 0,\n", "        aff: 0,\n        slip: 0,\n        wk: [1.0; 3],\n        wk_init: [0; 3],\n        grab: 0,\n"),
    ("    pub sell_confirm: Option<usize>,\n", "    pub sell_confirm: Option<usize>,\n    /// height the next hurt() lands at, for the King's weak points\n    pub hit_h: Option<f32>,\n    /// (enemy, height, damage) hits on the King from explosions and acid, applied after\n    pub part_hits: Vec<(usize, f32, f32)>,\n"),
    ("            sell_confirm: None,\n", "            sell_confirm: None,\n            hit_h: None,\n            part_hits: vec![],\n"),
    ("""        self.update_projs();
        self.update_gibs();""", """        self.update_projs();
        for (i, h, d) in std::mem::take(&mut self.part_hits) {
            if i < self.enemies.len() {
                self.boss_part_hit(i, h, d);
            }
        }
        self.update_gibs();"""),
    # explosions hit at their own height
    ("""            let e = &mut self.enemies[i];
            e.hp -= dmg;
            e.aggro = true;
            e.flash = 5;""", """            self.part_hits.push((i, -z.max(4.0), dmg));
            let e = &mut self.enemies[i];
            e.hp -= dmg;
            e.aggro = true;
            e.flash = 5;"""),
    ("""            let ad = 6.0 * self.spell_power() * self.status_mult(i);
            let en = &mut self.enemies[i];""", """            let ad = 6.0 * self.spell_power() * self.status_mult(i);
            self.part_hits.push((i, -z, ad));
            let en = &mut self.enemies[i];"""),
    ("""                                let d = 16.0 * (1.0 + 0.15 * self.rank(Sk::Kindling) as f32) * self.spell_power();
                                self.hurt(i, d, [255, 170, 80], false);""", """                                let d = 16.0 * (1.0 + 0.15 * self.rank(Sk::Kindling) as f32) * self.spell_power();
                                self.hit_h = Some(-z);
                                self.hurt(i, d, [255, 170, 80], false);"""),
    # the King without his sword arm can't swing
    ("let armed = e.body.whole(T_ARMF) || e.k.boss();", "let armed = if e.k == Kind::King { e.wk[1] > 0.0 } else { e.body.whole(T_ARMF) || e.k.boss() };"),
])

S = 'src/spells.rs'
edit(S, [
    ("""        if e.frozen > 0 {
            m *= 1.0 + 0.2 * self.rank(Sk::DeepFreeze) as f32;
        }
        m
    }""", """        if e.frozen > 0 {
            m *= 1.0 + 0.2 * self.rank(Sk::DeepFreeze) as f32;
        }
        // the King's bared core
        if e.k == Kind::King && e.wk[crate::kingparts::CHEST] <= 0.0 {
            m *= 1.4;
        }
        m
    }"""),
    ("""        let dmg = dmg * self.power_mult() * self.status_mult(i);
        let (chill, war) = (self.rank(Sk::Chill), self.rank(Sk::Warlord));""", """        let dmg = dmg * self.power_mult() * self.status_mult(i);
        if let Some(h) = self.hit_h.take() {
            self.boss_part_hit(i, h, dmg);
        }
        let (chill, war) = (self.rank(Sk::Chill), self.rank(Sk::Warlord));"""),
    ("""        self.hurt(i, d, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);
    }

    /// A lightning strike""", """        self.hit_h = Some(h);
        self.hurt(i, d, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);
    }

    /// A lightning strike"""),
    # lightning is drawn to the metal breastplate
    ("""        self.hurt(i, dmg, if wet { [150, 210, 255] } else { [210, 225, 255] }, wet);""", """        self.hit_h = Some(-50.0);
        self.hurt(i, dmg, if wet { [150, 210, 255] } else { [210, 225, 255] }, wet);"""),
])
edit('src/gear.rs', [("""        self.hurt(i, dmg * mult, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);""", """        self.hit_h = Some(h);
        self.hurt(i, dmg * mult, if crit { [255, 220, 90] } else { [255, 255, 255] }, crit);""")])
edit('src/barrels.rs', [("""            let dmg = (150.0 * f + 25.0) * if boss { 0.6 } else { 1.0 } * self.status_mult(i);
            let e = &mut self.enemies[i];""", """            let dmg = (150.0 * f + 25.0) * if boss { 0.6 } else { 1.0 } * self.status_mult(i);
            // a keg goes off at knee height
            self.part_hits.push((i, -8.0, dmg));
            let e = &mut self.enemies[i];""")])

B = 'src/bosses.rs'
edit(B, [
    ("""        if e.skill_t > 0 {
            e.skill_t -= 1;
        }
        if e.slam > 0 {""", """        if e.skill_t > 0 {
            e.skill_t -= 1;
        }
        if self.king_grab(i) {
            return;
        }
        let e = &mut self.enemies[i];
        if e.slam > 0 {"""),
    ("""        let enraged = e.enraged;
        match bk {
            Kind::King if dist < 130.0 => {
                e.slam = SLAM_T;
                e.skill_t = if enraged { 220 } else { 340 };
                self.snd(Sfx::KingRoar, x, y, 1.0);
            }""", """        let enraged = e.enraged;
        let (knees, arm) = (e.wk[crate::kingparts::KNEES] > 0.0, e.wk[crate::kingparts::ARM] > 0.0);
        match bk {
            // without his sword arm he lunges to grab you; with his knees gone he can't slam
            Kind::King if !arm && dist < 125.0 && (!knees || rnd() < 0.6) => {
                e.grab = crate::kingparts::GRAB_T;
                e.skill_t = if enraged { 120 } else { 190 };
                self.float(x, y - 90.0, "!".into(), [255, 80, 60], true);
                self.snd(Sfx::KingRoar, x, y, 0.9);
            }
            Kind::King if knees && dist < 130.0 => {
                e.slam = SLAM_T;
                e.skill_t = if enraged { 220 } else { 340 };
                self.snd(Sfx::KingRoar, x, y, 1.0);
            }"""),
])

A = 'src/areas.rs'
s = open(A, encoding='utf8').read()
i0 = s.index("    pub fn boss_logic(&mut self, i: usize) {")
i1 = s.index("    pub fn chief_slain")
old = s[i0:i1]
print(old[:3000])
