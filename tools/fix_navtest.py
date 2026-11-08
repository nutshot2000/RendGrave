def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/nav.rs', [
    ("const FAR: u16 = u16::MAX;\n", "const FAR: u16 = u16::MAX;\n/// test switch: walk straight at the player like before (for --navtest comparisons)\npub static OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);\n"),
    ("""    pub fn dir(&self, x: f32, y: f32) -> Option<(f32, f32)> {
""", """    pub fn dir(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if OFF.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
"""),
    ("""        if best.0 == FAR || (best.1 == 0 && best.2 == 0) {
            // standing in a wall cell (hugging a corner): use any open neighbour's direction
            if here == FAR {
                return None;
            }
            return None;
        }""", """        if best.0 == FAR || (best.1 == 0 && best.2 == 0) {
            return None;
        }"""),
])
edit('src/main.rs', [
    ("""    if let Some(i) = args.iter().position(|a| a == "--maps") {""", """    if args.iter().any(|a| a == "--navtest") {
        nav_test();
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--maps") {"""),
    ("""/// test helper: stand a little to the west of the first reachable enemy""", """/// --navtest: in every dungeon floor, wake every enemy within reach and stand still for 20
/// seconds; count how many get to you with and without pathfinding
fn nav_test() {
    use areagen::AreaId;
    for area in [AreaId::Crypt(1), AreaId::Crypt(2), AreaId::Cellars(1), AreaId::Graveyard] {
        let mut out = vec![];
        for off in [true, false] {
            nav::OFF.store(off, std::sync::atomic::Ordering::Relaxed);
            let mut g = game::Game::new(7);
            g.saving = false;
            g.travel(area);
            let (px, py) = (g.player.x, g.player.y);
            let mut woke = 0;
            for e in g.enemies.iter_mut() {
                let d = ((e.x - px).powi(2) + (e.y - py).powi(2)).sqrt();
                if d < 400.0 && !e.k.ranged() && !e.k.boss() {
                    e.aggro = true;
                    woke += 1;
                } else {
                    e.alive = false;
                }
            }
            let inp = game::Input::default();
            let t0 = Instant::now();
            for _ in 0..1200 {
                g.player.hp = g.player.maxhp;
                g.player.inv = 5;
                g.update(&inp);
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / 1200.0;
            let near = g.enemies.iter().filter(|e| e.alive && ((e.x - g.player.x).powi(2) + (e.y - g.player.y).powi(2)).sqrt() < 60.0).count();
            out.push(format!("{}: {}/{} reached you ({:.2} ms/frame)", if off { "straight line" } else { "pathfinding" }, near, woke, ms));
        }
        println!("{:<10} {}", area.key(), out.join("   "));
    }
}

/// test helper: stand a little to the west of the first reachable enemy"""),
])
print("navtest ok")
