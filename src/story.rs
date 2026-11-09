// Act I's story: Elder Maren by the Ashford fire sets you on the Hollow King, then on the
// Bandit Chief. The objective tracker always says what to do next, and a guide marker points
// at it (or at the exit that leads towards it). Ashford's villagers wander and mutter.
use crate::areagen::AreaId;
use crate::audio::Sfx;
use crate::body::Kind;
use crate::game::*;
use crate::rng::*;

// story flags (bits of Game::story)
pub const MET: u8 = 1;
pub const IN_CRYPT: u8 = 2;
pub const TOLD_KING: u8 = 4;
pub const AT_RAVENHOLD: u8 = 8;
pub const TOLD_CHIEF: u8 = 16;

#[derive(Clone)]
pub struct Npc {
    pub x: f32,
    pub y: f32,
    /// 0 Elder Maren, 1.. villagers
    pub k: u8,
    pub tx: f32,
    pub ty: f32,
    pub t: i32,
    pub face: f32,
    pub phase: f32,
    pub bark: i32,
}

const INTRO: [&str; 4] = [
    "SO YOU'RE THE ONE WHO CAME DOWN THE OLD ROAD. GOOD. ASHFORD NEEDS A BLADE.",
    "THREE WINTERS AGO THE HOLLOW KING WOKE IN THE SUNKEN CRYPT. SINCE THEN THE DEAD WON'T STAY IN THE GROUND.",
    "THE CRYPT LIES UNDER THE HOLLOW PINES. FOLLOW THE ROAD EAST AND LOOK FOR THE OLD MAUSOLEUM TO THE NORTH.",
    "THE MERCHANT ON THE SQUARE WILL TRADE WITH YOU. TOUCH EVERY WAYPOINT STONE YOU FIND - THEY'LL BRING YOU HOME.",
];
const AFTER_KING: [&str; 4] = [
    "THE KING IS DUST? THEN THE PINES WILL GROW QUIET AGAIN. YOU HAVE ALL OUR THANKS.",
    "BUT THE DEAD WERE NEVER OUR ONLY TROUBLE. BANDITS HAVE TAKEN RAVENHOLD, THE OLD KEEP BEYOND THE FEN.",
    "THEY STOP EVERY CART ON THE ROAD. GO EAST THROUGH THE OLD GRAVEYARD AND THE BLACKWATER FEN.",
    "TAKE THIS FOR THE ROAD. AND MIND THE GAS IN THE FEN - ONE SPARK AND THE WHOLE BOG GOES UP.",
];
const AFTER_CHIEF: [&str; 3] = [
    "THE CHIEF IS DEAD AND THE ROAD IS OPEN. ASHFORD WILL SEE ANOTHER SPRING, THANKS TO YOU.",
    "BUT THE TRADERS FROM THE NORTH TELL OF WORSE. THE PASSES ARE FROZEN, AND SOMETHING WALKS IN THE ICE.",
    "REST NOW. WHEN THE SNOWS BREAK, THE NORTH WILL NEED YOU.",
];
const BARKS: [&str; 10] = [
    "STAY NEAR THE FIRE AFTER DARK.",
    "MY BROTHER WENT INTO THE PINES. HE CAME BACK WRONG.",
    "THE MERCHANT'S PRICES ARE A CRIME.",
    "HEARD SCREAMING FROM THE GRAVEYARD AGAIN.",
    "DON'T GO NEAR THE FEN WITH A TORCH.",
    "THEY SAY THE KING WEARS A CROWN OF RUST.",
    "BANDITS TOOK MY CART. AND MY HAT.",
    "BLESS YOU, TRAVELLER.",
    "THE WELL WATER TASTES OF IRON.",
    "AIM FOR THEIR LEGS. THEY CAN'T CHASE YOU WITHOUT LEGS.",
];

impl Game {
    pub fn has(&self, f: u8) -> bool {
        self.story & f != 0
    }

    /// old saves: work out the story so far from what's been done
    pub fn infer_story(&mut self) {
        if self.discovered.len() > 1 || self.king_dead || self.chief_dead {
            self.story |= MET;
        }
        if self.king_dead {
            self.story |= IN_CRYPT;
        }
        if self.discovered.contains(&AreaId::Ravenhold) || self.chief_dead {
            self.story |= AT_RAVENHOLD;
        }
    }

