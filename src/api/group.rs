//! [`Group`]: several objects that move, turn, scale and fade together.

use crate::animation::{
    Anim, Animation, Easing, all, create, fade_in, fade_out, lag, opacity_to, uncreate, write,
};
use crate::api::{Num, Pos, Stage};
use crate::world::arena::Id;
use crate::world::bounds::Aabb;
use crate::world::object::Object;
use crate::world::scene::Scene;
use crate::world::transform::{Vec3, euler_from_matrix, mat_apply, mat_mul, rotation_matrix};

/// Longest a group's `create()` takes by default, however many parts it has.
const MAX_BUILD_SECONDS: f32 = 3.0;
/// Gap between parts appearing, for groups small enough that this fits in the budget.
const BUILD_GAP: f32 = 0.3;

/// A set of objects treated as one thing. Build it up as you add parts, then animate it as a
/// whole:
///
/// ```no_run
/// use rough_animations::prelude::*;
///
/// fn main() -> Result {
///     let mut s = Stage::new(900, 600);
///     let mut rover = Group::new();
///     s.add_to(&mut rover, cuboid(120, 30, 70).base_at((0, 10, 0)).fill("white"));
///     let wheel = s.add_to(&mut rover, cylinder(14, 10).at((40, 14, 40)).rotated((deg(90), 0, 0)).fill("ink"));
///
///     s.play(rover.create());                          // the parts build up one after another
///     s.play(rover.move_by((60, 0, 0)).duration(2));   // the whole rover drives
///     s.play(rover.rotate_by((0, deg(90), 0)));        // and turns, wheels included
///     s.play(fade_out(wheel));                         // parts are still ordinary objects
///     s.render("rover.mp4")
/// }
/// ```
///
/// A group's *position* is the bottom centre of its bounding box (the same idea as
/// `base_at`), so `move_to((x, 0, z))` puts whatever it holds on the floor there. Rotation and
/// scaling happen around that point unless you set another with [`Group::pivot`].
#[derive(Clone, Debug, Default)]
pub struct Group {
    ids: Vec<Id>,
    pivot: Option<Vec3>,
}

impl Group {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a part that is already in the stage. Returns the same id, so it chains:
    /// `let wheel = rover.add(s.add(cylinder(..)));`.
    pub fn add(&mut self, id: Id) -> Id {
        self.ids.push(id);
        id
    }

