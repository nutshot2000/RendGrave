def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


R = 'src/render.rs'
edit(R, [
    ("use crate::world::*;", "use crate::world::*;\nuse crate::areagen::{BKind, Building};"),
    # plank floors in houses
    ("""        STAIRS | STAIRSUP => {""", """        PLANK => {
            let board = (y.rem_euclid(5) == 0) as i32 as f32;
            let seam = ((x + (y / 5) * 13).rem_euclid(23) == 0) as i32 as f32;
            scl([104, 74, 50], 0.8 + h2(x / 23, y / 5, 4) * 0.3 + s * 0.08 - board * 0.3 - seam * 0.2)
        }
        STAIRS | STAIRSUP => {"""),
    # wall top faces by style
    ("""                BRICK => {
                    let edge = north != BRICK;
                    scl([96, 88, 100], 0.85 + s * 0.15 + if edge { 0.25 } else { 0.0 })
                }
                ROCK =>""", """                BRICK => {
                    let edge = north != BRICK;
                    let aux = w.aux[i];
                    let base = if aux & AUX_HOUSE != 0 {
                        [92, 66, 46]
                    } else if aux & AUX_TOMB != 0 {
                        [132, 128, 134]
                    } else {
                        [96, 88, 100]
                    };
                    scl(base, 0.85 + s * 0.15 + if edge { 0.25 } else { 0.0 })
                }
                ROCK =>"""),
    ("""                        BRICK => {
                            let row = k / 4;
                            let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                            let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                            scl(base, 1.0 - k as f32 / h as f32 * 0.35)
                        }""", """                        BRICK => {
                            let st = style_of(w.aux[i]);
                            if st == 0 {
                                let row = k / 4;
                                let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                                let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                                scl(base, 1.0 - k as f32 / h as f32 * 0.35)
                            } else {
                                wall_px(st, wx, h - k, s)
                            }
                        }"""),
    # buildings drawn after the walls
    ("""    // ---- braziers: little fire pits on metal stands""", """    // ---- roofs, doorways, windows, pillars
    let pl = if g.player.dead { None } else { Some((g.player.x, g.player.y)) };
    for b in &g.buildings {
        draw_building(&mut c, b, pl);
    }
    // ---- braziers: little fire pits on metal stands"""),
    # lamps inside buildings
    ("""    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;
        add(bx, by + 1.0, 120.0, 1.4 * fl, [1.0, 0.55, 0.22]);
    }""", """    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;
        add(bx, by + 1.0, 120.0, 1.4 * fl, [1.0, 0.55, 0.22]);
    }
    for &(lx, ly, lc) in &g.lamps {
        let fl = 1.0 + (t as f32 * 0.17 + lx).sin() * 0.06;
        add(lx, ly, 90.0, 1.3 * fl, lc);
    }"""),
    # the painters themselves
    ("""/// One cell of a tree.""", r'''/// 0 plain brick, 1 timber-framed house, 2 stone tomb
fn style_of(aux: u8) -> u8 {
    if aux & AUX_HOUSE != 0 {
        1
    } else if aux & AUX_TOMB != 0 {
        2
    } else {
        0
    }
}

/// one pixel of a building's front wall at column x, height z above the ground
fn wall_px(style: u8, x: i32, z: i32, s: f32) -> [u8; 3] {
    match style {
        1 => {
            // timber frame: dark beams over warm plaster, a stone footing
            if z < 4 {
                let joint = z == 0 || (x + z / 2) % 9 == 0;
                return if joint { [62, 58, 62] } else { scl([112, 108, 112], 0.85 + s * 0.2) };
            }
            let post = x.rem_euclid(14) < 2;
            let rail = (z - 4).rem_euclid(19) < 2 || z >= 42;
            let px = x.rem_euclid(14) as i32;
            let pz = (z - 4).rem_euclid(19);
            let brace = (px - 2 - pz * 11 / 17).abs() < 1 && (x / 14) % 2 == 0;
            if post || rail || brace {
                scl([66, 44, 30], 0.85 + h2(x, z, 3) * 0.25)
            } else {
                let n = vnoise(x as f32 * 0.3, z as f32 * 0.3, 21);
                scl([186, 166, 132], 0.78 + n * 0.25 + s * 0.06 - (z as f32 / 44.0) * 0.08)
            }
        }
        _ => {
            // big weathered stone blocks with moss creeping up from the ground
            let row = z / 7;
            let off = if row % 2 == 0 { 0 } else { 7 };
            let joint = z.rem_euclid(7) == 0 || (x + off).rem_euclid(14) == 0;
            let mut c = if joint { [70, 66, 74] } else { scl([124, 120, 126], 0.78 + h2((x + off) / 14, row, 5) * 0.3 + s * 0.08) };
            let moss = vnoise(x as f32 * 0.2, z as f32 * 0.25, 33);
            if z < 14 && moss > 0.55 + z as f32 * 0.02 {
                c = mixc(c, [58, 84, 46], 0.7);
            }
            c
        }
    }
}

/// Roof, doorway, windows and trim for one building. Pixels are depth-tested against
/// the building's south edge so things in front draw over it and things behind are hidden.
fn draw_building(c: &mut Ctx, b: &Building, player: Option<(f32, f32)>) {
    let wh = wall_h(BRICK);
    let (x0, y0, bw, bh) = (b.x0, b.y0, b.w, b.h);
    let gy = y0 + bh - 1; // ground row of the front wall
    let depth = gy as f32;
    let cy = c.cy;
    let cx = c.cx;
    let t = c.t;
    let dc = b.door_c();
    let half = b.door / 2;
    let style = if b.k == BKind::House { 1 } else { 2 };
    // --- wall above the doorway, with a lintel or an arch
    for x in dc - half..dc + half {
        let dx = (x - dc) as f32 + 0.5;
        let dh = if style == 1 {
            30
        } else {
            25 + ((half as f32 * half as f32 - dx * dx).max(0.0).sqrt() * 0.6) as i32
        };
        for z in dh..=wh {
            let mut col = wall_px(style, x, z, 0.5);
            if style == 1 && z < dh + 3 {
                col = [72, 48, 32];
            } else if style == 2 && z < dh + 3 {
                col = scl([160, 156, 162], 0.9 + h2(x, z, 9) * 0.15);
            }
            c.put(x - cx, gy - z - cy, depth, col, [0; 3], z as f32);
        }
        for wy in gy - 3..=gy {
            c.put(x - cx, wy - wh - cy, wy as f32, if style == 1 { [92, 66, 46] } else { [132, 128, 134] }, [0; 3], (wh + 1) as f32);
        }
    }
    // --- windows (houses) with warm light
    if style == 1 {
        let lit = b.look > 0.2;
        for wx0 in [x0 + bw / 5, x0 + bw * 4 / 5 - 8] {
            for z in 12..=27 {
                for x in wx0 - 1..wx0 + 9 {
                    let frame = z == 12 || z == 27 || x == wx0 - 1 || x == wx0 + 8;
                    let mull = x == wx0 + 3 || x == wx0 + 4 || z == 20;
                    let (col, e) = if frame {
                        ([58, 40, 28], [0; 3])
                    } else if mull {
                        ([70, 50, 34], [0; 3])
                    } else if lit {
                        let fl = 0.85 + (t as f32 * 0.07 + b.look * 9.0).sin() * 0.1;
                        ([255, 206, 130], scl([230, 150, 70], fl))
                    } else {
                        ([30, 34, 48], [0; 3])
                    };
                    c.put(x - cx, gy - z - cy, depth + 0.1, col, e, z as f32);
                }
            }
            // sill
            for x in wx0 - 2..wx0 + 10 {
                c.put(x - cx, gy - 11 - cy, depth + 0.1, [96, 72, 52], [0; 3], 11.0);
            }
        }
    }
    // --- pillars and a carved slab by tomb doors
    if style == 2 {
        for px0 in [dc - half - 5, dc + half + 1] {
            for z in 0..=40 {
                let cap = z >= 37 || z <= 2;
                let (a, bb) = if cap { (px0 - 1, px0 + 5) } else { (px0, px0 + 4) };
                for x in a..bb {
                    let shade = 1.15 - (x - a) as f32 * 0.12;
                    let col = scl([166, 162, 168], shade * if cap { 1.05 } else { 0.95 });
                    c.put(x - cx, gy - z - cy, depth + 1.5, col, [0; 3], z as f32);
                }
            }
        }
        let rune = b.k == BKind::CryptGate;
        for z in 33..=37 {
            for x in dc - 6..dc + 6 {
                let carve = (x + z) % 3 == 0 && z > 33 && z < 37;
                let (col, e) = if rune && carve {
                    let p = 0.6 + (t as f32 * 0.05).sin() * 0.4;
                    ([140, 190, 255], scl([60, 110, 255], p))
                } else if carve {
                    ([60, 56, 62], [0; 3])
                } else {
                    ([96, 92, 100], [0; 3])
                };
                c.put(x - cx, gy - z - cy, depth + 0.2, col, e, z as f32);
            }
        }
    }
    // --- the roof (hidden while you're inside)
    if b.k == BKind::Ruin {
        return;
    }
    if let Some((px, py)) = player {
        if b.inside(px, py) {
            return;
        }
    }
    let ov = 3;
    let pitch = match b.k {
        BKind::House => 0.72,
        BKind::CryptGate => 0.6,
        _ => 0.5,
    };
    let r = bh as f32 * pitch;
    let ym = y0 as f32 + bh as f32 / 2.0;
    let ridge = ym - wh as f32 - r;
    let north = (y0 - ov - wh) as f32;
    let south = (y0 + bh + ov - wh) as f32;
    let slate = b.look > 0.55;
    let base: [u8; 3] = match b.k {
        BKind::House if slate => [76, 80, 96],
        BKind::House => [118, 76, 52],
        BKind::CryptGate => [96, 92, 104],
        _ => [130, 126, 134],
    };
    let rows = ((south - ridge) / if style == 1 { 3.5 } else { 6.0 }).max(2.0);
    let top = north.min(ridge).floor() as i32;
    for x in x0 - ov..x0 + bw + ov {
        let edge_x = x == x0 - ov || x == x0 + bw + ov - 1;
        for sy in top..=south as i32 {
            let syf = sy as f32;
            let (mut col, dy) = if syf < ridge {
                // north slope, facing away from us
                if north >= ridge {
                    continue;
                }
                let tn = (ridge - syf) / (ridge - north);
                (scl(base, 0.5 - tn * 0.1), wh as f32 + r * (1.0 - tn))
            } else {
                let ts = (syf - ridge) / (south - ridge).max(1.0);
                let rowf = ts * rows;
                let row = rowf as i32;
                let frac = rowf - row as f32;
                let mut l = 1.05 - ts * 0.18 + (h2(x / 6 + row * 3, row, 7) - 0.5) * 0.18;
                if style == 1 {
                    if frac > 0.72 {
                        l *= 0.62;
                    }
                    if (x + row * 3).rem_euclid(7) == 0 {
                        l *= 0.75;
                    }
                } else {
                    if frac > 0.85 {
                        l *= 0.7;
                    }
                    if (x + row * 7).rem_euclid(15) == 0 {
                        l *= 0.75;
                    }
                }
                let mut col = scl(base, l);
                if style == 2 && vnoise(x as f32 * 0.15, syf * 0.2, 44) > 0.68 {
                    col = mixc(col, [64, 92, 52], 0.6);
                }
                (col, wh as f32 + r * (1.0 - ts))
            };
            if (syf - ridge).abs() < 1.2 {
                col = scl(base, 0.72);
            }
            if sy >= south as i32 - 1 || edge_x {
                col = scl(col, 0.62);
            }
            c.put_w(x - cx, sy - cy, depth + 0.6, col, [0; 3], dy);
        }
    }
    // eave shadow on the wall below
    for x in x0..x0 + bw {
        for k in 1..4 {
            c.blend(x - cx, south as i32 + k - cy, depth + 0.6, [0, 0, 0], 0.35 - k as f32 * 0.08);
        }
    }
    // chimney
    if let Some((chx, _)) = b.chimney() {
        let base_y = ridge as i32 + 3;
        for sy in base_y - 14..=base_y {
            for x in chx..chx + 6 {
                let cap = sy < base_y - 12;
                let mortar = (sy - base_y).rem_euclid(3) == 0 || (x + (sy / 3) * 2).rem_euclid(4) == 0;
                let col = if cap { [70, 66, 70] } else if mortar { [70, 50, 44] } else { scl([136, 82, 64], 1.1 - (x - chx) as f32 * 0.07) };
                c.put_w(x - cx, sy - cy, depth + 0.7, col, [0; 3], (wh as f32 + r + 6.0).min(127.0));
            }
        }
    }
    // a cross on tomb ridges, finials on the crypt gate
    if style == 2 {
        let col = [150, 146, 154];
        let rx = x0 + bw / 2;
        let ry = ridge as i32;
        for k in 0..9 {
            c.put_w(rx - cx, ry - k - cy, depth + 0.7, col, [0; 3], wh as f32 + r);
        }
        for dx in -3..=3 {
            c.put_w(rx + dx - cx, ry - 6 - cy, depth + 0.7, col, [0; 3], wh as f32 + r);
        }
        if b.k == BKind::CryptGate {
            for fx in [x0 - ov + 2, x0 + bw + ov - 3] {
                for k in 0..6 {
                    c.put_w(fx - cx, ry - k - cy, depth + 0.7, [120, 116, 124], [0; 3], wh as f32 + r);
                }
            }
        }
    }
}

/// One cell of a tree.'''),
])
print("building render ok")
