def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub npcs: Vec<crate::story::Npc>,\n", "    pub npcs: Vec<crate::story::Npc>,\n    /// minimap fog of war: which minimap cells you've seen, per area (dungeons reset each visit)\n    pub seen: std::collections::HashMap<String, Vec<bool>>,\n"),
    ("            npcs: vec![],\n", "            npcs: vec![],\n            seen: std::collections::HashMap::new(),\n"),
    ("        self.ambient();\n", """        self.ambient();
        if self.frame % 6 == 0 {
            self.reveal();
        }
"""),
])
s = open(G, encoding='utf8').read()
s += '''
/// minimap size in cells
pub const MM: (usize, usize) = (84, 63);

impl Game {
    /// uncover the minimap around you
    pub fn reveal(&mut self) {
        let key = self.area.key();
        let (sx, sy) = (self.world.aw as f32 / MM.0 as f32, self.world.ah as f32 / MM.1 as f32);
        let (px, py) = (self.player.x / sx, self.player.y / sy);
        let (rx, ry) = (230.0 / sx, 170.0 / sy);
        let v = self.seen.entry(key).or_insert_with(|| vec![false; MM.0 * MM.1]);
        for y in (py - ry).floor().max(0.0) as usize..((py + ry).ceil() as usize).min(MM.1) {
            for x in (px - rx).floor().max(0.0) as usize..((px + rx).ceil() as usize).min(MM.0) {
                let (dx, dy) = ((x as f32 + 0.5 - px) / rx, (y as f32 + 0.5 - py) / ry);
                if dx * dx + dy * dy < 1.0 {
                    v[y * MM.0 + x] = true;
                }
            }
        }
    }
    pub fn seen_at(&self, x: usize, y: usize) -> bool {
        self.seen.get(&self.area.key()).map(|v| v[y * MM.0 + x]).unwrap_or(false)
    }
}
'''
open(G, 'w', encoding='utf8').write(s)

A = 'src/areas.rs'
edit(A, [
    ("        self.posts = sp.posts.clone();\n", "        self.posts = sp.posts.clone();\n        if to.is_dungeon() {\n            self.seen.remove(&to.key());\n        }\n"),
    ("""        let mut lines: Vec<String> = lines.to_vec();""", """        let mut lines: Vec<String> = lines.to_vec();
        // explored minimap, packed into hex, for the outdoor areas
        for (k, v) in &self.seen {
            if AreaId::from_key(k).map(|a| a.is_dungeon()).unwrap_or(true) {
                continue;
            }
            let mut hex = String::new();
            for c in v.chunks(4) {
                let n = c.iter().enumerate().fold(0u8, |a, (i, b)| a | ((*b as u8) << i));
                hex.push(char::from_digit(n as u32, 16).unwrap());
            }
            lines.push(format!("seen_{} {}", k, hex));
        }"""),
])
s = open(A, encoding='utf8').read()
a = '                "story" => self.story = v.parse().unwrap_or(0),'
assert s.count(a) == 1
s = s.replace(a, a + '''
                _ if k.starts_with("seen_") => {
                    let mut out = vec![];
                    for ch in v.chars() {
                        let n = ch.to_digit(16).unwrap_or(0);
                        for i in 0..4 {
                            out.push(n & (1 << i) != 0);
                        }
                    }
                    out.resize(crate::game::MM.0 * crate::game::MM.1, false);
                    self.seen.insert(k[5..].to_string(), out);
                }''')
open(A, 'w', encoding='utf8').write(s)

H = 'src/hud.rs'
edit(H, [
    ("""            let (wx, wy) = ((x as f32 * sx) as usize, (y as f32 * sy) as usize);
            let i = wy * W + wx;""", """            if !g.seen_at(x as usize, y as usize) {
                // not explored yet
                u.px(ox + x, oy + y, [12, 10, 16], 220);
                continue;
            }
            let (wx, wy) = ((x as f32 * sx) as usize, (y as f32 * sy) as usize);
            let i = wy * W + wx;"""),
])
s = open(H, encoding='utf8').read()
a = s.index('fn minimap(')
b = s.index('    for e in &g.enemies {\n        if e.alive {', a)
c = s.index('\n', b + 40)
s = s[:b] + '    for e in g.enemies.iter().filter(|e| g.seen_at(((e.x / sx) as usize).min(83), ((e.y / sy) as usize).min(62))) {\n        if e.alive {' + s[b + len('    for e in &g.enemies {\n        if e.alive {'):]
open(H, 'w', encoding='utf8').write(s)
print("fog ok")
