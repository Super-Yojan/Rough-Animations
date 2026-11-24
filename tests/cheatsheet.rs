//! Keeps docs/cheatsheet.md honest: everything it names is called here through the prelude,
//! with the argument styles it promises, and its claims are checked against the real behaviour.

use rough_animations::prelude::*;

const SHEET: &str = include_str!("../docs/cheatsheet.md");

#[test]
fn every_shape_and_builder_on_the_sheet_exists_and_chains() {
    let mut s = Stage::new(900, 600);
    // 3D solids
    let solids = [
        cuboid(80, 110, 80),
        cube(10),
        cylinder(5, 20),
        cone(5, 20),
        sphere(5),
    ];
    // 2D flat shapes, other
    let flats = [
        circle(10),
        ellipse(10, 5),
        rect(10, 5),
        ngon(6, 10),
        star(5, 10, 4),
        polygon([(0, 0), (10, 0), (5, 8)]),
    ];
    let others = [line((0, 0, 0), (10, 0, 0)), arrow((0, 0, 0), (0, 10, 0)), text("hi")];
    for o in solids.into_iter().chain(flats).chain(others) {
        // Every builder named on the sheet, with every promised argument style.
        let o = o
            .at((1, 2, 3))
            .at([1.5, 2.5, 3.5])
            .base_at((0, 0, 0))
            .fill("purple")
            .fill("#a882ff")
            .fill((168, 130, 255))
            .stroke("ink")
            .color("rose")
            .opacity(0.5)
            .opacity(1)
            .stroke_width(2)
            .rotated((0, deg(45), 0))
            .scaled((1, 1, 1))
            .layer(1)
            .hidden();
        s.add(o);
    }
    // Builders with narrower targets.
    s.add(circle(10).shift(1, 2).on(Plane::Front));
    s.add(text("t").font_size(30).on(Plane::Ground));
    s.add(prism(circle(10), 5));
    s.add(prism(rect(80, 80).minus(circle(25)), 40).base_at((0, 0, 0)));
    // Booleans.
    for op in [
        circle(10).union(circle(10).shift(5, 0)),
        circle(10).minus(circle(5)),
        circle(10).intersect(circle(10).shift(5, 0)),
        circle(10).xor(circle(10).shift(5, 0)),
    ] {
        s.add(op);
    }
}

#[test]
fn every_animation_and_combinator_on_the_sheet_exists() {
    let mut s = Stage::new(900, 600).fps(10);
    let a = s.add(cube(10).base_at((0, 0, 0)));
    let b = s.add(circle(10).fill("mint"));
    let c = s.add(text("x"));
    let ghost = s.add(star(5, 20, 8).at((30, 0, 0)));

    s.play(create(a));
    s.play(write(c));
    s.play(fade_in(b));
    s.play(uncreate(a));
    s.play(create(a)); // bring it back
    s.play(fade_out(b));
    s.play(move_to(a, (5, 0, 5)));
    s.play(move_by(a, (5, 0, 0)));
    s.play(move_from(a, (0, 20, 0)));
    s.play(rotate_to(a, (0, deg(10), 0)));
    s.play(rotate_by(a, (0, deg(90), 0)));
    s.play(scale_to(a, (2, 2, 2)));
    s.play(opacity_to(a, 0.5));
    s.play(fill_to(a, "rose"));
    s.play(stroke_to(a, "#123456"));
    s.play(morph(b, star(5, 70, 30)));
    s.play(transform_into(b, ghost));
    s.play(wait(1));
    // Tweaks.
    s.play(move_to(a, (0, 0, 0)).duration(2).ease(Easing::BackOut).delay(0.5));
    s.play(move_to(a, (1, 0, 0)).then(fade_out(a)).with(fill_to(a, "sky")));
    // Groups, no boxing.
    s.play(all([fade_out(a), fade_out(c)]));
    s.play(seq([fade_in(a), fade_in(c)]));
    s.play(lag(0.3, [fade_out(a), fade_out(c), fade_out(b)]));
    s.wait(0.5);
    s.seek_end_for_test();
}

