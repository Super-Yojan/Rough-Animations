//! The intro video, made with the library itself. Writes `docs/media/intro.mp4`
//! (needs ffmpeg and a LaTeX install for the formulas).
//!
//! Everything is laid out in screen pixels (`s.pixel(x, y)`) on a 1280x720 canvas.

use rough_animations::prelude::*;

const PURPLE: &str = "#3c3278";
const GRAY: &str = "#6a6a80";

/// Text centred on pixel `(x, y)`.
fn label(s: &mut Stage, content: &str, size: u32, x: u32, y: u32, color: &str) -> Id {
    s.add(text(content).font_size(size).color(color).at(s.pixel(x, y)))
}

/// A big title: lively hand-drawn wobble, which suits large text.
fn big(s: &mut Stage, content: &str, size: u32, x: u32, y: u32, color: &str) -> Id {
    s.add(text(content).font_size(size).color(color).animate().at(s.pixel(x, y)))
}

/// Text centred `(dx, dy)` pixels from where a world point appears on screen.
fn tag(s: &mut Stage, content: &str, p: (f32, f32, f32), dx: f32, dy: f32) -> Id {
    let q = s.scene.camera().project(p.0, p.1, p.2);
    s.add(text(content).font_size(30).at(s.pixel(q.x + dx, q.y + dy)))
}

/// A section header and a one-line caption under it.
fn section(s: &mut Stage, title: &str, caption: &str) -> Vec<Id> {
    vec![big(s, title, 60, 640, 70, PURPLE), label(s, caption, 32, 640, 128, GRAY)]
}

/// Erase things pen-style.
fn clear(ids: &[Id]) -> Anim {
    all(ids.iter().map(|&id| uncreate(id))).duration(0.7)
}

/// Write a list of things one after another.
fn write_all(ids: &[Id], gap: f32) -> Anim {
    lag(gap, ids.iter().map(|&id| write(id)))
}

