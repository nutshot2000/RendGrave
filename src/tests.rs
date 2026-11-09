// Regression tests for bugs found in review: saves, rewards, damage rules, loot odds.
// Run with `cargo test --release`.
use crate::areagen::AreaId;
use crate::body::Kind;
use crate::game::*;
use crate::skills::*;

fn set_rank(g: &mut Game, s: Sk, r: u8) {
    let i = NODES.iter().position(|n| n.sk == s).unwrap();
    g.player.sk.rank[i] = r;
}

/// a game in the crypt's boss floor with one enemy of `k` next to the player
fn arena(k: Kind) -> (Game, usize) {
    let mut g = Game::new(7);
    g.saving = false;
    g.travel(AreaId::Crypt(3));
    g.enemies.clear();
    let (x, y) = (g.player.x + 20.0, g.player.y);
    let e = g.scaled_enemy(k, x, y);
    g.enemies.push(e);
    (g, 0)
}

#[test]
fn truncated_saves_are_rejected() {
    assert!(!Game::save_ok("seed 1\nlevel 12\n"));
    assert!(!Game::save_ok("version 2\nseed 1\nlevel 12\ngold 5\n"));
    assert!(Game::save_ok("version 2\nseed 1\nlevel 12\nend\n"));
    // an old-format save with all its key lines still loads
    assert!(Game::save_ok("seed 1\narea ashford\nlevel 3\nxp 10\npoints 0\nranks 0,0\ngold 5\nplayed 9\n"));
}

#[test]
fn save_round_trip_keeps_everything() {
    let mut g = Game::new(3);
    g.saving = true;
    g.player.gold = 1234;
    g.player.kills = 77;
    g.player.equip = [None, None, None, None];
    set_rank(&mut g, Sk::Searing, 2);
    let tome = NODES.iter().position(|n| n.sk == Sk::Meteor).unwrap();
    g.player.sk.tomes[tome] = true;
    g.player.dead = true;
    g.save();
    let mut h = Game::blank(9);
    assert_eq!(h.load(), crate::areas::Load::Dead);
    assert_eq!(h.player.gold, 1234);
    assert_eq!(h.player.kills, 77);
    assert!(h.player.equip.iter().all(|e| e.is_none()), "empty equipment must load empty");
    assert_eq!(h.player.sk.r(Sk::Searing), 2);
    assert!(h.player.sk.tomes[tome]);
    // loading while dead costs the same tenth of your gold as waking normally
    let s = Game::start(true);
    assert_eq!(s.player.gold, 1234 * 9 / 10);
    let _ = std::fs::remove_file(crate::savefile::path(crate::areas::SAVE_FILE));
    let _ = std::fs::remove_file(crate::savefile::path("rendgrave_save.txt.bak"));
}

#[test]
fn searing_counts_for_explosions_and_everything() {
    let (mut g, i) = arena(Kind::Zombie);
    g.enemies[i].burning = 10;
    let base = g.status_mult(i);
    set_rank(&mut g, Sk::Searing, 3);
    assert!((g.status_mult(i) / base - 1.45).abs() < 0.01);
}

#[test]
fn deep_freeze_counts_for_boss_shatter() {
    let hit = |deep: u8| {
        let (mut g, i) = arena(Kind::King);
        set_rank(&mut g, Sk::DeepFreeze, deep);
        g.enemies[i].frozen = 20;
        let before = g.enemies[i].hp;
        assert!(g.shatter(i));
        before - g.enemies[i].hp
    };
    let (a, b) = (hit(0), hit(3));
    assert!((b / a - 1.6).abs() < 0.02, "deep freeze 0: {}  3: {}", a, b);
}

#[test]
fn immune_boss_takes_the_blow_normally() {
    let (mut g, i) = arena(Kind::King);
    g.enemies[i].frozen = 20;
    g.enemies[i].thaw = 100; // immune
    assert!(!g.shatter(i), "an immune boss must not swallow the hit");
    assert_eq!(g.enemies[i].frozen, 0);
}

#[test]
fn frozen_champions_crack_instead_of_dying() {
    let (mut g, i) = arena(Kind::Skeleton);
    g.enemies[i].elite = true;
    g.enemies[i].frozen = 100;
    g.shatter(i);
    assert!(g.enemies[i].alive, "a healthy champion survives one shatter");
    assert!(g.enemies[i].hp < g.enemies[i].maxhp * 0.75);
}

