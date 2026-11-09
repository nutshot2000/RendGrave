// Choices that are yours from the first minute (slice step 4):
//   a starting gift, chosen after Elder Maren's first talk - a spark of lightning, a satchel
//   of oil flasks to throw, or a better weapon
//   a respec at Maren's fire: all skill points back for 50 gold a level (tomes stay learned)
use crate::audio::Sfx;
use crate::game::*;
use crate::items::*;
use crate::rng::*;
use crate::skills::*;

pub const FLASKS: i32 = 5;

/// what the choice box is choosing
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChoiceKind {
    Start,
    Respec,
}

#[derive(Clone)]
pub struct Choice {
    pub kind: ChoiceKind,
    pub title: String,
    pub opts: Vec<(String, String)>,
    pub hover: Option<usize>,
}

impl Game {
    pub fn respec_cost(&self) -> i32 {
        50 * self.player.sk.level
    }

    /// open the starting-gift choice
    pub fn offer_start(&mut self) {
        self.choice = Some(Choice {
            kind: ChoiceKind::Start,
            title: "MAREN: TAKE ONE OF THESE FOR THE ROAD".into(),
            opts: vec![
                ("A SPARK OF LIGHTNING".into(), "LEARN LIGHTNING NOW: MAGIC FROM YOUR FIRST FIGHT. IT JUMPS THROUGH WATER AND BLOOD.".into()),
                ("OIL FLASKS".into(), "FIVE FLASKS TO THROW WITH Q. OIL TRIPS AND SLOWS - THEN SET IT ALIGHT. REFILLED IN ASHFORD.".into()),
                ("A BETTER WEAPON".into(), "A FINE MAGIC AXE OR SPEAR, IN YOUR HANDS AT ONCE.".into()),
            ],
            hover: None,
        });
        self.hold_block = true;
    }

    /// open the respec offer
    pub fn offer_respec(&mut self) {
        let spent: i32 = self.player.sk.rank.iter().map(|r| *r as i32).sum();
        if spent == 0 {
            return;
        }
        self.choice = Some(Choice {
            kind: ChoiceKind::Respec,
            title: "MAREN: WANT TO UNLEARN WHAT YOU'VE LEARNED?".into(),
            opts: vec![
                (format!("FORGET MY SKILLS - {} GOLD", self.respec_cost()), format!("ALL {} SKILL POINTS BACK TO SPEND AGAIN. TOMES YOU'VE READ STAY LEARNED.", spent)),
                ("NOT NOW".into(), "KEEP YOUR SKILLS AS THEY ARE.".into()),
            ],
            hover: None,
        });
        self.hold_block = true;
    }

    /// clicks and number keys on the choice box
    pub fn update_choice(&mut self, inp: &Input) {
        let Some(c) = &mut self.choice else { return };
        let n = c.opts.len();
        let hov = crate::hud::choice_at(inp.mouse, self.view, n);
        if hov.is_some() {
            c.hover = hov;
        }
        let kind = c.kind;
        let mut pick = if inp.lmb && !self.prev.lmb { hov } else { None };
        if let Some(s) = inp.sel {
            if s < n {
                pick = Some(s);
            }
        }
        if kind == ChoiceKind::Respec && inp.close && !self.prev.close {
            pick = Some(1);
        }
        let Some(k) = pick else { return };
        self.choice = None;
        self.hold_block = true;
        let (x, y) = (self.player.x, self.player.y);
        match kind {
            ChoiceKind::Start => self.take_gift(k as u8 + 1),
            ChoiceKind::Respec if k == 0 => {
                let cost = self.respec_cost();
                if self.player.gold < cost {
                    self.toast = Some((format!("YOU NEED {} GOLD", cost), 150));
                    return;
                }
                self.player.gold -= cost;
                let sk = &mut self.player.sk;
                let back: i32 = sk.rank.iter().map(|r| *r as i32).sum();
                sk.rank = [0; N];
                sk.points += back;
                self.player.sel = 0;
                self.apply_gear();
                self.toast = Some((format!("{} SKILL POINTS TO SPEND - PRESS K", back), 300));
                self.snd(Sfx::Chime, x, y, 1.0);
                self.save();
            }
            _ => {}
        }
    }

    /// the starting gift: 1 spark, 2 oil flasks, 3 a better weapon
    pub fn take_gift(&mut self, k: u8) {
        self.pick = k;
        let (x, y) = (self.player.x, self.player.y);
        match k {
            1 => {
                let i = NODES.iter().position(|n| n.sk == Sk::Lightning).unwrap();
                self.player.sk.rank[i] = self.player.sk.rank[i].max(1);
                self.player.sel = 0;
                self.toast = Some(("LIGHTNING LEARNED - RIGHT MOUSE TO CAST".into(), 300));
            }
            2 => {
                self.flasks = FLASKS;
                self.toast = Some(("OIL FLASKS - PRESS Q TO THROW ONE AT YOUR CURSOR".into(), 300));
            }
            _ => {
                let mut it = roll(3, 0.3);
                for _ in 0..400 {
                    if it.slot == Slot::Weapon && matches!(it.wk, WKind::Axe | WKind::Spear) && it.rarity == Rarity::Magic {
                        break;
                    }
                    it = roll(3, 0.3);
                }
                let name = it.name.clone();
                if let Some(old) = self.player.equip[0].replace(it) {
                    if self.player.bag.len() < crate::gear::BAG_SIZE {
                        self.player.bag.push(old);
                    }
                }
                self.apply_gear();
                self.toast = Some((format!("{} IS IN YOUR HANDS", name), 300));
            }
        }
        self.snd(Sfx::Chime, x, y, 1.0);
        self.save();
    }

    /// Q: lob an oil flask at the cursor
    pub fn throw_flask(&mut self, inp: &Input) {
        if self.flasks <= 0 {
            if self.pick == 2 {
                self.toast = Some(("NO FLASKS LEFT - THEY'RE REFILLED IN ASHFORD".into(), 120));
            }
            return;
        }
        self.flasks -= 1;
        let (px, py) = (self.player.x, self.player.y);
        let (tx, ty) = self.mouse_world(inp);
        let (dx, dy) = (tx - px, ty - py);
        let d = (dx * dx + dy * dy).sqrt().clamp(20.0, 200.0);
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        let sp = 3.2;
        let t = d / sp;
        let vz = (0.5 * 0.09 * t * t - 24.0) / t;
        self.projs.push(Proj { k: 6, x: px + dx / l * 6.0, y: py + dy / l * 3.0, z: 24.0, vx: dx / l * sp, vy: dy / l * sp, vz, life: 400 });
        self.snd(Sfx::Dodge, px, py, 0.6);
    }

    /// a flask lands: glass and a spreading pool of oil
    pub fn flask_break(&mut self, x: f32, y: f32) {
        self.world.splash(x, y, crate::world::OIL, 7.0, 3);
        for _ in 0..14 {
            self.ptc(x, y, rr(1.0, 6.0), rr(-1.2, 1.2), rr(-0.6, 0.6), rr(0.5, 1.8), 40, PtK::Spark, [200, 220, 230]);
        }
        self.snd(Sfx::Splash, x, y, 0.8);
    }
}
