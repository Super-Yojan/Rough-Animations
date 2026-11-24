use super::{Animation, Easing, Group};
use crate::api::Num;
use crate::world::arena::Id;
use crate::world::scene::Scene;

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
    let mut g = Group::new();
    for a in anims {
        g.push_at(0.0, a);
    }
    Anim::new(g)
}

/// Run one after another.
pub fn seq(anims: impl IntoIterator<Item = Anim>) -> Anim {
    let mut g = Group::new();
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
    let mut g = Group::new();
    for (i, a) in anims.into_iter().enumerate() {
        g.push_at(gap * i as f32, a);
    }
    Anim::new(g)
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
    fn combinators_compute_lengths() {
        assert_eq!(Animation::duration(&all([wait(1), wait(3), wait(2)])), 3.0);
        assert_eq!(Animation::duration(&seq([wait(1), wait(3)])), 4.0);
        assert_eq!(Animation::duration(&lag(0.5, [wait(1), wait(1), wait(1)])), 2.0);
        assert_eq!(Animation::duration(&wait(1).delay(2)), 3.0);
        assert_eq!(Animation::duration(&wait(1).then(wait(2))), 3.0);
        assert_eq!(Animation::duration(&wait(1).with(wait(2))), 2.0);
    }
}
