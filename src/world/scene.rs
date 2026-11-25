use crate::primitives::{DrawCtx, Pass};
use crate::render::camera::IsoCamera;
use crate::world::arena::{Arena, Id};
use crate::world::bounds::Aabb;
use crate::world::object::Object;
use roughr::core::{FillStyle, Options};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tiny_skia::{Color, PixmapMut};

/// Off-white paper, matching the default `Stage` background.
fn default_paper() -> Color {
    Color::from_rgba8(250, 249, 246, 255)
}

struct Item {
    id: Id,
    layer: i32,
    aabb: Aabb,
    rect: [f32; 4],
    depth: f32,
}

fn rects_overlap(a: &[f32; 4], b: &[f32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// Topologically sort one layer: an edge `i -> j` means `i` must be drawn before `j`.
/// Returns indices into `items`. Cycles (rare, e.g. three interlocking boxes) are broken by
/// drawing the farthest remaining object first.
fn order_layer(items: &[Item], view: [f32; 3]) -> Vec<usize> {
    let n = items.len();
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut indeg = vec![0usize; n];
    for i in 0..n {
        for j in i + 1..n {
            if !rects_overlap(&items[i].rect, &items[j].rect) {
                continue;
            }
            let (ij, ji) = (
                items[i].aabb.behind_from(&items[j].aabb, view),
                items[j].aabb.behind_from(&items[i].aabb, view),
            );
            let edge = match (ij, ji) {
                (true, false) => Some((i, j)),
                (false, true) => Some((j, i)),
                // Separated both ways: they only touch on screen, order doesn't matter.
                (true, true) => None,
                // Interpenetrating boxes: fall back to centre depth.
                (false, false) => Some(if items[j].depth < items[i].depth { (j, i) } else { (i, j) }),
            };
            if let Some((from, to)) = edge {
                succ[from].push(to);
                indeg[to] += 1;
            }
        }
    }

    let mut placed = vec![false; n];
    let mut order = Vec::with_capacity(n);
    for _ in 0..n {
        let pick = |only_free: bool| {
            (0..n)
                .filter(|&k| !placed[k] && (!only_free || indeg[k] == 0))
                .min_by(|&a, &b| items[a].depth.total_cmp(&items[b].depth).then(a.cmp(&b)))
        };
        let next = pick(true).or_else(|| pick(false)).expect("an unplaced item remains");
        placed[next] = true;
        order.push(next);
        for &s in &succ[next] {
            indeg[s] = indeg[s].saturating_sub(1);
        }
    }
    order
}

/// Whether `(x, y)` is inside a convex polygon given counter-clockwise (in a y-up frame; on
/// screen, with y down, the same order reads clockwise, so either winding is accepted).
fn point_in_convex(hull: &[euclid::default::Point2D<f32>], x: f32, y: f32) -> bool {
    if hull.len() < 3 {
        return false;
    }
    let mut sign = 0.0_f32;
    for i in 0..hull.len() {
        let (a, b) = (hull[i], hull[(i + 1) % hull.len()]);
        let cross = (b.x - a.x) * (y - a.y) - (b.y - a.y) * (x - a.x);
        if cross.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    true
}

fn stroke_seed(id: Id, step: u64) -> u64 {
    // DefaultHasher::new() uses fixed keys, so this is deterministic across runs.
    let mut h = DefaultHasher::new();
    (id, step).hash(&mut h);
    h.finish() & 0xFFFF_FFFF_FFFF_0000 // leave low bits for the per-polygon counter
}

/// Owns every object. Hand out [`Id`]s, mutate objects through them, render.
pub struct Scene {
    camera: IsoCamera,
    base_opts: Options,
    objects: Arena<Object>,
    /// Paper colour. Filled shapes get an opaque patch of it underneath, so nearer shapes hide
    /// the lines of farther ones. Set to `None` for see-through fills. A [`Stage`] keeps
    /// this in sync with its background.
    ///
    /// [`Stage`]: crate::api::stage::Stage
    pub paper: Option<Color>,
}

impl Scene {
    pub fn new(camera: IsoCamera, base_opts: Options) -> Self {
        Self {
            camera,
            base_opts,
            objects: Arena::new(),
            paper: Some(default_paper()),
        }
    }

    /// The rough-drawing options (roughness, hatching, ...) in force.
    pub fn options(&self) -> &Options {
        &self.base_opts
    }

    /// Replace the rough-drawing options (roughness, hatching, ...).
    pub fn set_options(&mut self, options: Options) {
        self.base_opts = options;
    }

    pub fn camera(&self) -> &IsoCamera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut IsoCamera {
        &mut self.camera
    }

    /// Add an object (or any geometry, which becomes an object at the origin).
    pub fn add(&mut self, object: impl Into<Object>) -> Id {
        self.objects.insert(object.into())
    }

    pub fn remove(&mut self, id: Id) -> Option<Object> {
        self.objects.remove(id)
    }

    pub fn get(&self, id: Id) -> Option<&Object> {
        self.objects.get(id)
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Object> {
        self.objects.get_mut(id)
    }

    /// Every object, for changing them all at once (e.g. applying a theme).
    pub fn objects_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.objects.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Ids in painter's order, back to front.
    ///
    /// Objects are grouped by layer (lower first). Inside a layer, whenever two objects overlap
    /// on screen, the one that lies entirely on the far side of the other (along x, y or z) is
    /// drawn first; this is exact for boxes and a safe approximation for the rest. Pairs that
    /// interpenetrate are ordered by centre depth, and ties keep insertion order.
    pub(crate) fn draw_order(&self) -> Vec<Id> {
        let view = self.camera.toward_viewer();
        let mut items: Vec<Item> = self
            .objects
            .iter()
            .filter(|(_, o)| o.visible && o.style.opacity > 0.0 && o.style.reveal > 0.0)
            .map(|(id, o)| {
                let aabb = o.bounds();
                Item {
                    id,
                    layer: o.layer,
                    rect: self.screen_rect(&aabb),
                    depth: o.depth(view).max(f32::MIN).min(f32::MAX),
                    aabb,
                }
            })
            .collect();
        items.sort_by_key(|i| i.layer); // stable: insertion order within a layer

        let mut out = Vec::with_capacity(items.len());
        let mut start = 0;
        while start < items.len() {
            let layer = items[start].layer;
            let end = items[start..]
                .iter()
                .position(|i| i.layer != layer)
                .map_or(items.len(), |n| start + n);
            let group = &items[start..end];
            out.extend(order_layer(group, view).into_iter().map(|i| group[i].id));
            start = end;
        }
        out
    }

    /// The topmost visible object under screen point `(x, y)`, if any.
    ///
    /// Each object is tested by the screen outline of its bounding box (the convex hull of the
    /// box's projected corners), so a round object counts a little beyond its drawn edge.
    /// Objects without a box (plain text) are never picked.
    pub fn pick(&self, x: f32, y: f32) -> Option<Id> {
        let order = self.draw_order();
        order.into_iter().rev().find(|&id| {
            let Some(o) = self.objects.get(id) else { return false };
            if o.geometry.bounds(&o.transform).is_none() {
                return false;
            }
            let pts: Vec<_> = o
                .bounds()
                .corners()
                .iter()
                .map(|c| self.camera.project(c.x, c.y, c.z))
                .collect();
            let hull = crate::primitives::common::convex_hull(&pts);
            point_in_convex(&hull, x, y)
        })
    }

    /// Screen-space rectangle `[min_x, min_y, max_x, max_y]` covered by a world box.
    fn screen_rect(&self, b: &Aabb) -> [f32; 4] {
        let mut r = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for c in b.corners() {
            let p = self.camera.project(c.x, c.y, c.z);
            r = [r[0].min(p.x), r[1].min(p.y), r[2].max(p.x), r[3].max(p.y)];
        }
        r
    }

    /// Draw the scene onto `pixmap`, depth-sorted back to front. Strokes are stable:
    /// rendering twice gives identical pixels.
    pub fn render(&self, pixmap: &mut PixmapMut) {
        self.render_wobble(pixmap, 0);
    }

    /// Like [`Scene::render`], but `wobble_step` reseeds the rough strokes. Incrementing it
    /// over time gives the "boiling line" hand-drawn look.
    pub fn render_wobble(&self, pixmap: &mut PixmapMut, wobble_step: u64) {
        let mut ctx = DrawCtx {
            pixmap,
            camera: &self.camera,
            base_opts: &self.base_opts,
            paper: self.paper,
            pass: Pass::Ink,
            seed: 0,
            draw_count: 0,
            svg: None,
        };
        self.paint(&mut ctx, wobble_step);
    }

    /// Draw the scene as SVG elements (no `<svg>` wrapper), back to front. The result has the
    /// same strokes as [`Scene::render_wobble`] with the same step, as vector paths.
    pub fn render_svg_body(&self, wobble_step: u64) -> String {
        let mut body = String::new();
        let mut scratch = tiny_skia::Pixmap::new(1, 1).expect("1x1 pixmap");
        {
            let mut pm = scratch.as_mut();
            let mut ctx = DrawCtx {
                pixmap: &mut pm,
                camera: &self.camera,
                base_opts: &self.base_opts,
                paper: self.paper,
                pass: Pass::Ink,
                seed: 0,
                draw_count: 0,
                svg: Some(&mut body),
            };
            self.paint(&mut ctx, wobble_step);
        }
        body
    }

    fn paint(&self, ctx: &mut DrawCtx, wobble_step: u64) {
        for id in self.draw_order() {
            let o = self.objects.get(id).expect("id from draw_order is live");
            // Pass 1: opaque patches that hide whatever is behind this object.
            let occludes = self.paper.is_some()
                && o.style.fill.is_some()
                && o.style.fill_style != Some(FillStyle::Solid);
            if occludes {
                ctx.pass = Pass::Underlay;
                o.geometry.draw(ctx, &o.transform, &o.style);
            }
            // Pass 2: the ink. Seeds depend only on this pass, so strokes don't change.
            ctx.pass = Pass::Ink;
            ctx.seed = stroke_seed(id, if o.geometry.wobbles() { wobble_step } else { 0 });
            ctx.draw_count = 0;
            o.geometry.draw(ctx, &o.transform, &o.style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{Box3D, Geometry};
    use crate::world::style::rgb;
    use tiny_skia::Pixmap;

    fn scene() -> Scene {
        Scene::new(IsoCamera::new(100.0, 100.0), Options::default())
    }

    #[test]
    fn draws_far_objects_first() {
        let mut s = scene();
        let near = s.add(Box3D::new(10., 10., 10.).at([50., 0., 50.]));
        let far = s.add(Box3D::new(10., 10., 10.).at([0., 0., 0.]));
        assert_eq!(s.draw_order(), vec![far, near]);
    }

    #[test]
    fn hidden_and_transparent_objects_are_skipped() {
        let mut s = scene();
        s.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]).hidden());
        s.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]).opacity(0.0));
        let shown = s.add(Box3D::new(1., 1., 1.).at([0., 0., 0.]));
        assert_eq!(s.draw_order(), vec![shown]);
    }

    #[test]
    fn mutating_through_id_changes_render_order() {
        let mut s = scene();
        let a = s.add(Box3D::new(10., 10., 10.).at([0., 0., 0.]));
        let b = s.add(Box3D::new(10., 10., 10.).at([50., 0., 50.]));
        assert_eq!(s.draw_order(), vec![a, b]);
        s.get_mut(a).unwrap().transform.position.x = 500.0;
        assert_eq!(s.draw_order(), vec![b, a]);
    }

    fn frame(s: &Scene, step: u64) -> Vec<u8> {
        let mut pm = Pixmap::new(200, 200).unwrap();
        s.render_wobble(&mut pm.as_mut(), step);
        pm.data().to_vec()
    }

    #[test]
    fn strokes_are_stable_and_wobble_reseeds() {
        let mut s = scene();
        s.add(
            Box3D::new(40., 40., 40.)
                .at([0., 20., 0.])
                .fill(rgb(200, 80, 80)),
        );
        assert_eq!(frame(&s, 0), frame(&s, 0));
        assert_ne!(frame(&s, 0), frame(&s, 1));
    }

    // ---- ordering ----------------------------------------------------------------------

    #[test]
    fn things_standing_on_a_big_slab_are_drawn_after_it() {
        let mut s = scene();
        // Listed in the "wrong" order on purpose; the slab's centre is far nearer than the
        // box's, so a centre-depth sort would draw the slab last and over the box.
        let boxed = s.add(Box3D::new(80., 110., 80.).at([30., 75., 30.]));
        let slab = s.add(Box3D::new(300., 20., 300.).at([110., 10., 110.]));
        assert_eq!(s.draw_order(), vec![slab, boxed]);
    }

    #[test]
    fn higher_objects_are_nearer() {
        let mut s = scene();
        let plate = s.add(Box3D::new(70., 20., 70.).at([30., 175., 30.]));
        let below = s.add(Box3D::new(80., 110., 80.).at([30., 75., 30.]));
        assert_eq!(s.draw_order(), vec![below, plate]);
    }

    #[test]
    fn a_box_in_front_in_z_or_x_is_drawn_later() {
        let mut s = scene();
        let front = s.add(Box3D::new(40., 40., 40.).at([0., 20., 100.]));
        let back = s.add(Box3D::new(40., 40., 40.).at([0., 20., 0.]));
        let right = s.add(Box3D::new(40., 40., 40.).at([100., 20., 0.]));
        let order = s.draw_order();
        let pos = |id| order.iter().position(|&x| x == id).unwrap();
        assert!(pos(back) < pos(front));
        assert!(pos(back) < pos(right));
    }

    #[test]
    fn non_overlapping_objects_keep_depth_then_insertion_order() {
        let mut s = scene();
        let a = s.add(Box3D::new(10., 10., 10.).at([0., 0., 0.]));
        let b = s.add(Box3D::new(10., 10., 10.).at([500., 0., -500.])); // far apart on screen
        assert_eq!(s.draw_order(), vec![a, b]);
    }

    #[test]
    fn ordering_is_always_a_permutation_even_with_awkward_overlaps() {
        // Pseudo-random piles of overlapping boxes, to shake out hangs, duplicates and drops
        // (including the interlocking cases that no ordering can satisfy).
        let mut seed = 0x9E3779B9u32;
        let mut rnd = move |range: f32| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32 * range
        };
        for _ in 0..200 {
            let mut s = scene();
            let ids: Vec<_> = (0..12)
                .map(|_| {
                    s.add(
                        Box3D::new(10. + rnd(60.), 10. + rnd(60.), 10. + rnd(60.))
                            .at([rnd(80.), rnd(80.), rnd(80.)]),
                    )
                })
                .collect();
            let mut order = s.draw_order();
            assert_eq!(order.len(), ids.len());
            order.sort_by_key(|id| ids.iter().position(|x| x == id));
            assert_eq!(order, ids);
        }
    }

    #[test]
    fn layers_still_beat_geometry() {
        let mut s = scene();
        let label = s.add(Box3D::new(1., 1., 1.).at([-500., -500., -500.]).layer(2));
        let boxed = s.add(Box3D::new(10., 10., 10.).at([0., 0., 0.]));
        assert_eq!(s.draw_order(), vec![boxed, label]);
    }

    // ---- occlusion ---------------------------------------------------------------------

    fn pixels(s: &Scene) -> Pixmap {
        let mut pm = Pixmap::new(300, 300).unwrap();
        s.render(&mut pm.as_mut());
        pm
    }

    /// A big front box, and a smaller one tucked entirely behind it on screen.
    fn front_and_hidden(paper: bool) -> (Scene, Scene) {
        let mk = |with_back: bool| {
            let mut s = scene();
            if !paper {
                s.paper = None;
            }
            // Stroke seeds come from object ids, so keep ids identical across both scenes
            // by adding the back box either way, hidden when it should be absent.
            let back = Box3D::new(30., 30., 30.).at([-45., 15., -45.]).fill(rgb(40, 40, 200));
            s.add(if with_back { back } else { back.hidden() });
            s.add(Box3D::new(90., 90., 90.).at([0., 45., 0.]).fill(rgb(250, 200, 80)));
            s
        };
        (mk(false), mk(true))
    }

    #[test]
    fn a_front_box_hides_the_lines_of_one_behind_it() {
        let (alone, with_back) = front_and_hidden(true);
        let (a, b) = (pixels(&alone), pixels(&with_back));
        // Everywhere the front box is opaque, adding the hidden box changes nothing.
        let opaque: Vec<usize> = (0..a.pixels().len())
            .filter(|&i| a.pixels()[i].alpha() == 255)
            .collect();
        assert!(opaque.len() > 3000, "front box should be opaque, got {}", opaque.len());
        for i in opaque {
            assert_eq!(a.pixels()[i], b.pixels()[i], "pixel {i} leaks the hidden box");
        }
    }

    #[test]
    fn without_paper_hidden_lines_show_through() {
        let (alone, with_back) = front_and_hidden(false);
        let (a, b) = (pixels(&alone), pixels(&with_back));
        assert_ne!(a.data(), b.data(), "see-through fills reveal what is behind");
    }

    #[test]
    fn faded_objects_only_partly_hide_what_is_behind() {
        let render_with = |opacity: f32| {
            let mut s = scene();
            s.add(Box3D::new(90., 90., 90.).at([0., 45., 0.]).fill(rgb(250, 200, 80)).opacity(opacity));
            pixels(&s)
        };
        let alpha_sum = |pm: &Pixmap| pm.pixels().iter().map(|p| p.alpha() as u32).sum::<u32>();
        let (full, half) = (render_with(1.0), render_with(0.5));
        assert!(alpha_sum(&half) < alpha_sum(&full) * 3 / 4);
        assert_eq!(alpha_sum(&render_with(0.0)), 0);
    }

    #[test]
    fn outline_only_shapes_do_not_hide_anything() {
        let mut s = scene();
        // No fill: the interior must stay transparent.
        s.add(Box3D::new(90., 90., 90.).at([0., 45., 0.]));
        let pm = pixels(&s);
        assert_eq!(pm.pixel(150, 120).unwrap().alpha(), 0);
    }

    #[test]
    fn timeline_keeps_the_paper_in_sync_with_its_background() {
        use crate::api::stage::Stage;
        let mut tl = Stage::with(scene(), 100, 100);
        tl.background = Color::from_rgba8(10, 20, 30, 255);
        tl.render_frame(0.0);
        assert_eq!(tl.scene.paper, Some(Color::from_rgba8(10, 20, 30, 255)));
        // And an explicit opt-out is respected.
        let mut tl = Stage::with(scene(), 100, 100);
        tl.scene.paper = None;
        tl.render_frame(0.0);
        assert_eq!(tl.scene.paper, None);
    }

    #[test]
    fn render_puts_pixels_on_canvas() {
        let mut s = scene();
        s.add(
            Box3D::new(40., 40., 40.)
                .at([0., 20., 0.])
                .fill(rgb(200, 80, 80)),
        );
        let mut pm = Pixmap::new(200, 200).unwrap();
        s.render(&mut pm.as_mut());
        assert!(pm.data().iter().any(|&b| b != 0));
    }
}
