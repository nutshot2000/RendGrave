// Paints one frame into the low-res buffers: albedo (lit by the GPU), emission (glows),
// info (light sample height, wall occluders, indoor flag) and UI. Walls and characters are
// drawn upright with a depth buffer keyed on their ground row, so everything sorts correctly.
use crate::body::*;
use crate::game::*;
use crate::gfx::{Frame, GLight};
use crate::hud;
use crate::rng::*;
use crate::world::*;
use crate::items::{Rarity, Slot};
use crate::areagen::{BKind, Building};
use crate::skills::*;
use crate::spells::METEOR_T;

#[inline]
pub fn pk(c: [u8; 3]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16 | 0xff00_0000
}
#[inline]
fn mixc(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}
#[inline]
fn scl(c: [u8; 3], f: f32) -> [u8; 3] {
    [(c[0] as f32 * f).clamp(0.0, 255.0) as u8, (c[1] as f32 * f).clamp(0.0, 255.0) as u8, (c[2] as f32 * f).clamp(0.0, 255.0) as u8]
}
#[inline]
fn unpk(v: u32) -> [u8; 3] {
    [(v & 255) as u8, ((v >> 8) & 255) as u8, ((v >> 16) & 255) as u8]
}
#[inline]
fn inf(dy: f32, occ: bool, indoor: bool) -> u32 {
    let r = (dy / 128.0 * 255.0).clamp(0.0, 255.0) as u32;
    r | if occ { 255 << 8 } else { 0 } | if indoor { 255 << 16 } else { 0 } | 0xff00_0000
}

pub struct Ctx<'a> {
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
    }
    #[inline]
    fn put(&mut self, sx: i32, sy: i32, depth: f32, c: [u8; 3], e: [u8; 3], dy: f32) {
        if sx < 0 || sy < 0 || sx >= self.f.w as i32 || sy >= self.f.h as i32 {
            return;
        }
        let i = sy as usize * self.f.w + sx as usize;
        if depth < self.f.z[i] {
            return;
        }
        self.f.z[i] = depth;
        self.f.alb[i] = pk(c);
        self.f.emi[i] = pk(e);
        let old = self.f.inf[i];
        self.f.inf[i] = (old & 0xffff_ff00) | ((dy / 128.0 * 255.0).clamp(0.0, 255.0) as u32);
    }
    #[inline]
    fn glow(&mut self, sx: i32, sy: i32, e: [u8; 3]) {
        if sx < 0 || sy < 0 || sx >= self.f.w as i32 || sy >= self.f.h as i32 {
            return;
        }
        let i = sy as usize * self.f.w + sx as usize;
        let o = unpk(self.f.emi[i]);
        self.f.emi[i] = pk([o[0].saturating_add(e[0]), o[1].saturating_add(e[1]), o[2].saturating_add(e[2])]);
    }
    #[inline]
    fn blend(&mut self, sx: i32, sy: i32, depth: f32, c: [u8; 3], a: f32) {
        if sx < 0 || sy < 0 || sx >= self.f.w as i32 || sy >= self.f.h as i32 {
            return;
        }
        let i = sy as usize * self.f.w + sx as usize;
        if depth < self.f.z[i] - 30.0 {
            return;
        }
        self.f.alb[i] = pk(mixc(unpk(self.f.alb[i]), c, a));
    }
}

fn fire_col(seed: u32, t: u64) -> [u8; 3] {
    let r = h2(seed as i32, (t / 3) as i32, 5);
    if r < 0.3 {
        [255, 236, 150]
    } else if r < 0.7 {
        [255, 150, 40]
    } else {
        [220, 70, 20]
    }
}

/// ground that blends into its neighbours (grass, moss, earth, mud, peat, ash)
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
        ROAD => {
            // packed earth with cobbles showing through
            let cob = h2(x / 3, y / 2 + (x / 3) % 2, 13);
            let base = if cob > 0.62 { [128, 112, 90] } else { [112, 92, 66] };
            scl(base, 0.84 + n * 0.2 + s * 0.1)
        }
        PEAT => {
            let fib = vnoise(x as f32 * 0.5, y as f32 * 0.12, 61);
            scl([50, 44, 34], (0.82 + fib * 0.26) * broad)
        }
        PLANK => {
            let board = (y.rem_euclid(5) == 0) as i32 as f32;
            let seam = ((x + (y / 5) * 13).rem_euclid(23) == 0) as i32 as f32;
            scl([104, 74, 50], 0.8 + h2(x / 23, y / 5, 4) * 0.3 + s * 0.08 - board * 0.3 - seam * 0.2)
        }
        STAIRS | STAIRSUP => {
            // steps going down into darkness (or up into light)
            let step = (y.rem_euclid(4) == 0) as i32 as f32;
            let depth = (y.rem_euclid(24)) as f32 / 24.0;
            let b = if w.g[i] == STAIRS { 0.75 - depth * 0.5 } else { 0.45 + depth * 0.5 };
            scl([96, 90, 100], b - step * 0.25)
        }
        FLOOR => {
            // worn flagstones of uneven size: the nearest of a few jittered points decides
            // which stone a pixel belongs to, and the gap between stones is the grout
            let (cs, cy_) = (11, 8);
            let (gx, gy) = (x.div_euclid(cs), y.div_euclid(cy_));
            let (mut d1, mut d2, mut id) = (1e9f32, 1e9f32, 0.0f32);
            for oy in -1..=1 {
                for ox in -1..=1 {
                    let (cx_, cyy) = (gx + ox, gy + oy);
                    let px = (cx_ * cs) as f32 + h2(cx_, cyy, 51) * cs as f32;
                    let py = (cyy * cy_) as f32 + h2(cx_, cyy, 52) * cy_ as f32;
                    let d = ((x as f32 - px) / 1.2).powi(2) + (y as f32 - py).powi(2);
                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                        id = h2(cx_, cyy, 53);
                    } else if d < d2 {
                        d2 = d;
                    }
                }
            }
            let gap = d2.sqrt() - d1.sqrt();
            let broad = 0.82 + 0.3 * vnoise(x as f32 * 0.02, y as f32 * 0.02, 54);
            if gap < 1.0 {
                scl([42, 39, 44], broad)
            } else {
                let base = if id < 0.07 { [70, 74, 66] } else if id > 0.9 { [86, 78, 72] } else { [76, 72, 80] };
                let crack = h2(x, y / 2, 55) < 0.012 && gap > 2.5;
                let wear = 0.86 + id * 0.18 + (gap.min(4.0) / 4.0) * 0.08 + s * 0.06;
                if crack {
                    scl([40, 38, 42], broad)
                } else {
                    scl(base, wear * broad)
                }
            }
        }
        _ => [60, 60, 60],
    };
    if w.stain[i] > 0 {
        c = mixc(c, [80, 8, 12], w.stain[i] as f32 / 255.0 * 0.8);
    }
    if w.scorch[i] > 0 {
        c = mixc(c, [18, 14, 14], w.scorch[i] as f32 / 255.0 * 0.85);
    }
    c
}

