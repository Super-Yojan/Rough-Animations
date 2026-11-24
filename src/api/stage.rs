use crate::animation::{Animation, Group};
use crate::api::{Error, IntoColor, Num, Result, Sketch};
use crate::render::camera::IsoCamera;
use crate::render::export::{VideoEncoder, VideoFormat, VideoOptions};
use crate::world::arena::Id;
use crate::world::object::{Home, Object};
use crate::world::scene::Scene;
use crate::world::transform::Vec3;
use std::collections::HashSet;
use std::path::Path;
use tiny_skia::{Color, Pixmap};

/// A scene plus a script of animations on a clock: add objects, `play` animations, `render`.
///
/// ```no_run
/// use rough_animations::prelude::*;
///
/// fn main() -> Result {
///     let mut s = Stage::new(900, 600);
///     let cube = s.add(cuboid(80, 80, 80).base_at((0, 20, 0)).fill("purple"));
///     let label = s.add(text("hello").at((0, 130, 0)));
///     s.play(create(cube));
///     s.play(write(label));
///     s.play(move_by(cube, (100, 0, 0)).duration(1.5));
///     s.wait(0.5);
///     s.render("out.mp4") // needs ffmpeg; use "out.png" for just the last frame
/// }
/// ```
///
/// Objects that a `create`, `fade_in` or `write` brings in are hidden until that animation
/// starts, so you don't need to hide them yourself.
pub struct Stage {
    pub scene: Scene,
    pub width: u32,
    pub height: u32,
    pub background: Color,
    /// Frames per second for `render`.
    pub fps: u32,
    /// Reseed rough strokes this many times per second for a "boiling line" look.
    /// `None` (default) keeps strokes perfectly still.
    pub wobble_fps: Option<f32>,
    script: Group,
    cursor: f32,
    /// Time up to which the script has been applied to the scene.
    played_to: f32,
    /// Objects already brought in by an animation (only the first one hides them).
    introduced: HashSet<Id>,
}

impl Stage {
    /// A `width` x `height` canvas with the hand-drawn look, an isometric camera whose origin
    /// is at the horizontal centre and 60% down, and 30 fps. Tune it with the builder methods
    /// ([`fps`](Self::fps), [`background`](Self::background), [`origin`](Self::origin),
    /// [`sketch`](Self::sketch), [`wobble`](Self::wobble)).
    pub fn new(width: impl Num, height: impl Num) -> Self {
        let (width, height) = (width.f32().max(1.0) as u32, height.f32().max(1.0) as u32);
        let camera = IsoCamera::new(width as f32 / 2.0, height as f32 * 0.6);
        Self::with(Scene::new(camera, Sketch::default().options()), width, height)
    }

    /// A stage around a scene you built yourself (custom camera or rough options).
    pub fn with(scene: Scene, width: u32, height: u32) -> Self {
        Self {
            scene,
            width,
            height,
            background: Color::from_rgba8(250, 249, 246, 255),
            fps: 30,
            wobble_fps: None,
            script: Group::new(),
            cursor: 0.0,
            played_to: -1.0,
            introduced: HashSet::new(),
        }
    }

    pub fn fps(mut self, fps: impl Num) -> Self {
        self.fps = (fps.f32().round() as u32).max(1);
        self
    }

    /// Page colour: a name, hex string or `(r, g, b)`.
    pub fn background(mut self, color: impl IntoColor) -> Self {
        self.background = color.into_color();
        self
    }

    /// Screen position (pixels) of the world origin `(0, 0, 0)`.
    pub fn origin(mut self, x: impl Num, y: impl Num) -> Self {
        let camera = self.scene.camera_mut();
        camera.origin_x = x.f32();
        camera.origin_y = y.f32();
        self
    }

    /// Zoom the camera: `zoom(1.5)` makes everything 50% bigger on screen around the origin.
    /// Line widths and text sizes are not affected. Set `origin` first if you move it.
    pub fn zoom(mut self, factor: impl Num) -> Self {
        self.scene.camera_mut().scale = factor.f32().max(0.01);
        self
    }

