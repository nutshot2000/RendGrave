# Starting gift, oil flasks, respec (slice step 4).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("mod areas;\n", "mod areas;\nmod agency;\n"),
    ("                        KeyCode::KeyF => i.potion = down,\n", "                        KeyCode::KeyF => i.potion = down,\n                        KeyCode::KeyQ => i.throw = down,\n"),
])

G = 'src/game.rs'
edit(G, [
    ("    pub inv: bool,\n}", "    pub inv: bool,\n    /// Q: throw an oil flask\n    pub throw: bool,\n}"),
    ("    pub loose_bows: Vec<crate::anatomy::LooseBow>,\n", "    pub loose_bows: Vec<crate::anatomy::LooseBow>,\n    /// the starting gift (0 not chosen yet, 1 spark, 2 oil flasks, 3 weapon) and flasks left\n    pub pick: u8,\n    pub flasks: i32,\n    /// a choice box on screen (the starting gift, a respec)\n    pub choice: Option<crate::agency::Choice>,\n"),
    ("            loose_bows: vec![],\n", "            loose_bows: vec![],\n            pick: 0,\n            flasks: 0,\n            choice: None,\n"),
    ("""        if self.talk.is_some() {
            self.update_talk(inp);
            self.prev = inp.clone();
            return;
        }""", """        if self.choice.is_some() {
            self.update_choice(inp);
            self.prev = inp.clone();
            return;
        }
        if self.talk.is_some() {
            self.update_talk(inp);
            self.prev = inp.clone();
            return;
        }"""),
    ("""            self.revive();
            return;
        }
        let (mwx, mwy) = self.mouse_world(inp);""", """            self.revive();
            return;
        }
        if inp.throw && !self.prev.throw && !self.player.dead {
            self.throw_flask(inp);
        }
        let (mwx, mwy) = self.mouse_world(inp);"""),
    ("                if p.k == 1 || p.k == 5 {\n", "                if p.k == 1 || p.k == 5 || p.k == 6 {\n"),
    ("                        5 => self.drop_keg(x - self.projs[k].vx * 2.0, y - self.projs[k].vy * 2.0),\n", "                        5 => self.drop_keg(x - self.projs[k].vx * 2.0, y - self.projs[k].vy * 2.0),\n                        6 => self.flask_break(x, y),\n"),
])

edit('src/story.rs', [
    ("""        let told_chief = self.has(TOLD_CHIEF);
        let Some((pages, page)) = &mut self.talk else { return };
        if close || (next && *page + 1 >= pages.len()) {""", """        let told_chief = self.has(TOLD_CHIEF);
        let Some((pages, page)) = &mut self.talk else { return };
        if close || (next && *page + 1 >= pages.len()) {
            let generic = pages.len() == 1;"""),
    ("""            if was_chief {
                self.toast = Some(("+300 GOLD".into(), 160));
                self.check_act_end();
            }""", """            if was_chief {
                self.toast = Some(("+300 GOLD".into(), 160));
                self.check_act_end();
            }
            // after the first talk, a gift for the road; later, the offer of a respec
            if self.pick == 0 && self.has(MET) {
                self.offer_start();
            } else if generic && self.pick != 0 {
                self.offer_respec();
            }"""),
])

A = 'src/areas.rs'
edit(A, [
    ("""        if to == AreaId::Ashford {
            self.restock();
        }""", """        if to == AreaId::Ashford {
            self.restock();
            if self.pick == 2 {
                self.flasks = crate::agency::FLASKS;
            }
        }"""),
    ('            format!("tips {}", self.tips),\n', '            format!("tips {}", self.tips),\n            format!("pick {}", self.pick),\n            format!("flasks {}", self.flasks),\n'),
    ('                "tips" => self.tips = v.parse().unwrap_or(0),', '                "tips" => self.tips = v.parse().unwrap_or(0),\n                "pick" => self.pick = v.parse().unwrap_or(0),\n                "flasks" => self.flasks = v.parse().unwrap_or(0),'),
])

