use crate::note::Note;
use crate::track::SelectMode;
use crate::track::Track;
use cursive::{
    event::{Event, EventResult, Key},
    theme::{BaseColor, ColorStyle},
    traits::Resizable,
    views, Printer, Rect, XY,
};
use std::path::Path;

pub struct TrackView {
    track: Track,
}

impl TrackView {
    pub fn new(track: Track) -> TrackView {
        TrackView { track }
    }
}

impl cursive::view::View for TrackView {
    fn draw(&self, printer: &Printer) {
        let mut y: u32 = 0;
        let bg_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Black.light());
        let voice_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Blue.dark());
        let voice_color_focus = ColorStyle::new(BaseColor::White.dark(), BaseColor::Blue.dark());
        let rest_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Magenta.dark());
        let rest_color_focus = ColorStyle::new(BaseColor::White.dark(), BaseColor::Magenta.dark());
        let track = &self.track;

        for p in 0..track.phrases.len() {
            let phrase = &track.phrases[p];
            let mut x: u32 = 0;

            //draw measure markers
            for x in 0..20 {
                if x % 4 == 0 {
                    //TODO combine this into one
                    printer.with_color(rest_color, |printer| {
                        printer.print_vline((8 * x, 12 * p), 12, " ")
                    });
                } else {
                    printer.with_color(voice_color, |printer| {
                        printer.print_vline((8 * x, 12 * p), 12, " ")
                    });
                }
            }

            //draw notes
            for n in 0..phrase.len() {
                let note = &phrase[n];
                let note_y = y + (11 - note.pitch % 12) as u32;
                if track.in_selection((p, n)) {
                    printer.with_color(bg_color, |printer| {
                        for y in y..(y + 12) {
                            printer.print(
                                (x, y),
                                &String::from(" ").repeat(note.length.try_into().unwrap()),
                            )
                        }
                    });
                }
                let color = if track.in_selection((p, n)) {
                    if note.voiced {
                        voice_color_focus
                    } else {
                        rest_color_focus
                    }
                } else {
                    if note.voiced {
                        voice_color
                    } else {
                        rest_color
                    }
                };
                printer.with_color(color, |printer| {
                    let lyrics = note.lyric.to_string();
                    let length = note.length.try_into().unwrap();
                    let lyrics = pad_to_width(lyrics, length);
                    printer.print((x, note_y), &lyrics)
                });
                x += note.length;
            }
            y += 12;
        }
    }

    fn important_area(&self, _view_size: XY<usize>) -> Rect {
        let track = &self.track;
        let selection = track.get_selection_bounds();
        let start = selection.0;
        let end = selection.1;
        let corner1 = (start.1 * 8, start.0 * 12);
        let corner2 = ((end.1 - 1) * 8 + 1, end.0 * 12);
        Rect::from_corners(corner1, corner2)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let track = &mut self.track;
        match event {
            Event::CtrlChar('s') => {
                track.write(Path::new("test.song"));
            }
            Event::Key(Key::Left) => {
                track.select_prev(1);
            }
            Event::Key(Key::Right) => {
                track.select_next(1);
            }
            Event::Key(Key::Up) => match track.select_mode {
                SelectMode::Note => {
                    track.toggle_selection_mode();
                    track.select_prev(1);
                    track.toggle_selection_mode();
                }
                SelectMode::Phrase => {
                    track.select_prev(1);
                }
            },
            Event::Key(Key::Down) => match track.select_mode {
                SelectMode::Note => {
                    track.toggle_selection_mode();
                    track.select_next(1);
                    track.toggle_selection_mode();
                }
                SelectMode::Phrase => {
                    track.select_next(1);
                }
            },
            Event::Shift(Key::Left) => {
                track.contract_selection();
            }
            Event::Shift(Key::Right) => {
                track.extend_selection();
            }
            Event::Char('[') => {
                track.change_pitch(-1);
            }
            Event::Char(']') => {
                track.change_pitch(1);
            }
            Event::Char('n') => {
                track.resize_note(-1);
            }
            Event::Char('m') => {
                track.resize_note(1);
            }
            Event::Char('t') => {
                track.toggle_voiced();
            }
            Event::Char('v') => {
                track.toggle_selection_mode();
            }
            Event::Char('e') => {
                return EventResult::with_cb(|s| {
                    s.add_layer(
                        views::EditView::new()
                            .on_submit(|s, l| {
                                s.call_on_name("view", |v: &mut TrackView| {
                                    v.track.change_lyrics(&l);
                                });
                                s.pop_layer();
                            })
                            .fixed_width(20),
                    )
                })
            }
            Event::Char('a') => {
                let note = Note::new(8, 70, true, "".to_string());
                track.add_after(note);
            }
            Event::Char('i') => {
                let note = Note::new(8, 70, true, "".to_string());
                track.add_before(note);
            }
            _ => {
                return EventResult::Ignored;
            }
        }
        EventResult::Consumed(None)
    }

    fn required_size(&mut self, _constraint: XY<usize>) -> XY<usize> {
        return XY::new(200, 12 * self.track.phrases.len());
    }
}

fn pad_to_width(s: String, width: usize) -> String {
    if s.len() > width {
        s[..width].to_string()
    } else {
        let tail = &String::from(" ").repeat(width - s.len());
        s + tail
    }
}