    /// (what to do, where it is: area, and a spot in that area if it's known)
    pub fn objective(&self) -> Option<(String, AreaId)> {
        if !self.has(MET) {
            return Some(("SPEAK TO ELDER MAREN BY THE FIRE".into(), AreaId::Ashford));
        }
        if !self.king_dead {
            if !self.has(IN_CRYPT) {
                return Some(("FIND THE SUNKEN CRYPT UNDER THE HOLLOW PINES".into(), AreaId::Crypt(1)));
            }
            return Some(("DESCEND THE CRYPT AND SLAY THE HOLLOW KING".into(), AreaId::Crypt(3)));
        }
        if !self.has(TOLD_KING) {
            return Some(("RETURN TO ELDER MAREN IN ASHFORD".into(), AreaId::Ashford));
        }
        if !self.chief_dead {
            if !self.has(AT_RAVENHOLD) {
                return Some(("CROSS THE GRAVEYARD AND THE FEN TO RAVENHOLD".into(), AreaId::Ravenhold));
            }
            return Some(("SLAY THE BANDIT CHIEF IN THE CELLARS".into(), AreaId::Cellars(2)));
        }
        if !self.has(TOLD_CHIEF) {
            return Some(("RETURN TO ELDER MAREN IN ASHFORD".into(), AreaId::Ashford));
        }
        None
    }

    /// where the guide marker points in this area
    pub fn guide(&self) -> Option<(f32, f32)> {
        let (_, target) = self.objective()?;
        if target == self.area {
            if target == AreaId::Ashford {
                return self.npcs.iter().find(|n| n.k == 0).map(|n| (n.x, n.y));
            }
            return self.enemies.iter().find(|e| e.alive && e.k.boss()).map(|e| (e.x, e.y));
        }
        if let Some(p) = self.portal {
            return Some(p);
        }
        let hop = next_hop(self.area, target)?;
        self.exits.iter().find(|e| e.to == hop).map(|e| (((e.x0 + e.x1) / 2) as f32, ((e.y0 + e.y1) / 2) as f32))
    }

    /// entering an area can tick off an objective
    pub fn story_arrive(&mut self, to: AreaId) {
        let before = self.objective().map(|o| o.0);
        if to == AreaId::Crypt(1) && self.has(MET) {
            self.story |= IN_CRYPT;
        }
        if to == AreaId::Ravenhold {
            self.story |= AT_RAVENHOLD;
        }
        // tell the player when that changed what they're doing
        let after = self.objective().map(|o| o.0);
        if after != before {
            if let Some(a) = after {
                self.toast = Some((format!("NEW OBJECTIVE: {}", a), 220));
            }
        }
    }