    /// The floor point `(x, 0, z)` that appears at screen pixel `(x, y)`, measured from the
    /// top-left of the canvas. Use it to place text exactly:
    /// `text("Title").at(s.pixel(640, 80))`. Call it after `origin` and `zoom`.
    pub fn pixel(&self, x: impl Num, y: impl Num) -> Vec3 {
        let (wx, wy, wz) = self.scene.camera().unproject_floor(x.f32(), y.f32());
        Vec3::new(wx, wy, wz)
    }

    /// Change how hand-drawn everything looks.
    pub fn sketch(mut self, sketch: Sketch) -> Self {
        self.scene.set_options(sketch.options());
        self
    }

    /// Redraw the rough strokes `fps` times a second for a "boiling line" look.
    pub fn wobble(mut self, fps: impl Num) -> Self {
        self.wobble_fps = Some(fps.f32());
        self
    }

    /// Add an object to the scene. It is visible from frame 0 unless an animation such as
    /// `create` or `fade_in` brings it in later.
    ///
    /// A `title(..)` or `label(..)` added without a position goes to its default spot: the top
    /// centre of the canvas, or just under it. Any `.at(..)` / `.base_at(..)` overrides that.
    pub fn add(&mut self, object: impl Into<Object>) -> Id {
        let mut object = object.into();
        if let Some(home) = object.home.take() {
            let (x, y) = self.home_pixel(home);
            object = object.base_at(self.pixel(x, y));
        }
        self.scene.add(object)
    }

    /// The pixel where a title or caption lives on this canvas.
    pub(crate) fn home_pixel(&self, home: Home) -> (f32, f32) {
        let (w, h) = (self.width as f32, self.height as f32);
        match home {
            Home::Title => (w / 2.0, h * 0.097),
            Home::Caption => (w / 2.0, h * 0.178),
        }
    }

    /// Append an animation after everything scripted so far.
    pub fn play(&mut self, anim: impl Animation + 'static) -> &mut Self {
        let mut intro = Vec::new();
        anim.introduces(&mut intro);
        for id in intro {
            // Only the first animation to bring an object in hides it from the start.
            if self.introduced.insert(id) {
                self.hide(id);
            }
        }
        let mut consumed = Vec::new();
        anim.consumes(&mut consumed);
        for id in consumed {
            self.hide(id);
        }

        let d = anim.duration();
        self.script.push_at(self.cursor, anim);
        self.cursor += d;
        self
    }

    fn hide(&mut self, id: Id) {
        if let Some(o) = self.scene.get_mut(id) {
            o.visible = false;
        }
    }

    /// Hold for `seconds`.
    pub fn wait(&mut self, seconds: impl Num) -> &mut Self {
        self.cursor += seconds.f32().max(0.0);
        self
    }

    /// Total length of the script in seconds.
    pub fn duration(&self) -> f32 {
        self.cursor.max(self.script.duration())
    }

    /// Number of frames needed to cover the whole script at `fps` (including the final state).
    pub fn frame_count(&self, fps: u32) -> u32 {
        (self.duration() * fps as f32).ceil() as u32 + 1
    }

    /// Advance the scene to `time` (seconds). Playback is forward-only: times must not
    /// decrease between calls.
    pub fn seek(&mut self, time: f32) {
        assert!(
            time >= self.played_to,
            "Stage::seek only moves forward ({time} < {})",
            self.played_to
        );
        self.script
            .update(&mut self.scene, time.min(self.script.duration()));
        self.played_to = time;
    }

    /// Seek to `time` and render that frame.
    pub fn render_frame(&mut self, time: f32) -> Pixmap {
        self.seek(time);
        let mut pixmap = Pixmap::new(self.width, self.height).expect("non-zero canvas size");
        pixmap.fill(self.background);
        // Occluding patches must match the page, whatever colour it is.
        if self.scene.paper.is_some() {
            self.scene.paper = Some(self.background);
        }
        let step = self.wobble_fps.map_or(0, |f| (time * f).floor() as u64);
        self.scene.render_wobble(&mut pixmap.as_mut(), step);
        pixmap
    }

