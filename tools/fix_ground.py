p = 'src/render.rs'
s = open(p, encoding='utf8').read()
a = s.index('fn ground_col(w: &World, i: usize, x: i32, y: i32) -> [u8; 3] {')
b = s.index('        ROAD => {', a)
new_head = '''/// ground that blends into its neighbours (grass, moss, earth, mud, peat, ash)
fn natural(g: u8) -> bool {
    matches!(g, GRASS | MOSS | DIRT | MUD | PEAT | BURNT)
}

fn ground_col(w: &World, i: usize, x: i32, y: i32) -> [u8; 3] {
    let s = w.shade[i] as f32 / 255.0;
    let n = vnoise(x as f32 * 0.08, y as f32 * 0.08, 3);
    let g0 = w.g[i];
    // where two kinds of natural ground meet, each pixel may show its neighbour's ground
    // instead, so the border breaks up into a soft speckled blend rather than a hard line
    let g = if natural(g0) {
        let r = 6.0;
        let ox = ((h2(x, y, 41) - 0.5) * 2.0 * r) as i32;
        let oy = ((h2(y, x, 43) - 0.5) * 1.4 * r) as i32;
        let (nx, ny) = (x + ox, y + oy);
        if World::inb(nx, ny) {
            let gn = w.g[ny as usize * W + nx as usize];
            if natural(gn) { gn } else { g0 }
        } else {
            g0
        }
    } else {
        g0
    };
    // broad, gentle light and shade across the whole ground, and a little fine grain
    let broad = 0.86 + 0.26 * vnoise(x as f32 * 0.011, y as f32 * 0.011, 77);
    let grain = 0.94 + 0.12 * vnoise(x as f32 * 0.21, y as f32 * 0.21, 5) + s * 0.06;
    let b = broad * grain;
    let mut c = match g {
        GRASS => {
            if s > 0.996 {
                [[170u8, 150, 70], [150, 110, 160], [190, 190, 200]][(w.shade[i] % 3) as usize]
            } else if h2(x, y / 3, 9) < 0.07 {
                // a blade of grass catching the light
                scl([70, 104, 56], b)
            } else if h2(x + 7, y / 2, 10) < 0.05 {
                scl([34, 56, 32], b)
            } else {
                scl([50, 80, 45], b)
            }
        }
        MOSS => {
            if h2(x / 2, y / 2, 12) < 0.08 {
                scl([62, 92, 58], b)
            } else {
                scl([44, 72, 50], b)
            }
        }
        DIRT => {
            let h = h2(x, y, 14);
            if h < 0.025 {
                // a pebble
                scl([112, 106, 98], b)
            } else if h2(x / 2, y, 15) < 0.09 {
                // fallen needles and leaf litter
                scl([94, 60, 40], b)
            } else {
                scl([80, 66, 50], b)
            }
        }
        MUD => scl([58, 48, 38], b * (0.95 + s * 0.1)),
        BURNT => scl([38, 33, 31], b * (0.95 + s * 0.15)),
'''
s = s[:a] + new_head + s[b:]
# peat now uses the shared shading too
old = """        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);
            scl([54, 42, 32], 0.75 + fib * 0.35 + s * 0.1)
        }"""
assert s.count(old) == 1
s = s.replace(old, """        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);
            scl([54, 42, 32], (0.8 + fib * 0.3) * broad)
        }""")
old = """            let b = if w.g[i] == STAIRS { 0.75 - depth * 0.5 } else { 0.45 + depth * 0.5 };"""
assert s.count(old) == 1
open(p, 'w', encoding='utf8').write(s)
print("ground ok")
