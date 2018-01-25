extern crate argparse;
#[macro_use]
extern crate serde_derive;
extern crate svg;

use argparse::ArgumentParser;
use std::io::{stdout, stderr};
use std::process;

pub mod draw;
pub mod game;
pub mod tile;

/// Place to store command line options
pub struct Options {
    mode: String,
    verbose: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            mode: String::from("definitions"),
            verbose: false,
        }
    }
}

struct AssetOptions {
    name: String,
}

impl AssetOptions {
    pub fn new() -> AssetOptions {
        AssetOptions {
            name: String::new(),
        }
    }
}

pub fn run() {
    let mut options = Options::new();
    let mut args = vec![];
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description("18xx tile and map designer.");
        parser.add_option(&["-V", "--version"],
                          argparse::Print(env!("CARGO_PKG_VERSION")
                                          .to_string()),
                          "Show version");
        parser.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue,
                        "Print debug information");
        parser.refer(&mut options.mode)
            .add_argument("mode",
                          argparse::Store,
                          "Mode to use (default: definitions)");
        parser.refer(&mut args)
            .add_argument("args",
                          argparse::List,
                          "Arguments for mode");
        parser.stop_on_first_argument(true);
        parser.parse_args_or_exit();
    }

    match options.mode.as_ref() {
        "d" | "def" | "definitions" => definitions(&options),
        "a" | "asset" | "assets" => {
            args.insert(0, String::from("game"));
            asset_mode(&options, args)
        }
        m => {
            println!("Unrecognized mode '{}'. See 'map18xx --help'", m);
            process::exit(1);
        }
    }
}

fn definitions(options: &Options) {
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

fn asset_mode(options: &Options, args: Vec<String>) {
    let mut asset_options = AssetOptions::new();
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description("Game mode");
        parser.refer(&mut asset_options.name).required()
            .add_argument("name",
                          argparse::Store,
                          "Game for which to generate files");
        match parser.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => process::exit(x),
        }
    }

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
    let map_render = draw::draw_map(&game);
    svg::save(format!("{}-map.svg", asset_options.name), &map_render)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-map.svg: {:?}", asset_options.name,
                      err.kind());
            process::exit(1);
    });
}
