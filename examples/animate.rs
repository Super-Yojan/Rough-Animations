//! Builds the architecture scene piece by piece, then spins and lifts it.
//! Writes `target/animate.mp4` and `target/animate.gif` (needs ffmpeg).

use rough_animations::prelude::*;

fn build() -> Stage {
    let mut s = Stage::new(800, 600).wobble(8);
    let slab = s.add(cuboid(240, 20, 240).base_at((100, 0, 100)).fill("slate"));
    let compute = s.add(cuboid(80, 110, 80).base_at((50, 20, 50)).fill("purple"));
    let storage = s.add(cuboid(80, 75, 80).base_at((160, 20, 50)).fill("mint"));
    let gateway = s.add(cuboid(80, 50, 80).base_at((50, 20, 165)).fill("amber"));
    let hub = s.add(cuboid(70, 25, 70).base_at((50, 155, 50)).fill("sky"));

    s.play(create(slab).duration(1.5));
    s.play(lag(0.3, [compute, storage, gateway].map(|b| {
        fade_in(b).duration(0.6).with(move_from(b, (0, 80, 0)).duration(0.8).ease(Easing::BounceOut))
    })));
    s.play(create(hub));
    s.wait(0.5);
    s.play(all([
        rotate_by(compute, (0, deg(90), 0)),
        move_by(hub, (0, 40, 0)),
        fill_to(storage, "rose"),
    ]).duration(1.2));
    s.wait(1);
    s
}

fn main() -> Result {
    // A stage plays once, so each output gets a fresh one.
    for path in ["target/animate.mp4", "target/animate.gif"] {
        build().render(path)?;
    }
    Ok(())
}
