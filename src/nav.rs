// Enemy pathfinding: a flow field. A few times a second we flood outwards from the player
// over a coarse grid (one nav cell = NC x NC world cells) and store each cell's walking
// distance to the player. A chasing enemy just steps to whichever neighbouring nav cell is
// closer, so a whole horde finds its way round walls, pillars and corners for one cheap search.
use crate::world::*;
use std::collections::VecDeque;

/// world cells per nav cell
pub const NC: i32 = 4;
/// the field covers this many nav cells either side of the player (600 world cells)
const R: i32 = 150;
const SIDE: i32 = 2 * R + 1;
const FAR: u16 = u16::MAX;
/// test switch: walk straight at the player like before (for --navtest comparisons)
pub static OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct Nav {
    ox: i32,
    oy: i32,
    d: Vec<u16>,
    q: VecDeque<(i32, i32)>,
}

impl Nav {
    pub fn new() -> Nav {
        Nav { ox: 0, oy: 0, d: vec![FAR; (SIDE * SIDE) as usize], q: VecDeque::new() }
    }

    /// can something walk through this nav cell? (samples catch any wall 4 or more cells thick)
    fn open(w: &World, gx: i32, gy: i32) -> bool {
        let (x, y) = (gx * NC, gy * NC);
        if x < 0 || y < 0 || x as usize >= w.aw || y as usize >= w.ah {
            return false;
        }
        !(w.solid(x + 1, y + 1) || w.solid(x + 3, y + 1) || w.solid(x + 1, y + 3) || w.solid(x + 3, y + 3))
    }

    /// flood from the player
    pub fn build(&mut self, w: &World, px: f32, py: f32) {
        let (cx, cy) = (px as i32 / NC, py as i32 / NC);
        self.ox = cx - R;
        self.oy = cy - R;
        self.d.fill(FAR);
        self.q.clear();
        self.d[(R * SIDE + R) as usize] = 0;
        self.q.push_back((R, R));
        while let Some((x, y)) = self.q.pop_front() {
            let dd = self.d[(y * SIDE + x) as usize] + 1;
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if nx < 0 || ny < 0 || nx >= SIDE || ny >= SIDE {
                    continue;
                }
                let k = (ny * SIDE + nx) as usize;
                if self.d[k] != FAR {
                    continue;
                }
                if !Nav::open(w, nx + self.ox, ny + self.oy) {
                    // mark walls so they are never looked at again
                    self.d[k] = FAR - 1;
                    continue;
                }
                self.d[k] = dd;
                self.q.push_back((nx, ny));
            }
        }
    }

    fn at(&self, lx: i32, ly: i32) -> u16 {
        if lx < 0 || ly < 0 || lx >= SIDE || ly >= SIDE {
            return FAR;
        }
        let v = self.d[(ly * SIDE + lx) as usize];
        if v >= FAR - 1 { FAR } else { v }
    }

    /// which way to walk from (x, y) to get closer to the player, as a unit vector
    pub fn dir(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if OFF.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        let (lx, ly) = (x as i32 / NC - self.ox, y as i32 / NC - self.oy);
        let here = self.at(lx, ly);
        let mut best = (here, 0, 0);
        for dy in -1..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                // no cutting corners past a wall
                if dx != 0 && dy != 0 && (self.at(lx + dx, ly) == FAR || self.at(lx, ly + dy) == FAR) {
                    continue;
                }
                let v = self.at(lx + dx, ly + dy);
                if v < best.0 {
                    best = (v, dx, dy);
                }
            }
        }
        if best.0 == FAR || (best.1 == 0 && best.2 == 0) {
            return None;
        }
        // head for the middle of the better cell
        let tx = ((lx + best.1 + self.ox) * NC) as f32 + NC as f32 / 2.0;
        let ty = ((ly + best.2 + self.oy) * NC) as f32 + NC as f32 / 2.0;
        let (vx, vy) = (tx - x, ty - y);
        let l = (vx * vx + vy * vy).sqrt();
        if l < 0.01 {
            return None;
        }
        Some((vx / l, vy / l))
    }
}
