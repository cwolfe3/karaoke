mod frame_splitter;
mod mic;
mod note;
mod song;
mod song_library;
mod song_panel;
mod song_view;
mod timer;
mod track;

use eframe::egui;

use crate::song_library::SongLibrary;
use crate::song_panel::TrackSession;

struct Karaoke {
    state: KaraokeState,
    library: SongLibrary,
    session: Option<TrackSession>,
    scroll_position: f32, // This is used to calculate the scrollbar
                          // offset. It approaches library.selection_index
                          // every tick.
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum KaraokeState {
    Library,
    SongSelection,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Message {
    FocusUp,
    FocusDown,
    FocusRight,
    FocusLeft,
    Focus(usize),
    SelectFocused,
    Play,
    Pause,
    Resume,
    Tick,
    None,
}

impl Karaoke {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut library = SongLibrary::new(std::path::Path::new("songs"));
        for song in library.songs.iter_mut() {
            if let Some(cover) = &mut song.album_cover {
                cover.load_texture(&cc.egui_ctx);
            }
        }
        Karaoke {
            state: KaraokeState::Library,
            library,
            session: None,
            scroll_position: 0.0,
        }
    }

    fn title(&self) -> String {
        String::from("Karaoke")
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::FocusUp => match self.state {
                KaraokeState::Library => self.library.select_previous(),
                KaraokeState::SongSelection => (),
                KaraokeState::Playing => (),
                KaraokeState::Paused => (),
            },
            Message::FocusDown => match self.state {
                KaraokeState::Library => self.library.select_next(),
                KaraokeState::SongSelection => (),
                KaraokeState::Playing => (),
                KaraokeState::Paused => (),
            },
            Message::FocusRight => match self.state {
                KaraokeState::Library => self.state = KaraokeState::SongSelection,
                KaraokeState::SongSelection => (),
                KaraokeState::Playing => (),
                KaraokeState::Paused => (),
            },
            Message::FocusLeft => match self.state {
                KaraokeState::Library => (),
                KaraokeState::SongSelection => self.state = KaraokeState::Library,
                KaraokeState::Playing => (),
                KaraokeState::Paused => (),
            },
            Message::Focus(i) => self.library.select(i),
            Message::SelectFocused => match self.state {
                KaraokeState::Library => self.handle_message(Message::FocusRight),
                KaraokeState::SongSelection => {
                    self.state = KaraokeState::Playing;
                    self.scroll_position = self.library.selection_index as f32;
                    let song = self
                        .library
                        .songs
                        .get(self.library.selection_index)
                        .unwrap();
                    self.session = if let Ok(session) = TrackSession::new(song.clone()) {
                        Some(session)
                    } else {
                        //TODO handle not being able to load song
                        None
                    }
                }
                KaraokeState::Playing => (),
                KaraokeState::Paused => (),
            },
            Message::Play => (),
            Message::Pause => (),
            Message::Resume => (),
            Message::Tick => self.tick(),
            Message::None => (),
        }
    }

    fn tick(&mut self) {
        match self.state {
            KaraokeState::Library => {
                self.update_scroll_position();
            }
            KaraokeState::SongSelection => {
                self.update_scroll_position();
            }
            KaraokeState::Playing => match &mut self.session {
                Some(session) => {
                    session.tick();
                    match session.state {
                        song_panel::State::Playing => (),
                        song_panel::State::Paused => (),
                        song_panel::State::Finished => {
                            self.state = KaraokeState::Library;
                        }
                    };
                }
                None => (),
            },
            KaraokeState::Paused => todo!(),
        }
    }

    fn update_scroll_position(&mut self) {
        let difference = self.library.selection_index as f32 - self.scroll_position;
        self.scroll_position += difference / 10.0;
    }
}