/// colour + emission of the flat top layer
fn top_col(w: &World, i: usize, m: u8, x: i32, y: i32, t: u64, g: [u8; 3]) -> ([u8; 3], [u8; 3]) {
    let s = w.shade[i] as f32 / 255.0;
    let z = [0u8; 3];
    match m {
        WATER => {
            // depth from how far the shore is: pale and murky at the edge, dark further out
            let dry = |dx: i32, dy: i32| !is_liquid(w.at(x + dx, y + dy));
            let near = dry(0, -1) || dry(0, 1) || dry(-1, 0) || dry(1, 0);
            let shallow = near || dry(0, -3) || dry(0, 3) || dry(-4, 0) || dry(4, 0);
            let mid = shallow || dry(0, -7) || dry(0, 7) || dry(-9, 0) || dry(9, 0);
            let slow = vnoise(x as f32 * 0.05 + t as f32 * 0.006, y as f32 * 0.08, 19);
            let base = if mid { [32, 60, 78] } else { [18, 36, 54] };
            let base = if shallow { [46, 78, 90] } else { base };
            let mut c = scl(mixc(base, g, if shallow { 0.3 } else { 0.15 }), 0.86 + slow * 0.24);
            // a thin pale line where the water meets the bank
            if near && dry(0, -1) {
                c = [96, 132, 150];
            }
            // small glints drifting on the surface, short horizontal flecks, not stripes
            let gx = (x + (t / 9) as i32) / 3;
            let gl = h2(gx, y, 23);
            let tw = ((t as f32 * 0.05 + h2(gx, y, 29) * 6.28).sin() + 1.0) * 0.5;
            if gl < 0.03 && tw > 0.6 {
                c = mixc(c, [170, 205, 225], 0.7);
            }
            (c, z)
        }
        ACID => {
            let wv = (x as f32 * 0.5 + t as f32 * 0.08 + s * 6.0).sin();
            let c = if wv > 0.8 { [190, 255, 120] } else { scl([100, 196, 50], 0.85 + s * 0.2) };
            (c, scl([60, 120, 20], 0.4 + wv.max(0.0) * 0.3))
        }
        BLOOD => {
            let edge = w.at(x, y - 1) != BLOOD;
            (if edge { [150, 30, 34] } else { scl([100, 10, 16], 0.85 + s * 0.25) }, z)
        }
        OIL => {
            let sheen = vnoise(x as f32 * 0.2 + t as f32 * 0.01, y as f32 * 0.2, 11);
            let c = if sheen > 0.72 { [[90u8, 60, 110], [60, 100, 90], [110, 90, 50]][(w.shade[i] % 3) as usize] } else { scl([26, 22, 22], 0.9 + s * 0.3) };
            (c, z)
        }
        FIRE => {
            let c = fire_col(i as u32, t + (w.shade[i] as u64));
            (c, scl(c, 0.9))
        }
        ICE => (if w.at(x, y - 1) != ICE { [225, 245, 255] } else { scl([160, 205, 232], 0.88 + s * 0.2) }, [10, 18, 26]),
        FLESH | BONE | METAL | CLOTH => {
            let c = w.col[i];
            let c = if c == [0, 0, 0] { [200, 190, 170] } else { c };
            if w.life[i] > 0 {
                let f = fire_col(i as u32, t);
                (mixc(c, f, 0.6), scl(f, 0.7))
            } else {
                (c, z)
            }
        }
        RUBBLE => {
            let c = w.col[i];
            (if c == [0, 0, 0] { scl([84, 80, 86], 0.8 + s * 0.3) } else { c }, z)
        }
        ASH => (scl([84, 80, 78], 0.8 + s * 0.3), z),
        GAS => {
            let wv = vnoise(x as f32 * 0.08 + t as f32 * 0.004, y as f32 * 0.08, 71);
            (mixc(g, [120, 170, 70], 0.18 + wv * 0.22), [0, (6.0 + wv * 10.0) as u8, 0])
        }
        _ => (g, z),
    }
}

