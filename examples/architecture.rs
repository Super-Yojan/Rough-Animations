//! A still isometric sketch. Writes `architecture.png`.

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(800, 600);
    s.add(cuboid(240, 20, 240).base_at((100, 0, 100)).fill("slate"));
    s.add(cuboid(80, 110, 80).base_at((50, 20, 50)).fill("purple"));
    s.add(cuboid(80, 75, 80).base_at((160, 20, 50)).fill("mint"));
    s.add(cuboid(80, 50, 80).base_at((50, 20, 165)).fill("amber"));
    s.add(cuboid(70, 25, 70).base_at((50, 155, 50)).fill("sky"));
    s.render("architecture.png")
}
