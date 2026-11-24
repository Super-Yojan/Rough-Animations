//! Placing things by screen pixel, and labelling other things: the layout helpers that every
//! script otherwise writes for itself.

use crate::api::shapes::{label, title};
use crate::api::{Num, Pos, Stage};
use crate::world::arena::Id;
use crate::world::object::{Home, Object};

/// Which side of an object a [`Stage::tag`] goes on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Above,
    Below,
    Left,
    Right,
}

/// Space between a tagged object and its tag, in pixels.
const TAG_GAP: f32 = 14.0;
/// A heading's title only gets the lively hand-drawn wobble at or above this size (in pixels); it
/// is hard to read when small.
const LIVELY_TITLE_MIN: f32 = 48.0;
/// Text size of a tag that is given as a plain string.
const TAG_SIZE: u32 = 28;

/// Something that can be a tag: a string (a small default label) or any text [`Object`] you
/// styled yourself, e.g. `label("hub").color("rose")`.
pub trait IntoTag {
    fn into_tag(self) -> Object;
}

impl IntoTag for &str {
    fn into_tag(self) -> Object {
        label(self).font_size(TAG_SIZE)
    }
}

impl IntoTag for String {
    fn into_tag(self) -> Object {
        self.as_str().into_tag()
    }
}

impl IntoTag for Object {
    fn into_tag(self) -> Object {
        self
    }
}

impl Stage {
    /// Where a world point lands on screen, in pixels from the top-left. The inverse of
    /// [`pixel`](Self::pixel) for points on the floor.
    pub fn screen(&self, point: impl Pos) -> (f32, f32) {
        let p = point.pos();
        let q = self.scene.camera().project(p.x, p.y, p.z);
        (q.x, q.y)
    }

    /// Add `object` standing on screen pixel `(x, y)`: its bottom centre sits there, which for
    /// text and flat shapes means centred on the pixel. The way to lay a scene out in pixels:
    /// `s.put(title("Hello"), 640, 80)`.
    pub fn put(&mut self, object: impl Into<Object>, x: impl Num, y: impl Num) -> Id {
        let at = self.pixel(x, y);
        self.add(object.into().base_at(at))
    }

    /// Add a label next to an existing object, on the given side, spaced to fit the text:
    /// `s.tag(tower, "compute", Side::Above)`. Pass a styled `label(..)` instead of a string to
    /// choose its look. It is placed where the object is *now*, so tag after moving things.
    pub fn tag(&mut self, target: Id, content: impl IntoTag, side: Side) -> Id {
        let bounds = self
            .scene
            .get(target)
            .expect("tag(): that object is not in the scene")
            .bounds();
        // The screen rectangle the object covers.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for c in bounds.corners() {
            let (x, y) = self.screen(c);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let tag = content.into_tag();
        let (w, h) = tag.dimensions().unwrap_or((0.0, 0.0));
        let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let (x, y) = match side {
            Side::Above => (cx, y0 - TAG_GAP - h / 2.0),
            Side::Below => (cx, y1 + TAG_GAP + h / 2.0),
            Side::Left => (x0 - TAG_GAP - w / 2.0, cy),
            Side::Right => (x1 + TAG_GAP + w / 2.0, cy),
        };
        self.put(tag, x, y)
    }

