// Game state and rules: the player, enemies and their AI, the sword, the four spells,
// projectiles, gibs (severed pieces), particles, lights and pickups.
use crate::body::*;
use crate::rng::*;
use crate::world::*;
use crate::skills::*;
use crate::spells::*;
use crate::areagen::*;
use crate::audio::{Sfx, Snd};
use crate::gear::GroundItem;
use crate::items::*;

#[derive(Default, Clone)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub lmb: bool,
    pub rmb: bool,
    pub mouse: (f32, f32),
    pub sel: Option<usize>,
    pub dodge: bool,
    pub potion: bool,
    pub restart: bool,
    pub tree: bool,
    pub close: bool,
    pub interact: bool,
    pub inv: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum St {
    Idle,
    Chase,
    Wind,
    Recover,
}

pub struct Enemy {
    pub k: Kind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: f32,
    pub maxhp: f32,
    pub body: Body,
    pub st: St,
    pub t: i32,
    pub stun: i32,
    pub frozen: i32,
    pub wet: i32,
    pub phase: f32,
    pub flash: i32,
    pub aggro: bool,
    pub alive: bool,
    pub hpbar: i32,
    pub speed: f32,
    pub reach: f32,
    pub dmg: f32,
    pub windup: i32,
    pub face: f32,
    pub twitch: i32,
    pub bleed: i32,
    pub burning: i32,
    pub hit_cd: i32,
    pub chill: i32,
    pub summoned: bool,
    pub elite: bool,
    /// on its back after slipping in oil
    pub slip: i32,
    /// slowed by frost or poison
    pub slow: i32,
    pub bleed_dot: i32,
    pub bleed_amt: f32,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub mana: f32,
    pub body: Body,
    pub swing: i32,
    pub swing_a: f32,
    pub swing_side: f32,
    pub swung: bool,
    pub cd: [i32; N],
    pub sel: usize,
    pub dodge: i32,
    pub dodge_cd: i32,
    pub dv: (f32, f32),
    pub inv: i32,
    pub wet: i32,
    pub potions: i32,
    pub gold: i32,
    pub phase: f32,
    pub dead: bool,
    pub face: f32,
    pub cast_t: i32,
    pub hurt: f32,
    pub moving: bool,
    pub burn: i32,
    pub indoor: bool,
    pub kills: i32,
    pub sk: Skills,
    pub maxhp: f32,
    pub maxmana: f32,
    pub whirl: i32,
    pub swing_max: i32,
    /// sliding helplessly on spilt oil
    pub slip: i32,
    pub slide: (f32, f32),
    /// Storm Shield, Ice Armour and Last Stand timers
    pub shield: i32,
    pub ice: i32,
    pub stand_cd: i32,
    pub equip: [Option<Item>; 4],
    pub bag: Vec<Item>,
    pub gear: GearStats,
}

pub struct Proj {
    pub k: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub life: i32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PtK {
    Blood,
    Acid,
    Spark,
    Ember,
    Smoke,
    Debris,
    Ice,
    Flame,
    Gold,
    Heal,
    Steam,
    Cloud,
}
pub struct Ptc {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub life: i32,
    pub max: i32,
    pub k: PtK,
    pub c: [u8; 3],
}

pub struct Gib {
    pub part: Part,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub a: f32,
    pub av: f32,
    pub mirror: bool,
    pub rest: i32,
    pub bleed: i32,
    pub frozen: bool,
    pub fx: bool,
    pub age: i32,
}

pub struct Bolt {
    pub pts: Vec<(f32, f32)>,
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub t: i32,
}

pub struct Lt {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub i: f32,
    pub c: [f32; 3],
    pub t: i32,
    pub max: i32,
}

pub struct Float {
    pub x: f32,
    pub y: f32,
    pub t: i32,
    pub txt: String,
    pub c: [u8; 3],
    pub big: bool,
}

pub struct Pickup {
    pub k: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vz: f32,
    pub amt: i32,
    pub t: i32,
}

pub struct Slash {
    pub x: f32,
    pub y: f32,
    pub a0: f32,
    pub a1: f32,
    pub t: i32,
}

pub struct Nova {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub t: i32,
}


pub struct Game {
    pub world: World,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub projs: Vec<Proj>,
    pub ptcs: Vec<Ptc>,
    pub gibs: Vec<Gib>,
    pub bolts: Vec<Bolt>,
    pub lights: Vec<Lt>,
    pub braziers: Vec<(f32, f32)>,
    pub floats: Vec<Float>,
    pub pickups: Vec<Pickup>,
    pub slashes: Vec<Slash>,
    pub novas: Vec<Nova>,
    pub cam: (f32, f32),
    pub shake: f32,
    pub flash: f32,
    pub hitstop: i32,
    pub frame: u64,
    pub banner: Option<(String, String, i32)>,
    pub view: (f32, f32),
    pub seed: u64,
    pub prev: Input,
    pub fields: Vec<Field>,
    pub meteors: Vec<Meteor>,
    pub spikes: Vec<Spike>,
    pub spawn_t: i32,
    pub tree: bool,
    pub tree_hover: Option<usize>,
    /// which school the skill tree screen shows
    pub tree_tab: usize,
    pub paused: bool,
    pub hold_block: bool,
    pub area: AreaId,
    pub alvl: i32,
    pub exits: Vec<Exit>,
    pub waypoint: Option<(f32, f32)>,
    pub discovered: Vec<AreaId>,
    pub wp_open: bool,
    pub wp_hover: Option<usize>,
    pub king_dead: bool,
    pub chief_dead: bool,
    /// the title screen is up (the town behind it is just scenery)
    pub title: bool,
    pub has_save: bool,
    pub menu_hover: Option<usize>,
    pub confirm_new: bool,
    /// something for main.rs to do (quit, fullscreen, audio, new game)
    pub req: Option<crate::menu::Req>,
    /// (music on, sound on, volume), kept up to date by main.rs for the menus
    pub audio_state: (bool, bool, f32),
    pub act1_done: bool,
    /// story flags, see story.rs
    pub story: u8,
    /// open dialogue: its pages and the page shown
    pub talk: Option<(Vec<&'static str>, usize)>,
    pub npcs: Vec<crate::story::Npc>,
    /// minimap fog of war: which minimap cells you've seen, per area (dungeons reset each visit)
    pub seen: std::collections::HashMap<String, Vec<bool>>,
    /// screenshots for the store page: hide hints, banners and toasts
    pub press: bool,
    /// counts down to the end-of-act card; -1 while the card is showing
    pub ending: i32,
    /// frames played
    pub played: u64,
    pub chests: Vec<crate::props::Chest>,
    pub shrines: Vec<crate::props::Shrine>,
    pub barrels: Vec<crate::barrels::Barrel>,
    /// flow field towards the player, for enemy pathfinding
    pub nav: crate::nav::Nav,
    pub nav_dirty: bool,
    /// shrine blessing: (kind, frames left)
    pub buff: Option<(u8, i32)>,
    pub fade: i32,
    pub exit_lock: i32,
    pub prompt: Option<String>,
    pub saving: bool,
    pub buildings: Vec<Building>,
    pub lamps: Vec<(f32, f32, [f32; 3])>,
    pub sounds: Vec<Snd>,
    pub toast: Option<(String, i32)>,
    pub ground: Vec<GroundItem>,
    pub inv_open: bool,
    pub shop_open: bool,
    pub shop: Vec<Item>,
    pub shop_hover: Option<(u8, usize)>,
    pub merchant: Option<(f32, f32)>,
    pub posts: Vec<(f32, f32)>,
    pub inv_hover: Option<(u8, usize)>,
}

fn enemy(k: Kind, x: f32, y: f32) -> Enemy {
    let (hp, speed, reach, dmg, windup) = match k {
        Kind::Zombie => (70.0, 0.42, 13.0, 9.0, 22),
        Kind::Skeleton => (45.0, 0.85, 17.0, 11.0, 16),
        Kind::Brute => (260.0, 0.5, 22.0, 24.0, 38),
        Kind::Crawler => (45.0, 1.0, 12.0, 7.0, 18),
        Kind::Archer => (35.0, 0.8, 150.0, 10.0, 34),
        Kind::King => (1500.0, 0.55, 30.0, 30.0, 40),
        Kind::Bloater => (150.0, 0.34, 14.0, 12.0, 30),
        Kind::Bandit => (95.0, 0.9, 16.0, 13.0, 18),
        Kind::Crossbow => (60.0, 0.8, 175.0, 14.0, 42),
        Kind::Chief => (2400.0, 0.62, 28.0, 34.0, 34),
        Kind::Player => (100.0, 1.0, 10.0, 0.0, 10),
    };
    let mut body = humanoid(k);
    body.mirror = rnd() < 0.5;
    Enemy {
        k,
        x,
        y,
        vx: 0.0,
        vy: 0.0,
        hp,
        maxhp: hp,
        body,
        st: St::Idle,
        t: ri(120) as i32,
        stun: 0,
        frozen: 0,
        wet: 0,
        phase: rnd() * 6.0,
        flash: 0,
        aggro: false,
        alive: true,
        hpbar: 0,
        speed: speed * rr(0.9, 1.15),
        reach,
        dmg,
        windup,
        face: 1.0,
        twitch: 0,
        bleed: 0,
        burning: 0,
        hit_cd: 0,
        chill: 0,
        summoned: false,
        elite: false,
        slip: 0,
        slow: 0,
        bleed_dot: 0,
        bleed_amt: 0.0,
    }
}

impl Game {
    pub fn new_enemy(k: Kind, x: f32, y: f32) -> Enemy {
        enemy(k, x, y)
    }
    /// test modes: straight into the Hollow Pines, no save file
    pub fn new(seed_: u64) -> Game {
        let mut g = Game::blank(seed_);
        g.apply_gear();
        g.travel(AreaId::Pines);
        g
    }
    pub fn blank(seed_: u64) -> Game {
        seed(seed_);
        let world = World::new();
        let mut body = humanoid(Kind::Player);
        body.compute();
        let player = Player {
            x: 100.0,
            y: 100.0,
            hp: 100.0,
            mana: 60.0,
            body,
            swing: 0,
            swing_a: 0.0,
            swing_side: 1.0,
            swung: false,
            cd: [0; N],
            sel: 0,
            dodge: 0,
            dodge_cd: 0,
            dv: (0.0, 0.0),
            inv: 0,
            wet: 0,
            potions: 3,
            gold: 0,
            phase: 0.0,
            dead: false,
            face: 1.0,
            cast_t: 0,
            hurt: 0.0,
            moving: false,
            burn: 0,
            indoor: false,
            kills: 0,
            sk: Skills::new(),
            maxhp: 100.0,
            maxmana: 60.0,
            whirl: 0,
            swing_max: 16,
            slip: 0,
            slide: (0.0, 0.0),
            shield: 0,
            ice: 0,
            stand_cd: 0,
            equip: [Some(starter_weapon()), None, Some(starter_armor()), None],
            bag: vec![],
            gear: GearStats::default(),
        };
        let enemies = vec![];
        Game {
            world,
            player,
            enemies,
            projs: vec![],
            ptcs: vec![],
            gibs: vec![],
            bolts: vec![],
            lights: vec![],
            braziers: vec![],
            floats: vec![],
            pickups: vec![],
            slashes: vec![],
            novas: vec![],
            cam: (0.0, 0.0),
            shake: 0.0,
            flash: 0.0,
            hitstop: 0,
            frame: 0,
            banner: None,
            view: (520.0, 300.0),
            seed: seed_,
            prev: Input::default(),
            fields: vec![],
            meteors: vec![],
            spikes: vec![],
            spawn_t: 1500,
            tree: false,
            tree_hover: None,
            tree_tab: 0,
            paused: false,
            hold_block: false,
            area: AreaId::Ashford,
            alvl: 1,
            exits: vec![],
            waypoint: None,
            discovered: vec![],
            wp_open: false,
            wp_hover: None,
            king_dead: false,
            chief_dead: false,
            title: false,
            has_save: false,
            menu_hover: None,
            confirm_new: false,
            req: None,
            audio_state: (true, true, 0.7),
            act1_done: false,
            story: 0,
            talk: None,
            npcs: vec![],
            seen: std::collections::HashMap::new(),
            press: false,
            ending: 0,
            played: 0,
            chests: vec![],
            shrines: vec![],
            barrels: vec![],
            nav: crate::nav::Nav::new(),
            nav_dirty: true,
            buff: None,
            fade: 0,
            exit_lock: 0,
            prompt: None,
            saving: false,
            buildings: vec![],
            lamps: vec![],
            sounds: vec![],
            toast: None,
            ground: vec![],
            inv_open: false,
            shop_open: false,
            shop: vec![],
            shop_hover: None,
            merchant: None,
            posts: vec![],
            inv_hover: None,
        }
    }

