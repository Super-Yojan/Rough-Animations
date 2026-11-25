//! Build a [`Stage`] from a JSON description, so scenes can be written without Rust.
//!
//! This is what the browser components feed the engine. The format mirrors the Rust API one to
//! one: an object is a `kind` plus the builder options you would chain, an animation is an `op`
//! plus its target. Angles are in **degrees**. Unknown fields are rejected, so a typo is an
//! error instead of being silently ignored.
//!
//! ```json
//! {
//!   "width": 900, "height": 600, "theme": "light",
//!   "objects": [
//!     { "id": "tower", "kind": "cuboid", "size": [80, 110, 80], "base_at": [0, 0, 0],
//!       "fill": "purple", "action": "open" }
//!   ],
//!   "timeline": [
//!     { "op": "create", "target": "tower" },
//!     { "op": "move_by", "target": "tower", "by": [100, 0, 0], "duration": 1.5 }
//!   ]
//! }
//! ```

use crate::api::color::parse_color;
use crate::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// A whole figure: canvas, camera, look, objects and the animation timeline.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub fps: Option<f32>,
    /// Scene scale; see `Stage::zoom`.
    #[serde(default)]
    pub zoom: Option<f32>,
    /// Screen pixel where `(0, 0, 0)` lands.
    #[serde(default)]
    pub origin: Option<[f32; 2]>,
    /// Turn of the scene about the vertical axis, in degrees.
    #[serde(default)]
    pub yaw: Option<f32>,
    /// A theme name: `light`, `dark`, `chalkboard`, `blueprint`, `sepia` or `terminal`.
    #[serde(default)]
    pub theme: Option<String>,
    /// Page colour. Use `"transparent"` to let the page behind the canvas show.
    #[serde(default)]
    pub background: Option<String>,
    /// `"sketch"` (default) or `"clean"` for straight lines.
    #[serde(default)]
    pub look: Option<String>,
    #[serde(default)]
    pub roughness: Option<f32>,
    /// Redraw strokes this many times a second ("boiling line").
    #[serde(default)]
    pub wobble: Option<f32>,
    #[serde(default)]
    pub objects: Vec<ObjectSpec>,
    #[serde(default)]
    pub timeline: Vec<AnimSpec>,
}

/// One object. `kind` picks the shape; the other fields are the parameters that shape needs
/// plus the builders that apply to any object.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSpec {
    /// A name animations refer to. Optional for objects that are never animated.
    #[serde(default)]
    pub id: Option<String>,
    pub kind: String,

    // Shape parameters.
    #[serde(default)]
    pub size: Option<Value>, // cuboid: [w, h, d]; cube: number
    #[serde(default)]
    pub radius: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub rx: Option<f32>,
    #[serde(default)]
    pub ry: Option<f32>,
    #[serde(default)]
    pub sides: Option<usize>,
    #[serde(default)]
    pub points: Option<Value>, // star: number of points; polygon: [[x, y], ..]
    #[serde(default)]
    pub outer: Option<f32>,
    #[serde(default)]
    pub inner: Option<f32>,
    #[serde(default)]
    pub from: Option<[f32; 3]>,
    #[serde(default)]
    pub to: Option<[f32; 3]>,
    #[serde(default)]
    pub text: Option<String>,
    /// `prism`: the flat shape to extrude.
    #[serde(default)]
    pub shape: Option<Box<ObjectSpec>>,

    // Boolean operations on flat shapes, applied before placement.
    #[serde(default)]
    pub union: Option<Box<ObjectSpec>>,
    #[serde(default)]
    pub minus: Option<Box<ObjectSpec>>,
    #[serde(default)]
    pub intersect: Option<Box<ObjectSpec>>,
    #[serde(default)]
    pub xor: Option<Box<ObjectSpec>>,

    // Builders.
    #[serde(default)]
    pub at: Option<[f32; 3]>,
    #[serde(default)]
    pub base_at: Option<[f32; 3]>,
    /// Rotation in degrees about x, y, z.
    #[serde(default)]
    pub rotated: Option<[f32; 3]>,
    #[serde(default)]
    pub scaled: Option<Value>, // number or [x, y, z]
    #[serde(default)]
    pub fill: Option<String>,
    #[serde(default)]
    pub stroke: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub stroke_width: Option<f32>,
    /// `"front"`, `"side"` or `"ground"`: lay a flat shape or text in a plane.
    #[serde(default)]
    pub plane: Option<String>,
    /// Shift in the shape's own plane.
    #[serde(default)]
    pub shift: Option<[f32; 2]>,
    #[serde(default)]
    pub solid: Option<bool>,
    #[serde(default)]
    pub hatched: Option<bool>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub tracking: Option<f32>,
    #[serde(default)]
    pub animate: Option<bool>,
    #[serde(default)]
    pub handwritten: Option<bool>,
    #[serde(default)]
    pub layer: Option<i32>,
    #[serde(default)]
    pub hidden: Option<bool>,
    /// A name reported when this part is clicked.
    #[serde(default)]
    pub action: Option<String>,
}

