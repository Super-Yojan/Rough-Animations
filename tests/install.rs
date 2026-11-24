//! The universal installer (`scripts/install-skill.sh`), exercised for real in temp folders.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script() -> PathBuf {
    root().join("scripts/install-skill.sh")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("install-skill-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(script()).args(args).output().unwrap()
}

fn ok(out: &Output) {
    assert!(
        out.status.success(),
        "failed: {}\n{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn files_in(dir: &Path) -> Vec<String> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                out.push(p.strip_prefix(base).unwrap().to_string_lossy().into_owned());
            }
        }
    }
    let mut v = Vec::new();
    walk(dir, dir, &mut v);
    v.sort();
    v
}

#[cfg(unix)]
fn assert_executable(p: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert!(fs::metadata(p).unwrap().permissions().mode() & 0o111 != 0, "{} is not executable", p.display());
}

#[test]
fn installs_the_skill_with_its_scripts_into_any_folder() {
    let skills = scratch("dir").join("agent-skills");
    let out = run(&["--from", root().to_str().unwrap(), "--dir", skills.to_str().unwrap()]);
    ok(&out);
    let dest = skills.join("rough-animations");
    assert_eq!(
        files_in(&dest),
        ["SKILL.md", "scripts/contact-sheet.sh", "scripts/frame.sh"],
        "exactly the skill, nothing stray"
    );
    // Identical to the repository's copy, and the scripts can still be run.
    assert_eq!(
        fs::read(dest.join("SKILL.md")).unwrap(),
        fs::read(root().join("skills/rough-animations/SKILL.md")).unwrap()
    );
    #[cfg(unix)]
    {
        assert_executable(&dest.join("scripts/frame.sh"));
        assert_executable(&dest.join("scripts/contact-sheet.sh"));
    }
    let usage = Command::new(dest.join("scripts/frame.sh")).output().unwrap();
    assert!(String::from_utf8_lossy(&usage.stderr).contains("usage: frame.sh"));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Installed"));
    // No temporary staging folders are left behind.
    assert_eq!(fs::read_dir(&skills).unwrap().count(), 1, "{:?}", files_in(&skills));
}

#[test]
fn running_it_again_updates_in_place_and_leaves_other_skills_alone() {
    let skills = scratch("update").join("skills");
    let other = skills.join("another-skill");
    fs::create_dir_all(&other).unwrap();
    fs::write(other.join("SKILL.md"), "someone else's skill").unwrap();

    let repo = root();
    let args = ["--from", repo.to_str().unwrap(), "--dir", skills.to_str().unwrap()];
    ok(&run(&args));
    let dest = skills.join("rough-animations");
    // Pretend an old version left a file behind and the SKILL.md was edited.
    fs::write(dest.join("stale-file.txt"), "from an old version").unwrap();
    fs::write(dest.join("SKILL.md"), "edited").unwrap();

    ok(&run(&args));
    assert!(!dest.join("stale-file.txt").exists(), "old files are removed on update");
    assert_ne!(fs::read_to_string(dest.join("SKILL.md")).unwrap(), "edited");
    assert_eq!(fs::read_to_string(other.join("SKILL.md")).unwrap(), "someone else's skill");
}

#[test]
fn project_install_goes_under_the_current_directory() {
    let project = scratch("project");
    let out = Command::new(script())
        .args(["--project", "--from", root().to_str().unwrap()])
        .current_dir(&project)
        .output()
        .unwrap();
    ok(&out);
    assert!(project.join(".claude/skills/rough-animations/SKILL.md").exists());
}

#[test]
fn the_default_is_the_users_claude_folder_and_honours_claude_config_dir() {
    // With CLAUDE_CONFIG_DIR set.
    let cfg = scratch("cfg");
    let out = Command::new(script())
        .args(["--from", root().to_str().unwrap()])
        .env("CLAUDE_CONFIG_DIR", &cfg)
        .output()
        .unwrap();
    ok(&out);
    assert!(cfg.join("skills/rough-animations/SKILL.md").exists());

    // Without it: under ~/.claude (with a throwaway HOME, so the real one is untouched).
    let home = scratch("home");
    let out = Command::new(script())
        .args(["--from", root().to_str().unwrap()])
        .env("HOME", &home)
        .env_remove("CLAUDE_CONFIG_DIR")
        .output()
        .unwrap();
    ok(&out);
    assert!(home.join(".claude/skills/rough-animations/SKILL.md").exists());
}

