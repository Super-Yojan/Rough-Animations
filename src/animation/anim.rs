use super::{Animation, Easing, Schedule, create, uncreate, write};
use crate::api::Num;
use crate::world::arena::Id;
use crate::world::scene::Scene;
use std::borrow::Borrow;

/// Any animation: every constructor (`move_to`, `create`, `all`, ...) returns one, so they can
/// be put in arrays and combined without boxing.
///
/// ```
/// use rough_animations::prelude::*;
/// let mut s = Stage::new(400, 300);
/// let (a, b) = (s.add(cube(40)), s.add(sphere(20)));
/// let intro = all([move_to(a, (0, 40, 0)).duration(1.2), fade_out(b)]).delay(0.5);
/// s.play(intro);
/// ```
pub struct Anim {
    inner: Box<dyn Animation>,
    /// The inner animation's own length; `target` rescales it.
    natural: f32,
    target: Option<f32>,
}

impl Anim {
    pub fn new(animation: impl Animation + 'static) -> Self {
        Self {
            natural: animation.duration(),
            inner: Box::new(animation),
            target: None,
        }
    }

    /// Make it last `seconds` (works on anything, including groups).
    pub fn duration(mut self, seconds: impl Num) -> Self {
        self.target = Some(seconds.f32().max(0.0));
        self
    }

    /// Easing curve, for single animations (`move_to`, `fade_in`, `morph`, ...).
    pub fn ease(mut self, easing: Easing) -> Self {
        self.inner.set_easing(easing);
        self
    }

    /// Wait `seconds` first. `duration` always means the total length, so call `.duration()`
    /// before `.delay()`: `move_to(a, p).duration(1).delay(0.5)` moves for 1 s after a 0.5 s wait.
    pub fn delay(self, seconds: impl Num) -> Anim {
        seq([super::wait(seconds), self])
    }

    /// Run `next` after this.
    pub fn then(self, next: Anim) -> Anim {
        seq([self, next])
    }

    /// Run `other` at the same time as this.
    pub fn with(self, other: Anim) -> Anim {
        all([self, other])
    }

    fn length(&self) -> f32 {
        self.target.unwrap_or(self.natural)
    }
}

impl Animation for Anim {
    fn duration(&self) -> f32 {
        self.length()
    }

    fn begin(&mut self, scene: &mut Scene) {
        self.inner.begin(scene);
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        let len = self.length();
        let local = if len > 0.0 { time * self.natural / len } else { self.natural };
        self.inner.update(scene, local.min(self.natural));
    }

    fn set_easing(&mut self, easing: Easing) {
        self.inner.set_easing(easing);
    }

    fn introduces(&self, out: &mut Vec<Id>) {
        self.inner.introduces(out);
    }

    fn consumes(&self, out: &mut Vec<Id>) {
        self.inner.consumes(out);
    }
}

/// Run all at once; lasts as long as the longest.
pub fn all(anims: impl IntoIterator<Item = Anim>) -> Anim {
    let mut g = Schedule::new();
    for a in anims {
        g.push_at(0.0, a);
    }
    Anim::new(g)
}

/// Run one after another.
pub fn seq(anims: impl IntoIterator<Item = Anim>) -> Anim {
    let mut g = Schedule::new();
    let mut t = 0.0;
    for a in anims {
        let d = Animation::duration(&a);
        g.push_at(t, a);
        t += d;
    }
    Anim::new(g)
}

/// Start each one `gap` seconds after the previous; they overlap.
pub fn lag(gap: impl Num, anims: impl IntoIterator<Item = Anim>) -> Anim {
    let gap = gap.f32().max(0.0);
    let mut g = Schedule::new();
    for (i, a) in anims.into_iter().enumerate() {
        g.push_at(gap * i as f32, a);
    }
    Anim::new(g)
}

/// Write each of `ids` in turn, 0.25 s apart (they overlap): `write_all(&labels)`.
/// Accepts an array, a `Vec`, a slice or a reference to one.
pub fn write_all<I>(ids: I) -> Anim
where
    I: IntoIterator,
    I::Item: Borrow<Id>,
{
    lag(0.25, ids.into_iter().map(|id| write(*id.borrow())))
}

/// Draw each of `ids` pen-style in turn, 0.3 s apart (they overlap): `create_all(&solids)`.
pub fn create_all<I>(ids: I) -> Anim
where
    I: IntoIterator,
    I::Item: Borrow<Id>,
{
    lag(0.3, ids.into_iter().map(|id| create(*id.borrow())))
}

/// Erase all of `ids` together, pen-style, over 0.7 s: `clear(&scene_one)`. Use it to wipe a
/// scene before the next one. Call `.duration(..)` on the result for a different speed.
pub fn clear<I>(ids: I) -> Anim
where
    I: IntoIterator,
    I::Item: Borrow<Id>,
{
    let erasing: Vec<Anim> = ids.into_iter().map(|id| uncreate(*id.borrow())).collect();
    if erasing.is_empty() {
        return all(erasing); // nothing to erase, so no pause either
    }
    all(erasing).duration(0.7)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::wait;

    #[test]
    fn duration_rescales_anything() {
        let a = seq([wait(1), wait(1)]);
        assert_eq!(Animation::duration(&a), 2.0);
        assert_eq!(Animation::duration(&a.duration(5)), 5.0);
    }

    #[test]
    fn the_all_helpers_take_any_collection_of_ids_and_have_sensible_timing() {
        use crate::api::shapes::cube;
        use crate::api::Stage;
        let mut s = Stage::new(100, 100);
        let ids: Vec<Id> = (0..4).map(|_| s.add(cube(5))).collect();
        let arr = [ids[0], ids[1], ids[2], ids[3]];
        // 4 items, each 1 s long, staggered: (4 - 1) * gap + 1.
        for w in [write_all(&ids), write_all(&arr), write_all(ids.iter()), write_all(ids.clone())] {
            assert!((Animation::duration(&w) - (3.0 * 0.25 + 1.0)).abs() < 1e-5);
        }
        assert!((Animation::duration(&create_all(&ids)) - (3.0 * 0.3 + 1.0)).abs() < 1e-5);
        assert!((Animation::duration(&clear(&ids)) - 0.7).abs() < 1e-5);
        assert!((Animation::duration(&clear(arr).duration(2)) - 2.0).abs() < 1e-5);
        // Empty lists are fine.
        let none: Vec<Id> = vec![];
        assert_eq!(Animation::duration(&clear(&none)), 0.0);
    }

    #[test]
    fn clear_really_erases_and_create_all_really_draws() {
        use crate::api::shapes::cube;
        use crate::api::Stage;
        let mut s = Stage::new(100, 100);
        let ids = [s.add(cube(5)), s.add(cube(5))];
        s.play(create_all(ids));
        s.play(clear(ids));
        let end = s.duration();
        s.seek(end);
        for id in ids {
            assert_eq!(s.scene.get(id).unwrap().style.reveal, 0.0, "erased");
        }
    }

    #[test]
    fn combinators_compute_lengths() {
        assert_eq!(Animation::duration(&all([wait(1), wait(3), wait(2)])), 3.0);
        assert_eq!(Animation::duration(&seq([wait(1), wait(3)])), 4.0);
        assert_eq!(Animation::duration(&lag(0.5, [wait(1), wait(1), wait(1)])), 2.0);
        assert_eq!(Animation::duration(&wait(1).delay(2)), 3.0);
        assert_eq!(Animation::duration(&wait(1).then(wait(2))), 3.0);
        assert_eq!(Animation::duration(&wait(1).with(wait(2))), 2.0);
    }
}
