use std::path::Path;

use crate::song::{self, Song};
use crate::track::Track;

pub struct SongLibrary {
    pub songs: Vec<Song>,
    pub selection_index: usize,
}

impl SongLibrary {
    pub fn new(path: &Path) -> SongLibrary {
        let songs = SongLibrary::read_songs(path);
        SongLibrary {
            songs,
            selection_index: 0,
        }
    }

    pub fn read_songs(path: &Path) -> Vec<Song> {
        let path = Path::new(path);
        let mut songs = Vec::new();

        if let Ok(dir_iter) = path.read_dir() {
            for dir in dir_iter {
                if let Ok(dir_entry_ok) = dir {
                    if let Ok(song) = SongLibrary::read_song(dir_entry_ok.path().as_path()) {
                        songs.push(song);
                    }
                }
            }
        }
        songs
    }

    fn read_song(path: &Path) -> Result<Song, std::io::Error> {
        let dir_iter = path.read_dir()?;
        let mut tracks: Vec<Track> = vec![];
        let mut img = None;
        for dir in dir_iter {
            let path = dir?.path();
            let extension = path.extension();
            match extension {
                Some(s) => {
                    let ext = s.to_string_lossy();
                    if ext == "track" {
                        let track = Track::read(path.as_path());
                        match track {
                            Ok(t) => tracks.push(t),
                            Err(_) => continue,
                        }
                    } else if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        // TODO defer image loading until needed
                        // very slow on debug target
                        let mut unloaded_image = song::Image::new();
                        unloaded_image.load_image(&path);
                        img = Some(unloaded_image);
                    }
                }
                None => (),
            }
        }
        if tracks.len() > 0 {
            // TODO load song metadata from fs
            let mut song = Song::new(
                tracks.get(0).unwrap().name.clone(),
                "ARTIST NAME PLACEHOLDER".to_string(),
                "ALBUM NAME PLACEHOLDER".to_string(),
                img,
            );
            for track in tracks {
                song.add_track(track.name.clone(), track);
            }
            Ok(song)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not read track information",
            ))
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
