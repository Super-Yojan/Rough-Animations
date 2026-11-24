//! Stacking, overlap and hidden-line checks. Writes `target/occlusion.png`.

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600).origin(450, 330);
    s.add(cuboid(300, 20, 300).base_at((110, 0, 110)).fill("slate"));
    s.add(cuboid(80, 110, 80).base_at((30, 20, 30)).fill("purple"));
    s.add(cuboid(70, 22, 70).base_at((30, 164, 30)).fill("sky")); // floats above the purple box
    s.add(cylinder(30, 90).base_at((150, 20, 30)).fill("mint")); // behind the amber box
    s.add(cuboid(90, 60, 90).base_at((110, 20, 125)).fill("amber"));
    s.add(sphere(34).base_at((225, 20, 190)).fill("rose"));
    s.add(text("in front").at((225, 120, 190)));
    s.render("target/occlusion.png")
}
