// --bosstest: a bot fights each boss at the level a player would normally be by then,
// with several builds, and reports how long the kill took and how much it got hurt.
// Used to tune the boss fights.
use crate::areagen::AreaId;
use crate::game::*;
use crate::items::*;
use crate::skills::*;

fn kit(g: &mut Game, lvl: i32, build: &str) {
    let p = &mut g.player;
    p.sk.level = lvl;
    p.sk.points = lvl - 1 + 1;
    p.sk.tomes = [true; N];
    let plan: Vec<Sk> = match build {
        "acid" => vec![Sk::Acid, Sk::Corrosion, Sk::Corrosion, Sk::Corrosion, Sk::Caustic, Sk::Caustic, Sk::AcidRain, Sk::PoisonCloud, Sk::AcidPools, Sk::Caustic, Sk::RotBurst],
        "fire" => vec![Sk::Fireball, Sk::Kindling, Sk::Kindling, Sk::Kindling, Sk::Searing, Sk::Searing, Sk::FireWall, Sk::Splinter, Sk::Searing, Sk::Combustion, Sk::Fireproof],
        "storm" => vec![Sk::Lightning, Sk::ChainMastery, Sk::ChainMastery, Sk::ChainMastery, Sk::Conductor, Sk::BallLightning, Sk::Conductor, Sk::Overload, Sk::StaticField, Sk::Overload],
        "frost" => vec![Sk::Nova, Sk::Brittle, Sk::Brittle, Sk::Chill, Sk::IceSpikes, Sk::Brittle, Sk::Chill, Sk::IceArmor, Sk::DeepFreeze, Sk::DeepFreeze, Sk::Shatterstorm],
        _ => vec![Sk::Edge, Sk::Edge, Sk::Edge, Sk::Cleave, Sk::Whirlwind, Sk::Bloodthirst, Sk::Dismember, Sk::Bloodthirst, Sk::Executioner, Sk::Momentum, Sk::Executioner],
    };
    for s in plan {
        p.sk.learn(s as usize);
    }
    // whatever is left goes into health
    while p.sk.learn(Sk::Vigor as usize) || p.sk.learn(Sk::Toughness as usize) {}
    // decent gear for the level: magic items of each slot
    let mut equip: [Option<Item>; 4] = [None, None, None, None];
    while equip.iter().any(|e| e.is_none()) {
        let it = roll(lvl, 0.15);
        let s = SLOTS.iter().position(|x| *x == it.slot).unwrap();
        if equip[s].is_none() && it.rarity >= Rarity::Magic && it.legend.is_none() {
            equip[s] = Some(it);
        }
    }
    g.player.equip = equip;
    g.apply_gear();
    g.recalc();
    g.player.hp = g.player.maxhp;
    g.player.mana = g.player.maxmana;
    g.player.potions = 6;
}

