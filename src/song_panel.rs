use cpal::traits::HostTrait;
use eframe::egui::{self, epaint};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::mic::Microphone;
use crate::note::Note;
use crate::song::Song;
use crate::track::Track;

pub struct TrackSession {
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

    font_id: epaint::text::FontId,
}

enum Message {
    Tick,
}

enum State {
    Playing,
    Paused,
    Finished,
}

impl TrackSession {
    pub fn new(backing_track: Track) -> Option<TrackSession> {
        if backing_track.phrases.is_empty() || backing_track.phrases.get(0)?.is_empty() {
            return None;
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

            font_id: epaint::text::FontId {
                size: 16.0,
                family: epaint::FontFamily::Proportional,
            },
        })
    }

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

    pub fn tick(&mut self) {
        match self.state {
            State::Playing => {
                self.elapsed_time_goal = self
                    .elapsed_time_goal
                    .saturating_add(self.last_update_time.elapsed());
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
                            self.mic.set_window_length(Duration::from_millis(
                                self.chunk_lengths[self.chunk_index].into(),
                            ))
                        }
                        None => (),
                    }
                }
            }
            State::Paused => {
                self.last_update_time = Instant::now();
            }
            State::Finished => {}
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.tick();
            }
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::focusable_noninteractive());

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        let mut shapes = vec![];
        let backing_phrase = &self.backing_track.phrases[self.phrase_index];
        let mut length = 0;

        let backing_stroke = epaint::Stroke::new(4.0, epaint::color::Color32::GRAY);
        let rest_stroke = epaint::Stroke::new(4.0, epaint::color::Color32::WHITE);
        let player_stroke = epaint::Stroke::new(4.0, epaint::color::Color32::BLUE);

        for (index, note) in backing_phrase.iter().enumerate() {
            let path = note_path(note.clone(), length)
                .iter()
                .map(|(x, y)| egui::Pos2 { x: *x, y: *y })
                .collect();

            length += note.length;
            if note.voiced {
                shapes.push(egui::Shape::line(path, backing_stroke));
            } else {
                shapes.push(egui::Shape::line(path, rest_stroke));
            }
        }

        let singing_phrase = &self.track.phrases.get(self.phrase_index);
        match singing_phrase {
            Some(phrase) => {
                let mut length = 0;
                for note in *phrase {
                    let path = note_path(note.clone(), length)
                        .iter()
                        .map(|(x, y)| egui::Pos2 { x: *x, y: *y })
                        .collect();
                    length += note.length;
                    if note.voiced {
                        shapes.push(egui::Shape::line(path, player_stroke));
                    } else {
                        shapes.push(egui::Shape::line(path, rest_stroke));
                    }
                }
            }
            None => (),
        }

        let mut lyrics: Vec<Arc<epaint::text::Galley>> = vec![];

        for (i, note) in backing_phrase.iter().enumerate() {
            if i == self.note_index {
                lyrics.push(ui.fonts().layout_no_wrap(
                    note.lyric.clone(),
                    self.font_id.clone(),
                    epaint::color::Color32::BLUE,
                ));
            } else {
                lyrics.push(ui.fonts().layout_no_wrap(
                    note.lyric.clone(),
                    self.font_id.clone(),
                    epaint::color::Color32::BLUE,
                ));
            }
        }

        let text_positions = set_text_in_line(egui::Pos2::ZERO, &lyrics);
        let partial_shapes: Vec<(&egui::Pos2, &Arc<epaint::text::Galley>)> =
            text_positions.iter().zip(lyrics.iter()).collect();
        for (text_position, lyric) in partial_shapes.iter() {
            shapes.push(egui::Shape::Text(epaint::TextShape {
                pos: **text_position,
                galley: (*lyric).clone(),
                underline: epaint::Stroke::default(),
                override_text_color: None,
                angle: 0.0,
            }));
        }

        painter.extend(shapes);
        response
    }
}

fn set_text_in_line(pos: egui::Pos2, lyrics: &Vec<Arc<epaint::text::Galley>>) -> Vec<egui::Pos2> {
    let mut current_pos = pos;
    let mut resulting_positions = vec![];
    resulting_positions.push(current_pos);
    for lyric in lyrics {
        current_pos = egui::Pos2 {
            x: lyric.size().x,
            y: pos.y,
        };
        resulting_positions.push(current_pos);
    }
    resulting_positions.pop();
    resulting_positions
}

fn note_path(note: Note, length: u32) -> Vec<(f32, f32)> {
    let y = (((note.pitch % 12) + 12) % 12) as f32;
    let x = length as f32;
    let mut note_path = vec![];
    note_path.push(note_to_frame_transform((x, y)));
    note_path.push(note_to_frame_transform((x + (note.length as f32), y)));
    note_path
}

fn note_to_frame_transform((x, y): (f32, f32)) -> (f32, f32) {
    (x * 0.5, 100.0 - 5.0 * y)
}
