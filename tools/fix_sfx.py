def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/audio.rs', [("    Wood,\n    Crumble,\n}", "    Wood,\n    Crumble,\n    Fuse,\n    Slip,\n    ChestOpen,\n    Shrine,\n    Shout,\n    Splash,\n}")])

edit('src/synth.rs', [("""    add(Sfx::Whirl, noise(&mut r, 0.5, 800.0, 2000.0, 300.0, |x| (x * 3.1416).sin() * (0.6 + 0.4 * (x * 30.0).sin())), 0.55);
    b""", """    add(Sfx::Whirl, noise(&mut r, 0.5, 800.0, 2000.0, 300.0, |x| (x * 3.1416).sin() * (0.6 + 0.4 * (x * 30.0).sin())), 0.55);
    for k in 0..3 {
        // a lit fuse: bright crackling hiss
        let s = 31.0 + k as f32 * 7.0;
        let hiss = noise(&mut r, 0.3, 3500.0, 5200.0, 1800.0, move |x| (0.55 + 0.45 * ((x * s).sin() * (x * s * 2.7).cos()).abs()) * (x * 3.1416).sin().sqrt());
        let pop = noise(&mut r, 0.05, 2500.0, 900.0, 600.0, exp_dec(40.0));
        add(Sfx::Fuse, mix(&[(&hiss, 1.0, 0.0), (&pop, 0.7, 0.03 + k as f32 * 0.07)]), 0.4);
    }
    for _ in 0..2 {
        // feet going out on oil: a wet squelch, then the body hitting the floor
        let sq = noise(&mut r, 0.22, 1400.0, 300.0, 120.0, exp_dec(7.0));
        let thud = tone(0.22, r.r(85.0, 100.0), 45.0, 0, exp_dec(9.0));
        add(Sfx::Slip, mix(&[(&sq, 0.8, 0.0), (&thud, 1.0, 0.13)]), 0.6);
    }
    {
        // a chest lid: a creaking hinge, a latch, a soft thump
        let creak = drive(tone(0.42, 170.0, 260.0, 2, |x| (x * 3.1416).sin() * (0.6 + 0.4 * (x * 70.0).sin())), 1.6);
        let latch = tone(0.04, 2300.0, 1700.0, 1, exp_dec(25.0));
        let thump = tone(0.16, 120.0, 70.0, 0, exp_dec(12.0));
        add(Sfx::ChestOpen, mix(&[(&latch, 0.7, 0.0), (&creak, 0.6, 0.03), (&thump, 0.9, 0.42)]), 0.55);
    }
    {
        // a shrine: a deep bell and a shimmer of light
        let b1 = tone(2.0, 196.0, 196.0, 0, exp_dec(2.2));
        let b2 = tone(2.0, 294.0, 294.0, 0, exp_dec(2.6));
        let b3 = tone(1.6, 523.0, 523.0, 0, exp_dec(3.2));
        let b4 = tone(1.2, 784.0, 784.0, 0, exp_dec(4.0));
        let sh = noise(&mut r, 1.2, 6000.0, 9000.0, 4000.0, |x| (x * 3.1416).sin() * 0.5);
        add(Sfx::Shrine, mix(&[(&b1, 1.0, 0.0), (&b2, 0.6, 0.0), (&b3, 0.45, 0.05), (&b4, 0.3, 0.1), (&sh, 0.25, 0.1)]), 0.6);
    }
    for k in 0..3 {
        // a bandit's shout: a rough voiced bark
        let f0 = 190.0 + k as f32 * 25.0;
        let v = drive(tone(0.26, f0, f0 * 0.72, 2, attack_dec(0.12, 5.0)), 2.2);
        let v2 = tone(0.26, f0 * 2.4, f0 * 1.9, 1, attack_dec(0.12, 6.0));
        let br = noise(&mut r, 0.26, 1200.0, 800.0, 300.0, attack_dec(0.1, 6.0));
        add(Sfx::Shout, mix(&[(&v, 0.7, 0.0), (&v2, 0.35, 0.0), (&br, 0.5, 0.0)]), 0.5);
    }
    for _ in 0..2 {
        // a barrel splitting: wood, then liquid glugging out
        let glug = noise(&mut r, 0.5, 700.0, 250.0, 80.0, |x| (1.0 - x) * (0.5 + 0.5 * (x * 46.0).sin()));
        let crack = noise(&mut r, 0.08, 3000.0, 1200.0, 400.0, exp_dec(30.0));
        add(Sfx::Splash, mix(&[(&crack, 0.8, 0.0), (&glug, 1.0, 0.04)]), 0.6);
    }
    b""")])

edit('src/barrels.rs', [
    ("""                    if f % 12 == 0 {
                        self.snd(Sfx::Zap, x, y, 0.25);
                    }""", """                    if f % 16 == 0 {
                        self.snd(Sfx::Fuse, x, y, 0.55);
                    }"""),
    ("self.snd(if k == 2 { Sfx::Acid } else { Sfx::HitFlesh }, x, y, 0.7);", "self.snd(if k == 2 { Sfx::Acid } else { Sfx::Splash }, x, y, 0.8);"),
])
s = open('src/game.rs', encoding='utf8').read()
a = "                self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.9 });"
assert s.count(a) == 1
s = s.replace(a, "                self.sounds.push(Snd { k: Sfx::Slip, x: p.x, y: p.y, v: 0.9 });")
a = "                self.sounds.push(Snd { k: Sfx::Dodge, x: sx3, y: sy3, v: 0.7 });"
assert s.count(a) == 1
s = s.replace(a, "                self.sounds.push(Snd { k: Sfx::Slip, x: sx3, y: sy3, v: 0.8 });")
a = "                    Kind::Bandit | Kind::Crossbow => Sfx::HitMetal,"
assert s.count(a) == 1
s = s.replace(a, "                    Kind::Bandit | Kind::Crossbow => Sfx::Shout,")
open('src/game.rs', 'w', encoding='utf8').write(s)
edit('src/props.rs', [
    ("                self.snd(Sfx::Chime, sx, sy, 1.0);\n                self.snd(Sfx::LevelUp, sx, sy, 0.5);", "                self.snd(Sfx::Shrine, sx, sy, 1.0);\n                self.snd(Sfx::Chime, sx, sy, 0.4);"),
    ("        self.snd(Sfx::Wood, x, y, 0.9);", "        self.snd(Sfx::ChestOpen, x, y, 1.0);"),
])
print("sfx ok")