    /// E near Maren
    pub fn talk_to_maren(&mut self) {
        let pages: Vec<&'static str> = if !self.has(MET) {
            self.story |= MET;
            INTRO.to_vec()
        } else if self.king_dead && !self.has(TOLD_KING) {
            self.story |= TOLD_KING;
            self.player.gold += 150;
            self.player.potions = (self.player.potions + 2).min(crate::shop::MAX_POTIONS);
            AFTER_KING.to_vec()
        } else if self.chief_dead && self.has(TOLD_KING) && !self.has(TOLD_CHIEF) {
            self.story |= TOLD_CHIEF;
            self.player.gold += 300;
            AFTER_CHIEF.to_vec()
        } else {
            vec![match self.objective().map(|o| o.1) {
                Some(AreaId::Crypt(_)) => "THE CRYPT LIES UNDER THE PINES. FOLLOW THE ROAD EAST, THEN NORTH TO THE MAUSOLEUM.",
                Some(AreaId::Ravenhold) | Some(AreaId::Cellars(_)) => "RAVENHOLD IS PAST THE GRAVEYARD AND THE FEN. THE CHIEF HIDES IN THE CELLARS UNDER THE KEEP.",
                _ => "ASHFORD STANDS BECAUSE OF YOU. REST BY THE FIRE A WHILE.",
            }]
        };
        self.talk = Some((pages, 0));
        self.hold_block = true;
        let (x, y) = (self.player.x, self.player.y);
        self.snd(Sfx::Click, x, y, 0.6);
    }

    /// the dialogue box: E, Space or a click turns the page
    pub fn update_talk(&mut self, inp: &Input) {
        let next = (inp.interact && !self.prev.interact) || (inp.lmb && !self.prev.lmb) || (inp.dodge && !self.prev.dodge);
        let close = inp.close && !self.prev.close;
        let told_chief = self.has(TOLD_CHIEF);
        let Some((pages, page)) = &mut self.talk else { return };
        if close || (next && *page + 1 >= pages.len()) {
            let generic = pages.len() == 1;
            let was_chief = told_chief && pages.len() == AFTER_CHIEF.len() && pages[0] == AFTER_CHIEF[0];
            let was_king = pages.len() == AFTER_KING.len() && pages[0] == AFTER_KING[0];
            self.talk = None;
            self.hold_block = true;
            if was_king {
                self.toast = Some(("+150 GOLD  +2 POTIONS".into(), 160));
                let (x, y) = (self.player.x, self.player.y);
                self.snd(Sfx::Coin, x, y, 1.0);
            }
            if was_chief {
                self.toast = Some(("+300 GOLD".into(), 160));
                self.check_act_end();
            }
            // after the first talk, a gift for the road; later, the offer of a respec
            if self.pick == 0 && self.has(MET) {
                self.offer_start();
            } else if generic && self.pick != 0 {
                self.offer_respec();
            }
            self.save();
        } else if next {
            *page += 1;
            let (x, y) = (self.player.x, self.player.y);
            self.snd(Sfx::Click, x, y, 0.4);
        }
    }

    /// villagers wander about the town and mutter when you pass
    pub fn update_npcs(&mut self) {
        let (px, py) = (self.player.x, self.player.y);
        for i in 0..self.npcs.len() {
            let n = &mut self.npcs[i];
            if n.bark > 0 {
                n.bark -= 1;
            }
            let near = ((n.x - px).powi(2) + (n.y - py).powi(2)).sqrt();
            if n.k == 0 {
                n.face = if px < n.x { -1.0 } else { 1.0 };
                continue;
            }
            n.t -= 1;
            if n.t <= 0 {
                n.t = 120 + ri(300) as i32;
                n.tx = rr(330.0, 690.0);
                n.ty = rr(320.0, 440.0);
            }
            let (dx, dy) = (n.tx - n.x, n.ty - n.y);
            let d = (dx * dx + dy * dy).sqrt();
            let mut moved = false;
            if d > 3.0 && n.t < 100 && near > 22.0 {
                let (vx, vy) = (dx / d * 0.35, dy / d * 0.35);
                let (mut x, mut y) = (n.x, n.y);
                moved = self.mv(&mut x, &mut y, vx, vy, 3.0);
                let n = &mut self.npcs[i];
                n.x = x;
                n.y = y;
                n.face = if vx < 0.0 { -1.0 } else { 1.0 };
                n.phase += 0.12;
                if !moved {
                    n.t = 0;
                }
            }
            let n = &mut self.npcs[i];
            if !moved && near < 40.0 {
                n.face = if px < n.x { -1.0 } else { 1.0 };
            }
            if near < 34.0 && n.bark == 0 && !self.player.dead {
                n.bark = 900;
                let (x, y) = (n.x, n.y);
                let line = BARKS[(i * 3 + (self.frame / 600) as usize) % BARKS.len()];
                self.float(x, y - 44.0, line.into(), [220, 210, 190], false);
            }
        }
    }
}

