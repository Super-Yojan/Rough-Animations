//! Keeps the agent skill honest: it must be well-formed, in sync with the cheat sheet, and every
//! script, example and error message it mentions must really exist.

use rough_animations::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const SKILL: &str = include_str!("../skills/rough-animations/SKILL.md");
const SHEET: &str = include_str!("../docs/cheatsheet.md");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn skill_dir() -> PathBuf {
    root().join("skills/rough-animations")
}

/// `(name, description)` from the YAML frontmatter.
fn frontmatter() -> (String, String) {
    let body = SKILL.strip_prefix("---\n").expect("SKILL.md starts with frontmatter");
    let end = body.find("\n---\n").expect("frontmatter is closed");
    let field = |key: &str| {
        body[..end]
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}:")))
            .unwrap_or_else(|| panic!("frontmatter has no `{key}`"))
            .trim()
            .to_string()
    };
    (field("name"), field("description"))
}

#[test]
fn the_frontmatter_is_valid() {
    let (name, description) = frontmatter();
    assert_eq!(name, "rough-animations");
    assert_eq!(
        name,
        skill_dir().file_name().unwrap().to_str().unwrap(),
        "the name must match the folder"
    );
    assert!(name.len() <= 64 && name.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
    assert!(
        (100..=1024).contains(&description.len()),
        "description is {} characters",
        description.len()
    );
    // It says what the skill does *and* when to reach for it.
    assert!(description.contains("Use when"), "{description}");
    for word in ["animation", "mp4", "LaTeX"] {
        assert!(description.contains(word), "the description should mention {word:?}");
    }
}

#[test]
fn the_api_section_is_exactly_the_cheat_sheet() {
    let start = SKILL.find("<!-- cheatsheet:start -->").expect("start marker");
    let end = SKILL.find("<!-- cheatsheet:end -->").expect("end marker");
    let embedded = SKILL[start + "<!-- cheatsheet:start -->".len()..end].trim();
    assert_eq!(
        embedded,
        SHEET.trim(),
        "the skill's API section is out of date: run scripts/sync-skill.sh"
    );
}

#[test]
fn every_script_it_mentions_exists_and_is_executable() {
    let mut mentioned = Vec::new();
    for word in SKILL.split(|c: char| c.is_whitespace() || c == '`' || c == '(' || c == ')') {
        if let Some(name) = word.rsplit('/').next().filter(|n| n.ends_with(".sh")) {
            mentioned.push(name.to_string());
        }
    }
    assert!(mentioned.len() >= 2, "expected the helper scripts to be named, got {mentioned:?}");
    for name in mentioned {
        let found = [skill_dir().join("scripts").join(&name), root().join("scripts").join(&name)]
            .into_iter()
            .find(|p| p.exists())
            .unwrap_or_else(|| panic!("the skill mentions {name}, which does not exist"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&found).unwrap().permissions().mode();
            assert!(mode & 0o111 != 0, "{} is not executable", found.display());
        }
    }
}

#[test]
fn every_example_it_names_exists() {
    let mut names = Vec::new();
    let mut words = SKILL.split(|c: char| c.is_whitespace() || c == '`' || c == ',' || c == '(');
    while let Some(w) = words.next() {
        if w == "--example" {
            names.push(words.next().unwrap_or("").to_string());
        }
        if let Some(rest) = w.strip_prefix("examples/") {
            let name = rest.trim_end_matches(|c: char| c == '.' || c == ':' || c == ')');
            names.push(name.trim_end_matches(".rs").to_string());
        }
    }
    for name in names.iter().filter(|n| !n.starts_with('<') && !n.is_empty()) {
        assert!(
            root().join("examples").join(format!("{name}.rs")).exists(),
            "the skill names examples/{name}.rs, which does not exist"
        );
    }
    assert!(names.len() >= 4);
}

/// Every `.rs` file under `src`, concatenated.
fn all_source() -> String {
    fn walk(dir: &Path, out: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push_str(&std::fs::read_to_string(path).unwrap());
            }
        }
    }
    let mut out = String::new();
    walk(&root().join("src"), &mut out);
    out
}

#[test]
fn the_error_messages_in_the_troubleshooting_table_are_real() {
    let source = all_source();
    for message in [
        "unknown colour",
        "on(): only flat shapes",
        "boolean operations need flat shapes",
        "has already played",
        "LaTeX error",
        "not found: install ffmpeg",
    ] {
        assert!(source.contains(message), "no source mentions {message:?}");
        // And the skill's table documents the ones an agent would search for.
    }
    for quoted in ["unknown colour", "on(): only flat shapes", "boolean operations need flat shapes", "has already played"] {
        assert!(SKILL.contains(quoted), "the skill should mention {quoted:?}");
    }
    // The facts the design guidance relies on.
    assert!(source.contains("Home::Title"), "title() has a default home");
}

fn have(tool: &str) -> bool {
    Command::new(tool).arg("-version").output().is_ok()
}

#[test]
fn the_helper_scripts_really_work_on_a_real_video() {
    if !have("ffmpeg") || !have("ffprobe") {
        eprintln!("skipping: ffmpeg not installed");
        return;
    }
    let dir = std::env::temp_dir().join(format!("skill-scripts-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // A real video made with the library: 2 seconds of a cube drawing itself.
    let mut s = Stage::new(160, 120).fps(10);
    let c = s.add(cube(30).base_at((0, 0, 0)).fill("purple"));
    s.play(create(c).duration(2));
    let video = dir.join("clip.mp4");
    s.render(&video).unwrap();

    let scripts = skill_dir().join("scripts");
    let run = |script: &str, args: &[&str]| {
        Command::new(scripts.join(script)).args(args).output().unwrap()
    };

    // frame.sh: a positive time and a negative one (counted from the end).
    let at = dir.join("at1.png");
    let out = run("frame.sh", &[video.to_str().unwrap(), "1.0", at.to_str().unwrap()]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(std::fs::metadata(&at).unwrap().len() > 100);
    let end = dir.join("end.png");
    let out = run("frame.sh", &[video.to_str().unwrap(), "-0.2", end.to_str().unwrap()]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(std::fs::metadata(&end).unwrap().len() > 100);
    assert_ne!(
        std::fs::read(&at).unwrap(),
        std::fs::read(&end).unwrap(),
        "the middle and the end of the animation differ"
    );

    // contact-sheet.sh: several frames in one image.
    let sheet = dir.join("sheet.png");
    let out = run(
        "contact-sheet.sh",
        &[video.to_str().unwrap(), "0.5", "4", sheet.to_str().unwrap(), "120"],
    );
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let img = tiny_skia::Pixmap::load_png(&sheet).unwrap();
    assert_eq!(img.width(), 4 * 120, "four columns of 120 px");
    assert!(img.height() > 90, "at least one row of frames");

    // Failures are explained, with a non-zero exit.
    for (script, args) in [
        ("frame.sh", vec!["no-such-file.mp4", "1"]),
        ("frame.sh", vec![video.to_str().unwrap(), "99"]),
        ("contact-sheet.sh", vec!["no-such-file.mp4"]),
        ("frame.sh", vec![]),
    ] {
        let out = run(script, &args);
        assert!(!out.status.success(), "{script} {args:?} should fail");
        assert!(!out.stderr.is_empty(), "{script} {args:?} should say why");
    }
    let _ = std::fs::remove_dir_all(dir);
}