    /// Render to `path`, choosing the output from the extension:
    ///
    /// - `.mp4`, `.webm`, `.gif`: the whole animation as a video (needs `ffmpeg`).
    /// - `.png`: just the final frame.
    ///
    /// Playback only moves forward, so render the video first and the `.png` after; the
    /// other way round fails because the stage has already reached the end.
    pub fn render(&mut self, path: impl AsRef<Path>) -> Result {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if ext == "png" {
            return self.still(path);
        }
        if VideoFormat::from_path(path).is_none() {
            return Err(Error::Invalid(format!(
                "can't render to {path:?}: use .mp4, .webm, .gif (video) or .png (last frame)"
            )));
        }
        if self.played_to > 0.0 {
            return Err(Error::Invalid(
                "this Stage has already played, so it can't render a video again: a Stage plays \
                 once. Build a fresh Stage for each video, and render the .png last."
                    .to_string(),
            ));
        }
        let opts = VideoOptions::fps(self.fps);
        self.render_video(path, &opts)?;
        Ok(())
    }

    /// Save the final frame as a PNG.
    pub fn still(&mut self, path: impl AsRef<Path>) -> Result {
        let end = self.duration();
        if end < self.played_to {
            return Err(Error::Invalid(
                "the stage already played past the end".to_string(),
            ));
        }
        if let Some(parent) = path.as_ref().parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        self.render_frame(end)
            .save_png(path.as_ref())
            .map_err(std::io::Error::other)?;
        Ok(())
    }

    /// Render the whole script to a video file piped through `ffmpeg`. The format comes from
    /// the extension: `.mp4`, `.webm` or `.gif`. Returns the number of frames encoded.
    ///
    /// Needs `ffmpeg` on `PATH` (or set [`VideoOptions::ffmpeg`]).
    pub fn render_video(
        &mut self,
        path: impl AsRef<Path>,
        opts: &VideoOptions,
    ) -> std::io::Result<u32> {
        let mut enc = VideoEncoder::start(path.as_ref(), self.width, self.height, opts)?;
        let n = self.frame_count(opts.fps);
        for i in 0..n {
            let pixmap = self.render_frame(i as f32 / opts.fps as f32);
            // The background is opaque, so premultiplied and straight RGBA are identical.
            enc.write_frame(pixmap.data())?;
        }
        enc.finish()?;
        Ok(n)
    }

    /// Render every frame as `frame_00000.png`, `frame_00001.png`, ... in `dir` (created if
    /// needed). Returns the number of frames written.
    pub fn render_png_sequence(&mut self, dir: impl AsRef<Path>, fps: u32) -> std::io::Result<u32> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let n = self.frame_count(fps);
        for i in 0..n {
            let pixmap = self.render_frame(i as f32 / fps as f32);
            pixmap
                .save_png(dir.join(format!("frame_{i:05}.png")))
                .map_err(std::io::Error::other)?;
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::easing::Easing;
    use crate::animation::*;
    use crate::api::deg;
    use crate::api::shapes::*;
    use crate::primitives::{Box3D, Geometry};
    use crate::render::camera::IsoCamera;
    use crate::world::style::rgb;
    use roughr::core::Options;

    fn timeline() -> Stage {
        Stage::with(
            Scene::new(IsoCamera::new(100.0, 100.0), Options::default()),
            200,
            200,
        )
    }

    fn pos(tl: &Stage, id: Id) -> f32 {
        tl.scene.get(id).unwrap().transform.position.x
    }

    #[test]
    fn play_and_wait_advance_the_clock() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(move_to(a, [10., 0., 0.]).duration(2.0)).wait(1.0);
        tl.play(move_to(a, [0., 0., 0.]).duration(0.5));
        assert!((tl.duration() - 3.5).abs() < 1e-6);
    }

