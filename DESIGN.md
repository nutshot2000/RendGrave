# Rendgrave - Game Design

A dark action RPG where every body, wall and puddle is simulated pixel by pixel.
You start as a nobody with a sword. By the end you call down meteors on armies.

## Pillars
1. **Damage you can see.** Every hit changes a body or the world: limbs come off, flesh
   melts to bone, armour dents, walls crumble, forests burn.
2. **The world is a weapon.** Puddles conduct, oil burns, ice shatters, gas explodes.
   Smart players fight with the environment more than with numbers.
3. **Slow-burn power.** Early game is a tense sword fight. Each new tool is earned and
   changes how you play. Late game is spectacle.
4. **Breadth.** Many biomes, many enemies, each one reacting differently to fire,
   acid, lightning, ice and steel.

## Structure
Four acts. Each act: a hub town, 3-4 outdoor areas, 2-3 dungeons, an act boss.
Areas have a fixed **area level**; enemies scale with the area, not with you.

| Act | Areas | Area levels | Theme |
|---|---|---|---|
| I - The Hollow Pines | Ashford (town), Hollow Pines, Old Graveyard, Mirefen Edge; dungeons: Sunken Crypt, Bandit Cellars | 1-12 | Undead rising in a dark forest |
| II - Mirefen | Bogwater (town), Mirefen, Rotwood, Leech Fens; dungeons: Drowned Temple, Hive Warrens | 12-25 | Swamp: peat fires, gas, disease |
| III - Frostmere | Hearthold (town), Frostmere Lakes, Glacier Pass, Whitepeak; dungeons: Ice Caves, Frozen Keep | 25-40 | Ice, snowstorms, things frozen for centuries |
| IV - The Cinder Wastes | The Last Fort (town), Ashen Plains, Lava Fields, The Rend; dungeons: Forge of Bones, The Grave Below | 40-60 | Lava, fire, the source of the dead |

- Outdoor areas are generated from biome rules each new game (seeded), with hand-placed
  landmarks (towns, crypt entrances, boss arenas).
- Dungeons regenerate on each visit: rooms, corridors, traps, floors 1-N, boss at the bottom.
- Waypoints link towns and areas. Death returns you to the last town, keeping gear,
  dropping some gold.

## Pacing and power curve
Level cap 60. Roughly 30-40 hours for a first playthrough.

| Level | What you have | How fights feel |
|---|---|---|
| 1-4 | Sword, dodge, potions | Careful melee. Two zombies are a threat. Learn to aim at legs. |
| 5-9 | First spell school opens (one weak spell each: Spark, Firebolt, Acid Spit, Chill) | Spells are a tool, not a solution. Mana is scarce. |
| 10-17 | Tier-2 spells and the first strong passives; first weapon types beyond swords | Builds start to form. |
| 18-29 | Tier-3 spells (Ball Lightning, Wall of Fire, Acid Rain, Ice Spikes, Whirlwind) | Crowd control and combos. Environment combos matter. |
| 30-44 | Tier-4 ultimates (Thunderstorm, Meteor, Acid Flood, Blizzard), found as **tomes** from act bosses and deep dungeons, then learned with points | Spectacle. Big armies. |
| 45-60 | Mastery passives, legendary gear, endgame dungeons | Optimising builds. |

Rules:
- One skill point per level, plus bonus points from act bosses (one each).
- Tier gates by level: tier 1 at 1, tier 2 at 5, tier 3 at 18, tier 4 at 30 (needs its tome).
- Starting mana is low (60). Mana grows with Wisdom and gear, not automatically.
- XP from enemies scales with area level, and drops sharply for enemies far below your level.
- No respawn treadmill in normal areas: an area repopulates when you leave and return.

## Skill schools
Storm, Flame, Venom, Frost, Blade, Body (existing six) plus later schools unlocked in
Acts II-III: **Blood** (life drain, bone spears from corpses) and **Earth** (raise walls,
quakes that crack the ground). Each school: 6-8 nodes over 4 tiers, a capstone.

## Enemies
Every enemy is a pixel body with its own materials, so each reacts differently to damage.
Families and roles, with how they interact with the simulation:

