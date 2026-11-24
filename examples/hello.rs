//! The smallest useful script. Writes `target/hello.mp4` (needs ffmpeg).

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600);
    let cube = s.add(cuboid(80, 80, 80).base_at((0, 0, 0)).fill("purple"));
    let title = s.add(text("hello").at((0, 130, 0)));

    s.play(create(cube));
    s.play(write(title));
    s.play(move_by(cube, (100, 0, 0)).duration(1.5));
    s.wait(0.5);
    s.render("target/hello.mp4")
}
