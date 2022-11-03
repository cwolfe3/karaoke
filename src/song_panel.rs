use std::time::{Instant, Duration};
use cpal::traits::HostTrait;

use crate::song::Song;
use crate::track::Track;
use crate::mic::Microphone;
use crate::note::Note;

struct TrackSession {
    backing_track: Track,
    mic: Microphone,
    track: Track,
    state: State,
    elapsed_time: Duration,
    elapsed_time_goal: Duration,
    last_update_time: Instant,
    phrase_index: usize,
    note_index: usize,
    chunk_lengths: Vec<u32>,
    chunk_index: usize,
}
#[derive(Debug)]

enum Message {
    Tick,
}

enum State {
    Playing,
    Paused,
    Finished
}

impl TrackSession {
    fn next_chunk(&mut self) {
        self.chunk_index += 1;
        if self.chunk_index >= self.chunk_lengths.len() {
            self.chunk_index = 0;
            self.note_index += 1;
            if self.note_index >= self.backing_track.phrases[self.phrase_index].len() {
                self.note_index = 0;
                self.phrase_index += 1;
                if self.phrase_index >= self.backing_track.phrases.len() {
                    self.state = State::Finished;
                    return;
                }
            }
            let note_length = self.backing_track.phrases[self.phrase_index][self.note_index].length;
            self.chunk_lengths = Self::split_into_chunks(note_length);
        }
    }

    fn ready(&self) -> bool {
        let mic_ready = self.mic.ready();
        let remaining = self.elapsed_time_goal.saturating_sub(self.elapsed_time);
        let chunk_length = self.chunk_lengths[self.chunk_index];
        return mic_ready && chunk_length <= remaining.as_millis() as u32;
    }

    fn split_into_chunks(num: u32) -> Vec<u32> {
        let max_window_length = 20;
        let k = (num + max_window_length - 1) / max_window_length;
        let remainder = num % k;
        let mut vec = vec![num / k; k as usize];
        vec[0] += remainder;
        return vec;
    }

    fn tick(&mut self) {
        match self.state {
            State::Playing => {
                self.elapsed_time_goal = self.elapsed_time_goal.saturating_add(self.last_update_time.elapsed());
                self.last_update_time = Instant::now();

                while self.ready() {
                    let current_phrase = self.backing_track.get_phrase(self.phrase_index);

                    //Check if end of song has been reached
                    match current_phrase {
                        Some(phrase) => {
                            let current_note = &phrase[self.note_index];
                            let sung_note = self.mic.consume().unwrap();
                            let difference = current_note.pitch as i32 - sung_note.pitch as i32;
                            let difference = difference % 12;
                            if self.phrase_index >= self.track.phrases.len() {
                                self.track.phrases.push(Vec::new());
                            }
                            self.track.phrases[self.phrase_index].push(sung_note);

                            self.next_chunk();
                            self.mic.set_window_length(Duration::from_millis(self.chunk_lengths[self.chunk_index].into()))
                        },
                        None => (),
                    }

                }
            },
            State::Paused => {
                self.last_update_time = Instant::now();
            },
            State::Finished => {

            }
        }
    }

    fn new(backing_track: Track) -> Option<TrackSession> {
        if backing_track.phrases.is_empty()
            || backing_track.phrases.get(0)?.is_empty() {
            return None
        }
        let initial_note = backing_track.get_phrase(0).unwrap().get(0).unwrap();
        let initial_note_length = initial_note.length;
        Some(TrackSession {
            backing_track,
            mic: Microphone::new(cpal::default_host().default_output_device().expect("")),
            track: Track::new(),
            state: State::Playing,
            elapsed_time: Duration::ZERO,
            elapsed_time_goal: Duration::ZERO,
            last_update_time: Instant::now(),
            phrase_index: 0,
            note_index: 0,
            chunk_lengths: Self::split_into_chunks(initial_note_length),
            chunk_index: 0,
        })
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.tick();
            }
        }
    }

    fn view(&self)  {
        // column![].push(
        //     Canvas::new(self)
        //     .width(Length::Fill)
        //     .height(Length::Fill)
        //     ).into()
    }

}

// impl canvas::Program<Message> for TrackSession {
//     type State = ();

//     fn draw(
//         &self,
//         _state: &Self::State,
//         _theme: &Theme,
//         bounds: Rectangle, 
//         _cursor: Cursor
//     ) -> Vec<Geometry> {
//         match self.state {
//             State::Playing => {
//                 let backing_stroke = Stroke {
//                     color: Color::new(0.5, 0.5, 0.5, 1.0),
//                     width: 5.0,
//                     ..Stroke::default()
//                 };
//                 let player_stroke = Stroke {
//                     color: Color::new(0.0, 0.2, 1.0, 1.0),
//                     width: 5.0,
//                     ..Stroke::default()
//                 };
//                 let rest_stroke = Stroke {
//                     color: Color::new(0.9, 0.9, 0.9, 1.0),
//                     width: 5.0,
//                     ..Stroke::default()
//                 };
//                 let mut frame = Frame::new(bounds.size()); 

//                 let backing_phrase = &self.backing_track.phrases[self.phrase_index];
//                 let mut length = 0;
//                 for (index, note) in backing_phrase.iter().enumerate() {
//                     let path = note_path(note.clone(), length);

//                     length += note.length;
//                     if note.voiced {
//                         frame.stroke(&path, backing_stroke);
//                     } else {
//                         frame.stroke(&path, rest_stroke);
//                     }
//                 }

//                 let singing_phrase = &self.track.phrases.get(self.phrase_index);
//                 match singing_phrase {
//                     Some(phrase) => {
//                         let mut length = 0;
//                         for note in *phrase {
//                             let path = note_path(note.clone(), length);
//                             length += note.length;
//                             if note.voiced {
//                                 frame.stroke(&path, player_stroke);
//                             } else {
//                                 frame.stroke(&path, rest_stroke);
//                             }
//                         }
//                     },
//                     None => ()
//                 }
                
//                 let lyrics : Vec<String> = backing_phrase.iter().map(|x| x.lyric.clone()).collect();
//                 let lyrics = lyrics.join(" ");
//                 let width = bounds.width;

//                 let text = Text {
//                     content: lyrics,
//                     position: Point {x: width / 2.0, y: 0.0},
//                     horizontal_alignment: Horizontal::Center,
//                     color: Color::new(0.5, 0.5, 0.7, 1.0),
//                     ..Text::default()
//                 };
//                 frame.fill_text(text);

//                 vec![frame.into_geometry()]
//             },
//             State::Paused => {
//                 Vec::new()
//             },
//             State::Finished => {
//                 Vec::new()
//             }
//         }
//     }

//     fn draw_phrase(&self, frame: Frame, _cursor: Cursor) -> Vec<Geometry> {
//         unimplemented!();
//     }
// }

// fn note_path(note: Note, length: u32) -> Path {
//     let y = (((note.pitch % 12) + 12) % 12) as f32;
//     let x = length as f32;
//     Path::line(
//         note_to_frame_transform(Point::new(x, y)),
//         note_to_frame_transform(Point::new(x + (note.length as f32), y))
//         )
// }

// fn note_to_frame_transform(point: Point) -> Point {
//     Point::new(point.x * 0.5, 100.0 - 5.0 * point.y)
// }
