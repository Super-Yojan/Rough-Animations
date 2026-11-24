//! Morphs flat shapes and an extruded solid into one another.
//! Writes `target/morph.mp4` (needs ffmpeg).

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    let flat = s.add(circle(55).at((-90, 0, 0)).fill("purple"));
    let solid = s.add(prism(circle(40), 60).base_at((110, 0, 0)).fill("mint"));
    let ghost = s.add(star(5, 75, 32).at((-90, 0, 60)).fill("gold")); // a stand-in to describe a target look

    s.play(all([create(flat), create(solid)]).duration(1.2));
    s.wait(0.3);
    s.play(all([morph(flat, rect(110, 110)), morph(solid, rect(66, 66))]));
    s.play(all([transform_into(flat, ghost), morph(solid, star(5, 38, 16))]).duration(1.2));
    let donut = circle(65).minus(circle(28));
    let blobs = circle(27).shift(-24, 0).union(circle(27).shift(27, 0));
    s.play(all([morph(flat, donut), fill_to(flat, "rose"), morph(solid, blobs)]).duration(1.2));
    let lens = circle(60).shift(-25, 0).intersect(circle(60).shift(25, 0));
    s.play(all([
        morph(flat, lens).ease(Easing::BackOut),
        fill_to(flat, "sky"),
        morph(solid, circle(40)),
    ]).duration(1.2));
    s.wait(0.8);
    s.render("target/morph.mp4")
}
