def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/body.rs', [("""    pub fn detach_everything(&mut self) -> Vec<Detached> {""", """    /// pixels currently being eaten by acid
    pub fn count_acid(&self) -> i32 {
        self.parts.iter().filter(|p| p.on).map(|p| p.px.iter().filter(|q| q.m != 0 && q.acid > 0).count() as i32).sum()
    }
    pub fn detach_everything(&mut self) -> Vec<Detached> {""")])

G = 'src/game.rs'
edit(G, [
    ("    pub summoned: bool,\n", "    pub summoned: bool,\n    /// bosses: frames until the next special move, slam wind-up, enraged\n    pub skill_t: i32,\n    pub slam: i32,\n    pub enraged: bool,\n"),
    ("        summoned: false,\n        elite: false,\n", "        summoned: false,\n        skill_t: 180,\n        slam: 0,\n        enraged: false,\n        elite: false,\n"),
    ("""            let burning = e.body.update_fx(&mut fl, &mut dr, false);
            e.burning = burning;""", """            let boss = e.k.boss();
            let burning = e.body.update_fx(&mut fl, &mut dr, boss);
            e.burning = burning;
            if boss {
                // a boss's body doesn't melt away: acid and fire wear its health down instead
                let acid = e.body.count_acid();
                e.hp -= acid.min(140) as f32 * 0.003;
                if acid > 0 {
                    e.body.tint_all(|p| p.acid = p.acid.saturating_sub(2));
                }
            }"""),
    ("            if e.hp <= 0.0 || !head || torso < 0.45 {", "            if e.hp <= 0.0 || (!e.k.boss() && (!head || torso < 0.45)) {"),
    ("            self.boss_logic(i);\n", "            self.boss_logic(i);\n            self.boss_moves(i);\n"),
    ("                if p.k == 1 {\n                    p.vz -= 0.09 / steps as f32;", "                if p.k == 1 || p.k == 5 {\n                    p.vz -= 0.09 / steps as f32;"),
    ("""                        1 => self.acid_hit(x, y, z, hit_e),""", """                        1 => self.acid_hit(x, y, z, hit_e),
                        5 => self.drop_keg(x - self.projs[k].vx * 2.0, y - self.projs[k].vy * 2.0),"""),
    ("""    pub fn shatter(&mut self, i: usize) {""", """    pub fn shatter(&mut self, i: usize) {
        if self.boss_shatter(i) {
            return;
        }"""),
])

edit('src/spells.rs', [("""    pub fn cut_enemy(&mut self, i: usize, h: f32, power: f32, dmg: f32, src_x: f32, crit: bool) {
        if self.enemies[i].frozen > 0 {
            self.shatter(i);
            return;
        }""", """    pub fn cut_enemy(&mut self, i: usize, h: f32, power: f32, dmg: f32, src_x: f32, crit: bool) {
        if self.enemies[i].frozen > 0 {
            self.shatter(i);
            return;
        }
        // a boss's hide is thick: blades scar it but rarely take anything off
        let power = if self.enemies[i].k.boss() { power * 0.35 } else { power };""")])

edit('src/render.rs', [
    ("""            shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k.big() { 11.0 } else if e.k == Kind::Bloater { 10.0 } else { 7.0 });""", """            shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k.big() { 11.0 } else if e.k == Kind::Bloater { 10.0 } else { 7.0 });
            if e.slam > 0 {
                // the slam is coming: a red ring on the ground, filling in
                let f = 1.0 - e.slam as f32 / crate::bosses::SLAM_T as f32;
                let r = crate::bosses::SLAM_R;
                let pulse = 0.5 + ((t as f32) * 0.5).sin() * 0.2;
                for k in 0..120 {
                    let a = k as f32 / 120.0 * std::f32::consts::TAU;
                    let (gx, gy) = ((e.x + a.cos() * r) as i32 - cx, (e.y + a.sin() * r * 0.7) as i32 - cy);
                    c.blend(gx, gy, -1e8, [255, 40, 30], pulse);
                    c.glow(gx, gy, [120, 10, 0]);
                }
                let fr = r * f;
                for k in 0..80 {
                    let a = k as f32 / 80.0 * std::f32::consts::TAU;
                    let (gx, gy) = ((e.x + a.cos() * fr) as i32 - cx, (e.y + a.sin() * fr * 0.7) as i32 - cy);
                    c.blend(gx, gy, -1e8, [255, 120, 60], 0.6);
                }
            }"""),
    ("""        if p.k == 3 || p.k == 4 {""", """        if p.k == 5 {
            // a thrown powder keg, fuse spitting sparks
            for dy in 0..6i32 {
                for dx in -2i32..=2 {
                    let col = if dy == 1 || dy == 4 { [40, 34, 30] } else if dx == dy - 3 || dx == 3 - dy { [200, 40, 30] } else { [118, 74, 42] };
                    c.put(sx + dx, sy - dy, p.y + 0.5, col, [0; 3], p.z + dy as f32);
                }
            }
            c.glow(sx + 1, sy - 7, [255, 200, 80]);
            continue;
        }
        if p.k == 3 || p.k == 4 {"""),
])
edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod bosses;\n")])
print("bosses wired")
