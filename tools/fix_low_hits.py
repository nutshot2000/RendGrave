def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/game.rs', [
    ("""                let mut hit_e = None;
                for (i, e) in self.enemies.iter().enumerate() {
                    if !e.alive || (e.y - y).abs() > 7.0 || (e.x - x).abs() > 16.0 {
                        continue;
                    }
                    if e.body.near_pixel(x - e.x, -z, 1) {
                        hit_e = Some(i);
                        break;
                    }
                }
                let hit_e = if kind >= 2 { None } else { hit_e };""", """                // a shot crossing an enemy's row hits anything of it at or below its height,
                // so crawlers, the legless and the fallen can be shot too
                let mut hit_e = None;
                let mut hit_z = z;
                'e: for (i, e) in self.enemies.iter().enumerate() {
                    if !e.alive || (e.y - y).abs() > 7.0 || (e.x - x).abs() > 16.0 {
                        continue;
                    }
                    let mut hz = z;
                    while hz >= 1.0 {
                        if e.body.near_pixel(x - e.x, -hz, 1) {
                            hit_e = Some(i);
                            hit_z = hz;
                            break 'e;
                        }
                        hz -= 3.0;
                    }
                }
                let hit_e = if kind >= 2 { None } else { hit_e };
                // barrels stand low too
                let hit_b = if kind == 2 { None } else { self.barrels.iter().position(|b| b.alive && (b.x - x).abs() < 6.0 && (b.y - y).abs() < 4.5 && z < 40.0) };
                if hit_e.is_some() || hit_b.is_some() {
                    let hz = if hit_e.is_some() { hit_z } else { 8.0 };
                    self.projs[k].z = hz;
                }
                let z = self.projs[k].z;
                if let Some(b) = hit_b {
                    if kind != 0 {
                        self.strike_barrel(b);
                    }
                }
                let wall = wall || hit_b.is_some();"""),
])
edit('src/barrels.rs', [
    ("""            let (x, y, bk) = (b.x, b.y, b.k);
            self.snd(Sfx::Wood, x, y, 0.8);
            if bk == 0 {
                if self.barrels[i].fuse < 0 {
                    self.barrels[i].fuse = 70;
                    self.float(x, y - 30.0, "FUSE LIT - RUN".into(), [255, 170, 60], true);
                }
            } else {
                self.break_barrel(i, false);
            }
        }
    }""", """            self.strike_barrel(i);
        }
    }

    /// something hit a barrel: a keg's fuse catches, oil and acid barrels split
    pub fn strike_barrel(&mut self, i: usize) {
        let (x, y, bk) = (self.barrels[i].x, self.barrels[i].y, self.barrels[i].k);
        if !self.barrels[i].alive {
            return;
        }
        self.snd(Sfx::Wood, x, y, 0.8);
        if bk == 0 {
            if self.barrels[i].fuse < 0 {
                self.barrels[i].fuse = 70;
                self.float(x, y - 30.0, "FUSE LIT - RUN".into(), [255, 170, 60], true);
            }
        } else {
            self.break_barrel(i, false);
        }
    }"""),
])
print("low hits ok")
