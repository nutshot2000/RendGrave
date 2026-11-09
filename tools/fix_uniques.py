# Four uniques that change how you fight (slice step 3).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


ALL = "[Legend::Stormcaller, Legend::Gravecleaver, Legend::MiresKiss, Legend::WintersGrudge, Legend::Pyre]"
ALL9 = "crate::items::ALL_LEGENDS"
I = 'src/items.rs'
s = open(I, encoding='utf8').read()
assert s.count(ALL) == 2
s = s.replace("let l = " + ALL + "[ri(5)];", "let l = ALL_LEGENDS[ri(ALL_LEGENDS.len())];")
s = s.replace(ALL + ".get(li as usize).copied()", "ALL_LEGENDS.get(li as usize).copied()")
open(I, 'w', encoding='utf8').write(s)
edit(I, [
    ("""    /// spear: thrusts breathe fire ahead
    Pyre,
}""", """    /// spear: thrusts breathe fire ahead
    Pyre,
    /// hammer: shattering a frozen foe drives ice into those behind it
    SplinterMaul,
    /// armour: you never slip, and hit harder standing in oil
    Mudwalker,
    /// sword: hits arc through severed metal armour lying on the ground
    Conduit,
    /// ring: burning ground near you restores mana; your own burns last longer
    EmberRing,
}

/// every legendary, in save order (only ever add at the end)
pub const ALL_LEGENDS: [Legend; 9] = [
    Legend::Stormcaller,
    Legend::Gravecleaver,
    Legend::MiresKiss,
    Legend::WintersGrudge,
    Legend::Pyre,
    Legend::SplinterMaul,
    Legend::Mudwalker,
    Legend::Conduit,
    Legend::EmberRing,
];
/// the uniques that turn the world into a weapon, given by the bosses
pub const UNIQUES: [Legend; 4] = [Legend::SplinterMaul, Legend::Mudwalker, Legend::Conduit, Legend::EmberRing];"""),
    ("""        Legend::Pyre => (WKind::Spear, "PYRE", vec![Affix::Damage(20), Affix::Burning]),
    };
    Item { slot: Slot::Weapon, wk, rarity: Rarity::Legendary, ilvl, base: weapon_base(wk, ilvl) + 6, affixes: aff, legend: Some(l), name: name.into() }""", """        Legend::Pyre => (WKind::Spear, "PYRE", vec![Affix::Damage(20), Affix::Burning]),
        Legend::SplinterMaul => (WKind::Hammer, "THE SPLINTERING MAUL", vec![Affix::Damage(20), Affix::Keen(10)]),
        Legend::Conduit => (WKind::Sword, "THE CONDUIT", vec![Affix::Damage(15), Affix::Crit(8)]),
        Legend::Mudwalker => {
            // light leathers: less armour than they look
            let base = ((3.0 + ilvl as f32) * 0.6) as i32;
            return Item { slot: Slot::Armor, wk: WKind::Sword, rarity: Rarity::Legendary, ilvl, base, affixes: vec![Affix::Speed(8), Affix::Regen(2)], legend: Some(l), name: "MUDWALKER'S LEATHERS".into() };
        }
        Legend::EmberRing => {
            return Item { slot: Slot::Ring, wk: WKind::Sword, rarity: Rarity::Legendary, ilvl, base: 0, affixes: vec![Affix::Wisdom(8)], legend: Some(l), name: "THE EMBER RING".into() };
        }
    };
    Item { slot: Slot::Weapon, wk, rarity: Rarity::Legendary, ilvl, base: weapon_base(wk, ilvl) + 6, affixes: aff, legend: Some(l), name: name.into() }"""),
    ("""            Legend::Pyre => "THRUSTS BREATHE FIRE",""", """            Legend::Pyre => "THRUSTS BREATHE FIRE",
            Legend::SplinterMaul => "SHATTERS DRIVE ICE INTO FOES BEHIND",
            Legend::Mudwalker => "NEVER SLIP - +25% DAMAGE STANDING IN OIL",
            Legend::Conduit => "HITS ARC THROUGH METAL ARMOUR ON THE GROUND",
            Legend::EmberRing => "FIRE NEAR YOU GIVES MANA - BURNS LAST LONGER","""),
])