#[test]
fn uninstall_removes_only_this_skill_and_refuses_to_delete_anything_else() {
    let skills = scratch("uninstall").join("skills");
    let args = |extra: &str| {
        vec![extra.to_string(), "--dir".into(), skills.to_str().unwrap().into()]
    };
    let sibling = skills.join("keep-me");
    fs::create_dir_all(&sibling).unwrap();
    fs::write(sibling.join("SKILL.md"), "keep").unwrap();

    // Nothing installed yet: fine, and says so.
    let out = run(&args("--uninstall").iter().map(String::as_str).collect::<Vec<_>>());
    ok(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Not installed"));

    ok(&run(&["--from", root().to_str().unwrap(), "--dir", skills.to_str().unwrap()]));
    let out = run(&args("--uninstall").iter().map(String::as_str).collect::<Vec<_>>());
    ok(&out);
    assert!(!skills.join("rough-animations").exists());
    assert!(sibling.join("SKILL.md").exists(), "siblings are untouched");

    // A folder with our name that is *not* our skill is never deleted.
    let imposter = skills.join("rough-animations");
    fs::create_dir_all(&imposter).unwrap();
    fs::write(imposter.join("SKILL.md"), "---\nname: something-else\n---\n").unwrap();
    let out = run(&args("--uninstall").iter().map(String::as_str).collect::<Vec<_>>());
    assert!(!out.status.success());
    assert!(stderr(&out).contains("won't delete"), "{}", stderr(&out));
    assert!(imposter.join("SKILL.md").exists());
}

/// A tarball laid out the way GitHub's `archive/<ref>.tar.gz` is: one top folder holding the repo.
fn fake_github_archive(dir: &Path, with_skill: bool) -> PathBuf {
    let top = dir.join("stage/rough-animations-main");
    if with_skill {
        let skill = top.join("skills/rough-animations");
        fs::create_dir_all(skill.join("scripts")).unwrap();
        for f in ["SKILL.md", "scripts/frame.sh", "scripts/contact-sheet.sh"] {
            fs::copy(root().join("skills/rough-animations").join(f), skill.join(f)).unwrap();
        }
    } else {
        fs::create_dir_all(&top).unwrap();
        fs::write(top.join("README.md"), "no skill here").unwrap();
    }
    let tarball = dir.join("archive.tar.gz");
    let status = Command::new("tar")
        .args(["-czf", tarball.to_str().unwrap(), "-C", dir.join("stage").to_str().unwrap(), "rough-animations-main"])
        .status()
        .unwrap();
    assert!(status.success());
    tarball
}

#[test]
fn the_piped_one_liner_downloads_and_installs() {
    if Command::new("curl").arg("--version").output().is_err() {
        eprintln!("skipping: curl not installed");
        return;
    }
    let work = scratch("piped");
    let tarball = fake_github_archive(&work, true);
    let skills = work.join("skills");

    // `curl ... | bash -s -- ...`: the script arrives on stdin, so it cannot find a checkout
    // next to itself and has to download.
    let mut child = Command::new("bash")
        .args(["-s", "--", "--dir", skills.to_str().unwrap()])
        .env("ROUGH_ANIMATIONS_TARBALL", format!("file://{}", tarball.display()))
        .current_dir(&work)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&fs::read(script()).unwrap()).unwrap();
    let out = child.wait_with_output().unwrap();
    ok(&out);
    assert!(String::from_utf8_lossy(&out.stdout).contains("Downloading"));
    let dest = skills.join("rough-animations");
    assert_eq!(files_in(&dest), ["SKILL.md", "scripts/contact-sheet.sh", "scripts/frame.sh"]);
    #[cfg(unix)]
    assert_executable(&dest.join("scripts/frame.sh"));
}

#[test]
fn download_failures_are_explained() {
    if Command::new("curl").arg("--version").output().is_err() {
        return;
    }
    let work = scratch("badurl");
    let skills = work.join("skills");
    // Run from the checkout the script would use it, so copy it elsewhere to force the download.
    let lonely = work.join("lonely/install-skill.sh");
    fs::create_dir_all(lonely.parent().unwrap()).unwrap();
    fs::copy(script(), &lonely).unwrap();
    let run_lonely = |tarball: &str| {
        Command::new(&lonely)
            .args(["--dir", skills.to_str().unwrap()])
            .env("ROUGH_ANIMATIONS_TARBALL", tarball)
            .output()
            .unwrap()
    };
    // A URL that does not exist.
    let out = run_lonely(&format!("file://{}/missing.tar.gz", work.display()));
    assert!(!out.status.success());
    assert!(stderr(&out).contains("could not download"), "{}", stderr(&out));
    assert!(!skills.join("rough-animations").exists());

    // An archive from the wrong repository: no skill inside.
    let wrong = fake_github_archive(&scratch("badurl-wrong"), false);
    let out = run_lonely(&format!("file://{}", wrong.display()));
    assert!(!out.status.success());
    assert!(stderr(&out).contains("no skills/rough-animations"), "{}", stderr(&out));
}

#[test]
fn bad_arguments_get_a_usage_hint_and_help_works() {
    for args in [&["--bogus"][..], &["--dir"][..], &["--ref"][..], &["--from"][..]] {
        let out = run(args);
        assert!(!out.status.success(), "{args:?} should fail");
        assert!(!out.stderr.is_empty(), "{args:?} should explain");
    }
    let help = run(&["--help"]);
    ok(&help);
    let text = String::from_utf8_lossy(&help.stdout);
    for needle in ["curl -fsSL", "--project", "--dir PATH", "--uninstall", "claude"] {
        assert!(text.contains(needle), "help should mention {needle:?}");
    }
    // Refuses a nonsense target.
    let out = run(&["--dir", "/", "--from", root().to_str().unwrap()]);
    assert!(!out.status.success());
}