/// Stage's seek is `pub`; this just makes the loop above easier to read.
trait SeekEnd {
    fn seek_end_for_test(&mut self);
}
impl SeekEnd for Stage {
    fn seek_end_for_test(&mut self) {
        let end = self.duration();
        self.seek(end); // every animation must actually run without panicking
    }
}

#[test]
fn the_stage_builders_named_on_the_sheet_exist() {
    let s = Stage::new(900, 600)
        .fps(24)
        .background("white")
        .origin(100, 200)
        .wobble(8)
        .sketch(Sketch::clean());
    assert_eq!(s.fps, 24);
    // Sketch::default() exists too, and a custom roughness chains.
    let _ = Stage::new(10, 10).sketch(Sketch::default().roughness(2.0));
}

#[test]
fn the_quick_start_runs_end_to_end_to_a_png() {
    let mut s = Stage::new(900, 600);
    let tower = s.add(cuboid(80, 110, 80).base_at((0, 0, 0)).fill("purple"));
    let note = s.add(text("compute").at((0, 150, 0)));
    s.play(create(tower));
    s.play(write(note));
    s.play(all([move_by(tower, (100, 0, 0)), fill_to(tower, "rose")]).duration(1.5));
    s.wait(0.5);
    let png = std::env::temp_dir().join(format!("sheet-quickstart-{}.png", std::process::id()));
    s.render(&png).unwrap();
    assert!(std::fs::metadata(&png).unwrap().len() > 1000);
    let _ = std::fs::remove_file(png);
}

#[test]
fn colour_names_on_the_sheet_match_the_library() {
    let listed: Vec<&str> = SHEET
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|w| w.chars().all(|c| c.is_ascii_lowercase()))
        .collect();
    for name in &listed {
        if rough_animations::api::color::parse_color(name).is_none() && !is_non_colour(name) {
            panic!("the sheet lists {name:?} as a colour but it doesn't parse");
        }
    }
    // And the other way round: every real colour name is on the sheet (grey is an alias).
    for (name, _) in rough_animations::api::color::NAMES {
        if *name != "grey" {
            assert!(listed.contains(name), "colour {name:?} is missing from the sheet");
        }
    }
}

/// Quoted lowercase words on the sheet that are not colours (strings in example code).
fn is_non_colour(w: &str) -> bool {
    ["compute", "out", "hi", "sheet"].contains(&w)
}

#[test]
#[should_panic(expected = "one of: purple, mint")]
fn a_wrong_colour_panics_and_lists_the_valid_names() {
    let _ = cube(1).fill("purpel");
}

#[test]
fn world_axes_are_as_described() {
    // x to the lower right, z to the lower left, y up.
    let cam = rough_animations::IsoCamera::new(0.0, 0.0);
    let (o, x, y, z) = (
        cam.project(0., 0., 0.),
        cam.project(1., 0., 0.),
        cam.project(0., 1., 0.),
        cam.project(0., 0., 1.),
    );
    assert!(x.x > o.x && x.y > o.y, "x goes lower right");
    assert!(z.x < o.x && z.y > o.y, "z goes lower left");
    assert!(y.y < o.y && y.x == o.x, "y goes up the screen");
}

#[test]
fn defaults_match_the_sheet() {
    let s = Stage::new(900, 600);
    let cam = s.scene.camera();
    assert_eq!((cam.origin_x, cam.origin_y), (450.0, 360.0), "centre, 60% down");
    assert_eq!(s.fps, 30);
}

#[test]
fn solids_are_centred_and_base_at_stands_them_on_the_point() {
    let centred = cuboid(10, 40, 10).at((0, 100, 0)).bounds();
    assert_eq!((centred.min.y, centred.max.y), (80.0, 120.0), "centred on y = 100");
    let standing = cuboid(10, 40, 10).base_at((0, 100, 0)).bounds();
    assert_eq!((standing.min.y, standing.max.y), (100.0, 140.0), "bottom at y = 100");
}

