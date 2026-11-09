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

#[test]
fn splintering_maul_freezes_those_behind() {
    let (mut g, i) = arena(Kind::Zombie);
    g.player.equip[0] = Some(crate::items::legendary(crate::items::Legend::SplinterMaul, 5));
    let (px, py) = (g.player.x, g.player.y);
    g.enemies[i].x = px + 20.0;
    g.enemies[i].y = py;
    let mut behind = g.scaled_enemy(Kind::Zombie, px + 60.0, py);
    behind.hp = 10000.0;
    g.enemies.push(behind);
    g.enemies[i].frozen = 50;
    assert!(g.shatter(i));
    assert!(g.enemies[1].frozen > 0, "the one behind is frozen by the shards");
}

#[test]
fn ember_ring_turns_fire_into_mana() {
    let mut g = Game::new(4);
    g.saving = false;
    g.player.equip[3] = Some(crate::items::legendary(crate::items::Legend::EmberRing, 5));
    g.player.mana = 0.0;
    let (px, py) = (g.player.x as i32, g.player.y as i32);
    for dx in -10..10 {
        g.world.set(px + dx, py + 6, crate::world::FIRE);
    }
    g.frame = 10;
    g.ember_ring();
    assert!(g.player.mana > 0.5);
}

#[test]
fn bosses_give_a_unique_you_lack() {
    let (mut g, i) = arena(Kind::King);
    g.enemies[i].hp = 0.0;
    g.enemies[i].alive = false;
    g.on_kill(i);
    let n = g.player.bag.iter().filter(|it| it.legend.map(|l| crate::items::UNIQUES.contains(&l)).unwrap_or(false)).count();
    assert!(n >= 1, "at least the one the boss gives");
    // and it survives a save
    let it = g.player.bag.iter().find(|it| it.legend.is_some() && crate::items::UNIQUES.contains(&it.legend.unwrap())).unwrap();
    let back = crate::items::Item::decode(&it.encode()).unwrap();
    assert_eq!(back.legend, it.legend);
    assert_eq!(back.slot, it.slot);
}

#[test]
fn starting_gifts() {
    let mut g = Game::new(8);
    g.saving = false;
    g.take_gift(1);
    assert!(g.player.sk.bar().contains(&Sk::Lightning), "spark: lightning on the bar");
    let mut g = Game::new(8);
    g.saving = false;
    g.take_gift(2);
    assert_eq!(g.flasks, crate::agency::FLASKS);
    let mut g = Game::new(8);
    g.saving = false;
    g.take_gift(3);
    let w = g.player.equip[0].as_ref().unwrap();
    assert!(matches!(w.wk, crate::items::WKind::Axe | crate::items::WKind::Spear));
}

#[test]
fn a_thrown_flask_spills_oil() {
    let mut g = Game::new(8);
    g.saving = false;
    g.take_gift(2);
    let (x, y) = (g.player.x, g.player.y);
    g.flask_break(x + 30.0, y);
    let oil = (-8..=8).flat_map(|dx| (-6..=6).map(move |dy| (dx, dy))).filter(|(dx, dy)| g.world.at(x as i32 + 30 + dx, y as i32 + dy) == crate::world::OIL).count();
    assert!(oil > 10);
}

#[test]
fn respec_gives_every_point_back() {
    let mut g = Game::new(8);
    g.saving = false;
    g.player.sk.level = 6;
    g.player.gold = 1000;
    set_rank(&mut g, Sk::Edge, 3);
    set_rank(&mut g, Sk::Vigor, 2);
    let pts = g.player.sk.points;
    g.offer_respec();
    let mut inp = Input::default();
    inp.sel = Some(0);
    g.update_choice(&inp);
    assert_eq!(g.player.sk.points, pts + 5);
    assert_eq!(g.player.gold, 1000 - 300);
    assert!(g.player.sk.rank.iter().all(|r| *r == 0));
}

#[test]
fn ravenhold_bridge_is_built() {
    for seed in [1u64, 2, 3] {
        let (w, sp) = crate::areagen::generate(AreaId::Ravenhold, seed);
        let (bx, by) = sp.setpiece.expect("the bridge set piece");
        let (bx, by) = (bx as i32, by as i32);
        let near = |m: u8| (-30..30).flat_map(|dx| (-70..70).map(move |dy| (dx, dy))).filter(|(dx, dy)| w.at(bx + dx, by + dy) == m).count();
        assert!(near(crate::world::OIL) > 500, "oil in the ditch");
        assert!(near(crate::world::WOOD) > 200, "a barricade");
        assert!(w.at(bx, by) == crate::world::E, "the bridge is walkable");
    }
}

#[test]
fn quiet_places_are_built() {
    for seed in [1u64, 2, 3, 4] {
        for (a, k) in [(AreaId::Pines, 0u8), (AreaId::Fen, 1), (AreaId::Graveyard, 2)] {
            let (_, sp) = crate::areagen::generate(a, seed);
            assert!(sp.lore.iter().any(|l| l.2 == k), "{:?} seed {} has its quiet place", a, seed);
        }
    }
}

#[test]
fn freeing_tomas_brings_him_home() {
    let mut g = Game::new(6);
    g.saving = false;
    g.travel(AreaId::Graveyard);
    let (x, y, _) = *g.lore.iter().find(|l| l.2 == crate::quiet::PEN).unwrap();
    assert!(g.npcs.iter().any(|n| n.k == crate::quiet::TOMAS), "Tomas is in his pen");
    g.player.x = x + 4.0;
    g.player.y = y;
    let gold = g.player.gold;
    assert!(g.update_lore(true));
    assert!(g.player.gold > gold);
    assert!(!g.npcs.iter().any(|n| n.k == crate::quiet::TOMAS), "he runs for home");
    g.king_dead = true;
    g.chief_dead = true;
    g.travel(AreaId::Ashford);
    assert!(g.npcs.iter().any(|n| n.k == crate::quiet::TOMAS), "and is by the well in Ashford");
    assert!(g.npcs.len() >= 8, "more people about after both bosses");
}

