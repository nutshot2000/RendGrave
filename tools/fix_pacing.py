# Pacing: start with only a sword; spells unlock across the level range; ultimates need a tome.


def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/skills.rs', [
    ("""/// minimum character level for each tier
pub const ROW_LEVEL: [i32; 4] = [1, 2, 4, 6];
""", """/// Minimum character level for a node. Blade and Body come early; the spell schools
/// open at level 4 and their ultimates at 30 (which also need a tome found in the world).
pub fn min_level(i: usize) -> i32 {
    let n = &NODES[i];
    if n.col >= 4 {
        [1, 5, 12, 20][n.row]
    } else {
        [4, 8, 18, 30][n.row]
    }
}
/// tier-4 spells must be found as tomes before they can be learned
pub fn needs_tome(i: usize) -> bool {
    NODES[i].active && NODES[i].row == 3
}
"""),
    ("""    pub points: i32,
}""", """    pub points: i32,
    pub tomes: [bool; N],
}"""),
    ("""        let mut rank = [0u8; N];
        for s in [Sk::Lightning, Sk::Fireball, Sk::Acid, Sk::Nova] {
            rank[s as usize] = 1;
        }
        Skills { rank, level: 1, xp: 0, points: 1 }""", """        // you begin with nothing but a sword
        Skills { rank: [0u8; N], level: 1, xp: 0, points: 0, tomes: [false; N] }"""),
    ("""    pub fn can_learn(&self, i: usize) -> Result<(), &'static str> {
        let n = &NODES[i];
        if self.rank[i] >= n.max {
            return Err("MAXED");
        }
        if self.points <= 0 {
            return Err("NO SKILL POINTS");
        }
        if self.level < ROW_LEVEL[n.row] {
            return Err(["", "NEEDS LEVEL 2", "NEEDS LEVEL 4", "NEEDS LEVEL 6"][n.row]);
        }
        if n.row > 0 {
            let above = NODES.iter().position(|m| m.col == n.col && m.row == n.row - 1).unwrap();
            if self.rank[above] == 0 {
                return Err("LEARN THE SKILL ABOVE FIRST");
            }
        }
        Ok(())
    }""", """    pub fn can_learn(&self, i: usize) -> Result<(), String> {
        let n = &NODES[i];
        if self.rank[i] >= n.max {
            return Err("MAXED".into());
        }
        if self.level < min_level(i) {
            return Err(format!("NEEDS LEVEL {}", min_level(i)));
        }
        if needs_tome(i) && !self.tomes[i] {
            return Err("FIND ITS TOME IN THE WORLD FIRST".into());
        }
        if n.row > 0 {
            let above = NODES.iter().position(|m| m.col == n.col && m.row == n.row - 1).unwrap();
            if self.rank[above] == 0 {
                return Err("LEARN THE SKILL ABOVE FIRST".into());
            }
        }
        if self.points <= 0 {
            return Err("NO SKILL POINTS".into());
        }
        Ok(())
    }"""),
])

edit('src/game.rs', [
    ("            mana: 100.0,\n", "            mana: 60.0,\n"),
    ("            maxmana: 100.0,\n", "            maxmana: 60.0,\n"),
    # no spells yet: the spell section simply does nothing
    ("""        let bar = self.player.sk.bar();
        let p = &mut self.player;
        if p.sel >= bar.len() {
            p.sel = 0;
        }
        let s = bar[p.sel];
        let nd = node(s);
        let si = s as usize;
        if inp.rmb && p.cd[si] == 0 && p.dodge == 0 && p.whirl == 0 {""", """        let bar = self.player.sk.bar();
        let p = &mut self.player;
        if p.sel >= bar.len() {
            p.sel = 0;
        }
        let s = bar.get(p.sel).copied().unwrap_or(Sk::Lightning);
        let nd = node(s);
        let si = s as usize;
        if inp.rmb && !bar.is_empty() && p.cd[si] == 0 && p.dodge == 0 && p.whirl == 0 {"""),
    # brutes sometimes carry a tome for a spell you don't know yet
    ("""            self.pickups.push(Pickup { k: 1, x: x + 6.0, y: y + 2.0, z: 10.0, vz: 2.0, amt: 1, t: 0 });
        }""", """            self.pickups.push(Pickup { k: 1, x: x + 6.0, y: y + 2.0, z: 10.0, vz: 2.0, amt: 1, t: 0 });
        }
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        let tome_chance = if k == Kind::Brute { 0.12 } else { 0.004 };
        if !missing.is_empty() && rnd() < tome_chance {
            let i = missing[ri(missing.len())];
            self.pickups.push(Pickup { k: 2, x: x - 6.0, y: y + 1.0, z: 12.0, vz: 2.5, amt: i as i32, t: 0 });
        }"""),
    ("""            } else {
                self.player.potions += 1;
                self.float(px, py - 56.0, "+POTION".into(), [255, 110, 120], false);
            }""", """            } else if p.k == 1 {
                self.player.potions += 1;
                self.float(px, py - 56.0, "+POTION".into(), [255, 110, 120], false);
            } else {
                let i = p.amt as usize;
                self.player.sk.tomes[i] = true;
                self.banner = Some((format!("TOME OF {}", NODES[i].name), "LEARN IT IN THE SKILL TREE AT LEVEL 30".into(), 260));
            }"""),
])

