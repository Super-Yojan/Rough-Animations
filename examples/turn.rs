//! Renders one scene from four turns of the camera: `target/turn-0.png` ... `turn-3.png`.
//! A stack of boxes with a face on the front, a cylinder and a hole-punched plate, so depth
//! ordering, back faces and wall decals are all visible.

use rough_animations::prelude::*;

fn scene(yaw: f32) -> Stage {
    let mut s = Stage::new(600, 440).zoom(1.3).origin(300, 300).yaw(yaw);
    s.add(cuboid(140, 30, 140).base_at((0, 0, 0)).fill("slate"));
    s.add(cuboid(80, 70, 80).base_at((0, 30, 0)).fill("purple"));
    // A decal on the +z face of the tall box: it must vanish when the camera is behind it.
    s.add(circle(14).fill("white").solid().on(Plane::Front).at((0, 70, 40.5)));
    s.add(cylinder(22, 60).base_at((-90, 0, 40)).fill("mint"));
    s.add(prism(rect(60, 60).minus(circle(16)), 14).base_at((80, 0, -60)).fill("rose"));
    s.add(sphere(24).base_at((70, 0, 70)).fill("sky"));
    s
}

fn main() -> Result {
    for (i, deg_) in [0.0, 40.0, 90.0, 180.0].into_iter().enumerate() {
        scene(deg(deg_)).render(format!("target/turn-{i}.png"))?;
    }
    Ok(())
}
