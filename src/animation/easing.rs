use std::f32::consts::PI;

/// Maps linear progress `t` in `[0, 1]` to eased progress (0 -> 0, 1 -> 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Easing {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    /// Smooth start and stop. The default, like manim's `smooth`.
    #[default]
    CubicInOut,
    SineInOut,
    /// Overshoots slightly, then settles.
    BackOut,
    BounceOut,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::QuadIn => t * t,
            Easing::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::CubicIn => t * t * t,
            Easing::CubicOut => 1.0 - (1.0 - t).powi(3),
            Easing::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::SineInOut => -((PI * t).cos() - 1.0) / 2.0,
            Easing::BackOut => {
                let (c1, c3) = (1.70158, 2.70158);
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Easing::BounceOut => {
                let (n1, d1) = (7.5625, 2.75);
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Easing; 10] = [
        Easing::Linear,
        Easing::QuadIn,
        Easing::QuadOut,
        Easing::QuadInOut,
        Easing::CubicIn,
        Easing::CubicOut,
        Easing::CubicInOut,
        Easing::SineInOut,
        Easing::BackOut,
        Easing::BounceOut,
    ];

    #[test]
    fn endpoints_are_fixed() {
        for e in ALL {
            assert!(e.apply(0.0).abs() < 1e-5, "{e:?} at 0");
            assert!((e.apply(1.0) - 1.0).abs() < 1e-5, "{e:?} at 1");
        }
    }

    #[test]
    fn input_is_clamped() {
        assert_eq!(Easing::Linear.apply(-3.0), 0.0);
        assert_eq!(Easing::Linear.apply(3.0), 1.0);
    }

    #[test]
    fn in_out_is_symmetric_at_midpoint() {
        for e in [Easing::QuadInOut, Easing::CubicInOut, Easing::SineInOut] {
            assert!((e.apply(0.5) - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn back_out_overshoots() {
        assert!(Easing::BackOut.apply(0.7) > 1.0);
    }
}
