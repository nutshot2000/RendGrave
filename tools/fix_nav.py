def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


G = 'src/game.rs'
edit(G, [
    ("    pub barrels: Vec<crate::barrels::Barrel>,\n", "    pub barrels: Vec<crate::barrels::Barrel>,\n    /// flow field towards the player, for enemy pathfinding\n    pub nav: crate::nav::Nav,\n"),
    ("            barrels: vec![],\n", "            barrels: vec![],\n            nav: crate::nav::Nav::new(),\n"),
    ("""    fn update_enemies(&mut self) {
        let bm = self.burn_mult();
        let (px, py) = (self.player.x, self.player.y);""", """    fn update_enemies(&mut self) {
        let bm = self.burn_mult();
        let (px, py) = (self.player.x, self.player.y);
        if self.frame % 8 == 0 || self.nav_dirty {
            self.nav.build(&self.world, px, py);
            self.nav_dirty = false;
        }"""),
    ("    pub nav: crate::nav::Nav,\n", "    pub nav: crate::nav::Nav,\n    pub nav_dirty: bool,\n"),
    ("            nav: crate::nav::Nav::new(),\n", "            nav: crate::nav::Nav::new(),\n            nav_dirty: true,\n"),
    # chasing: walk straight only when the way is clear and close; otherwise follow the flow field
    ("""                    St::Idle | St::Chase => {
                        e.st = St::Chase;
                        if e.k.ranged() {
                            let l = dist.max(1.0);
                            let sees = self.world.los(e.x, e.y - 1.0, px, py - 1.0);
                            if dist < 70.0 {
                                want = (-dx / l * spd, -dy / l * spd);
                            } else if dist > 125.0 || !sees {
                                want = (dx / l * spd, dy / l * spd);
                            }""", """                    St::Idle | St::Chase => {
                        e.st = St::Chase;
                        let l = dist.max(1.0);
                        let sees = self.world.los(e.x, e.y - 1.0, px, py - 1.0);
                        // the way to the player: straight at them when close and in sight,
                        // otherwise along the flow field round whatever is in the way
                        let toward = if sees && dist < 90.0 {
                            (dx / l, dy / l)
                        } else {
                            self.nav.dir(e.x, e.y).unwrap_or((dx / l, dy / l))
                        };
                        if e.k.ranged() {
                            if dist < 70.0 {
                                want = (-dx / l * spd, -dy / l * spd);
                            } else if dist > 125.0 || !sees {
                                want = (toward.0 * spd, toward.1 * spd);
                            }"""),
    ("""                        } else if dist > e.reach * 0.8 {
                            let l = dist.max(1.0);
                            want = (dx / l * spd, dy / l * spd);
                        }""", """                        } else if dist > e.reach * 0.8 {
                            want = (toward.0 * spd, toward.1 * spd);
                        }"""),
])
edit('src/areas.rs', [("        self.world = world;\n", "        self.world = world;\n        self.nav_dirty = true;\n")])
edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod nav;\n")])
print("nav ok")
