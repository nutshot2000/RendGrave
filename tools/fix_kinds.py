def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/body.rs', [
    ("""pub enum Kind {
    Player,
    Zombie,
    Skeleton,
    Brute,
}
""", """pub enum Kind {
    Player,
    Zombie,
    Skeleton,
    Brute,
    /// a zombie that lost its legs long ago
    Crawler,
    /// skeleton with a bow
    Archer,
    /// act I boss: a giant crowned skeleton in rusted armour
    King,
}
impl Kind {
    /// big, heavy enemies (bigger reach and footprint, resist knockback)
    pub fn big(self) -> bool {
        matches!(self, Kind::Brute | Kind::King)
    }
    /// made of bone: no blood, crumbles into bone chips
    pub fn bony(self) -> bool {
        matches!(self, Kind::Skeleton | Kind::Archer | Kind::King)
    }
}
"""),
    ("""pub fn humanoid(kind: Kind) -> Body {
    // sizes: legs, torso, arms, head (pixels)
    let (lw, ll, tw, th, aw, al, hs) = match kind {
        Kind::Player => (6, 20, 14, 19, 5, 17, 11),
        Kind::Zombie => (6, 21, 14, 19, 5, 19, 11),
        Kind::Skeleton => (5, 22, 13, 19, 4, 18, 10),
        Kind::Brute => (9, 22, 24, 23, 8, 22, 13),
    };""", """pub fn humanoid(orig: Kind) -> Body {
    // variants share a base body plan
    let kind = match orig {
        Kind::Crawler => Kind::Zombie,
        Kind::Archer | Kind::King => Kind::Skeleton,
        k => k,
    };
    let king = orig == Kind::King;
    // sizes: legs, torso, arms, head (pixels)
    let (lw, ll, tw, th, aw, al, hs) = if king {
        (8, 31, 21, 28, 6, 27, 15)
    } else {
        match kind {
            Kind::Player => (6, 20, 14, 19, 5, 17, 11),
            Kind::Zombie => (6, 21, 14, 19, 5, 19, 11),
            Kind::Skeleton => (5, 22, 13, 19, 4, 18, 10),
            _ => (9, 22, 24, 23, 8, 22, 13),
        }
    };"""),
    # the king wears a rusted breastplate over his ribs
    ("""        if sk {
            let spine = fx.abs() < 1.0;
            let ribs = y >= 2 && y < th * 6 / 10 && y % 3 != 1 && nx.abs() < 0.92 - (y as f32 / th as f32) * 0.2;""", """        if sk {
            if king && y >= 2 && y < th - 7 && nx.abs() < 0.82 {
                let rust = vary([128, 86, 58], x, y, s + 11, 0.3);
                let rivet = (y == 3 || y == th - 9) && x % 4 == 1;
                return Some((PM_METAL, if rivet { [190, 170, 130] } else { rust }, PM_BONE, bone));
            }
            let spine = fx.abs() < 1.0;
            let ribs = y >= 2 && y < th * 6 / 10 && y % 3 != 1 && nx.abs() < 0.92 - (y as f32 / th as f32) * 0.2;"""),
    ("""            Kind::Skeleton => {
                if eye {
                    return Some((PM_GLOW, [255, 70, 40], 0, [0; 3]));
                }""", """            Kind::Skeleton => {
                if king && y < 3 && (x % 3 == 0 || y == 2) {
                    return Some((PM_GLOW, [255, 200, 80], 0, [0; 3]));
                }
                if eye {
                    return Some((PM_GLOW, if king { [140, 200, 255] } else { [255, 70, 40] }, 0, [0; 3]));
                }"""),
    ("""    let weapon = match kind {""", """    let weapon = if orig == Kind::Archer {
        // a curved bow, held upright
        mk(PK::Weapon, 5, 20, (2.0, 10.0), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let t = y as f32 / 19.0;
            let bx = (3.6 - (t * std::f32::consts::PI).sin() * 3.4).round() as i32;
            if x == 4 - bx.min(4) || (x == 4 - bx.min(4) + 1 && (y == 0 || y == 19)) {
                return Some((PM_WOOD, vary([120, 84, 50], x, y, 41, 0.2), 0, [0; 3]));
            }
            if x == 4 {
                return Some((PM_CLOTH, [200, 196, 180], 0, [0; 3]));
            }
            None
        })
    } else if king {
        // a huge rusted greatsword
        mk(PK::Weapon, 6, 38, (3.0, 32.5), T_ARMF as i32, (aw as f32 / 2.0, al as f32 - 2.0), |x, y| {
            let fx = x as f32 - 2.5;
            if y >= 33 {
                return if fx.abs() < 1.0 { Some((PM_WOOD, [70, 46, 34], 0, [0; 3])) } else { None };
            }
            if y == 31 || y == 32 {
                return Some((PM_METAL, [150, 110, 60], 0, [0; 3]));
            }
            if fx.abs() < 2.0 && !(y == 0 && fx.abs() > 0.6) {
                let rust = h2(x, y, 98) < 0.45;
                return Some((PM_METAL, if rust { [130, 84, 54] } else if fx < 0.0 { [176, 172, 168] } else { [130, 128, 130] }, 0, [0; 3]));
            }
            None
        })
    } else {
        match kind {"""),
    ("""        Kind::Zombie => mk(PK::Weapon, 1, 1, (0.0, 0.0), T_ARMF as i32, (0.0, 0.0), |_, _| None),
    };""", """        _ => mk(PK::Weapon, 1, 1, (0.0, 0.0), T_ARMF as i32, (0.0, 0.0), |_, _| None),
        }
    };"""),
    ("""        leg_len: ll as f32,
        kind,
    };
    if kind == Kind::Zombie {
        b.parts[T_WEAPON].on = false;
    }""", """        leg_len: ll as f32,
        kind: orig,
    };
    if kind == Kind::Zombie {
        b.parts[T_WEAPON].on = false;
    }
    if orig == Kind::Crawler {
        b.parts[T_LEGF].on = false;
        b.parts[T_LEGB].on = false;
    }"""),
])
print("kinds ok")
