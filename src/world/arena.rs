/// Stable handle to an object stored in an [`Arena`].
///
/// Handles carry a generation, so a handle to a removed object never aliases
/// a later object that reuses its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// Generational arena. Animations hold `Id`s instead of references, so they
/// need no lifetimes and can mutate the scene freely.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: T) -> Id {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            Id {
                index,
                generation: slot.generation,
            }
        } else {
            self.slots.push(Slot {
                generation: 0,
                value: Some(value),
            });
            Id {
                index: (self.slots.len() - 1) as u32,
                generation: 0,
            }
        }
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation += 1;
        self.free.push(id.index);
        Some(value)
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation == id.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().filter_map(|s| s.value.as_mut())
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.value.as_ref().map(|v| {
                (
                    Id {
                        index: i as u32,
                        generation: s.generation,
                    },
                    v,
                )
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_remove() {
        let mut a = Arena::new();
        let id = a.insert("x");
        assert_eq!(a.get(id), Some(&"x"));
        assert_eq!(a.remove(id), Some("x"));
        assert_eq!(a.get(id), None);
        assert!(a.is_empty());
    }

    #[test]
    fn stale_handle_does_not_alias_reused_slot() {
        let mut a = Arena::new();
        let old = a.insert(1);
        a.remove(old);
        let new = a.insert(2);
        assert_eq!(a.get(old), None);
        assert_eq!(a.get(new), Some(&2));
        assert_eq!(a.remove(old), None);
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn iter_skips_removed() {
        let mut a = Arena::new();
        let x = a.insert(1);
        a.insert(2);
        a.remove(x);
        assert_eq!(a.iter().map(|(_, v)| *v).collect::<Vec<_>>(), vec![2]);
    }
}
