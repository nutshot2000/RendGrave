# The Ravenhold bridge: a hand-built encounter on the road to the gate (slice step 5).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


A = 'src/areagen.rs'
edit(A, [
    ("""    /// cracked walls hiding a secret
    pub cracks: Vec<(f32, f32)>,
}""", """    /// cracked walls hiding a secret
    pub cracks: Vec<(f32, f32)>,
    /// a hand-built encounter announced when you come near
    pub setpiece: Option<(f32, f32)>,
}"""),
    ("posts: vec![], cracks: vec![] }", "posts: vec![], cracks: vec![], setpiece: None }"),
    ("""    // a gallows by the gate, as a warning""", """    bridge_ambush(w, &mut sp, c0 as f32 - 120.0);
    // a gallows by the gate, as a warning"""),
    ("""fn ravenhold(w: &mut World, s: u32) -> Spawns {""", """/// The Ravenhold bridge. A stone-walled drainage ditch crosses the road short of the gate:
/// lamp oil pools under the bridge, water at either end. Bandits hold the far side behind a
/// wooden barricade, with a powder keg and a chest. Every way works:
///   fight across the bridge; hack or burn the barricade (it's wood); shoot the keg;
///   lure them down into the oil through the gaps in the ditch walls and set it alight;
///   or wade (or freeze the water and walk) across at either end and take them from behind
fn bridge_ambush(w: &mut World, sp: &mut Spawns, bx: f32) {
    let bx = bx as i32;
    // where the road crosses this column
    let rows: Vec<i32> = (0..w.ah as i32).filter(|&y| w.g[idx(bx, y)] == ROAD).collect();
    if rows.is_empty() {
        return;
    }
    let ry = rows[rows.len() / 2];
    let (x0, x1) = (bx - 14, bx + 14);
    let (y0, y1) = (ry - 160, ry + 160);
    for y in y0..y1 {
        for x in x0 - 3..=x1 + 3 {
            if !inarea(w, x, y) {
                continue;
            }
            let i = idx(x, y);
            let on_bridge = (y - ry).abs() <= 9;
            let wall = x < x0 || x > x1;
            // gaps in the ditch walls: steps down on the far ends, both sides
            let gap = ((y - (ry - 120)).abs() < 7 || (y - (ry + 125)).abs() < 7) && wall;
            if on_bridge {
                w.m[i] = E;
                w.g[i] = PLANK;
            } else if wall && !gap {
                w.m[i] = STONE;
                w.col[i] = [96, 92, 88];
            } else if wall {
                w.m[i] = E;
                w.g[i] = MUD;
            } else {
                w.m[i] = if (y - ry).abs() < 60 { OIL } else { WATER };
                w.g[i] = MUD;
            }
        }
    }
    // the barricade: a wall of planks across the far end of the bridge, man-high
    for y in ry - 9..=ry + 9 {
        for x in x1 + 8..x1 + 13 {
            let i = idx(x, y);
            w.m[i] = WOOD;
            w.col[i] = if (y + x) % 4 == 0 { [86, 60, 36] } else { [128, 90, 54] };
        }
    }
    // the bandits who hold it, and what they guard
    let fx = (x1 + 34) as f32;
    let r = ry as f32;
    sp.enemies.push((Kind::Bandit, fx, r - 14.0));
    sp.enemies.push((Kind::Bandit, fx, r + 14.0));
    sp.enemies.push((Kind::Bandit, fx + 22.0, r));
    sp.enemies.push((Kind::Crossbow, fx + 40.0, r - 30.0));
    sp.enemies.push((Kind::Crossbow, fx + 40.0, r + 30.0));
    barrels(w, sp, fx - 6.0, r - 30.0, 1, &[(0, 1.0)]);
    barrels(w, sp, fx - 6.0, r + 28.0, 2, &[(0, 0.5), (1, 0.5)]);
    sp.chests.push((fx + 26.0, r - 44.0, 1));
    sp.braziers.push((fx + 8.0, r + 44.0));
    sp.posts.push((bx as f32 - 30.0, r - 16.0));
    sp.setpiece = Some((bx as f32, r));
}

fn ravenhold(w: &mut World, s: u32) -> Spawns {"""),
])

G = 'src/game.rs'
edit(G, [
    ("    pub loose_bows: Vec<crate::anatomy::LooseBow>,\n", "    pub loose_bows: Vec<crate::anatomy::LooseBow>,\n    /// a hand-built encounter in this area, and whether it has been announced this visit\n    pub setpiece: Option<(f32, f32)>,\n"),
    ("            loose_bows: vec![],\n", "            loose_bows: vec![],\n            setpiece: None,\n"),
    ("        self.ember_ring();\n", "        self.ember_ring();\n        if let Some((sx, sy)) = self.setpiece {\n            if ((self.player.x - sx).powi(2) + (self.player.y - sy).powi(2)).sqrt() < 230.0 {\n                self.setpiece = None;\n                self.banner = Some((\"THE RAVENHOLD BRIDGE\".into(), \"OIL IN THE DITCH, WOOD IN THE BARRICADE - FIND A WAY OVER\".into(), 260));\n                self.snd(crate::audio::Sfx::Shout, sx, sy, 1.0);\n            }\n        }\n"),
])
edit('src/areas.rs', [("        self.cracks = sp.cracks.clone();\n", "        self.cracks = sp.cracks.clone();\n        self.setpiece = sp.setpiece;\n")])
print("bridge ok")
