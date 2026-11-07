// Small fast RNG plus value noise, shared by the whole game.
use std::cell::Cell;

thread_local!(static S: Cell<u64> = Cell::new(0x9E37_79B9_7F4A_7C15));

pub fn seed(s: u64) {
    S.with(|c| c.set(s | 1));
}
#[inline]
pub fn next() -> u64 {
    S.with(|c| {
        let mut x = c.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        c.set(x);
        x
    })
}
#[inline]
pub fn rnd() -> f32 {
    (next() >> 40) as f32 / (1u64 << 24) as f32
}
#[inline]
pub fn rr(a: f32, b: f32) -> f32 {
    a + (b - a) * rnd()
}
#[inline]
pub fn ri(n: usize) -> usize {
    (next() % n.max(1) as u64) as usize
}
#[inline]
pub fn h2(x: i32, y: i32, s: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ s.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / 4_294_967_295.0
}
pub fn vnoise(x: f32, y: f32, s: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (u, v) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = h2(xi, yi, s);
    let b = h2(xi + 1, yi, s);
    let c = h2(xi, yi + 1, s);
    let d = h2(xi + 1, yi + 1, s);
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}
pub fn fbm(x: f32, y: f32, s: u32) -> f32 {
    vnoise(x, y, s) * 0.55 + vnoise(x * 2.1, y * 2.1, s + 7) * 0.3 + vnoise(x * 4.3, y * 4.3, s + 13) * 0.15
}
