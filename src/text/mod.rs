//! Text as vector outlines: plain text from a font, and LaTeX through an external toolchain.
//!
//! Both produce a [`GlyphRun`]: one [`Region`] per glyph (letter, symbol or rule), so the
//! normal rough-drawing path can ink them and animations can write them letter by letter.

mod flatten;
pub mod handwriting;
pub mod font;
pub mod tex;

pub use font::{Align, Font, FontError};
pub use tex::{TexError, TexOptions};

/// Extra space after each letter, in ems, that [`Text`](crate::primitives::Text) adds by default so
/// handwriting-style glyphs stay distinct.
pub const DEFAULT_TRACKING: f32 = 0.06;

use crate::planar::{Point2, Region};

/// Positioned glyph outlines in "em" units (1.0 is the font size), y up, with the bounding box
/// centred on the origin.
#[derive(Clone, Debug, Default)]
pub struct GlyphRun {
    pub glyphs: Vec<Region>,
}

/// Combined `(min, max)` corners of all glyphs.
fn union_bounds(glyphs: &[Region]) -> Option<(Point2, Point2)> {
    glyphs
        .iter()
        .filter_map(Region::bounds)
        .reduce(|(alo, ahi), (lo, hi)| {
            (
                [alo[0].min(lo[0]), alo[1].min(lo[1])],
                [ahi[0].max(hi[0]), ahi[1].max(hi[1])],
            )
        })
}

impl GlyphRun {
    /// Centre the combined bounding box of `glyphs` on the origin.
    pub fn centered(mut glyphs: Vec<Region>) -> Self {
        glyphs.retain(|g| !g.is_empty());
        if let Some((lo, hi)) = union_bounds(&glyphs) {
            let (cx, cy) = ((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0);
            for g in &mut glyphs {
                *g = std::mem::take(g).translated(-cx, -cy);
            }
        }
        Self { glyphs }
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// `(width, height)` of the run in em units.
    pub fn size(&self) -> (f32, f32) {
        union_bounds(&self.glyphs).map_or((0.0, 0.0), |(lo, hi)| (hi[0] - lo[0], hi[1] - lo[1]))
    }
}
