def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/body.rs', [
    ("""    /// boss of the Ravenhold bandits
    Chief,
}""", """    /// boss of the Ravenhold bandits
    Chief,
    /// a thief with a sack of loot who runs from you
    Robber,
}"""),
    ("        Kind::Bandit | Kind::Crossbow => return bandit(orig),", "        Kind::Bandit | Kind::Crossbow | Kind::Robber => return bandit(orig),"),
    ("""    b.parts[T_WEAPON] = if orig == Kind::Crossbow {""", """    if orig == Kind::Robber {
        // no weapon, a dark green hood and cloak
        b.parts[T_WEAPON].on = false;
        for (pi, p) in b.parts.iter_mut().enumerate() {
            if pi == T_WEAPON {
                continue;
            }
            for q in p.px.iter_mut() {
                if q.m == PM_CLOTH {
                    q.c = mul([52, 74, 50], 0.7 + (q.c[0] as f32 / 255.0) * 0.6);
                }
            }
        }
        b.kind = orig;
        b.compute();
        return b;
    }
    b.parts[T_WEAPON] = if orig == Kind::Crossbow {"""),
])
G = 'src/game.rs'
edit(G, [
    ("        Kind::Chief => (3800.0, 0.62, 28.0, 34.0, 34),", "        Kind::Chief => (3800.0, 0.62, 28.0, 34.0, 34),\n        Kind::Robber => (150.0, 1.18, 0.0, 0.0, 30),"),
    # the robber runs: override the chase
    ("""                        if e.k.bandit() && e.k != Kind::Chief && e.hp < e.maxhp * 0.25 && dist < 140.0 {""", """                        if e.k == Kind::Robber {
                            let l = dist.max(1.0);
                            want = (-dx / l * spd, -dy / l * spd);
                        }
                        if e.k.bandit() && e.k != Kind::Chief && e.hp < e.maxhp * 0.25 && dist < 140.0 {"""),
    ("""        self.champion_death(i);""", """        self.champion_death(i);
        if k == Kind::Robber {
            self.robber_loot(x, y);
        }"""),
    ("            self.champion_tick(i);\n", "            self.champion_tick(i);\n            self.robber_tick(i);\n"),
])
edit('src/areas.rs', [
    ("            Kind::Chief => 900.0,\n", "            Kind::Chief => 900.0,\n            Kind::Robber => 60.0,\n"),
    ("""        self.chests = sp.chests.iter()""", """        // now and then a grave robber is out on the prowl
        if !to.is_dungeon() && to != AreaId::Ashford && rnd() < 0.4 {
            for _ in 0..200 {
                let (rx, ry) = (rr(80.0, self.world.aw as f32 - 80.0), rr(80.0, self.world.ah as f32 - 80.0));
                let (ax, ay) = sp.arrival(from);
                let d = ((rx - ax).powi(2) + (ry - ay).powi(2)).sqrt();
                if d > 260.0 && d < 700.0 && self.world.at(rx as i32, ry as i32) == crate::world::E && !self.world.solid(rx as i32 + 3, ry as i32) && !self.world.solid(rx as i32 - 3, ry as i32) {
                    let mut r = self.scaled_enemy(Kind::Robber, rx, ry);
                    r.elite = false;
                    r.aff = 0;
                    r.skill_t = 25 * 60;
                    self.enemies.push(r);
                    break;
                }
            }
        }
        self.chests = sp.chests.iter()"""),
])
s = open('src/champions.rs', encoding='utf8').read()
s += '''
impl Game {
    /// the grave robber: once he's seen you he runs, and gets away if you don't catch him
    pub fn robber_tick(&mut self, i: usize) {
        if self.enemies[i].k != crate::body::Kind::Robber || !self.enemies[i].aggro {
            return;
        }
        let e = &mut self.enemies[i];
        let (x, y) = (e.x, e.y);
        if e.skill_t == 25 * 60 {
            self.float(x, y - 70.0, "A GRAVE ROBBER - CATCH HIM!".into(), [255, 214, 90], true);
            self.snd(Sfx::Shout, x, y, 1.0);
        }
        let e = &mut self.enemies[i];
        e.skill_t -= 1;
        if e.skill_t % 3 == 0 {
            // gold trickles from his sack as he runs
            self.ptc(x + rr(-3.0, 3.0), y, rr(20.0, 30.0), rr(-0.5, 0.5), rr(-0.3, 0.3), rr(0.5, 1.5), 40, PtK::Spark, [255, 210, 80]);
        }
        let e = &mut self.enemies[i];
        if e.skill_t <= 0 {
            e.alive = false;
            for _ in 0..30 {
                self.ptc(x, y, rr(0.0, 40.0), rr(-1.0, 1.0), rr(-0.5, 0.5), rr(0.2, 1.0), 80, PtK::Smoke, [80, 76, 70]);
            }
            self.float(x, y - 60.0, "HE GOT AWAY".into(), [200, 180, 160], true);
            self.snd(Sfx::Travel, x, y, 0.6);
        }
    }

    /// caught him: his sack bursts
    pub fn robber_loot(&mut self, x: f32, y: f32) {
        let lv = self.alvl;
        for k in 0..10 {
            let amt = (8 + ri(10) as i32) * (1 + lv / 2);
            self.pickups.push(Pickup { k: 0, x: x + rr(-16.0, 16.0), y: y + rr(-8.0, 8.0), z: 10.0, vz: rr(2.0, 3.5), amt, t: k * 3 });
        }
        let mut item = crate::items::roll(lv, 0.4);
        for _ in 0..20 {
            if item.rarity >= crate::items::Rarity::Rare {
                break;
            }
            item = crate::items::roll(lv, 0.4);
        }
        self.ground.push(crate::gear::GroundItem { item, x, y: y + 6.0, z: 12.0, vz: 3.0, t: 10 });
        self.banner = Some(("CAUGHT THE GRAVE ROBBER".into(), "HIS SACK BURSTS OPEN".into(), 150));
        self.snd(Sfx::Coin, x, y, 1.0);
    }
}
'''
open('src/champions.rs', 'w', encoding='utf8').write(s)
print("robber wired")
