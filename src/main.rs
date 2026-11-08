// Rendgrave: a pixel-simulated action RPG.
// Run normally to play. `--showcase <dir>` plays a scripted demo and saves screenshots there.
mod areas;
mod shop;
mod talents;
mod nav;
mod barrels;
mod mapdump;
mod props;
mod gear;
mod items;
mod audio;
mod synth;
mod body;
mod areagen;
mod game;
mod gfx;
mod hud;
mod render;
mod rng;
mod skills;
mod spells;
mod world;

use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

struct Showcase {
    dir: String,
    log: Vec<String>,
    lab: bool,
    spells: bool,
    tour: bool,
    loot: bool,
    /// steps taken while the game is paused (the skill tree stops the clock)
    sub: i32,
}

struct App {
    gfx: Option<gfx::Gfx>,
    game: game::Game,
    input: game::Input,
    frame: Option<gfx::Frame>,
    last: Instant,
    acc: f64,
    mouse_px: (f64, f64),
    show: Option<Showcase>,
    lights: Vec<gfx::GLight>,
    fps_t: Instant,
    fps_n: u32,
    sel_pending: Option<usize>,
    audio: Option<audio::Audio>,
}

impl App {
    fn step_lab(&mut self) -> Option<String> {
        use body::*;
        let g = &mut self.game;
        let f = g.frame;
        let inp = &mut self.input;
        inp.lmb = false;
        inp.rmb = false;
        if f == 1 {
            for y in 600..740 {
                for x in 20..300 {
                    g.world.set(x, y, world::E);
                    g.world.g[y as usize * world::W + x as usize] = world::GRASS;
                }
            }
            g.world.splash(232.0, 690.0, world::WATER, 10.0, 0);
            g.enemies.clear();
            g.player.x = 60.0;
            g.player.y = 680.0;
            let kinds = [Kind::Zombie, Kind::Zombie, Kind::Zombie, Kind::Skeleton, Kind::Brute, Kind::Zombie];
            for (k, kind) in kinds.iter().enumerate() {
                let mut e = game::Game::new_enemy(*kind, 100.0 + k as f32 * 30.0, 680.0);
                e.stun = 100000;
                e.body.mirror = true;
                e.face = -1.0;
                g.enemies.push(e);
            }
            g.cam = (40.0, 680.0 - 26.0 - g.view.1 / 2.0);
            g.banner = None;
        }
        g.cam = (40.0, 680.0 - 26.0 - g.view.1 / 2.0);
        g.player.hp = 100.0;
        if f == 10 {
            // 0: acid on the feet
            g.enemies[0].body.each_in(0.0, -3.0, 6.0, |p, _, _| p.acid = 250);
            g.enemies[0].body.fx = true;
            // 1: sword through both legs
            g.enemies[1].body.cut(-30.0, -9.0, 30.0, -8.0, 60.0);
            // 2: sword through the waist
            g.enemies[2].body.cut(-30.0, -25.0, 30.0, -23.0, 90.0);
            // 5: one arm off
            g.enemies[5].body.cut(-30.0, -36.0, 30.0, -30.0, 14.0);
        }
        if f == 12 {
            let (sx, sy) = (g.enemies[3].x, g.enemies[3].y);
            inp.mouse = (sx - g.cam.0, sy - 30.0 - g.cam.1);
            let inp2 = inp.clone();
            g.cast_lightning(sx, sy - 30.0, &inp2);
            g.explode(g.enemies[4].x + 10.0, g.enemies[4].y, 20.0, 13.0);
        }
        let shots = [(9, "lab_00_before"), (14, "lab_01_hit"), (60, "lab_02_1s"), (160, "lab_03_2s"), (320, "lab_04_5s")];
        for (fr, name) in shots {
            if f == fr {
                return Some(name.to_string());
            }
        }
        None
    }
    /// gear test: every weapon type on a lineup, loot on the ground, the inventory screen
    fn step_loot(&mut self) -> Option<String> {
        use body::Kind;
        use items::*;
        let g = &mut self.game;
        let f = g.frame;
        if f == 1 {
            for y in 560..760 {
                for x in 20..440 {
                    g.world.set(x, y, world::E);
                    g.world.g[y as usize * world::W + x as usize] = world::GRASS;
                }
            }
            g.enemies.clear();
            for (k, kind) in [Kind::Zombie, Kind::Skeleton, Kind::Brute, Kind::Zombie, Kind::Skeleton].iter().enumerate() {
                let mut e = game::Game::new_enemy(*kind, 140.0 + k as f32 * 60.0, 660.0);
                e.stun = 100000;
                e.hp = 4000.0;
                e.maxhp = 4000.0;
                e.body.mirror = true;
                e.elite = k >= 3;
                g.enemies.push(e);
            }
            g.player.x = 70.0;
            g.player.y = 690.0;
            g.player.bag.clear();
            for l in [Legend::Stormcaller, Legend::Gravecleaver, Legend::WintersGrudge] {
                g.player.bag.push(legendary(l, 6));
            }
            for _ in 0..15 {
                g.player.bag.push(roll(6, 0.2));
            }
            for k in 0..5 {
                let it = if k == 0 { legendary(Legend::Pyre, 5) } else { roll(5, 0.25) };
                g.ground.push(gear::GroundItem { item: it, x: 60.0 + k as f32 * 14.0, y: 720.0 + (k % 2) as f32 * 8.0, z: 0.0, vz: 0.0, t: 100 });
            }
            g.banner = None;
        }
        g.cam = (30.0, 662.0 - 26.0 - g.view.1 / 2.0);
        g.player.hp = g.player.maxhp;
        let tests: [(u64, WKind, Option<Legend>, usize); 5] = [(20, WKind::Axe, None, 0), (60, WKind::Hammer, None, 1), (100, WKind::Spear, None, 2), (140, WKind::Dagger, None, 3), (180, WKind::Sword, Some(Legend::Stormcaller), 4)];
        for (at, wk, leg, ti) in tests {
            if f == at || f == at + 12 {
                let mut it = match leg {
                    Some(l) => legendary(l, 6),
                    None => {
                        let mut it = roll(6, 0.0);
                        it.slot = Slot::Weapon;
                        it.wk = wk;
                        it.name = format!("TEST {:?}", wk).to_uppercase();
                        it.legend = None;
                        it
                    }
                };
                it.base = 20;
                if f == at {
                    g.player.equip[0] = Some(it);
                    g.apply_gear();
                }
                let (ex, ey) = (g.enemies[ti].x, g.enemies[ti].y);
                g.player.x = ex - 22.0;
                g.player.y = ey;
                let mut inp = self.input.clone();
                inp.mouse = (ex - g.cam.0, ey - 12.0 - g.cam.1);
                g.weapon_hit(0.0, &inp);
            }
        }
        if f == 230 {
            g.player.x = 70.0;
            g.player.y = 690.0;
        }
        if f == 240 {
            g.inv_open = true;
            g.inv_hover = Some((1, 0));
            let (cx0, cy0, _) = hud::inv_cell(1, 1, g.view.0 as i32, g.view.1 as i32);
            self.input.mouse = (cx0 as f32 + 10.0, cy0 as f32 + 10.0);
            g.prev.mouse = self.input.mouse;
        }
        let shots = [(36, "g1_axe"), (76, "g2_hammer"), (116, "g3_spear"), (156, "g4_dagger_elite"), (196, "g5_stormcaller"), (236, "g6_ground_loot"), (240, "g7_inventory")];
        for (fr, name) in shots {
            if f == fr {
                return Some(name.to_string());
            }
        }
        None
    }
    /// visit every area and take a picture of each
    fn step_tour(&mut self) -> Option<String> {
        use areagen::AreaId;
        let g = &mut self.game;
        let f = g.frame as i32;
        g.player.hp = g.player.maxhp;
        let stops = [
            (AreaId::Ashford, "t01_ashford"),
            (AreaId::Pines, "t02_pines"),
            (AreaId::Graveyard, "t03_graveyard"),
            (AreaId::Fen, "t04_fen"),
            (AreaId::Ravenhold, "t05_ravenhold"),
            (AreaId::Crypt(1), "t06_crypt1"),
            (AreaId::Crypt(3), "t07_crypt3_boss"),
            (AreaId::Cellars(1), "t08_cellars1"),
            (AreaId::Cellars(2), "t09_cellars2_boss"),
        ];
        for (k, (a, name)) in stops.iter().enumerate() {
            let at = 1 + k as i32 * 70;
            if f == at {
                g.travel(*a);
                if !g.discovered.contains(a) {
                    g.discovered.push(*a);
                }
                for e in g.enemies.iter_mut() {
                    e.stun = 100000;
                }
                if matches!(a, AreaId::Crypt(3) | AreaId::Cellars(2)) {
                    if let Some(b) = g.enemies.iter().position(|e| e.k.boss()) {
                        let (bx, by) = (g.enemies[b].x, g.enemies[b].y);
                        g.player.x = bx - 70.0;
                        g.player.y = by + 10.0;
                        g.enemies[b].aggro = true;
                    }
                } else if let Some(b) = g.buildings.iter().find(|b| b.k != areagen::BKind::Ruin).copied() {
                    // stand in front of a building to check how it looks
                    g.player.x = b.door_c() as f32 + 34.0;
                    g.player.y = (b.y0 + b.h) as f32 + 34.0;
                } else if *a != AreaId::Ashford {
                    place_near_enemy(g, 80.0);
                }
                g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);
            }
            if f == at + 55 {
                return Some(name.to_string());
            }
        }
        let kb = 1 + stops.len() as i32 * 70;
        let end = kb + 130;
        if f == end {
            g.travel(AreaId::Pines);
            if let Some((wx, wy)) = g.waypoint {
                g.player.x = wx + 6.0;
                g.player.y = wy + 8.0;
            }
        }
        if f == end + 50 {
            // menus stop the clock, so count steps instead of frames from here
            g.wp_open = g.wp_open || g.merchant.is_none();
            let sh = self.show.as_mut().unwrap();
            sh.sub += 1;
            match sh.sub {
                1 => return Some("t14_waypoints".into()),
                3 => {
                    g.wp_open = false;
                    g.travel(AreaId::Ashford);
                    if let Some((mx, my)) = g.merchant {
                        g.player.x = mx - 10.0;
                        g.player.y = my + 24.0;
                        g.cam = (g.player.x - g.view.0 / 2.0, g.player.y - 26.0 - g.view.1 / 2.0);
                    }
                    g.banner = None;
                    // open the shop in the same step so the clock stays stopped
                    g.player.gold = 3000;
                    g.shop_open = true;
                }
                50 => return Some("t15_shop".into()),
                _ => {}
            }
            return None;
        }
        // a powder keg in the cellars with a crowd of bandits round it
        if f == kb {
            g.travel(AreaId::Cellars(1));
            if let Some(b) = g.barrels.iter().position(|b| b.k == 0) {
                let (bx, by) = (g.barrels[b].x, g.barrels[b].y);
                let mut n = 0;
                for e in g.enemies.iter_mut() {
                    if n < 7 && !e.k.boss() {
                        let a = n as f32 / 7.0 * std::f32::consts::TAU;
                        e.x = bx + a.cos() * 26.0;
                        e.y = by + 12.0 + a.sin() * 12.0;
                        e.stun = 100000;
                        n += 1;
                    } else {
                        e.stun = 100000;
                    }
                }
                // a slick of oil nearby, and an oil barrel next to the keg
                g.player.x = bx - 110.0;
                g.player.y = by + 20.0;
                g.cam = (bx - g.view.0 / 2.0, by - 26.0 - g.view.1 / 2.0);
            }
        }
        if f == kb + 30 {
            return Some("t10_keg_before".into());
        }
        if f == kb + 31 {
            if let Some(b) = g.barrels.iter_mut().find(|b| b.k == 0 && b.alive) {
                b.fuse = 2;
            }
        }
        if f == kb + 36 {
            return Some("t11_keg_blast".into());
        }
        if f == kb + 60 {
            return Some("t12_keg_after".into());
        }
        if f == kb + 120 {
            return Some("t13_keg_aftermath".into());
        }
        None
    }
    /// every new spell, cast at a lineup of enemies, plus the skill tree screen
    fn step_spells(&mut self) -> Option<String> {
        use body::*;
        use skills::*;
        let g = &mut self.game;
        let f = g.frame;
        let inp = &mut self.input;
        inp.lmb = false;
        inp.rmb = false;
        inp.tree = false;
        if f == 1 {
            for y in 560..760 {
                for x in 20..420 {
                    g.world.set(x, y, world::E);
                    g.world.g[y as usize * world::W + x as usize] = world::GRASS;
                }
            }
            g.world.splash(250.0, 640.0, world::WATER, 12.0, 0);
            g.enemies.clear();
            g.player.x = 70.0;
            g.player.y = 660.0;
            g.player.sk.level = 40;
            g.player.sk.points = 400;
            g.player.sk.tomes = [true; N];
            // several passes: some skills need ones later in the list
            for _ in 0..4 {
                for i in 0..N {
                    while g.player.sk.learn(i) {}
                }
            }
            g.recalc();
            let kinds = [Kind::Zombie, Kind::Skeleton, Kind::Zombie, Kind::Brute, Kind::Zombie, Kind::Skeleton, Kind::Zombie, Kind::Zombie];
            for (k, kind) in kinds.iter().enumerate() {
                let mut e = game::Game::new_enemy(*kind, 160.0 + (k % 4) as f32 * 34.0, 640.0 + (k / 4) as f32 * 30.0);
                e.stun = 100000;
                e.hp = 2000.0;
                e.maxhp = 2000.0;
                e.body.mirror = true;
                g.enemies.push(e);
            }
            g.banner = None;
        }
        g.cam = (40.0, 655.0 - 26.0 - g.view.1 / 2.0);
        g.player.hp = g.player.maxhp;
        g.player.mana = 999.0;
        // with the tree open the clock stops: page through every school
        if g.tree && f == 699 {
            let sh = self.show.as_mut().unwrap();
            sh.sub += 1;
            let k = sh.sub;
            let pages = [(0, "s11_tab_storm"), (2, "s12_tab_venom"), (3, "s13_tab_frost"), (4, "s14_tab_blade"), (5, "s15_tab_body")];
            for (n, (tab, name)) in pages.iter().enumerate() {
                let at = 4 + n as i32 * 4;
                if k == at {
                    g.tree_tab = *tab;
                    g.tree_hover = None;
                }
                if k == at + 2 {
                    return Some(name.to_string());
                }
            }
            if k == 30 {
                g.tree = false;
            }
            return None;
        }
        if f == 705 {
            // a fresh crowd for the new skills
            g.enemies.clear();
            g.fields.clear();
            for k in 0..8 {
                let mut e = game::Game::new_enemy(Kind::Zombie, 170.0 + (k % 4) as f32 * 30.0, 645.0 + (k / 4) as f32 * 26.0);
                e.stun = 100000;
                e.hp = 400.0;
                e.maxhp = 400.0;
                g.enemies.push(e);
            }
            g.player.x = 90.0;
            g.player.y = 660.0;
        }
        let at = |g: &game::Game, i: usize| (g.enemies[i].x, g.enemies[i].y);
        let mut cast = |g: &mut game::Game, s: Sk, x: f32, y: f32| {
            let i2 = inp.clone();
            g.cast(s, x, y, &i2);
        };
        match f {
            712 => cast(g, Sk::Fireball, 220.0, 655.0),
            732 => {
                for e in g.enemies.iter_mut() {
                    e.body.tint_all(|p| {
                        if p.m == body::PM_CLOTH || p.m == body::PM_FLESH {
                            p.burn = 200;
                        }
                    });
                }
            }
            740 => cast(g, Sk::Combustion, 0.0, 0.0),
            770 => cast(g, Sk::PoisonCloud, 230.0, 660.0),
            820 => {
                cast(g, Sk::StormShield, 0.0, 0.0);
                cast(g, Sk::IceArmor, 0.0, 0.0);
            }
            840 => g.blast(260.0, 670.0, 20.0),
            856 => cast(g, Sk::RotBurst, 260.0, 670.0),
            875 => {
                g.player.x = 200.0;
                g.player.y = 665.0;
                cast(g, Sk::BladeFury, 0.0, 0.0);
            }
            20 => {
                let (x, y) = at(g, 3);
                cast(g, Sk::Meteor, x, y);
            }
            30 => cast(g, Sk::IceSpikes, 260.0, 640.0),
            110 => {
                let (x, y) = at(g, 0);
                cast(g, Sk::AcidRain, x + 10.0, y + 10.0)
            }
            200 => cast(g, Sk::FireWall, 300.0, 700.0),
            260 => cast(g, Sk::Blizzard, 220.0, 668.0),
            330 => cast(g, Sk::BallLightning, 400.0, 650.0),
            420 => cast(g, Sk::Thunderstorm, 0.0, 0.0),
            520 => cast(g, Sk::AcidFlood, 250.0, 690.0),
            600 => {
                g.player.x = 210.0;
                g.player.y = 655.0;
                cast(g, Sk::Whirlwind, 0.0, 0.0);
            }
            699 => {
                g.tree = true;
                g.tree_tab = 1;
                g.tree_hover = Some(Sk::Meteor as usize);
            }
            _ => {}
        }
        if f > 700 {
            g.frame += 0;
        }
        let shots = [(716, "s16_splinter"), (745, "s17_combustion"), (815, "s18_poison"), (826, "s18b_shields"), (860, "s18c_rotburst"), (905, "s19_fury"), (66, "s01_meteor_spikes"), (80, "s02_meteor_impact"), (190, "s03_acid_rain"), (240, "s04_firewall"), (320, "s05_blizzard"), (360, "s06_ball"), (470, "s07_storm"), (570, "s08_flood"), (612, "s09_whirl"), (699, "s10_tree")];
        for (fr, name) in shots {
            if f == fr {
                return Some(name.to_string());
            }
        }
        None
    }
    fn step_showcase(&mut self) -> Option<String> {
        if self.show.as_ref().map(|s| s.loot).unwrap_or(false) {
            return self.step_loot();
        }
        if self.show.as_ref().map(|s| s.tour).unwrap_or(false) {
            return self.step_tour();
        }
        if self.show.as_ref().map(|s| s.spells).unwrap_or(false) {
            return self.step_spells();
        }
        if self.show.as_ref().map(|s| s.lab).unwrap_or(false) {
            return self.step_lab();
        }
        let g = &mut self.game;
        let f = g.frame;
        let inp = &mut self.input;
        inp.lmb = false;
        inp.rmb = false;
        inp.sel = None;
        g.player.hp = g.player.hp.max(60.0);
        g.player.mana = 100.0;
        if f == 1 {
            g.player.sk.level = 8;
            for s in [skills::Sk::Lightning, skills::Sk::Fireball, skills::Sk::Acid, skills::Sk::Nova] {
                g.player.sk.rank[s as usize] = 1;
            }
            place_near_enemy(g, 70.0);
            let (x, y) = (g.player.x + 36.0, g.player.y);
            g.world.splash(x, y, world::WATER, 9.0, 0);
        }
        if f == 600 {
            g.travel(areagen::AreaId::Crypt(1));
            place_near_enemy(g, 60.0);
        }
        let near = g
            .enemies
            .iter()
            .filter(|e| e.alive)
            .min_by(|a, b| {
                let da = (a.x - g.player.x).hypot(a.y - g.player.y);
                let db = (b.x - g.player.x).hypot(b.y - g.player.y);
                da.partial_cmp(&db).unwrap()
            })
            .map(|e| (e.x, e.y, e.vx));
        if let Some((ex, ey, _)) = near {
            let aim_h = if (300..440).contains(&f) { 8.0 } else { 30.0 };
            inp.mouse = (ex - g.cam.0.floor(), ey - aim_h - g.cam.1.floor());
            let d = (ex - g.player.x).hypot(ey - g.player.y);
            // keep a little distance for spells, close in for the sword
            let close = (300..440).contains(&f) || (560..600).contains(&f) || (820..900).contains(&f);
            let want = if close { 18.0 } else { 70.0 };
            inp.left = false;
            inp.right = false;
            inp.up = false;
            inp.down = false;
            if d > want + 6.0 {
                if ex > g.player.x + 2.0 {
                    inp.right = true;
                } else if ex < g.player.x - 2.0 {
                    inp.left = true;
                }
                if ey > g.player.y + 2.0 {
                    inp.down = true;
                } else if ey < g.player.y - 2.0 {
                    inp.up = true;
                }
            }
            let cast = |inp: &mut game::Input, s: usize| {
                inp.sel = Some(s);
                inp.rmb = true;
            };
            match f {
                90 | 100 => cast(inp, 0),
                150 | 175 | 200 | 225 => cast(inp, 2),
                470 | 500 => cast(inp, 1),
                540 => cast(inp, 3),
                660 | 690 => cast(inp, 0),
                740 | 760 => cast(inp, 1),
                800 => cast(inp, 3),
                _ => {}
            }
            if (close && f % 18 == 0) || (f > 300 && f < 440 && f % 18 == 0) {
                inp.lmb = true;
            }
        }
        let shots = [(40, "01_forest"), (94, "02_lightning"), (290, "03_acid"), (380, "04_sword"), (440, "05_sword_more"), (480, "06_fireball"), (548, "07_nova"), (590, "08_shatter"), (640, "09_crypt"), (664, "10_crypt_lightning"), (770, "11_crypt_fire"), (880, "12_crypt_after")];
        for (fr, name) in shots {
            if f == fr {
                return Some(name.to_string());
            }
        }
        None
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gfx.is_some() {
            return;
        }
        let attrs = Window::default_attributes().with_title("Rendgrave").with_inner_size(winit::dpi::PhysicalSize::new(1600u32, 900u32));
        let w = Arc::new(el.create_window(attrs).expect("window"));
        w.set_cursor_visible(false);
        let g = pollster::block_on(gfx::Gfx::new(w));
        self.frame = Some(gfx::Frame::new(g.rw as usize, g.rh as usize));
        self.gfx = Some(g);
        self.last = Instant::now();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => {
                self.game.save();
                el.exit();
            }
            WindowEvent::Resized(s) => {
                if let Some(g) = &mut self.gfx {
                    g.resize(s.width, s.height);
                    self.frame = Some(gfx::Frame::new(g.rw as usize, g.rh as usize));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(k) = event.physical_key {
                    let i = &mut self.input;
                    match k {
                        KeyCode::KeyW | KeyCode::ArrowUp => i.up = down,
                        KeyCode::KeyS | KeyCode::ArrowDown => i.down = down,
                        KeyCode::KeyA | KeyCode::ArrowLeft => i.left = down,
                        KeyCode::KeyD | KeyCode::ArrowRight => i.right = down,
                        KeyCode::Space => i.dodge = down,
                        KeyCode::KeyF => i.potion = down,
                        KeyCode::KeyR => i.restart = down,
                        KeyCode::KeyE => i.interact = down,
                        KeyCode::KeyI => i.inv = down,
                        KeyCode::KeyM if down && !event.repeat => {
                            if let Some(a) = &mut self.audio {
                                a.toggle_music();
                                self.game.toast = Some((if a.music_on { "MUSIC ON" } else { "MUSIC OFF" }.into(), 90));
                            }
                        }
                        KeyCode::KeyN if down && !event.repeat => {
                            if let Some(a) = &mut self.audio {
                                a.toggle_sfx();
                                self.game.toast = Some((if a.sfx_on { "SOUND ON" } else { "SOUND OFF" }.into(), 90));
                            }
                        }
                        KeyCode::Minus | KeyCode::NumpadSubtract if down => {
                            if let Some(a) = &mut self.audio {
                                a.change_volume(-0.1);
                                self.game.toast = Some((format!("VOLUME {}%", (a.volume * 100.0).round() as i32), 90));
                            }
                        }
                        KeyCode::Equal | KeyCode::NumpadAdd if down => {
                            if let Some(a) = &mut self.audio {
                                a.change_volume(0.1);
                                self.game.toast = Some((format!("VOLUME {}%", (a.volume * 100.0).round() as i32), 90));
                            }
                        }
                        KeyCode::Digit1 if down => self.sel_pending = Some(0),
                        KeyCode::Digit2 if down => self.sel_pending = Some(1),
                        KeyCode::Digit3 if down => self.sel_pending = Some(2),
                        KeyCode::Digit4 if down => self.sel_pending = Some(3),
                        KeyCode::Digit5 if down => self.sel_pending = Some(4),
                        KeyCode::Digit6 if down => self.sel_pending = Some(5),
                        KeyCode::Digit7 if down => self.sel_pending = Some(6),
                        KeyCode::Digit8 if down => self.sel_pending = Some(7),
                        KeyCode::Digit9 if down => self.sel_pending = Some(8),
                        KeyCode::KeyK | KeyCode::Tab => i.tree = down,
                        KeyCode::Escape => i.close = down,
                        KeyCode::F11 if down => {
                            if let Some(g) = &self.gfx {
                                let full = g.window.fullscreen().is_some();
                                g.window.set_fullscreen(if full { None } else { Some(winit::window::Fullscreen::Borderless(None)) });
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let down = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.input.lmb = down,
                    MouseButton::Right => self.input.rmb = down,
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                let n = self.game.player.sk.bar().len().max(1) as i32;
                let cur = self.game.player.sel as i32;
                let next = if dy < 0.0 { cur + 1 } else if dy > 0.0 { cur - 1 } else { cur };
                self.sel_pending = Some(next.rem_euclid(n) as usize);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_px = (position.x, position.y);
            }
            WindowEvent::RedrawRequested => self.redraw(el),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(g) = &self.gfx {
            g.window.request_redraw();
        }
    }
}

impl App {
    fn redraw(&mut self, el: &ActiveEventLoop) {
        let Some(g) = &mut self.gfx else { return };
        let (rw, rh, sc) = (g.rw, g.rh, g.scale);
        let size = g.window.inner_size();
        let offx = (size.width.saturating_sub(rw * sc)) / 2;
        let offy = (size.height.saturating_sub(rh * sc)) / 2;
        self.game.view = (rw as f32, rh as f32);
        let mut shot = None;
        if self.show.is_some() {
            shot = self.step_showcase();
            self.game.update(&self.input.clone());
        } else {
            self.input.mouse = (((self.mouse_px.0 - offx as f64) / sc as f64) as f32, ((self.mouse_px.1 - offy as f64) / sc as f64) as f32);
            let now = Instant::now();
            self.acc += now.duration_since(self.last).as_secs_f64();
            self.last = now;
            let mut n = 0;
            while self.acc >= 1.0 / 60.0 && n < 4 {
                let mut inp = self.input.clone();
                inp.sel = self.sel_pending.take();
                self.game.update(&inp);
                self.acc -= 1.0 / 60.0;
                n += 1;
            }
            if n == 4 {
                self.acc = 0.0;
            }
        }
        if let Some(a) = &mut self.audio {
            use areagen::AreaId;
            let g = &self.game;
            let want = if g.player.dead {
                audio::Track::Title
            } else if g.enemies.iter().any(|e| e.alive && e.k.boss() && e.aggro) {
                audio::Track::Boss
            } else {
                match g.area {
                    AreaId::Ashford => audio::Track::Ashford,
                    AreaId::Pines | AreaId::Graveyard | AreaId::Fen | AreaId::Ravenhold => audio::Track::Pines,
                    AreaId::Crypt(_) | AreaId::Cellars(_) => audio::Track::Crypt,
                }
            };
            let duck = if g.paused || g.tree || g.wp_open || g.inv_open || g.shop_open { 0.45 } else { 1.0 };
            a.set_music(Some(want), duck);
            let snds = std::mem::take(&mut self.game.sounds);
            a.update(&snds, (self.game.player.x, self.game.player.y));
        } else {
            self.game.sounds.clear();
        }
        let Some(frame) = &mut self.frame else { return };
        if frame.w as u32 != rw || frame.h as u32 != rh {
            *frame = gfx::Frame::new(rw as usize, rh as usize);
        }
        render::draw(&self.game, frame, &self.input, &mut self.lights);
        let indoor = self.game.player.indoor;
        let gl = gfx::Globals {
            screen: [0.0, 0.0, 0.0, (self.game.frame % 1000) as f32],
            low: [0.0; 4],
            amb_out: [0.15, 0.19, 0.30, self.game.flash],
            amb_in: [0.035, 0.035, 0.05, self.game.player.hurt.min(1.0)],
            misc: [0.0, 0.0, if indoor { 1.1 } else { 0.85 }, 1.15],
        };
        let path = shot.as_ref().map(|s| format!("{}/{}.png", self.show.as_ref().unwrap().dir, s));
        let t0 = Instant::now();
        let g = self.gfx.as_mut().unwrap();
        g.render(frame, gl, &self.lights, path.as_deref());
        if let (Some(sh), Some(name)) = (&mut self.show, &shot) {
            let alive = self.game.enemies.iter().filter(|e| e.alive).count();
            sh.log.push(format!("{} frame={} enemies_alive={} gibs={} ptcs={} lights={} render_ms={:.1}", name, self.game.frame, alive, self.game.gibs.len(), self.game.ptcs.len(), self.lights.len(), t0.elapsed().as_secs_f64() * 1000.0));
        }
        self.fps_n += 1;
        if self.fps_t.elapsed().as_secs_f64() > 2.0 {
            let fps = self.fps_n as f64 / self.fps_t.elapsed().as_secs_f64();
            if let Some(g) = &self.gfx {
                g.window.set_title(&format!("Rendgrave  -  {:.0} fps", fps));
            }
            if let Some(sh) = &mut self.show {
                sh.log.push(format!("fps {:.1} at frame {}", fps, self.game.frame));
            }
            self.fps_t = Instant::now();
            self.fps_n = 0;
        }
        if let Some(sh) = &self.show {
            let done = if sh.loot {
                shot.as_deref() == Some("g7_inventory")
            } else if sh.tour {
                shot.as_deref() == Some("t15_shop")
            } else if sh.spells {
                shot.as_deref() == Some("s19_fury")
            } else {
                self.game.frame > if sh.lab { 330 } else { 900 }
            };
            if done {
                std::fs::write(format!("{}/log.txt", sh.dir), sh.log.join("\n")).ok();
                el.exit();
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--lowtest") {
        low_test();
        return;
    }
    if args.iter().any(|a| a == "--navtest") {
        nav_test();
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--maps") {
        mapdump::dump(&args.get(i + 1).cloned().unwrap_or_else(|| "maps".into()));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--sfx-dump") {
        // write every synthesised sound effect to WAV files for listening
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "sfx".into());
        std::fs::create_dir_all(&dir).ok();
        let t0 = Instant::now();
        let bank = synth::build_bank();
        println!("bank built in {:.0} ms", t0.elapsed().as_secs_f64() * 1000.0);
        let mut keys: Vec<_> = bank.keys().copied().collect();
        keys.sort_by_key(|k| format!("{:?}", k));
        for k in keys {
            for (n, v) in bank[&k].iter().enumerate() {
                let path = format!("{}/{:?}_{}.wav", dir, k, n);
                let mut out = Vec::with_capacity(44 + v.len() * 2);
                let data = (v.len() * 2) as u32;
                out.extend_from_slice(b"RIFF");
                out.extend_from_slice(&(36 + data).to_le_bytes());
                out.extend_from_slice(b"WAVEfmt ");
                out.extend_from_slice(&16u32.to_le_bytes());
                out.extend_from_slice(&1u16.to_le_bytes());
                out.extend_from_slice(&1u16.to_le_bytes());
                out.extend_from_slice(&audio::SR.to_le_bytes());
                out.extend_from_slice(&(audio::SR * 2).to_le_bytes());
                out.extend_from_slice(&2u16.to_le_bytes());
                out.extend_from_slice(&16u16.to_le_bytes());
                out.extend_from_slice(b"data");
                out.extend_from_slice(&data.to_le_bytes());
                for s in v {
                    out.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
                }
                std::fs::write(&path, out).ok();
                let peak = v.iter().fold(0.0f32, |a, b| a.max(b.abs()));
                println!("{:?}_{}  {:.2}s  peak {:.2}", k, n, v.len() as f32 / audio::SR as f32, peak);
            }
        }
        return;
    }
    let lab = args.iter().any(|a| a == "--lab");
    let spells = args.iter().any(|a| a == "--spells");
    let tour = args.iter().any(|a| a == "--tour");
    let loot = args.iter().any(|a| a == "--loot");
    let fresh = args.iter().any(|a| a == "--new");
    let show = args.iter().position(|a| a == "--showcase" || a == "--lab" || a == "--spells" || a == "--tour" || a == "--loot").map(|i| {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "shots".into());
        std::fs::create_dir_all(&dir).ok();
        Showcase { dir, log: vec![], lab, spells, tour, loot, sub: 0 }
    });
    let seed = if show.is_some() { 7 } else { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1) };
    let el = EventLoop::new().expect("event loop");
    el.set_control_flow(ControlFlow::Poll);
    let testing = show.is_some();
    let mut app = App {
        gfx: None,
        game: if show.is_some() { game::Game::new(seed) } else { game::Game::start(!fresh) },
        input: game::Input::default(),
        frame: None,
        last: Instant::now(),
        acc: 0.0,
        mouse_px: (800.0, 450.0),
        show,
        lights: vec![],
        fps_t: Instant::now(),
        fps_n: 0,
        sel_pending: None,
        audio: if testing { None } else { audio::Audio::new() },
    };
    el.run_app(&mut app).expect("run");
}

/// --navtest: in every dungeon floor, wake every enemy within reach and stand still for 20
/// seconds; count how many get to you with and without pathfinding
fn nav_test() {
    use areagen::AreaId;
    for area in [AreaId::Crypt(1), AreaId::Crypt(2), AreaId::Crypt(3), AreaId::Cellars(1), AreaId::Cellars(2), AreaId::Graveyard, AreaId::Ravenhold, AreaId::Pines] {
        let mut out = vec![];
        for off in [true, false] {
            nav::OFF.store(off, std::sync::atomic::Ordering::Relaxed);
            let mut g = game::Game::new(7);
            g.saving = false;
            g.travel(area);
            let (px, py) = (g.player.x, g.player.y);
            let mut woke = 0;
            for e in g.enemies.iter_mut() {
                let d = ((e.x - px).powi(2) + (e.y - py).powi(2)).sqrt();
                if d > 150.0 && d < 560.0 && !e.k.ranged() && !e.k.boss() && !g.world.los(e.x, e.y - 1.0, px, py - 1.0) {
                    e.aggro = true;
                    woke += 1;
                } else {
                    e.alive = false;
                }
            }
            let inp = game::Input::default();
            let t0 = Instant::now();
            for _ in 0..1800 {
                g.player.hp = g.player.maxhp;
                g.player.inv = 5;
                g.update(&inp);
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / 1800.0;
            let near = g.enemies.iter().filter(|e| e.alive && ((e.x - g.player.x).powi(2) + (e.y - g.player.y).powi(2)).sqrt() < 60.0).count();
            out.push(format!("{}: {}/{} reached you ({:.2} ms/frame)", if off { "straight line" } else { "pathfinding" }, near, woke, ms));
        }
        println!("{:<10} {}", area.key(), out.join("   "));
    }
}

/// --lowtest: fire a fireball and an arrow-height shot at a crawler and at barrels lying in its path
fn low_test() {
    use body::Kind;
    let mut g = game::Game::new(7);
    g.saving = false;
    g.travel(areagen::AreaId::Crypt(1));
    let (px, py) = (g.player.x, g.player.y);
    g.enemies.clear();
    let mut c = game::Game::new_enemy(Kind::Crawler, px + 60.0, py);
    c.stun = 100000;
    let hp0 = c.hp;
    g.enemies.push(c);
    g.projs.push(game::Proj { k: 0, x: px + 10.0, y: py, z: 26.0, vx: 4.0, vy: 0.0, vz: 0.0, life: 90 });
    let inp = game::Input::default();
    for _ in 0..40 {
        g.update(&inp);
    }
    let e = &g.enemies[0];
    println!("fireball at a crawler: hp {:.0} -> {:.0}, alive {}", hp0, e.hp.max(0.0), e.alive);
    // an oil barrel and a keg on a firing line
    for (k, name) in [(1u8, "oil barrel"), (0u8, "powder keg")] {
        let (bx, by) = (px + 50.0, py + 30.0);
        for y in by as i32 - 6..by as i32 + 4 {
            for x in bx as i32 - 8..bx as i32 + 8 {
                g.world.set(x, y, world::E);
            }
        }
        g.world.rect(bx as i32 - 4, by as i32 - 3, bx as i32 + 5, by as i32 + 1, world::WOOD);
        g.barrels = vec![barrels::Barrel::new(bx, by, k, &g.world)];
        g.projs.push(game::Proj { k: 3, x: px, y: by, z: 26.0, vx: 4.0, vy: 0.0, vz: 0.0, life: 90 });
        for _ in 0..30 {
            g.update(&inp);
        }
        let b = &g.barrels[0];
        println!("arrow at a {}: alive {}, fuse {}", name, b.alive, b.fuse);
    }
}

/// test helper: stand a little to the west of the first reachable enemy
fn place_near_enemy(g: &mut game::Game, d: f32) {
    for k in 0..g.enemies.len() {
        let (ex, ey) = (g.enemies[k].x, g.enemies[k].y);
        for off in [d, d * 0.7, -d, -d * 0.7] {
            let (x, y) = (ex - off, ey);
            if !g.world.solid(x as i32, y as i32) && !g.world.solid(x as i32 + 3, y as i32) && !g.world.solid(x as i32 - 3, y as i32) && g.world.at(x as i32, y as i32) == world::E {
                g.player.x = x;
                g.player.y = y;
                g.cam = (x - g.view.0 / 2.0, y - 26.0 - g.view.1 / 2.0);
                return;
            }
        }
    }
}
