# Boss rewards can't be lost; potion cap; legendary odds; sell safety; better upgrade score.
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


A = 'src/areas.rs'
edit(A, [
    # King: gold and tome straight to you
    ("""        for k in 0..6 {
            self.pickups.push(Pickup { k: 0, x: x + rr(-14.0, 14.0), y: y + rr(-8.0, 8.0), z: 14.0, vz: rr(1.5, 3.0), amt: 50 + ri(40) as i32, t: k * 6 });
        }
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        if !missing.is_empty() {
            let i = missing[ri(missing.len())];
            self.pickups.push(Pickup { k: 2, x, y: y + 6.0, z: 16.0, vz: 3.0, amt: i as i32, t: 0 });
        }""", """        let gold: i32 = (0..6).map(|_| 50 + ri(40) as i32).sum();
        self.boss_gold(x, y, gold);
        let missing: Vec<usize> = (0..N).filter(|&i| needs_tome(i) && !self.player.sk.tomes[i]).collect();
        if !missing.is_empty() {
            let i = missing[ri(missing.len())];
            self.player.sk.tomes[i] = true;
            self.toast = Some((format!("THE KING'S TOME: {} - LEARN IT AT LEVEL 30", NODES[i].name), 400));
        }"""),
    ("""        for k in 0..8 {
            self.pickups.push(Pickup { k: 0, x: x + rr(-16.0, 16.0), y: y + rr(-8.0, 8.0), z: 14.0, vz: rr(1.5, 3.0), amt: 60 + ri(50) as i32, t: k * 5 });
        }""", """        let gold: i32 = (0..8).map(|_| 60 + ri(50) as i32).sum();
        self.boss_gold(x, y, gold);"""),
    ("""    /// the boss is dead: a way home opens where it stood""", """    /// a boss's gold goes straight into your purse (it can't be lost by quitting), with a
    /// shower of coins for show
    fn boss_gold(&mut self, x: f32, y: f32, gold: i32) {
        self.player.gold += gold;
        self.float(x, y - 80.0, format!("+{} GOLD", gold), [255, 214, 90], true);
        self.snd(Sfx::Coin, x, y, 1.0);
        for _ in 0..50 {
            self.ptc(x + rr(-14.0, 14.0), y, rr(10.0, 40.0), rr(-1.5, 1.5), rr(-0.8, 0.8), rr(1.0, 3.0), 70, PtK::Spark, [255, 210, 80]);
        }
    }

    /// the boss is dead: a way home opens where it stood"""),
])