/// One step of the timeline. `op` picks the animation; the other fields are its arguments.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimSpec {
    pub op: String,
    #[serde(default)]
    pub target: Option<String>,
    /// For `create_all`, `write_all` and `clear`.
    #[serde(default)]
    pub targets: Option<Vec<String>>,
    /// Destination: a position, rotation (degrees), scale, opacity or colour, by `op`.
    #[serde(default)]
    pub to: Option<Value>,
    /// Offset for `move_by`, `move_from` and `rotate_by` (degrees).
    #[serde(default)]
    pub by: Option<[f32; 3]>,
    #[serde(default)]
    pub duration: Option<f32>,
    #[serde(default)]
    pub delay: Option<f32>,
    #[serde(default)]
    pub ease: Option<String>,
    /// Seconds for `wait`, or the gap for `lag`.
    #[serde(default)]
    pub seconds: Option<f32>,
    /// Children of `all`, `seq` and `lag`.
    #[serde(default)]
    pub anims: Option<Vec<AnimSpec>>,
    /// `morph`: the shape to turn into.
    #[serde(default)]
    pub shape: Option<Box<ObjectSpec>>,
}

/// A built stage with the names of its objects.
pub struct Built {
    pub stage: Stage,
    pub names: Vec<(Id, String)>,
}

impl Spec {
    pub fn parse(json: &str) -> Result<Spec, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid figure: {e}"))
    }

    /// Build a fresh stage. Call it again to start over from time 0, which is how a player
    /// rewinds: a stage only plays forward.
    pub fn build(&self) -> Result<Built, String> {
        self.build_at(1.0)
    }

    /// Like [`build`](Self::build), drawn at `ratio` device pixels per figure pixel. The stage
    /// is `ratio` times bigger and everything in it scales to match, so the picture is the
    /// same, only sharper.
    pub fn build_at(&self, ratio: f32) -> Result<Built, String> {
        if !(self.width >= 1.0 && self.height >= 1.0) {
            return Err("width and height must be at least 1".into());
        }
        if !(ratio.is_finite() && ratio > 0.0 && ratio <= 8.0) {
            return Err("pixel ratio must be between 0 and 8".into());
        }
        let mut stage = Stage::new((self.width * ratio).round(), (self.height * ratio).round())
            .pixel_ratio(ratio);
        if let Some(name) = &self.theme {
            if Theme::by_name(name).is_none() {
                return Err(format!("unknown theme {name:?}"));
            }
            stage = stage.theme(name.as_str());
        }
        if let Some(fps) = self.fps {
            stage = stage.fps(fps);
        }
        // Without an explicit origin the default (centre, 60% down) already follows the size.
        if let Some([x, y]) = self.origin {
            stage = stage.origin(x * ratio, y * ratio);
        }
        stage = stage.zoom(self.zoom.unwrap_or(1.0) * ratio);
        if let Some(yaw) = self.yaw {
            stage = stage.yaw(yaw.to_radians());
        }
        match self.look.as_deref() {
            None | Some("sketch") => {}
            Some("clean") => stage = stage.sketch(Sketch::clean()),
            Some(other) => return Err(format!("unknown look {other:?}: use \"sketch\" or \"clean\"")),
        }
        if let Some(r) = self.roughness {
            stage = stage.sketch(Sketch::default().roughness(r));
        }
        if let Some(bg) = &self.background {
            stage = if bg.eq_ignore_ascii_case("transparent") {
                stage.background(tiny_skia::Color::TRANSPARENT)
            } else {
                stage.background(color(bg)?)
            };
        }
        if let Some(w) = self.wobble {
            stage = stage.wobble(w);
        }

        let mut ids: HashMap<String, Id> = HashMap::new();
        let mut names = Vec::new();
        for o in &self.objects {
            let id = stage.add(build_object(o)?);
            if let Some(name) = &o.id {
                if ids.insert(name.clone(), id).is_some() {
                    return Err(format!("two objects are named {name:?}"));
                }
                names.push((id, name.clone()));
            }
        }
        for a in &self.timeline {
            stage.play(build_anim(a, &ids)?);
        }
        Ok(Built { stage, names })
    }
}