    // ------------------------------------------------------------ helpers
    pub fn ptc(&mut self, x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32, life: i32, k: PtK, c: [u8; 3]) {
        if self.ptcs.len() < 7000 {
            self.ptcs.push(Ptc { x, y, z, vx, vy, vz, life, max: life, k, c });
        }
    }
    pub fn snd(&mut self, k: Sfx, x: f32, y: f32, v: f32) {
        self.sounds.push(Snd { k, x, y, v });
    }
    pub fn light(&mut self, x: f32, y: f32, r: f32, i: f32, c: [f32; 3], t: i32) {
        self.lights.push(Lt { x, y, r, i, c, t, max: t });
    }
    /// floating text. Damage numbers landing on the same spot add up into one number,
    /// repeated words aren't stacked, and other text steps up so it doesn't overlap
    pub fn float(&mut self, x: f32, y: f32, txt: String, c: [u8; 3], big: bool) {
        let num: Option<i32> = txt.parse().ok();
        for f in self.floats.iter_mut() {
            if f.t < 38 || (f.x - x).abs() > 16.0 || (f.y - y).abs() > 18.0 {
                continue;
            }
            if let (Some(a), Ok(b)) = (num, f.txt.parse::<i32>()) {
                if f.c == c {
                    f.txt = (a + b).to_string();
                    f.t = 60;
                    f.big = f.big || big;
                    return;
                }
            } else if f.txt == txt {
                f.t = 60;
                return;
            }
        }
        let mut y = y;
        for _ in 0..4 {
            if self.floats.iter().any(|f| f.t > 30 && (f.x - x).abs() < 30.0 && (f.y - y).abs() < 7.0) {
                y -= 9.0;
            }
        }
        self.floats.push(Float { x, y, t: 60, txt, c, big });
    }
    fn walkable(&self, x: f32, y: f32, r: f32) -> bool {
        for &(dx, dy) in &[(0.0, 0.0), (r, 0.0), (-r, 0.0), (0.0, r * 0.5), (0.0, -r * 0.5)] {
            if self.world.solid((x + dx) as i32, (y + dy) as i32) {
                return false;
            }
        }
        true
    }
    pub(crate) fn mv(&self, x: &mut f32, y: &mut f32, dx: f32, dy: f32, r: f32) -> bool {
        let mut ok = true;
        if self.walkable(*x + dx, *y, r) {
            *x += dx;
        } else {
            ok = false;
        }
        if self.walkable(*x, *y + dy, r) {
            *y += dy;
        } else {
            ok = false;
        }
        ok
    }
    fn under(&self, x: f32, y: f32) -> [bool; 6] {
        // fire, acid, water, oil, blood, ice
        let mut r = [false; 6];
        for dy in -1..=1 {
            for dx in -3..=3 {
                let i = self.world.at(x as i32 + dx, y as i32 + dy);
                match i {
                    FIRE => r[0] = true,
                    ACID => r[1] = true,
                    WATER => r[2] = true,
                    OIL => r[3] = true,
                    BLOOD => r[4] = true,
                    ICE => r[5] = true,
                    _ => {}
                }
            }
        }
        r
    }
    pub fn mouse_world(&self, inp: &Input) -> (f32, f32) {
        (self.cam.0.floor() + inp.mouse.0, self.cam.1.floor() + inp.mouse.1)
    }

    pub(crate) fn spawn_gibs(&mut self, ex: f32, ey: f32, mirror: bool, ds: Vec<Detached>, push: (f32, f32), power: f32, frozen: bool) {
        let mx = if mirror { -1.0 } else { 1.0 };
        for d in ds {
            let z = (-d.xf.oy).max(1.0);
            let gx = ex + d.xf.ox * mx;
            let n = d.part.count();
            let light = (60.0 / (n as f32 + 20.0)).min(2.0);
            self.gibs.push(Gib {
                part: d.part,
                x: gx,
                y: ey + rr(0.0, 2.0),
                z,
                vx: push.0 * power * light * rr(0.6, 1.3) + rr(-0.5, 0.5),
                vy: push.1 * power * light * rr(0.6, 1.3) + rr(-0.3, 0.3),
                vz: rr(0.6, 2.2) * light.min(1.4),
                a: d.xf.a,
                av: rr(-0.25, 0.25) * light,
                mirror,
                rest: 0,
                bleed: if frozen { 0 } else { 120 },
                frozen,
                fx: true,
                age: 0,
            });
        }
    }

    pub fn blood(&mut self, x: f32, y: f32, z: f32, n: usize, dir: (f32, f32), k: Kind) {
        let c = match k {
            Kind::Zombie => [90u8, 30, 26],
            Kind::Skeleton | Kind::Archer | Kind::King => [200, 196, 180],
            Kind::Crawler => [90u8, 30, 26],
            Kind::Bloater => [110, 150, 50],
            _ => [140, 14, 18],
        };
        for _ in 0..n {
            let sp = rr(0.3, 2.2);
            let kind = if k.bony() { PtK::Debris } else { PtK::Blood };
            self.ptc(x, y, z, dir.0 * sp + rr(-0.6, 0.6), dir.1 * sp + rr(-0.5, 0.5), rr(0.2, 2.0), 300, kind, c);
        }
    }

