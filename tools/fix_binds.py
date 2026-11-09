# Gamepad: put any learned spell on X, Y or B from the skill tree.
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)

G = 'src/game.rs'
edit(G, [
    ("    /// the gamepad is auto-aiming: the cursor is on an enemy but not at a chosen height\n    pub auto: bool,\n}",
     "    /// the gamepad is auto-aiming: the cursor is on an enemy but not at a chosen height\n    pub auto: bool,\n    /// X, Y or B pressed over a spell in the skill tree: put it on that button\n    pub bind: Option<usize>,\n}"),
    ("    pub tree_hover: Option<usize>,\n", "    pub tree_hover: Option<usize>,\n    /// the spells on the gamepad's X, Y and B (unset: the first spells on the bar)\n    pub binds: [Option<Sk>; 3],\n"),
    ("            tree_hover: None,\n", "            tree_hover: None,\n            binds: [None; 3],\n"),
    ("""            self.tree_hover = crate::hud::node_at(inp.mouse, self.view, self.tree_tab);""", """            self.tree_hover = crate::hud::node_at(inp.mouse, self.view, self.tree_tab);
            if let (Some(b), Some(s)) = (inp.bind, self.bindable()) {
                for k in self.binds.iter_mut() {
                    if *k == Some(s) {
                        *k = None;
                    }
                }
                self.binds[b] = Some(s);
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Chime, x, y, 0.5);
                self.toast = Some((format!("{} ON {}", node(s).name, ["X", "Y", "B"][b]), 120));
            }"""),
    ("""    /// a screen that wants a cursor rather than a fight""", """    /// the learned spell under the pointer in the skill tree, if any
    pub fn bindable(&self) -> Option<Sk> {
        let i = self.tree_hover.filter(|_| self.tree)?;
        let n = NODES.get(i)?;
        (n.active && self.player.sk.rank[i] > 0).then_some(n.sk)
    }
    /// which spell-bar slot the gamepad's X (0), Y (1) or B (2) casts
    pub fn pad_slot(&self, k: usize) -> Option<usize> {
        let bar = self.player.sk.bar();
        if let Some(i) = self.binds[k].and_then(|s| bar.iter().position(|&b| b == s)) {
            return Some(i);
        }
        // unset buttons take the first spells on the bar that no other button has
        let taken: Vec<Sk> = self.binds.iter().flatten().copied().filter(|s| bar.contains(s)).collect();
        let free: Vec<usize> = (0..bar.len()).filter(|&i| !taken.contains(&bar[i])).collect();
        let nth = (0..k).filter(|&j| self.binds[j].map_or(true, |s| !bar.contains(&s))).count();
        free.get(nth).copied()
    }
    /// a screen that wants a cursor rather than a fight"""),
])

P = 'src/pad.rs'
edit(P, [
    ("""            inp.lmb |= b.a;
            inp.rmb |= b.x;
            inp.close |= b.b || b.start;""", """            inp.lmb |= b.a;
            // in the skill tree, X / Y / B over a learned spell puts it on that button
            if g.bindable().is_some() {
                for (k, (now, was)) in [(b.x, pb.x), (b.y, pb.y), (b.b, pb.b)].into_iter().enumerate() {
                    if now && !was {
                        inp.bind = Some(k);
                    }
                }
                inp.close |= b.start;
            } else {
                inp.rmb |= b.x;
                inp.close |= b.b || b.start;
            }"""),
    ("""        let n = g.player.sk.bar().len();
        for (k, held) in [b.x, b.y, b.b].into_iter().enumerate() {
            if held && k < n {
                inp.sel = Some(k);
                inp.rmb = true;
            }
        }""", """        let n = g.player.sk.bar().len();
        for (k, held) in [b.x, b.y, b.b].into_iter().enumerate() {
            if let Some(slot) = g.pad_slot(k).filter(|_| held) {
                inp.sel = Some(slot);
                inp.rmb = true;
            }
        }"""),
])

A = 'src/areas.rs'
edit(A, [
    ('            format!("flasks {}", self.flasks),\n        ];', '            format!("flasks {}", self.flasks),\n            format!("binds {}", self.binds.iter().map(|b| b.map_or("-".to_string(), |s| format!("{:?}", s))).collect::<Vec<_>>().join(",")),\n        ];'),
    ('                "pick" => self.pick = v.parse().unwrap_or(0),', '''                "pick" => self.pick = v.parse().unwrap_or(0),
                "binds" => {
                    for (k, name) in v.split(',').take(3).enumerate() {
                        self.binds[k] = NODES.iter().find(|n| format!("{:?}", n.sk) == name).map(|n| n.sk);
                    }
                }'''),
])

H = 'src/hud.rs'
edit(H, [
    ("""        if k < 9 {
            u.text(x + 1, by + 1, &format!("{}", k + 1), [230, 220, 200], 1, true);
        }""", """        if crate::pad::active() {
            // which button casts it
            if let Some(b) = (0..3).find(|&b| g.pad_slot(b) == Some(k)) {
                let c = [[120, 200, 255], [255, 220, 90], [255, 110, 100]][b];
                u.text(x + 1, by + 1, ["X", "Y", "B"][b], c, 1, true);
            }
        } else if k < 9 {
            u.text(x + 1, by + 1, &format!("{}", k + 1), [230, 220, 200], 1, true);
        }"""),
    ("""        u.text(fw - Ui::text_w(&extra, 1) - 18, ty + th - 10, &extra, [150, 180, 255], 1, true);""", """        u.text(fw - Ui::text_w(&extra, 1) - 18, ty + th - 10, &extra, [150, 180, 255], 1, true);
        if crate::pad::active() && n.active && sk.rank[i] > 0 {
            let on = (0..3).find(|&b| g.binds[b] == Some(n.sk)).map(|b| format!("  (ON {} NOW)", ["X", "Y", "B"][b])).unwrap_or_default();
            u.text_c(fw / 2, ty + th - 10, &format!("PRESS X, Y OR B TO PUT IT ON THAT BUTTON{}", on), [120, 200, 255], 1);
        }"""),
])
print("binds ok")
