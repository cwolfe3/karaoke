use std::time::{Instant, Duration};
use cpal::traits::HostTrait;
use iced::time;
use iced::executor;
use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::{self, Canvas, Geometry, Cursor, Frame, Path, Stroke};
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
    Tick(Instant)
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
        match message {
            Message::Tick(duration) => {
                match self.mic1.consume() {
                    Some(note) => {
                        self.song1.add_after(note)
                    },
                    None => (),
                }
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
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }
}

impl canvas::Program<Message> for SongSession {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let voiced_stroke = Stroke {
            color: Color::new(0.0, 0.2, 1.0, 1.0),
            width: 5.0,
            ..Stroke::default()
        };
        let rest_stroke = Stroke {
            color: Color::WHITE,
            width: 5.0,
            ..Stroke::default()
        };
        let mut frame = Frame::new(bounds.size()); 
        let mut length = 0;
        for phrase in &self.song1.phrases {
            for note in phrase {
                let path = note_path(note.clone(), length);
                length += note.length;
                if note.voiced {
                    frame.stroke(&path, voiced_stroke);
                } else {
                    frame.stroke(&path, rest_stroke);
                }
            }
        }
        vec![frame.into_geometry()]
    }
}

fn note_path(note: Note, length: u32) -> Path {
    let y = note.pitch as f32;
    let x = length as f32;
    Path::line(
        note_to_frame_transform(Point::new(x, y)),
        note_to_frame_transform(Point::new(x + (note.length as f32), y))
        )
}

fn note_to_frame_transform(point: Point) -> Point {
    Point::new(point.x / 100.0, 100.0 - point.y)
}