#[test]
fn degrees_are_radians_underneath() {
    assert!((deg(90) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn the_gotchas_are_true() {
    // create/write/fade_in hide their object until they start...
    for make in [create as fn(Id) -> Anim, write, fade_in] {
        let mut s = Stage::new(100, 100);
        let a = s.add(cube(10));
        s.wait(1);
        s.play(make(a));
        assert!(!s.scene.get(a).unwrap().visible);
    }
    // ...but other animations don't.
    let mut s = Stage::new(100, 100);
    let a = s.add(cube(10));
    s.play(move_by(a, (1, 0, 0)));
    assert!(s.scene.get(a).unwrap().visible);

    // `.duration()` is the total, so `.delay()` goes after it.
    let mut s = Stage::new(100, 100);
    let a = s.add(cube(10));
    s.play(move_by(a, (1, 0, 0)).duration(1).delay(1));
    assert_eq!(s.duration(), 2.0);
}

#[test]
fn a_stage_plays_once_and_png_goes_last() {
    let dir = std::env::temp_dir().join(format!("sheet-once-{}", std::process::id()));
    let mut s = Stage::new(80, 60).fps(10);
    let a = s.add(cube(10));
    s.play(create(a).duration(0.3));
    s.render(dir.join("a.png")).unwrap(); // reaches the end
    // Rendering the same stage again from the start isn't possible; the sheet says to build a
    // fresh one per output. A second .png (the same final frame) is still fine:
    s.render(dir.join("b.png")).unwrap();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn unsupported_output_extensions_are_rejected_with_the_valid_ones() {
    let mut s = Stage::new(40, 40);
    let err = s.render("nope.avi").unwrap_err().to_string();
    for ext in [".mp4", ".webm", ".gif", ".png"] {
        assert!(err.contains(ext), "{err}");
    }
}

#[test]
fn a_video_after_a_png_is_a_clear_error() {
    let dir = std::env::temp_dir().join(format!("sheet-order-{}", std::process::id()));
    let mut s = Stage::new(80, 60).fps(10);
    let a = s.add(cube(10));
    s.play(create(a).duration(0.3));
    s.render(dir.join("a.png")).unwrap();
    let err = s.render(dir.join("a.mp4")).unwrap_err().to_string();
    assert!(err.contains("fresh Stage"), "{err}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn floats_work_wherever_the_sheet_says_numbers_do() {
    let mut s = Stage::new(900.0, 600.0).fps(30.0).origin(450.5, 360.5).wobble(8.0);
    let a = s.add(cuboid(80.5, 110.0, 80).base_at((0.5, 20.0, 0)).opacity(0.5).stroke_width(2.5));
    s.play(move_by(a, (1.5, 0.0, 2)).duration(1.5));
    s.wait(0.25);
    s.add(ngon(6, 10.5));
    s.add(star(5, 10.0, 4.5));
}

#[test]
fn nothing_stale_is_on_the_sheet() {
    for gone in ["Timeline", "boxed", "parallel(", "sequence(", "lagged_start", "rgb(", "Box3D", "OptionsBuilder"] {
        assert!(!SHEET.contains(gone), "the sheet still mentions {gone:?}");
    }
}

#[test]
fn latex_functions_work_when_installed() {
    let have = std::process::Command::new("pdflatex").arg("--version").output().is_ok()
        && (std::process::Command::new("pdftocairo").arg("-v").output().is_ok()
            || std::process::Command::new("mutool").arg("-v").output().is_ok());
    if !have {
        return;
    }
    let mut s = Stage::new(200, 100);
    s.add(math(r"\frac12").unwrap().font_size(30));
    s.add(latex(r"\textbf{Bold} and $x^2$").unwrap());
}
