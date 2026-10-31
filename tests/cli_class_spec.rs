use std::path::{Path, PathBuf};
use std::process::Command;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

fn isolated_data_dir(root: &Path) -> PathBuf {
    let data = root.join("data");
    if let Some(seed) = std::env::var_os("WOW_SIM_TEST_CVAR_SEED") {
        let cvars = data.join("wow-sim");
        std::fs::create_dir_all(&cvars).unwrap();
        std::fs::copy(seed, cvars.join("cvars.json")).unwrap();
    }
    data
}

#[test]
fn cli_class_and_spec_are_visible_to_exec_lua() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    for (class, spec, expected) in [
        ("dRuId", "restoration", "CLI_CONTEXT:Druid:4"),
        ("Mage", "FIRE", "CLI_CONTEXT:Mage:2"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let output = Command::new("timeout")
            .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
            .args([
                "--no-addons",
                "--no-saved-vars",
                "--class",
                class,
                "--spec",
                spec,
                "--exec-lua",
                "print('CLI_CONTEXT:' .. UnitClass('player') .. ':' .. GetSpecialization())",
                "dump-tree",
                "--filter",
                "__no_such_frame__",
            ])
            .env("XDG_DATA_HOME", isolated_data_dir(root.path()))
            .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
            .env("WOW_SIM_NO_SOUND", "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(stderr.contains(expected), "{stdout}\n{stderr}");
    }
}

fn assert_monk_specialization(spec: &str, expected: &str) {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let output = Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--class",
            "Monk",
            "--spec",
            spec,
            "--exec-lua",
            r#"
                assert(C_AddOns.LoadAddOn('Blizzard_PlayerSpells'))
                local index = GetSpecialization()
                local frame = CreateFrame('Frame', nil, UIParent, 'ClassSpecContentFrameTemplate')
                frame:Setup(index, UnitSex('player'), 200, 3)
                local id, name = C_SpecializationInfo.GetSpecializationInfo(index)
                print('MONK_CONTEXT:' .. index .. ':' .. id .. ':' .. name .. ':' .. frame.SpecName:GetText())
            "#,
            "dump-tree",
            "--filter",
            "__no_such_frame__",
        ])
        .env("XDG_DATA_HOME", isolated_data_dir(root.path()))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .env("WOW_SIM_NO_SOUND", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stderr.contains(expected), "{stdout}\n{stderr}");
}

#[test]
fn monk_mistweaver_flag_matches_specialization_identity() {
    assert_monk_specialization("Mistweaver", "MONK_CONTEXT:2:270:Mistweaver:Mistweaver");
}

#[test]
fn monk_windwalker_flag_matches_specialization_identity() {
    assert_monk_specialization("Windwalker", "MONK_CONTEXT:3:269:Windwalker:Windwalker");
}

#[test]
fn spec_without_class_is_seeded_then_resolved_again_after_startup() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let addons = root.path().join("addons");
    let addon = addons.join("Issue5ContextProbe");
    std::fs::create_dir_all(&addon).unwrap();
    std::fs::write(
        addon.join("Issue5ContextProbe.toc"),
        "## Interface: 120105\nIssue5ContextProbe.lua\n",
    )
    .unwrap();
    std::fs::write(addon.join("Issue5ContextProbe.lua"), r#"
        Issue5Initial = UnitClass('player') .. ':' .. GetSpecialization()
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('PLAYER_ENTERING_WORLD')
        frame:RegisterUnitEvent('PLAYER_SPECIALIZATION_CHANGED', 'player')
        frame:SetScript('OnEvent', function(_, event, unit)
            if event == 'PLAYER_ENTERING_WORLD' then
                Issue5WorldCount = (Issue5WorldCount or 0) + 1
                A_Admin.SetPlayerClass(5)
            else
                Issue5Observed = UnitClass('player') .. ':' .. GetSpecialization() .. ':' .. tostring(unit)
            end
        end)
    "#).unwrap();
    let output = Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .arg("--no-saved-vars")
        .arg("--addons-path").arg(addons)
        .args(["--spec", "Holy", "--exec-lua", "print('EVENT_CONTEXT:' .. tostring(Issue5Observed) .. ':' .. tostring(Issue5WorldCount) .. ':' .. tostring(Issue5Initial))", "dump-tree", "--filter", "__no_such_frame__"])
        .env("XDG_DATA_HOME", isolated_data_dir(root.path()))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .env("WOW_SIM_NO_SOUND", "1")
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stderr.contains("EVENT_CONTEXT:Priest:2:player:1:Paladin:1"),
        "{stdout}\n{stderr}"
    );
}

#[test]
fn unknown_class_and_specialization_report_valid_choices() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    for (args, expected) in [
        (vec!["--class", "NotAClass"], "valid classes: Warrior"),
        (
            vec!["--class", "Mage", "--spec", "NotASpec"],
            "valid specializations: Arcane",
        ),
        (vec!["--spec", "NotASpec"], "valid specializations: Holy"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let output = Command::new("timeout")
            .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
            .args(["--no-addons", "--no-saved-vars"])
            .args(args)
            .arg("dump-tree")
            .env("XDG_DATA_HOME", isolated_data_dir(root.path()))
            .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
            .env("WOW_SIM_NO_SOUND", "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            matches!(output.status.code(), Some(1 | 2)),
            "{stdout}\n{stderr}"
        );
        assert!(stderr.contains(expected), "{stdout}\n{stderr}");
    }
}

#[test]
fn numeric_class_and_spec_indices_are_visible_to_exec_lua() {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let output = Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--class",
            "8",
            "--spec",
            "2",
            "--exec-lua",
            "print('NUMERIC_CONTEXT:' .. UnitClass('player') .. ':' .. GetSpecialization())",
            "dump-tree",
            "--filter",
            "__no_such_frame__",
        ])
        .env("XDG_DATA_HOME", isolated_data_dir(root.path()))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .env("WOW_SIM_NO_SOUND", "1")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stderr.contains("NUMERIC_CONTEXT:Mage:2"),
        "{stdout}\n{stderr}"
    );
}
