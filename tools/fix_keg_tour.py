p = 'src/main.rs'
s = open(p, encoding='utf8').read()
a = """        if f == end + 50 {
            g.wp_open = true;
            return Some("t7_waypoints".into());
        }
        None
    }"""
b = """        if f == end + 50 {
            g.wp_open = true;
            return Some("t10_waypoints".into());
        }
        // a powder keg in the cellars with a crowd of bandits round it
        let kb = end + 60;
        if f == kb {
            g.wp_open = false;
            g.travel(AreaId::Cellars(1));
            if let Some(b) = g.barrels.iter().position(|b| b.k == 0) {
                let (bx, by) = (g.barrels[b].x, g.barrels[b].y);
                let mut n = 0;
                for e in g.enemies.iter_mut() {
                    if n < 7 && !e.k.boss() {
                        let a = n as f32 / 7.0 * std::f32::consts::TAU;
                        e.x = bx + a.cos() * 26.0;
                        e.y = by + 12.0 + a.sin() * 12.0;
                        e.stun = 100000;
                        n += 1;
                    } else {
                        e.stun = 100000;
                    }
                }
                // a slick of oil nearby, and an oil barrel next to the keg
                g.player.x = bx - 110.0;
                g.player.y = by + 20.0;
                g.cam = (bx - g.view.0 / 2.0, by - 26.0 - g.view.1 / 2.0);
            }
        }
        if f == kb + 30 {
            return Some("t11_keg_before".into());
        }
        if f == kb + 31 {
            if let Some(b) = g.barrels.iter_mut().find(|b| b.k == 0 && b.alive) {
                b.fuse = 2;
            }
        }
        if f == kb + 36 {
            return Some("t12_keg_blast".into());
        }
        if f == kb + 60 {
            return Some("t13_keg_after".into());
        }
        if f == kb + 120 {
            return Some("t14_keg_aftermath".into());
        }
        None
    }"""
assert s.count(a) == 1
s = s.replace(a, b)
open(p, 'w', encoding='utf8').write(s)
print("keg tour ok")
