use eframe::egui;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{frame_splitter::FrameSplitter, track::Track};

pub type Name = String;
pub type Artist = String;
pub type Album = String;

#[derive(Clone)]
pub struct Song {
    pub name: Name,
    pub artist: Artist,
    pub album: Album,
    pub tracks: HashMap<String, Track>,
    pub album_cover: Option<Image>,
    pub video_path: Option<PathBuf>,
}

impl Song {
    pub fn new(
        name: Name,
        artist: Artist,
        album: Album,
        album_cover: Option<Image>,
        video_path: Option<PathBuf>,
    ) -> Self {
        Song {
            name,
            artist,
            album,
            tracks: HashMap::new(),
            album_cover,
            video_path,
        }
    }

    pub fn add_track(&mut self, name: String, track: Track) {
        self.tracks.insert(name, track);
    }

    pub fn num_tracks(&self) -> usize {
        self.tracks.len()
    }
}

#[derive(Clone)]
pub struct Image {
    pub image: Option<image::DynamicImage>,
    pub texture: Option<egui::TextureHandle>,
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

    pub fn load_rgb_image_from_memory(
        &mut self,
        ctx: &egui::Context,
        width: usize,
        height: usize,
        data: Vec<u8>,
    ) {
        let rgba_data = data;
        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba_data);
        self.texture = Some(ctx.load_texture("PLACEHOLDER", image, egui::TextureFilter::Nearest));
    }

    pub fn load_texture(&mut self, ctx: &egui::Context) {
        if let Some(cover) = &self.image {
            let size = [cover.width() as _, cover.height() as _];
            let buffer = cover.to_rgba8();
            let pixels = buffer.as_flat_samples();
            let color_data = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            self.texture =
                Some(ctx.load_texture("PLACEHOLDER", color_data, egui::TextureFilter::Linear));
        }
    }
}
