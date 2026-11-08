p = 'src/areagen.rs'
s = open(p, encoding='utf8').read()
a = s.index('// ------------------------------------------------------------------ dungeons')
s = s[:a] + open('tools/dungeon_new.rs', encoding='utf8').read()


def rep(a, b):
    global s
    assert s.count(a) == 1, a[:60]
    s = s.replace(a, b)


rep("            AreaId::Crypt(_) | AreaId::Cellars(_) => (1280, 960),", "            AreaId::Crypt(_) | AreaId::Cellars(_) => (1600, 1200),")
rep("""    /// (x, y, kind): see props::SHRINES
    pub shrines: Vec<(f32, f32, u8)>,
}""", """    /// (x, y, kind): see props::SHRINES
    pub shrines: Vec<(f32, f32, u8)>,
    /// (x, y, kind): 0 powder keg, 1 oil, 2 acid
    pub barrels: Vec<(f32, f32, u8)>,
}""")
rep("chests: vec![], shrines: vec![] }", "chests: vec![], shrines: vec![], barrels: vec![] }")
rep("""/// a stack of logs or crates""", """/// a row of barrels; each stands on a small block of wood, so fire, blasts and blades
/// can reach it (see props.rs)
fn barrels(w: &mut World, sp: &mut Spawns, x: f32, y: f32, n: usize, mix: &[(u8, f32)]) {
    let tot: f32 = mix.iter().map(|m| m.1).sum();
    for k in 0..n {
        let (bx, by) = (x + k as f32 * 13.0 + rr(-1.5, 1.5), y + rr(-2.0, 2.0) + if k % 2 == 1 { 5.0 } else { 0.0 });
        if !open_spot(w, bx, by, 6) {
            continue;
        }
        let mut r = rnd() * tot;
        let mut kind = mix[0].0;
        for &(kk, p) in mix {
            r -= p;
            if r <= 0.0 {
                kind = kk;
                break;
            }
        }
        w.rect(bx as i32 - 4, by as i32 - 3, bx as i32 + 5, by as i32 + 1, WOOD);
        sp.barrels.push((bx, by, kind));
    }
}
/// a stack of logs or crates""")
rep("    sp.chests.push((lx - 85.0, ly - 40.0, 0));\n", "    sp.chests.push((lx - 85.0, ly - 40.0, 0));\n    barrels(w, &mut sp, lx + 60.0, ly - 20.0, 3, &[(1, 0.7), (0, 0.3)]);\n")
rep("""    for _ in 0..6 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32), rr(r0 as f32 + 90.0, r1 as f32 - 90.0));
        w.splash(x, y, OIL, rr(4.0, 7.0), 0);
    }""", """    for _ in 0..6 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32), rr(r0 as f32 + 90.0, r1 as f32 - 90.0));
        w.splash(x, y, OIL, rr(4.0, 7.0), 0);
    }
    // the bandits' powder and lamp oil, stacked about the yard
    for _ in 0..7 {
        let (x, y) = (rr(yx0 as f32, yx1 as f32 - 40.0), rr(r0 as f32 + 100.0, r1 as f32 - 100.0));
        if (y - gate_y).abs() > 30.0 {
            barrels(w, &mut sp, x, y, 2 + ri(3), &[(0, 0.6), (1, 0.4)]);
        }
    }""")
open(p, 'w', encoding='utf8').write(s)
print("dungeons ok")