    #[test]
    fn move_tween_interpolates_and_lands_exactly() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(move_to(a, [10., 0., 0.]).duration(2.0).ease(Easing::Linear));
        tl.seek(1.0);
        assert!((pos(&tl, a) - 5.0).abs() < 1e-5);
        tl.seek(2.0);
        assert_eq!(pos(&tl, a), 10.0);
        tl.seek(99.0);
        assert_eq!(pos(&tl, a), 10.0);
    }

    #[test]
    fn chained_relative_moves_accumulate() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(move_by(a, [10., 0., 0.]).duration(1.0));
        tl.play(move_by(a, [10., 0., 0.]).duration(1.0));
        tl.seek(0.5);
        tl.seek(2.0);
        assert_eq!(pos(&tl, a), 20.0);
    }

    #[test]
    fn jumping_past_several_animations_still_runs_them_in_order() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(move_by(a, [10., 0., 0.]).duration(1.0));
        tl.play(move_by(a, [5., 0., 0.]).duration(1.0));
        tl.seek(10.0);
        assert_eq!(pos(&tl, a), 15.0);
    }

    #[test]
    fn fade_in_reveals_a_hidden_object() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]).hidden());
        tl.wait(1.0);
        tl.play(fade_in(a).duration(1.0).ease(Easing::Linear));
        tl.seek(0.5);
        assert!(!tl.scene.get(a).unwrap().visible);
        tl.seek(1.5);
        let o = tl.scene.get(a).unwrap();
        assert!(o.visible);
        assert!((o.style.opacity - 0.5).abs() < 1e-5);
        tl.seek(2.0);
        assert_eq!(tl.scene.get(a).unwrap().style.opacity, 1.0);
    }

    #[test]
    fn create_reveals_from_zero_and_shows_the_object() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]).hidden());
        tl.wait(1.0);
        tl.play(create(a).duration(1.0).ease(Easing::Linear));
        tl.seek(0.5);
        assert!(!tl.scene.get(a).unwrap().visible);
        tl.seek(1.5);
        let o = tl.scene.get(a).unwrap();
        assert!(o.visible);
        assert!((o.style.reveal - 0.5).abs() < 1e-5);
        tl.seek(2.0);
        assert_eq!(tl.scene.get(a).unwrap().style.reveal, 1.0);
    }

    #[test]
    fn uncreate_goes_back_to_zero() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(uncreate(a).duration(1.0));
        tl.seek(1.0);
        assert_eq!(tl.scene.get(a).unwrap().style.reveal, 0.0);
    }

    fn inked(px: &Pixmap) -> usize {
        px.pixels()
            .iter()
            .filter(|p| p.red() != 250 || p.green() != 249 || p.blue() != 246)
            .count()
    }

    #[test]
    fn create_draws_progressively_more_ink() {
        let mut tl = timeline();
        let a = tl.add(
            Box3D::new(40., 40., 40.)
                .at([0., 20., 0.])
                .fill(rgb(200, 80, 80))
                .hidden(),
        );
        tl.play(create(a).duration(1.0).ease(Easing::Linear));
        let counts: Vec<usize> = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
            .iter()
            .map(|&t| inked(&tl.render_frame(t)))
            .collect();
        assert_eq!(counts[0], 0, "nothing drawn at t=0: {counts:?}");
        assert!(counts.windows(2).all(|w| w[0] < w[1]), "ink should grow: {counts:?}");
    }

    #[test]
    fn finished_create_matches_a_plain_render() {
        let draw = |animated: bool| {
            let mut tl = timeline();
            let obj = Box3D::new(40., 40., 40.).at([0., 20., 0.]).fill(rgb(200, 80, 80));
            if animated {
                let a = tl.add(obj.hidden());
                tl.play(create(a).duration(0.5));
            } else {
                tl.add(obj);
                tl.wait(0.5);
            }
            tl.render_frame(0.5).data().to_vec()
        };
        assert_eq!(draw(true), draw(false));
    }

    fn region_of(tl: &Stage, id: Id) -> crate::planar::Region {
        tl.scene.get(id).unwrap().geometry.region().unwrap().clone()
    }

    #[test]
    fn morph_runs_from_source_to_exact_target() {
        use crate::planar::Region;
        use crate::primitives::Shape2D;
        let (src, dst) = (Region::circle(10.), Region::rect(30., 30.));
        let mut tl = timeline();
        let a = tl.add(Shape2D::new(src.clone()).at([5., 0., 0.]).fill(rgb(1, 2, 3)));
        tl.play(morph(a, dst.clone()).duration(1.0).ease(Easing::Linear));
        tl.seek(0.0);
        assert_eq!(region_of(&tl, a), src);
        tl.seek(0.5);
        let mid = region_of(&tl, a).area();
        assert!(mid > src.area() && mid < dst.area(), "{mid}");
        tl.seek(1.0);
        assert_eq!(region_of(&tl, a), dst);
        // Only the shape changed.
        let o = tl.scene.get(a).unwrap();
        assert_eq!(o.transform.position.x, 5.0);
        assert_eq!(o.style.fill, Some(rgb(1, 2, 3)));
    }

    #[test]
    fn chained_morphs_start_from_where_the_last_ended() {
        use crate::planar::Region;
        use crate::primitives::Shape2D;
        let mut tl = timeline();
        let a = tl.add(Shape2D::circle(10.).at([0., 0., 0.]));
        let (sq, star) = (Region::rect(20., 20.), Region::star(5, 20., 8.));
        tl.play(seq([
            morph(a, sq.clone()).duration(1.0),
            morph(a, star.clone()).duration(1.0),
        ]));
        tl.seek(1.0);
        assert_eq!(region_of(&tl, a), sq);
        tl.seek(2.0);
        assert_eq!(region_of(&tl, a), star);
    }

    #[test]
    fn transform_into_takes_on_the_targets_shape_and_look() {
        use crate::planar::Region;
        use crate::primitives::Shape2D;
        let mut tl = timeline();
        let a = tl.add(Shape2D::circle(10.).at([0., 0., 0.]).fill(rgb(255, 0, 0)));
        let target = Region::star(5, 20., 8.);
        let b = tl.add(
            Shape2D::new(target.clone())
                .at([40., 0., 10.])
                .fill(rgb(0, 0, 255))
                .opacity(0.5)
                .hidden(),
        );
        tl.play(transform_into(a, b).duration(1.0).ease(Easing::Linear));
        tl.seek(1.0);
        assert_eq!(region_of(&tl, a), target);
        let o = tl.scene.get(a).unwrap();
        assert_eq!((o.transform.position.x, o.transform.position.z), (40.0, 10.0));
        assert_eq!(o.style.fill, Some(rgb(0, 0, 255)));
        assert_eq!(o.style.opacity, 0.5);
        // The target is left untouched.
        let t = tl.scene.get(b).unwrap();
        assert!(!t.visible);
        assert_eq!(t.transform.position.x, 40.0);
    }

    #[test]
    fn prisms_morph_too() {
        use crate::planar::Region;
        use crate::primitives::Prism;
        let mut tl = timeline();
        let a = tl.add(Prism::new(Region::circle(10.), 10.).at([0., 0., 0.]));
        let dst = Region::rect(30., 30.);
        tl.play(morph(a, dst.clone()).duration(1.0));
        tl.seek(1.0);
        assert_eq!(region_of(&tl, a), dst);
    }

    #[test]
    #[should_panic(expected = "no region")]
    fn morphing_a_box_is_a_clear_error() {
        use crate::planar::Region;
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(morph(a, Region::circle(5.)));
        tl.seek(0.0);
    }

    #[test]
    fn a_mid_morph_frame_differs_from_both_ends() {
        use crate::planar::Region;
        use crate::primitives::Shape2D;
        let mut tl = timeline();
        let a = tl.add(
            Shape2D::circle(30.).at([0., 0., 0.]).fill(rgb(200, 80, 80)),
        );
        tl.play(morph(a, Region::star(5, 45., 15.)).duration(1.0).ease(Easing::Linear));
        let (f0, f1, f2) = (
            tl.render_frame(0.0).data().to_vec(),
            tl.render_frame(0.5).data().to_vec(),
            tl.render_frame(1.0).data().to_vec(),
        );
        assert_ne!(f0, f1);
        assert_ne!(f1, f2);
        assert_ne!(f0, f2);
    }

    // ---- high-level API -------------------------------------------------------------

    #[test]
    fn new_has_sensible_defaults_and_builders_chain() {
        let s = Stage::new(900, 600);
        assert_eq!((s.width, s.height, s.fps), (900, 600, 30));
        assert_eq!((s.scene.camera().origin_x, s.scene.camera().origin_y), (450.0, 360.0));
        let s = Stage::new(100, 100)
            .fps(60)
            .background("#000")
            .origin(10, 20)
            .wobble(8)
            .sketch(Sketch::clean());
        assert_eq!(s.fps, 60);
        assert_eq!(s.background, Color::from_rgba8(0, 0, 0, 255));
        assert_eq!((s.scene.camera().origin_x, s.scene.camera().origin_y), (10.0, 20.0));
        assert_eq!(s.wobble_fps, Some(8.0));
    }

    #[test]
    fn zoom_and_pixel_place_things_on_screen() {
        let s = Stage::new(1280, 720).zoom(1.6).origin(640, 432);
        assert_eq!(s.scene.camera().scale, 1.6);
        // `pixel` is the inverse of the camera's projection on the floor.
        let w = s.pixel(200, 100);
        let p = s.scene.camera().project(w.x, w.y, w.z);
        assert!((p.x - 200.0).abs() < 1e-3 && (p.y - 100.0).abs() < 1e-3, "{p:?}");
        // Zoom changes how big things look, not their world size.
        let mut z = Stage::new(400, 400).zoom(2);
        let c = z.add(cube(10).base_at((0, 0, 0)));
        let b = z.scene.get(c).unwrap().bounds();
        assert_eq!(b.max.y - b.min.y, 10.0);
    }

    #[test]
    fn create_fade_in_and_write_hide_their_target_until_they_start() {
        for make in [create as fn(Id) -> Anim, fade_in, write] {
            let mut s = Stage::new(100, 100);
            let a = s.add(cube(10)); // added visible, no .hidden()
            s.wait(1);
            s.play(make(a).duration(1));
            assert!(!s.scene.get(a).unwrap().visible, "hidden from frame 0");
            s.seek(0.5);
            assert!(!s.scene.get(a).unwrap().visible, "still waiting");
            s.seek(1.5);
            assert!(s.scene.get(a).unwrap().visible, "visible once it starts");
        }
    }

    #[test]
    fn only_the_first_introduction_hides_the_object() {
        let mut s = Stage::new(100, 100);
        let a = s.add(cube(10));
        s.play(create(a));
        s.play(uncreate(a));
        s.play(create(a)); // second introduction must not re-hide at frame 0
        s.seek(0.0);
        // Hidden only because the first create hasn't started yet when frame 0 is shown...
        assert!(s.scene.get(a).unwrap().visible, "the first create ran at t=0");
    }

    #[test]
    fn introductions_inside_groups_are_found() {
        let mut s = Stage::new(100, 100);
        let (a, b) = (s.add(cube(10)), s.add(cube(10)));
        s.wait(1);
        s.play(all([create(a), fade_in(b).delay(0.5)]));
        assert!(!s.scene.get(a).unwrap().visible && !s.scene.get(b).unwrap().visible);
    }

    #[test]
    fn transform_into_hides_its_target_and_morph_accepts_shapes() {
        let mut s = Stage::new(100, 100);
        let a = s.add(circle(10));
        let ghost = s.add(star(5, 20, 8).at((30, 0, 0))); // no .hidden()
        s.play(transform_into(a, ghost).duration(1));
        assert!(!s.scene.get(ghost).unwrap().visible);
        s.seek(1.0);
        assert_eq!(s.scene.get(a).unwrap().transform.position.x, 30.0);

        let mut s = Stage::new(100, 100);
        let a = s.add(circle(10));
        s.play(morph(a, rect(30, 30)).ease(Easing::Linear));
        s.seek(1.0);
        let area = s.scene.get(a).unwrap().geometry.region().unwrap().area();
        assert!((area - 900.0).abs() < 1.0, "{area}");
    }

    #[test]
    fn anim_methods_compose_without_boxing() {
        let mut s = Stage::new(100, 100);
        let a = s.add(cube(10));
        s.play(move_to(a, (10, 0, 0)).duration(2).ease(Easing::Linear));
        assert_eq!(s.duration(), 2.0);
        s.play(move_by(a, (10, 0, 0)).duration(1).delay(1)); // 1 s of motion after a 1 s wait
        assert_eq!(s.duration(), 4.0);
        s.seek(1.0);
        assert!((s.scene.get(a).unwrap().transform.position.x - 5.0).abs() < 1e-4);
        s.seek(4.0);
        assert_eq!(s.scene.get(a).unwrap().transform.position.x, 20.0);
    }

    #[test]
    fn duration_after_delay_rescales_the_whole_thing() {
        // `duration` always means the total length, so put `.delay()` last.
        let mut s = Stage::new(100, 100);
        let a = s.add(cube(10));
        s.play(move_by(a, (10, 0, 0)).delay(1).duration(1));
        assert_eq!(s.duration(), 1.0);
    }

    #[test]
    fn integer_arguments_work_everywhere() {
        let mut s = Stage::new(100, 100);
        let a = s.add(sphere(10).at((0, 0, 0)).opacity(1));
        s.play(opacity_to(a, 0).duration(1));
        s.play(scale_to(a, (2, 2, 2)));
        s.play(rotate_by(a, (0, deg(90), 0)));
        s.wait(1);
        s.seek(s.duration());
        assert_eq!(s.scene.get(a).unwrap().transform.scale.x, 2.0);
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rough-anim-stage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn render_picks_the_output_from_the_extension() {
        let mut s = Stage::new(120, 80).fps(10);
        let c = s.add(cube(20).base_at((0, 0, 0)).fill("purple"));
        s.play(create(c).duration(0.3));

        let bad = s.render(temp("x.txt")).unwrap_err();
        assert!(matches!(bad, Error::Invalid(m) if m.contains(".mp4")));

        // Video first, then the final frame, as documented.
        let ffmpeg = std::process::Command::new("ffmpeg").arg("-version").output().is_ok();
        if ffmpeg {
            let mp4 = temp("out.mp4");
            s.render(&mp4).unwrap();
            assert!(std::fs::metadata(&mp4).unwrap().len() > 0);
        }
        let png = temp("last.PNG"); // extension match is case-insensitive
        s.render(&png).unwrap();
        let img = Pixmap::load_png(&png).unwrap();
        assert_eq!((img.width(), img.height()), (120, 80));
    }

    #[test]
    fn rendering_a_video_after_the_stage_already_played_is_an_error_not_a_panic() {
        let mut s = Stage::new(80, 60).fps(10);
        let c = s.add(cube(10));
        s.play(create(c).duration(0.3));
        s.render(temp("first.png")).unwrap(); // reaches the end
        let err = s.render(temp("second.mp4")).unwrap_err().to_string();
        assert!(err.contains("fresh Stage"), "{err}");
    }

    #[test]
    fn a_zero_length_stage_can_still_render_a_video_after_a_png() {
        // Nothing to play, so the png didn't advance the clock past 0.
        let mut s = Stage::new(40, 40);
        s.add(cube(5));
        s.render(temp("static.png")).unwrap();
        if std::process::Command::new("ffmpeg").arg("-version").output().is_ok() {
            s.render(temp("static.mp4")).unwrap();
        }
    }

    #[test]
    fn canvas_size_and_fps_accept_floats_and_ints() {
        let s = Stage::new(900.0, 600).fps(29.97);
        assert_eq!((s.width, s.height, s.fps), (900, 600, 30));
        let s = Stage::new(0, -5); // nonsense is clamped, never a zero-size canvas
        assert_eq!((s.width, s.height), (1, 1));
    }

    #[test]
    fn a_png_alone_is_the_finished_scene() {
        let mut s = Stage::new(160, 120);
        let c = s.add(circle(30).fill("rose"));
        s.play(create(c).duration(1));
        let png = temp("end.png");
        s.still(&png).unwrap();
        let img = Pixmap::load_png(&png).unwrap();
        let bg = s.background.to_color_u8();
        assert!(
            img.pixels().iter().any(|p| (p.red(), p.green(), p.blue()) != (bg.red(), bg.green(), bg.blue())),
            "the finished shape should be drawn"
        );
    }

    #[test]
    fn fade_out_goes_to_zero() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(fade_out(a).duration(1.0));
        tl.seek(1.0);
        assert_eq!(tl.scene.get(a).unwrap().style.opacity, 0.0);
    }

    #[test]
    fn parallel_lasts_as_long_as_longest_and_runs_together() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        let b = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(all([
            move_to(a, [10., 0., 0.])
                .duration(1.0)
                .ease(Easing::Linear)
                ,
            move_to(b, [10., 0., 0.])
                .duration(3.0)
                .ease(Easing::Linear)
                ,
        ]));
        assert_eq!(tl.duration(), 3.0);
        tl.seek(1.0);
        assert_eq!(pos(&tl, a), 10.0);
        assert!((pos(&tl, b) - 10.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn sequence_runs_back_to_back() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(seq([
            move_to(a, [10., 0., 0.])
                .duration(1.0)
                .ease(Easing::Linear)
                ,
            wait(1.0),
            move_to(a, [30., 0., 0.])
                .duration(1.0)
                .ease(Easing::Linear)
                ,
        ]));
        assert_eq!(tl.duration(), 3.0);
        tl.seek(1.5);
        assert_eq!(pos(&tl, a), 10.0);
        tl.seek(2.5);
        assert!((pos(&tl, a) - 20.0).abs() < 1e-5);
    }

    #[test]
    fn lagged_start_staggers() {
        let mut tl = timeline();
        let ids: Vec<_> = (0..3)
            .map(|_| tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.])))
            .collect();
        let anims = ids.iter().map(|&i| {
            move_to(i, [10., 0., 0.])
                .duration(1.0)
                .ease(Easing::Linear)
                
        });
        tl.play(lag(0.5, anims));
        assert_eq!(tl.duration(), 2.0);
        tl.seek(0.5);
        assert!((pos(&tl, ids[0]) - 5.0).abs() < 1e-5);
        assert_eq!(pos(&tl, ids[1]), 0.0);
    }

    #[test]
    fn fill_to_from_no_fill_starts_transparent_then_reaches_target() {
        let mut tl = timeline();
        let a = tl.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        tl.play(fill_to(a, rgb(255, 0, 0)).duration(1.0));
        tl.seek(1.0);
        assert_eq!(tl.scene.get(a).unwrap().style.fill, Some(rgb(255, 0, 0)));
    }

    #[test]
    #[should_panic(expected = "only moves forward")]
    fn seeking_backwards_panics() {
        let mut tl = timeline();
        tl.wait(2.0);
        tl.seek(1.0);
        tl.seek(0.5);
    }

    #[test]
    fn frame_count_includes_final_frame() {
        let mut tl = timeline();
        tl.wait(1.0);
        assert_eq!(tl.frame_count(30), 31);
    }

    fn have_ffmpeg() -> bool {
        std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .is_ok()
    }

    #[test]
    fn exports_mp4_and_gif() {
        if !have_ffmpeg() {
            eprintln!("skipping: ffmpeg not installed");
            return;
        }
        let dir = std::env::temp_dir().join(format!("rough-anim-test-{}", std::process::id()));
        for name in ["out/a.mp4", "out/a.gif", "out/a.webm"] {
            let mut tl = timeline();
            let a = tl.add(
                Box3D::new(20., 20., 20.)
                    .at([0., 10., 0.])
                    .fill(rgb(200, 80, 80)),
            );
            tl.play(move_by(a, [30., 0., 0.]).duration(0.5));
            let path = dir.join(name);
            let n = tl.render_video(&path, &VideoOptions::fps(10)).unwrap();
            assert_eq!(n, 6);
            assert!(std::fs::metadata(&path).unwrap().len() > 0, "{name}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn odd_dimensions_still_encode_to_mp4() {
        if !have_ffmpeg() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("rough-anim-odd-{}", std::process::id()));
        let mut tl = Stage::with(
            Scene::new(IsoCamera::new(50.0, 50.0), Options::default()),
            101,
            75,
        );
        tl.wait(0.2);
        tl.render_video(dir.join("odd.mp4"), &VideoOptions::fps(10))
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn renders_background_and_object() {
        let mut tl = timeline();
        tl.add(
            Box3D::new(40., 40., 40.)
                .at([0., 20., 0.])
                .fill(rgb(200, 80, 80)),
        );
        tl.wait(0.1);
        let px = tl.render_frame(0.0);
        let corner = px.pixel(0, 0).unwrap();
        assert_eq!(
            (corner.red(), corner.green(), corner.blue()),
            (250, 249, 246)
        );
        assert!(
            px.pixels()
                .iter()
                .any(|p| p.red() != 250 || p.green() != 249)
        );
    }
}
