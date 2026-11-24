//! Labels, a title and a LaTeX formula, written onto a scene.
//! Writes `target/text.mp4` and `target/text.png` (needs ffmpeg; the formula needs LaTeX and
//! is skipped when it isn't installed).

use rough_animations::prelude::*;

fn build() -> Stage {
    let mut s = Stage::new(900, 600).wobble(8);
    let slab = s.add(cuboid(240, 20, 240).base_at((100, 0, 100)).fill("slate"));
    let boxes = [
        s.add(cuboid(80, 110, 80).base_at((50, 20, 50)).fill("purple")),
        s.add(cuboid(80, 75, 80).base_at((160, 20, 50)).fill("mint")),
        s.add(cuboid(80, 50, 80).base_at((50, 20, 165)).fill("amber")),
    ];
    // Labels are upright on screen, centred on their position.
    let labels = [
        s.add(text("compute").font_size(26).at((50, 178, 50))),
        s.add(text("storage").font_size(26).at((160, 138, 50))),
        s.add(text("gateway").font_size(26).at((40, 108, 235))),
    ];
    let title = s.add(text("System architecture").font_size(44).color("#3c3278").at((100, 330, 100)));
    // A caption lying on the slab, in the same plane.
    let floor = s.add(text("platform").font_size(36).on(Plane::Ground).color("#5a5a6e").at((100, 20, 205)));
    let formula = math(r"\text{latency} \approx \sum_{i=1}^{n} \frac{d_i}{v_i}")
        .map(|f| s.add(f.font_size(28).color("rose").at((215, 205, -60))));

    s.play(write(title).duration(1.4));
    s.play(create(slab));
    s.play(lag(0.3, boxes.map(|b| create(b).duration(0.9))));
    s.play(write(floor).duration(0.8));
    s.play(lag(0.25, labels.map(|l| write(l).duration(0.8))));
    match formula {
        Ok(f) => drop(s.play(write(f).duration(1.8))),
        Err(e) => eprintln!("note: skipping the LaTeX formula ({e})"),
    }
    s.wait(1);
    s
}

fn main() -> Result {
    build().render("target/text.mp4")?;
    build().render("target/text.png")
}
