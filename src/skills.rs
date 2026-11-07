// The skill tree: six schools, four tiers each. Actives go on the spell bar,
// passives modify everything else. Each tier needs the one above it and a minimum level.

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
}

pub const N: usize = 24;

pub struct Node {
    pub sk: Sk,
    pub name: &'static str,
    pub desc: &'static str,
    pub col: usize,
    pub row: usize,
    pub max: u8,
    pub active: bool,
    pub mana: f32,
    pub cd: i32,
}

pub const SCHOOLS: [(&str, [u8; 3]); 6] = [
    ("STORM", [150, 190, 255]),
    ("FLAME", [255, 150, 60]),
    ("VENOM", [140, 230, 70]),
    ("FROST", [170, 225, 255]),
    ("BLADE", [230, 225, 215]),
    ("BODY", [240, 110, 120]),
];

/// minimum character level for each tier
pub const ROW_LEVEL: [i32; 4] = [1, 2, 4, 6];

macro_rules! node {
    ($sk:ident, $name:expr, $desc:expr, $c:expr, $r:expr, $max:expr, $act:expr, $mana:expr, $cd:expr) => {
        Node { sk: Sk::$sk, name: $name, desc: $desc, col: $c, row: $r, max: $max, active: $act, mana: $mana, cd: $cd }
    };
}

pub const NODES: [Node; N] = [
    node!(Lightning, "LIGHTNING", "A BOLT THAT CHAINS BETWEEN FOES AND RUNS THROUGH WATER AND BLOOD.", 0, 0, 1, true, 14.0, 22),
    node!(ChainMastery, "CHAIN MASTERY", "+1 CHAIN JUMP AND +15% LIGHTNING DAMAGE PER RANK.", 0, 1, 3, false, 0.0, 0),
    node!(BallLightning, "BALL LIGHTNING", "A SLOW ORB THAT ZAPS EVERY ENEMY IT DRIFTS PAST.", 0, 2, 1, true, 24.0, 70),
    node!(Thunderstorm, "THUNDERSTORM", "FOR 6 SECONDS THE SKY STRIKES ENEMIES AROUND YOU.", 0, 3, 1, true, 45.0, 600),
    node!(Fireball, "FIREBALL", "EXPLODES, SETS FLESH, CLOTH, GRASS AND OIL ALIGHT.", 1, 0, 1, true, 20.0, 36),
    node!(Kindling, "KINDLING", "BIGGER FIREBALLS AND +30% BURN DAMAGE PER RANK.", 1, 1, 3, false, 0.0, 0),
    node!(FireWall, "WALL OF FIRE", "A LINE OF FLAME TOWARDS THE CURSOR THAT BURNS FOR SECONDS.", 1, 2, 1, true, 26.0, 120),
    node!(Meteor, "METEOR", "AFTER A MOMENT A METEOR CRASHES DOWN AT THE CURSOR.", 1, 3, 1, true, 50.0, 420),
    node!(Acid, "ACID SHOT", "LOBS GLOBS OF ACID THAT EAT FLESH TO THE BONE.", 2, 0, 1, true, 10.0, 16),
    node!(Corrosion, "CORROSION", "+1 GLOB AND STRONGER ACID PER RANK.", 2, 1, 3, false, 0.0, 0),
    node!(AcidRain, "ACID RAIN", "A CLOUD THAT RAINS ACID ON AN AREA FOR 5 SECONDS.", 2, 2, 1, true, 30.0, 300),
    node!(AcidFlood, "ACID FLOOD", "A WAVE OF ACID BURSTS OUT AND SPREADS ACROSS THE GROUND.", 2, 3, 1, true, 45.0, 420),
    node!(Nova, "FROST NOVA", "FREEZES EVERYTHING AROUND YOU. FROZEN FOES SHATTER.", 3, 0, 1, true, 28.0, 80),
    node!(Brittle, "BRITTLE", "FROZEN LONGER AND A WIDER NOVA PER RANK.", 3, 1, 3, false, 0.0, 0),
    node!(IceSpikes, "ICE SPIKES", "SPIKES TEAR UP FROM THE GROUND IN A LINE AND TAKE LEGS OFF.", 3, 2, 1, true, 24.0, 90),
    node!(Blizzard, "BLIZZARD", "A STORM OF ICE AT THE CURSOR THAT CHILLS AND FREEZES.", 3, 3, 1, true, 45.0, 480),
    node!(Edge, "KEEN EDGE", "SWORD CUTS DEEPER AND HITS 10% HARDER PER RANK.", 4, 0, 3, false, 0.0, 0),
    node!(Whirlwind, "WHIRLWIND", "SPIN WITH YOUR BLADE AND CUT EVERYTHING AROUND YOU.", 4, 1, 1, true, 18.0, 90),
    node!(Bloodthirst, "BLOODTHIRST", "HEAL A LITTLE FOR EVERY CUT THAT DRAWS BLOOD.", 4, 2, 3, false, 0.0, 0),
    node!(Executioner, "EXECUTIONER", "+8% CRITICAL CHANCE PER RANK. CRITS CUT FAR DEEPER.", 4, 3, 3, false, 0.0, 0),
    node!(Vigor, "VIGOR", "+15 MAXIMUM HEALTH PER RANK.", 5, 0, 5, false, 0.0, 0),
    node!(Wisdom, "WISDOM", "+15 MAXIMUM MANA AND FASTER MANA REGEN PER RANK.", 5, 1, 5, false, 0.0, 0),
    node!(Swiftness, "SWIFTNESS", "+7% MOVE SPEED AND QUICKER DODGES PER RANK.", 5, 2, 3, false, 0.0, 0),
    node!(Fortune, "FORTUNE", "+25% GOLD AND MORE POTIONS PER RANK.", 5, 3, 3, false, 0.0, 0),
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
}

impl Skills {
    pub fn new() -> Skills {
        let mut rank = [0u8; N];
        for s in [Sk::Lightning, Sk::Fireball, Sk::Acid, Sk::Nova] {
            rank[s as usize] = 1;
        }
        Skills { rank, level: 1, xp: 0, points: 1 }
    }
    #[inline]
    pub fn r(&self, s: Sk) -> i32 {
        self.rank[s as usize] as i32
    }
    /// can this node take another point right now? Returns the reason if not.
    pub fn can_learn(&self, i: usize) -> Result<(), &'static str> {
        let n = &NODES[i];
        if self.rank[i] >= n.max {
            return Err("MAXED");
        }
        if self.points <= 0 {
            return Err("NO SKILL POINTS");
        }
        if self.level < ROW_LEVEL[n.row] {
            return Err(["", "NEEDS LEVEL 2", "NEEDS LEVEL 4", "NEEDS LEVEL 6"][n.row]);
        }
        if n.row > 0 {
            let above = NODES.iter().position(|m| m.col == n.col && m.row == n.row - 1).unwrap();
            if self.rank[above] == 0 {
                return Err("LEARN THE SKILL ABOVE FIRST");
            }
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
    /// the spell bar: every learned active skill, in tree order
    pub fn bar(&self) -> Vec<Sk> {
        NODES.iter().filter(|n| n.active && self.rank[n.sk as usize] > 0).map(|n| n.sk).collect()
    }
}