    // ------------------------------------------------------------ update
    pub fn update(&mut self, inp: &Input) {
        if self.fade > 0 {
            self.fade -= 1;
        }
        if self.title {
            // the town drifts by behind the title
            self.frame += 1;
            self.cam.0 = 380.0 + (self.frame as f32 * 0.002).sin() * 120.0;
            self.cam.1 = 250.0 + (self.frame as f32 * 0.0013).cos() * 40.0;
            self.world.update();
            self.update_menu(inp);
            self.prev = inp.clone();
            return;
        }
        if self.ending == -1 {
            if (inp.lmb && !self.prev.lmb) || (inp.close && !self.prev.close) || (inp.interact && !self.prev.interact) {
                self.ending = 0;
                self.hold_block = true;
            }
            self.prev = inp.clone();
            return;
        }
        if self.talk.is_some() {
            self.update_talk(inp);
            self.prev = inp.clone();
            return;
        }
        if self.ending > 0 {
            self.ending -= 1;
            if self.ending == 0 {
                self.ending = -1;
            }
        }
        let menu_up = self.inv_open || self.shop_open || self.wp_open;
        if inp.tree && !self.prev.tree && !self.player.dead {
            self.tree = !self.tree;
            self.hold_block = true;
        } else if inp.close && !self.prev.close {
            if self.tree {
                self.tree = false;
            } else if !self.player.dead && !menu_up {
                self.paused = !self.paused;
                self.menu_hover = None;
            }
            self.hold_block = true;
        }
        if inp.inv && !self.prev.inv && !self.player.dead && !self.wp_open {
            self.inv_open = !self.inv_open;
            self.tree = false;
            self.hold_block = true;
            let (x, y) = (self.player.x, self.player.y);
            self.snd(Sfx::Click, x, y, 0.6);
        }
        if self.shop_open {
            if (inp.close && !self.prev.close) || (inp.interact && !self.prev.interact) || (inp.inv && !self.prev.inv) {
                self.shop_open = false;
                self.hold_block = true;
            } else {
                self.update_shop(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.inv_open {
            if inp.close && !self.prev.close {
                self.inv_open = false;
                self.hold_block = true;
            } else {
                self.update_inventory(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.wp_open {
            if inp.close && !self.prev.close {
                self.wp_open = false;
            } else {
                self.update_waypoints(inp);
            }
            self.prev = inp.clone();
            return;
        }
        if self.paused && !self.tree {
            self.update_menu(inp);
            self.prev = inp.clone();
            return;
        }
        if self.tree {
            if inp.left && !self.prev.left {
                self.tree_tab = (self.tree_tab + 5) % 6;
            }
            if inp.right && !self.prev.right {
                self.tree_tab = (self.tree_tab + 1) % 6;
            }
            if inp.lmb && !self.prev.lmb {
                if let Some(t) = crate::hud::tab_at(inp.mouse, self.view) {
                    self.tree_tab = t;
                    let (x, y) = (self.player.x, self.player.y);
                    self.snd(Sfx::Click, x, y, 0.5);
                }
            }
            self.tree_hover = crate::hud::node_at(inp.mouse, self.view, self.tree_tab);
            if inp.lmb && !self.prev.lmb {
                if let Some(i) = self.tree_hover {
                    if self.player.sk.learn(i) {
                        let (x, y) = (self.player.x, self.player.y);
                        self.snd(Sfx::Chime, x, y, 0.7);
                        self.recalc();
                        self.flash = self.flash.max(0.05);
                    }
                }
            }
            self.prev = inp.clone();
            return;
        }
        self.frame += 1;
        self.played += 1;
        if self.hitstop > 0 {
            self.hitstop -= 1;
            self.prev = inp.clone();
            return;
        }
        self.world.update();
        let evs = std::mem::take(&mut self.world.events);
        let mut booms = 0;
        for e in evs {
            match e {
                Ev::Steam(x, y) => self.ptc(x as f32, y as f32, 2.0, rr(-0.2, 0.2), rr(-0.2, 0.2), 0.6, 50, PtK::Steam, [200, 200, 210]),
                Ev::Fume(x, y) => self.ptc(x as f32, y as f32, 1.0, rr(-0.2, 0.2), rr(-0.2, 0.2), 0.4, 60, PtK::Steam, [150, 220, 90]),
                Ev::Boom(x, y) => {
                    if booms < 4 {
                        booms += 1;
                        self.explode(x as f32, y as f32, 3.0, 7.0);
                    }
                }
            }
        }
        let mut inp2 = inp.clone();
        if self.hold_block {
            if inp.lmb || inp.rmb {
                inp2.lmb = false;
                inp2.rmb = false;
            } else {
                self.hold_block = false;
            }
        }
        self.update_player(&inp2);
        self.update_enemies();
        self.update_projs();
        self.update_gibs();
        self.update_ptcs();
        self.update_misc();
        self.update_spells();
        self.update_whirl();
        self.update_area(&inp2);
        self.update_ground();
        self.update_npcs();
        self.ambient();
        if self.frame % 6 == 0 {
            self.reveal();
        }
        self.tick_buffs();
        self.update_barrels();
        self.prev = inp.clone();
    }

    fn update_player(&mut self, inp: &Input) {
        if inp.restart && !self.prev.restart && self.player.dead {
            self.revive();
            return;
        }
        let (mwx, mwy) = self.mouse_world(inp);
        let p = &mut self.player;
        for c in p.cd.iter_mut() {
            if *c > 0 {
                *c -= 1;
            }
        }
        if let Some(s) = inp.sel {
            if s < p.sk.bar().len() {
                p.sel = s;
            }
        }
        p.hurt *= 0.93;
        if p.inv > 0 {
            p.inv -= 1;
        }
        if p.dodge_cd > 0 {
            p.dodge_cd -= 1;
        }
        if p.cast_t > 0 {
            p.cast_t -= 1;
        }
        if p.dead {
            return;
        }
        p.mana = (p.mana + 0.16 * (1.0 + 0.2 * p.sk.r(Sk::Wisdom) as f32) + p.gear.regen / 60.0).min(p.maxmana);
        let indoor = self.world.aux[(p.y as usize).min(H - 1) * W + (p.x as usize).min(W - 1)] & AUX_INDOOR != 0;
        self.player.indoor = indoor;
        let p = &mut self.player;
        // aim + facing
        let ax = mwx - p.x;
        let ay = mwy - (p.y - 24.0);
        let aim = ay.atan2(ax);
        if p.swing == 0 {
            p.face = if ax < 0.0 { -1.0 } else { 1.0 };
        }
        // movement
        let mut dx = (inp.right as i32 - inp.left as i32) as f32;
        let mut dy = (inp.down as i32 - inp.up as i32) as f32;
        let l = (dx * dx + dy * dy).sqrt();
        if l > 0.0 {
            dx /= l;
            dy /= l;
        }
        let u = self.under(self.player.x, self.player.y);
        let hm = self.haste_mult();
        let p = &mut self.player;
        let mut speed = 1.25 * (1.0 + 0.07 * p.sk.r(Sk::Swiftness) as f32) * (1.0 + p.gear.speed / 100.0) * hm;
        if p.whirl > 0 {
            speed *= 0.75;
        }
        if u[2] {
            speed *= 0.7;
            p.wet = 240;
        }
        if u[3] {
            speed *= 0.78;
            if l > 0.0 && p.dodge == 0 && p.slip == 0 && p.sk.r(Sk::Titan) == 0 && rnd() < 0.005 {
                p.slip = 34;
                p.slide = (dx * 2.6, dy * 2.6);
                self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.9 });
            }
        }
        if p.swing > 0 {
            speed *= 0.55;
        }
        if inp.dodge && !self.prev.dodge && p.dodge_cd == 0 {
            let (ddx, ddy) = if l > 0.0 { (dx, dy) } else { (aim.cos(), aim.sin() * 0.6) };
            p.dodge = 16;
            p.dodge_cd = 38;
            self.sounds.push(Snd { k: Sfx::Dodge, x: p.x, y: p.y, v: 0.8 });
            p.inv = 18;
            p.dv = (ddx * 3.1, ddy * 3.1);
        }
        let (mut vx, mut vy) = (dx * speed, dy * speed);
        if p.slip > 0 {
            p.slip -= 1;
            let t = p.slip as f32 / 34.0;
            vx = p.slide.0 * t;
            vy = p.slide.1 * t;
        }
        if p.dodge > 0 {
            p.dodge -= 1;
            vx = p.dv.0 * (0.5 + p.dodge as f32 / 32.0);
            vy = p.dv.1 * (0.5 + p.dodge as f32 / 32.0);
        }
        p.moving = l > 0.0 || p.dodge > 0;
        let (mut x, mut y) = (p.x, p.y);
        self.mv(&mut x, &mut y, vx, vy, 3.0);
        let p = &mut self.player;
        p.x = x;
        p.y = y;
        if p.moving {
            let before = (p.phase / std::f32::consts::PI) as i32;
            p.phase += 0.2;
            if (p.phase / std::f32::consts::PI) as i32 != before && p.dodge == 0 {
                self.sounds.push(Snd { k: Sfx::Step, x: p.x, y: p.y, v: 0.5 });
            }
        }
        if p.wet > 0 {
            p.wet -= 1;
        }
        // ground hazards
        if u[0] && p.inv == 0 && p.wet == 0 {
            p.burn = p.burn.max(90);
        }
        if u[2] {
            p.burn = 0;
        }
        if p.burn > 0 {
            p.burn -= 1;
            p.hp -= 0.09 * (1.0 - 0.3 * p.sk.r(Sk::Fireproof) as f32).max(0.1);
            p.hurt = p.hurt.max(0.2);
            if p.burn % 4 == 0 {
                let (x, y) = (p.x + rr(-5.0, 5.0), p.y);
                self.ptc(x, y, rr(4.0, 40.0), rr(-0.2, 0.2), 0.0, rr(0.4, 0.9), 26, PtK::Flame, [255, 160, 60]);
            }
        }
        let p = &mut self.player;
        if u[1] && p.inv == 0 {
            p.hp -= 0.12;
            p.hurt = p.hurt.max(0.25);
        }
        // potion
        if inp.potion && !self.prev.potion && p.potions > 0 && p.hp < p.maxhp {
            p.potions -= 1;
            self.sounds.push(Snd { k: Sfx::Potion, x: p.x, y: p.y, v: 1.0 });
            p.hp = (p.hp + 45.0 * (1.0 + 0.25 * p.sk.r(Sk::SecondWind) as f32)).min(p.maxhp);
            let (x, y) = (p.x, p.y);
            for _ in 0..30 {
                self.ptc(x + rr(-6.0, 6.0), y, rr(0.0, 40.0), 0.0, 0.0, rr(0.3, 1.0), 50, PtK::Heal, [255, 80, 90]);
            }
            self.float(x, y - 50.0, "+45".into(), [255, 110, 120], false);
        }
        let sf = self.swing_frames();
        let wk = self.wkind();
        let p = &mut self.player;
        // weapon
        if inp.lmb && p.swing == 0 && p.dodge == 0 && p.whirl == 0 {
            p.swing = sf;
            p.swing_max = sf;
            self.sounds.push(Snd { k: Sfx::Swing, x: p.x, y: p.y, v: 0.8 });
            p.swung = false;
            p.swing_a = aim;
            p.swing_side = -p.swing_side;
            p.face = if ax < 0.0 { -1.0 } else { 1.0 };
        }
        if p.swing > 0 {
            p.swing -= 1;
            if p.swing <= p.swing_max * 2 / 3 && !p.swung {
                p.swung = true;
                let (sa, side) = (p.swing_a, p.swing_side);
                let arc = wstats(wk).arc.min(1.3);
                self.slashes.push(Slash { x: self.player.x, y: self.player.y - 22.0, a0: sa - arc * side, a1: sa + arc * side, t: 10 });
                self.sword_hit(sa, inp);
            }
        }
        // spells
        let bar = self.player.sk.bar();
        let p = &mut self.player;
        if p.sel >= bar.len() {
            p.sel = 0;
        }
        let s = bar.get(p.sel).copied().unwrap_or(Sk::Lightning);
        let nd = node(s);
        let si = s as usize;
        if inp.rmb && !bar.is_empty() && p.cd[si] == 0 && p.dodge == 0 && p.whirl == 0 {
            if p.mana >= nd.mana {
                p.mana -= nd.mana;
                p.cd[si] = nd.cd;
                p.cast_t = 14;
                p.face = if ax < 0.0 { -1.0 } else { 1.0 };
                self.cast(s, mwx, mwy, inp);
                let ov = self.rank(Sk::Overload);
                if ov > 0 && node(s).col == 0 && rnd() < 0.12 * ov as f32 {
                    self.player.mana += node(s).mana;
                    self.player.cd[s as usize] = 0;
                    self.banner_small("OVERLOAD");
                }
            } else if !self.prev.rmb {
                let (x, y) = (p.x, p.y);
                self.float(x, y - 56.0, "NO MANA".into(), [120, 160, 255], false);
            }
        }
        // animation
        let p = &mut self.player;
        let b = &mut p.body;
        b.mirror = p.face < 0.0;
        let ph = p.phase;
        let walk = if p.moving { 1.0 } else { 0.0 };
        b.parts[T_LEGF].ang = ph.sin() * 0.55 * walk;
        b.parts[T_LEGB].ang = -ph.sin() * 0.55 * walk;
        b.parts[T_ARMB].ang = ph.sin() * 0.35 * walk + 0.1;
        b.root.1 = -b.leg_len + 1.5 - (ph * 2.0).sin().abs() * 1.2 * walk;
        b.parts[T_TORSO].ang = 0.08 * walk;
        if p.swing > 0 {
            let t = 1.0 - p.swing as f32 / p.swing_max.max(1) as f32;
            let e = 1.0 - (1.0 - t).powi(3);
            // local aim: arm angle measured from straight down, in the facing frame
            let fa = if p.face < 0.0 { std::f32::consts::PI - p.swing_a } else { p.swing_a };
            let target = fa - std::f32::consts::FRAC_PI_2;
            let side = p.swing_side;
            if wk == WKind::Spear {
                // a straight thrust
                b.parts[T_ARMF].ang = target;
                b.parts[T_TORSO].ang = 0.12 * (1.0 - (e - 0.5).abs() * 2.0);
            } else {
                b.parts[T_ARMF].ang = target - 1.6 * side + 3.2 * side * e;
                b.parts[T_TORSO].ang = 0.15 * (e - 0.5) * side;
            }
        } else if p.cast_t > 0 {
            b.parts[T_ARMB].ang = -1.9;
            b.parts[T_ARMF].ang = -ph.sin() * 0.35 * walk - 0.3;
        } else {
            b.parts[T_ARMF].ang = -ph.sin() * 0.35 * walk - 0.25 + (self.frame as f32 * 0.05).sin() * 0.03;
        }
        if p.dodge > 0 {
            b.parts[T_TORSO].ang = (16 - p.dodge) as f32 / 16.0 * std::f32::consts::TAU;
        }
        if p.whirl > 0 {
            b.mirror = (p.whirl / 3) % 2 == 0;
            b.parts[T_ARMF].ang = -1.57;
            b.parts[T_ARMB].ang = 1.4;
            b.parts[T_TORSO].ang = 0.1;
        }
        b.compute();
        self.last_stand();
        let p = &mut self.player;
        if p.hp <= 0.0 && !p.dead {
            p.dead = true;
            self.sounds.push(Snd { k: Sfx::Death, x: p.x, y: p.y, v: 1.0 });
            p.hp = 0.0;
            let (x, y) = (p.x, p.y);
            self.banner = Some(("YOU DIED".into(), "PRESS R TO WAKE IN ASHFORD".into(), 100000));
            self.blood(x, y, 20.0, 60, (0.0, 0.0), Kind::Player);
            let mut b = std::mem::replace(&mut self.player.body, humanoid(Kind::Player));
            b.mirror = self.player.face < 0.0;
            b.compute();
            let ds = b.detach_everything();
            let m = b.mirror;
            self.spawn_gibs(x, y, m, ds, (0.0, 0.0), 0.6, false);
            self.player.body.parts.iter_mut().for_each(|p| p.on = false);
        }
    }

    fn sword_hit(&mut self, aim: f32, inp: &Input) {
        let (px, py) = (self.player.x, self.player.y);
        let (mwx, mwy) = self.mouse_world(inp);
        // wooden things in the arc break
        for dy in -26i32..=26 {
            for dx in -34i32..=34 {
                let (x, y) = (px as i32 + dx, py as i32 + dy);
                let d = ((dx * dx) as f32 + (dy * dy * 2) as f32).sqrt();
                if d > 30.0 {
                    continue;
                }
                let a = (dy as f32).atan2(dx as f32);
                let da = (a - aim).sin().atan2((a - aim).cos()).abs();
                if da < 1.2 && self.world.at(x, y) == WOOD && rnd() < 0.5 {
                    self.world.set(x, y, E);
                    self.ptc(x as f32, y as f32, rr(2.0, 8.0), rr(-1.0, 1.0), rr(-1.0, 1.0), rr(0.5, 1.5), 200, PtK::Debris, [120, 86, 52]);
                    self.sounds.push(Snd { k: Sfx::Wood, x: x as f32, y: y as f32, v: 0.6 });
                }
            }
        }
        self.weapon_hit(aim, inp);
    }

    #[allow(dead_code)]
    fn old_sword_hit(&mut self, aim: f32, inp: &Input) {
        let (px, py) = (self.player.x, self.player.y);
        let (mwx, mwy) = self.mouse_world(inp);
        let edge = self.rank(Sk::Edge);
        let ex_r = self.rank(Sk::Executioner);
        let crit_ch = 0.15 + 0.08 * ex_r as f32;
        let mut any = false;
        let mut crit_any = false;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive {
                continue;
            }
            let (dx, dy) = (e.x - px, e.y - py);
            let big = if e.k.big() { 8.0 } else { 0.0 };
            let d = (dx * dx + dy * dy * 1.6).sqrt();
            if d > 34.0 + big {
                continue;
            }
            let a = dy.atan2(dx);
            let da = (a - aim).sin().atan2((a - aim).cos()).abs();
            if da > 1.35 && d > 10.0 {
                continue;
            }
            // cut height: wherever the cursor is on the enemy, otherwise a random spot
            let (rel_mx, rel_my) = (mwx - e.x, mwy - e.y);
            let h = if rel_mx.abs() < 14.0 && rel_my < 2.0 && rel_my > -66.0 {
                rel_my
            } else {
                -[8.0, 14.0, 24.0, 30.0, 38.0][ri(5)] * if e.k == Kind::King { 1.5 } else if e.k == Kind::Brute { 1.15 } else { 1.0 }
            };
            let crit = rnd() < crit_ch;
            let power = 26.0 + edge as f32 * 8.0 + if crit { 20.0 + 6.0 * ex_r as f32 } else { 0.0 };
            let dmg = 14.0 * (1.0 + 0.1 * edge as f32);
            self.cut_enemy(i, h, power, dmg, px, crit);
            any = true;
            crit_any |= crit;
        }
        if any {
            self.hitstop = if crit_any { 5 } else { 3 };
            self.shake = self.shake.max(if crit_any { 4.0 } else { 2.5 });
        }
    }

    pub fn cast_lightning(&mut self, mx: f32, my: f32, _inp: &Input) {
        self.sounds.push(Snd { k: Sfx::Zap, x: self.player.x, y: self.player.y, v: 0.7 });
        let (px, py) = (self.player.x, self.player.y);
        let hand = (px + self.player.face * 9.0, py - 30.0);
        // the enemy under the cursor, or the closest one to it
        let mut best: Option<usize> = None;
        let mut bd = 1e9;
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.alive {
                continue;
            }
            let over = (mx - e.x).abs() < 12.0 && my < e.y + 3.0 && my > e.y - 64.0;
            let d = if over { 0.0 } else { ((e.x - mx).powi(2) + (e.y - my).powi(2)).sqrt() };
            let dp = ((e.x - px).powi(2) + (e.y - py).powi(2)).sqrt();
            if d < 40.0 && dp < 240.0 && d < bd && self.world.los(px, py, e.x, e.y) {
                bd = d;
                best = Some(i);
            }
        }
        self.flash = self.flash.max(0.13);
        self.shake = self.shake.max(3.0);
        let mut hit: Vec<usize> = vec![];
        let mut from = hand;
        let mut cur = best;
        let mut jumps = 0;
        while let Some(i) = cur {
            hit.push(i);
            let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
            let to = (ex, ey - 30.0);
            self.add_bolt(from, to);
            from = to;
            jumps += 1;
            if jumps > 4 + self.rank(Sk::ChainMastery) {
                break;
            }
            cur = None;
            let mut bd = 100.0;
            for (j, f) in self.enemies.iter().enumerate() {
                if !f.alive || hit.contains(&j) {
                    continue;
                }
                let d = ((f.x - ex).powi(2) + (f.y - ey).powi(2)).sqrt();
                if d < bd {
                    bd = d;
                    cur = Some(j);
                }
            }
        }
        // conduction through puddles: under the struck enemies, or where the bolt lands
        let mut ground: Vec<(f32, f32)> = hit.iter().map(|&i| (self.enemies[i].x, self.enemies[i].y)).collect();
        if hit.is_empty() {
            self.add_bolt(hand, (mx, my));
            ground.push((mx, my));
            self.world.ignite(mx as i32, my as i32);
            for _ in 0..12 {
                self.ptc(mx, my, 1.0, rr(-2.0, 2.0), rr(-1.5, 1.5), rr(0.5, 2.5), 20, PtK::Spark, [200, 220, 255]);
            }
        }
        let mut cells = vec![];
        for &(gx, gy) in &ground {
            for dy in -2..=2 {
                for dx in -3..=3 {
                    let (x, y) = (gx as i32 + dx, gy as i32 + dy);
                    if conductive(self.world.at(x, y)) {
                        cells.extend(self.world.conduct(x, y, 6000));
                    }
                }
            }
        }
        if !cells.is_empty() {
            let set: std::collections::HashSet<usize> = cells.iter().copied().collect();
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if !e.alive || hit.contains(&i) {
                    continue;
                }
                let mut inside = false;
                for dx in -3..=3 {
                    let (x, y) = (e.x as i32 + dx, e.y as i32);
                    if World::inb(x, y) && set.contains(&(y as usize * W + x as usize)) {
                        inside = true;
                    }
                }
                if inside {
                    hit.push(i);
                    let to = (e.x, e.y - 20.0);
                    self.add_bolt((e.x + rr(-6.0, 6.0), e.y), to);
                }
            }
            // and the player, if standing in it
            let (ppx, ppy) = (self.player.x as i32, self.player.y as i32);
            if World::inb(ppx, ppy) && set.contains(&(ppy as usize * W + ppx as usize)) && self.player.inv == 0 {
                self.player.hp -= 12.0;
                self.player.hurt = 1.0;
            }
            for k in 0..cells.len().min(400) {
                let i = cells[ri(cells.len())];
                if k % 4 == 0 {
                    self.ptc((i % W) as f32, (i / W) as f32, 1.0, rr(-0.5, 0.5), rr(-0.5, 0.5), rr(0.5, 1.5), 14, PtK::Spark, [170, 200, 255]);
                }
            }
        }
        for &i in &hit {
            self.zap_enemy(i, 20.0);
        }
        self.hitstop = self.hitstop.max(2);
    }

    pub fn add_bolt(&mut self, a: (f32, f32), b: (f32, f32)) {
        let mut bolt = Bolt { pts: vec![], a, b, t: 12 };
        bolt.pts = jag(a, b);
        let n = bolt.pts.len();
        let mid = bolt.pts[n / 2];
        self.bolts.push(bolt);
        self.light(mid.0, mid.1 + 20.0, 150.0, 1.7, [0.6, 0.75, 1.0], 10);
        self.light(b.0, b.1 + 30.0, 90.0, 2.0, [0.7, 0.85, 1.0], 12);
    }

    pub fn cast_fireball(&mut self, mx: f32, my: f32) {
        self.sounds.push(Snd { k: Sfx::Fireball, x: self.player.x, y: self.player.y, v: 0.8 });
        let p = &self.player;
        let (dx, dy) = (mx - p.x, my - p.y);
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        let (sx, sy) = (p.x + p.face * 8.0, p.y + 1.0);
        self.projs.push(Proj { k: 0, x: sx, y: sy, z: 26.0, vx: dx / l * 4.0, vy: dy / l * 4.0, vz: 0.0, life: 90 });
    }
    pub fn cast_acid(&mut self, mx: f32, my: f32) {
        self.sounds.push(Snd { k: Sfx::Swing, x: self.player.x, y: self.player.y, v: 0.5 });
        let n = 3 + self.rank(Sk::Corrosion);
        let p = &self.player;
        let (dx, dy) = (mx - p.x, my - p.y);
        let base = dy.atan2(dx);
        let dist = (dx * dx + dy * dy).sqrt().clamp(30.0, 200.0);
        for k in 0..n {
            let a = base + (k as f32 - (n - 1) as f32 / 2.0) * 0.08 + rr(-0.03, 0.03);
            let sp = 3.4 + rr(-0.2, 0.2);
            let t = dist / sp;
            // lob so it lands near the cursor
            let vz = (0.5 * 0.09 * t * t - 26.0) / t;
            self.projs.push(Proj { k: 1, x: p.x + p.face * 8.0, y: p.y + 1.0, z: 26.0, vx: a.cos() * sp, vy: a.sin() * sp, vz, life: 200 });
        }
    }
    pub fn cast_nova(&mut self) {
        self.sounds.push(Snd { k: Sfx::Frost, x: self.player.x, y: self.player.y, v: 1.0 });
        let (x, y) = (self.player.x, self.player.y);
        self.novas.push(Nova { x, y, r: 4.0, t: 0 });
        self.light(x, y, 150.0, 2.0, [0.5, 0.8, 1.0], 30);
        self.shake = self.shake.max(2.0);
    }

    pub fn explode(&mut self, x: f32, y: f32, z: f32, r: f32) {
        self.sounds.push(Snd { k: Sfx::Explosion, x: x, y: y, v: (r / 16.0).min(1.3) });
        self.shake = self.shake.max(6.0);
        self.hitstop = self.hitstop.max(3);
        self.flash = self.flash.max(0.12);
        self.light(x, y, 110.0 + r * 3.0, 0.8 + r * 0.03, [1.0, 0.6, 0.25], 26);
        let ri_ = r as i32;
        for dy in -ri_..=ri_ {
            for dx in -ri_..=ri_ {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d > r {
                    continue;
                }
                let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                if !World::inb(cx, cy) {
                    continue;
                }
                let i = cy as usize * W + cx as usize;
                let m = self.world.m[i];
                self.world.scorch[i] = self.world.scorch[i].saturating_add((120.0 * (1.0 - d / r)) as u8);
                match m {
                    BRICK | STONE if d < r * 0.42 && rnd() < 0.8 => {
                        self.world.set(cx, cy, E);
                        if rnd() < 0.4 {
                            self.ptc(cx as f32, cy as f32, rr(4.0, 20.0), dx as f32 * 0.15 + rr(-0.5, 0.5), dy as f32 * 0.15 + rr(-0.5, 0.5), rr(1.0, 3.0), 300, PtK::Debris, [90, 84, 92]);
                        }
                    }
                    TREE | WOOD | FLESH | CLOTH | OIL => {
                        if rnd() < 0.6 {
                            self.world.ignite(cx, cy);
                        }
                    }
                    WATER if rnd() < 0.3 => {
                        self.world.set(cx, cy, E);
                        if rnd() < 0.2 {
                            self.ptc(cx as f32, cy as f32, 2.0, 0.0, 0.0, 0.8, 50, PtK::Steam, [210, 210, 220]);
                        }
                    }
                    ICE => self.world.set(cx, cy, WATER),
                    E | ASH | BONE | RUBBLE => {
                        if rnd() < 0.45 * (1.0 - d / r) + 0.1 {
                            self.world.ignite(cx, cy);
                        }
                    }
                    _ => {}
                }
            }
        }
        for _ in 0..(r * 2.6) as i32 {
            let a = rnd() * std::f32::consts::TAU;
            let sp = rr(0.5, 3.5);
            self.ptc(x, y, z, a.cos() * sp, a.sin() * sp * 0.6, rr(0.0, 2.5), rr(15.0, 40.0) as i32, PtK::Flame, [255, 170, 60]);
        }
        for _ in 0..30 {
            let a = rnd() * std::f32::consts::TAU;
            self.ptc(x, y, z, a.cos() * rr(0.2, 1.0), a.sin() * 0.5, rr(0.3, 1.0), rr(60.0, 120.0) as i32, PtK::Smoke, [50, 44, 46]);
        }
        for _ in 0..20 {
            let a = rnd() * std::f32::consts::TAU;
            self.ptc(x, y, z, a.cos() * rr(1.0, 4.0), a.sin() * rr(1.0, 3.0), rr(1.0, 4.0), 60, PtK::Ember, [255, 190, 90]);
        }
        // hurt everything nearby
        let mut shat = vec![];
        for i in 0..self.enemies.len() {
            let e = &mut self.enemies[i];
            if !e.alive {
                continue;
            }
            let d = ((e.x - x).powi(2) + (e.y - y).powi(2) * 2.0).sqrt();
            if d > r * 2.2 {
                continue;
            }
            if e.frozen > 0 {
                shat.push(i);
                continue;
            }
            let f = 1.0 - d / (r * 2.2);
            let dmg = 36.0 * f + 6.0;
            e.hp -= dmg;
            e.aggro = true;
            e.flash = 5;
            e.hpbar = 180;
            e.stun = e.stun.max(16);
            let (rx, ry) = (x - e.x, -z);
            e.body.each_in(rx, ry, r * 1.3, |p, dd, _| {
                if dd < r * 1.3 {
                    if matches!(p.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) {
                        p.burn = p.burn.max(rr(60.0, 160.0) as u8);
                    }
                    p.chr = p.chr.saturating_add(((1.0 - dd / (r * 1.3)) * 120.0) as u8);
                }
                if dd < r * 0.55 && rnd() < 0.5 {
                    Body::hurt_px(p, 200);
                }
            });
            e.body.dirty = true;
            let l = d.max(1.0);
            e.vx += (e.x - x) / l * 2.5 * f;
            e.vy += (e.y - y) / l * 1.5 * f;
            let (ex, ey) = (e.x, e.y);
            let txt = format!("{}", dmg as i32);
            self.float(ex, ey - 60.0, txt, [255, 170, 80], false);
        }
        for i in shat {
            self.shatter(i);
        }
    }

    fn update_projs(&mut self) {
        let mut k = 0;
        while k < self.projs.len() {
            let mut dead = false;
            let (steps, kind) = (3, self.projs[k].k);
            for _ in 0..steps {
                let p = &mut self.projs[k];
                p.x += p.vx / steps as f32;
                p.y += p.vy / steps as f32;
                if p.k == 1 {
                    p.vz -= 0.09 / steps as f32;
                }
                p.z += p.vz / steps as f32;
                let (x, y, z) = (p.x, p.y, p.z);
                let m = self.world.at(x as i32, y as i32);
                let trunk = m != TREE || self.world.aux[(y.max(0.0) as usize).min(H - 1) * W + (x.max(0.0) as usize).min(W - 1)] & AUX_TRUNK != 0;
                let wall = is_wall(m) && trunk && (z as i32) < wall_h(m) + 2;
                let ground = z <= 0.0 && kind != 2;
                // a shot crossing an enemy's row hits anything of it at or below its height,
                // so crawlers, the legless and the fallen can be shot too
                let mut hit_e = None;
                let mut hit_z = z;
                'e: for (i, e) in self.enemies.iter().enumerate() {
                    if !e.alive || (e.y - y).abs() > 7.0 || (e.x - x).abs() > 16.0 {
                        continue;
                    }
                    let mut hz = z;
                    while hz >= 1.0 {
                        if e.body.near_pixel(x - e.x, -hz, 1) {
                            hit_e = Some(i);
                            hit_z = hz;
                            break 'e;
                        }
                        hz -= 3.0;
                    }
                }
                let hit_e = if kind >= 2 { None } else { hit_e };
                // barrels stand low too
                let hit_b = if kind == 2 { None } else { self.barrels.iter().position(|b| b.alive && (b.x - x).abs() < 6.0 && (b.y - y).abs() < 4.5 && z < 40.0) };
                if hit_e.is_some() || hit_b.is_some() {
                    let hz = if hit_e.is_some() { hit_z } else { 8.0 };
                    self.projs[k].z = hz;
                }
                let z = self.projs[k].z;
                if let Some(b) = hit_b {
                    if kind != 0 {
                        self.strike_barrel(b);
                    }
                }
                let wall = wall || hit_b.is_some();
                let pl = &self.player;
                let hit_p = (kind == 3 || kind == 4) && !pl.dead && (pl.y - y).abs() < 6.0 && (pl.x - x).abs() < 5.0 && z < 48.0;
                if wall || ground || hit_e.is_some() || hit_p || self.projs[k].life <= 0 {
                    match kind {
                        0 => {
                            let r = 13.0 + 2.0 * self.rank(Sk::Kindling) as f32;
                            self.explode(x, y, z.max(2.0), r);
                            if self.rank(Sk::Splinter) > 0 {
                                let n1 = self.lights.len();
                                for q in 0..3 {
                                    let a = q as f32 * 2.1 + rr(-0.4, 0.4);
                                    self.explode(x + a.cos() * 16.0, y + a.sin() * 10.0, 2.0, r * 0.55);
                                }
                                self.lights.truncate(n1);
                            }
                        }
                        1 => self.acid_hit(x, y, z, hit_e),
                        3 | 4 => {
                            self.sounds.push(Snd { k: Sfx::ArrowHit, x: x, y: y, v: 0.7 });
                            if hit_p && self.player.inv == 0 {
                                self.sounds.push(Snd { k: Sfx::Hurt, x: self.player.x, y: self.player.y, v: 1.0 });
                                let d = if kind == 4 { 15.0 } else { 10.0 } * (1.0 + 0.1 * (self.alvl - 1) as f32) * rr(0.85, 1.15);
                                let d = d * self.armor_mult();
                                self.player.hp -= d;
                                self.player.hurt = 1.0;
                                self.player.inv = 10;
                                self.shake = self.shake.max(3.0);
                                let (ppx, ppy) = (self.player.x, self.player.y);
                                self.blood(ppx, ppy, z, 8, (self.projs[k].vx.signum(), 0.0), Kind::Player);
                                self.float(ppx, ppy - 60.0, format!("-{}", d as i32), [255, 80, 80], false);
                            } else {
                                self.ptc(x, y, z, 0.0, 0.0, 0.0, 120, PtK::Debris, [120, 90, 60]);
                            }
                        }
                        _ => {
                            for _ in 0..30 {
                                self.ptc(x, y, z, rr(-2.0, 2.0), rr(-1.5, 1.5), rr(-1.0, 2.0), 20, PtK::Spark, [190, 215, 255]);
                            }
                            self.light(x, y, 120.0, 2.0, [0.6, 0.75, 1.0], 12);
                        }
                    }
                    dead = true;
                    break;
                }
            }
            if dead {
                self.projs.swap_remove(k);
                continue;
            }
            let p = &mut self.projs[k];
            p.life -= 1;
            let (x, y, z) = (p.x, p.y, p.z);
            if kind == 0 {
                for _ in 0..3 {
                    self.ptc(x + rr(-1.5, 1.5), y, z + rr(-1.5, 1.5), rr(-0.3, 0.3), rr(-0.2, 0.2), rr(0.1, 0.6), rr(10.0, 22.0) as i32, PtK::Flame, [255, 160, 50]);
                }
                if rnd() < 0.3 {
                    self.ptc(x, y, z, 0.0, 0.0, 0.4, 70, PtK::Smoke, [60, 50, 50]);
                }
            } else if kind == 1 {
                if rnd() < 0.5 {
                    self.ptc(x, y, z, rr(-0.2, 0.2), rr(-0.2, 0.2), -0.2, 20, PtK::Acid, [140, 230, 70]);
                }
            } else if kind == 2 {
                let life = self.projs[k].life;
                if rnd() < 0.6 {
                    let a = rnd() * std::f32::consts::TAU;
                    self.ptc(x + a.cos() * 4.0, y, z + a.sin() * 4.0, a.cos() * 0.6, 0.0, a.sin() * 0.6, 8, PtK::Spark, [190, 215, 255]);
                }
                self.ball_zap(x, y, z, life);
            }
            k += 1;
        }
    }

