use std::{collections::HashMap, path::Path};

use crate::track::Track;

pub type Name = String;
pub type Artist = String;
pub type Album = String;

pub struct Song {
    pub name: Name,
    pub artist: Artist,
    pub album: Album,
    pub tracks: HashMap<String, Track>,
    pub album_cover: Option<Image>,
}

impl Song {
    pub fn new(name: Name, artist: Artist, album: Album, album_cover: Option<Image>) -> Self {
        Song {
            name,
            artist,
            album,
            tracks: HashMap::new(),
            album_cover,
        }
    }

    pub fn add_track(&mut self, name: String, track: Track) {
        self.tracks.insert(name, track);
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }
}

pub struct Image {
    pub image: Option<image::DynamicImage>,
    pub texture: Option<eframe::egui::TextureHandle>,
}

impl Image {
    pub fn new() -> Image {
        Image {
            image: None,
            texture: None,
        }
    }

    pub fn load_image(&mut self, path: &Path) {
        self.image = image::open(path).ok();
    }

    pub fn load_texture(&mut self, ctx: &eframe::egui::Context) {
        if let Some(cover) = &self.image {
            let size = [cover.width() as _, cover.height() as _];
            let buffer = cover.to_rgba8();
            let pixels = buffer.as_flat_samples();
            let color_data =
                eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.texture = Some(ctx.load_texture(
                "self.name",
                color_data,
                eframe::egui::TextureFilter::Linear,
            ));
        }
    }
}
