use std::process::Command;

#[test]
fn run_tests_starts_with_demo_party_and_respects_no_party() {
    let root = tempfile::tempdir().expect("create fixture");
    let addons = root.path().join("AddOns");
    let addon = addons.join("PartyStartupFixture");
    std::fs::create_dir_all(addon.join("tests")).unwrap();
    std::fs::write(
        addon.join("PartyStartupFixture.toc"),
        format!(
            "## Interface: {}\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
    .unwrap();
    for (disabled, source) in [
        (
            false,
            "test('demo party', function() assertEquals(5, GetNumGroupMembers()); assertEquals('Thrynn', UnitName('party1')); assertTrue(IsInGroup()) end)",
        ),
        (
            true,
            "test('solo', function() assertEquals(0, GetNumGroupMembers()); assertNil(UnitName('party1')); assertFalse(IsInGroup()) end)",
        ),
    ] {
        std::fs::write(addon.join("tests/party.lua"), source).unwrap();
        let mut command = Command::new("timeout");
        command
            .args([
                "90",
                env!("CARGO_BIN_EXE_wow-sim"),
                "--no-addons",
                "--no-saved-vars",
                "--addons-path",
            ])
            .arg(&addons)
            .args(["run-tests", "PartyStartupFixture"])
            .env("WOW_SIM_WTF_PATH", root.path().join("wtf"));
        if disabled {
            command.env("WOW_SIM_NO_PARTY", "1");
        } else {
            command.env_remove("WOW_SIM_NO_PARTY");
        }
        let output = command.output().expect("run addon test fixture");
        assert!(
            output.status.success(),
            "disabled={disabled}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
