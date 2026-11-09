// Quiet between the fights, and a town that remembers what you did (slice step 6).
//   the Hollow Pines: a burned camp off the road, a charred note, coin under a stone
//   the Blackwater Fen: an old stone overlook where the dead don't come; rest there
//   the Old Graveyard: a villager penned in for the dead to find - break him out
//   Ashford: more people about once the King is dead, Tomas once you've saved him,
//   lanterns strung over the square once the Chief is dead, and new things to say
use crate::areagen::AreaId;
use crate::audio::Sfx;
use crate::game::*;
use crate::rng::*;

/// lore spot kinds
pub const CAMP: u8 = 0;
pub const OVERLOOK: u8 = 1;
pub const PEN: u8 = 2;
/// the villager you can free, as an Npc kind
pub const TOMAS: u8 = 9;

const NOTE: [&str; 3] = [
    "A NOTE, CHARRED AT THE EDGES, PINNED UNDER A STONE BY THE DEAD FIRE.",
    "\"WE CAMPED OFF THE ROAD TO KEEP CLEAR OF THE DEAD. IT WASN'T THE DEAD THAT CAME. THEY HAD TORCHES AND CROSSBOWS, AND THEY TOOK THE CART.\"",
    "\"IF ANYONE FINDS THIS, TELL MARA IN ASHFORD I'M SORRY. THE COIN IS UNDER THE FLAT STONE.\"",
];
const OVERLOOK_TEXT: [&str; 2] = [
    "AN OLD STONE OVERLOOK ABOVE THE FEN. THE MIST MOVES SLOWLY OVER THE BLACK WATER, AND NOTHING DEAD COMES UP HERE.",
    "FAR TO THE EAST, CAMPFIRES BURN ON THE WALLS OF RAVENHOLD. YOU SIT A WHILE AND YOUR WOUNDS CLOSE.",
];
const TOMAS_TEXT: [&str; 3] = [
    "\"THANK THE GODS. THE BANDITS PENNED ME IN HERE FOR THE DEAD TO FIND - SAID IT WAS CHEAPER THAN A ROPE.\"",
    "\"I'M TOMAS, FROM ASHFORD. TAKE THIS - IT'S ALL THEY DIDN'T FIND ON ME.\"",
    "\"I'LL BE BY THE WELL IF YOU EVER NEED A FRIEND. NOW I'M RUNNING, AND I'M NOT STOPPING.\"",
];
pub const BARKS_KING: [&str; 6] = [
    "THE PINES ARE QUIET AT LAST.",
    "A TRADER CAME BACK UP THE OLD ROAD TODAY.",
    "MY BROTHER STAYED DEAD THIS TIME. THANK YOU.",
    "WE CAN BURY OUR DEAD AGAIN.",
    "THEY'RE SINGING ABOUT YOU IN THE TAVERN. BADLY.",
    "NOW IF SOMEONE WOULD DEAL WITH THOSE BANDITS...",
];
pub const BARKS_CHIEF: [&str; 5] = [
    "RAVENHOLD IS FREE! DRINKS ARE ON THE HOUSE.",
    "THE CARTS ARE RUNNING AGAIN.",
    "I GOT MY HAT BACK. AND MY CART.",
    "LOOK AT THE LANTERNS! WE HAVEN'T LIT THEM IN YEARS.",
    "ASHFORD WILL REMEMBER YOU.",
];
pub const BARKS_TOMAS: [&str; 3] = ["YOU SAVED MY LIFE. I'LL NOT FORGET IT.", "STILL HAVE NIGHTMARES ABOUT THAT PEN.", "MARA SAYS THANK YOU TOO."];

impl Game {
    /// the lore spots of this area, and Tomas in his pen if he's still there
    pub fn place_quiet(&mut self) {
        if self.area == AreaId::Graveyard && self.lore_done & (1 << PEN) == 0 {
            if let Some(&(x, y, _)) = self.lore.iter().find(|l| l.2 == PEN) {
                self.npcs.push(crate::story::Npc { x, y, k: TOMAS, tx: x, ty: y, t: 0, face: 1.0, phase: 0.0, bark: 0 });
            }
        }
        if self.area == AreaId::Ashford {
            self.ashford_life();
        }
    }

