// The skill tree: six schools of nine skills each. Every school branches: a root skill,
// two paths of three skills, an ultimate at the end of either path, and a mastery that
// improves the ultimate. Actives go on the spell bar, passives change everything else.
// With about 50 points over a whole game you can master roughly two schools, so builds differ.
//
// The order of `Sk` and `NODES` is the save format (ranks are saved by index): only ever add
// new skills at the end.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sk {
    Lightning,
    ChainMastery,
    BallLightning,
    Thunderstorm,
    Fireball,
    Kindling,
    FireWall,
    Meteor,
    Acid,
    Corrosion,
    AcidRain,
    AcidFlood,
    Nova,
    Brittle,
    IceSpikes,
    Blizzard,
    Edge,
    Whirlwind,
    Bloodthirst,
    Executioner,
    Vigor,
    Wisdom,
    Swiftness,
    Fortune,
    // ---- added with the deeper trees
    Conductor,
    Overload,
    StaticField,
    StormShield,
    EyeOfStorm,
    Searing,
    Splinter,
    Fireproof,
    Combustion,
    MeteorShower,
    Caustic,
    PoisonCloud,
    AcidPools,
    RotBurst,
    Plague,
    Chill,
    IceArmor,
    Shatterstorm,
    DeepFreeze,
    AbsoluteZero,
    Cleave,
    Dismember,
    Momentum,
    BladeFury,
    Warlord,
    Toughness,
    SecondWind,
    Regeneration,
    LastStand,
    Titan,
}

pub const N: usize = 54;

pub struct Node {
    pub sk: Sk,
    pub name: &'static str,
    pub desc: &'static str,
    /// school (tab)
    pub col: usize,
    /// tier, top to bottom
    pub row: usize,
    /// 0 left path, 1 middle, 2 right path
    pub lane: usize,
    pub max: u8,
    pub active: bool,
    pub mana: f32,
    pub cd: i32,
    /// needs at least one point in any of these first
    pub req: &'static [Sk],
}

pub const SCHOOLS: [(&str, [u8; 3]); 6] = [
    ("STORM", [150, 190, 255]),
    ("FLAME", [255, 150, 60]),
    ("VENOM", [140, 230, 70]),
    ("FROST", [170, 225, 255]),
    ("BLADE", [230, 225, 215]),
    ("BODY", [240, 110, 120]),
];

/// Minimum character level for a node, by tier. Blade and Body come early; the spell schools
/// open at level 4; ultimates need level 30 (and, if they are spells, a tome found in the world).
pub fn min_level(i: usize) -> i32 {
    let n = &NODES[i];
    if n.col >= 4 {
        [1, 3, 6, 12, 22, 34][n.row]
    } else {
        [4, 6, 10, 18, 30, 40][n.row]
    }
}
/// ultimate spells must be found as tomes before they can be learned
pub fn needs_tome(i: usize) -> bool {
    NODES[i].active && NODES[i].row == 4
}

macro_rules! node {
    ($sk:ident, $name:expr, $desc:expr, $c:expr, $r:expr, $lane:expr, $max:expr, $act:expr, $mana:expr, $cd:expr, [$($req:ident),*]) => {
        Node { sk: Sk::$sk, name: $name, desc: $desc, col: $c, row: $r, lane: $lane, max: $max, active: $act, mana: $mana, cd: $cd, req: &[$(Sk::$req),*] }
    };
}