    /// A section header and a one-line caption centred near the top of the canvas, sized to
    /// it (the header gets the lively wobble only on canvases big enough for it): `s.heading("Boolean operations", "combine any flat shapes")`. Returns the ids, header
    /// first, ready for `write_all` and `clear`. Pass `""` as the caption for a header alone
    /// (then there is just one id).
    pub fn heading(&mut self, header: &str, caption: &str) -> Vec<Id> {
        let h = self.height as f32;
        let (title_at, caption_at) = (self.home_pixel(Home::Title), self.home_pixel(Home::Caption));
        let size = h / 12.0;
        let mut head = title(header).font_size(size);
        if size < LIVELY_TITLE_MIN {
            head = head.steady(); // too small to wobble legibly
        }
        let mut ids = vec![self.put(head, title_at.0, title_at.1)];
        if !caption.is_empty() {
            ids.push(self.put(label(caption).font_size(h * 0.044), caption_at.0, caption_at.1));
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::IntoColor;
    use crate::api::shapes::*;

    fn stage() -> Stage {
        Stage::new(1280, 720)
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.5
    }

    #[test]
    fn screen_and_pixel_are_inverses_on_the_floor() {
        let s = stage().zoom(1.4);
        let w = s.pixel(300, 500);
        let (x, y) = s.screen(w);
        assert!(near(x, 300.0) && near(y, 500.0), "({x}, {y})");
        // Height moves a point up the screen.
        let (_, y_up) = s.screen((w.x, 100.0, w.z));
        assert!(y_up < y - 100.0);
    }

    #[test]
    fn put_centres_text_and_stands_solids_on_the_pixel() {
        let mut s = stage();
        let t = s.put(text("hi"), 200, 100);
        let (x, y) = s.screen(s.scene.get(t).unwrap().transform.position);
        assert!(near(x, 200.0) && near(y, 100.0));

        let c = s.put(cube(50).fill("mint"), 640, 500);
        let o = s.scene.get(c).unwrap();
        assert!((o.bounds().min.y).abs() < 1e-3, "the cube stands on the floor");
        let (x, y) = s.screen(o.transform.position);
        assert!(near(x, 640.0), "centred horizontally on the pixel: {x}");
        // Its bottom centre, not its middle, is at the pixel.
        let (_, base_y) = s.screen((o.transform.position.x, 0.0, o.transform.position.z));
        assert!(near(base_y, 500.0), "base at {base_y}, middle at {y}");
        // Accepts floats too.
        s.put(circle(10), 12.5, 99.5);
    }

    /// The screen rectangle covered by an object's bounds.
    fn rect(s: &Stage, id: Id) -> (f32, f32, f32, f32) {
        let b = s.scene.get(id).unwrap().bounds();
        let pts: Vec<_> = b.corners().iter().map(|&c| s.screen(c)).collect();
        (
            pts.iter().map(|p| p.0).fold(f32::MAX, f32::min),
            pts.iter().map(|p| p.1).fold(f32::MAX, f32::min),
            pts.iter().map(|p| p.0).fold(f32::MIN, f32::max),
            pts.iter().map(|p| p.1).fold(f32::MIN, f32::max),
        )
    }

    #[test]
    fn tags_sit_on_the_requested_side_without_touching() {
        let mut s = stage();
        let b = s.put(cuboid(80, 60, 80).fill("amber"), 640, 400);
        let (x0, y0, x1, y1) = rect(&s, b);
        let centre = |s: &Stage, id: Id| {
            let o = s.scene.get(id).unwrap();
            let (cx, cy) = s.screen(o.transform.position);
            let (w, h) = o.dimensions().unwrap();
            (cx, cy, w, h)
        };

        let above = s.tag(b, "top", Side::Above);
        let (cx, cy, _, h) = centre(&s, above);
        assert!(near(cx, (x0 + x1) / 2.0), "centred over the object");
        assert!(cy + h / 2.0 < y0, "entirely above: text bottom {} vs top {y0}", cy + h / 2.0);

        let below = s.tag(b, "bottom", Side::Below);
        let (_, cy, _, h) = centre(&s, below);
        assert!(cy - h / 2.0 > y1);

        let left = s.tag(b, "left", Side::Left);
        let (cx, _, w, _) = centre(&s, left);
        assert!(cx + w / 2.0 < x0, "text right edge {} vs object left {x0}", cx + w / 2.0);

        let right = s.tag(b, "a longer label", Side::Right);
        let (cx, _, w, _) = centre(&s, right);
        assert!(cx - w / 2.0 > x1, "text left edge {} vs object right {x1}", cx - w / 2.0);
        // The gap doesn't grow with a longer label: it's measured to the text's own edge.
        assert!(near(cx - w / 2.0 - x1, TAG_GAP));
    }

    #[test]
    fn a_styled_label_can_be_a_tag() {
        let mut s = stage();
        let b = s.put(cube(40), 640, 400);
        let t = s.tag(b, label("big").font_size(60).color("rose"), Side::Above);
        let o = s.scene.get(t).unwrap();
        assert_eq!(o.style.fill, Some("rose".into_color()));
        assert!(o.dimensions().unwrap().1 > 30.0);
    }

    #[test]
    #[should_panic(expected = "not in the scene")]
    fn tagging_a_missing_object_is_a_clear_error() {
        let mut s = stage();
        let b = s.add(cube(10));
        s.scene.remove(b);
        s.tag(b, "x", Side::Above);
    }

    #[test]
    fn heading_is_a_title_over_a_steady_caption_sized_to_the_canvas() {
        for (w, h) in [(1280, 720), (640, 360), (1920, 1080)] {
            let mut s = Stage::new(w, h);
            let ids = s.heading("Hello", "world");
            assert_eq!(ids.len(), 2);
            let (head, cap) = (s.scene.get(ids[0]).unwrap(), s.scene.get(ids[1]).unwrap());
            // The title wobbles only when it is big enough to stay readable; the caption never does.
            let lively = h as f32 / 12.0 >= LIVELY_TITLE_MIN;
            assert_eq!(head.geometry.wobbles(), lively, "{w}x{h}");
            assert!(!cap.geometry.wobbles());
            // Centred, near the top, caption below the title.
            let (hx, hy) = s.screen(head.transform.position);
            let (cx, cy) = s.screen(cap.transform.position);
            assert!(near(hx, w as f32 / 2.0) && near(cx, w as f32 / 2.0));
            assert!(hy < h as f32 * 0.15 && cy > hy && cy < h as f32 * 0.25, "{hy} {cy}");
            // The text scales with the canvas height (60 and 32 px at 720p).
            let ratio = head.dimensions().unwrap().1 / (h as f32 / 720.0);
            assert!(ratio > 30.0 && ratio < 60.0, "{ratio}");
        }
    }

    #[test]
    fn a_title_and_label_added_without_a_position_go_to_their_default_spots() {
        for (w, h) in [(1280, 720), (800, 600)] {
            let mut s = Stage::new(w, h);
            let (t, c) = (s.add(title("Hello")), s.add(label("caption")));
            let at = |s: &Stage, id: Id| s.screen(s.scene.get(id).unwrap().transform.position);
            let ((tx, ty), (cx, cy)) = (at(&s, t), at(&s, c));
            assert!(near(tx, w as f32 / 2.0) && near(cx, w as f32 / 2.0), "centred");
            assert!(near(ty, h as f32 * 0.097) && near(cy, h as f32 * 0.178), "{ty} {cy}");
            assert!(ty < cy, "the caption sits under the title");
        }
    }

    #[test]
    fn an_explicit_position_beats_the_default_spot() {
        let mut s = stage();
        let a = s.add(title("x").at(s.pixel(100, 400)));
        let b = s.add(label("y").base_at(s.pixel(900, 650)));
        let c = s.put(title("z"), 300, 300);
        let pos = |s: &Stage, id: Id| s.screen(s.scene.get(id).unwrap().transform.position);
        let ((ax, ay), (bx, by), (cx, cy)) = (pos(&s, a), pos(&s, b), pos(&s, c));
        assert!(near(ax, 100.0) && near(ay, 400.0));
        assert!(near(bx, 900.0) && near(by, 650.0));
        assert!(near(cx, 300.0) && near(cy, 300.0));
    }

    #[test]
    fn styling_a_title_keeps_its_default_spot_and_follows_zoom_and_origin() {
        let mut s = Stage::new(1280, 720).zoom(1.4).origin(300, 500);
        let t = s.add(title("big").font_size(120).color("rose"));
        let (x, y) = s.screen(s.scene.get(t).unwrap().transform.position);
        assert!(near(x, 640.0) && near(y, 720.0 * 0.097), "({x}, {y})");
    }

    #[test]
    fn a_heading_without_a_caption_is_just_the_header() {
        let mut s = stage();
        let ids = s.heading("Just a title", "");
        assert_eq!(ids.len(), 1);
        assert_eq!(s.scene.len(), 1, "no invisible empty caption was added");
    }

    #[test]
    fn heading_matches_the_hand_placed_intro_layout() {
        // At 1280x720 it is exactly where the intro video used to hand-place its headers.
        let mut s = stage();
        let ids = s.heading("T", "c");
        let (_, y1) = s.screen(s.scene.get(ids[0]).unwrap().transform.position);
        let (_, y2) = s.screen(s.scene.get(ids[1]).unwrap().transform.position);
        assert!((y1 - 70.0).abs() < 1.0 && (y2 - 128.0).abs() < 1.0, "{y1} {y2}");
    }
}