fn main() -> Result {
    let mut s = Stage::new(1280, 720).zoom(1.4).wobble(8);

    // ---- 1. Title ------------------------------------------------------------------------
    let title = big(&mut s, "rough-animations", 116, 640, 215, PURPLE);
    let tagline = label(&mut s, "hand-drawn 3D & 2D animation, in Rust", 46, 640, 315, GRAY);
    let row = vec![
        s.add(cube(80).base_at(s.pixel(280, 560)).fill("purple")),
        s.add(cylinder(42, 80).base_at(s.pixel(520, 560)).fill("mint")),
        s.add(cone(42, 95).base_at(s.pixel(760, 560)).fill("amber")),
        s.add(sphere(44).base_at(s.pixel(1000, 560)).fill("rose")),
    ];
    s.play(write(title).duration(1.8));
    s.play(write(tagline).duration(1.4));
    s.play(lag(0.25, row.iter().map(|&id| create(id).duration(1.0))));
    s.wait(1.2);
    s.play(clear(&[vec![title, tagline], row].concat()));

    // ---- 2. Solids that hide each other ----------------------------------------------------
    let head = section(&mut s, "Isometric solids", "that stack and hide each other properly");
    let c = s.pixel(640, 515); // centre of the slab, on the floor
    let (cx, cz) = (c.x, c.z);
    let slab = s.add(cuboid(240, 20, 240).base_at((cx, 0.0, cz)).fill("slate"));
    let compute = s.add(cuboid(80, 100, 80).base_at((cx - 50.0, 20.0, cz - 50.0)).fill("purple"));
    let storage = s.add(cuboid(80, 75, 80).base_at((cx + 60.0, 20.0, cz - 50.0)).fill("mint"));
    let gateway = s.add(cuboid(80, 50, 80).base_at((cx - 50.0, 20.0, cz + 65.0)).fill("amber"));
    let hub = s.add(cuboid(70, 25, 70).base_at((cx - 50.0, 125.0, cz - 50.0)).fill("sky"));
    let names = vec![
        tag(&mut s, "hub", (cx - 50.0, 150.0, cz - 50.0), 100.0, -4.0),
        tag(&mut s, "storage", (cx + 60.0, 95.0, cz - 50.0), 90.0, -22.0),
        tag(&mut s, "gateway", (cx - 50.0, 70.0, cz + 65.0), -105.0, -14.0),
    ];
    s.play(write_all(&head, 0.3));
    s.play(create(slab).duration(1.2));
    s.play(lag(0.3, [compute, storage, gateway].map(|b| create(b).duration(0.9))));
    s.play(create(hub).duration(0.9));
    s.play(write_all(&names, 0.25));
    // Slide the amber box in front of the green one, and back: it is drawn over it, then behind.
    s.play(seq([move_by(gateway, (110, 0, 0)), move_by(gateway, (-110, 0, 0))]).duration(2.4));
    s.wait(0.6);
    s.play(clear(&[head, vec![slab, compute, storage, gateway, hub], names].concat()));

    // ---- 3. Booleans ------------------------------------------------------------------------
    let head = section(&mut s, "Boolean operations", "combine any flat shapes");
    let (a, b) = (|| circle(60).shift(-25, 0), || circle(60).shift(25, 0));
    let results = [
        ("a.union(b)", a().union(b()), "purple"),
        ("a.intersect(b)", a().intersect(b()), "mint"),
        ("a.minus(b)", a().minus(b()), "amber"),
        ("a.xor(b)", a().xor(b()), "rose"),
    ];
    let mut shapes = Vec::new();
    let mut captions = Vec::new();
    for (i, (code, shape, color)) in results.into_iter().enumerate() {
        let x = 190 + 300 * i as u32;
        shapes.push(s.add(shape.at(s.pixel(x, 400)).fill(color)));
        captions.push(label(&mut s, code, 38, x, 540, GRAY));
    }
    s.play(write_all(&head, 0.3));
    s.play(lag(0.45, shapes.iter().map(|&id| create(id).duration(1.1))));
    s.play(write_all(&captions, 0.25));
    s.wait(1.6);
    s.play(clear(&[head, shapes, captions].concat()));

    // ---- 4. Morphing -----------------------------------------------------------------------
    let head = section(&mut s, "Morph anything into anything", "flat shapes and extruded solids alike");
    let flat = s.add(circle(65).at(s.pixel(390, 430)).fill("purple"));
    let solid = s.add(prism(circle(48), 70).base_at(s.pixel(890, 470)).fill("mint"));
    let note = label(&mut s, "morph(id, star(5, 75, 32))", 40, 640, 650, GRAY);
    let donut = circle(75).minus(circle(32));
    let blobs = circle(32).shift(-28, 0).union(circle(32).shift(32, 0));
    let lens = circle(70).shift(-30, 0).intersect(circle(70).shift(30, 0));
    s.play(write_all(&head, 0.3));
    s.play(all([create(flat), create(solid), write(note)]).duration(1.2));
    s.wait(0.3);
    s.play(all([morph(flat, rect(125, 125)), morph(solid, rect(78, 78))]).duration(1.0));
    s.play(all([
        morph(flat, star(5, 85, 36)).ease(Easing::BackOut),
        fill_to(flat, "gold"),
        morph(solid, star(5, 46, 20)),
    ]).duration(1.2));
    s.play(all([morph(flat, donut), fill_to(flat, "rose"), morph(solid, blobs)]).duration(1.2));
    s.play(all([morph(flat, lens), fill_to(flat, "sky"), morph(solid, circle(48))]).duration(1.2));
    s.wait(0.6);
    s.play(clear(&[head, vec![flat, solid, note]].concat()));

    // ---- 5. Text and LaTeX -----------------------------------------------------------------
    let head = section(&mut s, "Text & handwritten LaTeX", "typeset by LaTeX, then written out by hand");
    let f1 = s.add(math(r"\int_0^1 x^2\,dx = \frac{1}{3}")?.font_size(72).color("rose").handwritten().at(s.pixel(640, 290)));
    let f2 = s.add(math(r"e^{i\pi} + 1 = 0")?.font_size(72).color("blue").handwritten().at(s.pixel(640, 475)));
    let floor = s.add(text("on any plane").font_size(40).on(Plane::Ground).color("#5a5a6e").at(s.pixel(640, 615)));
    s.play(write_all(&head, 0.3));
    s.play(write(f1).duration(2.0));
    s.play(write(f2).duration(1.6));
    s.play(write(floor).duration(1.4));
    s.wait(1.2);
    s.play(clear(&[head, vec![f1, f2, floor]].concat()));

    // ---- 6. Code ---------------------------------------------------------------------------
    let head = section(&mut s, "A few lines of code", "that's the whole script");
    let source = "let mut s = Stage::new(900, 600);\n\
                  let tower = s.add(cuboid(80, 110, 80).fill(\"purple\"));\n\
                  s.play(create(tower));\n\
                  s.play(move_by(tower, (100, 0, 0)));\n\
                  s.render(\"out.mp4\")";
    let code = s.add(
        Object::new(rough_animations::primitives::Text::laid_out(source, &Font::default_font(), Align::Left, 1.15))
            .font_size(40)
            .color("#2d2d3a")
            .at(s.pixel(640, 310)),
    );
    let demo = s.add(cuboid(80, 80, 80).base_at(s.pixel(330, 625)).fill("purple"));
    s.play(write_all(&head, 0.3));
    s.play(write(code).duration(5.0));
    s.play(create(demo).duration(1.0));
    s.play(move_by(demo, (150, 0, -150)).duration(1.8));
    s.wait(1.0);
    s.play(clear(&[head, vec![code, demo]].concat()));

    // ---- 7. Outro --------------------------------------------------------------------------
    let title = big(&mut s, "rough-animations", 116, 640, 290, PURPLE);
    let line = label(&mut s, "script it. render it. share it.", 54, 640, 395, GRAY);
    let foot = label(&mut s, "inspired by manim and rough.js", 34, 640, 470, GRAY);
    s.play(write(title).duration(1.6));
    s.play(write(line).duration(1.4));
    s.play(write(foot).duration(1.0));
    s.wait(2.0);

    s.render("docs/media/intro.mp4")
}
