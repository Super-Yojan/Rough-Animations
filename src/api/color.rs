use tiny_skia::Color;

/// Named colours, matching the palette used across the examples.
pub const NAMES: &[(&str, (u8, u8, u8))] = &[
    ("purple", (168, 130, 255)),
    ("mint", (115, 210, 165)),
    ("amber", (255, 185, 110)),
    ("sky", (130, 195, 255)),
    ("rose", (255, 120, 140)),
    ("gold", (250, 215, 90)),
    ("slate", (225, 225, 232)),
    ("ink", (35, 35, 40)),
    ("indigo", (60, 50, 120)),
    ("stone", (106, 106, 128)),
    ("red", (220, 60, 60)),
    ("green", (80, 180, 100)),
    ("blue", (70, 120, 220)),
    ("orange", (245, 150, 60)),
    ("yellow", (245, 215, 70)),
    ("pink", (245, 150, 190)),
    ("teal", (60, 170, 170)),
    ("brown", (150, 100, 70)),
    ("gray", (150, 150, 158)),
    ("grey", (150, 150, 158)),
    ("black", (0, 0, 0)),
    ("white", (255, 255, 255)),
];

/// Parse `"#rrggbb"`, `"#rgb"` or a name from [`NAMES`] (case-insensitive).
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let byte = |i: usize, n: usize| u8::from_str_radix(hex.get(i..i + n)?, 16).ok();
        return match hex.len() {
            6 => Some(Color::from_rgba8(byte(0, 2)?, byte(2, 2)?, byte(4, 2)?, 255)),
            3 => Some(Color::from_rgba8(byte(0, 1)? * 17, byte(1, 1)? * 17, byte(2, 1)? * 17, 255)),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    NAMES
        .iter()
        .find(|(n, _)| *n == lower)
        .map(|&(_, (r, g, b))| Color::from_rgba8(r, g, b, 255))
}

/// Anything that can be a colour: a name or hex string (`"purple"`, `"#a882ff"`), an
/// `(r, g, b)` tuple of 0-255 values, or a `tiny_skia::Color`.
///
/// An unknown name panics with the list of valid ones, so a typo fails loudly and immediately.
pub trait IntoColor {
    fn into_color(self) -> Color;
}

impl IntoColor for Color {
    fn into_color(self) -> Color {
        self
    }
}

impl IntoColor for &str {
    fn into_color(self) -> Color {
        parse_color(self).unwrap_or_else(|| {
            let names: Vec<_> = NAMES.iter().map(|(n, _)| *n).collect();
            panic!(
                "unknown colour {self:?}: use \"#rrggbb\", (r, g, b), or one of: {}",
                names.join(", ")
            )
        })
    }
}

impl IntoColor for String {
    fn into_color(self) -> Color {
        self.as_str().into_color()
    }
}

impl IntoColor for (u8, u8, u8) {
    fn into_color(self) -> Color {
        Color::from_rgba8(self.0, self.1, self.2, 255)
    }
}

impl IntoColor for [u8; 3] {
    fn into_color(self) -> Color {
        (self[0], self[1], self[2]).into_color()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_hex_and_tuples() {
        assert_eq!("purple".into_color(), Color::from_rgba8(168, 130, 255, 255));
        assert_eq!("PURPLE".into_color(), "purple".into_color());
        assert_eq!("#a882ff".into_color(), "purple".into_color());
        assert_eq!("#fff".into_color(), Color::from_rgba8(255, 255, 255, 255));
        assert_eq!((10, 20, 30).into_color(), Color::from_rgba8(10, 20, 30, 255));
        assert_eq!([10u8, 20, 30].into_color(), (10, 20, 30).into_color());
    }

    #[test]
    fn bad_input_is_rejected() {
        assert!(parse_color("purpel").is_none());
        assert!(parse_color("#12").is_none());
        assert!(parse_color("#gggggg").is_none());
        assert!(parse_color("").is_none());
    }

    #[test]
    #[should_panic(expected = "unknown colour \"purpel\"")]
    fn a_typo_panics_with_the_valid_names() {
        let _ = "purpel".into_color();
    }
}
