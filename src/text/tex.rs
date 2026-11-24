//! LaTeX to glyph outlines.
//!
//! Source is compiled with `pdflatex`, the PDF is converted to SVG by `pdftocairo` (poppler)
//! or `mutool` (MuPDF), and the SVG glyph paths become [`Region`]s. Results are cached on disk,
//! so each distinct formula is compiled once.

use super::GlyphRun;
use super::flatten::Flattener;
use crate::planar::Region;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

/// The document is typeset at this size; output is divided by it so 1.0 equals one em.
const POINT_SIZE: f32 = 12.0;

/// Bumped whenever the document template or conversion changes, to invalidate the cache.
const CACHE_VERSION: u32 = 1;

#[derive(Debug)]
pub enum TexError {
    /// A required program isn't installed.
    ToolNotFound { tool: String, hint: String },
    /// LaTeX rejected the source; `message` is the first error from its log.
    Compile { message: String },
    /// The PDF to SVG conversion failed.
    Convert(String),
    /// The SVG could not be read.
    Svg(String),
    /// The source produced no visible output.
    Empty,
    Io(std::io::Error),
}

impl fmt::Display for TexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TexError::ToolNotFound { tool, hint } => write!(f, "`{tool}` not found: {hint}"),
            TexError::Compile { message } => write!(f, "LaTeX error: {message}"),
            TexError::Convert(m) => write!(f, "PDF to SVG conversion failed: {m}"),
            TexError::Svg(m) => write!(f, "could not read the SVG: {m}"),
            TexError::Empty => write!(f, "the LaTeX source produced no visible output"),
            TexError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TexError {}

impl From<std::io::Error> for TexError {
    fn from(e: std::io::Error) -> Self {
        TexError::Io(e)
    }
}

/// How LaTeX is run.
#[derive(Clone, Debug)]
pub struct TexOptions {
    /// LaTeX executable (`pdflatex`, `xelatex`, ...). Must produce a PDF.
    pub latex: String,
    /// Extra lines for the document preamble, e.g. `\usepackage{mathrsfs}`.
    pub preamble: String,
    /// Reuse compiled results from the on-disk cache.
    pub cache: bool,
}

impl Default for TexOptions {
    fn default() -> Self {
        Self {
            latex: "pdflatex".into(),
            preamble: String::new(),
            cache: true,
        }
    }
}

/// Typeset `body` (the contents of the document) into glyph outlines.
pub fn typeset(body: &str, opts: &TexOptions) -> Result<GlyphRun, TexError> {
    let doc = format!(
        "\\documentclass[{pt}pt]{{article}}\n\\usepackage{{amsmath,amssymb}}\n{pre}\n\\pagestyle{{empty}}\n\\begin{{document}}\n{body}\n\\end{{document}}\n",
        pt = POINT_SIZE as u32,
        pre = opts.preamble,
    );

    let cache_dir = std::env::temp_dir().join("rough-animations-tex");
    std::fs::create_dir_all(&cache_dir)?;
    let mut h = DefaultHasher::new();
    (CACHE_VERSION, &doc, &opts.latex).hash(&mut h);
    let cached = cache_dir.join(format!("{:016x}.svg", h.finish()));

    if !(opts.cache && cached.exists()) {
        compile_to_svg(&doc, opts, &cache_dir, &cached)?;
    }
    let svg = std::fs::read_to_string(&cached)?;
    let run = svg_to_glyphs(&svg)?;
    if run.is_empty() {
        return Err(TexError::Empty);
    }
    Ok(run)
}

