def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


p = 'src/world.rs'
s = open(p, encoding='utf8').read()
# drop the old single fixed map generator (gen.rs replaces it)
i = s.index("    pub fn generate(seed_: u64) -> (World, Spawns) {")
j = s.rindex("}\n")  # end of impl World
s = s[:i].rstrip() + "\n}\n"
i = s.index("pub struct Spawns {")
j = s.index("}\n", i) + 2
s = s[:i] + s[j:]
open(p, 'w', encoding='utf8').write(s)

edit(p, [
    ("pub const ASH: u8 = 16;", "pub const ASH: u8 = 16;\n/// dungeon bedrock: very tall, dark, indestructible\npub const ROCK: u8 = 17;"),
    ("pub const MUD: u8 = 5;", "pub const MUD: u8 = 5;\npub const STAIRS: u8 = 6;\npub const STAIRSUP: u8 = 7;"),
    ("pub const AUX_INDOOR: u8 = 2;", "pub const AUX_INDOOR: u8 = 2;\npub const AUX_DEAD: u8 = 4;"),
    ("    matches!(m, STONE | BRICK | TREE | WOOD)\n", "    matches!(m, STONE | BRICK | TREE | WOOD | ROCK)\n"),
    ("        WOOD => 8,\n        _ => 0,", "        WOOD => 8,\n        ROCK => 44,\n        _ => 0,"),
    ("    fn blob(", "    pub fn blob("),
    ("    fn rect(", "    pub fn rect("),
    ("    fn ground_rect(", "    pub fn ground_rect("),
    ("""    #[inline]
    fn keep(&mut self, x: usize, y: usize) {""", """    pub fn wake_all(&mut self) {
        for c in self.act.iter_mut() {
            *c = 30;
        }
    }
    #[inline]
    fn keep(&mut self, x: usize, y: usize) {"""),
])
print("world ok")
