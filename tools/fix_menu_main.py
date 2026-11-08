def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b in pairs:
        assert s.count(a) == 1, (path, a[:90], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


edit('src/main.rs', [
    ("mod areas;\n", "mod areas;\nmod menu;\n"),
    ("        game: if show.is_some() { game::Game::new(seed) } else { game::Game::start(!fresh) },",
     "        game: if show.is_some() { game::Game::new(seed) } else if fresh { game::Game::start(false) } else { title_game() },"),
    ("    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {", "    fn about_to_wait(&mut self, el: &ActiveEventLoop) {"),
    ("""            if n == 4 {
                self.acc = 0.0;
            }
        }""", """            if n == 4 {
                self.acc = 0.0;
            }
        }
        // menu requests the game can't carry out itself
        if let Some(r) = self.game.req.take() {
            use menu::Req;
            match r {
                Req::Quit => {
                    self.game.save();
                    el.exit();
                }
                Req::Fullscreen => {
                    if let Some(g) = &self.gfx {
                        let full = g.window.fullscreen().is_some();
                        g.window.set_fullscreen(if full { None } else { Some(winit::window::Fullscreen::Borderless(None)) });
                    }
                }
                Req::Music => {
                    if let Some(a) = &mut self.audio {
                        a.toggle_music();
                    }
                }
                Req::Sound => {
                    if let Some(a) = &mut self.audio {
                        a.toggle_sfx();
                    }
                }
                Req::Volume(d) => {
                    if let Some(a) = &mut self.audio {
                        a.change_volume(d);
                    }
                }
                Req::NewGame => self.game = game::Game::start(false),
                Req::Continue => self.game = game::Game::start(true),
                Req::ToTitle => {
                    self.game.save();
                    self.game = title_game();
                }
            }
        }
        if let Some(a) = &self.audio {
            self.game.audio_state = (a.music_on, a.sfx_on, a.volume);
        }"""),
    ("""            let want = if g.player.dead {
                audio::Track::Title""", """            let want = if g.player.dead || g.title {
                audio::Track::Title"""),
    ("""/// test helper: stand a little to the west of the first reachable enemy""", """/// the title screen: Ashford at night behind the menu
fn title_game() -> game::Game {
    let mut g = game::Game::new(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1));
    g.saving = false;
    g.travel(areagen::AreaId::Ashford);
    g.banner = None;
    g.fade = 60;
    g.title = true;
    g.has_save = std::path::Path::new(areas::SAVE_FILE).exists();
    g.sounds.clear();
    g
}

/// test helper: stand a little to the west of the first reachable enemy"""),
])
print("main menu wired")
