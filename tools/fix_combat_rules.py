# One set of damage rules (review items 3, 7, 8, 13).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


S = 'src/spells.rs'
edit(S, [
    # Searing and Deep Freeze in one place, used by every damage source
    ("""    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        let mut dmg = dmg * self.power_mult();
        let (sear, deep, chill, war) = (self.rank(Sk::Searing), self.rank(Sk::DeepFreeze), self.rank(Sk::Chill), self.rank(Sk::Warlord));
        let e = &mut self.enemies[i];
        if e.burning > 0 {
            dmg *= 1.0 + 0.15 * sear as f32;
        }
        if e.frozen > 0 {
            dmg *= 1.0 + 0.2 * deep as f32;
        }
        if chill > 0 {""", """    /// Searing and Deep Freeze: every source of damage goes through this
    pub fn status_mult(&self, i: usize) -> f32 {
        let e = &self.enemies[i];
        let mut m = 1.0;
        if e.burning > 0 {
            m *= 1.0 + 0.15 * self.rank(Sk::Searing) as f32;
        }
        if e.frozen > 0 {
            m *= 1.0 + 0.2 * self.rank(Sk::DeepFreeze) as f32;
        }
        m
    }

    pub fn hurt(&mut self, i: usize, dmg: f32, col: [u8; 3], big: bool) {
        let dmg = dmg * self.power_mult() * self.status_mult(i);
        let (chill, war) = (self.rank(Sk::Chill), self.rank(Sk::Warlord));
        let e = &mut self.enemies[i];
        if chill > 0 {"""),
    # a shatter that doesn't happen lets the blow land normally
    ("""    pub fn cut_enemy(&mut self, i: usize, h: f32, power: f32, dmg: f32, src_x: f32, crit: bool) {
        if self.enemies[i].frozen > 0 {
            self.shatter(i);
            return;
        }""", """    pub fn cut_enemy(&mut self, i: usize, h: f32, power: f32, dmg: f32, src_x: f32, crit: bool) {
        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }"""),
    # Whirlwind and Blade Fury swing your actual weapon
    ("""            let edge = self.rank(Sk::Edge);
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if !e.alive {
                    continue;
                }
                let d = ((e.x - px).powi(2) + ((e.y - py) * 1.3).powi(2)).sqrt();
                if d > 36.0 + if e.k.big() { 8.0 } else { 0.0 } {
                    continue;
                }
                let h = -[10.0, 18.0, 26.0, 34.0][ri(4)];
                let crit = rnd() < 0.08 * self.rank(Sk::Executioner) as f32 + 0.1;
                let pw = 20.0 + edge as f32 * 8.0 + if crit { 20.0 } else { 0.0 };
                self.cut_enemy(i, h, pw, 9.0 * (1.0 + 0.1 * edge as f32), px, crit);
            }""", """            let edge = self.rank(Sk::Edge);
            let g = self.player.gear;
            let ws = crate::items::wstats(self.wkind());
            // each spin is three-quarters of a normal swing with your weapon
            let dmg = g.weapon_dmg * ws.dmg_mult * (1.0 + g.dmg_pct / 100.0) * (1.0 + 0.1 * edge as f32) * 0.75;
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if !e.alive {
                    continue;
                }
                let d = ((e.x - px).powi(2) + ((e.y - py) * 1.3).powi(2)).sqrt();
                if d > 36.0 + if e.k.big() { 8.0 } else { 0.0 } {
                    continue;
                }
                let h = -[10.0, 18.0, 26.0, 34.0][ri(4)];
                let crit = rnd() < 0.15 + 0.08 * self.rank(Sk::Executioner) as f32 + g.crit / 100.0;
                let pw = 20.0 + edge as f32 * 8.0 + g.keen + if crit { 20.0 } else { 0.0 };
                self.cut_enemy(i, h, pw, dmg * if crit { 1.5 } else { 1.0 }, px, crit);
                if self.enemies[i].alive {
                    self.on_hit(i, dmg, h);
                }
            }"""),
])

edit('src/gear.rs', [
    ("    fn on_hit(&mut self, i: usize, dmg: f32, h: f32) {", "    pub(crate) fn on_hit(&mut self, i: usize, dmg: f32, h: f32) {"),
    ("""    fn crush_enemy(&mut self, i: usize, h: f32, dmg: f32, crit: bool) {
        if self.enemies[i].frozen > 0 {
            self.shatter(i);
            return;
        }""", """    fn crush_enemy(&mut self, i: usize, h: f32, dmg: f32, crit: bool) {
        if self.enemies[i].frozen > 0 && self.shatter(i) {
            return;
        }"""),
])

