# Rendgrave

A pixel-simulated action RPG in Rust (wgpu). Every character is built from layered pixels
(armour or cloth over flesh over bone), so swords cut limbs off, acid melts them away,
fire chars and burns, lightning scorches and arcs through puddles, and frost shatters.

## Release

    python tools/package.py   # builds dist/Rendgrave-EA-<version>-windows.zip and dist/press/ for itch.io

The store page text is in docs/itch_page.md, the readme that ships with the game in docs/README_PLAYERS.txt.

## Run

    cargo run --release          # title screen: continue or start a new game
    cargo run --release -- --new # fresh character

## The world

| Area | Level | What's there |
|---|---|---|
| Ashford | 1 | The town: waypoint, campfire, houses, the merchant |
| The Hollow Pines | 3 | Big forest, a river with a bridge, an abandoned logging camp, the crypt mausoleum |
| Sunken Crypt 1-3 | 5-7 | Dungeon floors; the Hollow King waits at the bottom |
| The Old Graveyard | 8 | Ruined chapel, a dozen tombs, the plague pit |
| The Blackwater Fen | 10 | Bog pools and boardwalks, peat that smoulders, swamp gas that explodes, bloaters |
| Ravenhold | 12 | A walled bandit castle: tents, barracks, the broken keep |
| Ravenhold Cellars 1-2 | 13-14 | Storerooms full of bandits; the Bandit Chief guards his hoard |

Chests (E) spill gold, potions and gear; iron-bound ones are richer. Shrines (E) give a
one-minute blessing: Power, Haste, Warding or Mending.

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Left mouse | Attack with your weapon (aim at the legs to take them off) |
| Right mouse | Cast the selected spell |
| 1-9 / mouse wheel | Choose a spell from the spell bar |
| K or Tab | Skill tree (spend skill points) |
| Space | Dodge roll |
| F | Drink a potion |
| E | Use exits, stairs, waypoints, chests and shrines |
| I | Gear: equip with a click, sell with a right click |
| M / N | Music on/off, sound effects on/off |
| - / + | Volume |
| R | After dying: wake in Ashford |
| F11 | Fullscreen |
| Esc | Pause (or close the skill tree). Close the window to quit |

## Test modes

    cargo run --release -- --showcase shots   # scripted fight, saves screenshots to shots/
    cargo run --release -- --lab lab          # enemy lineup, one effect each, close-up screenshots
    cargo run --release -- --spells spl       # every spell unlocked and cast, plus the skill tree screen
    cargo run --release -- --tour tour        # visits and screenshots every area
    cargo run --release -- --sfx-dump sfx     # writes every synthesised sound effect to WAV
    cargo run --release -- --loot loot        # every weapon type on a lineup, loot, the gear screen
    cargo run --release -- --maps maps        # overview picture of every area's layout
    cargo run --release -- --menus menus      # title screen, pause menu, end-of-act card
    cargo run --release -- --extras extras    # secret walls, urns, champion types, grave robber, kill streaks
    add --press to any picture mode           # hides hints and banners, for store screenshots
    cargo run --release -- --navtest          # do hidden enemies find their way to you? (with and without pathfinding)
    cargo run --release -- --bosstest         # a bot fights both bosses with five builds: time to kill, damage taken

## Code map

- `world.rs`   the material grid: liquids, fire, acid, peat and swamp gas
- `areagen.rs` the area generators (towns, forests, fen, castle, dungeons)
- `props.rs`   chests and shrines
- `barrels.rs` powder kegs, oil and acid barrels, blasts
- `nav.rs`     enemy pathfinding (a flow field from the player)
- `body.rs`    layered pixel bodies, cutting, burning, acid, splitting into pieces
- `game.rs`    player, enemies and AI, spells, projectiles, gibs, particles
- `render.rs`  CPU painting of albedo / emission / light info with depth sorting
- `hud.rs`     orbs, spell bar, text, banners
- `gfx.rs` + `shader.wgsl`  GPU lighting with shadows, bloom, tonemapping, upscale
