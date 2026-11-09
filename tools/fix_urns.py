def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


B = 'src/barrels.rs'
edit(B, [
    ("    /// 0 powder keg, 1 oil, 2 acid\n    pub k: u8,", "    /// 0 powder keg, 1 oil, 2 acid, 3 clay urn, 4 crate\n    pub k: u8,"),
    ("""        if bk == 0 {
            if self.barrels[i].fuse < 0 {
                self.barrels[i].fuse = 70;
                self.float(x, y - 30.0, "FUSE LIT - RUN".into(), [255, 170, 60], true);
            }
        } else {
            self.break_barrel(i, false);
        }
    }""", """        if bk == 0 {
            if self.barrels[i].fuse < 0 {
                self.barrels[i].fuse = 70;
                self.float(x, y - 30.0, "FUSE LIT - RUN".into(), [255, 170, 60], true);
            }
        } else if bk >= 3 {
            self.smash(i);
        } else {
            self.break_barrel(i, false);
        }
    }

    /// an urn or a crate breaks open: shards or splinters, and now and then something inside
    pub fn smash(&mut self, i: usize) {
        let (x, y, k) = (self.barrels[i].x, self.barrels[i].y, self.barrels[i].k);
        self.clear_barrel(i);
        let urn = k == 3;
        self.snd(if urn { Sfx::Crumble } else { Sfx::Wood }, x, y, 1.0);
        let col = if urn { [150, 96, 66] } else { [120, 84, 52] };
        for _ in 0..22 {
            self.ptc(x, y, rr(3.0, 12.0), rr(-2.2, 2.2), rr(-1.2, 1.2), rr(1.0, 3.5), 240, PtK::Debris, col);
        }
        if urn {
            // a puff of old dust
            for _ in 0..6 {
                self.ptc(x, y, 6.0, rr(-0.4, 0.4), rr(-0.2, 0.2), 0.3, 60, PtK::Smoke, [110, 100, 90]);
            }
        }
        let lv = self.alvl;
        let r = crate::rng::rnd();
        if r < 0.45 {
            let amt = (3 + ri(6) as i32) * (1 + lv / 2);
            self.pickups.push(Pickup { k: 0, x, y: y + 4.0, z: 8.0, vz: 2.2, amt, t: 0 });
        } else if r < 0.51 {
            self.pickups.push(Pickup { k: 1, x, y: y + 4.0, z: 8.0, vz: 2.2, amt: 1, t: 0 });
        } else if r < 0.525 {
            let item = crate::items::roll(lv, 0.05);
            self.ground.push(crate::gear::GroundItem { item, x, y: y + 6.0, z: 10.0, vz: 2.5, t: 0 });
        }
    }"""),
    ("""            } else if fire || wrecked {
                self.break_barrel(i, fire);
            }""", """            } else if b.k >= 3 {
                // urns only care about being knocked apart; crates also burn
                if wrecked || (fire && b.k == 4) {
                    self.smash(i);
                }
            } else if fire || wrecked {
                self.break_barrel(i, fire);
            }"""),
])

A = 'src/areagen.rs'
edit(A, [
    ("""            // barrels here and there: powder kegs, oil and acid
            if rnd() < 0.5 {""", """            // urns in the crypt, crates in the cellars, along a wall
            if rnd() < 0.6 {
                let n = 3 + ri(4);
                let (bx, by) = (r.x as f32 + rr(20.0, (r.w - 20 - n as i32 * 13).max(21) as f32), r.y as f32 + if rnd() < 0.5 { 16.0 } else { r.h as f32 - 10.0 });
                barrels(w, &mut sp, bx, by, n, if cellars { &[(4, 1.0)] } else { &[(3, 1.0)] });
            }
            // barrels here and there: powder kegs, oil and acid
            if rnd() < 0.5 {"""),
])
# outdoors: crates in the bandit yard, urns in the tombs
s = open(A, encoding='utf8').read()
a = "        building(w, &mut sp, x, y, 58, 46, 16, BKind::Tomb);\n"
assert s.count(a) == 1
s = s.replace(a, a + "        barrels(w, &mut sp, x as f32 + 10.0, y as f32 + 12.0, 2 + ri(2), &[(3, 1.0)]);\n")
a = "            barrels(w, &mut sp, x, y, 2 + ri(3), &[(0, 0.6), (1, 0.4)]);\n"
assert s.count(a) == 1
s = s.replace(a, "            barrels(w, &mut sp, x, y, 2 + ri(3), &[(0, 0.5), (1, 0.3), (4, 0.2)]);\n")
open(A, 'w', encoding='utf8').write(s)

R = 'src/render.rs'
edit(R, [
    ("""    for b in g.barrels.iter().filter(|b| b.alive) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);""", """    for b in g.barrels.iter().filter(|b| b.alive && b.k >= 3) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);
        if b.k == 3 {
            // a clay urn: round belly, narrow neck, a lip
            for k in 0..13i32 {
                let half = if k < 9 { (4.6 - ((k as f32 - 4.0) / 4.5).powi(2) * 2.2) as i32 } else if k < 11 { 2 } else { 3 };
                for dx in -half..=half {
                    let shade = 0.72 + (dx + half) as f32 / (2 * half).max(1) as f32 * 0.45;
                    let band = k == 6 && (dx + k) % 2 == 0;
                    let col = if band { [90, 60, 40] } else if k == 12 { [120, 80, 56] } else { [150, 98, 66] };
                    c.put(sx + dx, sy - k, b.y + 0.5, scl(col, shade), [0; 3], k as f32);
                }
            }
        } else {
            // a crate: planks with a cross brace
            for k in 0..10i32 {
                for dx in -5i32..=5 {
                    let edge = dx.abs() == 5 || k == 0 || k == 9;
                    let brace = (dx + 5) == k || (dx + 5) == 9 - k;
                    let col = if edge || brace { [86, 58, 36] } else if (k / 3) % 2 == 0 { [128, 92, 58] } else { [116, 82, 52] };
                    c.put(sx + dx, sy - k, b.y + 0.5, col, [0; 3], k as f32);
                }
            }
        }
    }
    for b in g.barrels.iter().filter(|b| b.alive && b.k < 3) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);"""),
])
edit('src/mapdump.rs', [("dot(b.0, b.1, 2, if b.2 == 0 { [255, 120, 0] } else if b.2 == 1 { [150, 100, 60] } else { [120, 255, 60] });", "dot(b.0, b.1, if b.2 >= 3 { 1 } else { 2 }, match b.2 { 0 => [255, 120, 0], 1 => [150, 100, 60], 2 => [120, 255, 60], _ => [190, 150, 110] });")])
print("urns ok")
