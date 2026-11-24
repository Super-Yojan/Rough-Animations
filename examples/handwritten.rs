//! Typeset LaTeX next to the same formulas made `.handwritten()`.
//! Writes `target/handwritten.png` (needs a LaTeX install).

use rough_animations::prelude::*;
fn main() -> Result {
    let mut s = Stage::new(1200, 820);
    let f1 = r"\int_0^1 x^2\,dx = \frac{1}{3}";
    let f2 = r"e^{i\pi} + 1 = 0";
    let f4 = r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}";
    s.add(text("typeset").font_size(30).color("gray").at(s.pixel(300, 35)));
    s.add(text("handwritten").font_size(30).color("gray").at(s.pixel(900, 35)));
    for (i, f) in [f1, f2, f4].iter().enumerate() {
        let y = 130 + i as u32 * 165;
        s.add(math(f)?.font_size(46).color("ink").at(s.pixel(300, y)));
        s.add(math(f)?.font_size(46).color("ink").handwritten().at(s.pixel(900, y)));
    }
    // And small, where readability matters most.
    for (i, f) in [f1, f4].iter().enumerate() {
        let y = 640 + i as u32 * 80;
        s.add(math(f)?.font_size(26).color("ink").at(s.pixel(300, y)));
        s.add(math(f)?.font_size(26).color("ink").handwritten().at(s.pixel(900, y)));
    }
    s.render("target/handwritten.png")
}
