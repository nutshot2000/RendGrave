def edit(path, pairs):
    s = open(path, encoding='utf8').read()
    for a, b, *n in pairs:
        n = n[0] if n else 1
        assert s.count(a) == n, (path, a[:70], s.count(a))
        s = s.replace(a, b)
    open(path, 'w', encoding='utf8').write(s)


M = 'src/main.rs'
edit(M, [
    ("mod areas;\n", "mod areas;\nmod audio;\nmod synth;\n"),
    ("    sel_pending: Option<usize>,\n}", "    sel_pending: Option<usize>,\n    audio: Option<audio::Audio>,\n}"),
    ("        sel_pending: None,\n    };", "        sel_pending: None,\n        audio: if show.is_some() { None } else { audio::Audio::new() },\n    };"),
    # move the `show` binding check before App construction uses it: compute flag first
    ("""        game: if show.is_some() { game::Game::new(seed) } else { game::Game::start(!fresh) },""",
     """        game: if show.is_some() { game::Game::new(seed) } else { game::Game::start(!fresh) },"""),
    ("""                        KeyCode::KeyE => i.interact = down,""", """                        KeyCode::KeyE => i.interact = down,
                        KeyCode::KeyM if down && !event.repeat => {
                            if let Some(a) = &mut self.audio {
                                a.toggle_music();
                                self.game.toast = Some((if a.music_on { "MUSIC ON" } else { "MUSIC OFF" }.into(), 90));
                            }
                        }
                        KeyCode::KeyN if down && !event.repeat => {
                            if let Some(a) = &mut self.audio {
                                a.toggle_sfx();
                                self.game.toast = Some((if a.sfx_on { "SOUND ON" } else { "SOUND OFF" }.into(), 90));
                            }
                        }
                        KeyCode::Minus | KeyCode::NumpadSubtract if down => {
                            if let Some(a) = &mut self.audio {
                                a.change_volume(-0.1);
                                self.game.toast = Some((format!("VOLUME {}%", (a.volume * 100.0).round() as i32), 90));
                            }
                        }
                        KeyCode::Equal | KeyCode::NumpadAdd if down => {
                            if let Some(a) = &mut self.audio {
                                a.change_volume(0.1);
                                self.game.toast = Some((format!("VOLUME {}%", (a.volume * 100.0).round() as i32), 90));
                            }
                        }"""),
    # every redraw: pick the music, play this frame's sounds
    ("""        let Some(frame) = &mut self.frame else { return };""", """        if let Some(a) = &mut self.audio {
            use areagen::AreaId;
            let g = &self.game;
            let want = if g.player.dead {
                audio::Track::Title
            } else if g.enemies.iter().any(|e| e.alive && e.k == body::Kind::King && e.aggro) {
                audio::Track::Boss
            } else {
                match g.area {
                    AreaId::Ashford => audio::Track::Ashford,
                    AreaId::Pines | AreaId::Graveyard => audio::Track::Pines,
                    AreaId::Crypt(_) => audio::Track::Crypt,
                }
            };
            let duck = if g.paused || g.tree || g.wp_open { 0.45 } else { 1.0 };
            a.set_music(Some(want), duck);
            let snds = std::mem::take(&mut self.game.sounds);
            a.update(&snds, (self.game.player.x, self.game.player.y));
        } else {
            self.game.sounds.clear();
        }
        let Some(frame) = &mut self.frame else { return };"""),
])

H = 'src/hud.rs'
edit(H, [
    ("""        u.text_c(fw / 2, fh / 3 + 22, "ESC RESUME    K SKILL TREE    CLOSE THE WINDOW TO QUIT", [200, 190, 190], 1);""",
     """        u.text_c(fw / 2, fh / 3 + 22, "ESC RESUME    K SKILL TREE    CLOSE THE WINDOW TO QUIT", [200, 190, 190], 1);
        u.text_c(fw / 2, fh / 3 + 34, "M MUSIC ON-OFF    N SOUND ON-OFF    - AND + VOLUME", [150, 140, 150], 1);"""),
    ("""    if g.fade > 0 {""", """    if let Some((msg, t)) = &g.toast {
        let a = ((*t).min(20) as f32 / 20.0 * 255.0) as u8;
        let w = Ui::text_w(msg, 1);
        u.rect(fw / 2 - w / 2 - 5, 74, w + 10, 13, [10, 6, 10], a.min(210));
        u.text(fw / 2 - w / 2, 78, msg, [240, 220, 170], 1, false);
    }
    if g.fade > 0 {"""),
])
print("audio main ok")
