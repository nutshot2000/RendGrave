# Rendgrave

A pixel-simulated action RPG in Rust (wgpu). Every character is built from layered pixels
(armour or cloth over flesh over bone), so swords cut limbs off, acid melts them away,
fire chars and burns, lightning scorches and arcs through puddles, and frost shatters.

## Run

    cargo run --release

## Controls

| Key | Action |
|---|---|
| WASD | Move |
| Left mouse | Sword (aim at the legs to take them off) |
| Right mouse | Cast the selected spell |
| 1-4 | Lightning, Fireball, Acid shot, Frost nova |
| Space | Dodge roll |
| F | Drink a potion |
| R | New game after death or victory |
| F11 | Fullscreen |
| Esc | Quit |

## Test modes

    cargo run --release -- --showcase shots   # scripted fight, saves screenshots to shots/
    cargo run --release -- --lab lab          # enemy lineup, one effect each, close-up screenshots

## Code map

- `world.rs`   the material grid: liquids, fire, acid, worldgen (forest + crypt)
- `body.rs`    layered pixel bodies, cutting, burning, acid, splitting into pieces
- `game.rs`    player, enemies and AI, spells, projectiles, gibs, particles
- `render.rs`  CPU painting of albedo / emission / light info with depth sorting
- `hud.rs`     orbs, spell bar, text, banners
- `gfx.rs` + `shader.wgsl`  GPU lighting with shadows, bloom, tonemapping, upscale