/// Check a colour so a typo is an error, not a panic.
fn color(s: &str) -> Result<&str, String> {
    parse_color(s)
        .map(|_| s)
        .ok_or_else(|| format!("unknown colour {s:?}: use a name like \"purple\" or \"#rrggbb\""))
}

fn need<T: Copy>(v: Option<T>, kind: &str, field: &str) -> Result<T, String> {
    v.ok_or_else(|| format!("{kind} needs \"{field}\""))
}

fn vec3(v: &Value, what: &str) -> Result<[f32; 3], String> {
    let a = v.as_array().filter(|a| a.len() == 3);
    a.and_then(|a| Some([a[0].as_f64()? as f32, a[1].as_f64()? as f32, a[2].as_f64()? as f32]))
        .ok_or_else(|| format!("{what} must be [x, y, z]"))
}

fn build_object(o: &ObjectSpec) -> Result<Object, String> {
    let k = o.kind.as_str();
    let mut obj = match k {
        "cuboid" => {
            let s = o.size.as_ref().ok_or("cuboid needs \"size\": [w, h, d]")?;
            let [w, h, d] = vec3(s, "cuboid size")?;
            cuboid(w, h, d)
        }
        "cube" => cube(o.size.as_ref().and_then(Value::as_f64).ok_or("cube needs \"size\": number")? as f32),
        "cylinder" => cylinder(need(o.radius, k, "radius")?, need(o.height, k, "height")?),
        "cone" => cone(need(o.radius, k, "radius")?, need(o.height, k, "height")?),
        "sphere" => sphere(need(o.radius, k, "radius")?),
        "circle" => circle(need(o.radius, k, "radius")?),
        "ellipse" => ellipse(need(o.rx, k, "rx")?, need(o.ry, k, "ry")?),
        "rect" => rect(need(o.width, k, "width")?, need(o.height, k, "height")?),
        "ngon" => ngon(need(o.sides, k, "sides")?, need(o.radius, k, "radius")?),
        "star" => star(
            o.points.as_ref().and_then(Value::as_u64).ok_or("star needs \"points\": number")? as usize,
            need(o.outer, k, "outer")?,
            need(o.inner, k, "inner")?,
        ),
        "polygon" => {
            let pts = o.points.as_ref().and_then(Value::as_array).ok_or("polygon needs \"points\": [[x, y], ..]")?;
            let pts: Option<Vec<(f32, f32)>> = pts
                .iter()
                .map(|p| {
                    let p = p.as_array().filter(|p| p.len() == 2)?;
                    Some((p[0].as_f64()? as f32, p[1].as_f64()? as f32))
                })
                .collect();
            polygon(pts.ok_or("polygon points must be [x, y] pairs")?)
        }
        "prism" => {
            let shape = o.shape.as_ref().ok_or("prism needs \"shape\": a flat shape")?;
            prism(build_object(shape)?, need(o.height, k, "height")?)
        }
        "line" => line(need(o.from, k, "from")?, need(o.to, k, "to")?),
        "arrow" => arrow(need(o.from, k, "from")?, need(o.to, k, "to")?),
        "text" => text(o.text.as_deref().ok_or("text needs \"text\"")?),
        "label" => label(o.text.as_deref().ok_or("label needs \"text\"")?),
        "title" => title(o.text.as_deref().ok_or("title needs \"text\"")?),
        other => {
            return Err(format!(
                "unknown kind {other:?}: cuboid cube cylinder cone sphere circle ellipse rect ngon \
                 star polygon prism line arrow text label title"
            ));
        }
    };

    match o.plane.as_deref() {
        None => {}
        Some("front") => obj = obj.on(Plane::Front),
        Some("side") => obj = obj.on(Plane::Side),
        Some("ground") => obj = obj.on(Plane::Ground),
        Some(p) => return Err(format!("unknown plane {p:?}: front, side or ground")),
    }
    // Booleans first: combine shapes before they are placed.
    for (op, other) in [("union", &o.union), ("minus", &o.minus), ("intersect", &o.intersect), ("xor", &o.xor)] {
        if let Some(other) = other {
            let other = build_object(other)?;
            obj = match op {
                "union" => obj.union(other),
                "minus" => obj.minus(other),
                "intersect" => obj.intersect(other),
                _ => obj.xor(other),
            };
        }
    }
    if let Some([x, y, z]) = o.rotated {
        obj = obj.rotated((x.to_radians(), y.to_radians(), z.to_radians()));
    }
    if let Some(s) = &o.scaled {
        obj = match s.as_f64() {
            Some(n) => obj.scaled((n as f32, n as f32, n as f32)),
            None => obj.scaled(vec3(s, "scaled")?),
        };
    }
    if let Some(c) = &o.fill {
        obj = obj.fill(color(c)?);
    }
    if let Some(c) = &o.stroke {
        obj = obj.stroke(color(c)?);
    }
    if let Some(c) = &o.color {
        obj = obj.color(color(c)?);
    }
    if let Some(v) = o.opacity {
        obj = obj.opacity(v);
    }
    if let Some(v) = o.stroke_width {
        obj = obj.stroke_width(v);
    }
    if let Some([dx, dy]) = o.shift {
        obj = obj.shift(dx, dy);
    }
    if o.solid == Some(true) {
        obj = obj.solid();
    }
    if o.hatched == Some(true) {
        obj = obj.hatched();
    }
    if let Some(v) = o.font_size {
        obj = obj.font_size(v);
    }
    if let Some(v) = o.tracking {
        obj = obj.tracking(v);
    }
    if o.animate == Some(true) {
        obj = obj.animate();
    }
    if o.handwritten == Some(true) {
        obj = obj.handwritten();
    }
    if let Some(p) = o.base_at {
        obj = obj.base_at(p);
    } else if let Some(p) = o.at {
        obj = obj.at(p);
    }
    if let Some(l) = o.layer {
        obj = obj.layer(l);
    }
    if o.hidden == Some(true) {
        obj = obj.hidden();
    }
    if let Some(a) = &o.action {
        obj = obj.action(a);
    }
    Ok(obj)
}