/// one step of the game driven by a pretend controller
fn pad_step(g: &mut Game, pad: &mut crate::pad::Pad, held: &str, ls: (f32, f32), rs: (f32, f32)) -> Input {
    pad.fake(held, ls, rs);
    let mut inp = Input::default();
    pad.apply(g, &mut inp);
    let aim = g.mouse_world(&inp);
    g.update(&inp);
    // the tests read the aim where it pointed when it was made
    inp.mouse = (aim.0 - g.cam.0.floor(), aim.1 - g.cam.1.floor());
    inp
}

#[test]
fn gamepad_auto_aims_and_attacks() {
    let (mut g, i) = arena(Kind::Zombie);
    g.view = (480.0, 270.0);
    for _ in 0..5 {
        g.update(&Input::default());
    }
    let mut pad = crate::pad::Pad::off();
    // facing the zombie with the right stick at rest: the cursor sits on it
    let inp = pad_step(&mut g, &mut pad, "", (0.9, 0.0), (0.0, 0.0));
    let (mx, my) = g.mouse_world(&inp);
    let e = &g.enemies[i];
    assert!((mx - e.x).abs() < 4.0 && (my - (e.y - 22.0)).abs() < 4.0, "auto-aim missed: {:?} vs {:?}", (mx, my), (e.x, e.y));
    assert!(inp.auto && inp.stick.0 > 0.8);
    // A swings the weapon
    let inp = pad_step(&mut g, &mut pad, "a", (0.0, 0.0), (0.0, 0.0));
    assert!(inp.lmb && !inp.interact);
    // the right stick aims by hand, straight down
    let inp = pad_step(&mut g, &mut pad, "", (0.0, 0.0), (0.0, 1.0));
    let (mx, my) = g.mouse_world(&inp);
    assert!(!inp.auto && (mx - g.player.x).abs() < 1.0 && my > g.player.y);
}

#[test]
fn gamepad_casts_dodges_and_uses() {
    let (mut g, _) = arena(Kind::Zombie);
    g.view = (480.0, 270.0);
    set_rank(&mut g, Sk::Lightning, 1);
    set_rank(&mut g, Sk::Fireball, 1);
    let mut pad = crate::pad::Pad::off();
    let inp = pad_step(&mut g, &mut pad, "y", (0.0, 0.0), (0.0, 0.0));
    assert_eq!(inp.sel, Some(1));
    assert!(inp.rmb);
    let inp = pad_step(&mut g, &mut pad, "rb lb rt", (0.0, 0.0), (0.0, 0.0));
    assert!(inp.dodge && inp.potion && inp.throw);
    // with something to use in reach, A uses it instead of swinging
    pad_step(&mut g, &mut pad, "", (0.0, 0.0), (0.0, 0.0));
    g.prompt = Some("E  OPEN CHEST".into());
    let mut inp = Input::default();
    pad.fake("a", (0.0, 0.0), (0.0, 0.0));
    pad.apply(&g, &mut inp);
    assert!(inp.interact && !inp.lmb);
}

#[test]
fn gamepad_a_that_opens_a_talk_does_not_skip_it() {
    let mut g = Game::new(5);
    g.saving = false;
    g.view = (480.0, 270.0);
    let mut pad = crate::pad::Pad::off();
    pad_step(&mut g, &mut pad, "", (0.0, 0.0), (0.0, 0.0));
    g.talk = Some((vec!["ONE", "TWO", "THREE"], 0));
    // A still held from opening it: nothing turns
    for _ in 0..3 {
        pad_step(&mut g, &mut pad, "a", (0.0, 0.0), (0.0, 0.0));
    }
    assert_eq!(g.talk.as_ref().unwrap().1, 0);
    // let go and press again: the next page
    pad_step(&mut g, &mut pad, "", (0.0, 0.0), (0.0, 0.0));
    pad_step(&mut g, &mut pad, "a", (0.0, 0.0), (0.0, 0.0));
    assert_eq!(g.talk.as_ref().unwrap().1, 1);
    // in a menu the stick moves the pointer
    let before = pad.cursor;
    pad_step(&mut g, &mut pad, "", (1.0, 0.0), (0.0, 0.0));
    assert!(pad.cursor.0 > before.0 + 3.0);
}

#[test]
fn gamepad_button_names_replace_keys() {
    let mut pad = crate::pad::Pad::off();
    pad.fake("", (0.0, 0.0), (0.0, 0.0));
    assert_eq!(crate::pad::tr("E  OPEN CHEST"), "A  OPEN CHEST");
    assert_eq!(crate::pad::tr("CLICK TO EQUIP OR TAKE OFF   RIGHT CLICK TO SELL"), "A TO EQUIP OR TAKE OFF   X TO SELL");
    assert_eq!(crate::pad::tr("PRESS R TO WAKE IN ASHFORD"), "PRESS A TO WAKE IN ASHFORD");
    assert_eq!(crate::pad::tr("DESCEND THE CRYPT AND SLAY THE HOLLOW KING"), "DESCEND THE CRYPT AND SLAY THE HOLLOW KING");
    pad.release();
    assert_eq!(crate::pad::tr("E  OPEN CHEST"), "E  OPEN CHEST");
}
