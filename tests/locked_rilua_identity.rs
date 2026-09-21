#[path = "../build/locked_rilua.rs"]
mod locked_rilua;

use locked_rilua::parse_locked_rilua_revision;

const REVISION_A: &str = "1111111111111111111111111111111111111111";
const REVISION_B: &str = "2222222222222222222222222222222222222222";

fn package(source: Option<&str>) -> String {
    let source = source
        .map(|value| format!("source = \"{value}\"\n"))
        .unwrap_or_default();
    format!("[[package]]\nname = \"rilua\"\nversion = \"0.1.21\"\n{source}")
}

#[test]
fn locked_compiler_uses_resolved_revision_not_package_version_or_query() {
    let first = package(Some(&format!(
        "git+https://example.test/rilua?branch=main#{REVISION_A}"
    )));
    let second = package(Some(&format!(
        "git+https://example.test/rilua?rev=tag#{REVISION_B}"
    )));
    let unrelated = "[[package]]\nname = \"another-package\"\nversion = \"1.0.0\"\n";
    assert_eq!(
        parse_locked_rilua_revision(&(unrelated.to_owned() + &first)),
        Ok(REVISION_A)
    );
    assert_eq!(parse_locked_rilua_revision(&second), Ok(REVISION_B));
    assert_eq!(
        parse_locked_rilua_revision(&second.replace('\n', "\r\n")),
        Ok(REVISION_B)
    );
}

#[test]
fn locked_compiler_rejects_unidentified_or_ambiguous_sources() {
    for input in [
        String::new(),
        package(None),
        package(Some("registry+https://example.test/index")),
        package(Some("git+https://example.test/rilua?branch=main")),
        package(Some("git+https://example.test/rilua#1234")),
        package(Some(
            "git+https://example.test/rilua#xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        )),
        package(Some(&format!("git+#{REVISION_A}"))),
    ] {
        assert!(parse_locked_rilua_revision(&input).is_err(), "{input}");
    }
    let one = package(Some(&format!(
        "git+https://example.test/rilua#{REVISION_A}"
    )));
    let two = package(Some(&format!(
        "git+https://example.test/rilua#{REVISION_B}"
    )));
    assert!(parse_locked_rilua_revision(&(one + &two)).is_err());
}

#[test]
fn compiled_identity_matches_the_locked_rilua_revision() {
    let locked = parse_locked_rilua_revision(include_str!("../Cargo.lock")).unwrap();
    assert_eq!(env!("WOW_SIM_RILUA_COMPILER_REVISION"), locked);
}