    fn acid_hit(&mut self, x: f32, y: f32, z: f32, e: Option<usize>) {
        self.sounds.push(Snd { k: Sfx::Acid, x: x, y: y, v: 0.7 });
        if let Some(i) = e {
            let en = &mut self.enemies[i];
            en.aggro = true;
            en.hp -= 6.0;
            en.hpbar = 180;
            en.flash = 3;
            let (rx, ry) = (x - en.x, -z);
            en.body.each_in(rx, ry, 6.0, |p, _, _| {
                p.acid = p.acid.max(rr(150.0, 240.0) as u8);
            });
            let (ex, ey) = (en.x, en.y);
            self.float(ex, ey - 60.0, "6".into(), [170, 240, 90], false);
            for _ in 0..8 {
                self.ptc(x, y, z, rr(-1.0, 1.0), rr(-0.6, 0.6), rr(0.0, 1.0), 200, PtK::Acid, [130, 220, 60]);
            }
        } else {
            self.world.splash(x, y, ACID, 3.5 + 1.2 * self.rank(Sk::AcidPools) as f32, 3);
            for _ in 0..10 {
                self.ptc(x, y, 1.0, rr(-1.0, 1.0), rr(-0.6, 0.6), rr(0.5, 1.5), 40, PtK::Acid, [140, 230, 70]);
            }
        }
        self.light(x, y, 50.0, 0.8, [0.5, 1.0, 0.3], 14);
    }