GE = 'src/gear.rs'
edit(GE, [
    ("""        self.player.equip[0].as_ref().and_then(|w| w.legend) == Some(l)""", """        self.player.equip.iter().any(|it| it.as_ref().and_then(|w| w.legend) == Some(l))"""),
    ("""    pub(crate) fn on_hit(&mut self, i: usize, dmg: f32, h: f32) {
        let g = self.player.gear;""", """    pub(crate) fn on_hit(&mut self, i: usize, dmg: f32, h: f32) {
        if self.has_legend(Legend::Conduit) {
            self.conduit_arc(i, dmg);
        }
        let g = self.player.gear;"""),
])

G = 'src/game.rs'
edit(G, [
    ("    fn under(&self, x: f32, y: f32) -> [bool; 6] {", "    pub(crate) fn under(&self, x: f32, y: f32) -> [bool; 6] {"),
    ("""        let u = self.under(self.player.x, self.player.y);
        let hm = self.haste_mult()""", """        let u = self.under(self.player.x, self.player.y);
        let mud = self.has_legend(Legend::Mudwalker);
        let hm = self.haste_mult()"""),
    ("p.dodge == 0 && p.slip == 0 && p.sk.r(Sk::Titan) == 0 && rnd() < 0.005 {", "p.dodge == 0 && p.slip == 0 && p.sk.r(Sk::Titan) == 0 && !mud && rnd() < 0.005 {"),
    ("""        if p.burn > 0 {
            p.burn -= 1;""", """        if p.burn > 0 && !(self.frame % 3 == 0 && self.player.gear_ember) {
            let p = &mut self.player;
            p.burn -= 1;"""),
])
# shatter with the Splintering Maul
s = open(G, encoding='utf8').read()
a = """        self.float(ex, ey - 60.0, "SHATTERED".into(), [180, 220, 255], true);"""
assert s.count(a) == 1
s = s.replace(a, a + """
        if self.has_legend(Legend::SplinterMaul) {
            self.splinter(ex, ey);
        }""")
open(G, 'w', encoding='utf8').write(s)