B = 'src/bosses.rs'
edit(B, [
    ("""    /// bosses don't shatter: the ice cracks off them for a chunk of damage instead
    pub fn boss_shatter(&mut self, i: usize) -> bool {
        if !self.enemies[i].k.boss() {
            return false;
        }
        let e = &mut self.enemies[i];
        e.frozen = 0;
        // only the first crack of each freeze hurts; then it's immune until it thaws
        let fresh = e.thaw == 0 || e.thaw > 180;
        e.thaw = 180;
        if !fresh {
            return true;
        }
        let dmg = e.maxhp * 0.02;
        let (x, y) = (e.x, e.y);
        self.snd(Sfx::Shatter, x, y, 0.9);
        for _ in 0..30 {
            self.ptc(x, y, rr(10.0, 60.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.0, 2.0), 60, PtK::Ice, [200, 230, 255]);
        }
        self.hurt(i, dmg, [180, 220, 255], true);
        true
    }""", """    /// Bosses and champions don't shatter outright: the ice cracks off them for a chunk of
    /// damage instead. Returns false when nothing cracked, so the blow lands as normal.
    pub fn boss_shatter(&mut self, i: usize) -> Option<bool> {
        let e = &self.enemies[i];
        let boss = e.k.boss();
        // a champion with plenty of health left takes a heavy crack, not instant death
        let champ = !boss && e.elite && e.hp > e.maxhp * 0.3;
        if !boss && !champ {
            return None;
        }
        // only the first crack of each freeze hurts; then a boss is immune until it thaws
        let fresh = !boss || e.thaw == 0 || e.thaw > 180;
        if !fresh {
            let e = &mut self.enemies[i];
            e.frozen = 0;
            return Some(false);
        }
        let dmg = if boss { e.maxhp * 0.02 } else { e.maxhp * 0.3 };
        let (x, y) = (e.x, e.y);
        self.snd(Sfx::Shatter, x, y, 0.9);
        for _ in 0..30 {
            self.ptc(x, y, rr(10.0, 60.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.0, 2.0), 60, PtK::Ice, [200, 230, 255]);
        }
        // still frozen while it's hurt, so Deep Freeze counts
        self.hurt(i, dmg, [180, 220, 255], true);
        let e = &mut self.enemies[i];
        e.frozen = 0;
        if boss {
            e.thaw = 180;
        }
        Some(true)
    }"""),
])

G = 'src/game.rs'
edit(G, [
    ("""    pub fn shatter(&mut self, i: usize) {
        if self.boss_shatter(i) {
            return;
        }""", """    /// returns whether the blow was used up by the shatter
    pub fn shatter(&mut self, i: usize) -> bool {
        if let Some(cracked) = self.boss_shatter(i) {
            return cracked;
        }"""),
    # explosions and acid use the same Searing / Deep Freeze rules
    ("""            let f = 1.0 - d / (r * 2.2);
            let dmg = (36.0 * f + 6.0) * sp;
            e.hp -= dmg;""", """            let f = 1.0 - d / (r * 2.2);
            let dmg = (36.0 * f + 6.0) * sp * self.status_mult(i);
            let e = &mut self.enemies[i];
            e.hp -= dmg;"""),
    ("""            let ad = 6.0 * self.spell_power();
            let en = &mut self.enemies[i];""", """            let ad = 6.0 * self.spell_power() * self.status_mult(i);
            let en = &mut self.enemies[i];"""),
    # Swiftness: quicker, longer dodges as promised
    ("""            p.dodge = 16;
            p.dodge_cd = 38;
            self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.8 });
            p.inv = 18;
            p.dv = (ddx * 3.1, ddy * 3.1);""", """            let sw = p.sk.r(Sk::Swiftness);
            p.dodge = 16;
            p.dodge_cd = 38 - 5 * sw;
            self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.8 });
            p.inv = 18;
            let dv = 3.1 * (1.0 + 0.06 * sw as f32);
            p.dv = (ddx * dv, ddy * dv);"""),
])
edit('src/barrels.rs', [("""            let dmg = (150.0 * f + 25.0) * if boss { 0.6 } else { 1.0 };
            let e = &mut self.enemies[i];""", """            let dmg = (150.0 * f + 25.0) * if boss { 0.6 } else { 1.0 } * self.status_mult(i);
            let e = &mut self.enemies[i];""")])
print("combat ok")