pub const NODES: [Node; N] = [
    node!(Lightning, "LIGHTNING", "A BOLT THAT CHAINS BETWEEN FOES AND RUNS THROUGH WATER AND BLOOD.", 0, 0, 1, 1, true, 14.0, 22, []),
    node!(ChainMastery, "CHAIN MASTERY", "+1 CHAIN JUMP AND +15% LIGHTNING DAMAGE PER RANK.", 0, 1, 0, 3, false, 0.0, 0, [Lightning]),
    node!(BallLightning, "BALL LIGHTNING", "A SLOW ORB THAT ZAPS EVERY ENEMY IT DRIFTS PAST.", 0, 2, 0, 1, true, 24.0, 70, [ChainMastery]),
    node!(Thunderstorm, "THUNDERSTORM", "FOR 6 SECONDS THE SKY STRIKES ENEMIES AROUND YOU.", 0, 4, 1, 1, true, 45.0, 600, [StaticField, StormShield]),
    node!(Fireball, "FIREBALL", "EXPLODES, SETS FLESH, CLOTH, GRASS AND OIL ALIGHT.", 1, 0, 1, 1, true, 20.0, 36, []),
    node!(Kindling, "KINDLING", "BIGGER FIREBALLS AND +30% BURN DAMAGE PER RANK.", 1, 1, 0, 3, false, 0.0, 0, [Fireball]),
    node!(FireWall, "WALL OF FIRE", "A LINE OF FLAME TOWARDS THE CURSOR THAT BURNS FOR SECONDS.", 1, 2, 0, 1, true, 26.0, 120, [Kindling]),
    node!(Meteor, "METEOR", "AFTER A MOMENT A METEOR CRASHES DOWN AT THE CURSOR.", 1, 4, 1, 1, true, 50.0, 420, [Fireproof, Combustion]),
    node!(Acid, "ACID SHOT", "LOBS GLOBS OF ACID THAT EAT FLESH TO THE BONE.", 2, 0, 1, 1, true, 10.0, 16, []),
    node!(Corrosion, "CORROSION", "+1 GLOB AND STRONGER ACID PER RANK.", 2, 1, 0, 3, false, 0.0, 0, [Acid]),
    node!(AcidRain, "ACID RAIN", "A CLOUD THAT RAINS ACID ON AN AREA FOR 5 SECONDS.", 2, 2, 0, 1, true, 30.0, 300, [Corrosion]),
    node!(AcidFlood, "ACID FLOOD", "A WAVE OF ACID BURSTS OUT AND SPREADS ACROSS THE GROUND.", 2, 4, 1, 1, true, 45.0, 420, [AcidPools, RotBurst]),
    node!(Nova, "FROST NOVA", "FREEZES EVERYTHING AROUND YOU. FROZEN FOES SHATTER.", 3, 0, 1, 1, true, 28.0, 80, []),
    node!(Brittle, "BRITTLE", "FROZEN LONGER AND A WIDER NOVA PER RANK.", 3, 1, 0, 3, false, 0.0, 0, [Nova]),
    node!(IceSpikes, "ICE SPIKES", "SPIKES TEAR UP FROM THE GROUND IN A LINE AND TAKE LEGS OFF.", 3, 2, 0, 1, true, 24.0, 90, [Brittle]),
    node!(Blizzard, "BLIZZARD", "A STORM OF ICE AT THE CURSOR THAT CHILLS AND FREEZES.", 3, 4, 1, 1, true, 45.0, 480, [Shatterstorm, DeepFreeze]),
    node!(Edge, "KEEN EDGE", "YOUR BLADE CUTS DEEPER AND HITS 10% HARDER PER RANK.", 4, 0, 1, 3, false, 0.0, 0, []),
    node!(Whirlwind, "WHIRLWIND", "SPIN WITH YOUR BLADE AND CUT EVERYTHING AROUND YOU.", 4, 1, 0, 1, true, 18.0, 90, [Edge]),
    node!(Bloodthirst, "BLOODTHIRST", "HEAL A LITTLE FOR EVERY CUT THAT DRAWS BLOOD.", 4, 2, 0, 3, false, 0.0, 0, [Whirlwind]),
    node!(Executioner, "EXECUTIONER", "+8% CRITICAL CHANCE PER RANK. CRITS CUT FAR DEEPER.", 4, 3, 0, 3, false, 0.0, 0, [Bloodthirst]),
    node!(Vigor, "VIGOR", "+15 MAXIMUM HEALTH PER RANK.", 5, 0, 1, 5, false, 0.0, 0, []),
    node!(Wisdom, "WISDOM", "+15 MAXIMUM MANA AND FASTER MANA REGEN PER RANK.", 5, 1, 0, 5, false, 0.0, 0, [Vigor]),
    node!(Swiftness, "SWIFTNESS", "+7% MOVE SPEED AND QUICKER DODGES PER RANK.", 5, 2, 0, 3, false, 0.0, 0, [Wisdom]),
    node!(Fortune, "FORTUNE", "+25% GOLD AND MORE POTIONS PER RANK.", 5, 3, 0, 3, false, 0.0, 0, [Swiftness]),
    // ---- storm: the right path is about shocks and shields
    node!(Conductor, "CONDUCTOR", "WET OR BLEEDING FOES TAKE +30% MORE LIGHTNING DAMAGE PER RANK.", 0, 1, 2, 3, false, 0.0, 0, [Lightning]),
    node!(Overload, "OVERLOAD", "12% CHANCE PER RANK THAT A STORM SPELL COSTS NO MANA AND HAS NO COOLDOWN.", 0, 2, 2, 3, false, 0.0, 0, [Conductor]),
    node!(StaticField, "STATIC FIELD", "LIGHTNING STUNS FOR 50% LONGER PER RANK.", 0, 3, 0, 3, false, 0.0, 0, [BallLightning]),
    node!(StormShield, "STORM SHIELD", "FOR 8 SECONDS, ANYTHING THAT HITS YOU IS STRUCK BY LIGHTNING.", 0, 3, 2, 1, true, 30.0, 720, [Overload]),
    node!(EyeOfStorm, "EYE OF THE STORM", "THUNDERSTORM LASTS TWICE AS LONG AND STRIKES MORE OFTEN.", 0, 5, 1, 1, false, 0.0, 0, [Thunderstorm]),
    // ---- flame
    node!(Searing, "SEARING", "BURNING FOES TAKE +15% DAMAGE FROM EVERYTHING PER RANK.", 1, 1, 2, 3, false, 0.0, 0, [Fireball]),
    node!(Splinter, "SPLINTER", "FIREBALLS BURST INTO THREE MORE EXPLOSIONS.", 1, 2, 2, 1, false, 0.0, 0, [Searing]),
    node!(Fireproof, "FIREPROOF", "YOU TAKE 30% LESS BURN DAMAGE PER RANK.", 1, 3, 0, 3, false, 0.0, 0, [FireWall]),
    node!(Combustion, "COMBUSTION", "EVERY BURNING ENEMY NEAR YOU EXPLODES.", 1, 3, 2, 1, true, 30.0, 240, [Splinter]),
    node!(MeteorShower, "METEOR SHOWER", "THREE MORE METEORS FALL AROUND THE FIRST.", 1, 5, 1, 1, false, 0.0, 0, [Meteor]),
    // ---- venom
    node!(Caustic, "CAUSTIC", "ACID ON THE GROUND HURTS FOES 40% MORE PER RANK.", 2, 1, 2, 3, false, 0.0, 0, [Acid]),
    node!(PoisonCloud, "POISON CLOUD", "A CHOKING CLOUD AT THE CURSOR THAT HURTS AND SLOWS FOES FOR 7 SECONDS.", 2, 2, 2, 1, true, 24.0, 300, [Caustic]),
    node!(AcidPools, "ACID POOLS", "ACID SHOT LEAVES BIGGER PUDDLES PER RANK.", 2, 3, 0, 3, false, 0.0, 0, [AcidRain]),
    node!(RotBurst, "ROT BURST", "SEVERED LIMBS AND REMAINS NEAR THE CURSOR BURST IN ACID.", 2, 3, 2, 1, true, 20.0, 150, [PoisonCloud]),
    node!(Plague, "PLAGUE", "ACID RAIN AND ACID FLOOD ARE FAR BIGGER.", 2, 5, 1, 1, false, 0.0, 0, [AcidFlood]),
    // ---- frost
    node!(Chill, "CHILL", "YOUR HITS SLOW FOES FOR A SECOND MORE PER RANK.", 3, 1, 2, 3, false, 0.0, 0, [Nova]),
    node!(IceArmor, "ICE ARMOUR", "FOR 10 SECONDS TAKE 40% LESS DAMAGE; ATTACKERS ARE SLOWED.", 3, 2, 2, 1, true, 26.0, 900, [Chill]),
    node!(Shatterstorm, "SHATTERSTORM", "SHATTERED FOES SPRAY ICE SHARDS THAT HURT FOES NEARBY.", 3, 3, 0, 3, false, 0.0, 0, [IceSpikes]),
    node!(DeepFreeze, "DEEP FREEZE", "FROZEN FOES TAKE +20% DAMAGE PER RANK.", 3, 3, 2, 3, false, 0.0, 0, [IceArmor]),
    node!(AbsoluteZero, "ABSOLUTE ZERO", "WHEN A BLIZZARD ENDS, EVERYTHING IN IT FREEZES SOLID.", 3, 5, 1, 1, false, 0.0, 0, [Blizzard]),
    // ---- blade
    node!(Cleave, "CLEAVE", "YOUR SWINGS REACH 15% WIDER PER RANK.", 4, 1, 2, 3, false, 0.0, 0, [Edge]),
    node!(Dismember, "DISMEMBER", "YOUR CUTS GO 20% DEEPER PER RANK: LIMBS COME OFF FAR MORE OFTEN.", 4, 2, 2, 3, false, 0.0, 0, [Cleave]),
    node!(Momentum, "MOMENTUM", "+6% ATTACK SPEED PER RANK.", 4, 3, 2, 3, false, 0.0, 0, [Dismember]),
    node!(BladeFury, "BLADE FURY", "FOR 4 SECONDS YOU ARE A WHIRLWIND OF STEEL. YOU CAN STILL MOVE.", 4, 4, 1, 1, true, 40.0, 720, [Executioner, Momentum]),
    node!(Warlord, "WARLORD", "CRITICAL HITS KILL OUTRIGHT ANY FOE BELOW 30% HEALTH.", 4, 5, 1, 1, false, 0.0, 0, [BladeFury]),
    // ---- body
    node!(Toughness, "TOUGHNESS", "TAKE 4% LESS DAMAGE PER RANK.", 5, 1, 2, 5, false, 0.0, 0, [Vigor]),
    node!(SecondWind, "SECOND WIND", "POTIONS HEAL 25% MORE PER RANK.", 5, 2, 2, 3, false, 0.0, 0, [Toughness]),
    node!(Regeneration, "REGENERATION", "REGAIN 1 HEALTH A SECOND PER RANK.", 5, 3, 2, 3, false, 0.0, 0, [SecondWind]),
    node!(LastStand, "LAST STAND", "A KILLING BLOW LEAVES YOU ON 1 HEALTH AND UNTOUCHABLE FOR 2 SECONDS. ONCE EVERY 2 MINUTES.", 5, 4, 1, 1, false, 0.0, 0, [Fortune, Regeneration]),
    node!(Titan, "TITAN", "+20% MAXIMUM HEALTH, AND YOU NEVER SLIP.", 5, 5, 1, 1, false, 0.0, 0, [LastStand]),
];

