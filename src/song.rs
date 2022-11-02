use std::collections::HashMap;

use crate::track::Track;

pub type Name = String;
pub type Artist = String;
pub type Album = String;

pub struct Song {
    pub name: Name,
    pub artist: Artist,
    pub album: Album,
    pub tracks: HashMap<String, Track>,
}

impl Song {
    pub fn new(name: Name, artist: Artist, album: Album) -> Self {
        Song {
            name,
            artist,
            album,
            tracks: HashMap::new(),
        }
    }

    pub fn add_track(&mut self, name: String, track: Track) {
        self.tracks.insert(name, track);
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }
}