edit('src/spells.rs', [("        p.maxmana = 100.0 + 15.0 * w;", "        p.maxmana = 60.0 + 15.0 * w;")])

# tome pickup: a glowing book
edit('src/render.rs', [("""        } else {
            for dy in 0..6 {
                for dx in -2i32..=2 {
                    let neck = dy < 2 && dx.abs() > 0;""", """        } else if p.k == 2 {
            let sc = SCHOOLS[NODES[p.amt as usize].col].1;
            for dy in 0..6 {
                for dx in -4i32..=4 {
                    let spine = dx == 0;
                    let col = if spine { [60, 30, 20] } else if dy == 0 || dx.abs() == 4 { [150, 110, 50] } else { [110, 40, 34] };
                    c.put(sx + dx, sy - 7 + dy, p.y, col, [0; 3], 7.0 - dy as f32);
                }
            }
            let pulse = 0.6 + (t as f32 * 0.15).sin() * 0.4;
            c.put(sx - 2, sy - 5, p.y + 0.1, sc, scl(sc, pulse), 6.0);
            c.put(sx + 2, sy - 4, p.y + 0.1, sc, scl(sc, pulse), 6.0);
            c.glow(sx, sy - 10, scl(sc, pulse * 0.6));
        } else {
            for dy in 0..6 {
                for dx in -2i32..=2 {
                    let neck = dy < 2 && dx.abs() > 0;"""),
    ("""    for p in &g.projs {
        match p.k {""", """    for p in &g.pickups {
        if p.k == 2 {
            let sc = SCHOOLS[NODES[p.amt as usize].col].1;
            add(p.x, p.y, 70.0, 1.0, [sc[0] as f32 / 255.0, sc[1] as f32 / 255.0, sc[2] as f32 / 255.0]);
        }
    }
    for p in &g.projs {
        match p.k {"""),
])

# HUD: hide the empty spell bar, show the tome requirement in the tree
edit('src/hud.rs', [
    ("""    let bar = p.sk.bar();
    let n = bar.len() as i32;
    let sw = 20;
    let total = n * sw + (n - 1) * 3;""", """    let bar = p.sk.bar();
    let n = bar.len() as i32;
    let sw = 20;
    let total = (n * sw + (n - 1) * 3).max(0);"""),
    ("""    u.rect(bx - 4, by - 4, total + 8, sw + 8, [16, 10, 16], 200);
    for (k, &s) in bar.iter().enumerate() {""", """    if n > 0 {
        u.rect(bx - 4, by - 4, total + 8, sw + 8, [16, 10, 16], 200);
    } else {
        u.text_c(fw / 2, by + 6, "NO SPELLS YET - LEVEL UP AND PRESS K", [150, 140, 150], 1);
    }
    for (k, &s) in bar.iter().enumerate() {"""),
    ("""        if r == 0 {
            u.rect(x, y, bs, bs, [10, 8, 12], if can { 90 } else { 170 });
        }""", """        if r == 0 {
            u.rect(x, y, bs, bs, [10, 8, 12], if can { 90 } else { 170 });
            if needs_tome(i) && !sk.tomes[i] {
                u.text(x + 2, y + bs - 7, "?", [200, 160, 90], 1, true);
            }
        }"""),
    ("""        let rk = format!("{}   RANK {}/{}", kind, sk.rank[i], n.max);""", """        let rk = format!("{}   LEVEL {}   RANK {}/{}", kind, min_level(i), sk.rank[i], n.max);"""),
    ("""        u.text(18, ty + 18, "EACH TIER NEEDS THE ONE ABOVE IT AND A MINIMUM LEVEL.", [160, 150, 160], 1, false);""",
     """        u.text(18, ty + 18, "EACH TIER NEEDS THE ONE ABOVE IT AND A MINIMUM LEVEL. ULTIMATES NEED A TOME.", [160, 150, 160], 1, false);"""),
])

# test modes: give the scripted runs what they need
edit('src/main.rs', [
    ("""            g.player.sk.level = 7;
            g.player.sk.points = 30;""", """            g.player.sk.level = 40;
            g.player.sk.points = 60;
            g.player.sk.tomes = [true; N];"""),
    ("""        if f == 1 {
            g.player.x = 214.0;""", """        if f == 1 {
            g.player.sk.level = 8;
            for s in [skills::Sk::Lightning, skills::Sk::Fireball, skills::Sk::Acid, skills::Sk::Nova] {
                g.player.sk.rank[s as usize] = 1;
            }
            g.player.x = 214.0;"""),
])
print("pacing applied")
