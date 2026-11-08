// Every sound effect is built here from noise, oscillators, filters and envelopes,
// a few random variations each, so nothing needs to be recorded or licensed.
use crate::audio::{Sfx, SR};
use std::collections::HashMap;
use std::f32::consts::TAU;

struct Rng(u64);
impl Rng {
    fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn n(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
    fn r(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

fn len(secs: f32) -> usize {
    (secs * SR as f32) as usize
}
fn t(i: usize) -> f32 {
    i as f32 / SR as f32
}
/// one-pole low-pass coefficient for a cutoff in Hz
fn lp_k(hz: f32) -> f32 {
    1.0 - (-TAU * hz / SR as f32).exp()
}
fn norm(mut v: Vec<f32>, peak: f32) -> Vec<f32> {
    let m = v.iter().fold(0.0f32, |a, &b| a.max(b.abs())).max(1e-6);
    for s in v.iter_mut() {
        *s *= peak / m;
    }
    // tiny fade at the ends so nothing clicks
    let n = v.len();
    for i in 0..n.min(64) {
        let f = i as f32 / 64.0;
        v[i] *= f;
        v[n - 1 - i] *= f;
    }
    v
}

/// filtered noise with an envelope and a moving cutoff
fn noise(r: &mut Rng, secs: f32, cut0: f32, cut1: f32, hp: f32, env: impl Fn(f32) -> f32) -> Vec<f32> {
    let n = len(secs);
    let (mut lp, mut lp2, mut hpst) = (0.0f32, 0.0f32, 0.0f32);
    let hk = lp_k(hp);
    (0..n)
        .map(|i| {
            let x = i as f32 / n as f32;
            let k = lp_k(cut0 + (cut1 - cut0) * x);
            lp += (r.n() - lp) * k;
            lp2 += (lp - lp2) * k;
            hpst += (lp2 - hpst) * hk;
            (lp2 - hpst) * env(x)
        })
        .collect()
}
/// a tone gliding from f0 to f1 with an envelope; wave 0 sine, 1 triangle, 2 saw, 3 square
fn tone(secs: f32, f0: f32, f1: f32, wave: u8, env: impl Fn(f32) -> f32) -> Vec<f32> {
    let n = len(secs);
    let mut ph = 0.0f32;
    (0..n)
        .map(|i| {
            let x = i as f32 / n as f32;
            let f = f0 * (f1 / f0).powf(x);
            ph = (ph + f / SR as f32).fract();
            let w = match wave {
                0 => (ph * TAU).sin(),
                1 => 1.0 - 4.0 * (ph - 0.5).abs(),
                2 => ph * 2.0 - 1.0,
                _ => {
                    if ph < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };
            w * env(x)
        })
        .collect()
}
fn mix(parts: &[(&[f32], f32, f32)]) -> Vec<f32> {
    // (samples, gain, start seconds)
    let n = parts.iter().map(|(p, _, s)| p.len() + len(*s)).max().unwrap_or(0);
    let mut out = vec![0.0; n];
    for (p, g, s) in parts {
        let o = len(*s);
        for (i, v) in p.iter().enumerate() {
            out[o + i] += v * g;
        }
    }
    out
}
fn exp_dec(k: f32) -> impl Fn(f32) -> f32 {
    move |x| (-x * k).exp()
}
fn attack_dec(a: f32, k: f32) -> impl Fn(f32) -> f32 {
    move |x| if x < a { x / a } else { (-(x - a) * k).exp() }
}
/// soft distortion
fn drive(v: Vec<f32>, amt: f32) -> Vec<f32> {
    v.into_iter().map(|s| (s * amt).tanh()).collect()
}

pub fn build_bank() -> HashMap<Sfx, Vec<Vec<f32>>> {
    let mut r = Rng(0x5EED_1234_ABCD);
    let mut b: HashMap<Sfx, Vec<Vec<f32>>> = HashMap::new();
    let mut add = |k: Sfx, v: Vec<f32>, peak: f32| b.entry(k).or_default().push(norm(v, peak));

    for _ in 0..3 {
        // sword whoosh: band of noise sweeping up then down
        let c = r.r(900.0, 1400.0);
        add(Sfx::Swing, noise(&mut r, 0.2, c, c * 2.4, 400.0, |x| (x * 3.1416).sin().powi(2)), 0.5);
        // flesh: a dull thump and a wet smack
        let th = tone(0.14, r.r(120.0, 150.0), 55.0, 0, exp_dec(18.0));
        let wet = noise(&mut r, 0.13, 1800.0, 500.0, 120.0, exp_dec(22.0));
        add(Sfx::HitFlesh, mix(&[(&th, 1.0, 0.0), (&wet, 0.9, 0.0)]), 0.85);
        // bone: a dry crack and a woody knock
        let cr = noise(&mut r, 0.05, 6000.0, 3000.0, 1500.0, exp_dec(40.0));
        let kn = tone(0.08, r.r(900.0, 1300.0), 600.0, 1, exp_dec(30.0));
        add(Sfx::HitBone, mix(&[(&cr, 1.0, 0.0), (&kn, 0.6, 0.0)]), 0.8);
        // metal: inharmonic partials ringing out
        let base = r.r(420.0, 560.0);
        let parts: Vec<Vec<f32>> = [1.0, 2.31, 3.17, 4.53, 6.02].iter().enumerate().map(|(i, m)| tone(0.6, base * m, base * m * 0.995, 0, exp_dec(5.0 + i as f32 * 3.0))).collect();
        let tick = noise(&mut r, 0.02, 8000.0, 8000.0, 2000.0, exp_dec(60.0));
        add(Sfx::HitMetal, mix(&[(&parts[0], 1.0, 0.0), (&parts[1], 0.7, 0.0), (&parts[2], 0.5, 0.0), (&parts[3], 0.35, 0.0), (&parts[4], 0.25, 0.0), (&tick, 0.8, 0.0)]), 0.7);
        // footstep: a soft muffled tap
        let c0 = r.r(600.0, 900.0);
        add(Sfx::Step, noise(&mut r, 0.06, c0, 300.0, 80.0, exp_dec(30.0)), 0.25);
        // skeleton rattle: a burst of tiny clicks
        let mut rat = vec![0.0; len(0.32)];
        for _ in 0..14 {
            let o = (r.f() * (rat.len() as f32 - 900.0)) as usize;
            let c = tone(0.018, r.r(1600.0, 2800.0), 1200.0, 1, exp_dec(25.0));
            for (i, v) in c.iter().enumerate() {
                rat[o + i] += v * r.r(0.4, 1.0);
            }
        }
        add(Sfx::Rattle, rat, 0.45);
        // coin: two bright pings
        let p1 = tone(0.25, 1760.0 * r.r(0.97, 1.03), 1760.0, 0, exp_dec(9.0));
        let p2 = tone(0.3, 2349.0, 2349.0, 0, exp_dec(8.0));
        add(Sfx::Coin, mix(&[(&p1, 1.0, 0.0), (&p2, 0.8, 0.06)]), 0.4);
    }
    for _ in 0..2 {
        // severing: a crunch, a squelch and a heavy thud
        let crunch = noise(&mut r, 0.12, 4000.0, 1500.0, 500.0, exp_dec(18.0));
        let sq = noise(&mut r, 0.3, 900.0, 300.0, 60.0, |x| (1.0 - x) * (0.6 + 0.4 * (x * 60.0).sin()));
        let thud = tone(0.2, 90.0, 45.0, 0, exp_dec(12.0));
        add(Sfx::Sever, mix(&[(&crunch, 0.8, 0.0), (&sq, 1.0, 0.03), (&thud, 0.9, 0.0)]), 0.9);
        // zombie groan: a buzzy voice through a moving vowel filter, with a wobble
        let f = r.r(80.0, 105.0);
        let n = len(0.9);
        let mut ph = 0.0f32;
        let (mut a, mut bq) = (0.0f32, 0.0f32);
        let g: Vec<f32> = (0..n)
            .map(|i| {
                let x = i as f32 / n as f32;
                ph = (ph + f * (1.0 + 0.04 * (t(i) * 7.0).sin()) / SR as f32).fract();
                let saw = ph * 2.0 - 1.0;
                let k = lp_k(400.0 + 500.0 * (x * 3.1416).sin());
                a += (saw - a) * k;
                bq += (a - bq) * k;
                bq * (x * 3.1416).sin()
            })
            .collect();
        add(Sfx::Groan, drive(g, 3.0), 0.45);
        // arrow hit: a short thock
        let th = tone(0.07, 380.0, 180.0, 1, exp_dec(30.0));
        let sp = noise(&mut r, 0.05, 3000.0, 1500.0, 600.0, exp_dec(40.0));
        add(Sfx::ArrowHit, mix(&[(&th, 1.0, 0.0), (&sp, 0.6, 0.0)]), 0.55);
        // wood breaking
        let w1 = noise(&mut r, 0.18, 2500.0, 800.0, 200.0, exp_dec(14.0));
        let w2 = tone(0.1, 260.0, 140.0, 1, exp_dec(25.0));
        add(Sfx::Wood, mix(&[(&w1, 1.0, 0.0), (&w2, 0.7, 0.0)]), 0.6);
        // stone crumbling
        let mut cr = noise(&mut r, 0.5, 1800.0, 400.0, 80.0, exp_dec(5.0));
        for i in 0..cr.len() {
            if r.f() < 0.002 {
                for j in 0..200.min(cr.len() - i) {
                    cr[i + j] += r.n() * (1.0 - j as f32 / 200.0) * 0.8;
                }
            }
        }
        add(Sfx::Crumble, cr, 0.6);
    }
    // lightning: a sizzling crackle with a falling zap
    for _ in 0..2 {
        let n = len(0.4);
        let gate: Vec<f32> = (0..n).map(|i| if (i / 300) % 3 != 0 || r.f() < 0.3 { 1.0 } else { 0.1 }).collect();
        let cr = noise(&mut r, 0.4, 7000.0, 3000.0, 1200.0, exp_dec(6.0));
        let cr: Vec<f32> = cr.iter().zip(gate.iter()).map(|(a, g)| a * g).collect();
        let zap = tone(0.25, 2400.0, 300.0, 3, exp_dec(12.0));
        let boom = tone(0.3, 70.0, 40.0, 0, exp_dec(8.0));
        add(Sfx::Zap, mix(&[(&cr, 1.0, 0.0), (&zap, 0.35, 0.0), (&boom, 0.6, 0.0)]), 0.8);
    }
    // thunder: a long low rumble that swells and rolls away
    {
        let rum = noise(&mut r, 2.4, 300.0, 90.0, 25.0, |x| (if x < 0.06 { x / 0.06 } else { (-(x - 0.06) * 2.5).exp() }) * (0.7 + 0.3 * (x * 23.0).sin()));
        let crack = noise(&mut r, 0.15, 6000.0, 2000.0, 800.0, exp_dec(20.0));
        add(Sfx::Thunder, mix(&[(&rum, 1.0, 0.0), (&crack, 0.7, 0.0)]), 0.9);
    }
    // fireball: a roaring whoosh
    {
        let roar = noise(&mut r, 0.45, 500.0, 1800.0, 80.0, attack_dec(0.15, 3.0));
        let crack = noise(&mut r, 0.45, 5000.0, 5000.0, 2000.0, |x| if (x * 40.0).fract() < 0.15 { 0.6 } else { 0.05 });
        add(Sfx::Fireball, mix(&[(&roar, 1.0, 0.0), (&crack, 0.4, 0.0)]), 0.6);
    }
    // explosion: a deep thump, a roar and falling debris
    for _ in 0..2 {
        let thump = tone(0.5, 110.0, 32.0, 0, exp_dec(6.0));
        let roar = noise(&mut r, 1.1, 2400.0, 150.0, 30.0, exp_dec(3.5));
        let deb = noise(&mut r, 0.8, 4000.0, 1500.0, 900.0, |x| (x * 50.0).sin().abs() * (1.0 - x) * 0.5);
        add(Sfx::Explosion, drive(mix(&[(&thump, 1.0, 0.0), (&roar, 1.0, 0.0), (&deb, 0.4, 0.1)]), 1.6), 0.95);
    }
    // acid: a wet splat into a sizzle
    {
        let splat = noise(&mut r, 0.08, 1500.0, 500.0, 100.0, exp_dec(25.0));
        let sz = noise(&mut r, 0.5, 9000.0, 6000.0, 2500.0, |x| (1.0 - x) * (0.5 + 0.5 * ((x * 90.0).sin() > 0.3) as i32 as f32));
        add(Sfx::Acid, mix(&[(&splat, 1.0, 0.0), (&sz, 0.6, 0.04)]), 0.55);
    }
    // frost: a cold shimmer of high bells over a whisper
    {
        let mut parts = vec![];
        for k in 0..7 {
            parts.push((tone(0.5, 2000.0 + k as f32 * 370.0 + r.r(0.0, 120.0), 2000.0 + k as f32 * 370.0, 0, exp_dec(6.0)), 0.3, k as f32 * 0.03));
        }
        let wh = noise(&mut r, 0.6, 6000.0, 3000.0, 1500.0, attack_dec(0.2, 4.0));
        let mut v: Vec<(&[f32], f32, f32)> = parts.iter().map(|(p, g, s)| (p.as_slice(), *g, *s)).collect();
        v.push((&wh, 0.5, 0.0));
        add(Sfx::Frost, mix(&v), 0.55);
    }
    // shatter: glass-like tinks scattering
    {
        let mut out = vec![0.0; len(0.6)];
        for _ in 0..22 {
            let o = (r.f().powi(2) * (out.len() as f32 - 9000.0)) as usize;
            let c = tone(0.15, r.r(2500.0, 6000.0), r.r(2400.0, 5800.0), 0, exp_dec(18.0));
            for (i, v) in c.iter().enumerate() {
                out[o + i] += v * r.r(0.3, 1.0);
            }
        }
        let crash = noise(&mut r, 0.2, 8000.0, 4000.0, 2500.0, exp_dec(14.0));
        add(Sfx::Shatter, mix(&[(&out, 1.0, 0.0), (&crash, 0.7, 0.0)]), 0.7);
    }
    // the brute's roar and the king's: low distorted voices
    {
        let g = tone(0.75, 75.0, 58.0, 2, |x| (x * 3.1416).sin());
        let n = noise(&mut r, 0.75, 700.0, 300.0, 60.0, |x| (x * 3.1416).sin());
        add(Sfx::Roar, drive(mix(&[(&g, 0.8, 0.0), (&n, 0.6, 0.0)]), 2.5), 0.7);
        let g = tone(1.5, 60.0, 42.0, 2, |x| (x * 3.1416).sin().powf(0.6));
        let g2 = tone(1.5, 90.0, 63.0, 2, |x| (x * 3.1416).sin());
        let n = noise(&mut r, 1.5, 500.0, 200.0, 40.0, |x| (x * 3.1416).sin());
        add(Sfx::KingRoar, drive(mix(&[(&g, 0.8, 0.0), (&g2, 0.4, 0.0), (&n, 0.7, 0.0)]), 2.2), 0.85);
    }
    // bow: a plucked string and the whizz of the arrow
    {
        let pl = tone(0.2, 220.0, 180.0, 1, exp_dec(14.0));
        let wz = noise(&mut r, 0.25, 3000.0, 1500.0, 900.0, |x| (x * 3.1416).sin());
        add(Sfx::Bow, mix(&[(&pl, 1.0, 0.0), (&wz, 0.5, 0.03)]), 0.5);
    }
    // potion: bubbling gulps
    {
        let mut v = vec![];
        for k in 0..4 {
            v.push(tone(0.07, 300.0 + k as f32 * 70.0, 600.0 + k as f32 * 90.0, 0, exp_dec(10.0)));
        }
        add(Sfx::Potion, mix(&[(&v[0], 1.0, 0.0), (&v[1], 0.9, 0.09), (&v[2], 0.8, 0.18), (&v[3], 0.8, 0.27)]), 0.45);
    }
    // level up: a rising major arpeggio that rings
    {
        let notes = [523.25, 659.25, 783.99, 1046.5, 1318.5];
        let v: Vec<Vec<f32>> = notes.iter().map(|&f| tone(0.9, f, f, 1, exp_dec(4.0))).collect();
        add(Sfx::LevelUp, mix(&[(&v[0], 0.8, 0.0), (&v[1], 0.8, 0.08), (&v[2], 0.8, 0.16), (&v[3], 0.8, 0.24), (&v[4], 0.7, 0.32)]), 0.5);
    }
    // the player hurt: a grunt and a hit
    {
        let gr = tone(0.18, 170.0, 120.0, 2, attack_dec(0.1, 8.0));
        let hit = noise(&mut r, 0.1, 2000.0, 600.0, 150.0, exp_dec(25.0));
        add(Sfx::Hurt, mix(&[(&drive(gr, 2.0), 0.6, 0.0), (&hit, 1.0, 0.0)]), 0.7);
    }
    // death: a long falling tone into silence
    {
        let d = tone(1.6, 330.0, 82.0, 1, |x| 1.0 - x);
        let d2 = tone(1.6, 220.0, 55.0, 0, |x| 1.0 - x);
        add(Sfx::Death, mix(&[(&d, 0.6, 0.0), (&d2, 0.8, 0.0)]), 0.6);
    }
    // ui click, travel whoosh, waypoint chime, dodge, whirlwind
    add(Sfx::Click, tone(0.05, 1400.0, 900.0, 1, exp_dec(30.0)), 0.3);
    add(Sfx::Travel, noise(&mut r, 0.9, 300.0, 2500.0, 60.0, |x| (x * 3.1416).sin()), 0.5);
    {
        let a = tone(1.2, 880.0, 880.0, 0, exp_dec(3.0));
        let b2 = tone(1.2, 1318.5, 1318.5, 0, exp_dec(3.0));
        let c = tone(1.2, 1760.0, 1760.0, 0, exp_dec(3.5));
        add(Sfx::Chime, mix(&[(&a, 0.7, 0.0), (&b2, 0.6, 0.12), (&c, 0.5, 0.24)]), 0.45);
    }
    add(Sfx::Dodge, noise(&mut r, 0.16, 700.0, 1600.0, 300.0, |x| (x * 3.1416).sin()), 0.35);
    add(Sfx::Whirl, noise(&mut r, 0.5, 800.0, 2000.0, 300.0, |x| (x * 3.1416).sin() * (0.6 + 0.4 * (x * 30.0).sin())), 0.55);
    b
}
