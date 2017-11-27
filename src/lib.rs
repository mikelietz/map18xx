extern crate argparse;
#[macro_use]
extern crate serde_derive;
extern crate svg;

use argparse::ArgumentParser;

pub mod draw;
pub mod game;
pub mod map;
pub mod tile;

/// Orientation that hexes should be in
pub enum Orientation {
    /// Hexes should have a flat top
    Horizontal,
    /// Hexes should have apoint at the top
    Vertical,
}

/// Place to store command line options
struct Options {
    game: Option<String>,
}

impl Options {
    pub fn new() -> Options {
        Options {
            game: None,
        }
    }
}

pub fn run() {
    let mut options = Options::new();
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description(
            "18xx tile and map designer. Will generate definitions.svg when no
            mode arguments given. Game mode can be used with --game.");
        parser.add_option(&["-V", "--version"],
                          argparse::Print(env!("CARGO_PKG_VERSION")
                                          .to_string()),
                          "Show version");
        parser.refer(&mut options.game)
            .add_option(&["-g", "--game"],
                        argparse::StoreOption,
                        "Generate files for a game map")
            .metavar("MAP");
        parser.parse_args_or_exit();
    }

    if let &Some(ref name) = &options.game {
        game_mode(name, &options);
    } else {
        definitions();
    }
}

fn definitions() {
    let definitions = tile::definitions();
    let info = map::MapInfo::default();
    let document = svg::Document::new()
        .set("width", 11.5 * info.scale)
        .set("height",
             2.0 * info.scale * (definitions.len() as f64 / 5.0).ceil())
        .add(draw::draw_tile_definitions(&definitions));
    svg::save("definitions.svg", &document).unwrap();
}

fn game_mode(name: &String, _options: &Options) {
    println!("Processing map '{}'", name);
    let _game = game::Game::new()
        .set_directory(["games", name.as_str()].iter().collect());
}
