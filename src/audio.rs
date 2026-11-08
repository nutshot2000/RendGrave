// Audio: music that follows the area (crossfading, boss theme, death theme) and sound
// effects synthesised in code at startup. Effects are panned and attenuated by where
// they happen relative to the player.
use rodio::buffer::SamplesBuffer;
use rodio::mixer::Mixer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::collections::HashMap;
use std::num::NonZero;

pub const SR: u32 = 44100;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Swing,
    HitFlesh,
    HitBone,
    HitMetal,
    Sever,
    Zap,
    Thunder,
    Fireball,
    Explosion,
    Acid,
    Frost,
    Shatter,
    Step,
    Groan,
    Rattle,
    Roar,
    KingRoar,
    Bow,
    ArrowHit,
    Coin,
    Potion,
    LevelUp,
    Hurt,
    Death,
    Click,
    Travel,
    Chime,
    Dodge,
    Whirl,
    Wood,
    Crumble,
}

pub struct Snd {
    pub k: Sfx,
    pub x: f32,
    pub y: f32,
    pub v: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Title,
    Ashford,
    Pines,
    Crypt,
    Boss,
}
impl Track {
    fn file(self) -> &'static str {
        match self {
            Track::Title => "rg_title.wav",
            Track::Ashford => "rg_ashford.wav",
            Track::Pines => "rg_pines.wav",
            Track::Crypt => "rg_crypt.wav",
            Track::Boss => "rg_boss.wav",
        }
    }
}

struct Playing {
    track: Track,
    player: Player,
    vol: f32,
    target: f32,
}

pub struct Audio {
    _sink: MixerDeviceSink,
    mixer: Mixer,
    music: Vec<Playing>,
    cache: HashMap<&'static str, SamplesBuffer>,
    bank: HashMap<Sfx, Vec<Vec<f32>>>,
    last: HashMap<Sfx, u64>,
    pub music_on: bool,
    pub sfx_on: bool,
    pub volume: f32,
    frame: u64,
    voices: Vec<(u64, u32)>,
}

impl Audio {
    pub fn new() -> Option<Audio> {
        let sink = DeviceSinkBuilder::open_default_sink().ok()?;
        let mixer = sink.mixer().clone();
        let mut a = Audio {
            _sink: sink,
            mixer,
            music: vec![],
            cache: HashMap::new(),
            bank: crate::synth::build_bank(),
            last: HashMap::new(),
            music_on: true,
            sfx_on: true,
            volume: 0.8,
            frame: 0,
            voices: vec![],
        };
        a.load_settings();
        Some(a)
    }

    fn music_dir() -> Option<std::path::PathBuf> {
        let mut cands = vec![std::path::PathBuf::from("assets/music")];
        if let Ok(exe) = std::env::current_exe() {
            let mut d = exe.parent().map(|p| p.to_path_buf());
            for _ in 0..4 {
                if let Some(p) = d {
                    cands.push(p.join("assets/music"));
                    d = p.parent().map(|q| q.to_path_buf());
                }
            }
        }
        cands.into_iter().find(|p| p.is_dir())
    }

    fn load(&mut self, t: Track) -> Option<SamplesBuffer> {
        if let Some(b) = self.cache.get(t.file()) {
            return Some(b.clone());
        }
        let path = Self::music_dir()?.join(t.file());
        let bytes = std::fs::read(path).ok()?;
        let (ch, rate, data) = read_wav(&bytes)?;
        let buf = SamplesBuffer::new(NonZero::new(ch)?, NonZero::new(rate)?, data);
        self.cache.insert(t.file(), buf.clone());
        Some(buf)
    }

    /// ask for a track; it crossfades in over the current one
    pub fn set_music(&mut self, want: Option<Track>, duck: f32) {
        let level = if self.music_on { self.volume * 0.55 * duck } else { 0.0 };
        let cur = self.music.iter().find(|p| p.target > 0.0).map(|p| p.track);
        if want != cur {
            for p in self.music.iter_mut() {
                p.target = 0.0;
            }
            if let Some(t) = want {
                if let Some(p) = self.music.iter_mut().find(|p| p.track == t) {
                    p.target = level.max(0.0001);
                } else if let Some(buf) = self.load(t) {
                    let player = Player::connect_new(&self.mixer);
                    player.set_volume(0.0);
                    player.append(buf.repeat_infinite());
                    self.music.push(Playing { track: t, player, vol: 0.0, target: level.max(0.0001) });
                }
            }
        } else {
            for p in self.music.iter_mut() {
                if p.target > 0.0 {
                    p.target = level.max(0.0001);
                }
            }
        }
    }

