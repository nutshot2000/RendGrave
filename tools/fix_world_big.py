def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


W_ = 'src/world.rs'
edit(W_, [
    ("pub const W: usize = 1024;\npub const H: usize = 768;", "/// the largest map the engine holds; each area uses its own size inside it (aw x ah)\npub const W: usize = 2048;\npub const H: usize = 1536;"),
    ("pub const ROCK: u8 = 17;", "pub const ROCK: u8 = 17;\n/// swamp gas: invisible-ish, explodes when fire reaches it\npub const GAS: u8 = 18;"),
    ("pub const PLANK: u8 = 8;", "pub const PLANK: u8 = 8;\n/// bog peat: once lit it smoulders for a long time and creeps outwards\npub const PEAT: u8 = 9;"),
    ("    Fume(i32, i32),\n}", "    Fume(i32, i32),\n    Boom(i32, i32),\n}"),
    ("    pub events: Vec<Ev>,\n}", "    pub events: Vec<Ev>,\n    /// this area's size\n    pub aw: usize,\n    pub ah: usize,\n}"),
    ("            act: vec![0; CW * CH],\n            stamp: vec![0; n],", "            act: vec![0; CW * CH],\n            stamp: vec![0; n],\n            aw: W,\n            ah: H,"),
    ("""            FIRE => self.fire(x, y, i, t),""", """            FIRE => self.fire(x, y, i, t),
            GAS => {
                // drifts a little
                if rnd() < 0.03 {
                    if let Some((j, nx, ny)) = self.nb(x, y, N4[ri(4)]) {
                        if self.m[j] == E {
                            self.m[j] = GAS;
                            self.m[i] = E;
                            self.stamp[j] = t;
                            self.keep(nx as usize, ny as usize);
                        }
                    }
                }
            }"""),
    # fire meets gas and peat
    ("""                E => {
                    let gg = self.g[j];
                    if (gg == GRASS || gg == MOSS) && rnd() < 0.035 {""", """                GAS => {
                    self.m[j] = FIRE;
                    self.life[j] = rr(6.0, 14.0) as u8;
                    self.stamp[j] = t;
                    self.wake(nx, ny);
                    if rnd() < 0.08 {
                        self.events.push(Ev::Boom(nx, ny));
                    }
                }
                E if self.g[j] == PEAT && rnd() < 0.02 => {
                    self.m[j] = FIRE;
                    self.life[j] = rr(150.0, 250.0) as u8;
                    self.stamp[j] = t;
                    self.wake(nx, ny);
                }
                E => {
                    let gg = self.g[j];
                    if (gg == GRASS || gg == MOSS) && rnd() < 0.035 {"""),
    ("""    fn fire(&mut self, x: usize, y: usize, i: usize, t: u8) {
        let l = self.life[i];
        if l <= 1 {""", """    fn fire(&mut self, x: usize, y: usize, i: usize, t: u8) {
        let l = self.life[i];
        let peat = self.g[i] == PEAT;
        if peat && l <= 1 {
            self.g[i] = BURNT;
        }
        if l <= 1 {"""),
    ("""        self.life[i] = l - 1;
        if (self.g[i] == GRASS || self.g[i] == MOSS) && rnd() < 0.1 {
            self.g[i] = BURNT;
        }""", """        // peat smoulders: it burns slowly
        if !(peat && rnd() < 0.8) {
            self.life[i] = l - 1;
        }
        if (self.g[i] == GRASS || self.g[i] == MOSS) && rnd() < 0.1 {
            self.g[i] = BURNT;
        }"""),
])

edit('src/game.rs', [
    ("        self.cam.0 = self.cam.0.clamp(0.0, W as f32 - vw);\n        self.cam.1 = self.cam.1.clamp(-40.0, H as f32 - vh);",
     "        self.cam.0 = self.cam.0.clamp(0.0, (self.world.aw as f32 - vw).max(0.0));\n        self.cam.1 = self.cam.1.clamp(-20.0, (self.world.ah as f32 - vh).max(0.0));"),
    ("""                Ev::Fume(x, y) => self.ptc(x as f32, y as f32, 1.0, rr(-0.2, 0.2), rr(-0.2, 0.2), 0.4, 60, PtK::Steam, [150, 220, 90]),""",
     """                Ev::Fume(x, y) => self.ptc(x as f32, y as f32, 1.0, rr(-0.2, 0.2), rr(-0.2, 0.2), 0.4, 60, PtK::Steam, [150, 220, 90]),
                Ev::Boom(x, y) => {
                    if booms < 4 {
                        booms += 1;
                        self.explode(x as f32, y as f32, 3.0, 7.0);
                    }
                }"""),
    ("""        let evs = std::mem::take(&mut self.world.events);""", """        let evs = std::mem::take(&mut self.world.events);
        let mut booms = 0;"""),
])
edit('src/hud.rs', [
    ("    let sx = W as f32 / mw as f32;\n    let sy = H as f32 / mh as f32;", "    let sx = g.world.aw as f32 / mw as f32;\n    let sy = g.world.ah as f32 / mh as f32;"),
    ("                TREE => [24, 52, 28],\n                ROCK => [8, 7, 10],", "                TREE => [24, 52, 28],\n                ROCK => [8, 7, 10],\n                GAS => [90, 120, 60],"),
    ("                    BURNT => [30, 26, 26],", "                    BURNT => [30, 26, 26],\n                    PEAT => [44, 34, 26],"),
])
R = 'src/render.rs'
edit(R, [
    ("""        PLANK => {""", """        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);
            scl([54, 42, 32], 0.75 + fib * 0.35 + s * 0.1)
        }
        PLANK => {"""),
    ("""        ASH => (scl([84, 80, 78], 0.8 + s * 0.3), z),""", """        ASH => (scl([84, 80, 78], 0.8 + s * 0.3), z),
        GAS => {
            let wv = vnoise(x as f32 * 0.08 + t as f32 * 0.004, y as f32 * 0.08, 71);
            (mixc(g, [120, 170, 70], 0.18 + wv * 0.22), [0, (6.0 + wv * 10.0) as u8, 0])
        }"""),
])
print("world big ok")
