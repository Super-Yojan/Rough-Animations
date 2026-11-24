use super::{Anim, Animation};
use crate::api::Num;
use crate::world::arena::Id;
use crate::world::scene::Scene;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Pending,
    Running,
    Done,
}

struct Item {
    start: f32,
    anim: Box<dyn Animation>,
    state: State,
}

/// Animations placed at start offsets on a shared clock. [`seq`](super::seq), [`all`](super::all) and
/// [`lag`](super::lag) are all just groups with different offsets.
#[derive(Default)]
pub struct Schedule {
    items: Vec<Item>,
    duration: f32,
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an animation that starts `start` seconds into the group.
    pub fn push_at(&mut self, start: f32, anim: impl Animation + 'static) {
        let start = start.max(0.0);
        self.duration = self.duration.max(start + anim.duration());
        self.items.push(Item {
            start,
            anim: Box::new(anim),
            state: State::Pending,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Animation for Schedule {
    fn duration(&self) -> f32 {
        self.duration
    }

    fn introduces(&self, out: &mut Vec<Id>) {
        for item in &self.items {
            item.anim.introduces(out);
        }
    }

    fn consumes(&self, out: &mut Vec<Id>) {
        for item in &self.items {
            item.anim.consumes(out);
        }
    }

    fn update(&mut self, scene: &mut Scene, time: f32) {
        for item in &mut self.items {
            if item.state == State::Done || time < item.start {
                continue;
            }
            if item.state == State::Pending {
                item.anim.begin(scene);
                item.state = State::Running;
            }
            let dur = item.anim.duration();
            let local = (time - item.start).min(dur);
            item.anim.update(scene, local);
            if local >= dur {
                item.state = State::Done;
            }
        }
    }
}

/// Do nothing for `seconds`.
pub fn wait(seconds: impl Num) -> Anim {
    Anim::new(Wait(seconds.f32().max(0.0)))
}

pub struct Wait(f32);

impl Animation for Wait {
    fn duration(&self) -> f32 {
        self.0
    }
    fn update(&mut self, _scene: &mut Scene, _time: f32) {}
}
