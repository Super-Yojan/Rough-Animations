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
    // Text builders (and they are harmless on shapes).
    s.add(text("t").font_size(30).tracking(0.1).animate().solid().hatched());
    s.add(text("t").stroke_width(1));
    s.add(text("t").handwritten().animate().tracking(0.1));
    s.add(cube(10).tracking(0.1).animate().handwritten());
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
fn the_themes_on_the_sheet_exist_work_and_behave_as_described() {
    let section = SHEET
        .split("## Themes")
        .nth(1)
        .and_then(|rest| rest.split("\n## ").next())
        .expect("the sheet has a Themes section");
    // Every built-in theme is documented, and each one works by the name the sheet gives.
    for name in rough_animations::api::theme::NAMES {
        assert!(section.contains(&format!("\"{name}\"")), "the sheet does not mention theme {name:?}");
        let mut s = Stage::new(300, 200).theme(*name);
        s.heading("T", "c");
        s.add(cube(20).base_at((0, 0, 0)));
        let end = s.duration();
        s.render_frame(end);
    }
    // The documented custom-theme chain, and the other ways to pass a theme.
    let ocean = Theme::custom("#06283d", "#e0f2ff").title("#7dd3fc").caption("gray").ink("white").background("#000").sketch(Sketch::clean());
    let _ = Stage::new(300, 200).theme(ocean);
    let _ = Stage::new(300, 200).theme(Theme::chalkboard()).theme("sepia");
    let _ = Theme::by_name("dark").unwrap();

    // "Anything you colour yourself is left alone" and "the theme reaches objects added before it".
    let mut s = Stage::new(300, 200);
    let before = s.add(text("early"));
    let mut s = s.theme("dark");
    let after = s.add(text("late"));
    let own = s.add(text("mine").color("rose"));
    let fill = |s: &Stage, id: Id| s.scene.get(id).unwrap().style.fill;
    assert_eq!(fill(&s, before), fill(&s, after), "objects added before and after look the same");
    assert_eq!(fill(&s, own), Some("rose".into_color()), "an explicit colour survives");
    assert_ne!(fill(&s, after), Some("rose".into_color()));

    // The gotcha: a dark `background` alone leaves dark ink, a theme does not.
    let dark_ink = Theme::dark().ink;
    let mut bare = Stage::new(300, 200).background("#101018");
    let t = bare.add(text("x"));
    assert_ne!(fill(&bare, t), Some(dark_ink));
    let mut themed = Stage::new(300, 200).theme("dark");
    let t = themed.add(text("x"));
    assert_eq!(fill(&themed, t), Some(dark_ink));
    assert!(SHEET.contains("`.steady()`"), "the sheet mentions steady()");
}

#[test]
fn the_group_api_on_the_sheet_works_as_described() {
    let mut s = Stage::new(900, 600);
    let mut car = Group::new();
    let body = s.add_to(&mut car, cuboid(60, 20, 30).base_at((0, 10, 0)).fill("white"));
    let wheel = s.add_to(&mut car, cylinder(8, 6).at((20, 8, 16)).rotated((deg(90), 0, 0)));
    assert_eq!(car.len(), 2, "add_to puts the part in the group, and returns its id");

    // Every animation named on the sheet, and each returns an Anim that takes the usual tweaks.
    s.play(car.create());
    s.play(car.fade_in().duration(1));
    s.play(car.move_by((10, 0, 0)).duration(1).ease(Easing::Linear));
    s.play(car.move_to((50, 0, 20)));
    s.play(car.rotate_by((0, deg(90), 0)));
    s.play(car.clone().pivot((0, 0, 0)).rotate_by((0, deg(45), 0)));
    s.play(car.scale_by(2));
    s.play(car.opacity_to(0.5));
    s.play(car.fade_out().delay(0.2));
    s.play(car.write());
    s.play(car.uncreate());
    s.play(all([car.move_by((1, 0, 0)), fade_in(wheel)]));
    s.play(create_all(&car));
    s.play(write_all(&car));
    s.play(clear(&car));
    s.place(&car, (0, 0, 0));
    let end = s.duration();
    s.seek(end);

    // The claims: move_to puts the bottom centre at the target; place does the same instantly.
    let mut s = Stage::new(900, 600);
    let mut g = Group::new();
    s.add_to(&mut g, cuboid(10, 10, 10).base_at((0, 0, 0)));
    s.add_to(&mut g, cuboid(10, 10, 10).base_at((20, 0, 0)));
    s.place(&g, (100, 3, 40));
    let b = g.bounds(&s.scene).unwrap();
    assert!(((b.min.x + b.max.x) / 2.0 - 100.0).abs() < 1e-3 && (b.min.y - 3.0).abs() < 1e-3);
    let _ = body;
}

