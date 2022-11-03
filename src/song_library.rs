use std::path::Path;
use std::fs::{File, DirEntry};

use crate::song::Song;
use crate::track::Track;

pub struct SongLibrary {
    pub songs: Vec<Song>,
    pub selection_index: usize,
}

impl SongLibrary {
    pub fn read_songs(path: &Path) -> SongLibrary {
        let path = Path::new(path);
        let mut library = SongLibrary {
            songs: Vec::new(),
            selection_index: 0,
        };

        match path.read_dir() {
            Ok(dir_iter) => {
                for dir in dir_iter {
                    match dir {
                        Ok(dir_entry_ok) => {
                            library.read_song(dir_entry_ok.path().as_path());
                        },
                        Err(_) => (),
                    }
                }
                library
            },
            Err(_) => {
                library
            }
        }
    }

    fn read_song(&mut self, path: &Path) {
       match path.read_dir() {
           Ok(dir_iter) => {
                let mut song = Song::new("SONG NAME PLACEHOLDER".to_string(), 
                                         "ARTIST NAME PLACEHOLDER".to_string(), 
                                         "ALBUM NAME PLACEHOLDER".to_string());
                for dir in dir_iter {
                    match dir {
                        Ok(dir_entry_ok) => {
                            let path = dir_entry_ok.path();
                            let extension = path.extension();
                            match extension {
                                Some(s) => {
                                    if s.to_string_lossy() == "track" {
                                        let track = Track::read(path.as_path());
                                        song.add_track("track 1".to_string(), track);
                                    }
                                },
                                None => ()
                            }
                        },
                        Err(_) => return ()
                    }
                }
                if song.num_tracks() > 0 {
                    self.songs.push(song);
                }
           },
           Err(_) => ()
       }
    }

    // TODO Handle the case of empty libraries
    pub fn select_next(&mut self) {
        self.selection_index += 1;
        if self.selection_index >= self.songs.len() {
            self.selection_index = 0;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selection_index > 0 {
            self.selection_index -= 1;
        } else {
            self.selection_index = self.songs.len() - 1;
        }
    }

    pub fn select(&mut self, i: usize) {
        if i >= self.songs.len() {
            self.selection_index = self.songs.len() - 1;
        } else {
            self.selection_index = i;
        }
    }
}
