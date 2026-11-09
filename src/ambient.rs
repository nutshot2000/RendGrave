// Ambient life: small things drifting through the view so no area ever looks frozen.
// Fireflies in the woods, fog over the graves, mist and bubbles in the fen, embers on the
// wind at Ravenhold, dust in the dungeons.
use crate::areagen::AreaId;
use crate::game::*;
use crate::rng::*;
use crate::world::*;

impl Game {
    pub fn ambient(&mut self) {
        if self.ptcs.len() > 1400 {
            return;
        }
        let (cx, cy) = self.cam;
        let (vw, vh) = self.view;
        let spot = |m: f32| (cx + rr(-m, vw + m), cy + rr(-m, vh + 60.0));
        let f = self.frame;
        match self.area {
            AreaId::Ashford | AreaId::Pines => {
                // fireflies: slow, wandering, softly lit
                if f % 7 == 0 {
                    let (x, y) = spot(20.0);
                    if self.world.at(x as i32, y as i32) == E {
                        self.ptc(x, y, rr(6.0, 30.0), rr(-0.25, 0.25), rr(-0.15, 0.15), rr(-0.05, 0.05), rr(160.0, 260.0) as i32, PtK::Heal, [140, 190, 60]);
                    }
                }
            }
            AreaId::Graveyard => {
                // low fog rolling between the stones
                if f % 3 == 0 {
                    let (x, y) = spot(60.0);
                    self.ptc(x, y, 1.0, rr(0.1, 0.3), rr(-0.03, 0.03), 0.0, rr(220.0, 320.0) as i32, PtK::Cloud, [118, 120, 132]);
                }
                if f % 11 == 0 {
                    let (x, y) = spot(20.0);
                    self.ptc(x, y, rr(6.0, 30.0), rr(-0.2, 0.2), rr(-0.1, 0.1), 0.0, 200, PtK::Heal, [90, 120, 170]);
                }
            }
            AreaId::Fen => {
                // mist over the bog, and bubbles rising through the black water
                if f % 3 == 0 {
                    let (x, y) = spot(60.0);
                    self.ptc(x, y, 1.0, rr(0.05, 0.2), rr(-0.03, 0.03), 0.0, rr(220.0, 320.0) as i32, PtK::Cloud, [86, 108, 80]);
                }
                for _ in 0..2 {
                    let (x, y) = spot(0.0);
                    if self.world.at(x as i32, y as i32) == WATER && rnd() < 0.25 {
                        self.ptc(x, y, 0.5, 0.0, 0.0, 0.25, 30, PtK::Steam, [120, 150, 110]);
                    }
                }
                if f % 9 == 0 {
                    let (x, y) = spot(20.0);
                    self.ptc(x, y, rr(4.0, 24.0), rr(-0.2, 0.2), rr(-0.1, 0.1), 0.0, 220, PtK::Heal, [120, 200, 80]);
                }
            }
            AreaId::Ravenhold => {
                // embers from the cookfires, carried on the wind
                if f % 4 == 0 {
                    let (x, y) = spot(40.0);
                    self.ptc(x, y, rr(0.0, 20.0), rr(0.2, 0.6), rr(-0.1, 0.1), rr(0.4, 0.9), rr(60.0, 110.0) as i32, PtK::Ember, [255, 150, 60]);
                }
            }
            AreaId::Crypt(_) | AreaId::Cellars(_) => {
                if f % 20 == 0 {
                    for &(x, y) in self.cracks.clone().iter() {
                        if (x - cx - vw / 2.0).abs() < vw && (y - cy - vh / 2.0).abs() < vh {
                            self.ptc(x + rr(-8.0, 8.0), y + 2.0, rr(10.0, 36.0), rr(-0.1, 0.1), 0.15, 0.05, 90, PtK::Smoke, [130, 120, 110]);
                        }
                    }
                }
                // dust hanging in the still air
                if f % 5 == 0 {
                    let (x, y) = spot(10.0);
                    if self.world.at(x as i32, y as i32) == E {
                        self.ptc(x, y, rr(4.0, 40.0), rr(-0.05, 0.05), rr(-0.03, 0.03), rr(-0.02, 0.02), 240, PtK::Heal, [52, 48, 44]);
                    }
                }
            }
        }
    }
}