    pub fn ids(&self) -> &[Id] {
        &self.ids
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn contains(&self, id: Id) -> bool {
        self.ids.contains(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = Id> + '_ {
        self.ids.iter().copied()
    }

    /// Turn and scale around this fixed world point instead of the bottom centre.
    pub fn pivot(mut self, point: impl Pos) -> Self {
        self.pivot = Some(point.pos());
        self
    }

    /// The box around every part, as the scene has them now (`None` for an empty group).
    pub fn bounds(&self, scene: &Scene) -> Option<Aabb> {
        let boxes: Vec<Aabb> = self.ids.iter().filter_map(|&id| scene.get(id)).map(Object::bounds).collect();
        Aabb::from_points(boxes.iter().flat_map(|b| [b.min, b.max]))
    }

    // ---- animations that move the whole group -------------------------------------------

    /// Move the group so the bottom centre of its bounding box ends up at `to`.
    pub fn move_to(&self, to: impl Pos) -> Anim {
        self.motion(Motion::MoveTo(to.pos()))
    }

    /// Move every part by `by`.
    pub fn move_by(&self, by: impl Pos) -> Anim {
        self.motion(Motion::MoveBy(by.pos()))
    }

    /// Turn the whole group by these Euler angles in radians (see [`deg`](crate::api::deg)),
    /// like turning a physical object: parts orbit the pivot and each one turns too, so wheels
    /// and other already-tilted parts stay correct.
    pub fn rotate_by(&self, angles: impl Pos) -> Anim {
        self.motion(Motion::Rotate(angles.pos()))
    }

    /// Grow or shrink the group by `factor` (2 is twice the size) around its pivot.
    pub fn scale_by(&self, factor: impl Num) -> Anim {
        self.motion(Motion::Scale(factor.f32()))
    }

    fn motion(&self, motion: Motion) -> Anim {
        Anim::new(GroupMotion {
            ids: self.ids.clone(),
            motion,
            pivot: self.pivot,
            easing: Easing::default(),
            parts: Vec::new(),
            anchor: Vec3::zero(),
        })
    }

    // ---- animations that act on every part ----------------------------------------------

    /// Draw the parts pen-style, one after another, so the group builds up. Takes about 3 s at
    /// most however many parts there are; use `.duration(..)` to change it.
    pub fn create(&self) -> Anim {
        let n = self.ids.len();
        let gap = if n > 1 {
            BUILD_GAP.min((MAX_BUILD_SECONDS - 1.0) / (n - 1) as f32)
        } else {
            0.0
        };
        lag(gap, self.ids.iter().map(|&id| create(id)))
    }

    /// The same as [`create`](Self::create); for text parts it reads better.
    pub fn write(&self) -> Anim {
        let n = self.ids.len();
        let gap = if n > 1 {
            BUILD_GAP.min((MAX_BUILD_SECONDS - 1.0) / (n - 1) as f32)
        } else {
            0.0
        };
        lag(gap, self.ids.iter().map(|&id| write(id)))
    }

    /// Erase every part together, pen-style.
    pub fn uncreate(&self) -> Anim {
        all(self.ids.iter().map(|&id| uncreate(id)))
    }

    /// Fade every part in at once.
    pub fn fade_in(&self) -> Anim {
        all(self.ids.iter().map(|&id| fade_in(id)))
    }

    /// Fade every part out at once.
    pub fn fade_out(&self) -> Anim {
        all(self.ids.iter().map(|&id| fade_out(id)))
    }

    /// Fade every part to this opacity (0 to 1) at once.
    pub fn opacity_to(&self, opacity: impl Num) -> Anim {
        let opacity = opacity.f32();
        all(self.ids.iter().map(|&id| opacity_to(id, opacity)))
    }
}

impl From<Vec<Id>> for Group {
    fn from(ids: Vec<Id>) -> Self {
        Self { ids, pivot: None }
    }
}

impl<const N: usize> From<[Id; N]> for Group {
    fn from(ids: [Id; N]) -> Self {
        Self::from(ids.to_vec())
    }
}

impl FromIterator<Id> for Group {
    fn from_iter<I: IntoIterator<Item = Id>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl Extend<Id> for Group {
    fn extend<I: IntoIterator<Item = Id>>(&mut self, iter: I) {
        self.ids.extend(iter);
    }
}

impl IntoIterator for Group {
    type Item = Id;
    type IntoIter = std::vec::IntoIter<Id>;
    fn into_iter(self) -> Self::IntoIter {
        self.ids.into_iter()
    }
}

/// So a group works with `create_all(&rover)`, `write_all(&rover)` and `clear(&rover)`.
impl<'a> IntoIterator for &'a Group {
    type Item = &'a Id;
    type IntoIter = std::slice::Iter<'a, Id>;
    fn into_iter(self) -> Self::IntoIter {
        self.ids.iter()
    }
}

impl Stage {
    /// Add an object and put it in `group` in one step. Returns its id, for animating that part
    /// on its own as well.
    pub fn add_to(&mut self, group: &mut Group, object: impl Into<Object>) -> Id {
        let id = self.add(object);
        group.add(id)
    }

    /// Move `group` right now (no animation) so the bottom centre of its bounding box is at
    /// `base`. Handy for positioning something you built at the origin.
    pub fn place(&mut self, group: &Group, base: impl Pos) {
        let base = base.pos();
        let Some(anchor) = base_center(&self.scene, &group.ids) else {
            return;
        };
        let delta = base - anchor;
        for &id in &group.ids {
            if let Some(o) = self.scene.get_mut(id) {
                o.transform.position += delta;
            }
        }
    }
}

/// Bottom centre of the box around `ids`: the middle in x and z, the lowest y.
fn base_center(scene: &Scene, ids: &[Id]) -> Option<Vec3> {
    let b = Group::from(ids.to_vec()).bounds(scene)?;
    Some(Vec3::new((b.min.x + b.max.x) / 2.0, b.min.y, (b.min.z + b.max.z) / 2.0))
}

#[derive(Clone, Copy, Debug)]
enum Motion {
    MoveTo(Vec3),
    MoveBy(Vec3),
    Rotate(Vec3),
    Scale(f32),
}

/// A part's transform when the motion began.
struct Part {
    id: Id,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

/// Moves, turns or scales a set of objects as one rigid thing. Everything is worked out from
/// where the parts are when it begins, so it chains after other animations.
struct GroupMotion {
    ids: Vec<Id>,
    motion: Motion,
    pivot: Option<Vec3>,
    easing: Easing,
    parts: Vec<Part>,
    /// The group's bottom centre when the motion began.
    anchor: Vec3,
}

impl Animation for GroupMotion {
    fn duration(&self) -> f32 {
        1.0
    }

    fn set_easing(&mut self, easing: Easing) {
        self.easing = easing;
    }

    fn begin(&mut self, scene: &mut Scene) {
        self.anchor = base_center(scene, &self.ids).unwrap_or_else(Vec3::zero);
        self.parts = self
            .ids
            .iter()
            .filter_map(|&id| {
                let t = scene.get(id)?.transform;
                Some(Part {
                    id,
                    position: t.position,
                    rotation: t.rotation,
                    scale: t.scale,
                })
            })
            .collect();
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        let t = self.easing.apply(time);
        let pivot = self.pivot.unwrap_or(self.anchor);
        for part in &self.parts {
            let Some(o) = scene.get_mut(part.id) else {
                continue;
            };
            let tf = &mut o.transform;
            match self.motion {
                Motion::MoveTo(to) => tf.position = part.position + (to - self.anchor) * t,
                Motion::MoveBy(by) => tf.position = part.position + by * t,
                Motion::Rotate(angles) => {
                    // Turn the part's position around the pivot, and compose the same turn with
                    // the part's own orientation (matrices, because adding Euler angles is only
                    // right for parts that have not been turned already).
                    let turn = rotation_matrix(angles * t);
                    tf.position = pivot + mat_apply(&turn, part.position - pivot);
                    let own = rotation_matrix(part.rotation);
                    tf.rotation = euler_from_matrix(&mat_mul(&turn, &own));
                }
                Motion::Scale(factor) => {
                    let k = 1.0 + (factor - 1.0) * t;
                    tf.position = pivot + (part.position - pivot) * k;
                    tf.scale = part.scale * k;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::deg;
    use crate::api::shapes::*;
    use crate::world::transform::Transform;
    use std::f32::consts::FRAC_PI_2;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-3
    }

    fn pos(s: &Stage, id: Id) -> Vec3 {
        s.scene.get(id).unwrap().transform.position
    }

    /// A body and two wheels (the wheels turned a quarter about X, like a rover's).
    fn rover(s: &mut Stage) -> (Group, Id, Id, Id) {
        let mut g = Group::new();
        let body = s.add_to(&mut g, cuboid(40, 10, 20).base_at((0, 10, 0)));
        let w1 = s.add_to(&mut g, cylinder(8, 6).at((-12, 8, 13)).rotated((deg(90), 0, 0)));
        let w2 = s.add_to(&mut g, cylinder(8, 6).at((12, 8, 13)).rotated((deg(90), 0, 0)));
        (g, body, w1, w2)
    }

    #[test]
    fn building_and_converting_groups() {
        let mut s = Stage::new(100, 100);
        let (a, b, c) = (s.add(cube(1)), s.add(cube(1)), s.add(cube(1)));
        let mut g = Group::new();
        assert!(g.is_empty());
        assert_eq!(g.add(a), a, "add returns the id");
        g.extend([b, c]);
        assert_eq!((g.len(), g.contains(b), g.ids()), (3, true, &[a, b, c][..]));
        assert_eq!(Group::from([a, b]).len(), 2);
        assert_eq!(Group::from(vec![a]).len(), 1);
        assert_eq!([a, b, c].into_iter().collect::<Group>().len(), 3);
        assert_eq!(g.iter().collect::<Vec<_>>(), vec![a, b, c]);
        assert_eq!(g.clone().into_iter().count(), 3);
        // add_to puts the object in the stage and the group.
        let before = s.scene.len();
        let d = s.add_to(&mut g, cube(2));
        assert_eq!((s.scene.len(), g.contains(d)), (before + 1, true));
    }

    #[test]
    fn move_by_shifts_every_part_equally_and_keeps_the_shape() {
        let mut s = Stage::new(100, 100);
        let (g, body, w1, w2) = rover(&mut s);
        let before: Vec<Vec3> = [body, w1, w2].iter().map(|&i| pos(&s, i)).collect();
        s.play(g.move_by((50, 0, -10)).ease(Easing::Linear));
        s.seek(0.5);
        for (i, &id) in [body, w1, w2].iter().enumerate() {
            assert!(close(pos(&s, id), before[i] + Vec3::new(25., 0., -5.)), "halfway");
        }
        s.seek(1.0);
        for (i, &id) in [body, w1, w2].iter().enumerate() {
            assert!(close(pos(&s, id), before[i] + Vec3::new(50., 0., -10.)), "arrived");
        }
    }

    #[test]
    fn move_to_puts_the_bottom_centre_of_the_group_at_the_target() {
        let mut s = Stage::new(100, 100);
        let mut g = Group::new();
        s.add_to(&mut g, cuboid(10, 10, 10).base_at((0, 0, 0)));
        s.add_to(&mut g, cuboid(10, 10, 10).base_at((20, 0, 0)));
        // Combined x from -5 to 25, so the middle is 10; y starts at 0; z is centred on 0.
        assert!(close(base_center(&s.scene, g.ids()).unwrap(), Vec3::new(10., 0., 0.)));
        s.play(g.move_to((100, 5, 40)));
        s.seek(1.0);
        assert!(close(base_center(&s.scene, g.ids()).unwrap(), Vec3::new(100., 5., 40.)));
        // The parts kept their spacing.
        let b = g.bounds(&s.scene).unwrap();
        assert!((b.max.x - b.min.x - 30.0).abs() < 1e-3);
        // And chaining a second move starts from where the first ended.
        s.play(g.move_by((0, 0, 10)));
        s.seek(2.0);
        assert!(close(base_center(&s.scene, g.ids()).unwrap(), Vec3::new(100., 5., 50.)));
    }

    #[test]
    fn rotating_turns_the_group_like_one_rigid_object() {
        let mut s = Stage::new(100, 100);
        let (g, body, w1, w2) = rover(&mut s);
        let ids = [body, w1, w2];
        let start: Vec<Vec3> = ids.iter().map(|&i| pos(&s, i)).collect();
        let anchor = base_center(&s.scene, g.ids()).unwrap();
        s.play(g.rotate_by((0, FRAC_PI_2, 0)).ease(Easing::Linear));

        for t in [0.3, 0.7, 1.0] {
            s.seek(t);
            let now: Vec<Vec3> = ids.iter().map(|&i| pos(&s, i)).collect();
            // Distances between parts never change: it is rigid.
            for i in 0..3 {
                for j in i + 1..3 {
                    assert!(((now[i] - now[j]).length() - (start[i] - start[j]).length()).abs() < 1e-3, "t={t}");
                }
                // Each part stays the same distance from the pivot.
                assert!(((now[i] - anchor).length() - (start[i] - anchor).length()).abs() < 1e-3);
            }
        }
        // A quarter turn about Y sends a point at +x of the pivot to -z of it.
        let w = pos(&s, w2) - anchor;
        let w0 = start[2] - anchor;
        assert!(close(w, Vec3::new(w0.z, w0.y, -w0.x)), "{w:?} from {w0:?}");
    }

    #[test]
    fn already_turned_parts_keep_the_right_orientation_after_the_group_turns() {
        // The wheels are turned a quarter about X. After the group yaws a quarter about Y, their
        // orientation must be (yaw * their own turn), which adding Euler angles would get wrong.
        let mut s = Stage::new(100, 100);
        let (g, _, w1, _) = rover(&mut s);
        s.play(g.rotate_by((0, FRAC_PI_2, 0)));
        s.seek(1.0);
        let got = Transform {
            rotation: s.scene.get(w1).unwrap().transform.rotation,
            ..Transform::IDENTITY
        }
        .rotation_matrix();
        let expected = mat_mul(
            &rotation_matrix(Vec3::new(0., FRAC_PI_2, 0.)),
            &rotation_matrix(Vec3::new(FRAC_PI_2, 0., 0.)),
        );
        for r in 0..3 {
            for c in 0..3 {
                assert!((got[r][c] - expected[r][c]).abs() < 1e-3, "{got:?} vs {expected:?}");
            }
        }
        // The wheel's axis (local +Y turned by the part's own rotation) points along the world
        // direction the whole thing was turned to: z -> x after a quarter yaw.
        let axis = mat_apply(&got, Vec3::new(0., 1., 0.));
        assert!(close(axis, Vec3::new(1., 0., 0.)), "{axis:?}");
    }

    #[test]
    fn orientation_is_the_true_composition_for_any_part_and_any_group_turn() {
        // Adding Euler angles is only right for some combinations (the rover's wheels happen to
        // be one). Parts with a Z turn, or a group that rolls or pitches, need real matrices.
        let part_turns = [
            Vec3::new(0., 0., FRAC_PI_2),          // tilted in its own plane, like an eyebrow
            Vec3::new(0.4, 0.7, 0.3),              // a bit of everything
            Vec3::new(FRAC_PI_2, 0., 0.),          // the rover's wheels
            Vec3::new(0., 1.0, -0.5),
        ];
        let group_turns = [
            Vec3::new(0., FRAC_PI_2, 0.),          // yaw
            Vec3::new(FRAC_PI_2, 0., 0.),          // pitch
            Vec3::new(0., 0., 0.8),                // roll
            Vec3::new(0.5, -0.9, 0.3),
        ];
        for own in part_turns {
            for turn in group_turns {
                let mut s = Stage::new(100, 100);
                let mut g = Group::new();
                let part = s.add_to(&mut g, cube(4).at((6, 2, -3)).rotated(own));
                s.add_to(&mut g, cube(4).at((-9, 1, 5))); // so the pivot is not the part itself
                s.play(g.rotate_by(turn));
                s.seek(1.0);

                let got = Transform {
                    rotation: s.scene.get(part).unwrap().transform.rotation,
                    ..Transform::IDENTITY
                }
                .rotation_matrix();
                let expected = mat_mul(&rotation_matrix(turn), &rotation_matrix(own));
                for r in 0..3 {
                    for c in 0..3 {
                        assert!(
                            (got[r][c] - expected[r][c]).abs() < 1e-3,
                            "part turned {own:?}, group turned {turn:?}: {got:?} vs {expected:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_full_turn_comes_back_to_where_it_started() {
        let mut s = Stage::new(100, 100);
        let (g, body, w1, _) = rover(&mut s);
        let (p0, q0) = (pos(&s, body), pos(&s, w1));
        s.play(g.rotate_by((0, 2.0 * std::f32::consts::PI, 0)));
        s.seek(1.0);
        assert!(close(pos(&s, body), p0) && close(pos(&s, w1), q0));
    }

    #[test]
    fn an_explicit_pivot_changes_what_it_turns_around() {
        let mut s = Stage::new(100, 100);
        let mut g = Group::new();
        let a = s.add_to(&mut g, cube(4).at((10, 0, 0)));
        let g = g.pivot((0, 0, 0));
        s.play(g.rotate_by((0, FRAC_PI_2, 0)));
        s.seek(1.0);
        // About the origin, +x goes to -z (a single part would otherwise just spin in place).
        assert!(close(pos(&s, a), Vec3::new(0., 0., -10.)), "{:?}", pos(&s, a));
    }

    #[test]
    fn scale_by_grows_the_group_around_its_base() {
        let mut s = Stage::new(100, 100);
        let mut g = Group::new();
        let a = s.add_to(&mut g, cuboid(10, 10, 10).base_at((0, 0, 0)));
        let b = s.add_to(&mut g, cuboid(10, 10, 10).base_at((30, 0, 0)));
        let anchor = base_center(&s.scene, g.ids()).unwrap();
        let gap = (pos(&s, a) - pos(&s, b)).length();
        s.play(g.scale_by(2));
        s.seek(1.0);
        assert!(((pos(&s, a) - pos(&s, b)).length() - 2.0 * gap).abs() < 1e-3, "spacing doubles");
        assert!(close(s.scene.get(a).unwrap().transform.scale, Vec3::new(2., 2., 2.)));
        assert!(close(base_center(&s.scene, g.ids()).unwrap(), anchor), "the base stays put");
    }

    #[test]
    fn create_write_and_fade_hide_the_parts_until_they_start() {
        for make in [
            Group::create as fn(&Group) -> Anim,
            Group::write,
            Group::fade_in,
        ] {
            let mut s = Stage::new(100, 100);
            let (g, ..) = rover(&mut s);
            s.wait(1);
            s.play(make(&g));
            assert!(g.iter().all(|id| !s.scene.get(id).unwrap().visible), "hidden at the start");
            let end = s.duration();
            s.seek(end);
            assert!(g.iter().all(|id| s.scene.get(id).unwrap().visible), "visible at the end");
        }
    }

    #[test]
    fn building_staggers_but_fading_is_together_and_long_groups_stay_quick() {
        let mut s = Stage::new(100, 100);
        let dur = |a: Anim| Animation::duration(&a);
        let three: Group = (0..3).map(|_| s.add(cube(1))).collect();
        assert!((dur(three.create()) - (0.3 * 2.0 + 1.0)).abs() < 1e-4, "small groups: 0.3 s apart");
        let forty: Group = (0..40).map(|_| s.add(cube(1))).collect();
        assert!(dur(forty.create()) <= 3.0 + 1e-4, "big groups still build in about 3 s");
        assert!((dur(forty.fade_in()) - 1.0).abs() < 1e-4, "fading is all at once");
        assert!((dur(forty.uncreate()) - 1.0).abs() < 1e-4);
        assert!((dur(three.create().duration(5)) - 5.0).abs() < 1e-4, ".duration() rescales");
    }

    #[test]
    fn fades_and_opacity_reach_every_part() {
        let mut s = Stage::new(100, 100);
        let (g, ..) = rover(&mut s);
        s.play(g.fade_out());
        s.play(g.opacity_to(0.5));
        s.seek(2.0);
        assert!(g.iter().all(|id| s.scene.get(id).unwrap().style.opacity == 0.5));
    }

    #[test]
    fn empty_and_partly_removed_groups_do_not_break_anything() {
        let mut s = Stage::new(100, 100);
        let empty = Group::new();
        assert!(empty.bounds(&s.scene).is_none());
        for a in [
            empty.move_by((1, 2, 3)),
            empty.move_to((1, 2, 3)),
            empty.rotate_by((0, 1, 0)),
            empty.scale_by(2),
            empty.create(),
            empty.fade_in(),
            empty.uncreate(),
        ] {
            s.play(a);
        }
        s.place(&empty, (1, 2, 3)); // a no-op
        let end = s.duration();
        s.seek(end);

        // A part removed from the scene is skipped; the rest still move.
        let (g, body, w1, _) = rover(&mut s);
        s.scene.remove(w1);
        let before = pos(&s, body);
        s.play(g.move_by((10, 0, 0)));
        let end = s.duration();
        s.seek(end);
        assert!(close(pos(&s, body), before + Vec3::new(10., 0., 0.)));
    }

    #[test]
    fn place_positions_a_group_immediately() {
        let mut s = Stage::new(100, 100);
        let (g, ..) = rover(&mut s);
        s.place(&g, (200, 30, -40));
        assert!(close(base_center(&s.scene, g.ids()).unwrap(), Vec3::new(200., 30., -40.)));
    }

    #[test]
    fn a_group_works_with_the_id_list_helpers() {
        use crate::animation::{clear, create_all, write_all};
        let mut s = Stage::new(100, 100);
        let (g, ..) = rover(&mut s);
        let dur = |a: Anim| Animation::duration(&a);
        assert!(dur(create_all(&g)) > 1.0);
        assert!(dur(write_all(&g)) > 1.0);
        assert!((dur(clear(&g)) - 0.7).abs() < 1e-4);
        s.play(create_all(&g));
        s.play(clear(&g));
        let end = s.duration();
        s.seek(end);
        assert!(g.iter().all(|id| s.scene.get(id).unwrap().style.reveal == 0.0));
    }
}
