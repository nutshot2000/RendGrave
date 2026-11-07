# Tall trees with walk-under canopies, fewer of them, taller crypt walls,
# see-through walls/canopies in front of the player, finer light-height encoding.


def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


def block(path, start, end, new):
    s = open(path, encoding='utf8').read()
    i = s.index(start)
    j = s.index(end, i)
    s = s[:i] + new + s[j:]
    open(path, 'w', encoding='utf8').write(s)


# ---------------- world.rs ----------------
edit('src/world.rs', [
    ("        BRICK => 30,\n        TREE => 27,", "        BRICK => 44,\n        TREE => 84,"),
    # only trunks (and real walls) block movement
    ("""    pub fn solid(&self, x: i32, y: i32) -> bool {
        is_wall(self.at(x, y))
    }""", """    pub fn solid(&self, x: i32, y: i32) -> bool {
        if !Self::inb(x, y) {
            return true;
        }
        let i = y as usize * W + x as usize;
        let m = self.m[i];
        is_wall(m) && (m != TREE || self.aux[i] & AUX_TRUNK != 0)
    }
    /// does this cell stop light and line of sight? (canopies don't, trunks do)
    pub fn blocks(&self, i: usize) -> bool {
        let m = self.m[i];
        is_wall(m) && m != WOOD && (m != TREE || self.aux[i] & AUX_TRUNK != 0)
    }"""),
    ("""            let m = self.at(x as i32, y as i32);
            if is_wall(m) && m != WOOD {
                return false;
            }""", """            if !Self::inb(x as i32, y as i32) || self.blocks(y as usize * W + x as usize) {
                return false;
            }"""),
    # forest: fewer, much bigger trees
    ("            if placed > 300 {", "            if placed > 120 {"),
    ("            let r = rr(5.5, 10.0);", "            let r = rr(10.0, 16.0);"),
    ("            if rnd() > dens * 1.4 {", "            if rnd() > dens * 1.2 {"),
    ("            if near_road(x, y, r + 22.0)", "            if near_road(x, y, r * 0.6 + 12.0)"),
    ("""            for dy in 0..(r as i32 + 3) {
                for dx in -1..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if w.at(xx, yy) == TREE {
                        w.aux[yy as usize * W + xx as usize] |= AUX_TRUNK;
                    }
                }
            }""", """            // remember each canopy cell's offset from the trunk; the trunk is a small disc
            for dy in -(r as i32) - 6..=(r as i32) + 6 {
                for dx in -(r as i32) - 6..=(r as i32) + 6 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if w.at(xx, yy) == TREE {
                        let i = yy as usize * W + xx as usize;
                        if w.col[i] == [0, 0, 0] {
                            w.col[i] = [(dx + 128) as u8, (dy + 128) as u8, r as u8];
                        }
                        if dx * dx + dy * dy <= 5 {
                            w.aux[i] |= AUX_TRUNK;
                        }
                    }
                }
            }"""),
])

# ---------------- shader: light sample height now covers 0..128 ----------------
edit('src/shader.wgsl', [("  let dy = inf.r * 64.0;", "  let dy = inf.r * 128.0;")])

