// ------------------------------------------------------------------ dungeons
struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cave: bool,
}
impl Room {
    fn c(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

/// a dungeon floor: the Sunken Crypt, or (cellars) the vaults under Ravenhold.
/// Big open halls joined by wide passages, so fights have room and you can see them.
fn dungeon(w: &mut World, floor: u8, cellars: bool) -> Spawns {
    let mut sp = Spawns::new();
    let (aw, ah) = (w.aw as i32, w.ah as i32);
    for y in 0..w.ah {
        for x in 0..w.aw {
            let i = y * W + x;
            w.m[i] = ROCK;
            w.g[i] = FLOOR;
            w.aux[i] = AUX_INDOOR;
        }
    }
    let last = if cellars { CELLAR_FLOORS } else { CRYPT_FLOORS };
    let boss = floor == last;
    let mut rooms: Vec<Room> = vec![];
    let target = 10 + ri(3);
    // boss arena first, at the far end
    if boss {
        rooms.push(Room { x: aw - 420 + ri(60) as i32, y: rr(140.0, ah as f32 - 400.0) as i32, w: 330, h: 250, cave: false });
    }
    for _ in 0..3000 {
        if rooms.len() >= target {
            break;
        }
        let (rw, rh) = (rr(150.0, 290.0) as i32, rr(110.0, 200.0) as i32);
        let (x, y) = (rr(40.0, (aw - rw - 40) as f32) as i32, rr(40.0, (ah - rh - 40) as f32) as i32);
        if rooms.iter().any(|r| x < r.x + r.w + 46 && x + rw + 46 > r.x && y < r.y + r.h + 46 && y + rh + 46 > r.y) {
            continue;
        }
        rooms.push(Room { x, y, w: rw, h: rh, cave: !cellars && rnd() < 0.35 });
    }
    // order rooms into a winding chain from the far side to the boss / stairs down
    if boss {
        let b = rooms.remove(0);
        rooms.sort_by_key(|r| r.c().0);
        rooms.push(b);
    } else {
        rooms.sort_by_key(|r| r.c().0 + r.c().1 / 3);
    }
    let ok = |x: i32, y: i32| x >= 8 && y >= 8 && x < aw - 8 && y < ah - 8;
    let carve = |w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32| {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                if ok(x, y) {
                    w.m[idx(x, y)] = E;
                }
            }
        }
    };
    let disc = |w: &mut World, cx: i32, cy: i32, r: i32| {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r && ok(cx + dx, cy + dy) {
                    w.m[idx(cx + dx, cy + dy)] = E;
                }
            }
        }
    };
    let sd = (next() & 0xffff) as u32;
    for r in &rooms {
        if r.cave {
            // a natural cavern: a lumpy oval
            let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
            for y in r.y - 10..r.y + r.h + 10 {
                for x in r.x - 10..r.x + r.w + 10 {
                    let (nx, ny) = ((x as f32 - cx) / (r.w as f32 / 2.0), (y as f32 - cy) / (r.h as f32 / 2.0));
                    let lim = 1.0 - 0.3 * vnoise(x as f32 * 0.04, y as f32 * 0.04, sd);
                    if nx * nx + ny * ny < lim && ok(x, y) {
                        w.m[idx(x, y)] = E;
                    }
                }
            }
        } else {
            carve(w, r.x, r.y, r.x + r.w, r.y + r.h);
        }
    }
    let corridor = |w: &mut World, a: (i32, i32), b: (i32, i32)| {
        let hw = 16 + ri(6) as i32;
        if rnd() < 0.5 {
            carve(w, a.0, a.1 - hw, b.0, a.1 + hw);
            carve(w, b.0 - hw, a.1, b.0 + hw, b.1);
            disc(w, b.0, a.1, hw + 8);
        } else {
            carve(w, a.0 - hw, a.1, a.0 + hw, b.1);
            carve(w, a.0, b.1 - hw, b.0, b.1 + hw);
            disc(w, a.0, b.1, hw + 8);
        }
    };
    for k in 0..rooms.len() - 1 {
        corridor(w, rooms[k].c(), rooms[k + 1].c());
    }
    for _ in 0..3 {
        let (a, b) = (ri(rooms.len() - 1), ri(rooms.len() - 1));
        if a != b {
            corridor(w, rooms[a].c(), rooms[b].c());
        }
    }
    // line the carved spaces with brick: paint a ring round every open cell that touches rock
    let snapshot = w.m.clone();
    for y in 5..ah - 5 {
        for x in 5..aw - 5 {
            let i = idx(x, y);
            if snapshot[i] != E {
                continue;
            }
            if snapshot[i - 1] == E && snapshot[i + 1] == E && snapshot[i - W] == E && snapshot[i + W] == E {
                continue;
            }
            for dy in -4i32..=4 {
                for dx in -4i32..=4 {
                    let j = idx(x + dx, y + dy);
                    if snapshot[j] == ROCK {
                        w.m[j] = BRICK;
                    }
                }
            }
        }
    }
    // dress the rooms
    let n = rooms.len();
    let kegs: &[(u8, f32)] = if cellars { &[(0, 0.5), (1, 0.4), (2, 0.1)] } else { &[(0, 0.25), (1, 0.5), (2, 0.25)] };
    for (k, r) in rooms.iter().enumerate() {
        let (cx, cy) = r.c();
        let ordinary = k > 0 && k < n - 1 || (!boss && k == n - 1);
        if ordinary {
            if cellars && rnd() < 0.5 {
                // a storeroom: plank floor, crates along the walls
                w.ground_rect(r.x, r.y, r.x + r.w, r.y + r.h, PLANK, AUX_INDOOR);
                for _ in 0..(r.w * r.h / 2500) {
                    let side = rnd() < 0.5;
                    let (bx, by) = if side {
                        (r.x + 8 + ri((r.w - 24).max(1) as usize) as i32, if rnd() < 0.5 { r.y + 8 } else { r.y + r.h - 14 })
                    } else {
                        (if rnd() < 0.5 { r.x + 8 } else { r.x + r.w - 20 }, r.y + 8 + ri((r.h - 20).max(1) as usize) as i32)
                    };
                    stack(w, bx, by, 8 + ri(5) as i32, 6 + ri(3) as i32);
                }
            } else if !r.cave && r.w > 190 && r.h > 140 && rnd() < 0.6 {
                // a pillared hall
                let (nx, ny) = (3, 2);
                for gy in 0..ny {
                    for gx in 0..nx {
                        let px = r.x + (gx + 1) * r.w / (nx + 1) - 6;
                        let py = r.y + (gy + 1) * r.h / (ny + 1) - 6;
                        w.rect(px, py, px + 12, py + 12, BRICK);
                    }
                }
            }
            if !cellars && !r.cave && rnd() < 0.3 {
                // coffins along the north wall
                for c in 0..4 {
                    let x = r.x + 14 + c * 20;
                    if x + 8 < r.x + r.w - 10 {
                        w.rect(x, r.y + 8, x + 8, r.y + 24, WOOD);
                    }
                }
            }
            if rnd() < 0.3 {
                w.splash(cx as f32 + rr(-30.0, 30.0), cy as f32 + rr(-20.0, 20.0), OIL, rr(7.0, 12.0), 0);
            }
            if !cellars && rnd() < 0.18 {
                let (ax, ay) = (cx + rr(-30.0, 30.0) as i32, cy + rr(-15.0, 15.0) as i32);
                for dy in -11i32..=11 {
                    for dx in -16i32..=16 {
                        if (dx * dx) as f32 / 256.0 + (dy * dy) as f32 / 121.0 < 1.0 && w.at(ax + dx, ay + dy) == E {
                            w.set(ax + dx, ay + dy, ACID);
                        }
                    }
                }
            }
            for _ in 0..(r.w * r.h / 500) {
                let (bx, by) = (r.x + ri(r.w as usize) as i32, r.y + ri(r.h as usize) as i32);
                if w.at(bx, by) == E {
                    let i = idx(bx, by);
                    if !cellars && rnd() < 0.4 {
                        w.m[i] = BONE;
                        w.col[i] = [196, 188, 166];
                    } else {
                        w.stain[i] = w.stain[i].saturating_add(90);
                    }
                }
            }
            // barrels here and there: powder kegs, oil and acid
            if rnd() < 0.5 {
                let (bx, by) = (r.x as f32 + rr(30.0, (r.w - 60).max(31) as f32), r.y as f32 + if rnd() < 0.5 { 20.0 } else { r.h as f32 - 12.0 });
                barrels(w, &mut sp, bx, by, 2 + ri(3), kegs);
            }
            // treasure in some rooms, a shrine now and then
            if rnd() < 0.25 {
                let (tx, ty) = (r.x as f32 + rr(20.0, r.w as f32 - 20.0), r.y as f32 + 18.0);
                if open_spot(w, tx, ty, 6) {
                    sp.chests.push((tx, ty, if rnd() < 0.3 { 1 } else { 0 }));
                }
            } else if rnd() < 0.1 && sp.shrines.is_empty() && open_spot(w, cx as f32, cy as f32, 12) {
                sp.shrines.push((cx as f32, cy as f32, ri(4) as u8));
            }
        }
        // light: braziers round the room
        let spots = [(r.x + 14, r.y + 16), (r.x + r.w - 14, r.y + 16), (r.x + 14, r.y + r.h - 10), (r.x + r.w - 14, r.y + r.h - 10)];
        let want = if r.w > 180 { 4 } else { 2 };
        for &(bx, by) in spots.iter().take(want) {
            if open_spot(w, bx as f32, by as f32, 3) {
                sp.braziers.push((bx as f32, by as f32));
            }
        }
        // enemies: bigger rooms, bigger packs
        if k > 0 && !(boss && k == n - 1) {
            let cnt = 3 + ri(3) + if floor > 1 { 1 } else { 0 } + (r.w * r.h / 20000) as usize;
            let mix: &[(Kind, f32)] = if cellars {
                &[(Kind::Bandit, 0.55), (Kind::Crossbow, 0.35), (Kind::Zombie, 0.1)]
            } else {
                &[(Kind::Skeleton, 0.45), (Kind::Archer, 0.2), (Kind::Zombie, 0.2), (Kind::Crawler, 0.15)]
            };
            pack(&mut sp.enemies, w, cx as f32 - 30.0, cy as f32, cnt / 2 + 1, mix);
            pack(&mut sp.enemies, w, cx as f32 + 30.0, cy as f32, cnt / 2, mix);
            if rnd() < 0.2 + floor as f32 * 0.08 {
                sp.enemies.push((Kind::Brute, cx as f32 + rr(-10.0, 10.0), cy as f32 + rr(-6.0, 6.0)));
            }
        }
    }
    // stairs up in the first room, down (or the boss) in the last
    let (ux, uy) = rooms[0].c();
    for y in uy - 20..uy - 8 {
        for x in ux - 9..ux + 9 {
            w.m[idx(x, y)] = E;
            w.g[idx(x, y)] = STAIRSUP;
        }
    }
    let up = if floor > 1 {
        if cellars { AreaId::Cellars(floor - 1) } else { AreaId::Crypt(floor - 1) }
    } else if cellars {
        AreaId::Ravenhold
    } else {
        AreaId::Pines
    };
    let up_label = if floor > 1 { if cellars { format!("DEPTH {}", floor - 1) } else { format!("FLOOR {}", floor - 1) } } else { up.name() };
    sp.exits.push(Exit { x0: ux - 10, y0: uy - 22, x1: ux + 10, y1: uy - 6, to: up, label: up_label });
    sp.entries.push((up, ux as f32, uy as f32 + 6.0));
    sp.start = (ux as f32, uy as f32 + 6.0);
    let (dx, dy) = rooms[n - 1].c();
    if boss {
        sp.enemies.push((if cellars { Kind::Chief } else { Kind::King }, dx as f32, dy as f32 - 10.0));
        if cellars {
            // the chief's hoard, and his powder store
            sp.chests.push((dx as f32 - 90.0, dy as f32 - 80.0, 1));
            sp.chests.push((dx as f32 + 90.0, dy as f32 - 80.0, 1));
            barrels(w, &mut sp, dx as f32 - 130.0, dy as f32 + 90.0, 4, &[(0, 1.0)]);
            barrels(w, &mut sp, dx as f32 + 90.0, dy as f32 + 90.0, 3, &[(0, 0.5), (1, 0.5)]);
        } else {
            barrels(w, &mut sp, dx as f32 - 120.0, dy as f32 + 90.0, 3, &[(1, 1.0)]);
        }
        for k in 0..4 {
            sp.braziers.push((dx as f32 + [-110.0, 110.0, -110.0, 110.0][k], dy as f32 + [-80.0, -80.0, 80.0, 80.0][k]));
        }
    } else {
        for y in dy - 6..dy + 6 {
            for x in dx - 9..dx + 9 {
                w.m[idx(x, y)] = E;
                w.g[idx(x, y)] = STAIRS;
            }
        }
        let down = if cellars { AreaId::Cellars(floor + 1) } else { AreaId::Crypt(floor + 1) };
        let label = if cellars { format!("DEPTH {}", floor + 1) } else { format!("FLOOR {}", floor + 1) };
        sp.exits.push(Exit { x0: dx - 10, y0: dy - 8, x1: dx + 10, y1: dy + 8, to: down, label });
        sp.entries.push((down, dx as f32, dy as f32 + 18.0));
    }
    sp
}
