# Safe, validated, versioned saves (review items 1, 6, 18).
def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        n = s.count(a)
        assert n == 1, (path, a[:90], n)
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("mod areas;\n", "mod areas;\nmod savefile;\n"),
    ("g.has_save = std::path::Path::new(areas::SAVE_FILE).exists();", "g.has_save = savefile::exists(areas::SAVE_FILE);"),
])

edit('src/game.rs', [
    ("    pub toast: Option<(String, i32)>,\n", "    pub toast: Option<(String, i32)>,\n    /// a save that failed, to tell the player about\n    pub save_fail: std::cell::RefCell<Option<String>>,\n"),
])
s = open('src/game.rs', encoding='utf8').read()
# find the struct initialiser's toast line
import re
m = re.search(r"\n(\s+)toast: None,\n", s)
assert m, "toast init"
s = s[:m.end()] + m.group(1) + "save_fail: std::cell::RefCell::new(None),\n" + s[m.end():]
open('src/game.rs', 'w', encoding='utf8').write(s)

A = 'src/areas.rs'
s = open(A, encoding='utf8').read()
start = s.index("    // ---------------------------------------------------------------- saving")
end = s.rindex("}")  # the impl's closing brace
new = r'''    // ---------------------------------------------------------------- saving
    pub fn save(&self) {
        if !self.saving {
            return;
        }
        let p = &self.player;
        let join = |v: Vec<String>| v.join(",");
        let mut lines = vec![
            "version 2".to_string(),
            format!("seed {}", self.seed),
            format!("area {}", self.area.key()),
            format!("level {}", p.sk.level),
            format!("xp {}", p.sk.xp),
            format!("points {}", p.sk.points),
            format!("gold {}", p.gold),
            format!("potions {}", p.potions),
            format!("kills {}", p.kills),
            format!("dead {}", p.dead as u8),
            format!("waypoints {}", join(self.discovered.iter().map(|a| a.key()).collect())),
            format!("king {}", self.king_dead as u8),
            format!("chief {}", self.chief_dead as u8),
            format!("act1 {}", self.act1_done as u8),
            format!("story {}", self.story),
            format!("played {}", self.played),
        ];
        // skills by name, so adding or reordering skills never scrambles a save
        for (i, n) in NODES.iter().enumerate() {
            if p.sk.rank[i] > 0 || p.sk.tomes[i] {
                lines.push(format!("skill {:?} {} {}", n.sk, p.sk.rank[i], p.sk.tomes[i] as u8));
            }
        }
        // explored minimap, packed into hex, for the outdoor areas
        for (k, v) in &self.seen {
            if AreaId::from_key(k).map(|a| a.is_dungeon()).unwrap_or(true) {
                continue;
            }
            let mut hex = String::new();
            for c in v.chunks(4) {
                let n = c.iter().enumerate().fold(0u8, |a, (i, b)| a | ((*b as u8) << i));
                hex.push(char::from_digit(n as u32, 16).unwrap());
            }
            lines.push(format!("seen_{} {}", k, hex));
        }
        // "equipped" always comes first, so an empty slot loads as empty
        lines.push("equipped".into());
        for (k, it) in self.player.equip.iter().enumerate() {
            if let Some(it) = it {
                lines.push(format!("equip{} {}", k, it.encode()));
            }
        }
        for it in &self.player.bag {
            lines.push(format!("bag {}", it.encode()));
        }
        // a save without this last line was cut short and is never trusted
        lines.push("end".into());
        if let Err(e) = crate::savefile::write_safe(SAVE_FILE, &(lines.join("\n") + "\n")) {
            *self.save_fail.borrow_mut() = Some(format!("COULD NOT SAVE: {}", e).to_uppercase());
        }
    }

    /// is this a whole save? new saves end with "end"; older ones must have the key lines
    pub fn save_ok(txt: &str) -> bool {
        let has = |k: &str| txt.lines().any(|l| l.split(' ').next() == Some(k));
        if has("version") {
            txt.lines().rev().find(|l| !l.trim().is_empty()).map(|l| l.trim() == "end").unwrap_or(false)
        } else {
            ["seed", "level", "xp", "gold", "ranks", "played"].iter().all(|k| has(k))
        }
    }

    /// load the save, or its backup if the save is damaged
    pub fn load(&mut self) -> Load {
        let main = crate::savefile::read(SAVE_FILE);
        let txt = match &main {
            Some(t) if Self::save_ok(t) => t.clone(),
            None => return Load::None,
            Some(_) => match crate::savefile::read(&format!("{}.bak", SAVE_FILE)) {
                Some(b) if Self::save_ok(&b) => {
                    self.toast = Some(("YOUR SAVE WAS DAMAGED - LOADED THE BACKUP".into(), 400));
                    b
                }
                _ => return Load::Damaged,
            },
        };
        let mut was_dead = false;
        for line in txt.lines() {
            let mut it = line.splitn(2, ' ');
            let (k, v) = (it.next().unwrap_or(""), it.next().unwrap_or("").trim());
            let nums = |v: &str| v.split(',').filter_map(|x| x.parse::<i64>().ok()).collect::<Vec<_>>();
            match k {
                "seed" => self.seed = v.parse().unwrap_or(self.seed),
                "area" => self.area = AreaId::from_key(v).unwrap_or(AreaId::Ashford),
                "level" => self.player.sk.level = v.parse().unwrap_or(1),
                "xp" => self.player.sk.xp = v.parse().unwrap_or(0),
                "points" => self.player.sk.points = v.parse().unwrap_or(0),
                // the old format: ranks by position
                "ranks" => {
                    for (i, r) in nums(v).iter().enumerate().take(N) {
                        self.player.sk.rank[i] = *r as u8;
                    }
                }
                "tomes" => {
                    for (i, t) in nums(v).iter().enumerate().take(N) {
                        self.player.sk.tomes[i] = *t != 0;
                    }
                }
                // the new format: by name
                "skill" => {
                    let mut f = v.split(' ');
                    let (name, r, t) = (f.next().unwrap_or(""), f.next().and_then(|x| x.parse::<u8>().ok()).unwrap_or(0), f.next() == Some("1"));
                    if let Some(i) = NODES.iter().position(|n| format!("{:?}", n.sk) == name) {
                        self.player.sk.rank[i] = r.min(NODES[i].max);
                        self.player.sk.tomes[i] = t;
                    }
                }
                "gold" => self.player.gold = v.parse().unwrap_or(0),
                "potions" => self.player.potions = v.parse::<i32>().unwrap_or(3).clamp(0, crate::shop::MAX_POTIONS),
                "kills" => self.player.kills = v.parse().unwrap_or(0),
                "dead" => was_dead = v == "1",
                "waypoints" => self.discovered = v.split(',').filter_map(AreaId::from_key).collect(),
                "king" => self.king_dead = v == "1",
                "chief" => self.chief_dead = v == "1",
                "act1" => self.act1_done = v == "1",
                "story" => self.story = v.parse().unwrap_or(0),
                _ if k.starts_with("seen_") => {
                    let mut out = vec![];
                    for ch in v.chars() {
                        let n = ch.to_digit(16).unwrap_or(0);
                        for i in 0..4 {
                            out.push(n & (1 << i) != 0);
                        }
                    }
                    out.resize(crate::game::MM.0 * crate::game::MM.1, false);
                    self.seen.insert(k[5..].to_string(), out);
                }
                "played" => self.played = v.parse().unwrap_or(0),
                "equipped" => self.player.equip = [None, None, None, None],
                "equip0" | "equip1" | "equip2" | "equip3" => {
                    let s: usize = k[5..].parse().unwrap_or(0);
                    self.player.equip[s] = crate::items::Item::decode(v);
                }
                "bag" => {
                    if let Some(it) = crate::items::Item::decode(v) {
                        if self.player.bag.len() < crate::gear::BAG_SIZE {
                            self.player.bag.push(it);
                        }
                    }
                }
                _ => {}
            }
        }
        self.infer_story();
        self.apply_gear();
        self.player.hp = self.player.maxhp;
        self.player.mana = self.player.maxmana;
        if was_dead {
            Load::Dead
        } else {
            Load::Ok
        }
    }
'''
s = s[:start] + new + s[end:]
s = s.replace('''        if load && g.load() {
            let to = g.area.surface();
            g.area = AreaId::Ashford;
            g.travel(to);
            g.banner = Some(("WELCOME BACK".into(), to.name(), 200));
        } else {''', '''        let loaded = if load { g.load() } else { Load::None };
        if loaded == Load::Dead {
            // you quit while dead: the same price as waking normally
            g.player.gold = g.player.gold * 9 / 10;
            g.area = AreaId::Ashford;
            g.travel(AreaId::Ashford);
            g.banner = Some(("YOU WAKE IN ASHFORD".into(), "THE DEAD TOOK A TENTH OF YOUR GOLD".into(), 220));
        } else if loaded == Load::Ok {
            // dungeons are rebuilt each visit: you continue at their entrance
            let was = g.area;
            let to = was.surface();
            g.area = AreaId::Ashford;
            g.travel(to);
            g.banner = Some(("WELCOME BACK".into(), to.name(), 200));
            if was.is_dungeon() {
                g.toast = Some(("DUNGEONS CHANGE EACH VISIT - YOU WAIT AT THE ENTRANCE".into(), 300));
            }
            // quit during the last talk with Maren: the act still ends
            if g.has(crate::story::TOLD_CHIEF) && !g.act1_done {
                g.check_act_end();
            }
        } else {
            if loaded == Load::Damaged {
                // keep the damaged file for a look, and don't write over it
                let _ = std::fs::copy(crate::savefile::path(SAVE_FILE), crate::savefile::path("rendgrave_save.damaged.txt"));
            }''', 1)
