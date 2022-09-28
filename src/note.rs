#[derive(Debug)]
pub struct Note {
	pub length: u32,
	pub pitch: i8,
	pub voiced: bool,
	pub lyric: String,
}

impl Note {
	pub fn new(length: u32, pitch: i8, voiced: bool, lyric: String) -> Note {
		Note {
			length,
			pitch,
			voiced, 
			lyric,
		}
	}
}
