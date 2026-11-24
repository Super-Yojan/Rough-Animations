//! The Claude Code plugin and marketplace manifests, checked for consistency and, when the
//! `claude` CLI is installed, by the real validator.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn json(path: &str) -> Value {
    let text = std::fs::read_to_string(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path} is not valid JSON: {e}"))
}

fn cargo_version() -> String {
    let manifest = std::fs::read_to_string(root().join("Cargo.toml")).unwrap();
    manifest
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|v| v.strip_suffix('"'))
        .expect("Cargo.toml has a version")
        .to_string()
}

#[test]
fn the_plugin_manifest_describes_the_skill() {
    let plugin = json(".claude-plugin/plugin.json");
    assert_eq!(plugin["name"], "rough-animations");
    assert!(plugin["description"].as_str().unwrap().len() > 40);
    assert_eq!(
        plugin["version"].as_str().unwrap(),
        cargo_version(),
        "bump plugin.json and marketplace.json together with Cargo.toml"
    );
    assert!(plugin["repository"].as_str().unwrap().starts_with("https://github.com/"));
}

#[test]
fn the_marketplace_lists_the_plugin_at_the_repo_root() {
    let market = json(".claude-plugin/marketplace.json");
    assert_eq!(market["name"], "rough-animations");
    assert!(market["description"].as_str().unwrap().len() > 20);
    let plugins = market["plugins"].as_array().unwrap();
    assert_eq!(plugins.len(), 1);
    let entry = &plugins[0];
    assert_eq!(entry["name"], "rough-animations");
    assert_eq!(entry["source"], "./", "the plugin is this repository itself");
    assert_eq!(entry["version"].as_str().unwrap(), cargo_version());
    // The plugin root really holds the skill where plugins look for it.
    assert!(root().join("skills/rough-animations/SKILL.md").is_file());
    assert_eq!(entry["name"], json(".claude-plugin/plugin.json")["name"]);
}

#[test]
fn the_in_repo_skill_is_a_link_to_the_same_files() {
    let link = root().join(".claude/skills/rough-animations");
    assert!(link.symlink_metadata().unwrap().file_type().is_symlink(), "one copy, linked");
    assert_eq!(
        link.canonicalize().unwrap(),
        root().join("skills/rough-animations").canonicalize().unwrap()
    );
    assert!(link.join("SKILL.md").is_file() && link.join("scripts/frame.sh").is_file());
}

#[test]
fn manifests_carry_no_personal_details() {
    // These are published when the repository is. A GitHub handle is fine; an email is not.
    for path in [".claude-plugin/plugin.json", ".claude-plugin/marketplace.json"] {
        let text = std::fs::read_to_string(root().join(path)).unwrap();
        assert!(!text.contains('@'), "{path} contains an '@': no email addresses in manifests");
    }
}

#[test]
fn the_real_validator_accepts_it_and_finds_the_skill() {
    if Command::new("claude").arg("--version").output().is_err() {
        eprintln!("skipping: the claude CLI is not installed");
        return;
    }
    let out = Command::new("claude")
        .args(["plugin", "validate"])
        .arg(root())
        .output()
        .unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "claude plugin validate failed:\n{text}");
    assert!(
        text.contains("skills: rough-animations:rough-animations"),
        "the validator should find the skill:\n{text}"
    );
    // Everything before the "plugin directory" section is about the manifests themselves. That
    // later section is for submitting to Anthropic's catalog (it asks for a LICENSE and an
    // icon), which is not needed to install from this repository.
    let manifest_part = text.split("Plugin directory checks").next().unwrap();
    assert!(!manifest_part.contains('✘'), "the manifests have a problem:\n{text}");
    assert!(!text.contains("Validation failed"), "{text}");

    // The marketplace file on its own validates with no warnings.
    let out = Command::new("claude")
        .args(["plugin", "validate"])
        .arg(root().join(".claude-plugin/marketplace.json"))
        .output()
        .unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success(), "{text}");
    assert!(!text.contains("warning"), "marketplace.json should validate cleanly:\n{text}");
}