#[test]
fn whirlwind_scales_with_your_weapon() {
    let spin = |wd: f32| {
        let (mut g, i) = arena(Kind::Brute);
        g.player.gear.weapon_dmg = wd;
        g.enemies[i].x = g.player.x + 10.0;
        g.player.whirl = 26;
        let before = g.enemies[i].hp;
        g.update_whirl();
        before - g.enemies[i].hp
    };
    let (a, b) = (spin(12.0), spin(100.0));
    assert!(b > a * 4.0, "weapon 12: {}  weapon 100: {}", a, b);
}

#[test]
fn legendaries_stay_rare() {
    crate::rng::seed(11);
    let share = |luck: f32| {
        let n = 20000;
        let l = (0..n).filter(|_| crate::items::roll(8, luck).rarity == crate::items::Rarity::Legendary).count();
        l as f32 / n as f32
    };
    assert!(share(0.3) < 0.07, "iron chest legendary share {}", share(0.3));
    assert!(share(0.5) < 0.10, "boss legendary share {}", share(0.5));
    assert!(share(0.0) < 0.02);
}

#[test]
fn boss_rewards_are_yours_at_once() {
    let (mut g, i) = arena(Kind::King);
    let gold = g.player.gold;
    let bag = g.player.bag.len();
    let tomes = g.player.sk.tomes.iter().filter(|t| **t).count();
    g.enemies[i].hp = 0.0;
    g.enemies[i].alive = false;
    g.on_kill(i);
    assert!(g.player.gold > gold + 200, "boss gold goes straight to the purse");
    assert!(g.player.bag.len() >= bag + 2, "boss items go straight into the bag");
    assert!(g.player.bag[bag..].iter().any(|it| it.rarity >= crate::items::Rarity::Rare));
    assert_eq!(g.player.sk.tomes.iter().filter(|t| **t).count(), tomes + 1);
    assert!(g.portal.is_some());
}

#[test]
fn potions_are_capped() {
    let mut g = Game::new(5);
    g.saving = false;
    g.player.potions = crate::shop::MAX_POTIONS;
    let (x, y) = (g.player.x, g.player.y);
    g.pickups.push(Pickup { k: 1, x, y, z: 0.0, vz: 0.0, amt: 1, t: 100 });
    for _ in 0..5 {
        g.update_misc_for_test();
    }
    assert_eq!(g.player.potions, crate::shop::MAX_POTIONS);
}

#[test]
fn king_weak_points_break_and_change_the_fight() {
    use crate::kingparts::*;
    let (mut g, i) = arena(Kind::King);
    let maxhp = g.enemies[i].maxhp;
    let metal_legs = |g: &Game| [crate::body::T_LEGF, crate::body::T_LEGB].iter().map(|&p| g.enemies[i].body.parts[p].px.iter().filter(|q| q.m == crate::body::PM_METAL).count()).sum::<usize>();
    assert!(metal_legs(&g) > 20, "the King has knee plates");
    // knees: a hit low drains the knee pool and strips plates as it goes
    g.boss_part_hit(i, -10.0, maxhp * 0.035);
    assert!(g.enemies[i].wk[KNEES] > 0.3 && g.enemies[i].wk[KNEES] < 0.7);
    g.boss_part_hit(i, -10.0, maxhp * 0.05);
    assert_eq!(g.enemies[i].wk[KNEES], 0.0);
    assert_eq!(metal_legs(&g), 0, "every knee plate is gone");
    // chest: the core is bared and he takes more damage
    let before = g.status_mult(i);
    g.boss_part_hit(i, -50.0, maxhp * 0.2);
    assert!((g.status_mult(i) / before - 1.4).abs() < 0.01);
    // arm: it comes off, and so does his sword swing
    g.boss_part_hit(i, -30.0, maxhp * 0.2);
    assert!(!g.enemies[i].body.parts[crate::body::T_ARMF].on);
    assert!(g.gibs.len() > 0);
    // a hit on the head breaks nothing
    let wk = g.enemies[i].wk;
    g.boss_part_hit(i, -70.0, maxhp);
    assert_eq!(g.enemies[i].wk, wk);
}