pub fn node(s: Sk) -> &'static Node {
    &NODES[s as usize]
}

/// XP needed to go from `level` to the next
pub fn xp_need(level: i32) -> i32 {
    50 + level * 45 + level * level * 6
}

#[derive(Clone)]
pub struct Skills {
    pub rank: [u8; N],
    pub level: i32,
    pub xp: i32,
    pub points: i32,
    pub tomes: [bool; N],
}

impl Skills {
    pub fn new() -> Skills {
        // you begin with nothing but a sword
        Skills { rank: [0u8; N], level: 1, xp: 0, points: 0, tomes: [false; N] }
    }
    #[inline]
    pub fn r(&self, s: Sk) -> i32 {
        self.rank[s as usize] as i32
    }
    /// points spent in a school
    pub fn spent(&self, col: usize) -> i32 {
        NODES.iter().enumerate().filter(|(_, n)| n.col == col).map(|(i, _)| self.rank[i] as i32).sum()
    }
    /// can this node take another point right now? Returns the reason if not.
    pub fn can_learn(&self, i: usize) -> Result<(), String> {
        let n = &NODES[i];
        if self.rank[i] >= n.max {
            return Err("MAXED".into());
        }
        if !n.req.is_empty() && !n.req.iter().any(|s| self.r(*s) > 0) {
            let names: Vec<&str> = n.req.iter().map(|s| node(*s).name).collect();
            return Err(format!("NEEDS {}", names.join(" OR ")));
        }
        if self.level < min_level(i) {
            return Err(format!("NEEDS LEVEL {}", min_level(i)));
        }
        if needs_tome(i) && !self.tomes[i] {
            return Err("FIND ITS TOME IN THE WORLD FIRST".into());
        }
        if self.points <= 0 {
            return Err("NO SKILL POINTS".into());
        }
        Ok(())
    }
    pub fn learn(&mut self, i: usize) -> bool {
        if self.can_learn(i).is_ok() {
            self.rank[i] += 1;
            self.points -= 1;
            true
        } else {
            false
        }
    }
    /// the spell bar: every learned active skill, school by school
    pub fn bar(&self) -> Vec<Sk> {
        let mut v: Vec<&Node> = NODES.iter().filter(|n| n.active && self.rank[n.sk as usize] > 0).collect();
        v.sort_by_key(|n| (n.col, n.row, n.lane));
        v.into_iter().map(|n| n.sk).collect()
    }
}