G = 'src/gear.rs'
edit(G, [
    ("""        let (chance, rolls, luck) = loot_odds(k, elite);
        for r in 0..rolls {
            if rnd() < chance {
                let item = roll(self.alvl, luck);""", """        let (chance, rolls, luck) = loot_odds(k, elite);
        let mut to_bag = 0;
        for r in 0..rolls {
            if rnd() < chance {
                let mut item = roll(self.alvl, luck);
                if k.boss() {
                    // a boss always gives at least one rare, and its hoard goes straight
                    // into your bag so it can't be lost by dying or quitting
                    for _ in 0..40 {
                        if r > 0 || item.rarity >= Rarity::Rare {
                            break;
                        }
                        item = roll(self.alvl, luck);
                    }
                    if self.player.bag.len() < BAG_SIZE {
                        if item.rarity == Rarity::Legendary {
                            self.banner = Some((item.name.clone(), "LEGENDARY - IN YOUR BAG".into(), 300));
                        }
                        self.player.bag.push(item);
                        to_bag += 1;
                        continue;
                    }
                }"""),
    ("""                self.ground.push(GroundItem { item, x: x + rr(-8.0, 8.0), y: y + rr(-4.0, 4.0), z: 12.0, vz: rr(1.8, 3.2), t: -(r as i32) * 8 });
            }
        }
    }""", """                self.ground.push(GroundItem { item, x: x + rr(-8.0, 8.0), y: y + rr(-4.0, 4.0), z: 12.0, vz: rr(1.8, 3.2), t: -(r as i32) * 8 });
            }
        }
        if to_bag > 0 {
            self.float(x, y - 96.0, format!("{} ITEM{} ADDED TO YOUR BAG", to_bag, if to_bag > 1 { "S" } else { "" }), [120, 220, 255], true);
        }
    }"""),
    # selling a rare or a legendary takes a second right-click
    ("""            Some((1, i)) if rclick && i < self.player.bag.len() => {
                let it = self.player.bag.remove(i);""", """            Some((1, i)) if rclick && i < self.player.bag.len() && self.player.bag[i].rarity >= Rarity::Rare && self.sell_confirm != Some(i) => {
                self.sell_confirm = Some(i);
                self.toast = Some((format!("RIGHT-CLICK AGAIN TO SELL {}", self.player.bag[i].name), 150));
            }
            Some((1, i)) if rclick && i < self.player.bag.len() => {
                self.sell_confirm = None;
                let it = self.player.bag.remove(i);"""),
    ("""    pub fn update_inventory(&mut self, inp: &Input) {
        self.inv_hover = crate::hud::inv_at(inp.mouse, self.view);""", """    pub fn update_inventory(&mut self, inp: &Input) {
        self.inv_hover = crate::hud::inv_at(inp.mouse, self.view);
        if self.inv_hover.map(|h| h.0) != Some(1) || self.inv_hover.map(|h| h.1) != self.sell_confirm {
            self.sell_confirm = None;
        }"""),
])
edit('src/game.rs', [
    ("    pub toast: Option<(String, i32)>,\n", "    pub toast: Option<(String, i32)>,\n    /// a rare item waiting for a second right-click to sell\n    pub sell_confirm: Option<usize>,\n"),
    ("            save_fail: std::cell::RefCell::new(None),\n", "            save_fail: std::cell::RefCell::new(None),\n            sell_confirm: None,\n"),
    # potions you find respect the cap; extras become gold
    ("""            } else if p.k == 1 {
                self.player.potions += 1;
                self.sounds.push(Snd { k: Sfx::Coin, x: px, y: py, v: 0.6 });
                self.float(px, py - 56.0, "+POTION".into(), [255, 110, 120], false);""", """            } else if p.k == 1 && self.player.potions >= crate::shop::MAX_POTIONS {
                let g = 15 + self.alvl * 3;
                self.player.gold += g;
                self.sounds.push(Snd { k: Sfx::Coin, x: px, y: py, v: 0.6 });
                self.float(px, py - 56.0, format!("POTIONS FULL  +{} GOLD", g), [255, 214, 90], false);
            } else if p.k == 1 {
                self.player.potions += 1;
                self.sounds.push(Snd { k: Sfx::Coin, x: px, y: py, v: 0.6 });
                self.float(px, py - 56.0, "+POTION".into(), [255, 110, 120], false);"""),
])
edit('src/story.rs', [("""            self.player.potions += 2;""", """            self.player.potions = (self.player.potions + 2).min(crate::shop::MAX_POTIONS);""")])

I = 'src/items.rs'
edit(I, [
    ("""    let r = rnd() - luck;
    let rarity = if r < 0.006 + ilvl as f32 * 0.0008 {
        Rarity::Legendary
    } else if r < 0.07 + ilvl as f32 * 0.004 {
        Rarity::Rare
    } else if r < 0.40 {
        Rarity::Magic""", """    // luck multiplies the odds of each tier rather than shifting the roll, so a lucky
    // source is a few times likelier to give a legendary, not dozens of times
    let leg = (0.006 + ilvl as f32 * 0.0008) * (1.0 + luck * 7.0);
    let rare = (0.07 + ilvl as f32 * 0.004) * (1.0 + luck * 4.0);
    let magic = (0.40 + luck).min(0.9);
    let r = rnd();
    let rarity = if r < leg {
        Rarity::Legendary
    } else if r < leg + rare {
        Rarity::Rare
    } else if r < magic.max(leg + rare + 0.05) {
        Rarity::Magic"""),
    # a weapon's score is its damage per second, not its base number
    ("""        let mut s = match self.slot {
            Slot::Weapon => self.base as f32,""", """        let mut s = match self.slot {
            Slot::Weapon => {
                let w = wstats(self.wk);
                self.base as f32 * w.dmg_mult * 16.0 / w.frames as f32
            }"""),
])
print("rewards ok")
