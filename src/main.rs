//use std::env;

mod note;
mod song;
mod song_view;
mod mic;
mod song_panel;

//use std::path::Path;
//use song::Song;
//use song_view::SongView;
//use cursive::views;
//use cursive::traits::Nameable;

fn main() {
    song_panel::main();
	//let song = Song::read(Path::new("test.song"));

	//let mut siv = cursive::default();
	//siv.add_global_callback('q', |s| s.quit());
	//siv.add_layer(views::ScrollView::new(SongView::new(song).with_name("view")).scroll_x(true));
	//siv.run();
}
