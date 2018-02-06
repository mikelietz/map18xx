#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate svg;

use std::fs::OpenOptions;
use std::process;

pub mod draw;
pub mod game;
pub mod tile;

/// Place to store command line options
pub struct Options {
    pub verbose: bool,
    pub debug_coordinates: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            verbose: false,
            debug_coordinates: false,
        }
    }
}

pub struct AssetOptions {
    pub name: String,
}

impl AssetOptions {
    pub fn new() -> AssetOptions {
        AssetOptions {
            name: String::new(),
        }
    }
}

pub struct NewGameOptions {
    pub game: String,
    pub name: String,
}

impl NewGameOptions {
    pub fn new() -> NewGameOptions {
        NewGameOptions {
            game: String::new(),
            name: String::new(),
        }
    }
}

pub fn definitions(options: &Options) {
    let definitions = tile::definitions(options);
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm", (definitions.len() as f64/4.0).ceil()*42.0+0.0))
        .add(draw::draw_tile_definitions(&definitions));
    svg::save("definitions.svg", &document).unwrap_or_else(|err| {
        eprintln!("Failed to write definitions.svg: {:?}", err.kind());
        process::exit(1);
    });
}

pub fn asset_mode(options: &Options, asset_options: &AssetOptions) {
    println!("Processing game '{}'", asset_options.name);
    let definitions = tile::definitions(options);
    let game = game::Game::load(["games", asset_options.name.as_str()]
                                    .iter().collect(),
                                &definitions);

    println!("Exporting tile manifest...");
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm",
                     (game.manifest.tiles.len() as f64/3.0).ceil()*30.0+3.0))
        .add(draw::draw_tile_manifest(&game));
    svg::save(format!("{}-manifest.svg", asset_options.name), &document)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-manifest.svg: {:?}",
                      asset_options.name, err.kind());
            process::exit(1);
    });

    println!("Exporting tile sheets...");
    let sheets = draw::draw_tile_sheets(&game);
    for (i, sheet) in sheets.iter().enumerate() {
        let filename = format!("{}-sheet-{}.svg", asset_options.name, i);
        svg::save(filename, sheet).unwrap_or_else(|err| {
            eprintln!("Failed to write {}-sheet-{}.svg: {:?}",
                      asset_options.name, i, err.kind());
            process::exit(1);
        });
    }

    println!("Exporting map...");
    let map_render = draw::draw_map(&game.map, &options);
    svg::save(format!("{}-map.svg", asset_options.name), &map_render)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-map.svg: {:?}", asset_options.name,
                      err.kind());
            process::exit(1);
    });
}

pub fn newgame_mode(_options: &Options, newgame_options: &NewGameOptions) {
    println!("Starting new game of {}", newgame_options.game);
    let log = game::Log::new_game(newgame_options.game.clone());
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}.json", newgame_options.name));
    match file {
        Err(e) => {
            eprintln!("Couldn't create game file {}.json: {}",
                      newgame_options.name, e);
            process::exit(1);
        }
        Ok(file) => {
            println!("Writing to {}.json", newgame_options.name);
            serde_json::to_writer_pretty(file, &log).unwrap();
        }
    }
}