impl eframe::App for Karaoke {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_input(self, ctx);
        self.handle_message(Message::Tick);
        let screen_width = ctx.input().screen_rect().width();
        let screen_height = ctx.input().screen_rect().height();
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                KaraokeState::Library | KaraokeState::SongSelection => {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true),
                        |ui| {
                            let w = screen_width * 0.35;
                            let h = screen_height * 0.166;
                            egui::ScrollArea::vertical()
                                .auto_shrink([true, true])
                                .vertical_scroll_offset(
                                    convert_scroll_position(
                                        self.scroll_position,
                                        h + ui.spacing().item_spacing.y,
                                    ) - screen_height / 2.0,
                                )
                                .show(ui, |ui| {
                                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                        let mut clicked = false;
                                        let mut clicked_index = 0;
                                        for (i, song) in self.library.songs[..].iter().enumerate() {
                                            // TODO add default image
                                            let mut button;
                                            let button_label = [
                                                song.name.clone(),
                                                song.artist.clone(),
                                                song.album.clone(),
                                            ]
                                            .join("\n");
                                            if let Some(cover) = &song.album_cover {
                                                button = egui::Button::image_and_text(
                                                    cover.texture.as_ref().unwrap().id(),
                                                    egui::vec2(h * 0.8, h * 0.8),
                                                    button_label,
                                                );
                                            } else {
                                                button = egui::Button::new(button_label);
                                            }
                                            if self.library.selection_index == i
                                                && self.state == KaraokeState::Library
                                            {
                                                button = button.stroke(
                                                    ctx.style().visuals.widgets.hovered.fg_stroke,
                                                );
                                                button = button.fill(
                                                    ctx.style().visuals.widgets.hovered.bg_fill,
                                                );
                                            } else {
                                                button = button.stroke(
                                                    ctx.style().visuals.widgets.inactive.bg_stroke,
                                                );
                                                button = button.fill(
                                                    ctx.style().visuals.widgets.inactive.bg_fill,
                                                );
                                            }
                                            if ui.add_sized([w, h], button).clicked() {
                                                clicked = true;
                                                clicked_index = i;
                                            };
                                        }
                                        if clicked {
                                            if clicked_index == self.library.selection_index {
                                                self.handle_message(Message::SelectFocused);
                                            } else {
                                                self.handle_message(Message::Focus(clicked_index))
                                            }
                                        }
                                    })
                                });
                            ui.separator();
                            ui.vertical(|ui| {
                                if let Some(cover) = &self
                                    .library
                                    .songs
                                    .get(self.library.selection_index)
                                    .unwrap()
                                    .album_cover
                                {
                                    ui.image(
                                        cover.texture.as_ref().unwrap().id(),
                                        egui::vec2(
                                            (screen_width - w) * 0.5,
                                            (screen_width - w) * 0.5,
                                        ),
                                    );
                                }
                                let mut play_button = egui::Button::new("Play Now");
                                if self.state == KaraokeState::SongSelection {
                                    play_button = play_button
                                        .stroke(ctx.style().visuals.widgets.hovered.fg_stroke);
                                    play_button = play_button
                                        .fill(ctx.style().visuals.widgets.hovered.bg_fill);
                                }
                                ui.add_sized([(screen_width - w) * 0.5, 30.0], play_button);
                            });
                        },
                    )
                }
                KaraokeState::Playing => egui::Frame::none().show(ui, |ui| {
                    egui::Frame::canvas(ui.style()).show(ui, |ui| match &mut self.session {
                        Some(some_session) => some_session.draw(ui),
                        None => ui.label("Unable to play"),
                    });
                }),
                KaraokeState::Paused => egui::Frame::none().show(ui, |ui| {
                    ui.label("Paused");
                }),
            }
        });
        ctx.request_repaint();
    }
}

fn convert_scroll_position(scroll_position: f32, item_size: f32) -> f32 {
    (scroll_position + 0.5) * item_size as f32
}

fn handle_input(karaoke: &mut Karaoke, ctx: &egui::Context) {
    if ctx.input().key_pressed(egui::Key::ArrowDown) {
        karaoke.handle_message(Message::FocusDown);
    }
    if ctx.input().key_pressed(egui::Key::ArrowUp) {
        karaoke.handle_message(Message::FocusUp);
    }
    if ctx.input().key_pressed(egui::Key::ArrowLeft) {
        karaoke.handle_message(Message::FocusLeft);
    }
    if ctx.input().key_pressed(egui::Key::ArrowRight) {
        karaoke.handle_message(Message::FocusRight);
    }
    if ctx.input().key_pressed(egui::Key::Enter) {
        karaoke.handle_message(Message::SelectFocused);
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Karaoke",
        native_options,
        Box::new(|cc| Box::new(Karaoke::new(cc))),
    )

    //let song = Song::read(Path::new("test.song"));
    //let mut siv = cursive::default();
    //siv.add_global_callback('q', |s| s.quit());
    //siv.add_layer(views::ScrollView::new(SongView::new(song).with_name("view")).scroll_x(true));
    //siv.run();
}