/// the next area on the way from `from` to `to` (the world is a short chain of areas)
pub fn next_hop(from: AreaId, to: AreaId) -> Option<AreaId> {
    use std::collections::VecDeque;
    let nbrs = |a: AreaId| -> Vec<AreaId> {
        match a {
            AreaId::Ashford => vec![AreaId::Pines],
            AreaId::Pines => vec![AreaId::Ashford, AreaId::Graveyard, AreaId::Crypt(1)],
            AreaId::Graveyard => vec![AreaId::Pines, AreaId::Fen],
            AreaId::Fen => vec![AreaId::Graveyard, AreaId::Ravenhold],
            AreaId::Ravenhold => vec![AreaId::Fen, AreaId::Cellars(1)],
            AreaId::Crypt(1) => vec![AreaId::Pines, AreaId::Crypt(2)],
            AreaId::Crypt(f) => vec![AreaId::Crypt(f - 1), AreaId::Crypt(f + 1)],
            AreaId::Cellars(1) => vec![AreaId::Ravenhold, AreaId::Cellars(2)],
            AreaId::Cellars(f) => vec![AreaId::Cellars(f - 1), AreaId::Cellars(f + 1)],
        }
    };
    let mut prev: Vec<(AreaId, AreaId)> = vec![];
    let mut q = VecDeque::from([from]);
    let mut seen = vec![from];
    while let Some(a) = q.pop_front() {
        if a == to {
            let mut cur = to;
            loop {
                let p = prev.iter().find(|x| x.0 == cur)?.1;
                if p == from {
                    return Some(cur);
                }
                cur = p;
            }
        }
        for b in nbrs(a) {
            if !seen.contains(&b) {
                seen.push(b);
                prev.push((b, a));
                q.push_back(b);
            }
        }
    }
    None
}

/// the villagers of Ashford (and Maren)
pub fn town_folk(maren: (f32, f32), w: &crate::world::World) -> Vec<Npc> {
    let mut v = vec![Npc { x: maren.0, y: maren.1, k: 0, tx: maren.0, ty: maren.1, t: 0, face: 1.0, phase: 0.0, bark: 0 }];
    let mut k = 1u8;
    for _ in 0..400 {
        if k > 4 {
            break;
        }
        let (x, y) = (rr(380.0, 660.0), rr(330.0, 440.0));
        let open = (-4..=4).all(|d| !w.solid(x as i32 + d, y as i32)) && w.aux[y as usize * crate::world::W + x as usize] & crate::world::AUX_INDOOR == 0;
        if open {
            v.push(Npc { x, y, k, tx: x, ty: y, t: ri(200) as i32, face: 1.0, phase: rnd() * 6.0, bark: 0 });
            k += 1;
        }
    }
    let _ = Kind::Player;
    v
}

impl Game {
    /// short tips the first time each thing comes up, then never again
    pub fn first_tips(&mut self) {
        if self.toast.is_some() || self.banner.is_some() || self.press || self.player.dead {
            return;
        }
        let p = &self.player;
        let near_frozen = self.enemies.iter().any(|e| e.alive && e.frozen > 0 && (e.x - p.x).abs() < 150.0 && (e.y - p.y).abs() < 100.0);
        let king = self.enemies.iter().any(|e| e.alive && e.k == crate::body::Kind::King && e.aggro);
        let seen = |f: &dyn Fn(&Enemy) -> bool| self.enemies.iter().any(|e| e.alive && e.aggro && f(e) && (e.x - p.x).abs() < 220.0 && (e.y - p.y).abs() < 140.0);
        let bloater = seen(&|e| e.k == crate::body::Kind::Bloater);
        let shield = seen(&|e| e.shield > 0.0);
        let tips: [(bool, &str); 7] = [
            (p.sk.points > 0 && p.sk.level >= 2, "TIP: YOU HAVE A SKILL POINT - PRESS K TO SPEND IT"),
            (p.hp < p.maxhp * 0.45 && p.potions > 0, "TIP: PRESS F TO DRINK A POTION"),
            (!p.bag.is_empty(), "TIP: PRESS I TO SEE YOUR GEAR - CLICK AN ITEM TO EQUIP IT"),
            (near_frozen, "TIP: FROZEN FOES SHATTER WHEN YOU STRIKE THEM"),
            (king, "TIP: AIM AT THE KING'S KNEES, SWORD HAND OR CHEST TO BREAK THEM"),
            (bloater, "TIP: CUT A BLOATER AND IT LEAKS ACID - LEAD IT THROUGH ITS FRIENDS"),
            (shield, "TIP: SHIELDS BLOCK FROM THE FRONT - GET BEHIND, CUT THE SHIELD ARM OR BURN WOOD"),
        ];
        for (k, (when, text)) in tips.iter().enumerate() {
            if *when && self.tips & (1 << k) == 0 {
                self.tips |= 1 << k;
                self.toast = Some((text.to_string(), 300));
                return;
            }
        }
    }
}