# ---------------- render.rs ----------------
edit('src/render.rs', [
    ("    let r = (dy / 64.0 * 255.0).clamp(0.0, 255.0) as u32;", "    let r = (dy / 128.0 * 255.0).clamp(0.0, 255.0) as u32;"),
    ("        self.f.inf[i] = (old & 0xffff_ff00) | ((dy / 64.0 * 255.0).clamp(0.0, 255.0) as u32);",
     "        self.f.inf[i] = (old & 0xffff_ff00) | ((dy / 128.0 * 255.0).clamp(0.0, 255.0) as u32);"),
    ("""pub struct Ctx<'a> {
    pub f: &'a mut Frame,
    pub cx: i32,
    pub cy: i32,
    pub t: u64,
}
impl<'a> Ctx<'a> {""", """pub struct Ctx<'a> {
    pub f: &'a mut Frame,
    pub cx: i32,
    pub cy: i32,
    pub t: u64,
    /// the player's body on screen (x, y, ground row): walls in front of it turn see-through
    pub fade: Option<(f32, f32, f32)>,
}
impl<'a> Ctx<'a> {
    /// a wall or canopy pixel: dithered away where it would hide the player
    #[inline]
    fn put_w(&mut self, sx: i32, sy: i32, depth: f32, c: [u8; 3], e: [u8; 3], dy: f32) {
        if let Some((px, py, pd)) = self.fade {
            if depth > pd {
                let (dx, dyy) = ((sx as f32 - px) / 18.0, (sy as f32 - py) / 34.0);
                let d = dx * dx + dyy * dyy;
                if d < 1.0 {
                    let pat = (sx + sy * 2).rem_euclid(4);
                    if (d < 0.55 && pat != 0) || pat < 2 {
                        return;
                    }
                }
            }
        }
        self.put(sx, sy, depth, c, e, dy);
    }"""),
    # ground: shade under canopies, occluders only for real walls and trunks
    ("""            if m == E && y > 0 {
                // ambient occlusion at the foot of walls
                if is_wall(w.at(x, y - 1)) || is_wall(w.at(x, y - 2)) {
                    c = scl(c, 0.55);
                }
            }
            f.alb[o] = pk(c);
            f.emi[o] = pk(e);
            f.inf[o] = inf(0.0, is_wall(m) && m != WOOD, w.aux[i] & AUX_INDOOR != 0);""", """            if m == TREE {
                // the shade of the canopy overhead
                let off = w.col[i];
                let (dx, dy) = (off[0] as f32 - 128.0, off[1] as f32 - 128.0);
                let d = (dx * dx + dy * dy).sqrt() / (off[2].max(1) as f32);
                c = scl(c, 0.45 + d * 0.35);
            } else if m == E && y > 0 {
                // ambient occlusion at the foot of walls
                if w.solid(x, y - 1) || w.solid(x, y - 2) {
                    c = scl(c, 0.55);
                }
            }
            f.alb[o] = pk(c);
            f.emi[o] = pk(e);
            f.inf[o] = inf(0.0, w.blocks(i), w.aux[i] & AUX_INDOOR != 0);"""),
    ("    let mut c = Ctx { f, cx, cy, t };",
     "    let fade = if g.player.dead { None } else { Some((g.player.x - cx as f32, g.player.y - 26.0 - cy as f32, g.player.y)) };\n    let mut c = Ctx { f, cx, cy, t, fade };"),
])

block('src/render.rs', "    // ---- walls (drawn upright, depth = their ground row)", "    // ---- braziers", r'''    // ---- walls and trees (drawn upright, depth = their ground row)
    for wy in cy..cy + fh + 130 {
        for wx in cx..cx + fw {
            if !World::inb(wx, wy) {
                continue;
            }
            let i = wy as usize * W + wx as usize;
            let m = w.m[i];
            if !is_wall(m) {
                continue;
            }
            let s = w.shade[i] as f32 / 255.0;
            let sx = wx - cx;
            let burning = w.life[i] > 0;
            let fc = if burning { fire_col(i as u32 * 7, t) } else { [0; 3] };
            let depth = wy as f32;
            if m == TREE {
                draw_tree_cell(&mut c, w, i, wx, wy, sx, depth, burning, fc);
                continue;
            }
            let mut h = wall_h(m);
            if m == STONE {
                h += (vnoise(wx as f32 * 0.3, wy as f32 * 0.3, 6) * 6.0) as i32 - 3;
            }
            let top_y = wy - h - cy;
            let south = w.at(wx, wy + 1);
            let north = w.at(wx, wy - 1);
            let tc = match m {
                STONE => {
                    let edge = north != STONE;
                    scl([118, 116, 122], 0.85 + s * 0.2 + if edge { 0.2 } else { 0.0 })
                }
                BRICK => {
                    let edge = north != BRICK;
                    scl([96, 88, 100], 0.85 + s * 0.15 + if edge { 0.25 } else { 0.0 })
                }
                _ => {
                    let plank = (wx % 4 == 0) as i32 as f32;
                    scl([138, 98, 60], 0.85 + s * 0.15 - plank * 0.25)
                }
            };
            let (tc, te) = if burning { (mixc(tc, fc, 0.55), scl(fc, 0.8)) } else { (tc, [0; 3]) };
            c.put_w(sx, top_y, depth, tc, te, (h + 1) as f32);
            // front face where the south side is open
            if south != m || wall_h(south) < h {
                for k in 1..=h {
                    let syy = top_y + k;
                    let hh = (h - k) as f32;
                    let col = match m {
                        BRICK => {
                            let row = k / 4;
                            let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                            let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                            scl(base, 1.0 - k as f32 / h as f32 * 0.35)
                        }
                        STONE => scl([84, 82, 90], 0.95 - k as f32 / h as f32 * 0.35 + s * 0.1),
                        _ => scl([102, 70, 42], 0.9 - k as f32 / h as f32 * 0.3),
                    };
                    let (col, e) = if burning && h2(wx, syy + (t / 4) as i32, 1) < 0.5 { (mixc(col, fc, 0.6), scl(fc, 0.7)) } else { (col, [0; 3]) };
                    c.put_w(sx, syy, depth, col, e, hh + 1.0);
                }
            }
        }
    }
''')

