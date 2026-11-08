def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:80], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/areas.rs', [
    ("""    pub fn xp_for(&self, k: Kind) -> i32 {
        let base = match k {""", """    pub fn xp_for(&self, k: Kind) -> i32 {
        let base = base_xp(k);
        let area = 1.0 + 0.25 * (self.alvl - 1) as f32;
        let gap = self.player.sk.level - self.alvl;
        let penalty = if gap > 8 { 0.15 } else if gap > 4 { 0.5 } else { 1.0 };
        (base * area * penalty) as i32
    }
}

/// XP for a kill before area scaling
pub fn base_xp(k: Kind) -> f32 {
    {
        match k {"""),
    ("""            _ => 20.0,
        };
        let area = 1.0 + 0.25 * (self.alvl - 1) as f32;
        let gap = self.player.sk.level - self.alvl;
        let penalty = if gap > 8 { 0.15 } else if gap > 4 { 0.5 } else { 1.0 };
        (base * area * penalty) as i32
    }
""", """            _ => 20.0,
        }
    }
}

impl Game {
"""),
])
edit('src/mapdump.rs', [
    ("""        println!("{:<12} {}x{}""", """        let area = 1.0 + 0.25 * (id.level() - 1) as f32;
        let xp: f32 = sp.enemies.iter().map(|e| crate::areas::base_xp(e.0) * area * if matches!(e.0, Kind::King | Kind::Chief) { 1.0 } else { 1.0 + 2.0 * elite_share }).sum();
        println!("{:<12} xp {:>6.0}  lv {:>2}", id.key(), xp, id.level());
        println!("{:<12} {}x{}"""),
])
print("xp dump ok")
