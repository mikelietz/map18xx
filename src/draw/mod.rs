extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::ops::Deref;
use std::process;
use self::svg::node::element::Group;
use tile;
use tile::TileSpec;
use game;
use self::na::Vector2;
use game::Orientation;

mod helpers;
mod consts;
mod element;

const TILES_PER_ROW: f64 = 4.0;

/// Draws tile definitions
pub fn draw_tile_definitions(
        definitions: &HashMap<String, tile::TileDefinition>) -> Group {
    println!("Drawing tile definitions...");
    let mut g = Group::new();
    let mut i = 0.0;
    let info = game::Map::default();

    let mut keys: Vec<_> = definitions.keys().collect();
    keys.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
    for name in keys {
        let definition = &definitions[name];
        let pos = Vector2::new(1.1_f64 + 2.25 * (i % TILES_PER_ROW),
                                   1.0 + 2.0 * (i / TILES_PER_ROW).floor());
        let text_pos = helpers::scale(&info)
            * Vector2::new(pos.x - 1.0, pos.y - 0.7);
        g = g.add(draw_tile(definition, &pos, &info))
            .add(element::draw_text(&name, &text_pos,
                                    &tile::TextAnchor::Start, None, None));
        i += 1.0;
    }
    g
}

/// Draw a game's tile manifest
pub fn draw_tile_manifest(game: &game::Game) -> Group {
    let mut g = Group::new();
    let mut i = 0.0;

    for tile in &game.manifest.tiles {
        let pos = Vector2::new(1.1_f64 + 2.25 * (i % TILES_PER_ROW),
                                   1.0 + 2.0 * (i / TILES_PER_ROW).floor());
        g = g.add(draw_tile(tile, &pos, &game.map));
        i += 1.0;

        // Draw amount available
        let amount = match game.manifest.amounts(&game.log).get(tile.name()) {
            None => {
                eprintln!("No tile amount found for {}", tile.name());
                process::exit(1);
            }
            Some(amount) => amount.to_string(),
        };
        let text_pos = helpers::scale(&game.map) *
            Vector2::new(pos.x-1.0, pos.y-0.7);
        g = g.add(element::draw_text(&format!("{}×", amount), &text_pos,
                                     &tile::TextAnchor::Start, None, None));
    }

    g
}

/// Draws sheets with tiles of them for printing
pub fn draw_tile_sheets(game: &game::Game) -> Vec<svg::Document> {
    const TILES_PER_PAGE: u32 = 30;
    const TILES_PER_COL: u32 = 6;
    // Always draw vertical (fits more on a page)
    let mut info = game.map.clone();
    info.orientation = Orientation::Vertical;
    let mut drawn = 0;
    let mut sheets = vec![];
    let mut cur_doc = svg::Document::new()
        .set("width", "210mm")
        .set("height", "297mm")
        .add(element::draw_text(
                "Tile sheet 0",
                &(Vector2::new(2.0_f64, 0.5) * helpers::scale(&info)),
                &tile::TextAnchor::Start,
                Some("200%"), None));
    for tile in game.manifest.tiles.iter() {
        for _ in 0..*game.manifest.amounts(&None).get(tile.name()).unwrap() {
            let x = ((drawn % TILES_PER_PAGE) / TILES_PER_COL) as f64;
            let y = (drawn % TILES_PER_COL) as f64;
            let pos = Vector2::new(3.0_f64.sqrt() * (x + 1.0),
                                   2.0 * y + 1.75 + (x % 2.0));
            cur_doc = cur_doc.add(draw_tile(tile, &pos, &info));
            drawn += 1;
            // When a sheet is full, append this to the list and start a
            // new page
            if drawn % TILES_PER_PAGE == 0 {
                sheets.push(cur_doc);
                cur_doc = svg::Document::new()
                    .set("width", "210mm")
                    .set("height", "297mm")
                    .add(element::draw_text(
                            &format!("Tile sheet {}", drawn/TILES_PER_PAGE),
                            &(Vector2::new(2.0, 0.5) * helpers::scale(&info)),
                            &tile::TextAnchor::Start,
                            Some("200%"), None));
            }
        }
    }
    sheets.push(cur_doc);
    sheets
}

/// Convert location to cube coordinate
fn convert_coord(col: i32, row: i32, map: &game::Map) -> [f64; 3] {
    match map.orientation {
        Orientation::Vertical => {
            let x = (col - (row - (row % 2)) / 2) as f64;
            let z = row as f64;
            [x, -x-z, -z]
        }
        Orientation::Horizontal => {
            let x = col as f64;
            let z = (row - (col - (col % 2)) / 2) as f64;
            [x, -x-z, -z]
        }
    }
}

