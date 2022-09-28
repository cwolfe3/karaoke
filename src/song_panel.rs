use std::time::{Instant, Duration};
use cpal::traits::HostTrait;
use iced::time;
use iced::executor;
use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::{self, Canvas, Geometry, Cursor, Frame, Path};
use iced::{
    Application,
    Command,
    Column,
    Element,
    Length,
    Rectangle,
    Color,
    Settings,
    Point,
    Subscription,
};

use crate::song::Song;
use crate::mic::Microphone;
use crate::note::Note;

pub fn main() -> iced::Result {
    SongSession::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct SongSession {
    song: Song,
    mic1: Microphone,
    song1: Song,
    now: Instant,
}
#[derive(Debug)]
enum Message {
    Tick(Duration)
}

impl Application for SongSession {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            SongSession {
                song: Song::read(std::path::Path::new("test.song")),
                mic1: Microphone::new(cpal::default_host().default_output_device().expect("")),
                song1: Song::new("".to_string(), "".to_string(), "".to_string()),
                now: Instant::now()
            },
            Command::none(),
            )
    }

    fn title(&self) -> String {
        String::from("sdfsdf")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("SDFSDF");
        match message {
            Message::Tick(duration) => {
                let note = self.mic1.consume();
                self.song1.add_after(note.unwrap());
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Column::new().push(
            Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            ).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        unimplemented!();
    }
}

impl canvas::Program<Message> for SongSession {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size()); 
        for phrase in &self.song1.phrases {
            for note in phrase {
                let path = note_path(note.clone());
                frame.fill(&path, Color::BLACK);
            }
        }
        vec![frame.into_geometry()]
    }
}

fn note_path(note: Note) -> Path {
    Path::line(
        note_to_frame_transform(Point::new(0.0, 0.0)),
        note_to_frame_transform(Point::new((note.length as f32) * 100.0, 0.0))
        )
}

fn note_to_frame_transform(point: Point) -> Point {
    Point::new(point.x, point.y)
}
