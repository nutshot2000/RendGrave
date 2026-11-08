def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/areagen.rs', [("""    /// chimney position on the roof (houses only)""", """    pub fn pitch(&self) -> f32 {
        match self.k {
            BKind::House => 0.72,
            BKind::CryptGate => 0.6,
            _ => 0.5,
        }
    }
    /// height of the roof ridge above the wall tops
    pub fn roof_r(&self) -> f32 {
        self.h as f32 * self.pitch()
    }
    /// ridge height above the wall tops at column x (hipped ends slope down)
    pub fn roof_z(&self, x: i32) -> f32 {
        let ov = 3;
        let r = self.roof_r();
        if self.k == BKind::CryptGate {
            return r;
        }
        let hd = self.h as f32 / 2.0 + ov as f32;
        let d = ((x - (self.x0 - ov)).min(self.x0 + self.w + ov - 1 - x)) as f32 + 0.5;
        r.min(r / hd * d)
    }
    /// chimney position on the roof (houses only)""")])

edit('src/game.rs', [("""            let chims: Vec<(i32, i32, i32)> = self.buildings.iter().filter_map(|b| b.chimney().map(|(x, y)| (x, y, b.h))).collect();
            for (x, y, h) in chims {
                let z = 44.0 + h as f32 * 0.75 + 4.0;""", """            let chims: Vec<(i32, i32, f32)> = self.buildings.iter().filter_map(|b| b.chimney().map(|(x, y)| (x, y, b.roof_z(x + 3)))).collect();
            for (x, y, rz) in chims {
                let z = 44.0 + rz + 15.0;""")])

R = 'src/render.rs'
s = open(R, encoding='utf8').read()
a = s.index("    let ov = 3;\n    let pitch = match b.k {")
z = s.index("    // a cross on tomb ridges, finials on the crypt gate")
s = s[:a] + r'''    let ov = 3;
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
''' + s[z:]
open(R, 'w', encoding='utf8').write(s)
print("roofs ok")