pub fn run() {
    crate::rng::seed(99);
    if std::env::var("BOSSPROBE").is_ok() {
        let mut g = Game::new(7);
        g.saving = false;
        g.travel(AreaId::Crypt(3));
        let b = g.enemies.iter().position(|e| e.k.boss()).unwrap();
        let boss = g.enemies.swap_remove(b);
        g.enemies.clear();
        g.enemies.push(boss);
        g.enemies[0].stun = 100000;
        let (bx, by) = (g.enemies[0].x, g.enemies[0].y);
        g.player.x = bx - 80.0;
        g.player.y = by;
        for k in 0..3 {
            let hp0 = g.enemies[0].hp;
            g.projs.push(Proj { k: 0, x: bx - 40.0, y: by + 1.0, z: 26.0, vx: 4.0, vy: 0.0, vz: 0.0, life: 90 });
            let mut hit = false;
            for _ in 0..40 {
                g.update(&Input::default());
                if g.projs.is_empty() && !hit {
                    hit = true;
                }
            }
            println!("fireball {}: boss hp {:.0} -> {:.0}  frozen {} burning {}", k, hp0, g.enemies[0].hp, g.enemies[0].frozen, g.enemies[0].burning);
        }
        return;
    }
    for (area, lvl) in [(AreaId::Crypt(3), 10), (AreaId::Cellars(2), 17)] {
        println!("{} at level {}", area.name(), lvl);
        for build in ["sword", "acid", "fire", "storm", "frost"] {
            let mut g = Game::new(7);
            g.saving = false;
            g.travel(area);
            let Some(b) = g.enemies.iter().position(|e| e.k.boss()) else { continue };
            let boss = g.enemies.swap_remove(b);
            g.enemies.clear();
            g.enemies.push(boss);
            g.barrels.clear();
            kit(&mut g, lvl, build);
            let (bx, by) = (g.enemies[0].x, g.enemies[0].y);
            g.player.x = bx - 110.0;
            g.player.y = by;
            g.enemies[0].aggro = true;
            let caster = build != "sword" && build != "frost";
            let (mut taken, mut deaths, mut potions) = (0.0f32, 0, 0);
            let mut t = 0;
            let maxhp = g.enemies[0].maxhp;
            while t < 60 * 180 {
                t += 1;
                let Some(bi) = g.enemies.iter().position(|e| e.alive && e.k.boss()) else { break };
                let (bx, by) = (g.enemies[bi].x, g.enemies[bi].y);
                let (px, py) = (g.player.x, g.player.y);
                let (dx, dy) = (bx - px, by - py);
                let d = (dx * dx + dy * dy).sqrt();
                let mut inp = Input::default();
                inp.mouse = (bx - g.cam.0.floor(), by - 24.0 - g.cam.1.floor());
                let (away, toward) = if caster { (d < 90.0, d > 170.0) } else { (false, d > 22.0) };
                let slam = g.enemies[bi].slam > 0;
                let flee = away || (slam && d < 100.0);
                if flee || toward {
                    let s = if flee { -1.0 } else { 1.0 };
                    inp.right = dx * s > 4.0;
                    inp.left = dx * s < -4.0;
                    inp.down = dy * s > 4.0;
                    inp.up = dy * s < -4.0;
                }
                if build == "frost" {
                    inp.lmb = true;
                }
                if build != "sword" {
                    // the strongest spell that's ready
                    let bar = g.player.sk.bar();
                    let burning = g.enemies[bi].burning > 0;
                    let pick = bar.iter().enumerate().rev().find(|(_, s)| {
                        let n = node(**s);
                        let useful = match **s {
                            Sk::Combustion => burning,
                            Sk::StormShield => g.player.shield == 0 && d < 120.0,
                            Sk::IceArmor => g.player.ice == 0 && d < 120.0,
                            Sk::Nova => d < 60.0,
                            Sk::RotBurst => !g.gibs.is_empty(),
                            _ => true,
                        };
                        useful && g.player.cd[**s as usize] == 0 && g.player.mana >= n.mana
                    });
                    if let Some((k, _)) = pick {
                        inp.sel = Some(k);
                        inp.rmb = t % 2 == 0;
                    }
                } else {
                    inp.lmb = true;
                }
                if g.player.hp < g.player.maxhp * 0.35 && g.player.potions > 0 {
                    inp.potion = t % 2 == 0;
                }
                if false {
                    println!("    f{} mana {:.1} frame {} hitstop {} dead {} paused {} cast_t {} cd {:?}", t, g.player.mana, g.frame, g.hitstop, g.player.dead, g.paused, g.player.cast_t, &g.player.cd[4..7]);
                }
                if false {
                    let e = &g.enemies[bi];
                    println!("    s{} boss {:.0} frozen {} thaw {} chill {} d {:.0} spikes {} hp {:.0}", t / 60, e.hp, e.frozen, e.thaw, e.chill, d, g.spikes.len(), g.player.hp);
                }
                if std::env::var("BOSSDBG").is_ok() && build == "fire" && t % 300 == 0 && lvl == 17 {
                    let pr = g.projs.first().map(|p| format!("k{} ({:.0},{:.0},{:.0}) v({:.1},{:.1},{:.2}) life {}", p.k, p.x, p.y, p.z, p.vx, p.vy, p.vz, p.life)).unwrap_or_default();
                    println!("    t{} boss {:.0} frame {} hitstop {} dead {} hp {:.0} slip {} whirl {} talk {} ending {} paused {} stun {} st {:?} proj {}", t / 60, g.enemies[bi].hp, g.frame, g.hitstop, g.player.dead, g.player.hp, g.player.slip, g.player.whirl, g.talk.is_some(), g.ending, g.paused, g.enemies[bi].stun, g.enemies[bi].slam, pr);
                }
                let hp0 = g.player.hp;
                let pot0 = g.player.potions;
                g.update(&inp);
                if g.player.potions < pot0 {
                    potions += 1;
                }
                if g.player.hp < hp0 {
                    taken += hp0 - g.player.hp;
                }
                if g.player.dead {
                    deaths += 1;
                    g.player.dead = false;
                    g.player.hp = g.player.maxhp;
                    g.banner = None;
                }
            }
            let left = g.enemies.iter().find(|e| e.k.boss()).map(|e| e.hp.max(0.0)).unwrap_or(0.0);
            let done = g.enemies.iter().all(|e| !(e.alive && e.k.boss()));
            println!(
                "  {:<6} {}  boss hp {:>5.0}  player hp {:>4.0}  taken {:>5.0}  deaths {}  potions {}",
                build,
                if done { format!("killed in {:>5.1}s", t as f32 / 60.0) } else { format!("NOT killed, {:>3.0}% left", left / maxhp * 100.0) },
                maxhp,
                g.player.maxhp,
                taken,
                deaths,
                potions
            );
        }
    }
}
