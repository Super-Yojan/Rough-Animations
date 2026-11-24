use super::GlyphRun;
use super::flatten::Flattener;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use ttf_parser::{Face, OutlineBuilder};

/// The bundled font: Patrick Hand, a friendly handwriting face (SIL Open Font License, see
/// `assets/fonts/OFL.txt`).
const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/PatrickHand-Regular.ttf");

#[derive(Debug, Clone)]
pub struct FontError(String);

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "font error: {}", self.0)
    }
}

impl std::error::Error for FontError {}

/// How lines of a multi-line text line up with each other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

/// A TrueType / OpenType font, cheap to clone.
#[derive(Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
}

impl Font {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, FontError> {
        Face::parse(&data, 0).map_err(|e| FontError(e.to_string()))?;
        Ok(Self {
            data: Arc::new(data),
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FontError> {
        let data = std::fs::read(path.as_ref()).map_err(|e| FontError(e.to_string()))?;
        Self::from_bytes(data)
    }

    /// The bundled hand-drawn style font.
    pub fn default_font() -> Self {
        static FONT: OnceLock<Font> = OnceLock::new();
        FONT.get_or_init(|| Font::from_bytes(DEFAULT_FONT.to_vec()).expect("bundled font is valid"))
            .clone()
    }

    /// Lay out `text` (lines separated by `\\n`) as glyph outlines, in em units, centred on
    /// the origin. `line_spacing` scales the font's natural line height (1.0 = normal).
    pub fn layout(&self, text: &str, align: Align, line_spacing: f32) -> GlyphRun {
        let face = Face::parse(&self.data, 0).expect("validated on construction");
        let upem = face.units_per_em() as f32;
        let line_height =
            (face.ascender() as f32 - face.descender() as f32 + face.line_gap() as f32) / upem
                * line_spacing;

        let advance = |ch: char| {
            face.glyph_index(ch)
                .and_then(|g| face.glyph_hor_advance(g))
                .unwrap_or(0) as f32
                / upem
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let widths: Vec<f32> = lines
            .iter()
            .map(|l| l.chars().map(&advance).sum::<f32>())
            .collect();
        let widest = widths.iter().copied().fold(0.0, f32::max);

        let mut glyphs = Vec::new();
        for (row, line) in lines.iter().enumerate() {
            let mut x = match align {
                Align::Left => 0.0,
                Align::Center => (widest - widths[row]) / 2.0,
                Align::Right => widest - widths[row],
            };
            let y = -(row as f32) * line_height;
            for ch in line.chars() {
                if let Some(id) = face.glyph_index(ch) {
                    let mut b = GlyphBuilder {
                        flat: Flattener::new(0.04),
                        upem,
                        dx: x,
                        dy: y,
                    };
                    if face.outline_glyph(id, &mut b).is_some() {
                        glyphs.push(b.flat.into_region());
                    }
                }
                x += advance(ch);
            }
        }
        GlyphRun::centered(glyphs)
    }
}

struct GlyphBuilder {
    flat: Flattener,
    upem: f32,
    dx: f32,
    dy: f32,
}

impl GlyphBuilder {
    fn p(&self, x: f32, y: f32) -> [f32; 2] {
        [x / self.upem + self.dx, y / self.upem + self.dy]
    }
}

impl OutlineBuilder for GlyphBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.flat.move_to(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.flat.line_to(p);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.p(x1, y1), self.p(x, y));
        self.flat.quad_to(c, p);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        self.flat.cubic_to(c1, c2, p);
    }
    fn close(&mut self) {
        self.flat.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> Font {
        Font::default_font()
    }

    #[test]
    fn one_region_per_visible_glyph_and_spaces_are_skipped() {
        let run = font().layout("A b", Align::Center, 1.0);
        assert_eq!(run.glyphs.len(), 2);
        assert!(font().layout("   ", Align::Center, 1.0).is_empty());
        assert!(font().layout("", Align::Center, 1.0).is_empty());
    }

    #[test]
    fn counters_become_holes() {
        let run = font().layout("o", Align::Center, 1.0);
        assert_eq!(run.glyphs.len(), 1);
        assert_eq!(run.glyphs[0].contours.len(), 2, "outer + counter");
        // The glyph is centred on the origin, so the origin is inside the counter (not inked),
        // while a point just inside the left edge is on the ring.
        let g = &run.glyphs[0];
        assert!(!g.contains([0.0, 0.0]));
        let left = g.bounds().unwrap().0[0];
        assert!(g.contains([left + 0.02, 0.0]));
    }

    #[test]
    fn output_is_centred_and_scales_with_text_length() {
        let short = font().layout("ab", Align::Center, 1.0);
        let long = font().layout("abcdefgh", Align::Center, 1.0);
        for run in [&short, &long] {
            let (w, h) = run.size();
            assert!(w > 0.0 && h > 0.0);
            let lo = run.glyphs.iter().filter_map(|g| g.bounds()).map(|b| b.0[0]).fold(f32::MAX, f32::min);
            let hi = run.glyphs.iter().filter_map(|g| g.bounds()).map(|b| b.1[0]).fold(f32::MIN, f32::max);
            assert!((lo + hi).abs() < 1e-4, "centred horizontally");
        }
        assert!(long.size().0 > 3.0 * short.size().0);
    }

    #[test]
    fn lines_stack_downwards_and_alignment_shifts_them() {
        let one = font().layout("hello", Align::Left, 1.0);
        let two = font().layout("hello\nhello", Align::Left, 1.0);
        assert!(two.size().1 > 1.5 * one.size().1);
        assert_eq!(two.glyphs.len(), 10);

        let left = font().layout("wide line\ni", Align::Left, 1.0);
        let right = font().layout("wide line\ni", Align::Right, 1.0);
        // The lone "i" (last glyph) sits at opposite ends of the block.
        let x = |run: &GlyphRun| run.glyphs.last().unwrap().bounds().unwrap().0[0];
        assert!(x(&right) > x(&left) + 0.5);
    }

    #[test]
    fn bad_font_data_is_an_error() {
        assert!(Font::from_bytes(vec![1, 2, 3]).is_err());
        assert!(Font::from_file("/definitely/not/a/font.ttf").is_err());
    }
}