    /// once per frame: fades, and play the frame's sound effects
    pub fn update(&mut self, sounds: &[Snd], listener: (f32, f32)) {
        self.frame += 1;
        for p in self.music.iter_mut() {
            let step = 1.0 / 90.0;
            if p.vol < p.target {
                p.vol = (p.vol + step).min(p.target);
            } else if p.vol > p.target {
                p.vol = (p.vol - step).max(p.target);
            }
            p.player.set_volume(p.vol);
        }
        self.music.retain(|p| {
            let keep = p.vol > 0.0 || p.target > 0.0;
            if !keep {
                p.player.stop();
            }
            keep
        });
        self.voices.retain(|&(end, _)| end > self.frame);
        if !self.sfx_on {
            return;
        }
        for s in sounds {
            let min_gap = match s.k {
                Sfx::Step => 6,
                Sfx::HitFlesh | Sfx::HitBone | Sfx::Zap | Sfx::Coin => 3,
                Sfx::Groan | Sfx::Rattle => 20,
                _ => 2,
            };
            if let Some(&l) = self.last.get(&s.k) {
                if self.frame - l < min_gap {
                    continue;
                }
            }
            if self.voices.len() > 24 {
                break;
            }
            let (dx, dy) = (s.x - listener.0, (s.y - listener.1) * 1.3);
            let d = (dx * dx + dy * dy).sqrt();
            let att = (1.0 - d / 360.0).clamp(0.0, 1.0);
            let gain = s.v * att * att * self.volume;
            if gain < 0.01 {
                continue;
            }
            let Some(vars) = self.bank.get(&s.k) else { continue };
            let mono = &vars[(self.frame as usize + (s.x as usize)) % vars.len()];
            let pan = (dx / 240.0).clamp(-1.0, 1.0);
            let (gl, gr) = (gain * (1.0 - pan.max(0.0) * 0.7), gain * (1.0 + pan.min(0.0) * 0.7));
            let mut st = Vec::with_capacity(mono.len() * 2);
            for &v in mono {
                st.push(v * gl);
                st.push(v * gr);
            }
            let pitch = 0.92 + ((self.frame * 7919 + s.x as u64 * 31) % 160) as f32 / 1000.0;
            let len_frames = (mono.len() as f32 / SR as f32 * 60.0 / pitch) as u64 + 1;
            let buf = SamplesBuffer::new(NonZero::new(2).unwrap(), NonZero::new(SR).unwrap(), st);
            self.mixer.add(buf.speed(pitch));
            self.last.insert(s.k, self.frame);
            self.voices.push((self.frame + len_frames, 0));
        }
    }

    pub fn change_volume(&mut self, d: f32) {
        self.volume = (self.volume + d).clamp(0.0, 1.0);
        self.save_settings();
    }
    pub fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
        self.save_settings();
    }
    pub fn toggle_sfx(&mut self) {
        self.sfx_on = !self.sfx_on;
        self.save_settings();
    }
    fn save_settings(&self) {
        let _ = std::fs::write("rendgrave_settings.txt", format!("volume {}\nmusic {}\nsfx {}\n", self.volume, self.music_on as u8, self.sfx_on as u8));
    }
    fn load_settings(&mut self) {
        let Ok(t) = std::fs::read_to_string("rendgrave_settings.txt") else { return };
        for l in t.lines() {
            let mut it = l.split_whitespace();
            match (it.next(), it.next()) {
                (Some("volume"), Some(v)) => self.volume = v.parse().unwrap_or(0.8),
                (Some("music"), Some(v)) => self.music_on = v == "1",
                (Some("sfx"), Some(v)) => self.sfx_on = v == "1",
                _ => {}
            }
        }
    }
}

/// minimal PCM WAV reader (16-bit integer or 32-bit float)
fn read_wav(b: &[u8]) -> Option<(u16, u32, Vec<f32>)> {
    if b.len() < 12 || &b[0..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return None;
    }
    let mut i = 12;
    let (mut ch, mut rate, mut bits, mut fmt) = (2u16, 44100u32, 16u16, 1u16);
    while i + 8 <= b.len() {
        let id = &b[i..i + 4];
        let sz = u32::from_le_bytes([b[i + 4], b[i + 5], b[i + 6], b[i + 7]]) as usize;
        let body = &b[i + 8..(i + 8 + sz).min(b.len())];
        if id == b"fmt " && body.len() >= 16 {
            fmt = u16::from_le_bytes([body[0], body[1]]);
            ch = u16::from_le_bytes([body[2], body[3]]);
            rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
            bits = u16::from_le_bytes([body[14], body[15]]);
        } else if id == b"data" {
            let data: Vec<f32> = match (fmt, bits) {
                (1, 16) => body.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect(),
                (3, 32) => body.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
                _ => return None,
            };
            return Some((ch, rate, data));
        }
        i += 8 + sz + (sz & 1);
    }
    None
}
