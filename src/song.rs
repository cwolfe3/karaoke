use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::cmp;

use crate::note::Note;

pub type Phrase = Vec<Note>;
pub type NoteIndex = (usize, usize);

pub struct Song {
	pub name: String,
	pub artist: String,
	pub album: String,
	pub phrases: Vec<Phrase>,
	pub select_mode: SelectMode,
	pub select_begin: NoteIndex,
	pub select_end: NoteIndex,
	seek_pos: u32,
	seek_note_index: NoteIndex,
}

pub enum SelectMode {
	Note,
	Phrase,
}

impl Song {
	pub fn new(name: String, artist: String, album: String) -> Song {
		Song {
			name,
			artist,
			album,
			phrases: Vec::new(),
			select_mode: SelectMode::Note,
			select_begin: (0, 0),
			select_end: (1, 1),
			seek_pos: 0,
			seek_note_index: (0, 0),
		}
	}

	pub fn add_before(&mut self, note: Note) {
		let (p, n) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
			    if self.phrases.len() == 0 {
			        self.phrases.push(Vec::new());
			    }
				if self.phrases[p].len() == 0 {
					self.phrases[p].push(note);
					self.select_end.1 = n + 1
				} else {
					self.phrases[p].insert(n, note);
				}
			}
			SelectMode::Phrase => {
				self.phrases[p].push(note);
				self.select_begin.0 += 1;
				self.select_end.0 += 1;
			}
		}
	}

	pub fn add_after(&mut self, note: Note) {
		let (p, n) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
			    if self.phrases.len() == 0 {
			        self.phrases.push(Vec::new());
			    }
				if self.phrases[p].len() == 0 || n == self.phrases[p].len() {
					self.phrases[p].push(note);
				} else {
					self.phrases[p].insert(n + 1, note);
				}
				self.select_begin = (p, n + 1);
				self.select_end = (p + 1, n + 2);
			}
			SelectMode::Phrase => {
				if self.phrases.len() == 0 {
					self.phrases.push(Vec::new());
					self.phrases[0].push(note);
					self.select_begin = (0, 0);
					self.select_end = (1, 1);
				} else {
					self.phrases.insert(p + 1, Vec::new());
					self.phrases[p + 1].push(note);
					self.select_begin = (p + 1, n);
					self.select_end = (p + 2, n + 1);
				}
			}
		}
	}

	pub fn select_prev(&mut self, delta: usize) {
		let (_, n) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
				self.select_begin.1 = if self.select_begin.1 > delta {
					self.select_begin.1 - delta
				} else {
					0
				};
				let (p, n) = self.select_begin;
				self.select_end = (p + 1, n + 1);
			}
			SelectMode::Phrase => {
				let p = if self.select_begin.0 > delta {
					self.select_begin.0 - delta
				} else {
					0
				};
				self.select_begin.0 = p;
				self.select_begin.1 = cmp::min(n, self.phrases[p].len() - 1);
				let (p, n) = self.select_begin;
				self.select_end = if self.phrases[p].len() > 0 {
					(p + 1, n + 1)
				} else {
					(p + 1, n)
				}
			}
		}
	}

	pub fn select_next(&mut self, delta:usize) {
		let (p, n) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
				self.select_begin.1 = cmp::min(n + delta,
											   self.phrases[p].len() - 1);
				let (p, n) = self.select_begin;
				self.select_end = (p + 1, n + 1);
			}
			SelectMode::Phrase => {
				let p = cmp::min(p + delta, self.phrases.len() - 1);
				self.select_begin.0 = p;
				self.select_begin.1 = cmp::min(n, self.phrases[p].len() - 1);
				let (p, n) = self.select_begin;
				self.select_end = if self.phrases[p].len() > 0 {
					(p + 1, n + 1)
				} else {
					(p + 1, n)
				}
			}
		}
	}

	pub fn extend_selection(&mut self) {
		let (p, _) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
				if self.phrases[p].len() > 0 {
					self.select_end.1 = cmp::min(self.select_end.1 + 1, 
												 self.phrases[p].len());
				}
			}
			SelectMode::Phrase => {
				self.select_end.0 = cmp::min(self.select_end.0 + 1, 
											 self.phrases.len());
			}
		}
	}

	pub fn contract_selection(&mut self) {
		let (p, n) = self.select_begin;
		match self.select_mode {
			SelectMode::Note => {
				self.select_end.1 = cmp::max(self.select_end.1 - 1, n + 1);
			}
			SelectMode::Phrase => {
				self.select_end.0 = cmp::max(self.select_end.0 - 1, p + 1);
			}
		}
	}

	pub fn change_pitch(&mut self, pitch: i8) {
		self.apply_to_selection(&mut |note: &mut Note| {
			note.pitch += pitch;
		});
	}

	pub fn resize_note(&mut self, delta: i64) {
		self.apply_to_selection(&mut |note: &mut Note| {
			let new_length = delta + note.length as i64;
			let new_length = cmp::max(new_length, 0);
			note.length = new_length as u32;
		});
	}

	pub fn toggle_voiced(&mut self) {
		self.apply_to_selection(&mut |note: &mut Note| {
			note.voiced = !note.voiced;
		});
	}

	pub fn change_lyrics(&mut self, lyric: &str) {
		self.apply_to_selection(&mut |note: &mut Note| {
			note.lyric = lyric.to_string();
		});
	}

	pub fn apply_to_selection(&mut self, f: &mut dyn FnMut(&mut Note) ) {
		match self.select_mode {
			SelectMode::Phrase => {
				for p in self.select_begin.0..self.select_end.0 {
					for note in &mut self.phrases[p] {
						f(note);
					}
				}
			}
			SelectMode::Note => {
				let mut p = self.select_begin.0;
				let mut n = self.select_begin.1;
				while p < self.select_end.0 - 1 {
					while n < self.phrases[p].len() {
						f(&mut self.phrases[p][n]);
					}
					n = 0;
					p += 1;
				}
				while n < self.select_end.1 {
					f(&mut self.phrases[p][n]);
					n += 1;
				}
			}
		}
	}

	pub fn toggle_selection_mode(&mut self) {
		match self.select_mode {
			SelectMode::Phrase => {
				self.select_mode = SelectMode::Note;
				let sb = self.select_begin;
				let n = cmp::min(sb.1, self.phrases[sb.0].len() - 1);
				self.select_begin = (sb.0, n);
				self.select_end = (sb.0 + 1, n + 1);
			}
			SelectMode::Note => {
				self.select_mode = SelectMode::Phrase;
			}
		};
	}

	pub fn in_selection(&self, n: NoteIndex) -> bool {
		let (pb, nb) = self.select_begin;
		let (pe, ne) = self.select_end;
		match self.select_mode {
			SelectMode::Note => {
				pb <= n.0 && n.0 < pe && nb <= n.1 && n.1 < ne
			}
			SelectMode::Phrase => {
				pb <= n.0 && n.0 < pe
			}
		}
	}

	pub fn get_selection_bounds(&self) -> (NoteIndex, NoteIndex) {
		(self.select_begin, self.select_end)
	}

	pub fn read(path: &Path) -> Song {
		let display = path.display();
		let mut file = match File::open(&path) {
			Err(why) => panic!("Couldn't open {}: {}", display, why),
			Ok(file) => file,
		};
		let mut s = String::new();
		match file.read_to_string(&mut s) {
			Err(why) => panic!("Couldn't read {}: {}", display, why),
			Ok(_) => println!("Read from {}", display),
		}

		let mut song = Song::new(String::new(), String::new(), String::new());
		let phrases: Vec<&str> = s.split('\n').collect();
		for phrase in phrases {
			let notes = phrase.split('|');
			let mut first = true;
			for note in notes {
				let fields : Vec<&str> = note.split(':').collect();
				if fields.len() < 4 {
					continue;
				}
				let voiced = if fields[0] == "u" {
					false
				} else {
					true
				};
				let pitch = fields[1].parse::<i8>().unwrap();
				let length = fields[2].parse::<u32>().unwrap();
				let lyric = fields[3].to_string();
				let note = Note::new(length, pitch, voiced, lyric);
				if first {
					song.toggle_selection_mode();
					song.add_after(note);
					song.toggle_selection_mode();
					first = false;
				} else {
					song.add_after(note);
				}
			}
		}
		song
	}

	pub fn write(&self, path: &Path) {
		let display = path.display();
		let mut file = match File::create(&path) {
			Err(why) => panic!("Couldn't create {}: {}", display, why),
			Ok(file) => file,
		};
		let mut s = String::new();
		for phrase in &self.phrases {
			for note in phrase {
				let voiced = if note.voiced {
					"v"
				} else {
					"u"
				};
				s += &format!("{}:{}:{}:{}|", 
							  voiced, 
							  note.pitch, 
							  note.length, 
							  note.lyric);
				}
			s += "\n";
		}
		match file.write_all(s.as_bytes()) {
			Err(why) => panic!("Couldn't write {}: {}", display, why),
			Ok(_) => println!("Wrote to {}", display),
		}
	}

    pub fn get_phrase(&self, i: usize) -> Option<&Phrase> {
        self.phrases.get(i)
    }
}
