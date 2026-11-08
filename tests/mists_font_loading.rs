#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[test]
fn mists_startup_uses_known_font_keys_without_stale_path_errors() {
    let output = bounded_process::output(
        90,
        Command::new(env!("CARGO_BIN_EXE_wow-sim")).args([
            "--no-addons",
            "--no-saved-vars",
            "dump-tree",
            "--filter",
            "UIParent",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("asset-cache byte resolve failed: fdid 615960")
            && !stderr.contains("asset-cache byte resolve failed: fdid 615958"),
        "Mists startup should not report stale CASC path failures for known fonts\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("Font FRIZQT__.TTF not found in CASC")
            && !stderr.contains("Font ARIALN.TTF not found in CASC"),
        "Mists startup should load core Blizzard fonts without falling back to system fonts\nstderr:\n{stderr}"
    );
}