fn compile_to_svg(
    doc: &str,
    opts: &TexOptions,
    cache_dir: &Path,
    out: &Path,
) -> Result<(), TexError> {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    // A private work directory per call, so parallel compiles never collide.
    let work = cache_dir.join(format!(
        "work-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&work)?;
    let result = (|| {
        std::fs::write(work.join("doc.tex"), doc)?;
        let output = Command::new(&opts.latex)
            .current_dir(&work)
            .args(["-interaction=nonstopmode", "-halt-on-error", "-no-shell-escape", "doc.tex"])
            .output()
            .map_err(|e| not_found(e, &opts.latex, "install a TeX distribution (e.g. TeX Live)"))?;
        if !output.status.success() {
            let log = std::fs::read_to_string(work.join("doc.log")).unwrap_or_default();
            return Err(TexError::Compile {
                message: first_latex_error(&log),
            });
        }
        pdf_to_svg(&work.join("doc.pdf"), &work.join("doc.svg"))?;
        // Rename is atomic, so a concurrent reader never sees a half-written file.
        std::fs::rename(work.join("doc.svg"), out)?;
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&work);
    result
}

fn not_found(e: std::io::Error, tool: &str, hint: &str) -> TexError {
    if e.kind() == std::io::ErrorKind::NotFound {
        TexError::ToolNotFound {
            tool: tool.into(),
            hint: hint.into(),
        }
    } else {
        TexError::Io(e)
    }
}

/// The first `! ...` error line (plus the line after it, which names the culprit).
fn first_latex_error(log: &str) -> String {
    let mut lines = log.lines();
    while let Some(l) = lines.next() {
        if let Some(msg) = l.strip_prefix("! ") {
            let ctx = lines.next().unwrap_or("").trim();
            return if ctx.is_empty() {
                msg.to_string()
            } else {
                format!("{msg} ({ctx})")
            };
        }
    }
    "compilation failed (see the LaTeX log)".into()
}

/// Convert with whichever converter is installed.
fn pdf_to_svg(pdf: &Path, svg: &Path) -> Result<(), TexError> {
    let attempts: [(&str, Vec<&std::ffi::OsStr>); 2] = [
        (
            "pdftocairo",
            vec!["-svg".as_ref(), pdf.as_os_str(), svg.as_os_str()],
        ),
        (
            "mutool",
            vec![
                "draw".as_ref(),
                "-F".as_ref(),
                "svg".as_ref(),
                "-O".as_ref(),
                "text=path".as_ref(),
                "-o".as_ref(),
                svg.as_os_str(),
                pdf.as_os_str(),
            ],
        ),
    ];
    for (tool, args) in attempts {
        match Command::new(tool).args(&args).output() {
            Ok(o) if o.status.success() => {
                // mutool numbers its pages: `out.svg` becomes `out1.svg`.
                if !svg.exists() {
                    if let Some(numbered) = numbered_page(svg) {
                        std::fs::rename(numbered, svg)?;
                    }
                }
                return Ok(());
            }
            Ok(o) => {
                return Err(TexError::Convert(format!(
                    "{tool}: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                )));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(TexError::ToolNotFound {
        tool: "pdftocairo or mutool".into(),
        hint: "install poppler (`brew install poppler`) or mupdf to convert LaTeX output".into(),
    })
}

fn numbered_page(svg: &Path) -> Option<PathBuf> {
    let stem = svg.file_stem()?.to_str()?;
    let p = svg.with_file_name(format!("{stem}1.svg"));
    p.exists().then_some(p)
}

/// Read filled paths from an SVG as glyph regions: y flipped to point up, scaled so one em is
/// 1.0, centred on the origin.
pub fn svg_to_glyphs(svg: &str) -> Result<GlyphRun, TexError> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|e| TexError::Svg(e.to_string()))?;
    let mut regions = Vec::new();
    collect(tree.root(), &mut regions);
    let regions = regions
        .into_iter()
        .map(|r| r.map_points(|[x, y]| [x / POINT_SIZE, -y / POINT_SIZE]))
        .collect();
    Ok(GlyphRun::centered(regions))
}

fn collect(group: &usvg::Group, out: &mut Vec<Region>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => collect(g, out),
            usvg::Node::Path(path) => {
                let ts = path.abs_transform();
                if path.fill().is_some() {
                    out.extend(flatten(path.data(), ts));
                }
                // Fraction bars, radical tops and other rules are stroked lines in the PDF:
                // turn them into outlines so they become glyphs too.
                if let Some(stroke) = path.stroke() {
                    if let Some(outline) = path.data().stroke(&stroke.to_tiny_skia(), 1.0) {
                        out.extend(flatten(&outline, ts));
                    }
                }
            }
            _ => {}
        }
    }
}

fn flatten(
    path: &usvg::tiny_skia_path::Path,
    ts: usvg::tiny_skia_path::Transform,
) -> Option<Region> {
    use usvg::tiny_skia_path::{PathSegment, Point};
    let map = |p: Point| {
        let mut pts = [p];
        ts.map_points(&mut pts);
        [pts[0].x, pts[0].y]
    };
    let mut flat = Flattener::new(0.6);
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => flat.move_to(map(p)),
            PathSegment::LineTo(p) => flat.line_to(map(p)),
            PathSegment::QuadTo(c, p) => flat.quad_to(map(c), map(p)),
            PathSegment::CubicTo(c1, c2, p) => flat.cubic_to(map(c1), map(c2), map(p)),
            PathSegment::Close => flat.close(),
        }
    }
    let region = flat.into_region();
    (!region.is_empty()).then_some(region)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_the_first_latex_error_with_context() {
        let log = "This is pdfTeX\n! Undefined control sequence.\nl.7 $\\foo\n x\n";
        assert_eq!(
            first_latex_error(log),
            "Undefined control sequence. (l.7 $\\foo)"
        );
        assert!(first_latex_error("nothing here").contains("failed"));
    }

    #[test]
    fn reads_svg_paths_flipped_scaled_and_centred() {
        // A 12x24 filled rectangle (1 em wide, 2 em tall), off to one side of the page.
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
            <path d="M40 10 L52 10 L52 34 L40 34 Z" fill="black"/>
        </svg>"#;
        let run = svg_to_glyphs(svg).unwrap();
        assert_eq!(run.glyphs.len(), 1);
        let (w, h) = run.size();
        assert!((w - 1.0).abs() < 1e-4 && (h - 2.0).abs() < 1e-4, "{w}x{h}");
        let (lo, hi) = run.glyphs[0].bounds().unwrap();
        assert!((lo[0] + hi[0]).abs() < 1e-4 && (lo[1] + hi[1]).abs() < 1e-4);
    }

    #[test]
    fn transforms_and_unfilled_paths_are_handled() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
            <g transform="translate(10 10) scale(2)">
              <path d="M0 0 L6 0 L6 6 L0 6 Z" fill="black"/>
            </g>
            <path d="M0 0 L50 0" fill="none"/>
        </svg>"#;
        let run = svg_to_glyphs(svg).unwrap();
        assert_eq!(run.glyphs.len(), 1, "an invisible path is ignored");
        let (w, _) = run.size();
        assert!((w - 1.0).abs() < 1e-4, "6 * 2 = 12 units = 1 em, got {w}");
    }

    #[test]
    fn stroked_lines_become_outlines() {
        // A rule drawn as a stroked line: 12 long, 2 thick -> 1 em by 1/6 em.
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
            <path d="M10 50 L22 50" fill="none" stroke="black" stroke-width="2"/>
        </svg>"#;
        let run = svg_to_glyphs(svg).unwrap();
        assert_eq!(run.glyphs.len(), 1);
        let (w, h) = run.size();
        assert!((w - 1.0).abs() < 1e-3 && (h - 2.0 / 12.0).abs() < 1e-3, "{w}x{h}");
    }

    #[test]
    fn unreadable_svg_is_an_error() {
        assert!(matches!(svg_to_glyphs("not svg"), Err(TexError::Svg(_))));
    }

    fn have_toolchain() -> bool {
        let ok = |t: &str, a: &str| Command::new(t).arg(a).output().is_ok();
        ok("pdflatex", "--version") && (ok("pdftocairo", "-v") || ok("mutool", "-v"))
    }

    #[test]
    fn compiles_real_latex_when_installed() {
        if !have_toolchain() {
            eprintln!("skipping: no LaTeX toolchain");
            return;
        }
        let opts = TexOptions::default();
        let x2 = typeset("\\begin{align*} x^2 \\end{align*}", &opts).unwrap();
        assert_eq!(x2.glyphs.len(), 2, "x and the superscript 2");
        // A fraction is numerator, denominator and the bar.
        let frac = typeset("\\begin{align*} \\frac{1}{2} \\end{align*}", &opts).unwrap();
        assert_eq!(frac.glyphs.len(), 3);
        // The same source again comes from the cache and is identical.
        let again = typeset("\\begin{align*} \\frac{1}{2} \\end{align*}", &opts).unwrap();
        assert_eq!(again.glyphs, frac.glyphs);
        // Output is normalised: about one em tall for a single letter, centred.
        let (_, h) = x2.size();
        assert!(h > 0.3 && h < 1.5, "{h}");
    }

    #[test]
    fn latex_errors_are_reported() {
        if !have_toolchain() {
            return;
        }
        let err = typeset("$\\definitelynotamacro$", &TexOptions::default()).unwrap_err();
        match err {
            TexError::Compile { message } => assert!(message.contains("Undefined control sequence"), "{message}"),
            other => panic!("expected a compile error, got {other}"),
        }
    }

    #[test]
    fn missing_latex_is_a_helpful_error() {
        let opts = TexOptions {
            latex: "definitely-not-latex-xyz".into(),
            cache: false,
            ..TexOptions::default()
        };
        match typeset("x", &opts) {
            Err(TexError::ToolNotFound { tool, .. }) => assert_eq!(tool, "definitely-not-latex-xyz"),
            other => panic!("expected ToolNotFound, got {other:?}"),
        }
    }
}