pub fn draw(g: &Game, f: &mut Frame, inp: &Input, lights_out: &mut Vec<GLight>) {
    let (fw, fh) = (f.w as i32, f.h as i32);
    let sh = g.shake;
    let (jx, jy) = if sh > 0.3 { (rr(-sh, sh) as i32, rr(-sh, sh) as i32) } else { (0, 0) };
    let cx = g.cam.0.floor() as i32 + jx;
    let cy = g.cam.1.floor() as i32 + jy;
    let w = &g.world;
    let t = g.frame;
    // ---- ground and flat layer
    for sy in 0..fh {
        for sx in 0..fw {
            let (x, y) = (cx + sx, cy + sy);
            let o = (sy * fw + sx) as usize;
            f.z[o] = -1e9;
            if !World::inb(x, y) {
                f.alb[o] = pk([8, 8, 12]);
                f.emi[o] = pk([0, 0, 0]);
                f.inf[o] = inf(0.0, true, false);
                continue;
            }
            let i = y as usize * W + x as usize;
            let m = w.m[i];
            let gc = ground_col(w, i, x, y);
            let (mut c, mut e) = if m == E || is_wall(m) { (gc, [0, 0, 0]) } else { top_col(w, i, m, x, y, t, gc) };
            if m == TREE {
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
            f.inf[o] = inf(0.0, w.blocks(i), w.aux[i] & AUX_INDOOR != 0);
            let _ = &mut e;
        }
    }
    // electric sparkle on charged puddles
    for &(i, l) in &w.zaps {
        let (x, y) = ((i as usize % W) as i32 - cx, (i as usize / W) as i32 - cy);
        if x >= 0 && y >= 0 && x < fw && y < fh && h2(i as i32, (t / 2) as i32, 3) < 0.35 {
            let o = (y * fw + x) as usize;
            let v = (l as f32 / 20.0).min(1.0);
            f.emi[o] = pk(scl([150, 190, 255], v));
            f.alb[o] = pk([200, 220, 255]);
        }
    }
    let fade = if g.player.dead { None } else { Some((g.player.x - cx as f32, g.player.y - 26.0 - cy as f32, g.player.y)) };
    let mut c = Ctx { f, cx, cy, t, fade };
    // ---- shadows under characters
    let shadow = |c: &mut Ctx, x: f32, y: f32, rw: f32| {
        let (sx, sy) = (x as i32 - c.cx, y as i32 - c.cy);
        for dy in -2..=2 {
            for dx in -(rw as i32)..=(rw as i32) {
                let d = (dx as f32 / rw).powi(2) + (dy as f32 / 2.5).powi(2);
                if d < 1.0 {
                    c.blend(sx + dx, sy + dy, -1e8, [0, 0, 0], 0.45 * (1.0 - d));
                }
            }
        }
    };
    if !g.player.dead {
        shadow(&mut c, g.player.x, g.player.y, 7.0);
    }
    for e in &g.enemies {
        if e.alive {
            shadow(&mut c, e.x, e.y, if e.k == Kind::King { 15.0 } else if e.k.big() { 11.0 } else if e.k == Kind::Bloater { 10.0 } else { 7.0 });
            if e.slam > 0 {
                // the slam is coming: a red ring on the ground, filling in
                let f = 1.0 - e.slam as f32 / crate::bosses::SLAM_T as f32;
                let r = crate::bosses::SLAM_R;
                let pulse = 0.5 + ((t as f32) * 0.5).sin() * 0.2;
                // the danger zone, tinted red, filling in from the middle as the slam comes
                let (ri, rj) = (r as i32, (r * 0.7) as i32);
                for dy in -rj..=rj {
                    for dx in -ri..=ri {
                        let d = ((dx * dx) as f32 / (r * r) + (dy * dy) as f32 / (r * r * 0.49)).sqrt();
                        if d >= 1.0 {
                            continue;
                        }
                        let (gx, gy) = (e.x as i32 + dx - cx, e.y as i32 + dy - cy);
                        let inside = d < f;
                        let edge = d > 0.93 || (d > f - 0.05 && inside);
                        let a = if edge { 0.75 * pulse + 0.2 } else if inside { 0.32 } else { 0.12 };
                        c.blend(gx, gy, -1e8, if inside { [255, 70, 40] } else { [200, 30, 20] }, a);
                        if edge && (dx + dy) % 2 == 0 {
                            c.glow(gx, gy, [140, 20, 0]);
                        }
                    }
                }
            }
        }
    }
    // ---- walls and trees (drawn upright, depth = their ground row)
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
                ROCK => scl([20, 18, 24], 0.8 + s * 0.4),
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
                            let st = style_of(w.aux[i]);
                            if st == 0 {
                                let row = k / 4;
                                let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                                let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                                if w.aux[i] & AUX_CRACK != 0 {
                                    // a cracked section: lighter, crumbling, split by dark fissures
                                    let fis = (wx * 3 + k * 2).rem_euclid(11) == 0 || (wx * 5 - k * 3).rem_euclid(13) == 0;
                                    let col = if fis { [24, 20, 26] } else { mixc(base, [128, 112, 100], 0.25) };
                                    scl(col, 1.0 - k as f32 / h as f32 * 0.3)
                                } else {
                                    scl(base, 1.0 - k as f32 / h as f32 * 0.35)
                                }
                            } else {
                                wall_px(st, wx, h - k, s)
                            }
                        }
                        STONE => scl([84, 82, 90], 0.95 - k as f32 / h as f32 * 0.35 + s * 0.1),
                        ROCK => scl([46, 42, 48], 0.9 - k as f32 / h as f32 * 0.5 + h2(wx, k / 3, 7) * 0.2),
                        _ => scl([102, 70, 42], 0.9 - k as f32 / h as f32 * 0.3),
                    };
                    let (col, e) = if burning && h2(wx, syy + (t / 4) as i32, 1) < 0.5 { (mixc(col, fc, 0.6), scl(fc, 0.7)) } else { (col, [0; 3]) };
                    c.put_w(sx, syy, depth, col, e, hh + 1.0);
                }
            }
        }
    }
    // ---- roofs, doorways, windows, pillars
    let pl = if g.player.dead { None } else { Some((g.player.x, g.player.y)) };
    for b in &g.buildings {
        draw_building(&mut c, b, pl);
    }
    // ---- braziers: little fire pits on metal stands
    for &(bx, by) in &g.braziers {
        let (sx, sy) = (bx as i32 - cx, by as i32 - cy);
        for k in 0..8 {
            c.put(sx - 2, sy - k, by, [70, 70, 78], [0; 3], k as f32);
            c.put(sx + 2, sy - k, by, [56, 56, 62], [0; 3], k as f32);
        }
        for dx in -3..=3 {
            c.put(sx + dx, sy - 8, by, [90, 90, 98], [0; 3], 8.0);
        }
        for k in 0..14 {
            let fx = (h2(k, (t / 3) as i32, 2) * 5.0) as i32 - 2;
            let fy = (h2(k, (t / 3) as i32, 3) * 7.0) as i32;
            let col = fire_col((k * 13) as u32, t);
            c.put(sx + fx, sy - 9 - fy, by + 0.1, col, scl(col, 1.0), 9.0 + fy as f32);
        }
    }
    // ---- chests: wooden boxes, iron-bound for the good ones; the lid swings open
    for ch in &g.chests {
        let (sx, sy) = (ch.x as i32 - cx, ch.y as i32 - cy);
        let iron = ch.tier > 0;
        let wood = if iron { [96, 64, 40] } else { [122, 82, 48] };
        let band = if iron { [150, 150, 160] } else { [80, 56, 34] };
        for dx in -6i32..=6 {
            for k in 0..7 {
                let edge = dx.abs() == 6 || k == 0;
                let pc = if (dx.abs() == 3 || k == 3) && iron { band } else if edge { scl(wood, 0.7) } else { scl(wood, 0.9 + (dx + 6) as f32 * 0.012) };
                c.put(sx + dx, sy - k, ch.y, pc, [0; 3], k as f32);
            }
        }
        let open = ch.open;
        let lid = if open { (ch.t as f32 / 10.0).min(1.0) } else { 0.0 };
        for dx in -6i32..=6 {
            for k in 0..4 {
                // the lid rises and tips back as it opens
                let (yy, zz) = (sy - 7 - k - (lid * 3.0) as i32, 7.0 + k as f32 + lid * 3.0);
                let pc = if k == 3 { scl(wood, 1.1) } else if iron && dx.abs() == 3 { band } else { wood };
                if !(open && lid >= 1.0 && k > 1) {
                    c.put(sx + dx, yy - (lid * k as f32) as i32, ch.y - 0.1, pc, [0; 3], zz);
                }
            }
        }
        if !open {
            c.put(sx, sy - 6, ch.y + 0.2, [230, 196, 90], [40, 30, 6], 6.0);
            if (t / 40) % 3 == 0 {
                c.glow(sx + ((t / 3) % 10) as i32 - 5, sy - 10, [60, 50, 20]);
            }
        } else {
            for dx in -4..=4 {
                c.put(sx + dx, sy - 7, ch.y + 0.1, [36, 26, 18], [0; 3], 7.0);
            }
        }
    }
    // ---- shrines: a carved stone pillar with a floating light
    for s in &g.shrines {
        let (sx, sy) = (s.x as i32 - cx, s.y as i32 - cy);
        let col = crate::props::SHRINES[s.k as usize].2;
        for dy in -5..=5 {
            for dx in -9..=9 {
                if (dx * dx) as f32 / 81.0 + (dy * dy) as f32 / 25.0 < 1.0 {
                    c.put(sx + dx, sy + dy, s.y - 20.0, [70, 68, 74], [0; 3], 0.5);
                }
            }
        }
        for k in 0..20 {
            let half = if k < 3 { 5 } else if k > 16 { 4 } else { 3 };
            for dx in -half..=half {
                let rune = !s.used && dx == 0 && k % 4 == 1 && k > 3;
                let base = if dx < 0 { [120, 118, 126] } else { [92, 90, 100] };
                c.put(sx + dx, sy - k, s.y, if rune { col } else { base }, if rune { scl(col, 0.8) } else { [0; 3] }, k as f32);
            }
        }
        let bob = ((t as f32 * 0.05).sin() * 2.0) as i32;
        let orb = if s.used { [60, 58, 64] } else { col };
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                if dx * dx + dy * dy <= 5 {
                    c.put(sx + dx, sy - 26 + dy + bob, s.y + 0.2, orb, if s.used { [0; 3] } else { scl(col, 1.0) }, 26.0);
                }
            }
        }
        if !s.used {
            for k in 0..3 {
                let a = t as f32 * 0.04 + k as f32 * 2.1;
                c.glow(sx + (a.cos() * 8.0) as i32, sy - 22 + (a.sin() * 3.0) as i32 + bob, scl(col, 0.7));
            }
        }
    }
    // ---- the merchant's stall: a counter under a striped awning, the merchant behind it
    if let Some((mx, my)) = g.merchant {
        let (sx, sy) = (mx as i32 - cx, my as i32 - cy);
        // posts
        for k in 0..30 {
            c.put(sx - 15, sy - k, my - 6.0, [92, 64, 40], [0; 3], k as f32);
            c.put(sx + 15, sy - k, my - 6.0, [80, 56, 36], [0; 3], k as f32);
        }
        // the merchant: hooded, round, holding up a lantern
        let robe = [70, 52, 96];
        for k in 0..20 {
            let half = if k < 12 { 5 } else { 4 };
            for dx in -half..=half {
                let shade = 0.8 + (dx + half) as f32 / (2 * half) as f32 * 0.35;
                c.put(sx + dx, sy - 6 - k, my - 4.0, scl(robe, shade), [0; 3], 6.0 + k as f32);
            }
        }
        for dy in 0..7 {
            for dx in -3i32..=3 {
                let face = dy > 1 && dy < 6 && dx.abs() < 3;
                c.put(sx + dx, sy - 26 - dy, my - 4.0, if face { [214, 172, 138] } else { [56, 42, 80] }, [0; 3], 26.0 + dy as f32);
            }
        }
        c.put(sx - 1, sy - 29, my - 3.9, [30, 20, 20], [0; 3], 29.0);
        c.put(sx + 1, sy - 29, my - 3.9, [30, 20, 20], [0; 3], 29.0);
        let flick = 0.85 + (t as f32 * 0.3).sin() * 0.1;
        c.put(sx + 8, sy - 22, my - 3.0, [255, 220, 140], scl([255, 190, 90], flick), 22.0);
        c.put(sx + 8, sy - 21, my - 3.0, [255, 200, 110], scl([255, 170, 70], flick), 21.0);
        // counter with goods on it
        for dx in -14..=14 {
            for k in 0..9 {
                let col = if k == 8 { [140, 100, 64] } else if dx % 5 == 0 { [86, 58, 36] } else { [112, 78, 48] };
                c.put(sx + dx, sy - k, my + 0.2, col, [0; 3], k as f32);
            }
        }
        for &(dx, col) in &[(-10, [220, 40, 60]), (-6, [220, 40, 60]), (2, [190, 190, 205]), (8, [230, 200, 90])] {
            c.put(sx + dx, sy - 9, my + 0.3, col, [0; 3], 9.0);
            c.put(sx + dx, sy - 10, my + 0.3, col, [0; 3], 10.0);
        }
        // striped awning
        for dx in -18..=18 {
            for k in 0..6 {
                let stripe = ((dx + 18) / 4) % 2 == 0;
                let col = if stripe { [170, 40, 38] } else { [222, 206, 170] };
                c.put(sx + dx, sy - 30 - k + (dx.abs() / 6), my - 6.5, scl(col, 0.8 + k as f32 * 0.05), [0; 3], 30.0 + k as f32);
            }
        }
    }
    // ---- townsfolk: Elder Maren with her staff and lantern, villagers in plain clothes
    for n in &g.npcs {
        let (sx, sy) = (n.x as i32 - cx, n.y as i32 - cy);
        shadow(&mut c, n.x, n.y, 6.0);
        let pal: [[u8; 3]; 5] = [[86, 98, 80], [124, 72, 50], [70, 92, 124], [116, 104, 62], [92, 62, 96]];
        let robe = pal[n.k as usize % 5];
        let hair: [u8; 3] = if n.k == 0 { [214, 210, 200] } else { [[60, 40, 30], [150, 110, 60], [40, 30, 26], [110, 70, 40]][n.k as usize % 4] };
        let walk = (n.phase.sin() * 2.0) as i32;
        // legs
        for k in 0..9 {
            c.put(sx - 2 + if k < 5 { walk / 2 } else { 0 }, sy - k, n.y, [56, 46, 40], [0; 3], k as f32);
            c.put(sx + 1 - if k < 5 { walk / 2 } else { 0 }, sy - k, n.y, [48, 40, 36], [0; 3], k as f32);
        }
        // body: a robe for Maren, tunic and belt for villagers
        for k in 9..27 {
            let half = if n.k == 0 { 4 + (27 - k) / 7 } else { 4 };
            for dx in -half..=half {
                let shade = 0.78 + (dx + half) as f32 / (2 * half) as f32 * 0.35;
                let belt = n.k != 0 && k == 15;
                let col = if belt { [60, 40, 28] } else { scl(robe, shade) };
                c.put(sx + dx, sy - k, n.y, col, [0; 3], k as f32);
            }
        }
        // head
        let f = n.face as i32;
        for dy in 0..7 {
            for dx in -3i32..=3 {
                let hairy = dy > 4 || (dx * f < -1 && dy > 1);
                let col = if hairy { hair } else { [214, 172, 138] };
                c.put(sx + dx, sy - 27 - dy, n.y, col, [0; 3], 27.0 + dy as f32);
            }
        }
        c.put(sx + f * 2, sy - 30, n.y + 0.1, [30, 20, 20], [0; 3], 30.0);
        if n.k == 0 {
            // her staff, with a little lantern hung on it
            for k in 0..38 {
                c.put(sx + f * 7, sy - k, n.y + 0.2, [100, 72, 44], [0; 3], k as f32);
            }
            let fl = 0.8 + (t as f32 * 0.21).sin() * 0.15;
            for &(dx, dy) in &[(0, 0), (1, 0), (0, 1), (1, 1)] {
                c.put(sx + f * 7 + dx, sy - 40 + dy, n.y + 0.3, [255, 220, 150], scl([255, 180, 90], fl), 40.0);
            }
        }
    }
    // ---- lantern posts: a pole with a lantern hung from a short arm
    for &(px, py) in &g.posts {
        let (sx, sy) = (px as i32 - cx, py as i32 - cy);
        shadow(&mut c, px, py, 3.0);
        for k in 0..28 {
            c.put(sx, sy - k, py, [86, 62, 40], [0; 3], k as f32);
            c.put(sx + 1, sy - k, py, [66, 48, 32], [0; 3], k as f32);
        }
        for dx in 0..6 {
            c.put(sx + dx, sy - 28, py, [80, 58, 38], [0; 3], 28.0);
        }
        c.put(sx + 5, sy - 27, py, [60, 60, 66], [0; 3], 27.0);
        let fl = 0.85 + (t as f32 * 0.19 + px).sin() * 0.1;
        for dy in 0..4 {
            for dx in 4..7 {
                let rim = dx == 4 || dx == 6 || dy == 0 || dy == 3;
                c.put(sx + dx, sy - 26 + dy, py + 0.1, if rim { [70, 66, 70] } else { [255, 214, 130] }, if rim { [0; 3] } else { scl([255, 170, 70], fl) }, 26.0 - dy as f32);
            }
        }
    }
    // ---- barrels: powder kegs (painted cross), oil (dark, steel hoops), acid (green, glowing seam)
    for b in g.barrels.iter().filter(|b| b.alive && b.k >= 3) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);
        if b.k == 3 {
            // a clay urn: round belly, narrow neck, a lip
            for k in 0..13i32 {
                let half = if k < 9 { (4.6 - ((k as f32 - 4.0) / 4.5).powi(2) * 2.2) as i32 } else if k < 11 { 2 } else { 3 };
                for dx in -half..=half {
                    let shade = 0.72 + (dx + half) as f32 / (2 * half).max(1) as f32 * 0.45;
                    let band = k == 6 && (dx + k) % 2 == 0;
                    let col = if band { [90, 60, 40] } else if k == 12 { [120, 80, 56] } else { [150, 98, 66] };
                    c.put(sx + dx, sy - k, b.y + 0.5, scl(col, shade), [0; 3], k as f32);
                }
            }
        } else {
            // a crate: planks with a cross brace
            for k in 0..10i32 {
                for dx in -5i32..=5 {
                    let edge = dx.abs() == 5 || k == 0 || k == 9;
                    let brace = (dx + 5) == k || (dx + 5) == 9 - k;
                    let col = if edge || brace { [86, 58, 36] } else if (k / 3) % 2 == 0 { [128, 92, 58] } else { [116, 82, 52] };
                    c.put(sx + dx, sy - k, b.y + 0.5, col, [0; 3], k as f32);
                }
            }
        }
    }
    for b in g.barrels.iter().filter(|b| b.alive && b.k < 3) {
        let (sx, sy) = (b.x as i32 - cx, b.y as i32 - cy);
        let (wood, hoop) = match b.k {
            0 => ([118u8, 74, 42], [40u8, 34, 30]),
            1 => ([62, 54, 50], [130, 130, 138]),
            _ => ([84, 110, 70], [120, 124, 120]),
        };
        let lit = b.fuse > 0 && (b.fuse / 4) % 2 == 0;
        for k in 0..15i32 {
            let bulge = 1.0 - ((k as f32 - 7.0) / 7.5).powi(2);
            let half = (3.5 + bulge * 1.8) as i32;
            for dx in -half..=half {
                let shade = 0.75 + (dx + half) as f32 / (2 * half) as f32 * 0.45;
                let hooped = k == 2 || k == 12 || (b.k != 0 && k == 7);
                let mut col = if hooped { hoop } else if dx.rem_euclid(3) == 0 { scl(wood, 0.8) } else { wood };
                col = scl(col, shade);
                let mut em = [0u8; 3];
                if b.k == 0 && !hooped && (5..=9).contains(&k) && (dx == k - 7 || dx == 7 - k) {
                    col = [200, 40, 30];
                }
                if b.k == 2 && dx == half - 1 && k > 3 && k < 11 {
                    col = [150, 230, 90];
                    em = [20, 60, 10];
                }
                if b.k == 1 && dx == 1 - half && k > 3 && k < 11 {
                    col = [150, 150, 160];
                }
                if lit {
                    em = [70, 26, 0];
                }
                c.put(sx + dx, sy - k, b.y + 0.5, col, em, k as f32);
            }
        }
        for dx in -4..=4 {
            c.put(sx + dx, sy - 15, b.y + 0.5, scl(wood, 1.15), [0; 3], 15.0);
        }
        if b.k == 0 {
            c.put(sx + 2, sy - 16, b.y + 0.5, [70, 60, 46], [0; 3], 16.0);
            c.put(sx + 3, sy - 17, b.y + 0.5, [70, 60, 46], [0; 3], 17.0);
            if b.fuse > 0 {
                c.glow(sx + 3, sy - 18, [255, 200, 80]);
            }
        }
    }
    // ---- waypoint: a rune circle that glows brighter once found
    if let Some((wx, wy)) = g.waypoint {
        let found = g.discovered.contains(&g.area);
        let (sx, sy) = (wx as i32 - cx, wy as i32 - cy);
        let pulse = 0.6 + (t as f32 * 0.06).sin() * 0.3;
        for k in 0..90 {
            let a = k as f32 / 90.0 * std::f32::consts::TAU;
            let (x, y) = (sx + (a.cos() * 14.0) as i32, sy + (a.sin() * 9.0) as i32);
            c.glow(x, y, scl([90, 140, 255], pulse * if found { 1.0 } else { 0.35 }));
        }
        for k in 0..12 {
            c.put(sx, sy - 14 - k, wy, [130, 150, 190], scl([80, 120, 255], pulse * 0.6), 14.0 + k as f32);
        }
    }
    // ---- pickups
    for p in &g.pickups {
        let (sx, sy) = (p.x as i32 - cx, (p.y - p.z) as i32 - cy - (((p.t as f32) * 0.08).sin() * 1.5) as i32);
        if p.k == 0 {
            let n = (p.amt / 10).clamp(1, 5);
            for k in 0..n {
                let ox = (k % 3) * 3 - 3;
                let oy = -(k / 3) * 2;
                let shine = (t / 6 + k as u64) % 9 == 0;
                c.put(sx + ox, sy + oy, p.y, [240, 196, 70], if shine { [120, 100, 30] } else { [40, 30, 0] }, 2.0);
                c.put(sx + ox + 1, sy + oy, p.y, [200, 150, 40], [20, 14, 0], 2.0);
                c.put(sx + ox, sy + oy - 1, p.y, [255, 230, 120], [30, 24, 0], 3.0);
            }
        } else if p.k == 2 {
            let sc = SCHOOLS[NODES[p.amt as usize].col].1;
            for dy in 0..6 {
                for dx in -4i32..=4 {
                    let spine = dx == 0;
                    let col = if spine { [60, 30, 20] } else if dy == 0 || dx.abs() == 4 { [150, 110, 50] } else { [110, 40, 34] };
                    c.put(sx + dx, sy - 7 + dy, p.y, col, [0; 3], 7.0 - dy as f32);
                }
            }
            let pulse = 0.6 + (t as f32 * 0.15).sin() * 0.4;
            c.put(sx - 2, sy - 5, p.y + 0.1, sc, scl(sc, pulse), 6.0);
            c.put(sx + 2, sy - 4, p.y + 0.1, sc, scl(sc, pulse), 6.0);
            c.glow(sx, sy - 10, scl(sc, pulse * 0.6));
        } else {
            for dy in 0..6 {
                for dx in -2i32..=2 {
                    let neck = dy < 2 && dx.abs() > 0;
                    if neck {
                        continue;
                    }
                    let col = if dy < 2 { [150, 130, 110] } else { [220, 40, 60] };
                    c.put(sx + dx, sy - 6 + dy, p.y, col, if dy >= 2 { [70, 10, 16] } else { [0; 3] }, 6.0 - dy as f32);
                }
            }
        }
    }
    // ---- loot on the ground: a little shape in its rarity colour, rare and better throw a beam of light
    for gi in &g.ground {
        if gi.t < 0 {
            continue;
        }
        let it = &gi.item;
        let rc = it.rarity.col();
        let (sx, sy) = (gi.x as i32 - cx, (gi.y - gi.z) as i32 - cy);
        let shape: &[(i32, i32)] = match it.slot {
            Slot::Weapon => &[(-4, 0), (-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, -1), (-3, -1), (-3, 1)],
            Slot::Helm => &[(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, -1), (0, -1), (1, -1)],
            Slot::Armor => &[(-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (-1, -1), (0, -1), (1, -1), (-1, 1), (0, 1), (1, 1)],
            Slot::Ring => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
        };
        let pulse = 0.6 + (t as f32 * 0.1 + gi.x).sin() * 0.3;
        for &(dx, dy) in shape {
            c.put(sx + dx, sy + dy, gi.y + 0.2, mixc([160, 160, 170], rc, 0.5), scl(rc, 0.35 * pulse), 1.0);
        }
        if it.rarity >= Rarity::Rare {
            for k in 1..40 {
                let a = (1.0 - k as f32 / 40.0) * pulse;
                c.glow(sx, sy - k, scl(rc, a * 0.5));
                if k % 2 == 0 {
                    c.glow(sx - 1, sy - k, scl(rc, a * 0.15));
                    c.glow(sx + 1, sy - k, scl(rc, a * 0.15));
                }
            }
        }
    }
    // ---- gibs
    for gb in &g.gibs {
        let xf = Xf::new(0.0, 0.0, gb.a);
        draw_part(&mut c, &gb.part, &xf, gb.mirror, gb.x, gb.y - gb.z, gb.y + 0.2, Tint { frozen: gb.frozen, ..Default::default() }, gb.z);
    }
    // ---- characters
    if !g.player.dead {
        let p = &g.player;
        let tint = Tint { flash: (p.inv > 0 && p.inv % 4 < 2 && p.dodge == 0) as i32 as f32 * 0.5, burn: p.burn > 0, ..Default::default() };
        draw_body(&mut c, &p.body, p.x, p.y, tint);
    }
    for e in &g.enemies {
        if !e.alive {
            continue;
        }
        let jitter = if e.twitch > 0 { rr(-1.0, 1.0) } else { 0.0 };
        let tint = Tint {
            flash: if e.flash > 0 { 0.4 } else { 0.0 },
            frozen: e.frozen > 0,
            wet: e.wet > 0,
            shock: e.twitch > 0 && (t / 2) % 2 == 0,
            windup: e.st == St::Wind && e.k.big(),
            burn: false,
            elite: e.elite,
            elite_col: crate::champions::champ_col(e.aff),
        };
        draw_body(&mut c, &e.body, e.x + jitter, e.y, tint);
    }
    // ---- projectiles
    for p in &g.projs {
        let (sx, sy) = (p.x as i32 - cx, (p.y - p.z) as i32 - cy);
        if p.k == 5 {
            // a thrown powder keg, fuse spitting sparks
            for dy in 0..6i32 {
                for dx in -2i32..=2 {
                    let col = if dy == 1 || dy == 4 { [40, 34, 30] } else if dx == dy - 3 || dx == 3 - dy { [200, 40, 30] } else { [118, 74, 42] };
                    c.put(sx + dx, sy - dy, p.y + 0.5, col, [0; 3], p.z + dy as f32);
                }
            }
            c.glow(sx + 1, sy - 7, [255, 200, 80]);
            continue;
        }
        if p.k == 3 || p.k == 4 {
            // arrow: shaft along its flight, grey head
            let l = (p.vx * p.vx + p.vy * p.vy).sqrt().max(0.01);
            let (ux, uy) = (p.vx / l, p.vy / l * 0.6);
            for k in 0..7 {
                let (x, y) = ((p.x - ux * k as f32) as i32 - cx, (p.y - p.z - uy * k as f32) as i32 - cy);
                c.put(x, y, p.y + 0.5, if k == 0 { [200, 200, 210] } else if k == 6 { [220, 220, 220] } else { [120, 86, 54] }, [0; 3], p.z);
            }
            continue;
        }
        let r = if p.k == 1 { 2 } else { 3 };
        for dy in -r..=r {
            for dx in -r..=r {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d > r as f32 + 0.3 {
                    continue;
                }
                let col = if p.k == 0 {
                    if d < 1.2 { [255, 250, 220] } else { fire_col((dx * 7 + dy) as u32, t) }
                } else if p.k == 2 {
                    if d < 1.5 || h2(dx, dy + t as i32, 4) < 0.3 { [235, 245, 255] } else { [110, 150, 255] }
                } else if d < 1.0 {
                    [220, 255, 160]
                } else {
                    [110, 210, 50]
                };
                c.put(sx + dx, sy + dy, p.y + 0.5, col, scl(col, if p.k == 1 { 0.5 } else { 1.0 }), p.z);
            }
        }
        // shadow on the ground
        c.blend(p.x as i32 - cx, p.y as i32 - cy, -1e8, [0, 0, 0], 0.4);
    }
    // ---- particles
    for p in &g.ptcs {
        let (sx, sy) = (p.x as i32 - cx, (p.y - p.z) as i32 - cy);
        let a = p.life as f32 / p.max as f32;
        match p.k {
            PtK::Flame => {
                let col = if a > 0.7 { [255, 240, 170] } else if a > 0.4 { [255, 160, 50] } else { [200, 60, 20] };
                c.glow(sx, sy, scl(col, (a + 0.2) * 0.6));
            }
            PtK::Ember | PtK::Spark => {
                c.put(sx, sy, p.y + 0.3, p.c, scl(p.c, a + 0.2), p.z);
                c.glow(sx, sy, scl(p.c, a));
            }
            PtK::Heal => c.glow(sx, sy, scl(p.c, a)),
            PtK::Smoke | PtK::Steam => {
                let r = ((1.0 - a) * 3.0) as i32 + 1;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy <= r * r {
                            c.blend(sx + dx, sy + dy, p.y, p.c, a * 0.18);
                        }
                    }
                }
                if p.k == PtK::Steam && p.c[1] > 210 {
                    c.glow(sx, sy, [10, 30, 4]);
                }
            }
            PtK::Gold => c.glow(sx, sy, [200, 160, 40]),
            PtK::Cloud => {
                let r = 5;
                let a2 = (a * (1.0 - a) * 4.0).min(1.0);
                for dy in -r..=r {
                    for dx in -r..=r {
                        let d = ((dx * dx + dy * dy) as f32).sqrt() / r as f32;
                        if d < 1.0 {
                            c.blend(sx + dx, sy + dy, 1e7, p.c, a2 * 0.22 * (1.0 - d));
                        }
                    }
                }
            }
            PtK::Acid => {
                c.put(sx, sy, p.y + 0.3, p.c, [30, 70, 10], p.z);
            }
            PtK::Ice => {
                c.put(sx, sy, p.y + 0.3, p.c, [20, 40, 60], p.z);
            }
            _ => c.put(sx, sy, p.y + 0.3, p.c, [0; 3], p.z),
        }
    }
    // ---- meteors: a growing shadow and the rock coming down
    for m in &g.meteors {
        let f = m.t as f32 / METEOR_T as f32;
        let (gx, gy) = (m.x as i32 - cx, m.y as i32 - cy);
        let rw = 6.0 + f * 16.0;
        for dy in -6..=6 {
            for dx in -(rw as i32)..=(rw as i32) {
                let d = (dx as f32 / rw).powi(2) + (dy as f32 / (rw * 0.4)).powi(2);
                if d < 1.0 {
                    c.blend(gx + dx, gy + dy, -1e8, [0, 0, 0], 0.5 * f * (1.0 - d));
                }
            }
        }
        for k in 0..60 {
            let a = k as f32 / 60.0 * std::f32::consts::TAU;
            let (x, y) = (gx + (a.cos() * rw) as i32, gy + (a.sin() * rw * 0.4) as i32);
            c.glow(x, y, scl([255, 120, 40], f * 0.6));
        }
        let (hx, hy) = ((m.x + (1.0 - f) * 90.0) as i32 - cx, (m.y - (1.0 - f) * 240.0) as i32 - cy);
        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d < 4.2 {
                    let col = if d < 2.0 { [255, 240, 180] } else if d < 3.2 { [255, 140, 40] } else { [110, 70, 50] };
                    c.put(hx + dx, hy + dy, 1e7, col, scl(col, 0.9), 50.0);
                }
            }
        }
    }
    // ---- ice spikes
    for sp in &g.spikes {
        if sp.t <= 0 {
            continue;
        }
        let grow = (sp.t as f32 / 4.0).min(1.0) * (1.0 - ((sp.t - 40).max(0) as f32 / 20.0));
        let h = (sp.h * grow) as i32;
        let (sx, sy) = (sp.x as i32 - cx, sp.y as i32 - cy);
        for k in 0..h {
            let w = ((h - k) as f32 / 4.0) as i32;
            for dx in -w..=w {
                let col = if dx < 0 { [225, 245, 255] } else if dx == 0 { [190, 225, 250] } else { [120, 180, 225] };
                c.put(sx + dx, sy - k, sp.y, col, [20, 40, 60], k as f32);
            }
        }
    }
    // ---- lightning bolts
    for b in &g.bolts {
        let a = b.t as f32 / 12.0;
        for k in 0..b.pts.len() - 1 {
            let (p, q) = (b.pts[k], b.pts[k + 1]);
            let n = ((q.0 - p.0).abs().max((q.1 - p.1).abs()) as i32).max(1);
            for j in 0..=n {
                let tt = j as f32 / n as f32;
                let (x, y) = ((p.0 + (q.0 - p.0) * tt) as i32 - cx, (p.1 + (q.1 - p.1) * tt) as i32 - cy);
                c.put(x, y, 1e8, [240, 245, 255], scl([255, 255, 255], a + 0.4), 30.0);
                c.glow(x + 1, y, scl([60, 90, 200], a));
                c.glow(x - 1, y, scl([60, 90, 200], a));
                c.glow(x, y + 1, scl([40, 70, 180], a));
            }
        }
    }
    // ---- sword slashes
    for s in &g.slashes {
        let a = s.t as f32 / 10.0;
        let n = 28;
        for k in 0..n {
            let tt = k as f32 / n as f32;
            if tt > 1.0 - a * 0.5 + 0.5 {
                continue;
            }
            let ang = s.a0 + (s.a1 - s.a0) * tt;
            for r in 20..30 {
                let rr_ = r as f32;
                let (x, y) = ((s.x + ang.cos() * rr_) as i32 - cx, (s.y + ang.sin() * rr_ * 0.8) as i32 - cy);
                let fade = tt * a;
                if r > 27 || h2(k, r, 5) < 0.6 {
                    c.glow(x, y, scl([200, 220, 255], fade * 0.8));
                }
            }
        }
    }
    // ---- frost nova rings
    for nv in &g.novas {
        let n = 120;
        for k in 0..n {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU;
            for dr in 0..3 {
                let r = nv.r - dr as f32 * 2.0;
                let (x, y) = ((nv.x + ang.cos() * r) as i32 - cx, (nv.y + ang.sin() * r * 0.75) as i32 - cy);
                c.glow(x, y, scl([120, 200, 255], 1.0 - dr as f32 * 0.35));
            }
        }
    }
    // ---------------------------------------------------- lights
    let mut ls: Vec<GLight> = vec![];
    let mut add = |x: f32, y: f32, r: f32, i: f32, col: [f32; 3]| {
        let (sx, sy) = (x - cx as f32, y - cy as f32);
        if sx < -r || sy < -r || sx > fw as f32 + r || sy > fh as f32 + r {
            return;
        }
        ls.push(GLight { p: [sx, sy, r, i], c: [col[0], col[1], col[2], 0.0] });
    };
    if !g.player.dead {
        let fl = 1.0 + (t as f32 * 0.13).sin() * 0.03 + (t as f32 * 0.57).sin() * 0.02;
        add(g.player.x + g.player.face * 3.0, g.player.y - 2.0, 150.0, 1.25 * fl, [1.0, 0.78, 0.52]);
        if g.player.cast_t > 0 {
            let bar = g.player.sk.bar();
            let sc = bar.get(g.player.sel).map(|s| SCHOOLS[node(*s).col].1).unwrap_or([255, 255, 255]);
            let col = [sc[0] as f32 / 255.0, sc[1] as f32 / 255.0, sc[2] as f32 / 255.0];
            add(g.player.x + g.player.face * 10.0, g.player.y, 70.0, g.player.cast_t as f32 / 14.0 * 1.5, col);
        }
    }
    if let Some((wx, wy)) = g.waypoint {
        add(wx, wy + 2.0, 90.0, 0.9, [0.45, 0.6, 1.0]);
    }
    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;
        add(bx, by + 1.0, 120.0, 1.4 * fl, [1.0, 0.55, 0.22]);
    }
    for &(lx, ly, lc) in &g.lamps {
        let fl = 1.0 + (t as f32 * 0.17 + lx).sin() * 0.06;
        add(lx, ly, 90.0, 1.3 * fl, lc);
    }
    for &(px, py) in &g.posts {
        let fl = 1.0 + (t as f32 * 0.19 + px).sin() * 0.07;
        add(px + 4.0, py + 2.0, 85.0, 1.1 * fl, [1.0, 0.68, 0.32]);
    }
    for l in &g.lights {
        let k = if l.max > 0 { l.t as f32 / l.max as f32 } else { 1.0 };
        add(l.x, l.y, l.r, l.i * k, l.c);
    }
    for p in &g.pickups {
        if p.k == 2 {
            let sc = SCHOOLS[NODES[p.amt as usize].col].1;
            add(p.x, p.y, 70.0, 1.0, [sc[0] as f32 / 255.0, sc[1] as f32 / 255.0, sc[2] as f32 / 255.0]);
        }
    }
    for p in &g.projs {
        match p.k {
            0 => add(p.x, p.y, 110.0, 1.6, [1.0, 0.55, 0.2]),
            1 => add(p.x, p.y, 40.0, 0.5, [0.5, 1.0, 0.3]),
            2 => add(p.x, p.y, 100.0, 1.4 + (t as f32 * 0.9).sin() * 0.3, [0.55, 0.7, 1.0]),
            _ => {}
        }
    }
    for gi in &g.ground {
        if gi.item.rarity >= Rarity::Rare {
            let rc = gi.item.rarity.col();
            add(gi.x, gi.y, 45.0, 0.7, [rc[0] as f32 / 255.0, rc[1] as f32 / 255.0, rc[2] as f32 / 255.0]);
        }
    }
    for e in &g.enemies {
        if e.alive && e.elite {
            let c = crate::champions::champ_col(e.aff);
            add(e.x, e.y, 60.0, 0.6, [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]);
        }
        if e.alive && !matches!(e.k, Kind::Zombie | Kind::Crawler) && e.frozen == 0 {
            let col = if e.k == Kind::King { [0.5, 0.7, 1.0] } else if e.k.bony() { [1.0, 0.25, 0.15] } else { [1.0, 0.5, 0.15] };
            add(e.x, e.y + 1.0, 22.0, 0.5, col);
        }
        if e.alive && e.burning > 8 {
            add(e.x, e.y + 1.0, 70.0, (e.burning as f32 / 40.0).min(1.5), [1.0, 0.55, 0.2]);
        }
    }
    // fires: count burning cells in blocks, one light per block
    let bs = 16;
    let mut by = (cy / bs) * bs;
    while by < cy + fh + 40 {
        let mut bx = (cx / bs) * bs;
        while bx < cx + fw {
            let mut n = 0;
            let mut acid = 0;
            for y in (by..by + bs).step_by(2) {
                for x in (bx..bx + bs).step_by(2) {
                    if !World::inb(x, y) {
                        continue;
                    }
                    let i = y as usize * W + x as usize;
                    let m = w.m[i];
                    if m == FIRE || (w.life[i] > 0 && matches!(m, TREE | WOOD | FLESH | CLOTH)) {
                        n += 1;
                    } else if m == ACID {
                        acid += 1;
                    }
                }
            }
            if n > 0 {
                let fl = 1.0 + (t as f32 * 0.3 + bx as f32).sin() * 0.1;
                add(bx as f32 + 8.0, by as f32 + 9.0, 50.0 + (n as f32).sqrt() * 12.0, ((n as f32) * 0.05).min(1.6) * fl, [1.0, 0.5, 0.18]);
            }
            if acid > 6 {
                add(bx as f32 + 8.0, by as f32 + 9.0, 40.0, (acid as f32 * 0.008).min(0.4), [0.4, 1.0, 0.25]);
            }
            bx += bs;
        }
        by += bs;
    }
    ls.truncate(crate::gfx::MAX_LIGHTS);
    *lights_out = ls;
    // ---------------------------------------------------- UI
    hud::draw(g, c.f, inp, cx, cy);
}