    /// E near a lore spot (returns true if one is in reach)
    pub fn update_lore(&mut self, press: bool) -> bool {
        let (px, py) = (self.player.x, self.player.y);
        let Some(&(x, y, k)) = self.lore.iter().find(|l| ((l.0 - px).powi(2) + ((l.1 - py) * 1.4).powi(2)).sqrt() < 24.0) else { return false };
        let done = self.lore_done & (1 << k) != 0;
        if k == PEN && done {
            return false;
        }
        self.prompt = Some(match k {
            CAMP => "E  READ THE NOTE".into(),
            OVERLOOK => "E  REST A WHILE".into(),
            _ => "E  FREE THE VILLAGER".into(),
        });
        if !press {
            return true;
        }
        self.lore_done |= 1 << k;
        let pages: Vec<&'static str> = match k {
            CAMP => NOTE.to_vec(),
            OVERLOOK => OVERLOOK_TEXT.to_vec(),
            _ => TOMAS_TEXT.to_vec(),
        };
        self.talk_name = Some(match k {
            CAMP => "A TRAVELLER'S NOTE".into(),
            OVERLOOK => "THE OVERLOOK".into(),
            _ => "TOMAS".into(),
        });
        self.talk = Some((pages, 0));
        self.hold_block = true;
        match k {
            CAMP if !done => {
                let g = 60 + 15 * self.alvl;
                self.player.gold += g;
                self.toast = Some((format!("+{} GOLD - UNDER THE FLAT STONE", g), 240));
                self.snd(Sfx::Coin, x, y, 1.0);
            }
            OVERLOOK => {
                let p = &mut self.player;
                p.hp = p.maxhp;
                p.mana = p.maxmana;
                self.snd(Sfx::Chime, x, y, 0.8);
            }
            PEN => {
                let g = 120 + 10 * self.alvl;
                self.player.gold += g;
                self.player.potions = (self.player.potions + 2).min(crate::shop::MAX_POTIONS);
                self.toast = Some((format!("+{} GOLD  +2 POTIONS", g), 240));
                self.snd(Sfx::Coin, x, y, 1.0);
                // he runs for home
                if let Some(n) = self.npcs.iter().position(|n| n.k == TOMAS) {
                    let (nx, ny) = (self.npcs[n].x, self.npcs[n].y);
                    self.npcs.remove(n);
                    for _ in 0..16 {
                        self.ptc(nx, ny, rr(0.0, 10.0), rr(-1.0, 1.0), rr(-0.5, 0.5), rr(0.2, 0.8), 60, PtK::Smoke, [110, 100, 90]);
                    }
                }
            }
            _ => {}
        }
        self.save();
        true
    }

    /// Ashford changes as you win
    fn ashford_life(&mut self) {
        let wsq = |g: &Game, x: f32, y: f32| (-4..=4).all(|d| !g.world.solid(x as i32 + d, y as i32));
        let mut extra = 0;
        if self.king_dead {
            extra += 2;
        }
        if self.chief_dead {
            extra += 2;
        }
        let mut k = 5u8;
        for _ in 0..300 {
            if extra == 0 {
                break;
            }
            let (x, y) = (rr(400.0, 640.0), rr(340.0, 430.0));
            if wsq(self, x, y) {
                self.npcs.push(crate::story::Npc { x, y, k, tx: x, ty: y, t: ri(200) as i32, face: 1.0, phase: rnd() * 6.0, bark: 0 });
                k += 1;
                extra -= 1;
            }
        }
        if self.lore_done & (1 << PEN) != 0 {
            for _ in 0..200 {
                let (x, y) = (rr(520.0, 590.0), rr(400.0, 440.0));
                if wsq(self, x, y) {
                    self.npcs.push(crate::story::Npc { x, y, k: TOMAS, tx: x, ty: y, t: 0, face: -1.0, phase: 0.0, bark: 0 });
                    break;
                }
            }
        }
        if self.chief_dead {
            // lanterns strung over the square, red, gold and green
            let cols = [[1.0, 0.35, 0.3], [1.0, 0.8, 0.3], [0.4, 1.0, 0.45], [0.5, 0.6, 1.0]];
            for k in 0..14 {
                let a = k as f32 / 14.0 * std::f32::consts::TAU;
                let (lx, ly) = (552.0 + a.cos() * 120.0, 400.0 + a.sin() * 60.0);
                if !self.world.solid(lx as i32, ly as i32) {
                    self.posts.push((lx, ly));
                    self.lamps.push((lx + 4.0, ly - 20.0, cols[k % 4]));
                }
            }
        }
    }

    /// what a villager says when you pass
    pub fn bark_for(&self, k: u8, i: usize) -> &'static str {
        let f = (self.frame / 600) as usize;
        if k == TOMAS {
            BARKS_TOMAS[(i + f) % BARKS_TOMAS.len()]
        } else if self.chief_dead {
            BARKS_CHIEF[(i * 3 + f) % BARKS_CHIEF.len()]
        } else if self.king_dead {
            BARKS_KING[(i * 3 + f) % BARKS_KING.len()]
        } else {
            crate::story::BARKS[(i * 3 + f) % crate::story::BARKS.len()]
        }
    }
}
