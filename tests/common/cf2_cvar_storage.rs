//! Each case owns an empty persistent store, shared only by its environments.
use std::process::Command;

#[path = "executable_spawn.rs"]
mod executable_spawn;

pub fn run_isolated(selector: &str, body: impl FnOnce()) {
    let executable = std::env::current_exe().expect("locate integration executable");
    if executable
        .file_stem()
        .is_some_and(|name| name == "cf2-cvar-child")
    {
        let data = executable
            .parent()
            .expect("child fixture directory")
            .join("data");
        assert_eq!(
            std::env::var_os("XDG_DATA_HOME"),
            Some(data.into_os_string())
        );
        body();
        return;
    }

    let output = run_fixture_case(selector, &executable);
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "isolated case {selector}: {}",
        output.status
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("running 1 test\n"),
        "isolated invocation must select exactly one test"
    );
}

fn run_fixture_case(selector: &str, executable: &std::path::Path) -> std::process::Output {
    let fixture = tempfile::tempdir().expect("create case-owned CVar fixture");
    let child = fixture
        .path()
        .join(format!("cf2-cvar-child{}", std::env::consts::EXE_SUFFIX));
    // Outside deps, production CVarStorage reopens the same file across environments.
    std::fs::copy(executable, &child).expect("copy integration executable into fixture");
    let data = fixture.path().join("data");
    let storage = data.join("wow-sim");
    std::fs::create_dir_all(&storage).expect("create child-only CVar directory");
    std::fs::write(storage.join("cvars.json"), b"{}").expect("seed empty CVar overrides");
    executable_spawn::output_with_busy_retry(
        Command::new(child)
            .args([selector, "--exact", "--nocapture"])
            .env("XDG_DATA_HOME", data)
            .env_remove("CF2_TTS_ISOLATED_DATA_HOME"),
    )
    .expect("run exact selector with case-owned storage")
}
