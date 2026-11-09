def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


# villagers need the world to find open ground
edit('src/story.rs', [
    ("""pub fn town_folk(maren: (f32, f32)) -> Vec<Npc> {
    let mut v = vec![Npc { x: maren.0, y: maren.1, k: 0, tx: maren.0, ty: maren.1, t: 0, face: 1.0, phase: 0.0, bark: 0 }];
    for k in 1..=4u8 {
        let (x, y) = (rr(400.0, 640.0), rr(340.0, 430.0));
        v.push(Npc { x, y, k, tx: x, ty: y, t: ri(200) as i32, face: 1.0, phase: rnd() * 6.0, bark: 0 });
    }
    let _ = Kind::Player;
    v
}""", """pub fn town_folk(maren: (f32, f32), w: &crate::world::World) -> Vec<Npc> {
    let mut v = vec![Npc { x: maren.0, y: maren.1, k: 0, tx: maren.0, ty: maren.1, t: 0, face: 1.0, phase: 0.0, bark: 0 }];
    let mut k = 1u8;
    for _ in 0..400 {
        if k > 4 {
            break;
        }
        let (x, y) = (rr(380.0, 660.0), rr(330.0, 440.0));
        let open = (-4..=4).all(|d| !w.solid(x as i32 + d, y as i32)) && w.aux[y as usize * crate::world::W + x as usize] & crate::world::AUX_INDOOR == 0;
        if open {
            v.push(Npc { x, y, k, tx: x, ty: y, t: ri(200) as i32, face: 1.0, phase: rnd() * 6.0, bark: 0 });
            k += 1;
        }
    }
    let _ = Kind::Player;
    v
}"""),
])

G = 'src/game.rs'
edit(G, [
    ("    fn mv(&self, x: &mut f32, y: &mut f32, dx: f32, dy: f32, r: f32) -> bool {", "    pub(crate) fn mv(&self, x: &mut f32, y: &mut f32, dx: f32, dy: f32, r: f32) -> bool {"),
    ("    pub act1_done: bool,\n", "    pub act1_done: bool,\n    /// story flags, see story.rs\n    pub story: u8,\n    /// open dialogue: its pages and the page shown\n    pub talk: Option<(Vec<&'static str>, usize)>,\n    pub npcs: Vec<crate::story::Npc>,\n"),
    ("            act1_done: false,\n", "            act1_done: false,\n            story: 0,\n            talk: None,\n            npcs: vec![],\n"),
    ("""        if self.ending > 0 {
            self.ending -= 1;""", """        if self.talk.is_some() {
            self.update_talk(inp);
            self.prev = inp.clone();
            return;
        }
        if self.ending > 0 {
            self.ending -= 1;"""),
    ("        self.update_ground();\n", "        self.update_ground();\n        self.update_npcs();\n"),
])

A = 'src/areas.rs'
s = open(A, encoding='utf8').read()
assert s.count("        self.check_act_end();\n") == 2
s = s.replace("        self.check_act_end();\n", "")
open(A, 'w', encoding='utf8').write(s)
edit(A, [
    ("        self.merchant = sp.merchant;\n", "        self.merchant = sp.merchant;\n        self.npcs = match sp.maren {\n            Some(m) => crate::story::town_folk(m, &self.world),\n            None => vec![],\n        };\n        self.story_arrive(to);\n"),
    ("""        if let Some((mx, my)) = self.merchant {""", """        if let Some(m) = self.npcs.iter().find(|n| n.k == 0) {
            if ((self.player.x - m.x).powi(2) + ((self.player.y - m.y) * 1.4).powi(2)).sqrt() < 28.0 {
                self.prompt = Some("E  TALK TO ELDER MAREN".into());
                if press {
                    self.talk_to_maren();
                }
                return;
            }
        }
        if let Some((mx, my)) = self.merchant {"""),
    ("""            format!("act1 {}", self.act1_done as u8),""", """            format!("act1 {}", self.act1_done as u8),
            format!("story {}", self.story),"""),
    ("""                "act1" => self.act1_done = v == "1",""", """                "act1" => self.act1_done = v == "1",
                "story" => self.story = v.parse().unwrap_or(0),"""),
])
s = open(A, encoding='utf8').read()
# after a load, fill in story flags old saves don't have
a = "        self.apply_gear();\n        self.player.hp = self.player.maxhp;"
assert s.count(a) == 1
s = s.replace(a, "        self.infer_story();\n" + a)
open(A, 'w', encoding='utf8').write(s)

edit('src/areagen.rs', [
    ("""    /// the merchant's stall
    pub merchant: Option<(f32, f32)>,
}""", """    /// the merchant's stall
    pub merchant: Option<(f32, f32)>,
    /// where Elder Maren stands
    pub maren: Option<(f32, f32)>,
}"""),
    ("barrels: vec![], merchant: None }", "barrels: vec![], merchant: None, maren: None }"),
    ("""    sp.merchant = Some((mx, my));""", """    sp.merchant = Some((mx, my));
    sp.maren = Some((sx - 30.0, sy - 14.0));"""),
])

edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod story;\n")])
print("story wired")
