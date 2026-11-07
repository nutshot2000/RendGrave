// Paints one frame into the low-res buffers: albedo (lit by the GPU), emission (glows),
// info (light sample height, wall occluders, indoor flag) and UI. Walls and characters are
// drawn upright with a depth buffer keyed on their ground row, so everything sorts correctly.
use crate::body::*;
use crate::game::*;
use crate::gfx::{Frame, GLight};
use crate::hud;
use crate::rng::*;
use crate::world::*;
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
    let r = (dy / 64.0 * 255.0).clamp(0.0, 255.0) as u32;
    r | if occ { 255 << 8 } else { 0 } | if indoor { 255 << 16 } else { 0 } | 0xff00_0000
}

pub struct Ctx<'a> {
    pub f: &'a mut Frame,
    pub cx: i32,
    pub cy: i32,
    pub t: u64,
}
impl<'a> Ctx<'a> {
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
        self.f.inf[i] = (old & 0xffff_ff00) | ((dy / 64.0 * 255.0).clamp(0.0, 255.0) as u32);
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

fn ground_col(w: &World, i: usize, x: i32, y: i32) -> [u8; 3] {
    let s = w.shade[i] as f32 / 255.0;
    let n = vnoise(x as f32 * 0.08, y as f32 * 0.08, 3);
    let mut c = match w.g[i] {
        GRASS => {
            if s > 0.996 {
                [[170u8, 150, 70], [150, 110, 160], [190, 190, 200]][(w.shade[i] % 3) as usize]
            } else {
                let b = 0.8 + n * 0.35 + s * 0.12;
                scl([50, 82, 44], b)
            }
        }
        MOSS => scl([46, 70, 52], 0.8 + n * 0.3 + s * 0.12),
        DIRT => scl([92, 72, 54], 0.82 + n * 0.25 + s * 0.12),
        MUD => scl([62, 50, 40], 0.85 + s * 0.15),
        BURNT => scl([38, 33, 31], 0.85 + s * 0.2),
        FLOOR => {
            let (tx, ty) = (x.rem_euclid(10), (y + (x / 10) % 2 * 5).rem_euclid(10));
            let tile = h2(x / 10, (y + (x / 10) % 2 * 5) / 10, 8);
            if tx == 0 || ty == 0 {
                [40, 38, 44]
            } else {
                scl([78, 74, 82], 0.82 + tile * 0.25 + s * 0.08 + if ty == 1 { 0.08 } else { 0.0 })
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
            let wv = ((x as f32 * 0.35 + y as f32 * 0.5 + t as f32 * 0.05).sin() + (x as f32 * 0.13 - t as f32 * 0.03).sin()) * 0.5;
            let edge = !is_liquid(w.at(x, y - 1));
            let c = if edge { [110, 150, 180] } else if wv > 0.85 { [120, 170, 200] } else { scl([34, 66, 100], 0.9 + wv * 0.15) };
            (mixc(c, g, 0.12), z)
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
            if m == E && y > 0 {
                // ambient occlusion at the foot of walls
                if is_wall(w.at(x, y - 1)) || is_wall(w.at(x, y - 2)) {
                    c = scl(c, 0.55);
                }
            }
            f.alb[o] = pk(c);
            f.emi[o] = pk(e);
            f.inf[o] = inf(0.0, is_wall(m) && m != WOOD, w.aux[i] & AUX_INDOOR != 0);
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
    let mut c = Ctx { f, cx, cy, t };
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
            shadow(&mut c, e.x, e.y, if e.k == Kind::Brute { 11.0 } else { 7.0 });
        }
    }
    // ---- walls (drawn upright, depth = their ground row)
    for wy in cy..cy + fh + 40 {
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
            let mut h = wall_h(m);
            if m == TREE {
                h += (vnoise(wx as f32 * 0.2, wy as f32 * 0.2, 4) * 8.0) as i32 - 4;
            } else if m == STONE {
                h += (vnoise(wx as f32 * 0.3, wy as f32 * 0.3, 6) * 6.0) as i32 - 3;
            }
            let sx = wx - cx;
            let top_y = wy - h - cy;
            let south = w.at(wx, wy + 1);
            let north = w.at(wx, wy - 1);
            let burning = w.life[i] > 0;
            let fc = if burning { fire_col(i as u32 * 7, t) } else { [0; 3] };
            let depth = wy as f32;
            let tc = match m {
                TREE => {
                    let n = vnoise(wx as f32 * 0.35, wy as f32 * 0.35, 9);
                    let edge = !is_wall(north) || north != TREE;
                    let base = scl([42, 86, 44], 0.75 + n * 0.5 + s * 0.15);
                    if edge { scl(base, 1.3) } else { base }
                }
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
            c.put(sx, top_y, depth, tc, te, (h + 1) as f32);
            // front face where the south side is open
            if south != m || wall_h(south) < h {
                for k in 1..=h {
                    let syy = top_y + k;
                    let hh = (h - k) as f32;
                    let (col, draw) = match m {
                        TREE => {
                            let trunk = w.aux[i] & AUX_TRUNK != 0;
                            if k <= 13 {
                                let n = vnoise(wx as f32 * 0.4, (wy + k) as f32 * 0.3, 10);
                                (scl([34, 70, 38], 0.78 + n * 0.45 - k as f32 * 0.035), true)
                            } else if trunk {
                                let bark = ((wx * 3 + k) % 4 == 0) as i32 as f32;
                                (scl([76, 54, 38], 0.9 - bark * 0.25 - (k as f32 / h as f32) * 0.2), true)
                            } else {
                                ([0; 3], false)
                            }
                        }
                        BRICK => {
                            let row = k / 4;
                            let mortar = k % 4 == 0 || (wx + row * 3) % 8 == 0;
                            let base = if mortar { [52, 46, 58] } else { scl([92, 80, 96], 0.85 + h2(wx / 8, row, 2) * 0.25) };
                            (scl(base, 1.0 - k as f32 / h as f32 * 0.35), true)
                        }
                        STONE => (scl([84, 82, 90], 0.95 - k as f32 / h as f32 * 0.35 + s * 0.1), true),
                        _ => (scl([102, 70, 42], 0.9 - k as f32 / h as f32 * 0.3), true),
                    };
                    if draw {
                        let (col, e) = if burning && h2(wx, syy + (t / 4) as i32, 1) < 0.5 { (mixc(col, fc, 0.6), scl(fc, 0.7)) } else { (col, [0; 3]) };
                        c.put(sx, syy, depth, col, e, hh + 1.0);
                    }
                }
            }
        }
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
            flash: if e.flash > 0 { 0.75 } else { 0.0 },
            frozen: e.frozen > 0,
            wet: e.wet > 0,
            shock: e.twitch > 0 && (t / 2) % 2 == 0,
            windup: e.st == St::Wind && e.k == Kind::Brute,
            burn: false,
        };
        draw_body(&mut c, &e.body, e.x + jitter, e.y, tint);
    }
    // ---- projectiles
    for p in &g.projs {
        let (sx, sy) = (p.x as i32 - cx, (p.y - p.z) as i32 - cy);
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
            let a = k as f32 / 60.0 * 6.283;
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
            let ang = k as f32 / n as f32 * 6.283;
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
    for &(bx, by) in &g.braziers {
        let fl = 1.0 + (t as f32 * 0.21 + bx).sin() * 0.08 + (t as f32 * 0.73 + by).sin() * 0.05;
        add(bx, by + 1.0, 120.0, 1.4 * fl, [1.0, 0.55, 0.22]);
    }
    for l in &g.lights {
        let k = if l.max > 0 { l.t as f32 / l.max as f32 } else { 1.0 };
        add(l.x, l.y, l.r, l.i * k, l.c);
    }
    for p in &g.projs {
        match p.k {
            0 => add(p.x, p.y, 110.0, 1.6, [1.0, 0.55, 0.2]),
            1 => add(p.x, p.y, 40.0, 0.5, [0.5, 1.0, 0.3]),
            _ => add(p.x, p.y, 100.0, 1.4 + (t as f32 * 0.9).sin() * 0.3, [0.55, 0.7, 1.0]),
        }
    }
    for e in &g.enemies {
        if e.alive && e.k != Kind::Zombie && e.frozen == 0 {
            let col = if e.k == Kind::Skeleton { [1.0, 0.25, 0.15] } else { [1.0, 0.5, 0.15] };
            add(e.x, e.y + 1.0, 22.0, 0.5, col);
        }
        if e.alive && e.burning > 8 {
            add(e.x, e.y + 1.0, 70.0, (e.burning as f32 / 40.0).min(1.5), [1.0, 0.55, 0.2]);
        }
    }
    for b in &g.bolts {
        let _ = b;
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

#[derive(Default, Clone, Copy)]
pub struct Tint {
    pub flash: f32,
    pub frozen: bool,
    pub wet: bool,
    pub shock: bool,
    pub windup: bool,
    pub burn: bool,
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
            if tint.burn && h2(lx, ly + (t / 3) as i32, 9) < 0.15 {
                e = [200, 110, 30];
            }
            if tint.flash > 0.0 {
                col = mixc(col, [255, 255, 255], tint.flash * 0.8);
                e = scl([255, 255, 255], tint.flash * 0.1);
            }
            let dy = (oy - sy as f32).max(0.0) + z0 * 0.0;
            c.put(sx, sy, depth, col, e, dy);
        }
    }
}