    pub fn shatter(&mut self, i: usize) {
        self.sounds.push(Snd { k: Sfx::Shatter, x: self.enemies[i].x, y: self.enemies[i].y, v: 1.0 });
        if self.has_legend(Legend::WintersGrudge) {
            let (x, y) = (self.enemies[i].x, self.enemies[i].y);
            self.novas.push(Nova { x, y, r: 4.0, t: 0 });
        }
        let e = &mut self.enemies[i];
        e.alive = false;
        e.hp = 0.0;
        let (ex, ey, m) = (e.x, e.y, e.body.mirror);
        let ds = e.body.detach_everything();
        let mut pieces = vec![];
        for d in &ds {
            pieces.extend(shatter(d, 4));
        }
        for p in pieces.iter_mut() {
            for px in p.part.px.iter_mut() {
                if px.m != 0 {
                    px.c = [(px.c[0] as u16 / 2 + 90) as u8, (px.c[1] as u16 / 2 + 110) as u8, (px.c[2] as u16 / 2 + 127) as u8];
                }
            }
        }
        let dir = ((ex - self.player.x).signum(), 0.0);
        self.spawn_gibs(ex, ey, m, pieces, dir, 1.8, true);
        self.shatter_shards(ex, ey);
        for _ in 0..40 {
            self.ptc(ex, ey, rr(0.0, 50.0), rr(-2.0, 2.0), rr(-1.0, 1.0), rr(0.0, 2.0), 80, PtK::Ice, [200, 230, 255]);
        }
        self.shake = self.shake.max(5.0);
        self.hitstop = 5;
        self.float(ex, ey - 60.0, "SHATTERED".into(), [180, 220, 255], true);
        self.on_kill(i);
    }

    /// a bloater bursts: acid everywhere and a cloud of swamp gas
    fn bloater_burst(&mut self, x: f32, y: f32) {
        self.world.splash(x, y, ACID, 8.0, 3);
        self.world.blob(x, y - 2.0, 9.0, GAS, (self.frame as u32) & 0xffff, true);
        self.snd(Sfx::Acid, x, y, 1.0);
        self.snd(Sfx::Sever, x, y, 0.7);
        for _ in 0..50 {
            self.ptc(x, y, rr(5.0, 30.0), rr(-2.0, 2.0), rr(-1.2, 1.2), rr(0.5, 3.0), 90, PtK::Acid, [130, 220, 60]);
        }
        let d = ((self.player.x - x).powi(2) + ((self.player.y - y) * 1.4).powi(2)).sqrt();
        if d < 34.0 && self.player.inv == 0 && !self.player.dead {
            let dmg = 16.0 * (1.0 + 0.1 * (self.alvl - 1) as f32) * self.armor_mult();
            self.player.hp -= dmg;
            self.player.hurt = 1.0;
            self.player.inv = 14;
            let (px, py) = (self.player.x, self.player.y);
            self.float(px, py - 60.0, format!("-{}", dmg as i32), [150, 255, 90], false);
            self.snd(Sfx::Hurt, px, py, 1.0);
        }
    }

