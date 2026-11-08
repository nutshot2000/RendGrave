p = 'src/hud.rs'
s = open(p, encoding='utf8').read()
# swap the layout functions
a = s.index('/// layout of the skill tree screen')
b = s.index('pub fn node_at(')
b = s.index('\n}\n', b) + 3
s = s[:a] + open('tools/tree_ui.rs', encoding='utf8').read() + s[b:]
# swap the tree screen
a = s.index('fn draw_tree(')
b = s.index('pub fn draw(g: &Game')
s = s[:a] + open('tools/tree_draw.rs', encoding='utf8').read() + s[b:]
# icons for the new skills
old = """        Sk::Fortune => {
            disc(u, x + 8, y + 8, 5.5, [220, 170, 50]);
            disc(u, x + 8, y + 8, 3.5, [255, 230, 130]);
        }
"""
new = old + """        Sk::Conductor => {
            drop(u, x + 5, y + 4, 5, [80, 140, 230]);
            bolt(u, x + 9, y + 2, 2, [255, 245, 170]);
        }
        Sk::Overload => {
            disc(u, x + 8, y + 8, 6.0, [60, 70, 140]);
            bolt(u, x + 5, y + 1, 2, [255, 255, 255]);
        }
        Sk::StaticField => {
            for k in 0..5 {
                let a = k as f32 * 1.26 + t as f32 * 0.05;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 6.0) as i32, [200, 220, 255], 255);
            }
            bolt(u, x + 5, y + 3, 2, [180, 200, 255]);
        }
        Sk::StormShield => {
            disc(u, x + 8, y + 8, 6.5, [90, 120, 220]);
            disc(u, x + 8, y + 8, 4.5, [24, 20, 40]);
            bolt(u, x + 5, y + 3, 2, [220, 235, 255]);
        }
        Sk::EyeOfStorm => {
            disc(u, x + 8, y + 8, 6.5, [70, 80, 120]);
            disc(u, x + 8, y + 8, 3.0, [255, 245, 170]);
            disc(u, x + 8, y + 8, 1.2, [20, 20, 30]);
        }
        Sk::Searing => {
            for k in 0..5 {
                u.rect(x + 3 + k * 2, y + 12 - (k % 3) * 2, 2, 3 + (k % 3) * 2, fire(k), 255);
            }
            u.rect(x + 4, y + 2, 8, 2, [255, 240, 200], 255);
        }
        Sk::Splinter => {
            disc(u, x + 8, y + 8, 3.0, [255, 200, 90]);
            for &(dx, dy) in &[(-5, -4), (5, -3), (0, 6)] {
                disc(u, x + 8 + dx, y + 8 + dy, 2.0, fire(dx + dy));
            }
        }
        Sk::Fireproof => {
            disc(u, x + 8, y + 8, 6.0, [120, 110, 120]);
            u.rect(x + 5, y + 6, 6, 5, fire(1), 255);
        }
        Sk::Combustion => {
            for k in 0..8 {
                let a = k as f32 * 0.785;
                u.rect(x + 8 + (a.cos() * 5.0) as i32, y + 8 + (a.sin() * 5.0) as i32, 2, 2, fire(k), 255);
            }
            disc(u, x + 8, y + 8, 2.5, [255, 240, 200]);
        }
        Sk::MeteorShower => {
            for &(dx, dy) in &[(3, 4), (9, 2), (6, 10)] {
                disc(u, x + dx + 2, y + dy + 2, 2.0, fire(dx));
                u.px(x + dx, y + dy, [255, 200, 120], 200);
            }
        }
        Sk::Caustic => {
            u.rect(x + 2, y + 10, 12, 4, [110, 210, 50], 255);
            for k in 0..3 {
                u.px(x + 4 + k * 4, y + 7 - (k % 2), [200, 255, 140], 255);
            }
        }
        Sk::PoisonCloud => {
            disc(u, x + 6, y + 9, 4.0, [70, 120, 40]);
            disc(u, x + 10, y + 7, 4.5, [90, 150, 50]);
            disc(u, x + 9, y + 11, 3.0, [110, 180, 60]);
        }
        Sk::AcidPools => {
            disc(u, x + 8, y + 10, 5.5, [70, 150, 40]);
            disc(u, x + 8, y + 10, 3.0, [140, 230, 70]);
            drop(u, x + 8, y + 1, 4, [140, 230, 70]);
        }
        Sk::RotBurst => {
            u.rect(x + 4, y + 9, 8, 3, [190, 180, 160], 255);
            for k in 0..6 {
                let a = k as f32 * 1.05;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 5.0) as i32, [140, 230, 60], 255);
            }
        }
        Sk::Plague => {
            for k in 0..6 {
                drop(u, x + 2 + k * 2, y + 2 + (k % 3) * 4, 3, [120, 220, 60]);
            }
        }
        Sk::Chill => {
            u.rect(x + 7, y + 2, 2, 12, [190, 230, 255], 255);
            u.rect(x + 2, y + 7, 12, 2, [190, 230, 255], 255);
        }
        Sk::IceArmor => {
            u.rect(x + 3, y + 3, 10, 10, [140, 200, 240], 255);
            u.rect(x + 5, y + 5, 6, 6, [210, 240, 255], 255);
        }
        Sk::Shatterstorm => {
            for k in 0..6 {
                let a = k as f32 * 1.05 + 0.3;
                u.rect(x + 8 + (a.cos() * 5.0) as i32, y + 8 + (a.sin() * 5.0) as i32, 2, 1, [210, 240, 255], 255);
            }
            disc(u, x + 8, y + 8, 1.5, [255, 255, 255]);
        }
        Sk::DeepFreeze => {
            disc(u, x + 8, y + 8, 6.0, [80, 140, 200]);
            disc(u, x + 8, y + 8, 3.5, [200, 235, 255]);
        }
        Sk::AbsoluteZero => {
            for k in 0..4 {
                let a = k as f32 * 0.785;
                let (cx, cy) = (a.cos() * 6.0, a.sin() * 6.0);
                for s in -6..=6 {
                    let f = s as f32 / 6.0;
                    u.px(x + 8 + (cx * f) as i32, y + 8 + (cy * f) as i32, [230, 245, 255], 255);
                }
            }
        }
        Sk::Cleave => {
            for k in 0..9 {
                let a = 3.4 + k as f32 * 0.32;
                u.px(x + 8 + (a.cos() * 7.0) as i32, y + 10 + (a.sin() * 7.0) as i32, [240, 240, 250], 255);
            }
        }
        Sk::Dismember => {
            u.rect(x + 3, y + 4, 4, 9, [222, 184, 152], 255);
            u.rect(x + 9, y + 4, 4, 9, [222, 184, 152], 255);
            u.rect(x + 7, y + 2, 2, 12, [200, 30, 40], 255);
        }
        Sk::Momentum => {
            for k in 0..3 {
                u.rect(x + 2 + k * 4, y + 4 + k, 3, 8 - k * 2, [230, 225, 215], 255);
            }
        }
        Sk::BladeFury => {
            for k in 0..12 {
                let a = k as f32 * 0.52 + t as f32 * 0.2;
                u.px(x + 8 + (a.cos() * 6.0) as i32, y + 8 + (a.sin() * 6.0) as i32, if k % 3 == 0 { [255, 255, 255] } else { [180, 180, 200] }, 255);
            }
            disc(u, x + 8, y + 8, 1.5, [200, 40, 40]);
        }
        Sk::Warlord => {
            u.rect(x + 3, y + 9, 10, 4, [220, 180, 60], 255);
            for k in 0..3 {
                u.rect(x + 3 + k * 4, y + 5 + (k % 2), 2, 4, [220, 180, 60], 255);
            }
        }
        Sk::Toughness => {
            u.rect(x + 3, y + 3, 10, 9, [150, 150, 165], 255);
            u.rect(x + 5, y + 12, 6, 2, [150, 150, 165], 255);
        }
        Sk::SecondWind => {
            u.rect(x + 6, y + 2, 4, 3, [180, 170, 160], 255);
            disc(u, x + 8, y + 10, 4.5, [220, 40, 60]);
        }
        Sk::Regeneration => {
            u.rect(x + 7, y + 3, 2, 10, [255, 90, 110], 255);
            u.rect(x + 3, y + 7, 10, 2, [255, 90, 110], 255);
        }
        Sk::LastStand => {
            u.rect(x + 7, y + 2, 2, 12, [230, 225, 215], 255);
            disc(u, x + 8, y + 10, 3.0, [200, 30, 40]);
        }
        Sk::Titan => {
            disc(u, x + 8, y + 5, 3.0, [240, 110, 120]);
            u.rect(x + 3, y + 8, 10, 6, [240, 110, 120], 255);
        }
"""
assert s.count(old) == 1
s = s.replace(old, new)
open(p, 'w', encoding='utf8').write(s)
print("tree ui ok")
