// Gamepad support, laid out like a console action RPG.
//   left stick move                 A attack (or interact when something is in reach)
//   right stick aim (else auto-aim) X / Y / B cast spells 1, 2, 3
//   LT cast the selected spell      RB dodge, LB potion, RT throw an oil flask
//   d-pad left/right pick a spell   d-pad up skills, d-pad down / BACK gear
//   START pause
// In menus the left stick (or d-pad) moves a cursor: A clicks, X right-clicks, B goes back.
// Touch the mouse or keyboard and the game switches straight back to them.
use crate::game::{Game, Input};
use gilrs::{Axis, Button, Gilrs};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

/// whether the gamepad is the thing being played with (button names on screen follow it)
static ACTIVE: AtomicBool = AtomicBool::new(false);
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// show button names without a controller (the screenshot mode)
pub fn show_names(on: bool) {
    ACTIVE.store(on, Ordering::Relaxed);
}

/// on-screen text, with keyboard and mouse words swapped for buttons while a pad is in use
pub fn tr(s: &str) -> Cow<'_, str> {
    if !active() {
        return Cow::Borrowed(s);
    }
    const SWAPS: [(&str, &str); 27] = [
        ("CLICK OR PRESS A NUMBER - ESC CLOSES", "PICK WITH A - B CLOSES"),
        ("CLICK ONE, OR PRESS 1, 2 OR 3", "PICK ONE WITH A"),
        ("CLICK ONE, OR ESC", "PICK ONE WITH A, OR B"),
        ("CLICK AN ITEM TO EQUIP IT", "PICK AN ITEM WITH A TO EQUIP IT"),
        ("A D OR CLICK A TAB    K CLOSE", "D-PAD CHANGES TAB    B CLOSE"),
        ("RIGHT-CLICK AGAIN", "X AGAIN"),
        ("RIGHT CLICK", "X"),
        ("CLICK", "A"),
        (" OR ESC", " OR B"),
        ("I CLOSE", "B CLOSE"),
        ("E CLOSE", "B CLOSE"),
        ("E  CLOSE", "A  CLOSE"),
        ("E  NEXT", "A  NEXT"),
        ("PRESS K", "PRESS D-PAD UP"),
        ("PRESS I", "PRESS BACK"),
        ("PRESS R", "PRESS A"),
        ("PRESS F", "PRESS LB"),
        ("PRESS Q TO THROW ONE AT YOUR CURSOR", "PRESS RT TO THROW ONE"),
        ("E  WAYPOINT", "A  WAYPOINT"),
        ("E  PORTAL", "A  PORTAL"),
        ("E  TALK", "A  TALK"),
        ("E  TRADE", "A  TRADE"),
        ("E  TO ", "A  TO "),
        ("E  OPEN", "A  OPEN"),
        ("E  READ", "A  READ"),
        ("E  REST", "A  REST"),
        ("E  FREE", "A  FREE"),
    ];
    let mut out: Option<String> = None;
    for (a, b) in SWAPS {
        let cur = out.as_deref().unwrap_or(s);
        if cur.contains(a) {
            out = Some(cur.replace(a, b));
        }
    }
    // every interaction prompt starts with the key: "E  OPEN CHEST", "E  SHRINE OF ...", and so on
    if out.as_deref().unwrap_or(s).starts_with("E  ") {
        out = Some(out.as_deref().unwrap_or(s).replacen("E  ", "A  ", 1));
    }
    match out {
        Some(o) => Cow::Owned(o),
        None => Cow::Borrowed(s),
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
struct Btns {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    lb: bool,
    rb: bool,
    lt: bool,
    rt: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    back: bool,
}

pub struct Pad {
    gil: Option<Gilrs>,
    pub active: bool,
    pub cursor: (f32, f32),
    btn: Btns,
    prev: Btns,
    ls: (f32, f32),
    rs: (f32, f32),
    /// what A is doing while held: 0 nothing, 1 attacking, 2 interacting
    a_mode: u8,
    /// which way the hero is looking, for auto-aim
    facing: (f32, f32),
    was_menu: bool,
    /// buttons held when the game switched between a menu and the fight: ignored until let go,
    /// so the A that opened a conversation doesn't also turn its first page
    mask: Btns,
    /// where the mouse was when the pad took over; moving it well away hands control back
    pub anchor: Option<(f64, f64)>,
    /// the right stick, smoothed; where the reticle sits relative to the hero (world units);
    /// and how long free aim lingers after you let go of the stick
    rs_s: (f32, f32),
    aim_off: (f32, f32),
    hold: i32,
}

impl Btns {
    fn and_not(self, m: Btns) -> Btns {
        Btns {
            a: self.a && !m.a,
            b: self.b && !m.b,
            x: self.x && !m.x,
            y: self.y && !m.y,
            lb: self.lb && !m.lb,
            rb: self.rb && !m.rb,
            lt: self.lt && !m.lt,
            rt: self.rt && !m.rt,
            up: self.up && !m.up,
            down: self.down && !m.down,
            left: self.left && !m.left,
            right: self.right && !m.right,
            start: self.start && !m.start,
            back: self.back && !m.back,
        }
    }
    fn and(self, m: Btns) -> Btns {
        Btns {
            a: self.a && m.a,
            b: self.b && m.b,
            x: self.x && m.x,
            y: self.y && m.y,
            lb: self.lb && m.lb,
            rb: self.rb && m.rb,
            lt: self.lt && m.lt,
            rt: self.rt && m.rt,
            up: self.up && m.up,
            down: self.down && m.down,
            left: self.left && m.left,
            right: self.right && m.right,
            start: self.start && m.start,
            back: self.back && m.back,
        }
    }
}

fn dead(v: (f32, f32), dz: f32) -> (f32, f32) {
    let m = (v.0 * v.0 + v.1 * v.1).sqrt();
    if m < dz {
        return (0.0, 0.0);
    }
    // rescale so the edge of the dead zone reads as zero
    let k = ((m - dz) / (1.0 - dz)).min(1.0) / m;
    (v.0 * k, v.1 * k)
}
fn mag(v: (f32, f32)) -> f32 {
    (v.0 * v.0 + v.1 * v.1).sqrt()
}

impl Pad {
    pub fn new() -> Pad {
        Pad { gil: Gilrs::new().ok(), ..Pad::off() }
    }
    /// no controller at all (the test and screenshot modes)
    pub fn off() -> Pad {
        Pad {
            gil: None,
            active: false,
            cursor: (200.0, 120.0),
            btn: Btns::default(),
            prev: Btns::default(),
            ls: (0.0, 0.0),
            rs: (0.0, 0.0),
            a_mode: 0,
            facing: (1.0, 0.0),
            was_menu: false,
            mask: Btns::default(),
            anchor: None,
            rs_s: (0.0, 0.0),
            aim_off: (60.0, 0.0),
            hold: 0,
        }
    }

    /// the mouse or keyboard was used: hand control back to them
    pub fn release(&mut self) {
        if self.active {
            self.active = false;
            ACTIVE.store(false, Ordering::Relaxed);
        }
    }

    /// read the controller once a frame
    pub fn poll(&mut self) {
        let Some(gil) = &mut self.gil else { return };
        let mut touched = false;
        while let Some(ev) = gil.next_event() {
            match ev.event {
                gilrs::EventType::ButtonPressed(..) => touched = true,
                gilrs::EventType::AxisChanged(_, v, _) if v.abs() > 0.5 => touched = true,
                _ => {}
            }
        }
        self.prev = self.btn;
        let Some((_, gp)) = gil.gamepads().find(|(_, g)| g.is_connected()) else {
            self.btn = Btns::default();
            self.ls = (0.0, 0.0);
            self.rs = (0.0, 0.0);
            return;
        };
        let p = |b: Button| gp.is_pressed(b);
        self.btn = Btns {
            a: p(Button::South),
            b: p(Button::East),
            x: p(Button::West),
            y: p(Button::North),
            lb: p(Button::LeftTrigger),
            rb: p(Button::RightTrigger),
            lt: p(Button::LeftTrigger2),
            rt: p(Button::RightTrigger2),
            up: p(Button::DPadUp),
            down: p(Button::DPadDown),
            left: p(Button::DPadLeft),
            right: p(Button::DPadRight),
            start: p(Button::Start),
            back: p(Button::Select),
        };
        // sticks: gilrs has up as +y, the screen has down as +y
        self.ls = dead((gp.value(Axis::LeftStickX), -gp.value(Axis::LeftStickY)), 0.22);
        self.rs = dead((gp.value(Axis::RightStickX), -gp.value(Axis::RightStickY)), 0.25);
        if touched && !self.active {
            self.active = true;
            self.anchor = None;
            ACTIVE.store(true, Ordering::Relaxed);
        }
    }

    /// fold the controller into one step of input
    pub fn apply(&mut self, g: &Game, inp: &mut Input) {
        if !self.active {
            return;
        }
        let menu = g.menu_mode();
        if menu != self.was_menu {
            self.mask = self.btn;
        }
        self.mask = self.mask.and(self.btn);
        let (b, pb) = (self.btn.and_not(self.mask), self.prev.and_not(self.mask));
        let (vw, vh) = g.view;
        if menu {
            if !self.was_menu {
                self.cursor = (vw / 2.0, vh / 2.0);
            }
            // the cursor speeds up the further you push
            let m = mag(self.ls);
            let sp = 1.0 + 5.0 * m * m;
            self.cursor.0 = (self.cursor.0 + self.ls.0 * sp).clamp(0.0, vw - 1.0);
            self.cursor.1 = (self.cursor.1 + self.ls.1 * sp).clamp(0.0, vh - 1.0);
            inp.mouse = self.cursor;
            inp.up |= b.up;
            inp.down |= b.down;
            inp.left |= b.left;
            inp.right |= b.right;
            inp.lmb |= b.a;
            inp.rmb |= b.x;
            inp.close |= b.b || b.start;
            inp.inv |= b.back;
            if g.title || g.paused || g.talk.is_some() || g.player.dead {
                inp.interact |= b.a;
            }
            self.a_mode = if b.a { 0 } else { self.a_mode };
            self.was_menu = true;
            return;
        }
        self.was_menu = false;
        // moving
        inp.stick = self.ls;
        if mag(self.ls) > 0.3 {
            let m = mag(self.ls);
            self.facing = (self.ls.0 / m, self.ls.1 / m);
        }
        // A: attack, or use whatever is in reach - decided when it goes down
        if b.a && !pb.a {
            self.a_mode = if g.player.dead || g.prompt.is_some() { 2 } else { 1 };
        }
        if !b.a {
            self.a_mode = 0;
        }
        match self.a_mode {
            1 => inp.lmb = true,
            2 => {
                inp.interact = true;
                inp.restart = true;
            }
            _ => {}
        }
        // spells: X, Y, B cast slots 1-3; LT casts whichever is selected
        let n = g.player.sk.bar().len();
        for (k, held) in [b.x, b.y, b.b].into_iter().enumerate() {
            if held && k < n {
                inp.sel = Some(k);
                inp.rmb = true;
            }
        }
        inp.rmb |= b.lt;
        if n > 0 {
            if b.right && !pb.right {
                inp.sel = Some((g.player.sel + 1) % n);
            }
            if b.left && !pb.left {
                inp.sel = Some((g.player.sel + n - 1) % n);
            }
        }
        inp.dodge |= b.rb;
        inp.potion |= b.lb;
        inp.throw |= b.rt;
        inp.tree |= b.up;
        inp.inv |= b.down || b.back;
        inp.close |= b.start;
        // aiming: the stick is smoothed, and the reticle glides to where it should be
        let p = &g.player;
        let (cx, cy) = (g.cam.0.floor(), g.cam.1.floor());
        let (px, py) = (p.x, p.y - 24.0);
        self.rs_s.0 += (self.rs.0 - self.rs_s.0) * 0.35;
        self.rs_s.1 += (self.rs.1 - self.rs_s.1) * 0.35;
        let rm = mag(self.rs_s);
        let free = mag(self.rs) > 0.15;
        let target = if free || self.hold > 0 {
            if free {
                // a light push aims close, a full push aims far
                self.hold = 20;
            } else {
                self.hold -= 1;
            }
            if rm > 0.05 {
                let r = 18.0 + 120.0 * rm.min(1.0);
                let (ux, uy) = (self.rs_s.0 / rm, self.rs_s.1 / rm);
                if rm > 0.5 {
                    self.facing = (ux, uy);
                }
                (ux * r, uy * r)
            } else {
                self.aim_off
            }
        } else {
            // auto-aim: the nearest enemy roughly where you're facing
            let (fx, fy) = self.facing;
            let mut best: Option<(f32, f32, f32)> = None;
            for e in g.enemies.iter().filter(|e| e.alive) {
                let (dx, dy) = (e.x - p.x, e.y - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 190.0 || d < 0.01 {
                    continue;
                }
                let cos = (dx * fx + dy * fy) / d;
                // anything right next to you counts whichever way you face
                if cos < 0.55 && d > 26.0 {
                    continue;
                }
                let score = d * (2.0 - cos);
                if best.is_none_or(|b| score < b.0) {
                    let h = if e.k.big() { 40.0 } else { 22.0 };
                    best = Some((score, e.x - px, e.y - h - py));
                }
            }
            inp.auto = true;
            match best {
                Some((_, x, y)) => (x, y),
                None => (fx * 60.0, fy * 60.0),
            }
        };
        let k = if free { 0.4 } else { 0.25 };
        self.aim_off.0 += (target.0 - self.aim_off.0) * k;
        self.aim_off.1 += (target.1 - self.aim_off.1) * k;
        self.cursor = (px + self.aim_off.0 - cx, py + self.aim_off.1 - cy);
        inp.mouse = self.cursor;
    }
}

#[cfg(test)]
impl Pad {
    /// pretend a frame of controller state: buttons by name, and the two sticks
    pub fn fake(&mut self, held: &str, ls: (f32, f32), rs: (f32, f32)) {
        self.prev = self.btn;
        let h = |n: &str| held.split_whitespace().any(|w| w == n);
        self.btn = Btns {
            a: h("a"),
            b: h("b"),
            x: h("x"),
            y: h("y"),
            lb: h("lb"),
            rb: h("rb"),
            lt: h("lt"),
            rt: h("rt"),
            up: h("up"),
            down: h("down"),
            left: h("left"),
            right: h("right"),
            start: h("start"),
            back: h("back"),
        };
        self.ls = ls;
        self.rs = rs;
        self.active = true;
        ACTIVE.store(true, Ordering::Relaxed);
    }
}
