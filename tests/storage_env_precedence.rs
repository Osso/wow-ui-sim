use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[test]
fn saved_variables_storage_env_precedence() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let addons = root.join("addons");
    let addon = addons.join("StorageProbe");
    std::fs::create_dir_all(&addon).unwrap();
    std::fs::write(
        addon.join("StorageProbe.toc"),
        format!(
            "## Interface: {}\n## SavedVariables: StorageProbeValue\nStorageProbe.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
    .unwrap();
    std::fs::write(
        addon.join("StorageProbe.lua"),
        "StorageProbeValue = StorageProbeValue or 'missing'\n",
    )
    .unwrap();
    for (path, value) in [
        (root.join("legacy"), "env"),
        (root.join("explicit/SavedVariables"), "explicit"),
        (root.join("data/wow-sim/SavedVariables"), "default"),
    ] {
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(
            path.join("StorageProbe.lua"),
            format!("StorageProbeValue = '{value}'\n"),
        )
        .unwrap();
    }
    for (env_value, explicit, expected) in [
        (Some(root.join("legacy")), false, "env"),
        (Some(root.join("legacy")), true, "explicit"),
        (None, false, "default"),
    ] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_wow-sim"));
        cmd.arg("--addons-path")
            .arg(&addons)
            .env("XDG_DATA_HOME", root.join("data"))
            .env("WOW_SIM_WTF_PATH", root.join("missing-wtf"))
            .env_remove("WOW_SIM_NO_SAVED_VARS")
            .env_remove("WOW_SIM_NO_ADDONS");
        if let Some(value) = env_value {
            cmd.env("WOW_SIM_STORAGE_DIR", value);
        } else {
            cmd.env_remove("WOW_SIM_STORAGE_DIR");
        }
        if explicit {
            cmd.arg("--session-wtf-path").arg(root.join("explicit"));
        }
        let output = bounded_process::output(
            90,
            cmd.args([
                "--exec-lua",
                "print('STORAGE_PROBE=' .. tostring(StorageProbeValue))",
                "dump-tree",
                "--filter",
                "UIParent",
            ]),
        )
        .unwrap();
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "{text}");
        assert!(
            text.contains(&format!("STORAGE_PROBE={expected}")),
            "{text}"
        );
    }
}