# the uniques' effects
u = '''// The four uniques that turn the simulation into a weapon (see docs/SLICE_PLAN.md step 3).
use crate::body::*;
use crate::game::*;
use crate::items::*;
use crate::rng::*;
use crate::world::*;

impl Game {
    /// the Splintering Maul: a shattered foe's ice is driven on into those behind it
    pub fn splinter(&mut self, x: f32, y: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let (dx, dy) = (x - px, y - py);
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        let (ux, uy) = (dx / l, dy / l);
        let dmg = 30.0 * (1.0 + 0.1 * self.alvl as f32);
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if !e.alive {
                continue;
            }
            let (ex, ey) = (e.x - x, e.y - y);
            let along = ex * ux + ey * uy;
            let side = (ex * uy - ey * ux).abs();
            if along > 2.0 && along < 100.0 && side < 22.0 {
                self.hurt(i, dmg, [180, 225, 255], false);
                let e = &mut self.enemies[i];
                if e.alive && !e.k.boss() && e.frozen == 0 {
                    // frozen in turn: the next blow shatters them too
                    e.frozen = 90;
                    e.body.tint_all(|p| p.burn = 0);
                }
            }
        }
        for _ in 0..40 {
            let s = rr(0.0, 1.0);
            self.ptc(x + ux * s * 20.0, y + uy * s * 10.0, rr(5.0, 40.0), ux * rr(2.0, 5.0) + rr(-0.5, 0.5), uy * rr(1.0, 3.0) + rr(-0.4, 0.4), rr(0.0, 1.5), 40, PtK::Ice, [210, 240, 255]);
        }
    }

    /// the Conduit: every hit arcs from the target to severed metal armour on the ground,
    /// and from each piece on to the nearest other foe
    pub fn conduit_arc(&mut self, i: usize, dmg: f32) {
        let (tx, ty) = (self.enemies[i].x, self.enemies[i].y);
        let mut pieces: Vec<(f32, f32, f32)> = self
            .gibs
            .iter()
            .filter(|g| g.part.px.iter().filter(|p| p.m == PM_METAL).count() >= 3)
            .filter(|g| ((g.x - tx).powi(2) + (g.y - ty).powi(2)).sqrt() < 110.0)
            .map(|g| (g.x, g.y, g.z))
            .collect();
        pieces.truncate(3);
        for (gx, gy, gz) in pieces {
            self.add_bolt((tx, ty - 30.0), (gx, gy - gz));
            let mut best = None;
            let mut bd = 90.0;
            for (j, e) in self.enemies.iter().enumerate() {
                if j == i || !e.alive {
                    continue;
                }
                let d = ((e.x - gx).powi(2) + (e.y - gy).powi(2)).sqrt();
                if d < bd {
                    bd = d;
                    best = Some(j);
                }
            }
            if let Some(j) = best {
                let (ex, ey) = (self.enemies[j].x, self.enemies[j].y);
                self.add_bolt((gx, gy - gz), (ex, ey - 30.0));
                self.zap_enemy(j, dmg * 0.6);
            }
            self.light(gx, gy, 60.0, 1.0, [0.6, 0.8, 1.0], 8);
        }
    }

    /// the Ember Ring: fire near you pours mana into you
    pub fn ember_ring(&mut self) {
        self.player.gear_ember = self.has_legend(Legend::EmberRing);
        if !self.player.gear_ember || self.frame % 10 != 0 || self.player.dead {
            return;
        }
        let (px, py) = (self.player.x as i32, self.player.y as i32);
        let mut fires = vec![];
        for dy in -24..=24 {
            for dx in -40..=40 {
                if (dx + dy) % 2 == 0 && self.world.at(px + dx, py + dy) == FIRE {
                    fires.push((px + dx, py + dy));
                }
            }
        }
        if fires.is_empty() {
            return;
        }
        let gain = (fires.len().min(30) as f32) * 0.1;
        let p = &mut self.player;
        p.mana = (p.mana + gain).min(p.maxmana);
        for _ in 0..3 {
            let (fx, fy) = fires[ri(fires.len())];
            let (dx, dy) = (px - fx, py - fy);
            self.ptc(fx as f32, fy as f32, 4.0, dx as f32 * 0.04, dy as f32 * 0.04, 0.4, 25, PtK::Ember, [120, 160, 255]);
        }
    }

    /// a boss gives one of the four uniques you don't have yet
    pub fn give_unique(&mut self) {
        let have = |g: &Game, l: Legend| g.player.bag.iter().chain(g.player.equip.iter().flatten()).any(|it| it.legend == Some(l));
        let missing: Vec<Legend> = UNIQUES.iter().copied().filter(|&l| !have(self, l)).collect();
        if missing.is_empty() || self.player.bag.len() >= crate::gear::BAG_SIZE {
            return;
        }
        let it = legendary(missing[ri(missing.len())], self.alvl);
        self.banner = Some((it.name.clone(), "A UNIQUE - IN YOUR BAG. PRESS I".into(), 320));
        self.player.bag.push(it);
    }
}

/// standing in oil with the Mudwalker's leathers
pub fn mud_bonus(g: &Game) -> f32 {
    if g.has_legend(Legend::Mudwalker) && g.under(g.player.x, g.player.y)[3] { 1.25 } else { 1.0 }
}
'''
open('src/uniques.rs', 'w', encoding='utf8').write(u)
edit('src/main.rs', [("mod areas;\n", "mod areas;\nmod uniques;\n")])
edit(G, [
    ("    pub stand_cd: i32,\n", "    pub stand_cd: i32,\n    /// wearing the Ember Ring (burns last longer)\n    pub gear_ember: bool,\n"),
    ("            stand_cd: 0,\n", "            stand_cd: 0,\n            gear_ember: false,\n"),
    ("        self.first_tips();\n", "        self.first_tips();\n        self.ember_ring();\n"),
])
edit('src/spells.rs', [("""        let dmg = dmg * self.power_mult() * self.status_mult(i);
        if let Some(h) = self.hit_h.take() {""", """        let dmg = dmg * self.power_mult() * self.status_mult(i) * crate::uniques::mud_bonus(self);
        if let Some(h) = self.hit_h.take() {""")])
# the bosses hand out the uniques
edit('src/areas.rs', [
    ("""        self.flash = 0.25;
        self.open_portal();""", """        self.flash = 0.25;
        self.give_unique();
        self.open_portal();"""),
    ("""        self.flash = 0.2;
        self.open_portal();""", """        self.flash = 0.2;
        self.give_unique();
        self.open_portal();"""),
])
print("uniques ok")