    pub(crate) fn on_kill(&mut self, i: usize) {
        let e = &self.enemies[i];
        let (x, y, k) = (e.x, e.y, e.k);
        self.player.kills += 1;
        let gold = match k {
            Kind::King | Kind::Chief => 0,
            Kind::Brute => 40 + ri(30) as i32,
            Kind::Bandit | Kind::Crossbow => 10 + ri(16) as i32,
            Kind::Skeleton | Kind::Archer => 6 + ri(10) as i32,
            _ => 3 + ri(8) as i32,
        };
        let gold = gold * (1 + self.alvl / 3);
        let fortune = self.rank(Sk::Fortune) as f32;
        let gold = (gold as f32 * (1.0 + 0.25 * fortune)) as i32;
        self.pickups.push(Pickup { k: 0, x, y, z: 10.0, vz: 1.5, amt: gold, t: 0 });
        let elite = self.enemies[i].elite;
        let xp = self.xp_for(k) * if elite { 3 } else { 1 };
        self.gain_xp(xp);
        self.drop_loot(x, y, k, elite);
        if k == Kind::King {
            self.king_slain(x, y);
        }
        if k == Kind::Chief {
            self.chief_slain(x, y);
        }
        if k == Kind::Bloater {
            self.bloater_burst(x, y);
        }
        if rnd() < if k == Kind::Brute { 1.0 } else { 0.18 + 0.07 * fortune } {
            self.pickups.push(Pickup { k: 1, x: x + 6.0, y: y + 2.0, z: 10.0, vz: 2.0, amt: 1, t: 0 });
        }
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        let tome_chance = if k == Kind::Brute { 0.12 } else { 0.004 };
        if !missing.is_empty() && rnd() < tome_chance {
            let i = missing[ri(missing.len())];
            self.pickups.push(Pickup { k: 2, x: x - 6.0, y: y + 1.0, z: 12.0, vz: 2.5, amt: i as i32, t: 0 });
        }
    }

    fn update_enemies(&mut self) {
        let bm = self.burn_mult();
        let caustic = 1.0 + 0.4 * self.rank(Sk::Caustic) as f32;
        let mut retal: Vec<usize> = vec![];
        let (px, py) = (self.player.x, self.player.y);
        if self.frame % 8 == 0 || self.nav_dirty {
            self.nav.build(&self.world, px, py);
            self.nav_dirty = false;
        }
        let pdead = self.player.dead;
        let mut flames = vec![];
        let mut drips = vec![];
        for i in 0..self.enemies.len() {
            if !self.enemies[i].alive {
                continue;
            }
            let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
            // off-screen enemies sleep, unless they are hunting you (then they follow the flow field)
            let hunting = self.enemies[i].aggro && (ex - px).abs() < 580.0 && (ey - py).abs() < 580.0;
            let far = ((ex - px).abs() > 420.0 || (ey - py).abs() > 300.0) && !hunting;
            if far && !self.enemies[i].body.fx {
                continue;
            }
            let u = self.under(ex, ey);
            let e = &mut self.enemies[i];
            if e.flash > 0 {
                e.flash -= 1;
            }
            if e.hpbar > 0 {
                e.hpbar -= 1;
            }
            if e.hit_cd > 0 {
                e.hit_cd -= 1;
            }
            if e.wet > 0 {
                e.wet -= 1;
            }
            // what it's standing in
            if u[2] || u[4] {
                e.wet = 200;
            }
            if u[0] && e.frozen == 0 {
                e.body.each_in(0.0, -2.0, 7.0, |p, _, _| {
                    if matches!(p.m, PM_FLESH | PM_CLOTH | PM_HAIR | PM_WOOD) && rnd() < 0.1 {
                        p.burn = p.burn.max(100);
                    }
                });
            }
            if u[1] {
                e.body.each_in(0.0, -1.0, 6.0, |p, _, h| {
                    if h < 4.0 && rnd() < 0.08 {
                        p.acid = p.acid.max(160);
                    }
                });
                e.hp -= 0.05 * caustic;
            }
            if u[2] {
                e.body.each_in(0.0, -2.0, 8.0, |p, _, _| p.burn = 0);
            }
            // pixel effects: fire and acid eat at the body
            let mut fl = vec![];
            let mut dr = vec![];
            let burning = e.body.update_fx(&mut fl, &mut dr, false);
            e.burning = burning;
            if burning > 0 {
                e.hp -= (burning.min(60)) as f32 * 0.004 * bm;
                if e.k.shambler() && burning > 30 {
                    e.aggro = true;
                }
            }
            for f in fl {
                flames.push((ex + f.0, ey, -f.1));
            }
            for d in dr {
                drips.push((ex + d.0, ey, -d.1));
            }
            let ds = e.body.split();
            let had = !ds.is_empty();
            let big_piece = ds.iter().any(|d| d.part.count() > 25);
            let m = e.body.mirror;
            let k = e.k;
            if had {
                if big_piece {
                    self.sounds.push(Snd { k: Sfx::Sever, x: ex, y: ey, v: 1.0 });
                }
                e.bleed = 160;
                e.flash = 4;
                e.hpbar = 180;
                e.hp -= ds.iter().map(|d| d.part.count() as f32).sum::<f32>() * 0.05;
                let dir = ((ex - px).signum(), (ey - py).signum() * 0.3);
                self.spawn_gibs(ex, ey, m, ds, dir, 1.6, false);
                if big_piece && self.has_legend(Legend::Gravecleaver) {
                    self.explode(ex + rr(-6.0, 6.0), ey + 1.0, 14.0, 8.0);
                }
                let txt = "SEVERED".to_string();
                if big_piece {
                    self.float(ex, ey - 70.0, txt, [255, 120, 100], false);
                }
            }
            let e = &mut self.enemies[i];
            // how much body is left?
            let legs = e.body.whole(T_LEGF) as i32 + e.body.whole(T_LEGB) as i32;
            let head = e.body.parts[T_HEAD].on && e.body.frac(T_HEAD) > 0.3;
            let torso = e.body.frac(T_TORSO);
            let armed = e.body.whole(T_ARMF);
            if e.hp <= 0.0 || !head || torso < 0.45 {
                e.alive = false;
                let ds = e.body.detach_everything();
                let dir = ((ex - px).signum(), 0.0);
                self.spawn_gibs(ex, ey, m, ds, dir, 0.9, false);
                if !k.bony() {
                    self.blood(ex, ey, 20.0, 30, dir, k);
                } else {
                    for _ in 0..12 {
                        self.ptc(ex, ey, rr(0.0, 40.0), rr(-1.0, 1.0), rr(-0.5, 0.5), rr(0.0, 1.0), 200, PtK::Debris, [200, 194, 176]);
                    }
                }
                self.on_kill(i);
                continue;
            }
            let e = &mut self.enemies[i];
            // bleeding stumps
            if e.bleed_dot > 0 {
                e.bleed_dot -= 1;
                e.hp -= e.bleed_amt;
                e.bleed = e.bleed.max(4);
            }
            if e.bleed > 0 {
                e.bleed -= 1;
                if !k.bony() && rnd() < 0.25 {
                    let h = rr(5.0, 40.0);
                    let c = if k.shambler() { [90, 30, 26] } else { [140, 14, 18] };
                    let (x, y) = (e.x, e.y);
                    self.ptc(x + rr(-4.0, 4.0), y, h, rr(-0.4, 0.4), rr(-0.3, 0.3), rr(0.0, 0.5), 300, PtK::Blood, c);
                }
            }
            self.boss_logic(i);
            let e = &mut self.enemies[i];
            // AI
            if e.frozen > 0 {
                e.frozen -= 1;
                e.body.compute();
                continue;
            }
            if e.twitch > 0 {
                e.twitch -= 1;
            }
            let (dx, dy) = (px - e.x, py - e.y);
            let dist = (dx * dx + dy * dy * 1.6).sqrt();
            if !e.aggro && dist < if e.k.big() { 130.0 } else if e.k.ranged() { 190.0 } else { 160.0 } && self.world.los(e.x, e.y - 1.0, px, py - 1.0) {
                e.aggro = true;
                let k = match e.k {
                    Kind::Zombie | Kind::Crawler | Kind::Bloater => Sfx::Groan,
                    Kind::Brute | Kind::Chief => Sfx::Roar,
                    Kind::Bandit | Kind::Crossbow => Sfx::HitMetal,
                    Kind::King => Sfx::KingRoar,
                    _ => Sfx::Rattle,
                };
                let (ex3, ey3) = (e.x, e.y);
                self.sounds.push(Snd { k, x: ex3, y: ey3, v: 0.9 });
            }
            let e = &mut self.enemies[i];
            let mut want = (0.0f32, 0.0f32);
            if e.slow > 0 {
                e.slow -= 1;
            }
            if e.slip > 0 {
                e.slip -= 1;
            } else if u[3] && e.aggro && !e.k.big() && e.frozen == 0 && legs == 2 && rnd() < 0.012 {
                // feet go out from under it
                e.slip = 75;
                e.stun = e.stun.max(75);
                e.vx += e.face * 2.5;
                let (sx3, sy3) = (e.x, e.y);
                self.sounds.push(Snd { k: Sfx::Dodge, x: sx3, y: sy3, v: 0.7 });
            }
            let e = &mut self.enemies[i];
            let crawling = legs == 0 || e.slip > 0;
            let spd = e.speed * if e.slow > 0 { 0.55 } else { 1.0 } * if u[3] { 0.7 } else { 1.0 } * if legs == 2 { 1.0 } else if legs == 1 { 0.55 } else { 0.35 } * if e.burning > 25 && !e.k.bony() { 1.5 } else { 1.0 };
            if e.stun > 0 {
                e.stun -= 1;
            } else if e.aggro && !pdead {
                match e.st {
                    St::Idle | St::Chase => {
                        e.st = St::Chase;
                        let l = dist.max(1.0);
                        let sees = self.world.los(e.x, e.y - 1.0, px, py - 1.0);
                        // the way to the player: straight at them when close and in sight,
                        // otherwise along the flow field round whatever is in the way
                        let toward = if sees && dist < 90.0 {
                            (dx / l, dy / l)
                        } else {
                            self.nav.dir(e.x, e.y).unwrap_or((dx / l, dy / l))
                        };
                        if e.k.ranged() {
                            if dist < 70.0 {
                                want = (-dx / l * spd, -dy / l * spd);
                            } else if dist > 125.0 || !sees {
                                want = (toward.0 * spd, toward.1 * spd);
                            }
                            if dist < e.reach && sees && armed && e.hit_cd == 0 && dist > 40.0 {
                                e.st = St::Wind;
                                e.t = e.windup;
                            }
                        } else if dist > e.reach * 0.8 {
                            want = (toward.0 * spd, toward.1 * spd);
                        }
                        let e = &mut self.enemies[i];
                        if e.k.bandit() && e.k != Kind::Chief && e.hp < e.maxhp * 0.25 && dist < 140.0 {
                            let l = dist.max(1.0);
                            want = (-dx / l * spd * 1.15, -dy / l * spd * 1.15);
                        }
                        if e.k == Kind::Chief && dist > 50.0 && dist < 170.0 && e.hit_cd == 0 && rnd() < 0.012 {
                            let l = dist.max(1.0);
                            e.vx = dx / l * 7.0;
                            e.vy = dy / l * 7.0;
                            e.hit_cd = 50;
                            let (ex3, ey3) = (e.x, e.y);
                            self.sounds.push(Snd { k: Sfx::Roar, x: ex3, y: ey3, v: 1.0 });
                            self.shake = self.shake.max(3.0);
                        }
                        let e = &mut self.enemies[i];
                        if !e.k.ranged() && dist < e.reach && (armed || e.k.shambler()) && e.hit_cd == 0 {
                            e.st = St::Wind;
                            e.t = e.windup;
                            let k = match e.k {
                                Kind::Zombie | Kind::Crawler | Kind::Bloater => Some(Sfx::Groan),
                                Kind::Brute | Kind::Chief => Some(Sfx::Roar),
                                Kind::King if rnd() < 0.3 => Some(Sfx::KingRoar),
                                _ => None,
                            };
                            if let Some(k) = k {
                                let (ex3, ey3) = (e.x, e.y);
                                self.sounds.push(Snd { k, x: ex3, y: ey3, v: 0.7 });
                            }
                        }
                        e.face = if dx < 0.0 { -1.0 } else { 1.0 };
                    }
                    St::Wind => {
                        e.t -= 1;
                        if e.t <= 0 {
                            e.st = St::Recover;
                            e.t = 30;
                            e.hit_cd = 20;
                            // strike
                            let ok = dist < e.reach + 6.0 && !e.k.ranged();
                            let (dmg, k2) = (e.dmg, e.k);
                            let (ex2, ey2) = (e.x, e.y);
                            if k2.ranged() {
                                e.hit_cd = 50;
                                let (adx, ady) = (px - ex2, py - ey2);
                                let l = (adx * adx + ady * ady).sqrt().max(1.0);
                                let f = e.face;
                                let (pk, ps) = if k2 == Kind::Crossbow { (4, 5.8) } else { (3, 4.2) };
                                self.projs.push(Proj { k: pk, x: ex2 + f * 6.0, y: ey2 + 1.0, z: 30.0, vx: adx / l * ps, vy: ady / l * ps, vz: 0.0, life: 70 });
                                self.sounds.push(Snd { k: Sfx::Bow, x: ex2, y: ey2, v: 0.8 });
                            }
                            if k2.big() {
                                self.shake = self.shake.max(7.0);
                                let f = e.face;
                                let (sx, sy) = (ex2 + f * 18.0, ey2 + 2.0);
                                for _ in 0..24 {
                                    self.ptc(sx, sy, 1.0, rr(-2.0, 2.0), rr(-1.0, 1.0), rr(1.0, 3.0), 200, PtK::Debris, [90, 76, 64]);
                                }
                                for dy in -6i32..=6 {
                                    for dx in -8i32..=8 {
                                        let (cx, cy) = (sx as i32 + dx, sy as i32 + dy);
                                        if rnd() < 0.3 && World::inb(cx, cy) && !self.world.solid(cx, cy) {
                                            let i2 = cy as usize * W + cx as usize;
                                            self.world.scorch[i2] = self.world.scorch[i2].saturating_add(40);
                                        }
                                    }
                                }
                            }
                            if k2.big() {
                                self.sounds.push(Snd { k: Sfx::Crumble, x: ex2, y: ey2, v: 0.8 });
                            }
                            if ok && self.player.inv == 0 && !self.player.dead {
                                self.sounds.push(Snd { k: Sfx::Hurt, x: self.player.x, y: self.player.y, v: 1.0 });
                                self.sounds.push(Snd { k: Sfx::HitFlesh, x: self.player.x, y: self.player.y, v: 0.8 });
                                let am = self.armor_mult();
                                self.player.hp -= dmg * rr(0.85, 1.15) * am;
                                retal.push(i);
                                self.player.hurt = 1.0;
                                self.player.inv = 14;
                                self.shake = self.shake.max(4.0);
                                let (ppx, ppy) = (self.player.x, self.player.y);
                                self.blood(ppx, ppy, 30.0, 14, ((ppx - ex2).signum(), 0.0), Kind::Player);
                                self.hitstop = 2;
                                let txt = format!("-{}", dmg as i32);
                                self.float(ppx, ppy - 60.0, txt, [255, 80, 80], false);
                            }
                        }
                    }
                    St::Recover => {
                        e.t -= 1;
                        if e.t <= 0 {
                            e.st = St::Chase;
                        }
                    }
                }
            } else if !e.aggro {
                e.t -= 1;
                if e.t < 0 {
                    e.t = 90 + ri(160) as i32;
                    e.phase += 0.0;
                    e.vx = rr(-0.3, 0.3);
                    e.vy = rr(-0.2, 0.2);
                }
                if e.t > 60 {
                    want = (e.vx, e.vy);
                }
            }
            let e = &mut self.enemies[i];
            // separation from other enemies
            let (mut sx, mut sy) = (0.0, 0.0);
            let (ex, ey) = (e.x, e.y);
            for (j, f) in self.enemies.iter().enumerate() {
                if j == i || !f.alive {
                    continue;
                }
                let (ddx, ddy) = (ex - f.x, ey - f.y);
                let d2 = ddx * ddx + ddy * ddy * 2.0;
                if d2 < 120.0 && d2 > 0.01 {
                    let d = d2.sqrt();
                    sx += ddx / d * 0.25;
                    sy += ddy / d * 0.25;
                }
            }
            let e = &mut self.enemies[i];
            let knock = (e.vx, e.vy);
            let (mvx, mvy) = (want.0 + sx + if e.st == St::Idle { 0.0 } else { knock.0 }, want.1 + sy + if e.st == St::Idle { 0.0 } else { knock.1 });
            if e.st != St::Idle {
                e.vx *= 0.82;
                e.vy *= 0.82;
            }
            let r = if e.k == Kind::King { 7.0 } else if e.k.big() { 5.0 } else if e.k == Kind::Bloater { 4.0 } else { 3.0 };
            let (mut x, mut y) = (e.x, e.y);
            let moved = self.mv(&mut x, &mut y, mvx, mvy, r);
            if !moved && (want.0 != 0.0 || want.1 != 0.0) {
                // slide around obstacles
                let (a, b) = (-want.1, want.0);
                let s = if (self.frame / 90 + i as u64) % 2 == 0 { 1.0 } else { -1.0 };
                self.mv(&mut x, &mut y, a * s, b * s, r);
            }
            let e = &mut self.enemies[i];
            let sp = ((x - e.x).powi(2) + (y - e.y).powi(2)).sqrt();
            e.x = x;
            e.y = y;
            e.phase += sp * 0.22;
            // animation
            let b = &mut e.body;
            b.mirror = e.face < 0.0;
            let ph = e.phase;
            let walking = sp > 0.05;
            let w = if walking { 1.0 } else { 0.0 };
            let tw = if e.twitch > 0 { rr(-0.3, 0.3) } else { 0.0 };
            if crawling {
                b.root = (0.0, -2.0);
                b.parts[T_TORSO].ang = 1.35 + tw;
                b.parts[T_ARMF].ang = -1.5 + ph.sin() * 0.6;
                b.parts[T_ARMB].ang = -1.5 - ph.sin() * 0.6;
                b.parts[T_HEAD].ang = -1.0;
            } else {
                let limp = if legs == 1 { (ph.sin() * 3.0).max(0.0) } else { 0.0 };
                b.root = (0.0, -b.leg_len + 1.5 - (ph * 2.0).sin().abs() * 1.0 * w + limp);
                let lean = match e.k {
                    Kind::Zombie | Kind::Crawler => 0.22,
                    Kind::Brute | Kind::King | Kind::Chief => 0.12,
                    _ => 0.05,
                };
                b.parts[T_TORSO].ang = lean + tw + if legs == 1 { 0.15 } else { 0.0 };
                b.parts[T_LEGF].ang = ph.sin() * 0.5 * w;
                b.parts[T_LEGB].ang = -ph.sin() * 0.5 * w;
                b.parts[T_HEAD].ang = tw * 2.0 + if e.k.shambler() { 0.25 } else { 0.0 };
                match e.st {
                    St::Wind => {
                        let t = 1.0 - e.t as f32 / e.windup as f32;
                        b.parts[T_ARMF].ang = -0.5 - t * 2.4;
                        b.parts[T_ARMB].ang = if e.k.shambler() { -1.4 - t } else { 0.3 };
                        b.parts[T_TORSO].ang -= t * 0.2;
                    }
                    St::Recover if e.t > 22 => {
                        b.parts[T_ARMF].ang = 0.9;
                        b.parts[T_ARMB].ang = if e.k.shambler() { 0.4 } else { 0.2 };
                        b.parts[T_TORSO].ang += 0.25;
                    }
                    _ => {
                        if e.k.shambler() {
                            b.parts[T_ARMF].ang = -1.35 + ph.sin() * 0.15;
                            b.parts[T_ARMB].ang = -1.25 - ph.sin() * 0.15;
                        } else {
                            b.parts[T_ARMF].ang = -ph.sin() * 0.4 * w - 0.2;
                            b.parts[T_ARMB].ang = ph.sin() * 0.4 * w + 0.1;
                        }
                    }
                }
            }
            b.compute();
        }
        for i in retal {
            self.retaliate(i);
        }
        for (x, y, z) in flames {
            self.ptc(x, y, z, rr(-0.15, 0.15), 0.0, rr(0.3, 0.8), rr(14.0, 26.0) as i32, PtK::Flame, [255, 150, 50]);
        }
        for (x, y, z) in drips {
            self.ptc(x, y, z, 0.0, 0.0, -0.1, 200, PtK::Acid, [150, 220, 70]);
        }
    }