/// Draw the map of a game
pub fn draw_map(game: &game::Game, options: &super::Options) -> svg::Document {
    let width: f64;
    let height: f64;
    let offset: Vector2<f64>;
    match &game.map.orientation {
        &Orientation::Horizontal => {
            width = 0.3 * 3.0_f64.sqrt() + game.map.width as f64 * 1.5;
            height = (0.5 + game.map.height as f64) * 3.0_f64.sqrt();
            offset = Vector2::new(consts::BORDER + 1.0,
                                  consts::BORDER + 3.0_f64.sqrt() / 2.0);
        }
        &Orientation::Vertical => {
            width = (0.5 + game.map.width as f64) * 3.0_f64.sqrt();
            height = 0.3 * 3.0_f64.sqrt() + game.map.height as f64 * 1.5;
            offset = Vector2::new(consts::BORDER + 3.0_f64.sqrt() / 2.0,
                                  consts::BORDER + 1.0);
        }
    }
    let page_width = (width + 2.0 * consts::BORDER)
        * helpers::scale(&game.map);
    let page_height = (height + 2.0 * consts::BORDER)
        * helpers::scale(&game.map);
    let basis = helpers::get_basis(&game.map.orientation);
    let mut doc = svg::Document::new()
        .set("width", format!("{}", page_width))
        .set("height", format!("{}", page_height));

    // Draw tiles
    let placed = game.placed_tiles();
    let tiles = game.map.tiles();
    let tiles = game::top_tiles(&placed, &tiles);
    for (&(x, y), tile) in tiles.iter() {
        let pos = offset + basis
            * na::Vector3::from(convert_coord(x as i32, y as i32, &game.map))
                .component_mul(&na::Vector3::new(2.0, 1.0, 1.0));
        doc = doc.add(draw_tile(tile.deref(), &pos, &game.map));
    }

    // Draw barriers
    for barrier in game.map.barriers.iter() {
        let (x, y) = barrier.location.as_coord(&game.map.orientation);
        let pos = offset + basis
            * na::Vector3::from(convert_coord(x as i32, y as i32, &game.map))
                .component_mul(&na::Vector3::new(2.0, 1.0, 1.0));
        doc = doc.add(element::draw_barrier(barrier, &pos, &game.map));
    }

    // Draw tokens
    for (location, tokens) in game.tokens().iter() {
        let tile = tiles.get(&location).unwrap();
        let rot = helpers::rotate(&tile.orientation());
        for token in tokens {
            let mut g = Group::new();
            let city = tile.cities().get(token.station).unwrap().clone();
            let center = offset + basis
                * na::Vector3::from(convert_coord(location.0 as i32,
                                                  location.1 as i32,
                                                  &game.map))
                    .component_mul(&na::Vector3::new(2.0, 1.0, 1.0));
            let token_pos = helpers::city_circle_pos(&city, token.circle,
                                                     &center, &game.map,
                                                     &tile.orientation());
            if let Orientation::Vertical = game.map.orientation {
                let station_pos = (center + rot * basis * city.position())
                    * helpers::scale(&game.map);
                g = g.set("transform", format!("rotate(-30 {} {})",
                                               station_pos.x, station_pos.y));
            }
            g = g.add(element::draw_token(&token.name, &token.color,
                                          token.is_home,
                                          &token_pos, &game.map));
            doc = doc.add(g);
        }
    }

    doc.add(element::draw_coordinate_system(game, options, width, height))
}

/// Draws a single tile
pub fn draw_tile(tile: &tile::TileSpec,
                 pos: &Vector2<f64>,
                 map: &game::Map) -> Group
{
    let mut g = Group::new();
    let basis = helpers::get_basis(&map.orientation);
    let rotation = helpers::rotate(&tile.orientation());

    g = g.add(helpers::draw_hex_background(*pos, &map, tile.color()));

    // Draw white contrast lines first
    for path in tile.paths() {
        g = g.add(element::draw_path_contrast(&path, pos, &map,
                                              &tile.orientation()));
    }
    for city in tile.cities() {
        g = g.add(element::draw_city_contrast(city, pos, &map,
                                              &tile.orientation()));
    };

    // Draw terrain
    if let Some(terrain) = tile.terrain() {
        g = g.add(element::draw_terrain(&terrain, pos, &map));
    }

    // Draw elements
    if tile.is_lawson() {
        g = g.add(element::draw_lawson(*pos, &map));
    }
    for path in tile.paths() {
        g = g.add(element::draw_path(&path, pos, &map, &tile.orientation()));
    };

    for stop in tile.stops() {
        g = g.add(element::draw_stop(stop, *pos, &map, tile,
                                     &tile.orientation()));
    }

    for city in tile.cities() {
        g = g.add(element::draw_city(city, *pos, &map, tile,
                                     &tile.orientation()));
    }

    for arrow in tile.arrows() {
        g = g.add(element::draw_arrow(&arrow, pos, &map));
    }

    // Draw text on tile
    for text in tile.text_spec() {
        let text_pos = helpers::scale(&map) *
            (rotation * basis * text.position() + pos);
        let mut t = element::draw_text(&tile.get_text(&text.id), &text_pos,
                                       &text.anchor, text.size(), text.weight);
        // Rotate the tile number with the orientation of the map
        if let Orientation::Vertical = map.orientation {
            let mut angle = -30;
            if text.id == "number" {
                angle += (tile.orientation() * 180.0 / PI) as i32;
            }
            t = t.set("transform",
                      format!("rotate({} {} {})", angle, text_pos.x, text_pos.y));
        }
        g = g.add(t);
    }

    // Draw revenue track
    if let Some(track) = tile.revenue_track() {
        g = g.add(element::draw_revenue_track(&track, pos, &map));
    }

    // Draw outline last to prevent visual effects
    g.add(helpers::draw_hex_edge(*pos, &map))
}
