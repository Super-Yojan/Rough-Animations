//! The same scene in every built-in theme, tiled into one picture.
//! Writes `target/themes.png`.

use rough_animations::prelude::*;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

/// One small scene. Nothing in it picks a colour for lines or text, so the theme decides them.
fn scene(theme: &str) -> Stage {
    let mut s = Stage::new(600, 400).origin(300, 215).theme(theme);
    s.heading(theme, "page, lines, text, captions");
    // A point (k + d, 0, d - k) sits d below the origin and 1.73 k to its right, so these three
    // line up in a horizontal row and their labels line up too.
    let row = |k: i32| (k + 50, 0, 50 - k);
    let cube = s.add(cube(60).base_at(row(-90)).fill("mint"));
    let cyl = s.add(cylinder(28, 64).base_at(row(0)).fill("amber"));
    let ball = s.add(sphere(30).base_at(row(90)).fill("rose"));
    for (id, name) in [(cube, "cube"), (cyl, "cylinder"), (ball, "sphere")] {
        s.tag(id, label(name).font_size(20), Side::Below);
    }
    s
}

fn main() -> Result {
    let names = ["light", "dark", "chalkboard", "blueprint", "sepia", "terminal"];
    let (w, h, cols) = (600u32, 400u32, 3u32);
    let rows = names.len() as u32 / cols;
    let mut sheet = Pixmap::new(w * cols, h * rows).expect("sheet size");
    for (i, name) in names.iter().enumerate() {
        let mut stage = scene(name);
        let end = stage.duration();
        let frame = stage.render_frame(end);
        let (x, y) = ((i as u32 % cols) * w, (i as u32 / cols) * h);
        sheet.draw_pixmap(x as i32, y as i32, frame.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
    }
    sheet.save_png("target/themes.png").map_err(std::io::Error::other)?;
    println!("Wrote target/themes.png");
    Ok(())
}