    fn update_gibs(&mut self) {
        let mut k = 0;
        let mut flames = vec![];
        let mut stamp = vec![];
        while k < self.gibs.len() {
            let g = &mut self.gibs[k];
            g.age += 1;
            if g.age > 900 {
                g.rest = 999;
            }
            g.vz -= 0.16;
            let (nx, ny) = (g.x + g.vx, g.y + g.vy);
            if !self.world.solid(nx as i32, g.y as i32) || g.z > 12.0 {
                g.x = nx;
            } else {
                g.vx *= -0.4;
            }
            if !self.world.solid(g.x as i32, ny as i32) || g.z > 12.0 {
                g.y = ny;
            } else {
                g.vy *= -0.4;
            }
            g.z += g.vz;
            g.a += g.av;
            if g.z <= 0.0 {
                g.z = 0.0;
                if g.vz < -1.0 {
                    g.vz = -g.vz * 0.3;
                    g.av *= 0.6;
                } else {
                    g.vz = 0.0;
                }
                g.vx *= 0.7;
                g.vy *= 0.7;
                g.av *= 0.7;
                // settle lying down
                let target = if g.a.cos() * if g.mirror { -1.0 } else { 1.0 } > 0.0 { std::f32::consts::FRAC_PI_2 } else { -std::f32::consts::FRAC_PI_2 };
                let mut d = target - g.a;
                d = d.sin().atan2(d.cos());
                g.a += d * 0.15;
                if g.vx.abs() + g.vy.abs() < 0.05 {
                    g.rest += 1;
                }
            }
            if g.bleed > 0 {
                g.bleed -= 1;
                if rnd() < 0.15 && g.part.px.iter().any(|p| p.m == PM_FLESH || p.im == PM_FLESH) {
                    let (x, y, z) = (g.x, g.y, g.z);
                    self.ptc(x + rr(-3.0, 3.0), y, z + 2.0, rr(-0.3, 0.3), rr(-0.2, 0.2), 0.2, 300, PtK::Blood, [130, 14, 18]);
                }
            }
            let g = &mut self.gibs[k];
            if g.fx {
                // keep burning / dissolving on the ground
                let mut b = Body { parts: vec![std::mem::replace(&mut g.part, Part { k: PK::Torso, w: 0, h: 0, px: vec![], pivot: (0.0, 0.0), parent: -1, attach: (0.0, 0.0), ang: 0.0, init: 0, on: true })], xf: vec![Xf::new(0.0, 0.0, g.a)], mirror: g.mirror, root: (0.0, 0.0), dirty: false, fx: true, order: vec![0], leg_len: 0.0, kind: Kind::Zombie };
                let mut fl = vec![];
                let mut dr = vec![];
                b.update_fx(&mut fl, &mut dr, false);
                g.fx = b.fx;
                g.part = b.parts.pop().unwrap();
                for f in fl {
                    flames.push((g.x + f.0, g.y, g.z - f.1));
                }
            }
            if g.rest > 120 || g.part.count() == 0 {
                let g = self.gibs.swap_remove(k);
                stamp.push(g);
                continue;
            }
            k += 1;
        }
        for (x, y, z) in flames {
            self.ptc(x, y, z.max(1.0), 0.0, 0.0, rr(0.3, 0.7), 18, PtK::Flame, [255, 150, 50]);
        }
        // settled pieces become part of the world: they can burn, be eaten by acid, pile up
        for g in stamp {
            let xf = Xf::new(0.0, 0.0, g.a);
            let mx = if g.mirror { -1.0 } else { 1.0 };
            for ly in 0..g.part.h {
                for lx in 0..g.part.w {
                    let p = g.part.px[(ly * g.part.w + lx) as usize];
                    if p.m == 0 {
                        continue;
                    }
                    let (rx, ry) = xf.fwd(g.part.pivot, lx as f32 + 0.5, ly as f32 + 0.5);
                    let (x, y) = ((g.x + rx * mx) as i32, (g.y + ry * 0.5) as i32);
                    if !World::inb(x, y) {
                        continue;
                    }
                    let i = y as usize * W + x as usize;
                    let t = self.world.m[i];
                    if t == E || is_liquid(t) || t == ASH {
                        let m = match p.m {
                            PM_BONE => BONE,
                            PM_METAL => METAL,
                            PM_CLOTH | PM_HAIR => CLOTH,
                            PM_WOOD => RUBBLE,
                            PM_GLOW => BONE,
                            _ => FLESH,
                        };
                        self.world.m[i] = m;
                        self.world.life[i] = if p.burn > 0 && m != BONE && m != METAL { 60 } else { 0 };
                        let ch = p.chr as f32 / 255.0 * 0.8;
                        let mut c = p.c;
                        for j in 0..3 {
                            c[j] = (c[j] as f32 * (1.0 - ch) + 20.0 * ch) as u8;
                        }
                        if g.frozen {
                            c = [170, 205, 235];
                        }
                        self.world.col[i] = c;
                        self.world.wake(x, y);
                    }
                }
            }
        }
    }