#[test]
fn the_layout_helpers_on_the_sheet_work_as_described() {
    let mut s = Stage::new(1280, 720);
    // put: text is centred on the pixel, solids stand on it.
    let hello = s.put(title("Hello"), 640, 80);
    let note = s.put(label("note"), 200, 600);
    let tower = s.put(cuboid(60, 90, 60).fill("mint"), 900, 500);
    let (x, y) = s.screen(s.scene.get(hello).unwrap().transform.position);
    assert!((x - 640.0).abs() < 0.5 && (y - 80.0).abs() < 0.5);
    assert!(s.scene.get(tower).unwrap().bounds().min.y.abs() < 1e-3);

    // Defaults: label is steady, title is lively and bigger.
    let (lw, lh) = s.scene.get(note).unwrap().dimensions().unwrap();
    let (_, th) = s.scene.get(hello).unwrap().dimensions().unwrap();
    assert!(lw > 0.0 && th > lh);

    // tag: every side, with a string and with a styled label.
    let tags = [
        s.tag(tower, "above", Side::Above),
        s.tag(tower, "below", Side::Below),
        s.tag(tower, "left", Side::Left),
        s.tag(tower, label("right").color("rose"), Side::Right),
    ];
    assert_eq!(tags.len(), 4);

    // A title or label added with plain `add` and no position goes to its default spot.
    let mut d = Stage::new(1280, 720);
    let (t, l) = (d.add(title("Default")), d.add(label("under it")));
    let spot = |d: &Stage, id: Id| d.screen(d.scene.get(id).unwrap().transform.position);
    let ((tx, ty), (lx, ly)) = (spot(&d, t), spot(&d, l));
    assert!((tx - 640.0).abs() < 0.5 && (lx - 640.0).abs() < 0.5, "centred");
    assert!(ty < ly && ty < 720.0 * 0.2, "title at the top, label under it");

    // heading returns the header and the caption (just the header for an empty caption).
    assert_eq!(s.heading("Solo", "").len(), 1);
    let head = s.heading("Title", "caption");
    assert_eq!(head.len(), 2);

    // The id-list helpers take arrays, Vecs and references.
    s.play(write_all(&head));
    s.play(create_all([tower]));
    s.play(write_all(tags));
    s.play(clear(&head));
    s.play(clear(vec![tower, note]));
    let end = s.duration();
    s.seek(end);
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
    // `zoom` and `pixel`: text put at a pixel really lands on that pixel.
    let s = Stage::new(1280, 720).zoom(1.4);
    let w = s.pixel(640, 80);
    let p = s.scene.camera().project(w.x, w.y, w.z);
    assert!((p.x - 640.0).abs() < 1e-3 && (p.y - 80.0).abs() < 1e-3);
    let mut s = s;
    s.add(text("Title").at(s.pixel(640, 80)));
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
    // Only the paragraph that lists the colours: it starts at "Colours:" and ends at a blank line.
    let start = SHEET.find("Colours:").expect("the sheet lists colours");
    let paragraph = &SHEET[start..SHEET[start..].find("\n\n").map_or(SHEET.len(), |n| start + n)];
    let listed: Vec<&str> = paragraph
        .split('"')
        .skip(1)
        .step_by(2)
        .filter(|w| !w.starts_with('#') && w.chars().all(|c| c.is_ascii_lowercase()))
        .collect();
    assert!(listed.len() >= 10, "found only {listed:?}");
    for name in &listed {
        assert!(
            rough_animations::api::color::parse_color(name).is_some(),
            "the sheet lists {name:?} as a colour but it doesn't parse"
        );
    }
    // And the other way round: every real colour name is on the sheet (grey is an alias).
    for (name, _) in rough_animations::api::color::NAMES {
        if *name != "grey" {
            assert!(listed.contains(name), "colour {name:?} is missing from the sheet");
        }
    }
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
    s.add(math(r"\frac12").unwrap().handwritten());
}

#[test]
fn turning_picking_and_vector_output_on_the_sheet_work() {
    let mut s = Stage::new(400, 300).yaw(deg(30)).pixel_ratio(1.0);
    let part = s.add(cube(60).base_at((0, 0, 0)).fill("purple").action("open"));
    s.play(create(part));
    s.seek(s.duration());
    s.set_yaw(deg(60));
    // The cube stands at the origin, which is at 50% x and 60% down by default.
    let (id, action) = s.pick(200.0, 180.0).expect("the cube is under its own centre");
    assert_eq!((id, action), (part, Some("open")));
    assert!(s.pick(2.0, 2.0).is_none());
    let svg = s.to_svg();
    assert!(svg.starts_with("<svg ") && svg.contains("<path"));
}