s = s.replace('''            g.banner = Some(("RENDGRAVE".into(), "ASHFORD - THE LAST SAFE HEARTH".into(), 260));
        }
        g
    }''', '''            g.banner = Some(("RENDGRAVE".into(), "ASHFORD - THE LAST SAFE HEARTH".into(), 260));
            if loaded == Load::Damaged {
                g.toast = Some(("YOUR SAVE WAS DAMAGED - A COPY IS IN RENDGRAVE_SAVE.DAMAGED.TXT".into(), 600));
            }
        }
        g
    }''', 1)
s = s.replace("pub const SAVE_FILE: &str = \"rendgrave_save.txt\";", """pub const SAVE_FILE: &str = "rendgrave_save.txt";

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Load {
    None,
    Ok,
    /// saved while dead
    Dead,
    /// the save and its backup are both damaged
    Damaged,
}""")
open(A, 'w', encoding='utf8').write(s)

# settings files beside the executable too
edit('src/audio.rs', [
    ('let _ = std::fs::write("rendgrave_settings.txt", format!("volume {}\\nmusic {}\\nsfx {}\\n", self.volume, self.music_on as u8, self.sfx_on as u8));',
     'crate::savefile::write_quiet("rendgrave_settings.txt", &format!("volume {}\\nmusic {}\\nsfx {}\\n", self.volume, self.music_on as u8, self.sfx_on as u8));'),
    ('let Ok(t) = std::fs::read_to_string("rendgrave_settings.txt") else { return };', 'let Some(t) = crate::savefile::read("rendgrave_settings.txt") else { return };'),
])
edit('src/menu.rs', [
    ('    std::fs::read_to_string("rendgrave_options.txt")\n        .ok()\n        .and_then(', '    crate::savefile::read("rendgrave_options.txt")\n        .and_then('),
    ('let _ = std::fs::write("rendgrave_options.txt", format!("shake {}\\n", v));', 'crate::savefile::write_quiet("rendgrave_options.txt", &format!("shake {}\\n", v));'),
])
print("saves ok")
