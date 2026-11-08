def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/items.rs', [
    ("""    let rarity = if r < 0.02 + ilvl as f32 * 0.002 {
        Rarity::Legendary
    } else if r < 0.13 + ilvl as f32 * 0.005 {
        Rarity::Rare
    } else if r < 0.45 {""", """    let rarity = if r < 0.006 + ilvl as f32 * 0.0008 {
        Rarity::Legendary
    } else if r < 0.07 + ilvl as f32 * 0.004 {
        Rarity::Rare
    } else if r < 0.40 {"""),
    ("""    pub fn value(&self) -> i32 {""", """    /// a rough measure of how good an item is, for "is this an upgrade?"
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
    pub fn value(&self) -> i32 {"""),
])

edit('src/gear.rs', [
    ("""    pub fn drop_loot(&mut self, x: f32, y: f32, k: Kind, elite: bool) {
        let (chance, rolls, luck) = match k {
            Kind::King => (1.0, 2, 0.35),
            Kind::Chief => (1.0, 3, 0.4),
            Kind::Bandit | Kind::Crossbow => (0.14, 1, 0.02),
            Kind::Bloater => (0.12, 1, 0.0),
            Kind::Brute => (0.55, 1, 0.05),
            Kind::Skeleton | Kind::Archer => (0.11, 1, 0.0),
            _ => (0.09, 1, 0.0),
        };
        let (chance, rolls, luck) = if elite { ((chance * 3.0f32).min(1.0), rolls + 1, luck + 0.15) } else { (chance, rolls, luck) };
        for r in 0..rolls {
            if rnd() < chance {
                let item = roll(self.alvl, luck);""", """    /// drops are rare: most of the dead carry nothing, champions and brutes often do,
    /// bosses always do. Plain (common) items stop dropping once you are past the start
    /// and become a little gold instead, so what you do find is worth a look
    pub fn drop_loot(&mut self, x: f32, y: f32, k: Kind, elite: bool) {
        let (chance, rolls, luck) = loot_odds(k, elite);
        for r in 0..rolls {
            if rnd() < chance {
                let item = roll(self.alvl, luck);
                if item.rarity == Rarity::Common && self.player.sk.level >= 4 {
                    self.pickups.push(crate::game::Pickup { k: 0, x: x + rr(-6.0, 6.0), y: y + rr(-3.0, 3.0), z: 10.0, vz: 1.8, amt: item.value() * 2, t: 0 });
                    continue;
                }"""),
    ("""    pub fn update_ground(&mut self) {""", """    /// is this better than what you're wearing in its slot?
    pub fn is_upgrade(&self, it: &Item) -> bool {
        let s = SLOTS.iter().position(|x| *x == it.slot).unwrap_or(0);
        match &self.player.equip[s] {
            Some(cur) => it.score() > cur.score() * 1.05,
            None => true,
        }
    }

    pub fn update_ground(&mut self) {"""),
    ("""            let gi = self.ground.swap_remove(k);
            let it = gi.item;
            self.float(px, py - 62.0, it.name.clone(), it.rarity.col(), it.rarity >= Rarity::Rare);""", """            let gi = self.ground.swap_remove(k);
            let it = gi.item;
            self.float(px, py - 62.0, it.name.clone(), it.rarity.col(), it.rarity >= Rarity::Rare);
            if self.is_upgrade(&it) {
                self.float(px, py - 74.0, "UPGRADE - PRESS I".into(), [120, 255, 120], true);
                self.snd(Sfx::Chime, px, py, 0.6);
            }"""),
])
s = open('src/gear.rs', encoding='utf8').read()
s += """
/// (chance to drop, how many tries, rarity luck) for a kill
pub fn loot_odds(k: Kind, elite: bool) -> (f32, i32, f32) {
    let (chance, rolls, luck) = match k {
        Kind::King => (1.0, 2, 0.45),
        Kind::Chief => (1.0, 3, 0.5),
        Kind::Brute => (0.25, 1, 0.12),
        Kind::Bandit | Kind::Crossbow => (0.035, 1, 0.02),
        Kind::Bloater | Kind::Skeleton | Kind::Archer => (0.03, 1, 0.0),
        _ => (0.025, 1, 0.0),
    };
    if elite {
        ((chance * 8.0f32).min(0.7), rolls, luck + 0.2)
    } else {
        (chance, rolls, luck)
    }
}
"""
open('src/gear.rs', 'w', encoding='utf8').write(s)

edit('src/props.rs', [
    ("""        let items = if tier > 0 { 2 + (rnd() < 0.5) as i32 } else { 1 + (rnd() < 0.35) as i32 };
        let luck = if tier > 0 { 0.25 } else { 0.08 };""", """        // plain chests are mostly gold; iron ones always hold something good
        let items = if tier > 0 { 1 + (rnd() < 0.35) as i32 } else { (rnd() < 0.4) as i32 };
        let luck = if tier > 0 { 0.3 } else { 0.1 };"""),
])

edit('src/hud.rs', [
    ("""        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
        }
    }
    let (bx, by, _) = inv_cell(1, BAG_SIZE - 1, fw, fh);""", """        if let Some(it) = it {
            item_icon(u, x + 3, y + 3, it);
            if g.is_upgrade(it) {
                // a green arrow: better than what you're wearing
                for k in 0..4 {
                    u.rect(x + s - 6 - k, y + 2 + k, 1 + k * 2, 1, [90, 240, 90], 255);
                }
                u.rect(x + s - 7, y + 6, 3, 3, [90, 240, 90], 255);
            }
        }
    }
    let (bx, by, _) = inv_cell(1, BAG_SIZE - 1, fw, fh);"""),
])

edit('src/mapdump.rs', [
    ("""        println!("{:<12} {}x{}  {:>4.0} ms  enemies {:>3}  chests {}  shrines {}  exits {}", id.key(), w.aw, w.ah, ms, sp.enemies.len(), sp.chests.len(), sp.shrines.len(), sp.exits.len());""",
     """        // expected items worth picking up (magic or better) from clearing the whole area
        let elite_share = if id == AreaId::Ashford { 0.0 } else { 0.07 };
        let mut items = 0.0;
        for e in &sp.enemies {
            for (el, share) in [(false, 1.0 - elite_share), (true, elite_share)] {
                let (c, r, luck) = crate::gear::loot_odds(e.0, el);
                items += share * c * r as f32 * (0.40 + luck).min(1.0);
            }
        }
        for c in &sp.chests {
            items += if c.2 > 0 { 1.35 } else { 0.4 * 0.5 };
        }
        println!("{:<12} {}x{}  {:>4.0} ms  enemies {:>3}  chests {}  shrines {}  exits {}  items ~{:.1}", id.key(), w.aw, w.ah, ms, sp.enemies.len(), sp.chests.len(), sp.shrines.len(), sp.exits.len(), items);"""),
])
print("loot rarity ok")