# the tree painter
edit('src/render.rs', [("""#[derive(Default, Clone, Copy)]
pub struct Tint {""", r'''/// One cell of a tree. Canopy cells draw a domed leafy top high in the air plus a hanging
/// skirt on the south edge; trunk cells draw the bark from the ground up to the canopy.
#[allow(clippy::too_many_arguments)]
fn draw_tree_cell(c: &mut Ctx, w: &World, i: usize, wx: i32, wy: i32, sx: i32, depth: f32, burning: bool, fc: [u8; 3]) {
    let t = c.t;
    let off = w.col[i];
    let (dx, dy, r) = (off[0] as i32 - 128, off[1] as i32 - 128, off[2].max(4) as f32);
    let (tx, ty) = (wx - dx, wy - dy);
    let seed = h2(tx, ty, 77);
    let d = ((dx * dx + dy * dy) as f32).sqrt() / r;
    let dome = (1.0 - d * d).max(0.0);
    let base_h = wall_h(TREE) + (seed * 14.0) as i32 - 7;
    let h = base_h + (dome * 10.0) as i32;
    let skirt = 30 + (seed * 8.0) as i32;
    // per-tree tint: some darker pines, some brighter oaks
    let leaf = mixc([34, 76, 40], [62, 98, 44], seed);
    // canopy top
    let clump = vnoise(wx as f32 * 0.32, wy as f32 * 0.32, 9);
    let mut lv = 0.55 + dome * 0.55 - (dx + dy) as f32 / r * 0.16 + (clump - 0.5) * 0.4;
    if d > 0.9 {
        lv *= 0.75;
    }
    let mut col = scl(leaf, lv);
    if h2(wx, wy, 31) > 0.94 && dome > 0.3 {
        col = scl(col, 1.35);
    }
    let top_y = wy - h - c.cy;
    let (col, e) = if burning { (mixc(col, fc, 0.6), scl(fc, 0.8)) } else { (col, [0; 3]) };
    c.put_w(sx, top_y, depth, col, e, (h + 1) as f32);
    // hanging skirt of leaves along the south edge, with a ragged bottom
    if w.at(wx, wy + 1) != TREE {
        let len = skirt - (h2(wx, ty, 5) * 9.0) as i32 + (dome * 6.0) as i32;
        for k in 1..=len {
            let n = vnoise(wx as f32 * 0.45, (wy + k) as f32 * 0.3, 10);
            let col = scl(leaf, 0.62 + n * 0.35 - k as f32 / len as f32 * 0.35 - (dx as f32 / r) * 0.08);
            let (col, e) = if burning && h2(wx, k + (t / 4) as i32, 1) < 0.5 { (mixc(col, fc, 0.6), scl(fc, 0.7)) } else { (col, [0; 3]) };
            c.put_w(sx, top_y + k, depth, col, e, (h - k + 1) as f32);
        }
    }
    // trunk: from the ground up into the canopy
    if w.aux[i] & AUX_TRUNK != 0 && w.aux[i + W] & AUX_TRUNK == 0 {
        let canopy_bottom = wy - h + skirt - 4;
        let bark = scl([82, 58, 40], 1.05 - (dx as f32 + 2.0) * 0.12);
        for y in canopy_bottom..=wy {
            let k = wy - y;
            let groove = ((wx * 5 + y / 3) % 7 == 0) as i32 as f32;
            let col = scl(bark, 0.95 - groove * 0.3 - (k as f32 / (h as f32)) * 0.25);
            let (col, e) = if burning && h2(wx, y + (t / 4) as i32, 1) < 0.4 { (mixc(col, fc, 0.6), scl(fc, 0.6)) } else { (col, [0; 3]) };
            c.put_w(sx, y - c.cy, depth, col, e, k as f32 + 1.0);
        }
        // root flare
        if dx.abs() == 2 {
            c.put(sx + dx.signum(), wy - c.cy, depth, scl(bark, 0.7), [0; 3], 1.0);
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Tint {''')])
print("tree rework applied")
