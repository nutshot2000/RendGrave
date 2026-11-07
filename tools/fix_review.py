# Review pass: bug fixes, dead-code cleanup, pause screen, minimap.
import re


def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


# ---------------- game.rs ----------------
edit('src/game.rs', [
    # dead code
    ("""                let burning_wall = false;
""", ""),
    ("""                let _ = burning_wall;
""", ""),
    ("""        let under = {
            let (x, y) = (p.x, p.y);
            let _ = (x, y);
            [false; 6]
        };
        let _ = under;
""", ""),
    ("""            let fa = if p.face < 0.0 { std::f32::consts::PI - p.swing_a.max(-3.0) } else { 0.0 };
            let _ = fa;
""", ""),
    ("""pub const SPELL_NAMES: [&str; 4] = ["LIGHTNING", "FIREBALL", "ACID SHOT", "FROST NOVA"];
pub const SPELL_COST: [f32; 4] = [14.0, 20.0, 10.0, 28.0];
pub const SPELL_CD: [i32; 4] = [22, 36, 16, 80];
""", ""),
    ("    pub cleared: bool,\n", ""),
    ("            cleared: false,\n", ""),
    # brute slam: index the scorch map safely
    ("""                                        if rnd() < 0.3 && !self.world.solid(sx as i32 + dx, sy as i32 + dy) {
                                            let i2 = (sy as usize + dy as usize).min(H - 1) * W + (sx as usize + dx as usize).min(W - 1);
                                            self.world.scorch[i2] = self.world.scorch[i2].saturating_add(40);
                                        }""", """                                        let (cx, cy) = (sx as i32 + dx, sy as i32 + dy);
                                        if rnd() < 0.3 && World::inb(cx, cy) && !self.world.solid(cx, cy) {
                                            let i2 = cy as usize * W + cx as usize;
                                            self.world.scorch[i2] = self.world.scorch[i2].saturating_add(40);
                                        }"""),
    # spell keys past the end of the bar are ignored instead of resetting the selection
    ("""        if let Some(s) = inp.sel {
            p.sel = s;
        }""", """        if let Some(s) = inp.sel {
            if s < p.sk.bar().len() {
                p.sel = s;
            }
        }"""),
    # pause screen + no accidental swing/cast when a menu closes with a button held
    ("""    pub tree: bool,
    pub tree_hover: Option<usize>,
}""", """    pub tree: bool,
    pub tree_hover: Option<usize>,
    pub paused: bool,
    pub hold_block: bool,
}"""),
    ("""            tree: false,
            tree_hover: None,
""", """            tree: false,
            tree_hover: None,
            paused: false,
            hold_block: false,
"""),
    ("""        if (inp.tree && !self.prev.tree && !self.player.dead) || (inp.close && !self.prev.close && self.tree) {
            self.tree = !self.tree;
        }
        if self.tree {""", """        if inp.tree && !self.prev.tree && !self.player.dead {
            self.tree = !self.tree;
            self.hold_block = true;
        } else if inp.close && !self.prev.close {
            if self.tree {
                self.tree = false;
            } else if !self.player.dead {
                self.paused = !self.paused;
            }
            self.hold_block = true;
        }
        if self.paused && !self.tree {
            self.prev = inp.clone();
            return;
        }
        if self.tree {"""),
    ("""        self.update_player(inp);""", """        let mut inp2 = inp.clone();
        if self.hold_block {
            if inp.lmb || inp.rmb {
                inp2.lmb = false;
                inp2.rmb = false;
            } else {
                self.hold_block = false;
            }
        }
        self.update_player(&inp2);"""),
    # gibs that never settle are eventually laid down anyway
    ("""    pub frozen: bool,
    pub fx: bool,
}""", """    pub frozen: bool,
    pub fx: bool,
    pub age: i32,
}"""),
    ("""                frozen,
                fx: true,
            });""", """                frozen,
                fx: true,
                age: 0,
            });"""),
    ("""            let g = &mut self.gibs[k];
            g.vz -= 0.16;""", """            let g = &mut self.gibs[k];
            g.age += 1;
            if g.age > 900 {
                g.rest = 999;
            }
            g.vz -= 0.16;"""),
])

# ---------------- spells.rs: respawns never land in ponds or acid ----------------
edit('src/spells.rs', [
    ("""            if d < 260.0 || self.world.solid(x as i32, y as i32) || self.world.aux[y as usize * W + x as usize] & AUX_INDOOR != 0 {""",
     """            if d < 260.0 || self.world.at(x as i32, y as i32) != E || self.world.aux[y as usize * W + x as usize] & AUX_INDOOR != 0 {"""),
    ("""                if self.world.solid(ex as i32, ey as i32) {
                    continue;
                }""", """                if self.world.at(ex as i32, ey as i32) != E {
                    continue;
                }"""),
])

# ---------------- render.rs ----------------
edit('src/render.rs', [
    ("""    for b in &g.bolts {
        let _ = b;
    }
""", ""),
    # pieces in the air are lit at their real height
    ("let dy = (oy - sy as f32).max(0.0) + z0 * 0.0;", "let dy = (oy - sy as f32).max(0.0) + z0;"),
])

