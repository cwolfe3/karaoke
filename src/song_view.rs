use std::path::Path;
use crate::song::Song;
use crate::song::SelectMode;
use crate::song::Note;
use cursive::{
    Printer,
    XY,
    Rect,
    views,
    event::{Event, EventResult, Key},
    theme::{BaseColor, ColorStyle},
    traits::Resizable,
};

pub struct SongView {
    song: Song,
}

impl SongView {
    pub fn new(song: Song) -> SongView {
        SongView {
            song,
        }
    }
}

impl cursive::view::View for SongView {
    fn draw(&self, printer: &Printer) {
        let mut y: u32 = 0;
        let bg_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Black.light());
        let voice_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Blue.dark());
        let voice_color_focus = ColorStyle::new(BaseColor::White.dark(), BaseColor::Blue.dark());
        let rest_color = ColorStyle::new(BaseColor::White.dark(), BaseColor::Magenta.dark());
        let rest_color_focus = ColorStyle::new(BaseColor::White.dark(), BaseColor::Magenta.dark());

        for p in 0..self.song.phrases.len() {
            let phrase = &self.song.phrases[p];
            let mut x: u32 = 0;

            //draw measure markers
            for x in 0..20 {
                if x % 4 == 0 { //TODO combine this into one
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
                if self.song.in_selection((p, n)) {
                    printer.with_color(
                        bg_color,
                        |printer| {
                            for y in y..(y + 12) {
                                printer.print((x, y), &String::from(" ").repeat(note.length.try_into().unwrap()))
                            }
                        },
                    );
                }
                let color = if self.song.in_selection((p, n)) {
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
                printer.with_color(
                    color,
                    |printer| {
                        let lyrics = note.lyric.to_string();
                        let length = note.length.try_into().unwrap();
                        let lyrics = pad_to_width(lyrics, length);
                        printer.print((x, note_y), &lyrics)},
                );
                x += note.length;
            }
            y += 12;

        }
    }

    fn important_area(&self, _view_size: XY<usize>) -> Rect {
        let selection = self.song.get_selection_bounds();
        let start = selection.0;
        let end = selection.1;
        let corner1 = (start.1 * 8, start.0 * 12);
        let corner2 = ((end.1 - 1) * 8 + 1, end.0 * 12);
        Rect::from_corners(corner1, corner2)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::CtrlChar('s') => {
                self.song.write(Path::new("test.song"));
            }
            Event::Key(Key::Left) => {
                self.song.select_prev(1);
            }
            Event::Key(Key::Right) => {
                self.song.select_next(1);
            }
            Event::Key(Key::Up) => {
                match self.song.select_mode {
                    SelectMode::Note => {
                        self.song.toggle_selection_mode();
                        self.song.select_prev(1);
                        self.song.toggle_selection_mode();
                    }
                    SelectMode::Phrase => {
                        self.song.select_prev(1);
                    }
                }
            }
            Event::Key(Key::Down) => {
                match self.song.select_mode {
                    SelectMode::Note => {
                        self.song.toggle_selection_mode();
                        self.song.select_next(1);
                        self.song.toggle_selection_mode();
                    }
                    SelectMode::Phrase => {
                        self.song.select_next(1);
                    }
                }
            }
            Event::Shift(Key::Left) => {
                self.song.contract_selection();
            }
            Event::Shift(Key::Right) => {
                self.song.extend_selection();
            }
            Event::Char('[') => {
                self.song.change_pitch(-1);
            }
            Event::Char(']') => {
                self.song.change_pitch(1);
            }
            Event::Char('n') => {
                self.song.resize_note(-1);
            }
            Event::Char('m') => {
                self.song.resize_note(1);
            }
            Event::Char('t') => {
                self.song.toggle_voiced();
            }
            Event::Char('v') => {
                self.song.toggle_selection_mode();
            }
            Event::Char('e') => {
                return EventResult::with_cb(|s| {
                    s.add_layer(
                        views::EditView::new().on_submit(|s, l| {
                            s.call_on_name("view", |v: &mut SongView| {
                                v.song.change_lyrics(&l);
                            });
                            s.pop_layer();
                        }).fixed_width(20)
                    )
                })
            }
            Event::Char('a') => {
                let note = Note::new(8, 70, true, "".to_string());
                self.song.add_after(note);
            }
            Event::Char('i') => {
                let note = Note::new(8, 70, true, "".to_string());
                self.song.add_before(note);
            }
            _ => {
                return EventResult::Ignored;
            }
        }
        EventResult::Consumed(None)
    }

    fn required_size(&mut self, _constraint: XY<usize>) -> XY<usize> {
        return XY::new(200, 12 * self.song.phrases.len());
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

