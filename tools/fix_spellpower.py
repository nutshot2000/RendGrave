def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    # explosions: the enemy loop borrows enemies, so read the multiplier first
    ("""        // hurt everything nearby
        let mut shat = vec![];""", """        // hurt everything nearby
        let sp = self.spell_power();
        let mut shat = vec![];"""),
    ("            let dmg = 36.0 * f + 6.0;", "            let dmg = (36.0 * f + 6.0) * sp;"),
    ("""            let en = &mut self.enemies[i];
            en.aggro = true;
            en.hp -= 6.0;""", """            let ad = 6.0 * self.spell_power();
            let en = &mut self.enemies[i];
            en.aggro = true;
            en.hp -= ad;"""),
    ("""            let (ex, ey) = (en.x, en.y);
            self.float(ex, ey - 60.0, "6".into(), [170, 240, 90], false);""", """            let (ex, ey) = (en.x, en.y);
            self.float(ex, ey - 60.0, format!("{}", ad as i32), [170, 240, 90], false);"""),
    ("    pub summoned: bool,\n", "    pub summoned: bool,\n    /// bosses: a short freeze, then a spell of frost immunity\n    pub thaw: i32,\n"),
    ("        summoned: false,\n        skill_t: 180,\n", "        summoned: false,\n        thaw: 0,\n        skill_t: 180,\n"),
    ("""            let boss = e.k.boss();
            let burning = e.body.update_fx(&mut fl, &mut dr, boss);""", """            let boss = e.k.boss();
            if boss {
                // bosses break free fast and can't be frozen again for a few seconds
                if e.thaw > 0 {
                    e.thaw -= 1;
                    if e.thaw <= 180 {
                        e.frozen = 0;
                    }
                } else if e.frozen > 0 {
                    e.frozen = e.frozen.min(20);
                    e.thaw = 200;
                }
            }
            let burning = e.body.update_fx(&mut fl, &mut dr, boss);"""),
])
s = open(G, encoding='utf8').read()
s += '''
impl Game {
    /// spells grow with you: +7% damage per character level
    pub fn spell_power(&self) -> f32 {
        1.0 + 0.07 * (self.player.sk.level - 1) as f32
    }
}
'''
open(G, 'w', encoding='utf8').write(s)

S = 'src/spells.rs'
edit(S, [
    ("        let mult = 1.0 + 0.15 * self.rank(Sk::ChainMastery) as f32;\n        let (cond, stat)", "        let mult = (1.0 + 0.15 * self.rank(Sk::ChainMastery) as f32) * self.spell_power();\n        let (cond, stat)"),
    ("self.cut_enemy(i, h, 26.0, 14.0, sx", "self.cut_enemy(i, h, 26.0, 14.0 * self.spell_power(), sx"),
])
edit('src/talents.rs', [("let dmg = 4.0 * (1.0 + 0.4 * self.rank(Sk::Caustic) as f32) * (1.0 + 0.08 * (self.alvl - 1) as f32);", "let dmg = 4.0 * (1.0 + 0.4 * self.rank(Sk::Caustic) as f32) * self.spell_power();")])
edit('src/bosses.rs', [("        let dmg = e.maxhp * 0.04;", "        let dmg = e.maxhp * 0.02;")])
print("spell power ok")
