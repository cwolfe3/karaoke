//use std::env;

mod note;
mod song;
mod song_view;
mod mic;
mod song_panel;
mod track;
mod song_library;

use std::time::Duration;
use iced::time;
use iced::executor;
use iced::Theme;
use iced::{
    Application,
    Command,
    Element,
    Length,
    Settings,
    Subscription,
};

use iced::widget::{
    scrollable,
    button,
    container,
    text,
};

use iced_native::Event;
use iced_native::keyboard::Event as kbEvent;
use iced_native::keyboard::KeyCode;

use crate::song_library::SongLibrary;
//use std::path::Path;
//use song::Song;
//use song_view::SongView;
//use cursive::views;
//use cursive::traits::Nameable;

struct Karaoke {
    state: KaraokeState,
    library: SongLibrary,
}

enum KaraokeState {
    Library,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
enum Message {
    FocusUp,
    FocusDown,
    Focus(usize),
    SelectFocused,
    Play,
    Pause,
    Resume,
    Tick,
    None,
}

impl Application for Karaoke {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let library = SongLibrary::read_songs(std::path::Path::new("songs"));
        (
        Karaoke {
            state: KaraokeState::Library,
            library,
        },
        Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Karaoke")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FocusUp => {
                self.library.select_previous();
            },
            Message::FocusDown => {
                self.library.select_next();
            },
            Message::Focus(_) => (),
            Message::SelectFocused => {
                
            },
            Message::Play => (),
            Message::Pause => (),
            Message::Resume => (),
            Message::Tick => (),
            Message::None => (),
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        println!("VIEW CALLED");
        match &self.state {
            KaraokeState::Library => {
                let mut list = iced::widget::column![];
                // TODO why does this have to be mutable?
                for (i, song) in &mut self.library.songs[..].iter().enumerate() {
                    let mut song_option = button(text(song.name.clone()))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .on_press(Message::None); // To trick it to be a bright color
                    if i == self.library.selection_index {    
                        song_option = song_option.style(iced::theme::Button::Primary);
                    } else {
                        song_option = song_option.style(iced::theme::Button::Secondary);
                    }
                    let song_option = container(song_option)
                        .width(Length::FillPortion(65))
                        .height(Length::Units(150));
                    list = list.push(song_option); 
                }
                let scrollable = scrollable(list);
                iced::widget::row!()
                    .push(container(scrollable).width(Length::FillPortion(65)))
                    .push(iced::widget::column![]
                          .push(text("SONG_DETAILS"))
                          .width(Length::FillPortion(35)))
                    .into()
            },
            KaraokeState::Playing => {
                text("Playing").into()
            },
            KaraokeState::Paused => {
                text("Playing").into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let event_sub = iced_native::subscription::events().map(
            |event| {
                match event {
                    Event::Mouse(_) => Message::None,
                    Event::Window(_) => Message::None,
                    Event::Touch(_) => Message::None,
                    Event::Keyboard(key_event) => {
                        match key_event {
                            kbEvent::KeyPressed { key_code, modifiers } => {
                                match key_code {
                                    KeyCode::Up => {
                                        Message::FocusUp
                                    },
                                    KeyCode::Down => {
                                        Message::FocusDown
                                    }
                                    _ => Message::None,
                                }
                            }
                            kbEvent::KeyReleased { key_code, modifiers } => Message::None,
                            kbEvent::CharacterReceived(_) => Message::None,
                            kbEvent::ModifiersChanged(_) => Message::None,
                        }
                    },
                    Event::PlatformSpecific(_) => Message::None,
                }
            }
        );
        let tick_sub = time::every(Duration::from_millis(33)).map(|_| Message::Tick);
        Subscription::batch([event_sub, tick_sub])
    }
}

fn main() {
    Karaoke::run(Settings {
        antialiasing: true,
        ..Settings::default()
    });
	//let song = Song::read(Path::new("test.song"));

	//let mut siv = cursive::default();
	//siv.add_global_callback('q', |s| s.quit());
	//siv.add_layer(views::ScrollView::new(SongView::new(song).with_name("view")).scroll_x(true));
	//siv.run();
}
