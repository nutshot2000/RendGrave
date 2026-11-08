# Rendgrave

A pixel-simulated action RPG in Rust (wgpu). Every character is built from layered pixels
(armour or cloth over flesh over bone), so swords cut limbs off, acid melts them away,
fire chars and burns, lightning scorches and arcs through puddles, and frost shatters.

## Run

    cargo run --release          # continues your save
    cargo run --release -- --new # fresh character

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Left mouse | Sword (aim at the legs to take them off) |
| Right mouse | Cast the selected spell |
| 1-9 / mouse wheel | Choose a spell from the spell bar |
| K or Tab | Skill tree (spend skill points) |
| Space | Dodge roll |
| F | Drink a potion |
| E | Use exits, stairs and waypoints |
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

## Code map

- `world.rs`   the material grid: liquids, fire, acid, worldgen (forest + crypt)
- `body.rs`    layered pixel bodies, cutting, burning, acid, splitting into pieces
- `game.rs`    player, enemies and AI, spells, projectiles, gibs, particles
- `render.rs`  CPU painting of albedo / emission / light info with depth sorting
- `hud.rs`     orbs, spell bar, text, banners
- `gfx.rs` + `shader.wgsl`  GPU lighting with shadows, bloom, tonemapping, upscale