R = 'src/render.rs'
edit(R, [("""        if p.k == 3 || p.k == 4 {
            // arrow: shaft along its flight, grey head""", """        if p.k == 6 {
            // a thrown oil flask: dark glass, a cork
            for dy in 0..4i32 {
                for dx in -1i32..=1 {
                    c.put(sx + dx, sy - dy, p.y + 0.5, if dx == 1 { [90, 110, 120] } else { [40, 46, 52] }, [0; 3], p.z + dy as f32);
                }
            }
            c.put(sx, sy - 4, p.y + 0.5, [150, 110, 70], [0; 3], p.z + 4.0);
            continue;
        }
        if p.k == 3 || p.k == 4 {
            // arrow: shaft along its flight, grey head""")])

H = 'src/hud.rs'
edit(H, [
    ("""    u.text(qx + 5, qy + 2, &format!("{} F", p.potions), [255, 200, 200], 1, true);""", """    u.text(qx + 5, qy + 2, &format!("{} F", p.potions), [255, 200, 200], 1, true);
    if g.pick == 2 {
        // oil flasks
        let fx = qx + 26;
        for dy in 0..6 {
            for dx in -1i32..=1 {
                u.px(fx + dx, qy + 2 + dy, if dy == 0 { [150, 110, 70] } else if dx == 1 { [100, 120, 130] } else { [50, 56, 64] }, 255);
            }
        }
        u.text(fx + 4, qy + 2, &format!("{} Q", g.flasks), [200, 210, 220], 1, true);
    }"""),
    ("""    draw_talk(g, &mut u, fw, fh);""", """    draw_talk(g, &mut u, fw, fh);
    draw_choice(g, &mut u, fw, fh);"""),
    ("""fn draw_talk(g: &Game, u: &mut Ui, fw: i32, fh: i32) {""", """/// the boxes of a choice: the starting gift, the respec
pub fn choice_box(k: usize, n: usize, fw: i32, fh: i32) -> (i32, i32, i32, i32) {
    let w = ((fw - 40) / n as i32 - 8).min(170);
    let total = n as i32 * (w + 8) - 8;
    let x0 = fw / 2 - total / 2;
    (x0 + k as i32 * (w + 8), fh / 2 - 30, w, 74)
}
pub fn choice_at(m: (f32, f32), view: (f32, f32), n: usize) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    (0..n).find(|&k| {
        let (x, y, w, h) = choice_box(k, n, fw, fh);
        let (mx, my) = (m.0 as i32, m.1 as i32);
        mx >= x && mx < x + w && my >= y && my < y + h
    })
}
fn draw_choice(g: &Game, u: &mut Ui, fw: i32, fh: i32) {
    let Some(c) = &g.choice else { return };
    u.rect(0, 0, fw, fh, [6, 3, 8], 150);
    u.text_c(fw / 2, fh / 2 - 50, &c.title, [240, 214, 150], 1);
    let n = c.opts.len();
    for (k, (name, desc)) in c.opts.iter().enumerate() {
        let (x, y, w, h) = choice_box(k, n, fw, fh);
        let hov = c.hover == Some(k);
        u.rect(x, y, w, h, if hov { [60, 36, 28] } else { [20, 13, 18] }, 240);
        u.rect(x, y, w, 1, if hov { [255, 190, 110] } else { [90, 70, 60] }, 255);
        u.text_c(x + w / 2, y + 6, &format!("{}  {}", k + 1, name), if hov { [255, 236, 200] } else { [240, 214, 150] }, 1);
        for (j, l) in wrap(desc, ((w - 10) / 4) as usize).iter().enumerate() {
            u.text(x + 6, y + 20 + j as i32 * 9, l, [200, 190, 190], 1, false);
        }
    }
    let foot = if c.kind == crate::agency::ChoiceKind::Start { "CLICK ONE, OR PRESS 1, 2 OR 3" } else { "CLICK ONE, OR ESC" };
    u.text_c(fw / 2, fh / 2 + 52, foot, [150, 140, 150], 1);
}

fn draw_talk(g: &Game, u: &mut Ui, fw: i32, fh: i32) {"""),
    ("""    let rows: [(&str, &str); 16] = [""", """    let rows: [(&str, &str); 17] = ["""),
    ("""        ("F", "DRINK A POTION"),""", """        ("F", "DRINK A POTION"),
        ("Q", "THROW AN OIL FLASK (IF YOU HAVE THEM)"),"""),
])
print("agency ok")
