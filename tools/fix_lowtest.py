p = 'src/main.rs'
s = open(p, encoding='utf8').read()
a = """    if args.iter().any(|a| a == "--navtest") {"""
b = """    if args.iter().any(|a| a == "--lowtest") {
        low_test();
        return;
    }
    if args.iter().any(|a| a == "--navtest") {"""
assert s.count(a) == 1
s = s.replace(a, b)
a = """/// test helper: stand a little to the west of the first reachable enemy"""
b = """/// --lowtest: fire a fireball and an arrow-height shot at a crawler and at barrels lying in its path
fn low_test() {
    use body::Kind;
    let mut g = game::Game::new(7);
    g.saving = false;
    g.travel(areagen::AreaId::Crypt(1));
    let (px, py) = (g.player.x, g.player.y);
    g.enemies.clear();
    let mut c = game::Game::new_enemy(Kind::Crawler, px + 60.0, py);
    c.stun = 100000;
    let hp0 = c.hp;
    g.enemies.push(c);
    g.projs.push(game::Proj { k: 0, x: px + 10.0, y: py, z: 26.0, vx: 4.0, vy: 0.0, vz: 0.0, life: 90 });
    let inp = game::Input::default();
    for _ in 0..40 {
        g.update(&inp);
    }
    let e = &g.enemies[0];
    println!("fireball at a crawler: hp {:.0} -> {:.0}, alive {}", hp0, e.hp.max(0.0), e.alive);
    // an oil barrel and a keg on a firing line
    for (k, name) in [(1u8, "oil barrel"), (0u8, "powder keg")] {
        let (bx, by) = (px + 50.0, py + 30.0);
        for y in by as i32 - 6..by as i32 + 4 {
            for x in bx as i32 - 8..bx as i32 + 8 {
                g.world.set(x, y, world::E);
            }
        }
        g.world.rect(bx as i32 - 4, by as i32 - 3, bx as i32 + 5, by as i32 + 1, world::WOOD);
        g.barrels = vec![barrels::Barrel::new(bx, by, k, &g.world)];
        g.projs.push(game::Proj { k: 3, x: px, y: by, z: 26.0, vx: 4.0, vy: 0.0, vz: 0.0, life: 90 });
        for _ in 0..30 {
            g.update(&inp);
        }
        let b = &g.barrels[0];
        println!("arrow at a {}: alive {}, fuse {}", name, b.alive, b.fuse);
    }
}

/// test helper: stand a little to the west of the first reachable enemy"""
assert s.count(a) == 1
s = s.replace(a, b)
open(p, 'w', encoding='utf8').write(s)
print("lowtest ok")
