//use std::env;

mod song;
mod song_view;
mod mic;

//use std::path::Path;
//use song::Song;
//use song_view::SongView;
//use cursive::views;
//use cursive::traits::Nameable;

fn main() {
    mic::test();
    //let song = Song::read(Path::new("test.song"));

    //let mut siv = cursive::default();
    //siv.add_global_callback('q', |s| s.quit());
    //siv.add_layer(views::ScrollView::new(SongView::new(song).with_name("view")).scroll_x(true));
    //siv.run();
}