fn ease(name: &str) -> Result<Easing, String> {
    Ok(match name.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
        "linear" => Easing::Linear,
        "quadin" => Easing::QuadIn,
        "quadout" => Easing::QuadOut,
        "quadinout" => Easing::QuadInOut,
        "cubicin" => Easing::CubicIn,
        "cubicout" => Easing::CubicOut,
        "cubicinout" | "smooth" => Easing::CubicInOut,
        "sineinout" => Easing::SineInOut,
        "backout" => Easing::BackOut,
        "bounceout" => Easing::BounceOut,
        _ => return Err(format!("unknown easing {name:?}")),
    })
}

fn build_anim(a: &AnimSpec, ids: &HashMap<String, Id>) -> Result<Anim, String> {
    let op = a.op.as_str();
    let id_of = |name: &str| ids.get(name).copied().ok_or_else(|| format!("{op}: no object named {name:?}"));
    let target = || {
        let t = a.target.as_deref().ok_or_else(|| format!("{op} needs \"target\""))?;
        id_of(t)
    };
    let target_list = || -> Result<Vec<Id>, String> {
        a.targets
            .as_ref()
            .ok_or_else(|| format!("{op} needs \"targets\""))?
            .iter()
            .map(|t| id_of(t))
            .collect()
    };
    let to = || a.to.as_ref().ok_or_else(|| format!("{op} needs \"to\""));
    let by = || a.by.ok_or_else(|| format!("{op} needs \"by\""));
    let children = || -> Result<Vec<Anim>, String> {
        a.anims
            .as_ref()
            .ok_or_else(|| format!("{op} needs \"anims\""))?
            .iter()
            .map(|c| build_anim(c, ids))
            .collect()
    };
    let deg3 = |v: [f32; 3]| (v[0].to_radians(), v[1].to_radians(), v[2].to_radians());

    let mut anim = match op {
        "create" => create(target()?),
        "write" => write(target()?),
        "fade_in" => fade_in(target()?),
        "uncreate" => uncreate(target()?),
        "fade_out" => fade_out(target()?),
        "move_to" => move_to(target()?, vec3(to()?, "to")?),
        "move_by" => move_by(target()?, by()?),
        "move_from" => move_from(target()?, by()?),
        "rotate_to" => rotate_to(target()?, deg3(vec3(to()?, "to")?)),
        "rotate_by" => rotate_by(target()?, deg3(by()?)),
        "scale_to" => {
            let v = to()?;
            match v.as_f64() {
                Some(n) => scale_to(target()?, (n as f32, n as f32, n as f32)),
                None => scale_to(target()?, vec3(v, "to")?),
            }
        }
        "opacity_to" => opacity_to(target()?, to()?.as_f64().ok_or("opacity_to: \"to\" must be a number")? as f32),
        "fill_to" => fill_to(target()?, color(to()?.as_str().ok_or("fill_to: \"to\" must be a colour")?)?),
        "stroke_to" => stroke_to(target()?, color(to()?.as_str().ok_or("stroke_to: \"to\" must be a colour")?)?),
        "morph" => {
            let shape = a.shape.as_ref().ok_or("morph needs \"shape\"")?;
            morph(target()?, build_object(shape)?)
        }
        "wait" => wait(a.seconds.or(a.duration).ok_or("wait needs \"seconds\"")?),
        "all" => all(children()?),
        "seq" => seq(children()?),
        "lag" => lag(a.seconds.ok_or("lag needs \"seconds\" (the gap)")?, children()?),
        "create_all" => create_all(target_list()?),
        "write_all" => write_all(target_list()?),
        "clear" => clear(target_list()?),
        other => {
            return Err(format!(
                "unknown op {other:?}: create write fade_in uncreate fade_out move_to move_by move_from \
                 rotate_to rotate_by scale_to opacity_to fill_to stroke_to morph wait all seq lag \
                 create_all write_all clear"
            ));
        }
    };
    if let Some(d) = a.duration {
        if op != "wait" {
            anim = anim.duration(d);
        }
    }
    if let Some(e) = &a.ease {
        anim = anim.ease(ease(e)?);
    }
    if let Some(d) = a.delay {
        anim = anim.delay(d);
    }
    Ok(anim)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOWER: &str = r#"{
        "width": 400, "height": 300,
        "objects": [
            {"id": "tower", "kind": "cuboid", "size": [80, 110, 80], "base_at": [0, 0, 0],
             "fill": "purple", "action": "open"},
            {"kind": "prism", "shape": {"kind": "rect", "width": 60, "height": 60,
                                        "minus": {"kind": "circle", "radius": 16}},
             "height": 14, "base_at": [120, 0, 0], "fill": "rose"}
        ],
        "timeline": [
            {"op": "create", "target": "tower"},
            {"op": "move_by", "target": "tower", "by": [60, 0, 0], "duration": 1, "ease": "back_out"}
        ]
    }"#;

    #[test]
    fn a_figure_builds_and_plays() {
        let built = Spec::parse(TOWER).unwrap().build().unwrap();
        let mut stage = built.stage;
        assert_eq!(built.names.len(), 1);
        assert!(stage.duration() > 1.0);
        let end = stage.duration();
        let pix = stage.render_frame(end);
        assert_eq!((pix.width(), pix.height()), (400, 300));
        assert!(pix.data().iter().any(|&b| b != 0));
    }

    #[test]
    fn picking_finds_the_actionable_part() {
        let mut stage = Spec::parse(TOWER).unwrap().build().unwrap().stage;
        let end = stage.duration();
        stage.seek(end);
        // The tower stands at the origin, which is at 50% x and 60% down; after its move it
        // is 60 units along +x. Probe its screen centre.
        let p = stage.scene.camera().project(60.0, 55.0, 0.0);
        let (_, action) = stage.pick(p.x, p.y).expect("a hit");
        assert_eq!(action, Some("open"));
        assert!(stage.pick(5.0, 5.0).is_none());
    }

    #[test]
    fn mistakes_are_errors_not_panics() {
        for (json, needle) in [
            (r#"{"width":10,"height":10,"objects":[{"kind":"blob"}]}"#, "unknown kind"),
            (r#"{"width":10,"height":10,"objects":[{"kind":"sphere","radius":5,"fill":"nope"}]}"#, "unknown colour"),
            (r#"{"width":10,"height":10,"objects":[{"kind":"sphere"}]}"#, "needs \"radius\""),
            (r#"{"width":10,"height":10,"timeline":[{"op":"create","target":"x"}]}"#, "no object named"),
            (r#"{"width":10,"height":10,"oops":1}"#, "unknown field"),
            (r#"{"width":10,"height":10,"theme":"neon"}"#, "unknown theme"),
        ] {
            let err = Spec::parse(json).and_then(|s| s.build().map(|_| ())).unwrap_err();
            assert!(err.contains(needle), "{json}: {err}");
        }
    }

    #[test]
    fn a_higher_pixel_ratio_is_the_same_picture_only_sharper() {
        let spec = Spec::parse(TOWER).unwrap();
        let ink = |ratio: f32| {
            let mut s = spec.build_at(ratio).unwrap().stage;
            let end = s.duration();
            let pix = s.render_frame(end);
            let dark = pix.pixels().iter().filter(|p| p.red() < 128).count() as f32;
            (pix.width(), pix.height(), dark / (pix.width() * pix.height()) as f32)
        };
        let (w1, h1, c1) = ink(1.0);
        let (w2, h2, c2) = ink(2.0);
        assert_eq!((w2, h2), (w1 * 2, h1 * 2));
        // Ink coverage (fraction of dark pixels) matches within a modest margin: lines and
        // hatching grew with the canvas instead of staying hair-thin.
        assert!((c2 / c1 - 1.0).abs() < 0.25, "coverage {c1} vs {c2}");
    }

    #[test]
    fn svg_export_is_valid_and_has_the_drawing() {
        let mut stage = Spec::parse(TOWER).unwrap().build().unwrap().stage;
        let end = stage.duration();
        stage.seek(end);
        let svg = stage.to_svg();
        assert!(svg.starts_with("<svg ") && svg.trim_end().ends_with("</svg>"));
        assert!(svg.matches("<path").count() > 20, "outlines and hatching are paths");
        assert!(svg.contains("stroke-linecap"));
        // A real SVG parser accepts it, at the size we declared.
        let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("valid svg");
        assert_eq!((tree.size().width(), tree.size().height()), (400.0, 300.0));

        // Transparent page: no background rectangle, and still valid.
        let spec = r#"{"width":200,"height":150,"background":"transparent",
            "objects":[{"kind":"sphere","radius":30,"base_at":[0,0,0],"fill":"sky"}]}"#;
        let mut s = Spec::parse(spec).unwrap().build().unwrap().stage;
        let svg = s.to_svg();
        usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("valid svg");
        assert!(!svg.contains("M0 0H200V150H0Z"));
    }

    #[test]
    fn rebuilding_rewinds() {
        let spec = Spec::parse(TOWER).unwrap();
        let mut a = spec.build().unwrap().stage;
        let end = a.duration();
        let late = a.render_frame(end);
        let mut b = spec.build().unwrap().stage;
        let early = b.render_frame(0.0);
        assert_ne!(late.data(), early.data());
    }
}