#[test]
fn one_armed_king_calls_archers() {
    let (mut g, i) = arena(Kind::King);
    g.enemies[i].wk[crate::kingparts::ARM] = 0.0;
    g.enemies[i].hp = g.enemies[i].maxhp * 0.4;
    g.boss_logic(i);
    let archers = g.enemies.iter().filter(|e| e.k == Kind::Archer).count();
    let skels = g.enemies.iter().filter(|e| e.k == Kind::Skeleton).count();
    assert!(archers >= 3 && skels == 0, "archers {} skeletons {}", archers, skels);
}

#[test]
fn shields_block_from_the_front_only() {
    let (mut g, i) = arena(Kind::Skeleton);
    g.enemies[i].shield = 50.0;
    g.enemies[i].shield_max = 50.0;
    // the player stands to its left; it faces left: blocked
    g.enemies[i].x = g.player.x + 20.0;
    g.enemies[i].sface = -1.0;
    let hp = g.enemies[i].hp;
    assert!(g.shield_block(i, 10.0, false));
    assert_eq!(g.enemies[i].hp, hp);
    assert!(g.enemies[i].shield < 50.0);
    // facing away: the blow gets through
    g.enemies[i].sface = 1.0;
    assert!(!g.shield_block(i, 10.0, false));
    // a hammer breaks it fast
    g.enemies[i].sface = -1.0;
    g.shield_block(i, 30.0, true);
    assert_eq!(g.enemies[i].shield, 0.0);
}

#[test]
fn a_dropped_crossbow_shoots_through_a_line() {
    let (mut g, i) = arena(Kind::Crossbow);
    let (px, py) = (g.player.x, g.player.y);
    g.enemies[i].x = px + 10.0;
    g.enemies[i].face = 1.0;
    g.on_sever(i, &[crate::body::PK::ArmF]);
    assert_eq!(g.loose_bows.len(), 1);
    // two zombies in line beyond it
    for k in 0..2 {
        let mut z = g.scaled_enemy(Kind::Zombie, g.loose_bows[0].x + 40.0 + k as f32 * 20.0, g.loose_bows[0].y);
        z.stun = 1000;
        g.enemies.push(z);
    }
    g.enemies[i].alive = false;
    let before: Vec<f32> = g.enemies.iter().map(|e| e.hp).collect();
    let b = g.loose_bows[0];
    g.player.x = b.x - 15.0;
    g.player.y = b.y;
    g.hit_bows(b.x - 15.0, b.y, 0.0, 30.0, 1.2);
    assert!(g.loose_bows.is_empty());
    let hurt = (1..g.enemies.len()).filter(|&k| g.enemies[k].hp < before[k] || !g.enemies[k].alive).count();
    assert_eq!(hurt, 2, "the bolt goes through both");
    let _ = py;
}

#[test]
fn a_cut_bloater_leaks_acid() {
    let (mut g, i) = arena(Kind::Bloater);
    g.enemies[i].hp = 1000.0;
    g.hurt(i, 1.0, [255, 255, 255], false);
    assert!(g.enemies[i].leak > 0);
    let (x, y) = (g.enemies[i].x as i32, g.enemies[i].y as i32);
    for _ in 0..12 {
        g.frame += 1;
        g.anatomy_tick(i);
    }
    let acid = (-4..=4).flat_map(|dx| (-2..=3).map(move |dy| (dx, dy))).filter(|(dx, dy)| g.world.at(x + dx, y + dy) == crate::world::ACID).count();
    assert!(acid > 0, "acid on the floor under it");
}

#[test]
fn burning_enemies_run_at_their_friends() {
    let (mut g, i) = arena(Kind::Bandit);
    let (x, y) = (g.enemies[i].x, g.enemies[i].y);
    let friend = g.scaled_enemy(Kind::Bandit, x + 60.0, y);
    g.enemies.push(friend);
    g.enemies[i].burning = 40;
    let w = g.panic_want(i, 1.0).expect("it panics");
    assert!(w.0 > 0.5, "runs toward its friend");
    g.enemies[i].x = x + 55.0;
    g.panic_want(i, 1.0);
    let lit = g.enemies[1].body.parts.iter().flat_map(|p| p.px.iter()).filter(|p| p.m != 0 && p.burn > 0).count();
    assert!(lit > 0, "and sets it alight");
}
