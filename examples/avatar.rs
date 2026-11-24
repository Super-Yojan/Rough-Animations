//! A simple 3D avatar, built from boxes and flat shapes, that draws itself, blinks and smiles.
//! Writes `target/avatar.mp4` and `target/avatar.png` (needs ffmpeg).
//!
//! The head is a cube. Its visible front face is the plane `z = 50`, so the face is flat shapes
//! lying on it (`.on(Plane::Front)`), and the nose and ear are small boxes that stick out.

use rough_animations::prelude::*;

const SKIN: &str = "#f4c7a1";
const HAIR: &str = "#6b4a2b";
/// The head spans x, z in [-50, 50]; face features sit just in front of this plane.
const FACE_Z: f32 = 50.5;

fn main() -> Result {
    let mut s = Stage::new(900, 700).zoom(1.6).origin(450, 570).wobble(8);
    let intro = s.heading("Meet Pixel", "a 3D avatar, built from a handful of shapes");

    // The body: each part stands on the one below it. `hb` is the y of the head's base, and the
    // face and hair are measured up from it.
    let hb = 90.0;
    let torso = s.add(cuboid(130, 50, 76).base_at((0, 0, 0)).fill("sky"));
    let neck = s.add(cuboid(34, 40, 34).base_at((0, 50, 0)).fill(SKIN));
    let head = s.add(cuboid(100, 100, 100).base_at((0, hb, 0)).fill(SKIN));

    // Hair: a cap on top, a fringe flush with the forehead and a sideburn on the visible side.
    let hair = [
        s.add(cuboid(106, 14, 106).base_at((0, hb + 100.0, 0)).fill(HAIR)),
        s.add(cuboid(104, 14, 6).base_at((0, hb + 86.0, 47)).fill(HAIR)),
        s.add(cuboid(8, 26, 100).base_at((54, hb + 74.0, 0)).fill(HAIR)),
    ];
    let ear = s.add(cuboid(8, 22, 14).base_at((54, hb + 38.0, 0)).fill(SKIN));
    let nose = s.add(cuboid(10, 14, 12).base_at((0, hb + 40.0, 50)).fill("#e2a57d"));

    // Flat face features lying on the front of the head.
    let on_face = |shape: Object, x: f32, y: f32, lift: f32| {
        shape.on(Plane::Front).at((x, hb + y, FACE_Z + lift))
    };
    let eye_x = [-22.0, 22.0];
    let whites = eye_x.map(|x| s.add(on_face(circle(11).fill("white").solid(), x, 58.0, 0.0)));
    let pupils = eye_x.map(|x| s.add(on_face(circle(5.5).fill("ink").solid(), x + 2.0, 56.0, 0.4)));
    let shines = eye_x.map(|x| s.add(on_face(circle(1.8).fill("white").solid().stroke_width(0), x + 4.0, 59.0, 0.8)));
    let brows = [(-22.0, 8.0), (22.0, -8.0)].map(|(x, tilt)| {
        s.add(on_face(rect(20, 4.5).fill(HAIR).solid(), x, 75.0, 0.2).rotated((0, 0, deg(tilt))))
    });
    let mouth = s.add(on_face(rect(22, 4).fill("red").solid(), 0.0, 24.0, 0.2));
    let cheeks = [-36.0, 36.0].map(|x| {
        s.add(on_face(circle(7).fill("pink").solid().opacity(0.7).stroke_width(0), x, 36.0, 0.1))
    });

    // 1. The body and head draw themselves, then the hair and ear.
    s.play(write_all(&intro));
    s.play(lag(0.3, [torso, neck, head].map(|id| create(id).duration(0.9))));
    s.play(create_all(hair));
    s.play(create_all([ear, nose]));

    // 2. The face: eyes first, then brows and mouth.
    s.play(create_all(whites));
    s.play(create_all(pupils).duration(0.6));
    s.play(create_all(shines).duration(0.4));
    s.play(create_all(brows).duration(0.8));
    s.play(create(mouth).duration(0.6));
    s.wait(0.6);

    // 3. Blink twice.
    let eyes: Vec<Id> = whites.iter().chain(&pupils).chain(&shines).copied().collect();
    let blink = || all(eyes.iter().map(|&id| {
        seq([scale_to(id, (1, 0.08, 1)).duration(0.1), scale_to(id, (1, 1, 1)).duration(0.14)])
    }));
    s.play(blink());
    s.wait(0.5);
    s.play(blink());
    s.wait(0.4);

    // 4. Smile: the mouth turns into a smile, the brows lift and the cheeks flush.
    let smile = ellipse(13, 9).minus(ellipse(13, 9).shift(0, 4.5));
    s.play(all([
        morph(mouth, smile).duration(0.9),
        move_by(brows[0], (0, 3, 0)),
        move_by(brows[1], (0, 3, 0)),
        fade_in(cheeks[0]),
        fade_in(cheeks[1]),
    ]).duration(0.9));
    s.wait(0.5);
    s.play(blink());
    s.wait(1.2);

    s.render("target/avatar.mp4")?;
    s.render("target/avatar.png")
}