# ---------------- gfx.rs: tiny / minimised windows ----------------
edit('src/gfx.rs', [
    ("""    pub fn resize(&mut self, w: u32, h: u32) {
        let w = w.max(1);""", """    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        let w = w.max(1);"""),
    ("((sw - self.rw * self.scale) / 2) as f32, ((sh - self.rh * self.scale) / 2) as f32]",
     "(sw.saturating_sub(self.rw * self.scale) / 2) as f32, (sh.saturating_sub(self.rh * self.scale) / 2) as f32]"),
])

# ---------------- hud.rs: minimap + pause overlay ----------------
edit('src/hud.rs', [
    ("use crate::skills::*;", "use crate::skills::*;\nuse crate::world::*;"),
    ("""    if g.tree {
        draw_tree(g, &mut u, fw, fh);
    }""", """    minimap(g, &mut u, fw, cx, cy);
    if g.paused && !g.tree {
        u.rect(0, 0, fw, fh, [6, 3, 8], 170);
        u.text_c(fw / 2, fh / 3, "PAUSED", [240, 214, 150], 3);
        u.text_c(fw / 2, fh / 3 + 22, "ESC RESUME    K SKILL TREE    CLOSE THE WINDOW TO QUIT", [200, 190, 190], 1);
    }
    if g.tree {
        draw_tree(g, &mut u, fw, fh);
    }"""),
    ("""fn draw_tree(""", """/// a small map of the whole world, top right
fn minimap(g: &Game, u: &mut Ui, fw: i32, cx: i32, cy: i32) {
    let (mw, mh) = (84, 63);
    let (ox, oy) = (fw - mw - 8, 26);
    let sx = W as f32 / mw as f32;
    let sy = H as f32 / mh as f32;
    u.rect(ox - 2, oy - 2, mw + 4, mh + 4, [70, 58, 66], 220);
    for y in 0..mh {
        for x in 0..mw {
            let (wx, wy) = ((x as f32 * sx) as usize, (y as f32 * sy) as usize);
            let i = wy * W + wx;
            let m = g.world.m[i];
            let c = match m {
                TREE => [24, 52, 28],
                STONE => [86, 84, 92],
                BRICK => [120, 104, 112],
                WOOD => [110, 80, 50],
                WATER | ICE => [50, 90, 150],
                ACID => [100, 190, 50],
                FIRE => [255, 140, 40],
                OIL => [30, 26, 26],
                _ => match g.world.g[i] {
                    FLOOR => [60, 56, 64],
                    DIRT | MUD => [70, 56, 42],
                    BURNT => [30, 26, 26],
                    _ => [36, 60, 34],
                },
            };
            u.px(ox + x, oy + y, c, 210);
        }
    }
    // view rectangle
    let (vw, vh) = g.view;
    let (rx, ry) = (ox + (cx as f32 / sx) as i32, oy + (cy as f32 / sy) as i32);
    let (rw, rh) = ((vw / sx) as i32, (vh / sy) as i32);
    for k in 0..rw {
        u.px(rx + k, ry, [200, 190, 170], 140);
        u.px(rx + k, ry + rh, [200, 190, 170], 140);
    }
    for k in 0..rh {
        u.px(rx, ry + k, [200, 190, 170], 140);
        u.px(rx + rw, ry + k, [200, 190, 170], 140);
    }
    for e in &g.enemies {
        if e.alive {
            let (x, y) = (ox + (e.x / sx) as i32, oy + (e.y / sy) as i32);
            if x >= ox && y >= oy && x < ox + mw && y < oy + mh {
                u.px(x, y, if e.k == crate::body::Kind::Brute { [255, 150, 40] } else { [230, 50, 50] }, 255);
            }
        }
    }
    let (px, py) = (ox + (g.player.x / sx) as i32, oy + (g.player.y / sy) as i32);
    for &(dx, dy) in &[(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        u.px(px + dx, py + dy, [255, 255, 255], 255);
    }
}

fn draw_tree("""),
])

# ---------------- TAU everywhere instead of 6.283 ----------------
for path in ['src/game.rs', 'src/spells.rs', 'src/render.rs', 'src/hud.rs']:
    s = open(path, encoding='utf8').read()
    s = s.replace('* 6.283', '* std::f32::consts::TAU').replace('/ 48.0 * 6.283', '/ 48.0 * std::f32::consts::TAU')
    s = s.replace('a0 + 6.283', 'a0 + std::f32::consts::TAU').replace('6.283', 'std::f32::consts::TAU')
    open(path, 'w', encoding='utf8').write(s)

# ---------------- README controls ----------------
edit('README.md', [
    ("""| 1-4 | Lightning, Fireball, Acid shot, Frost nova |""", """| 1-9 / mouse wheel | Choose a spell from the spell bar |
| K or Tab | Skill tree (spend skill points) |"""),
    ("""| Esc | Quit |""", """| Esc | Pause (or close the skill tree). Close the window to quit |"""),
    ("""    cargo run --release -- --lab lab          # enemy lineup, one effect each, close-up screenshots""",
     """    cargo run --release -- --lab lab          # enemy lineup, one effect each, close-up screenshots
    cargo run --release -- --spells spl       # every spell unlocked and cast, plus the skill tree screen"""),
])
print("review fixes applied")