### Act I - Hollow Pines
| Enemy | Role | Notes |
|---|---|---|
| Shambler (zombie) | Fodder melee | Loses limbs, keeps crawling. Burns well. |
| Crawler | Fast low melee | Legless from the start, hard to hit high. |
| Skeleton warrior | Melee | Bone only: immune to bleeding, shatters when frozen. |
| Bone archer | Ranged | Keeps distance; arrows stick in walls and bodies. |
| Grave hound | Fast pack | Packs of 3-5 that flank. |
| Ghoul | Corpse eater | Eats dead bodies on the ground to heal and grow. |
| Bandit | Human melee | Bleeds, flees at low health, drops gold. |
| Bandit crossbowman | Ranged | Takes cover behind walls. |
| Brute | Armoured heavy | Break armour plates first. |
| Gravedigger (mini-boss) | Summoner | Raises zombies from the ground. |
| **Boss: The Hollow King** | Act boss | Giant skeleton in rusted armour; pieces of armour fall off in phases, crown resurrects it once. |

### Act II - Mirefen
| Enemy | Role | Notes |
|---|---|---|
| Bloater | Suicide | Bursts into acid when killed. Kill it next to its friends. |
| Bog leech | Swarm | Hides in water, latches on, drains life. |
| Mire witch | Caster | Hexes, summons swamp gas clouds. Lightning ignites them. |
| Peat golem | Tank | Made of peat: smoulders for a long time when lit. |
| Rotwood treant | Heavy | A walking tree. Very flammable. |
| Fen spider | Ambusher | Webs that slow you; webs burn. |
| Plague rat swarm | Swarm | Dozens of tiny bodies. Fire and nova clear them. |
| Drowned knight | Elite melee | Waterlogged: fire weak, lightning strong. |
| Hive drone / Hive queen | Swarm + boss | Hive Warrens dungeon. |
| **Boss: Mother Mire** | Act boss | Huge bog creature that floods the arena with acid and spawns leeches. |

### Act III - Frostmere
| Enemy | Role | Notes |
|---|---|---|
| Frost wraith | Caster | Freezes you; immune to frost, shattered by fire+ice. |
| Ice troll | Regenerating tank | Severed limbs regrow unless burned. |
| Snow wolf | Fast pack | Hides in snowstorms. |
| Frozen dead | Fodder | Thaw near fire into normal zombies. |
| Yeti | Heavy | Throws ice boulders that leave ice on the ground. |
| Ice archer | Ranged | Shots freeze puddles. |
| Crystal golem | Armour puzzle | Only shatters with blunt damage or after heating then freezing. |
| Glacier wyrm | Burrower | Moves under ice, bursts up. |
| **Boss: The Pale Warden** | Act boss | Fights on a frozen lake; the ice breaks under both of you. |

### Act IV - Cinder Wastes
| Enemy | Role | Notes |
|---|---|---|
| Ash zombie | Fodder | Explodes into embers on death. |
| Magma hound | Fast | Leaves fire trails. Water and ice turn it to stone. |
| Fire imp | Caster swarm | Throws fireballs, immune to fire. |
| Obsidian knight | Elite | Fire-immune armour that cracks when frozen. |
| Bone colossus | Siege | Huge, made of hundreds of bones; you take it apart piece by piece. |
| Cinder priest | Support | Resurrects nearby dead unless corpses are destroyed. |
| Lava serpent | Hazard | Swims in lava rivers. |
| **Boss: The Rend** | Final boss | The thing raising the dead. Multi-phase, the arena itself changes material. |

### Elites and affixes
Any enemy can spawn as an elite with 1-3 affixes: Burning (fire aura), Armoured,
Vampiric, Splitting (splits in two when cut), Shielded, Hasted, Explosive, Frozen aura.

## Loot (Stage 2)
- Weapon types that cut differently: sword (slice), axe (deep chop, splits bodies),
  hammer (crush, shatters bone and ice), spear (pierces in a line), daggers (fast, bleed).
- Rarities: common, magic (1-2 affixes), rare (3-4), legendary (unique mechanic).
- Affixes: of Burning, Thundering, Corroding, Freezing, of the Leech, Keen, Heavy...
- Armour shows on your pixel body and is dented and stripped by hits.

## Balance targets
- Act I zombie dies to 3-4 sword hits; brute to 12-15.
- Early player dies to ~8 zombie hits, so groups of 3+ are dangerous.
- A spell should cost roughly 1/4 to 1/2 of your mana pool when you first get it.
- Fights against same-level packs last 10-20 seconds; bosses 2-4 minutes.
