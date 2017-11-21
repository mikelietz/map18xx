extern crate map18xx;
extern crate svg;

use map18xx::{draw, map, tile};
use svg::Document;

fn main() {
    let definitions = tile::definitions();
    let info = map::MapInfo::default();
    let document = Document::new()
        .set("width", 3.0 * info.scale)
        .set("height", 2.0 * info.scale * definitions.len() as f64)
        .add(draw::draw_tile_definitions(&definitions));

    svg::save("definitions.svg", &document).unwrap();
}
