use roughr::core::{FillStyle, Options, OptionsBuilder};

/// How hand-drawn everything looks. `Sketch::default()` is the classic whiteboard look;
/// `Sketch::clean()` gives straight, tidy lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sketch {
    /// How far lines wander from straight. 0 is ruler-straight; the default is 1.3.
    pub roughness: f32,
    /// How much lines bow in the middle.
    pub bowing: f32,
    pub stroke_width: f32,
    /// Distance between hatch lines in fills.
    pub hachure_gap: f32,
    /// Angle of the hatch lines in degrees.
    pub hachure_angle: f32,
}

impl Default for Sketch {
    fn default() -> Self {
        Self {
            roughness: 1.3,
            bowing: 1.2,
            stroke_width: 1.6,
            hachure_gap: 6.0,
            hachure_angle: -41.0,
        }
    }
}

impl Sketch {
    /// Straight lines, same hatching.
    pub fn clean() -> Self {
        Self {
            roughness: 0.0,
            bowing: 0.0,
            ..Self::default()
        }
    }

    pub fn roughness(mut self, v: f32) -> Self {
        self.roughness = v;
        self
    }

    pub fn hatch_gap(mut self, v: f32) -> Self {
        self.hachure_gap = v;
        self
    }

    pub(crate) fn options(&self) -> Options {
        OptionsBuilder::default()
            .stroke_width(self.stroke_width as _)
            .roughness(self.roughness as _)
            .bowing(self.bowing as _)
            .fill_style(FillStyle::Hachure)
            .hachure_angle(self.hachure_angle as _)
            .hachure_gap(self.hachure_gap as _)
            .build()
            .expect("valid rough options")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_has_no_wobble_and_builders_chain() {
        let c = Sketch::clean();
        assert_eq!((c.roughness, c.bowing), (0.0, 0.0));
        assert_eq!(Sketch::default().roughness(2.0).hatch_gap(9.0).hachure_gap, 9.0);
    }

    #[test]
    fn options_carry_the_settings() {
        let o = Sketch::default().roughness(2.0).options();
        assert_eq!(o.roughness, Some(2.0));
        assert_eq!(o.fill_style, Some(FillStyle::Hachure));
    }
}