    fn update_ptcs(&mut self) {
        let mut k = 0;
        let mut land = vec![];
        let (px, py) = (self.player.x, self.player.y);
        while k < self.ptcs.len() {
            let p = &mut self.ptcs[k];
            p.life -= 1;
            match p.k {
                PtK::Flame | PtK::Smoke | PtK::Steam | PtK::Heal => {
                    p.vx *= 0.96;
                    p.vy *= 0.96;
                    p.vz = p.vz * 0.97 + 0.01;
                }
                PtK::Ember => {
                    p.vz -= 0.04;
                    p.vx *= 0.98;
                }
                PtK::Spark => {
                    p.vz -= 0.12;
                }
                _ => {
                    p.vz -= 0.14;
                }
            }
            p.x += p.vx;
            p.y += p.vy;
            p.z += p.vz;
            let landed = p.z <= 0.0 && matches!(p.k, PtK::Blood | PtK::Acid | PtK::Debris | PtK::Ice);
            if p.z < 0.0 {
                p.z = 0.0;
                p.vz = 0.0;
                p.vx *= 0.5;
                p.vy *= 0.5;
            }
            if landed || p.life <= 0 {
                let q = self.ptcs.swap_remove(k);
                if landed {
                    land.push(q);
                }
                continue;
            }
            k += 1;
        }
        let _ = (px, py);
        for q in land {
            let (x, y) = (q.x as i32, q.y as i32);
            if !World::inb(x, y) || self.world.solid(x, y) {
                continue;
            }
            let i = y as usize * W + x as usize;
            match q.k {
                PtK::Blood => {
                    if self.world.m[i] == E {
                        self.world.set_life(x, y, BLOOD, if rnd() < 0.3 { 2 } else { 0 });
                    } else {
                        self.world.stain[i] = self.world.stain[i].saturating_add(40);
                    }
                }
                PtK::Acid => {
                    if self.world.m[i] == E && rnd() < 0.5 {
                        self.world.set_life(x, y, ACID, 0);
                    }
                }
                PtK::Debris => {
                    if self.world.m[i] == E && rnd() < 0.5 {
                        self.world.m[i] = if q.c[0] > 150 { BONE } else { RUBBLE };
                        self.world.col[i] = q.c;
                    }
                }
                PtK::Ice => {
                    if self.world.m[i] == E && rnd() < 0.2 {
                        self.world.set(x, y, ICE);
                    }
                }
                _ => {}
            }
        }
    }

    fn update_misc(&mut self) {
        if let Some(t) = &mut self.toast {
            t.1 -= 1;
            if t.1 <= 0 {
                self.toast = None;
            }
        }
        let br = self.rank(Sk::Brittle);
        // chimney smoke
        if self.frame % 7 == 0 {
            let chims: Vec<(i32, i32, f32)> = self.buildings.iter().filter_map(|b| b.chimney().map(|(x, y)| (x, y, b.roof_z(x + 3)))).collect();
            for (x, y, rz) in chims {
                let z = 44.0 + rz + 15.0;
                self.ptc(x as f32 + 2.0, y as f32, z, rr(-0.05, 0.05) + 0.06, 0.0, rr(0.2, 0.35), rr(120.0, 200.0) as i32, PtK::Smoke, [70, 66, 70]);
            }
        }
        // lights
        self.lights.retain_mut(|l| {
            if l.t > 0 {
                l.t -= 1;
            }
            l.t != 0
        });
        // bolts flicker
        for b in self.bolts.iter_mut() {
            b.t -= 1;
            if b.t % 2 == 0 {
                b.pts = jag(b.a, b.b);
            }
        }
        self.bolts.retain(|b| b.t > 0);
        self.slashes.iter_mut().for_each(|s| s.t -= 1);
        self.slashes.retain(|s| s.t > 0);
        self.floats.iter_mut().for_each(|f| {
            f.t -= 1;
            f.y -= 0.35;
        });
        self.floats.retain(|f| f.t > 0);
        if let Some(b) = &mut self.banner {
            b.2 -= 1;
            if b.2 <= 0 {
                self.banner = None;
            }
        }
        // frost novas
        let mut nk = 0;
        while nk < self.novas.len() {
            let (x, y, r0) = (self.novas[nk].x, self.novas[nk].y, self.novas[nk].r);
            let r = r0 + 3.6;
            self.novas[nk].r = r;
            self.novas[nk].t += 1;
            for a in 0..48 {
                let ang = a as f32 / 48.0 * std::f32::consts::TAU;
                let (cx, cy) = (x + ang.cos() * r, y + ang.sin() * r * 0.75);
                if rnd() < 0.5 {
                    self.ptc(cx, cy, rr(1.0, 10.0), ang.cos() * 0.4, ang.sin() * 0.3, rr(0.0, 0.6), 30, PtK::Ice, [200, 235, 255]);
                }
            }
            let ri_ = r as i32 + 1;
            for dy in -ri_..=ri_ {
                for dx in -ri_..=ri_ {
                    let d = ((dx * dx) as f32 + (dy * dy) as f32 / 0.5625).sqrt();
                    if d > r || d < r - 5.0 {
                        continue;
                    }
                    let (cx, cy) = (x as i32 + dx, y as i32 + dy);
                    if !World::inb(cx, cy) {
                        continue;
                    }
                    let i = cy as usize * W + cx as usize;
                    match self.world.m[i] {
                        WATER | BLOOD => self.world.set(cx, cy, ICE),
                        FIRE => self.world.set(cx, cy, E),
                        TREE | WOOD | FLESH | CLOTH => self.world.life[i] = 0,
                        _ => {}
                    }
                }
            }
            for e in self.enemies.iter_mut() {
                if !e.alive || e.frozen > 0 {
                    continue;
                }
                let d = ((e.x - x).powi(2) + ((e.y - y) / 0.75).powi(2)).sqrt();
                if d < r {
                    e.frozen = 260 + 60 * br;
                    e.hp -= 8.0;
                    e.aggro = true;
                    e.hpbar = 180;
                    e.st = St::Chase;
                    e.body.tint_all(|p| p.burn = 0);
                }
            }
            if r > 84.0 + 10.0 * br as f32 {
                self.novas.swap_remove(nk);
            } else {
                nk += 1;
            }
        }
        // pickups
        let (px, py) = (self.player.x, self.player.y);
        let mut got = vec![];
        for (i, p) in self.pickups.iter_mut().enumerate() {
            p.t += 1;
            p.vz -= 0.15;
            p.z += p.vz;
            if p.z < 0.0 {
                p.z = 0.0;
                p.vz = -p.vz * 0.4;
                if p.vz < 0.4 {
                    p.vz = 0.0;
                }
            }
            let d = ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt();
            if d < 40.0 && p.t > 40 && !self.player.dead {
                p.x += (px - p.x) / d.max(1.0) * 2.2;
                p.y += (py - p.y) / d.max(1.0) * 2.2;
            }
            if d < 7.0 && p.t > 40 {
                got.push(i);
            }
        }
        for &i in got.iter().rev() {
            let p = self.pickups.swap_remove(i);
            if p.k == 0 {
                self.player.gold += p.amt;
                self.sounds.push(Snd { k: Sfx::Coin, x: px, y: py, v: 0.8 });
                self.float(px, py - 56.0, format!("+{} GOLD", p.amt), [255, 214, 90], false);
            } else if p.k == 1 {
                self.player.potions += 1;
                self.sounds.push(Snd { k: Sfx::Coin, x: px, y: py, v: 0.6 });
                self.float(px, py - 56.0, "+POTION".into(), [255, 110, 120], false);
            } else {
                let i = p.amt as usize;
                self.player.sk.tomes[i] = true;
                self.sounds.push(Snd { k: Sfx::Chime, x: px, y: py, v: 1.0 });
                self.banner = Some((format!("TOME OF {}", NODES[i].name), "LEARN IT IN THE SKILL TREE AT LEVEL 30".into(), 260));
            }
        }
        // camera: follow the player, lean towards the cursor
        let (vw, vh) = self.view;
        let tx = self.player.x - vw / 2.0 + (self.prev.mouse.0 - vw / 2.0) * 0.18;
        let ty = self.player.y - 26.0 - vh / 2.0 + (self.prev.mouse.1 - vh / 2.0) * 0.18;
        self.cam.0 += (tx - self.cam.0) * 0.1;
        self.cam.1 += (ty - self.cam.1) * 0.1;
        self.cam.0 = self.cam.0.clamp(0.0, (self.world.aw as f32 - vw).max(0.0));
        self.cam.1 = self.cam.1.clamp(-20.0, (self.world.ah as f32 - vh).max(0.0));
        self.shake *= 0.86;
        self.flash *= 0.8;
    }
}

pub fn jag(a: (f32, f32), b: (f32, f32)) -> Vec<(f32, f32)> {
    let mut pts = vec![a, b];
    let mut amp = (((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt() * 0.18).min(22.0);
    for _ in 0..5 {
        let mut np = vec![pts[0]];
        for k in 0..pts.len() - 1 {
            let (p, q) = (pts[k], pts[k + 1]);
            let (mx, my) = ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let l = (dx * dx + dy * dy).sqrt().max(0.01);
            let off = rr(-amp, amp);
            np.push((mx - dy / l * off, my + dx / l * off));
            np.push(q);
        }
        pts = np;
        amp *= 0.5;
    }
    pts
}

/// minimap size in cells
pub const MM: (usize, usize) = (84, 63);

impl Game {
    /// uncover the minimap around you
    pub fn reveal(&mut self) {
        let key = self.area.key();
        let (sx, sy) = (self.world.aw as f32 / MM.0 as f32, self.world.ah as f32 / MM.1 as f32);
        let (px, py) = (self.player.x / sx, self.player.y / sy);
        let (rx, ry) = (230.0 / sx, 170.0 / sy);
        let v = self.seen.entry(key).or_insert_with(|| vec![false; MM.0 * MM.1]);
        for y in (py - ry).floor().max(0.0) as usize..((py + ry).ceil() as usize).min(MM.1) {
            for x in (px - rx).floor().max(0.0) as usize..((px + rx).ceil() as usize).min(MM.0) {
                let (dx, dy) = ((x as f32 + 0.5 - px) / rx, (y as f32 + 0.5 - py) / ry);
                if dx * dx + dy * dy < 1.0 {
                    v[y * MM.0 + x] = true;
                }
            }
        }
    }
    pub fn seen_at(&self, x: usize, y: usize) -> bool {
        self.seen.get(&self.area.key()).map(|v| v[y * MM.0 + x]).unwrap_or(false)
    }
}