/// 0 plain brick, 1 timber-framed house, 2 stone tomb
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
    let r = b.roof_r();
    let ym = y0 as f32 + bh as f32 / 2.0;
    let ridge = ym - wh as f32 - r;
    let hd = bh as f32 / 2.0 + ov as f32;
    let slate = b.look > 0.55;
    let base: [u8; 3] = match b.k {
        BKind::House if slate => [78, 82, 98],
        BKind::House => [124, 78, 52],
        BKind::CryptGate => [96, 92, 104],
        _ => [130, 126, 134],
    };
    let row_h = if style == 1 { 3.0 } else { 5.0 };
    // walk each column from the back eave to the front eave: the surface height gives the
    // screen row; nearer rows overwrite farther ones and gaps on the steep front are filled
    for x in x0 - ov..x0 + bw + ov {
        let edge_x = x == x0 - ov || x == x0 + bw + ov - 1;
        let zh = b.roof_z(x);
        let mut prev: Option<i32> = None;
        for wy in y0 - ov..y0 + bh + ov {
            let dyy = (wy as f32 + 0.5 - ym).abs();
            let zm = (r * (1.0 - dyy / hd)).max(0.0);
            let (zz, face) = if zm <= zh {
                (zm, if (wy as f32 + 0.5) < ym { 1 } else { 0 })
            } else {
                (zh, if x < x0 + bw / 2 { 2 } else { 3 })
            };
            let sy = wy - wh - zz.round() as i32;
            let start = match prev {
                Some(p) if sy > p + 1 => p + 1,
                _ => sy,
            };
            prev = Some(sy);
            // shading by which way the surface faces (light from the upper left)
            let mut l: f32 = match face {
                0 => 1.02 - (1.0 - zz / r.max(1.0)) * 0.16,
                1 => 0.5,
                2 => 0.9,
                _ => 0.62,
            };
            let rowc = zz / row_h;
            let row = rowc as i32;
            let frac = rowc - row as f32;
            if frac < 0.24 {
                l *= 0.64;
            }
            let along = if face >= 2 { wy } else { x };
            if (along + row * 3).rem_euclid(if style == 1 { 7 } else { 13 }) == 0 {
                l *= 0.78;
            }
            l += (h2(along / 5 + row * 7, row, 7) - 0.5) * 0.14;
            if face <= 1 && zz > r - 1.3 {
                l = 0.74;
            }
            if face <= 1 && (zm - zh).abs() < 1.0 && zh < r {
                l *= 0.78;
            }
            let mut col = scl(base, l);
            if style == 2 && vnoise(x as f32 * 0.15, wy as f32 * 0.2 + zz * 0.1, 44) > 0.66 {
                col = mixc(col, [64, 92, 52], 0.55);
            }
            if wy >= y0 + bh + ov - 1 || edge_x {
                col = scl(col, 0.58);
            }
            for yy in start..=sy {
                c.put_w(x - cx, yy - cy, depth + 0.6, col, [0; 3], wh as f32 + zz);
            }
        }
    }
    let south = (y0 + bh + ov - wh) as f32;
    // eave shadow on the wall below
    for x in x0..x0 + bw {
        for k in 1..4 {
            c.blend(x - cx, south as i32 + k - cy, depth + 0.6, [0, 0, 0], 0.35 - k as f32 * 0.08);
        }
    }
    // chimney standing on the roof
    if let Some((chx, _)) = b.chimney() {
        let rz = b.roof_z(chx + 3);
        let base_y = (ym - wh as f32 - rz) as i32 + 4;
        for sy in base_y - 15..=base_y {
            for x in chx..chx + 6 {
                let cap = sy < base_y - 13;
                let mortar = (sy - base_y).rem_euclid(3) == 0 || (x + (sy / 3) * 2).rem_euclid(4) == 0;
                let col = if cap { [70, 66, 70] } else if mortar { [70, 50, 44] } else { scl([136, 82, 64], 1.1 - (x - chx) as f32 * 0.07) };
                c.put_w(x - cx, sy - cy, depth + 0.7, col, [0; 3], (wh as f32 + rz + 8.0).min(127.0));
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

/// One cell of a tree. Canopy cells draw a domed leafy top high in the air plus a hanging
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
    let h = base_h;
    let skirt = 30 + (seed * 8.0) as i32;
    // per-tree tint: some darker pines, some brighter oaks
    let dead = w.aux[i] & AUX_DEAD != 0;
    let leaf = if dead { mixc([58, 50, 44], [84, 76, 70], seed) } else { mixc([34, 76, 40], [62, 98, 44], seed) };
    if dead && h2(wx, wy, 13) < 0.55 && w.aux[i] & AUX_TRUNK == 0 {
        // bare branches: only a lattice of twigs
        if (wx + wy) % 3 != 0 {
            return;
        }
    }
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
        // rounded underside: the skirt hangs lowest in the middle, like the bottom of a ball
        let prof = (1.0 - (dx as f32 / r).powi(2)).max(0.0).sqrt();
        let len = (skirt as f32 * (0.3 + 0.7 * prof)) as i32 - (h2(wx, ty, 5) * 5.0) as i32;
        for k in 1..=len.max(1) {
            let n = vnoise(wx as f32 * 0.45, (wy + k) as f32 * 0.3, 10);
            let curve = k as f32 / len.max(1) as f32;
            let col = scl(leaf, (0.66 + n * 0.32 - curve * 0.38 - (dx as f32 / r) * 0.16) * (0.72 + 0.28 * prof));
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
pub struct Tint {
    pub flash: f32,
    pub frozen: bool,
    pub wet: bool,
    pub shock: bool,
    pub windup: bool,
    pub burn: bool,
    pub elite: bool,
    pub elite_col: [u8; 3],
}

fn draw_body(c: &mut Ctx, b: &Body, x: f32, y: f32, tint: Tint) {
    for &i in &b.order {
        let p = &b.parts[i];
        if !p.on {
            continue;
        }
        let xf = b.xf[i];
        draw_part(c, p, &xf, b.mirror, x, y, y, tint, 0.0);
    }
}

/// Inverse-map every screen pixel in the part's bounding box back into the part grid.
fn draw_part(c: &mut Ctx, p: &Part, xf: &Xf, mirror: bool, x: f32, y: f32, depth: f32, tint: Tint, z0: f32) {
    let mx = if mirror { -1.0 } else { 1.0 };
    let mut minx = f32::MAX;
    let mut miny = f32::MAX;
    let mut maxx = f32::MIN;
    let mut maxy = f32::MIN;
    for &(lx, ly) in &[(0.0, 0.0), (p.w as f32, 0.0), (0.0, p.h as f32), (p.w as f32, p.h as f32)] {
        let (a, b) = xf.fwd(p.pivot, lx, ly);
        let a = a * mx;
        minx = minx.min(a);
        maxx = maxx.max(a);
        miny = miny.min(b);
        maxy = maxy.max(b);
    }
    let (ox, oy) = (x - c.cx as f32, y - c.cy as f32);
    let t = c.t;
    for sy in (oy + miny).floor() as i32..=(oy + maxy).ceil() as i32 {
        for sx in (ox + minx).floor() as i32..=(ox + maxx).ceil() as i32 {
            let rx = (sx as f32 + 0.5 - ox) * mx;
            let ry = sy as f32 + 0.5 - oy;
            let (lx, ly) = xf.inv(p.pivot, rx, ry);
            let (lx, ly) = (lx.floor() as i32, ly.floor() as i32);
            let px = match p.get(lx, ly) {
                Some(px) => px,
                None => continue,
            };
            let mut col = px.c;
            let mut e = [0u8; 3];
            if px.chr > 0 {
                col = mixc(col, [22, 16, 14], px.chr as f32 / 255.0 * 0.85);
            }
            if px.acid > 0 {
                col = mixc(col, [150, 210, 60], 0.35 + (px.acid as f32 / 255.0) * 0.3);
                e = [16, 40, 4];
            }
            if px.burn > 0 {
                let f = fire_col((lx * 31 + ly) as u32, t);
                if h2(lx, ly + (t / 3) as i32, 7) < 0.4 {
                    col = mixc(col, f, 0.6);
                    e = scl(f, 0.45);
                } else {
                    e = scl(f, 0.1);
                }
            }
            if px.m == PM_GLOW {
                e = px.c;
            }
            if px.m == PM_METAL && px.chr < 40 && h2(lx + (t / 40) as i32, ly, 1) < 0.02 {
                e = [60, 60, 70];
            }
            if tint.frozen {
                col = mixc(col, [175, 220, 255], 0.6);
                e = [20, 36, 50];
            }
            if tint.wet {
                col = scl(mixc(col, [60, 80, 120], 0.12), 0.92);
            }
            if tint.shock {
                col = mixc(col, [200, 220, 255], 0.35);
                e = [16, 24, 60];
            }
            if tint.windup && (t / 4) % 2 == 0 {
                e = [e[0].saturating_add(70), e[1].saturating_add(20), e[2]];
            }
            if tint.elite {
                let edge = p.get(lx - 1, ly).is_none() || p.get(lx + 1, ly).is_none() || p.get(lx, ly - 1).is_none();
                if edge {
                    let pulse = 0.55 + ((t as f32 * 0.12) + ly as f32 * 0.3).sin() * 0.25;
                    let ec = tint.elite_col;
                    e = [(ec[0] as f32 * pulse) as u8, (ec[1] as f32 * pulse) as u8, (ec[2] as f32 * pulse) as u8];
                }
            }
            if tint.burn && h2(lx, ly + (t / 3) as i32, 9) < 0.15 {
                e = [200, 110, 30];
            }
            if tint.flash > 0.0 {
                // a brief pale flash on a hit; strong lights must not blow it out to white
                col = mixc(col, [255, 240, 225], tint.flash * 0.7);
                e = [e[0].max((tint.flash * 30.0) as u8), e[1].max((tint.flash * 26.0) as u8), e[2].max((tint.flash * 22.0) as u8)];
            }
            let dy = (oy - sy as f32).max(0.0) + z0;
            c.put(sx, sy, depth, col, e, dy);
        }
    }
}
