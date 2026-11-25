//! A figure for a slide or document: transparent background, cropped to the drawing.
//! Writes `target/figure.png`.

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600).transparent();
    s.add(cuboid(90, 90, 90).base_at((0, 0, 0)).fill("mint"));
    s.add(cuboid(60, 60, 60).base_at((-20, 0, 90)).fill("amber"));
    s.add(sphere(40).at((100, 20, 70)).fill("rose"));
    s.add(text("in front").at(s.pixel(450, 520)));
    s.still_cropped("target/figure.png", 16)?;
    Ok(())
}
