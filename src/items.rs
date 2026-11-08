// Loot: weapons, helms, armour and rings with rarities, random affixes and a few
// hand-made legendaries. Items are generated from the area level they drop in.
use crate::rng::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Weapon,
    Helm,
    Armor,
    Ring,
}
pub const SLOTS: [Slot; 4] = [Slot::Weapon, Slot::Helm, Slot::Armor, Slot::Ring];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WKind {
    Sword,
    Axe,
    Hammer,
    Spear,
    Dagger,
}
pub const WKINDS: [WKind; 5] = [WKind::Sword, WKind::Axe, WKind::Hammer, WKind::Spear, WKind::Dagger];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rarity {
    Common,
    Magic,
    Rare,
    Legendary,
}
impl Rarity {
    pub fn col(self) -> [u8; 3] {
        match self {
            Rarity::Common => [214, 208, 200],
            Rarity::Magic => [110, 150, 255],
            Rarity::Rare => [255, 214, 80],
            Rarity::Legendary => [255, 130, 40],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Legend {
    /// sword: every hit calls lightning down on the target
    Stormcaller,
    /// axe: severed limbs explode
    Gravecleaver,
    /// dagger: wounds fill with acid
    MiresKiss,
    /// hammer: hits freeze, frozen foes burst in a frost nova
    WintersGrudge,
    /// spear: thrusts breathe fire ahead
    Pyre,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Affix {
    Damage(i32),
    Keen(i32),
    Burning,
    Thundering,
    Corroding,
    Freezing,
    Leech(i32),
    Swift(i32),
    Crit(i32),
    Might(i32),
    Wisdom(i32),
    Armor(i32),
    Regen(i32),
    Speed(i32),
}
impl Affix {
    pub fn text(&self) -> String {
        match *self {
            Affix::Damage(v) => format!("+{}% DAMAGE", v),
            Affix::Keen(v) => format!("+{} CUT DEPTH", v),
            Affix::Burning => "HITS SET FOES ALIGHT".into(),
            Affix::Thundering => "HITS MAY CALL LIGHTNING".into(),
            Affix::Corroding => "HITS LEAVE ACID IN THE WOUND".into(),
            Affix::Freezing => "HITS MAY FREEZE".into(),
            Affix::Leech(v) => format!("HEAL {} PER HIT", v),
            Affix::Swift(v) => format!("+{}% ATTACK SPEED", v),
            Affix::Crit(v) => format!("+{}% CRITICAL CHANCE", v),
            Affix::Might(v) => format!("+{} HEALTH", v),
            Affix::Wisdom(v) => format!("+{} MANA", v),
            Affix::Armor(v) => format!("+{} ARMOUR", v),
            Affix::Regen(v) => format!("+{} MANA REGEN", v),
            Affix::Speed(v) => format!("+{}% MOVE SPEED", v),
        }
    }
    fn code(&self) -> String {
        match *self {
            Affix::Damage(v) => format!("d{}", v),
            Affix::Keen(v) => format!("k{}", v),
            Affix::Burning => "b".into(),
            Affix::Thundering => "t".into(),
            Affix::Corroding => "c".into(),
            Affix::Freezing => "f".into(),
            Affix::Leech(v) => format!("l{}", v),
            Affix::Swift(v) => format!("s{}", v),
            Affix::Crit(v) => format!("x{}", v),
            Affix::Might(v) => format!("m{}", v),
            Affix::Wisdom(v) => format!("w{}", v),
            Affix::Armor(v) => format!("a{}", v),
            Affix::Regen(v) => format!("r{}", v),
            Affix::Speed(v) => format!("v{}", v),
        }
    }
    fn from_code(s: &str) -> Option<Affix> {
        let (c, v) = s.split_at(1);
        let n = v.parse::<i32>().unwrap_or(0);
        Some(match c {
            "d" => Affix::Damage(n),
            "k" => Affix::Keen(n),
            "b" => Affix::Burning,
            "t" => Affix::Thundering,
            "c" => Affix::Corroding,
            "f" => Affix::Freezing,
            "l" => Affix::Leech(n),
            "s" => Affix::Swift(n),
            "x" => Affix::Crit(n),
            "m" => Affix::Might(n),
            "w" => Affix::Wisdom(n),
            "a" => Affix::Armor(n),
            "r" => Affix::Regen(n),
            "v" => Affix::Speed(n),
            _ => return None,
        })
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub slot: Slot,
    pub wk: WKind,
    pub rarity: Rarity,
    pub ilvl: i32,
    /// weapon damage, or armour value
    pub base: i32,
    pub affixes: Vec<Affix>,
    pub legend: Option<Legend>,
    pub name: String,
}

/// how each weapon type swings
pub struct WStats {
    pub frames: i32,
    pub reach: f32,
    pub arc: f32,
    pub power: f32,
    pub dmg_mult: f32,
}
pub fn wstats(k: WKind) -> WStats {
    match k {
        WKind::Sword => WStats { frames: 16, reach: 34.0, arc: 1.35, power: 26.0, dmg_mult: 1.0 },
        WKind::Axe => WStats { frames: 21, reach: 32.0, arc: 1.25, power: 40.0, dmg_mult: 1.35 },
        WKind::Hammer => WStats { frames: 25, reach: 32.0, arc: 1.3, power: 10.0, dmg_mult: 1.6 },
        WKind::Spear => WStats { frames: 15, reach: 50.0, arc: 0.45, power: 22.0, dmg_mult: 1.0 },
        WKind::Dagger => WStats { frames: 10, reach: 25.0, arc: 1.2, power: 16.0, dmg_mult: 0.6 },
    }
}

fn wk_name(k: WKind, tier: usize) -> &'static str {
    let t = tier.min(2);
    match k {
        WKind::Sword => ["RUSTY SWORD", "LONGSWORD", "BASTARD SWORD"][t],
        WKind::Axe => ["HATCHET", "BATTLE AXE", "EXECUTIONER AXE"][t],
        WKind::Hammer => ["MALLET", "WARHAMMER", "GRAVE MAUL"][t],
        WKind::Spear => ["PITCHFORK", "SPEAR", "GLAIVE"][t],
        WKind::Dagger => ["KNIFE", "DIRK", "STILETTO"][t],
    }
}
fn slot_name(s: Slot, tier: usize) -> &'static str {
    let t = tier.min(2);
    match s {
        Slot::Helm => ["LEATHER CAP", "IRON HELM", "GREAT HELM"][t],
        Slot::Armor => ["PADDED JERKIN", "CHAIN MAIL", "PLATE ARMOUR"][t],
        Slot::Ring => ["COPPER RING", "SILVER BAND", "GOLD SIGNET"][t],
        Slot::Weapon => "WEAPON",
    }
}

const PREFIX: [&str; 12] = ["CRUEL", "GRIM", "KEEN", "BITTER", "HOLLOW", "BLEAK", "SAVAGE", "PALE", "ASHEN", "WICKED", "ANCIENT", "WRETCHED"];
const SUFFIX: [&str; 12] = ["OF RUIN", "OF THE GRAVE", "OF EMBERS", "OF THE LEECH", "OF STORMS", "OF ROT", "OF WINTER", "OF THE PINES", "OF BONES", "OF DUSK", "OF THE CRYPT", "OF TEETH"];
const RARE_A: [&str; 12] = ["BONE", "GRAVE", "DREAD", "CORPSE", "STORM", "BLOOD", "DUSK", "RUIN", "HOLLOW", "ASH", "GLOOM", "WRAITH"];
const RARE_B: [&str; 12] = ["BITE", "EDGE", "REAPER", "SONG", "FANG", "WARD", "SHROUD", "HUNGER", "OATH", "MAW", "THORN", "VIGIL"];

fn roll_affix(slot: Slot, ilvl: i32) -> Affix {
    let s = 1.0 + ilvl as f32 * 0.12;
    let v = |a: f32, b: f32| (rr(a, b) * s).round() as i32;
    match slot {
        Slot::Weapon => match ri(9) {
            0 | 1 => Affix::Damage(v(8.0, 18.0)),
            2 => Affix::Keen(v(4.0, 9.0)),
            3 => Affix::Burning,
            4 => Affix::Thundering,
            5 => Affix::Corroding,
            6 => Affix::Freezing,
            7 => Affix::Leech(v(1.0, 2.5).max(1)),
            _ => Affix::Swift(v(6.0, 12.0)),
        },
        Slot::Ring => match ri(7) {
            0 => Affix::Crit(v(3.0, 6.0)),
            1 => Affix::Might(v(6.0, 14.0)),
            2 => Affix::Wisdom(v(6.0, 14.0)),
            3 => Affix::Regen(v(2.0, 5.0)),
            4 => Affix::Damage(v(5.0, 10.0)),
            5 => Affix::Leech(v(1.0, 2.0).max(1)),
            _ => Affix::Speed(v(3.0, 6.0)),
        },
        _ => match ri(6) {
            0 | 1 => Affix::Armor(v(3.0, 7.0)),
            2 => Affix::Might(v(8.0, 18.0)),
            3 => Affix::Wisdom(v(5.0, 12.0)),
            4 => Affix::Speed(v(3.0, 7.0)),
            _ => Affix::Regen(v(2.0, 4.0)),
        },
    }
}

pub fn legendary(l: Legend, ilvl: i32) -> Item {
    let (wk, name, aff): (WKind, &str, Vec<Affix>) = match l {
        Legend::Stormcaller => (WKind::Sword, "STORMCALLER", vec![Affix::Damage(25), Affix::Swift(10)]),
        Legend::Gravecleaver => (WKind::Axe, "GRAVECLEAVER", vec![Affix::Keen(14), Affix::Damage(20)]),
        Legend::MiresKiss => (WKind::Dagger, "THE MIRE'S KISS", vec![Affix::Swift(15), Affix::Leech(3)]),
        Legend::WintersGrudge => (WKind::Hammer, "WINTER'S GRUDGE", vec![Affix::Damage(30), Affix::Armor(8)]),
        Legend::Pyre => (WKind::Spear, "PYRE", vec![Affix::Damage(20), Affix::Burning]),
    };
    Item { slot: Slot::Weapon, wk, rarity: Rarity::Legendary, ilvl, base: weapon_base(wk, ilvl) + 6, affixes: aff, legend: Some(l), name: name.into() }
}

fn weapon_base(wk: WKind, ilvl: i32) -> i32 {
    let _ = wk;
    (11.0 + ilvl as f32 * 1.7) as i32
}

pub fn starter_weapon() -> Item {
    Item { slot: Slot::Weapon, wk: WKind::Sword, rarity: Rarity::Common, ilvl: 1, base: 12, affixes: vec![], legend: None, name: "RUSTY SWORD".into() }
}
pub fn starter_armor() -> Item {
    Item { slot: Slot::Armor, wk: WKind::Sword, rarity: Rarity::Common, ilvl: 1, base: 3, affixes: vec![], legend: None, name: "PADDED JERKIN".into() }
}

/// roll a random item for an area level; `luck` shifts rarity upward
pub fn roll(ilvl: i32, luck: f32) -> Item {
    let r = rnd() - luck;
    let rarity = if r < 0.006 + ilvl as f32 * 0.0008 {
        Rarity::Legendary
    } else if r < 0.07 + ilvl as f32 * 0.004 {
        Rarity::Rare
    } else if r < 0.40 {
        Rarity::Magic
    } else {
        Rarity::Common
    };
    if rarity == Rarity::Legendary {
        let l = [Legend::Stormcaller, Legend::Gravecleaver, Legend::MiresKiss, Legend::WintersGrudge, Legend::Pyre][ri(5)];
        return legendary(l, ilvl);
    }
    let slot = [Slot::Weapon, Slot::Weapon, Slot::Weapon, Slot::Helm, Slot::Armor, Slot::Ring][ri(6)];
    let wk = WKINDS[ri(5)];
    let tier = (ilvl as usize / 4).min(2);
    let base = match slot {
        Slot::Weapon => weapon_base(wk, ilvl) + ri(4) as i32,
        Slot::Ring => 0,
        Slot::Helm => (2.0 + ilvl as f32 * 0.6) as i32 + ri(2) as i32,
        Slot::Armor => (3.0 + ilvl as f32 * 1.0) as i32 + ri(3) as i32,
    };
    let n_aff = match rarity {
        Rarity::Common => {
            if slot == Slot::Ring {
                1
            } else {
                0
            }
        }
        Rarity::Magic => 1 + ri(2),
        _ => 3 + ri(2),
    };
    let mut affixes: Vec<Affix> = vec![];
    for _ in 0..n_aff * 3 {
        if affixes.len() >= n_aff {
            break;
        }
        let a = roll_affix(slot, ilvl);
        if !affixes.iter().any(|b| std::mem::discriminant(b) == std::mem::discriminant(&a)) {
            affixes.push(a);
        }
    }
    let base_name = if slot == Slot::Weapon { wk_name(wk, tier) } else { slot_name(slot, tier) };
    let name = match rarity {
        Rarity::Common => base_name.to_string(),
        Rarity::Magic => {
            if rnd() < 0.5 {
                format!("{} {}", PREFIX[ri(12)], base_name)
            } else {
                format!("{} {}", base_name, SUFFIX[ri(12)])
            }
        }
        _ => format!("{} {}", RARE_A[ri(12)], RARE_B[ri(12)]),
    };
    Item { slot, wk, rarity, ilvl, base, affixes, legend: None, name }
}

impl Item {
    /// one-line kind description for tooltips
    pub fn kind_text(&self) -> String {
        match self.slot {
            Slot::Weapon => format!("{:?}", self.wk).to_uppercase(),
            s => format!("{:?}", s).to_uppercase(),
        }
    }
    pub fn base_text(&self) -> String {
        match self.slot {
            Slot::Weapon => format!("{} DAMAGE", self.base),
            Slot::Ring => String::new(),
            _ => format!("{} ARMOUR", self.base),
        }
    }
    pub fn legend_text(&self) -> Option<&'static str> {
        self.legend.map(|l| match l {
            Legend::Stormcaller => "EVERY HIT CALLS LIGHTNING DOWN ON THE TARGET",
            Legend::Gravecleaver => "SEVERED LIMBS EXPLODE",
            Legend::MiresKiss => "WOUNDS FILL WITH ACID",
            Legend::WintersGrudge => "HITS FREEZE - FROZEN FOES BURST IN FROST",
            Legend::Pyre => "THRUSTS BREATHE FIRE",
        })
    }
    /// a rough measure of how good an item is, for "is this an upgrade?"
    pub fn score(&self) -> f32 {
        let mut s = match self.slot {
            Slot::Weapon => self.base as f32,
            Slot::Ring => 0.0,
            _ => self.base as f32 * 1.6,
        };
        let mut mult = 1.0;
        for a in &self.affixes {
            match *a {
                Affix::Damage(v) => mult += v as f32 / 100.0,
                Affix::Swift(v) => mult += v as f32 / 140.0,
                Affix::Keen(v) | Affix::Crit(v) => s += v as f32 * 0.8,
                Affix::Burning | Affix::Thundering | Affix::Corroding | Affix::Freezing => s += 6.0,
                Affix::Leech(v) => s += v as f32 * 3.0,
                Affix::Might(v) | Affix::Wisdom(v) => s += v as f32 * 0.3,
                Affix::Armor(v) => s += v as f32 * 1.6,
                Affix::Regen(v) | Affix::Speed(v) => s += v as f32 * 1.2,
            }
        }
        if self.legend.is_some() {
            s += 20.0;
        }
        s * mult
    }
    pub fn value(&self) -> i32 {
        let r = match self.rarity {
            Rarity::Common => 1,
            Rarity::Magic => 3,
            Rarity::Rare => 8,
            Rarity::Legendary => 25,
        };
        (5 + self.ilvl * 2) * r
    }
    /// single-line save format
    pub fn encode(&self) -> String {
        let aff: Vec<String> = self.affixes.iter().map(|a| a.code()).collect();
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            SLOTS.iter().position(|s| *s == self.slot).unwrap(),
            WKINDS.iter().position(|k| *k == self.wk).unwrap(),
            self.rarity as u8,
            self.ilvl,
            self.base,
            aff.join(","),
            self.legend.map(|l| l as u8 as i32).unwrap_or(-1),
            self.name
        )
    }
    pub fn decode(s: &str) -> Option<Item> {
        let f: Vec<&str> = s.splitn(8, '|').collect();
        if f.len() < 8 {
            return None;
        }
        let slot = *SLOTS.get(f[0].parse::<usize>().ok()?)?;
        let wk = *WKINDS.get(f[1].parse::<usize>().ok()?)?;
        let rarity = [Rarity::Common, Rarity::Magic, Rarity::Rare, Rarity::Legendary][f[2].parse::<usize>().ok()?.min(3)];
        let affixes = f[5].split(',').filter(|x| !x.is_empty()).filter_map(Affix::from_code).collect();
        let li: i32 = f[6].parse().unwrap_or(-1);
        let legend = if li < 0 { None } else { [Legend::Stormcaller, Legend::Gravecleaver, Legend::MiresKiss, Legend::WintersGrudge, Legend::Pyre].get(li as usize).copied() };
        Some(Item { slot, wk, rarity, ilvl: f[3].parse().ok()?, base: f[4].parse().ok()?, affixes, legend, name: f[7].to_string() })
    }
}

/// everything your gear adds up to
#[derive(Default, Clone, Copy)]
pub struct GearStats {
    pub weapon_dmg: f32,
    pub dmg_pct: f32,
    pub keen: f32,
    pub burning: bool,
    pub thundering: bool,
    pub corroding: bool,
    pub freezing: bool,
    pub leech: f32,
    pub swift: f32,
    pub crit: f32,
    pub hp: f32,
    pub mana: f32,
    pub armor: f32,
    pub regen: f32,
    pub speed: f32,
}
pub fn gear_stats(equip: &[Option<Item>; 4]) -> GearStats {
    let mut g = GearStats { weapon_dmg: 8.0, ..Default::default() };
    for it in equip.iter().flatten() {
        match it.slot {
            Slot::Weapon => g.weapon_dmg = it.base as f32,
            Slot::Helm | Slot::Armor => g.armor += it.base as f32,
            Slot::Ring => {}
        }
        for a in &it.affixes {
            match *a {
                Affix::Damage(v) => g.dmg_pct += v as f32,
                Affix::Keen(v) => g.keen += v as f32,
                Affix::Burning => g.burning = true,
                Affix::Thundering => g.thundering = true,
                Affix::Corroding => g.corroding = true,
                Affix::Freezing => g.freezing = true,
                Affix::Leech(v) => g.leech += v as f32,
                Affix::Swift(v) => g.swift += v as f32,
                Affix::Crit(v) => g.crit += v as f32,
                Affix::Might(v) => g.hp += v as f32,
                Affix::Wisdom(v) => g.mana += v as f32,
                Affix::Armor(v) => g.armor += v as f32,
                Affix::Regen(v) => g.regen += v as f32,
                Affix::Speed(v) => g.speed += v as f32,
            }
        }
    }
    g
}
