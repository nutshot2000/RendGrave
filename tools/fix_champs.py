def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub elite: bool,\n    /// on its back", "    pub elite: bool,\n    /// champion power, see champions.rs (0 none)\n    pub aff: u8,\n    /// on its back"),
    ("        elite: false,\n        slip: 0,\n", "        elite: false,\n        aff: 0,\n        slip: 0,\n"),
    ("    pub shield: i32,\n", "    pub shield: i32,\n    /// chilled by a frozen champion: slower\n    pub frost: i32,\n"),
    ("            shield: 0,\n", "            shield: 0,\n            frost: 0,\n"),
    ("                                retal.push(i);\n", "                                retal.push(i);\n                                champ_hits.push(i);\n"),
    ("        let mut retal: Vec<usize> = vec![];\n", "        let mut retal: Vec<usize> = vec![];\n        let mut champ_hits: Vec<usize> = vec![];\n"),
    ("        for i in retal {\n            self.retaliate(i);\n        }\n", "        for i in retal {\n            self.retaliate(i);\n        }\n        for i in champ_hits {\n            self.champion_hit(i);\n        }\n"),
    ("            self.boss_moves(i);\n", "            self.boss_moves(i);\n            self.champion_tick(i);\n"),
    ("""        if k == Kind::Bloater {
            self.bloater_burst(x, y);
        }""", """        if k == Kind::Bloater {
            self.bloater_burst(x, y);
        }
        self.champion_death(i);"""),
    ("        let hm = self.haste_mult();\n", "        let hm = self.haste_mult() * if self.player.frost > 0 { 0.6 } else { 1.0 };\n"),
])
edit('src/talents.rs', [("""        if p.stand_cd > 0 {
            p.stand_cd -= 1;
        }""", """        if p.stand_cd > 0 {
            p.stand_cd -= 1;
        }
        if p.frost > 0 {
            p.frost -= 1;
            if p.frost % 6 == 0 {
                let (x, y) = (p.x, p.y);
                self.ptc(x + rr(-6.0, 6.0), y, rr(5.0, 45.0), 0.0, 0.0, -0.3, 30, PtK::Ice, [200, 235, 255]);
            }
        }
        let p = &mut self.player;""")])
edit('src/areas.rs', [("""            e.dmg *= 1.3;
            e.speed *= 1.1;""", """            e.dmg *= 1.3;
            e.speed *= 1.1;
            e.aff = 1 + ri(5) as u8;
            if e.aff == 4 {
                e.speed *= 1.5;
            }""")])
R = 'src/render.rs'
edit(R, [
    ("    pub elite: bool,\n", "    pub elite: bool,\n    pub elite_col: [u8; 3],\n"),
    ("            elite: e.elite,\n        };", "            elite: e.elite,\n            elite_col: crate::champions::champ_col(e.aff),\n        };"),
    ("""                    e = [(255.0 * pulse) as u8, (190.0 * pulse) as u8, (60.0 * pulse) as u8];""", """                    let ec = tint.elite_col;
                    e = [(ec[0] as f32 * pulse) as u8, (ec[1] as f32 * pulse) as u8, (ec[2] as f32 * pulse) as u8];"""),
])
s = open(R, encoding='utf8').read()
# lights for champions use their colour too
a = """        if e.alive && e.elite {
            add(e.x, e.y, 60.0, 0.6, [1.0, 0.75, 0.3]);
        }"""
assert s.count(a) == 1
s = s.replace(a, """        if e.alive && e.elite {
            let c = crate::champions::champ_col(e.aff);
            add(e.x, e.y, 60.0, 0.6, [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]);
        }""")
open(R, 'w', encoding='utf8').write(s)
edit('src/hud.rs', [("""            u.text_c(sx, sy - 8, "CHAMPION", [255, 200, 90], 1);""", """            u.text_c(sx, sy - 8, &crate::champions::champ_name(e.aff), crate::champions::champ_col(e.aff), 1);""")])
edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod champions;\n")])
print("champions wired")
