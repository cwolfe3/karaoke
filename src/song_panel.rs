use cpal::traits::HostTrait;
use eframe::egui::{self, epaint};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::frame_splitter::FrameSplitter;
use crate::mic::Microphone;
use crate::note::Note;
use crate::song::{Image, Song};
use crate::track::Track;

pub struct TrackSession {
    song: Song,
    mic: Microphone,
    track: Track,
    pub state: State,
    frame_splitter: Option<FrameSplitter>,
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

pub enum State {
    Playing,
    Paused,
    Finished,
}

impl TrackSession {
    pub fn new(song: Song) -> Result<TrackSession, std::io::Error> {
        let backing_track = song.tracks.values().next().unwrap();
        let initial_note = backing_track.get_phrase(0).unwrap().get(0).unwrap();
        let initial_note_length = initial_note.length;
        let video_path = song.video_path.clone();
        Ok(TrackSession {
            song,
            mic: Microphone::new(cpal::default_host().default_output_device().expect("")),
            track: Track::new(),
            state: State::Playing,
            frame_splitter: Some(FrameSplitter::new(
                &video_path.expect("No video found").to_path_buf(),
            )?),
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

    fn finish(&mut self) {
        self.mic.pause();
        self.state = State::Finished;
    }

    fn next_chunk(&mut self) {
        let backing_track = self.song.tracks.values().next().unwrap();
        self.chunk_index += 1;
        if self.chunk_index >= self.chunk_lengths.len() {
            self.chunk_index = 0;
            self.note_index += 1;
            if self.note_index >= backing_track.phrases[self.phrase_index].len() {
                self.note_index = 0;
                self.phrase_index += 1;
                if self.phrase_index >= backing_track.phrases.len() {
                    self.finish();
                    return;
                }
            }
            let note_length = backing_track.phrases[self.phrase_index][self.note_index].length;
            self.chunk_lengths = Self::split_into_chunks(note_length);
        }
    }

    fn ready(&self) -> bool {
        let mic_ready = self.mic.ready();
        let remaining = self.elapsed_time_goal.saturating_sub(self.elapsed_time);
        println!("elapsed time: {:?}, elasped time goal: {:?}", self.elapsed_time, self.elapsed_time_goal);
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
        let backing_track = self.song.tracks.values().next().unwrap().clone();
        match self.state {
            State::Playing => {
                self.elapsed_time_goal = self
                    .elapsed_time_goal
                    .saturating_add(self.last_update_time.elapsed());
                self.last_update_time = Instant::now();

                while self.ready() {
                    let current_phrase = backing_track.get_phrase(self.phrase_index);

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
                        None => break,
                    }
                }
                println!("DONE");
            }
            State::Paused => {
                self.last_update_time = Instant::now();
            }
            State::Finished => {}
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let backing_track = self.song.tracks.values().next().unwrap();
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::focusable_noninteractive());

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();
        let screen_width = ui.ctx().input().screen_rect().width();
        let screen_height = ui.ctx().input().screen_rect().height();

        let frame_splitter = self.frame_splitter.as_mut().unwrap();
        let frame = frame_splitter.next_frame();
        let mut video_frame = Image::new();
        video_frame.load_rgb_image_from_memory(
            ui.ctx(),
            frame_splitter.width,
            frame_splitter.height,
            frame,
        );
        let mut mesh = egui::Mesh::with_texture(video_frame.texture.as_ref().unwrap().id());
        mesh.add_rect_with_uv(
            egui::Rect::from_two_pos(
                egui::pos2(0.0, 0.0),
                egui::pos2(screen_width, screen_height),
            ),
            egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            epaint::Color32::WHITE,
        );
        painter.add(egui::Shape::mesh(mesh));

        // ui.image(
        //     video_frame.texture.as_ref().unwrap().id(),
        //     egui::vec2(screen_width, screen_height),
        // );

        let mut shapes = vec![];
        let backing_phrase = &backing_track.phrases[self.phrase_index];
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
                    note.lyric.clone() + " ",
                    self.font_id.clone(),
                    epaint::color::Color32::BLUE,
                ));
            } else {
                lyrics.push(ui.fonts().layout_no_wrap(
                    note.lyric.clone() + " ",
                    self.font_id.clone(),
                    epaint::color::Color32::WHITE,
                ));
            }
        }

        let lyric_widths: Vec<f32> = lyrics.iter().map(|lyric| lyric.size().x).collect();
        let total_width = lyric_widths.iter().fold(0.0, |x, y| x + y);
        let mut current_x = (ui.available_width() - total_width) / 2.0;
        let current_y = lyrics.get(0).unwrap().size().y / 2.0 + 10.0;
        for lyric in lyrics {
            shapes.push(egui::Shape::Text(epaint::TextShape {
                pos: egui::Pos2 {
                    x: current_x,
                    y: current_y,
                },
                galley: lyric.clone(),
                underline: epaint::Stroke::default(),
                override_text_color: None,
                angle: 0.0,
            }));
            current_x += lyric.size().x;
        }

        painter.extend(shapes);
        response
    }
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
