// The title screen and the pause menu (both clickable lists), and the end-of-act card.
// Things only the window or the audio can do (quit, fullscreen, volume, starting a new
// game) are passed up to main.rs as a `Req`.
use crate::audio::Sfx;
use crate::game::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Req {
    Quit,
    Fullscreen,
    Music,
    Sound,
    Volume(f32),
    NewGame,
    Continue,
    ToTitle,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MI {
    Continue,
    NewGame,
    Resume,
    Music,
    Sound,
    Volume,
    Fullscreen,
    ToTitle,
    Quit,
}

impl Game {
    pub fn menu_items(&self) -> Vec<MI> {
        if self.title {
            let mut v = vec![];
            if self.has_save {
                v.push(MI::Continue);
            }
            v.extend([MI::NewGame, MI::Music, MI::Sound, MI::Volume, MI::Fullscreen, MI::Quit]);
            v
        } else {
            vec![MI::Resume, MI::Music, MI::Sound, MI::Volume, MI::Fullscreen, MI::ToTitle, MI::Quit]
        }
    }

    pub fn menu_text(&self, m: MI) -> String {
        let (music, sfx, vol) = self.audio_state;
        match m {
            MI::Continue => "CONTINUE".into(),
            MI::NewGame => {
                if self.confirm_new {
                    "START OVER? CLICK AGAIN".into()
                } else {
                    "NEW GAME".into()
                }
            }
            MI::Resume => "RESUME".into(),
            MI::Music => format!("MUSIC  {}", if music { "ON" } else { "OFF" }),
            MI::Sound => format!("SOUND  {}", if sfx { "ON" } else { "OFF" }),
            MI::Volume => format!("-   VOLUME {}%   +", (vol * 100.0).round() as i32),
            MI::Fullscreen => "FULLSCREEN  F11".into(),
            MI::ToTitle => "SAVE AND QUIT TO TITLE".into(),
            MI::Quit => if self.title { "QUIT".into() } else { "SAVE AND QUIT GAME".into() },
        }
    }

    /// mouse (or W / S and E / Space) over the current menu
    pub fn update_menu(&mut self, inp: &Input) {
        let items = self.menu_items();
        let n = items.len();
        let hov = crate::hud::menu_at(inp.mouse, self.view, n, self.title);
        if hov.is_some() && (inp.mouse != self.prev.mouse) {
            self.menu_hover = hov;
        }
        if inp.up && !self.prev.up {
            self.menu_hover = Some(self.menu_hover.map(|k| (k + n - 1) % n).unwrap_or(0));
        }
        if inp.down && !self.prev.down {
            self.menu_hover = Some(self.menu_hover.map(|k| (k + 1) % n).unwrap_or(0));
        }
        let click = inp.lmb && !self.prev.lmb && hov.is_some();
        let key = (inp.interact && !self.prev.interact) || (inp.dodge && !self.prev.dodge);
        let pick = if click { hov } else if key { self.menu_hover } else { None };
        let Some(k) = pick else { return };
        let Some(&m) = items.get(k) else { return };
        let (x, y) = (self.player.x, self.player.y);
        self.snd(Sfx::Click, x, y, 0.7);
        if m != MI::NewGame {
            self.confirm_new = false;
        }
        self.req = match m {
            MI::Continue => Some(Req::Continue),
            MI::NewGame => {
                if self.has_save && !self.confirm_new {
                    self.confirm_new = true;
                    None
                } else {
                    Some(Req::NewGame)
                }
            }
            MI::Resume => {
                self.paused = false;
                self.hold_block = true;
                None
            }
            MI::Music => Some(Req::Music),
            MI::Sound => Some(Req::Sound),
            MI::Volume => {
                // left half turns it down, right half up
                let (bx, _, bw, _) = crate::hud::menu_row(k, self.view.0 as i32, self.view.1 as i32, self.title);
                Some(Req::Volume(if click && (inp.mouse.0 as i32) < bx + bw / 2 { -0.1 } else { 0.1 }))
            }
            MI::Fullscreen => Some(Req::Fullscreen),
            MI::ToTitle => Some(Req::ToTitle),
            MI::Quit => Some(Req::Quit),
        };
    }

    /// when both of Act I's bosses are dead, show the end card a few seconds later
    pub fn check_act_end(&mut self) {
        if self.king_dead && self.chief_dead && !self.act1_done {
            self.act1_done = true;
            self.ending = 360;
            self.save();
        }
    }
}
