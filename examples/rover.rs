//! A rover built from boxes, wheels and a phone on a mast, treated as one `Group`: it builds
//! itself up, drives, turns on the spot and drives again.
//! Writes `target/rover.mp4` and `target/rover.png` (needs ffmpeg).

use rough_animations::prelude::*;

/// Build the rover and hand back all of its parts as one group.
fn rover(s: &mut Stage) -> Group {
    let mut rover = Group::new();
    s.add_to(&mut rover, cuboid(190, 8, 120).base_at((0, 14, 0)).fill("slate"));
    s.add_to(&mut rover, cuboid(190, 26, 120).base_at((0, 22, 0)).fill("white").stroke("ink"));
    s.add_to(&mut rover, rect(150, 80).on(Plane::Ground).at((0, 48.5, 0)).fill("#e5e7eb").solid());

    // Wheels, with their axes along z.
    for (x, z) in [(-62, -72), (62, -72), (-62, 72), (62, 72)] {
        s.add_to(&mut rover, cylinder(24, 18).at((x, 24, z)).rotated((deg(90), 0, 0)).fill("ink"));
    }

    // A mast with a phone on top.
    s.add_to(&mut rover, cuboid(8, 90, 8).base_at((70, 48, 0)).fill("gray"));
    s.add_to(&mut rover, cuboid(8, 70, 40).base_at((80, 138, 0)).fill("indigo"));
    s.add_to(&mut rover, circle(5).on(Plane::Side).at((84.5, 190, 10)).fill("black").solid());
    rover
}

fn main() -> Result {
    let mut s = Stage::new(1280, 720).origin(600, 470).zoom(1.15);
    s.heading("Rover", "body, wheels, mast, iPhone");

    let rover = rover(&mut s);
    s.place(&rover, (-170, 0, 170)); // set it down where the journey starts; it ends centred

    s.play(rover.create());                                          // the parts build up
    s.wait(0.4);
    s.play(rover.move_by((170, 0, 0)).duration(2));                  // drive forward
    s.play(rover.rotate_by((0, deg(90), 0)).duration(1.5));          // turn on the spot, wheels too
    s.play(rover.move_by((0, 0, -170)).duration(2));                 // drive the way it now faces
    s.wait(0.6);

    s.render("target/rover.mp4")?;
    s.render("target/rover.png")
}
