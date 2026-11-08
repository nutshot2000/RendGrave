// --maps: write an overview picture of every area, for checking the generators.
// Cells are drawn at half size; red = enemies, yellow = chests, cyan = shrines,
// white = exits, blue = waypoint, green = start.
use crate::areagen::*;
use crate::body::Kind;
use crate::world::*;

pub fn dump(dir: &str) {
    std::fs::create_dir_all(dir).ok();
    let mut list = vec![AreaId::Ashford, AreaId::Pines, AreaId::Graveyard, AreaId::Fen, AreaId::Ravenhold];
    for f in 1..=CRYPT_FLOORS {
        list.push(AreaId::Crypt(f));
    }
    for f in 1..=CELLAR_FLOORS {
        list.push(AreaId::Cellars(f));
    }
    for id in list {
        let t0 = std::time::Instant::now();
        let (w, sp) = generate(id, 7);
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let (iw, ih) = (w.aw / 2, w.ah / 2);
        let mut img = vec![0u8; iw * ih * 3];
        for y in 0..ih {
            for x in 0..iw {
                let i = (y * 2) * W + x * 2;
                let (m, g) = (w.m[i], w.g[i]);
                let c: [u8; 3] = if g == STAIRS || g == STAIRSUP {
                    [240, 220, 140]
                } else {
                    match m {
                        TREE => if w.aux[i] & AUX_DEAD != 0 { [70, 66, 60] } else { [24, 62, 30] },
                        ROCK => [10, 9, 12],
                        GAS => [110, 150, 60],
                        STONE => [96, 94, 102],
                        BRICK => [140, 120, 128],
                        WOOD => [120, 86, 52],
                        CLOTH => [150, 60, 50],
                        RUBBLE => [100, 92, 90],
                        BONE => [210, 200, 180],
                        WATER | ICE => [40, 80, 150],
                        ACID => [100, 190, 50],
                        OIL => [30, 26, 26],
                        _ => match g {
                            FLOOR => [66, 62, 70],
                            PLANK => [130, 96, 60],
                            DIRT => [86, 70, 50],
                            MUD => [64, 52, 40],
                            PEAT => [52, 40, 30],
                            MOSS => [44, 76, 40],
                            _ => [40, 70, 38],
                        },
                    }
                };
                img[(y * iw + x) * 3..(y * iw + x) * 3 + 3].copy_from_slice(&c);
            }
        }
        let mut dot = |x: f32, y: f32, r: i32, c: [u8; 3]| {
            for dy in -r..=r {
                for dx in -r..=r {
                    let (px, py) = (x as i32 / 2 + dx, y as i32 / 2 + dy);
                    if px >= 0 && py >= 0 && (px as usize) < iw && (py as usize) < ih {
                        img[(py as usize * iw + px as usize) * 3..(py as usize * iw + px as usize) * 3 + 3].copy_from_slice(&c);
                    }
                }
            }
        };
        for e in &sp.exits {
            for x in (e.x0..e.x1).step_by(2) {
                for y in (e.y0..e.y1).step_by(2) {
                    dot(x as f32, y as f32, 0, [255, 255, 255]);
                }
            }
        }
        for e in &sp.enemies {
            let boss = matches!(e.0, Kind::King | Kind::Chief | Kind::Brute);
            dot(e.1, e.2, if boss { 3 } else { 1 }, if boss { [255, 0, 255] } else { [255, 40, 40] });
        }
        for c in &sp.chests {
            dot(c.0, c.1, 3, [255, 220, 40]);
        }
        for b in &sp.barrels {
            dot(b.0, b.1, 2, if b.2 == 0 { [255, 120, 0] } else if b.2 == 1 { [150, 100, 60] } else { [120, 255, 60] });
        }
        for s in &sp.shrines {
            dot(s.0, s.1, 4, [40, 240, 255]);
        }
        if let Some((x, y)) = sp.waypoint {
            dot(x, y, 4, [80, 120, 255]);
        }
        dot(sp.start.0, sp.start.1, 3, [40, 255, 40]);
        let path = format!("{}/{}.png", dir, id.key());
        let file = std::fs::File::create(&path).expect("map file");
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), iw as u32, ih as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().expect("png header");
        wr.write_image_data(&img).expect("png data");
        // expected items worth picking up (magic or better) from clearing the whole area
        let elite_share = if id == AreaId::Ashford { 0.0 } else { 0.07 };
        let mut items = 0.0;
        for e in &sp.enemies {
            for (el, share) in [(false, 1.0 - elite_share), (true, elite_share)] {
                let (c, r, luck) = crate::gear::loot_odds(e.0, el);
                items += share * c * r as f32 * (0.40 + luck).min(1.0);
            }
        }
        for c in &sp.chests {
            items += if c.2 > 0 { 1.35 } else { 0.4 * 0.5 };
        }
        let area = 0.6 + 0.2 * (id.level() - 1) as f32;
        let xp: f32 = sp.enemies.iter().map(|e| crate::areas::base_xp(e.0) * area * if matches!(e.0, Kind::King | Kind::Chief) { 1.0 } else { 1.0 + 2.0 * elite_share }).sum();
        println!("{:<12} xp {:>6.0}  lv {:>2}", id.key(), xp, id.level());
        println!("{:<12} {}x{}  {:>4.0} ms  enemies {:>3}  chests {}  shrines {}  exits {}  items ~{:.1}", id.key(), w.aw, w.ah, ms, sp.enemies.len(), sp.chests.len(), sp.shrines.len(), sp.exits.len(), items);
    }
}
