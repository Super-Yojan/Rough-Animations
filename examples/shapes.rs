//! The shape primitives and boolean operations.
//! Writes `target/shapes.png` (solids) and `target/booleans.png` (2D booleans).

use rough_animations::prelude::*;

fn main() -> Result {
    let mut s = Stage::new(900, 600).origin(450, 250);
    s.add(cube(70).base_at((0, 0, 0)).fill("purple"));
    s.add(cylinder(35, 70).base_at((110, 0, 0)).fill("mint"));
    s.add(cone(35, 80).base_at((220, 0, 0)).fill("amber"));
    s.add(sphere(36).base_at((330, 0, 0)).fill("sky"));
    // Extrude a flat shape to get a solid; combine shapes first for holes and cut-outs.
    s.add(prism(rect(70, 70).minus(circle(20)), 30).base_at((0, 0, 110)).fill("rose"));
    s.add(prism(star(5, 40, 18), 25).base_at((110, 0, 110)).fill("gold"));
    s.add(prism(circle(25).union(circle(25).shift(30, 0)), 40).base_at((235, 0, 110)).fill("sky"));
    s.add(arrow((0, 0, 0), (0, 130, 0)).at((-70, 0, -70)).stroke("red"));
    s.add(line((0, 0, 0), (120, 0, 0)).at((-70, 0, -70)));
    s.render("target/shapes.png")?;

    // The four operations on two overlapping circles, lying on the floor.
    let mut s = Stage::new(900, 600).origin(450, 120);
    let (a, b) = (|| circle(45), || circle(45).shift(45, 0));
    s.add(a().union(b()).at((0, 0, 0)).fill("purple"));
    s.add(a().intersect(b()).at((190, 0, 0)).fill("mint"));
    s.add(a().minus(b()).at((0, 0, 150)).fill("amber"));
    s.add(a().xor(b()).at((190, 0, 150)).fill("rose"));
    s.render("target/booleans.png")
}
