use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};

const ADDON: &str = "RunTestsOutputFixture";
const FILE: &str = "failure_output.lua";

fn run_fixture(source: &str) -> Output {
    let root = tempfile::tempdir().expect("create run-tests fixture");
    let addons = root.path().join("AddOns");
    let addon = addons.join(ADDON);
    let tests = addon.join("tests");
    std::fs::create_dir_all(&tests).expect("create fixture tests directory");
    std::fs::write(
        addon.join(format!("{ADDON}.toc")),
        format!(
            "## Interface: {}\n## Title: {ADDON}\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
    .expect("write fixture TOC");
    std::fs::write(tests.join(FILE), source).expect("write fixture tests");
    let data = root.path().join("data");
    copy_current_cvars(&data);
    Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args(["--no-addons", "--no-saved-vars", "--addons-path"])
        .arg(addons)
        .args(["run-tests", ADDON])
        .env("XDG_DATA_HOME", data)
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .output()
        .expect("run bounded real run-tests command")
}

fn copy_current_cvars(data: &Path) {
    let source = dirs::data_local_dir()
        .expect("locate current CVar directory")
        .join("wow-sim/cvars.json");
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOATIME);
    }
    let mut file = match options.open(&source) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => panic!("open current CVar file {}: {error}", source.display()),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read current CVars");
    let storage = data.join("wow-sim");
    std::fs::create_dir_all(&storage).expect("create child-only CVar directory");
    std::fs::write(storage.join("cvars.json"), bytes).expect("copy current CVars byte-for-byte");
}

#[test]
fn delay_dispatches_addon_timer_and_updates_before_exec_lua() {
    let root = tempfile::tempdir().expect("create delay fixture");
    let addons = root.path().join("AddOns");
    let addon = addons.join("DelayFixture");
    std::fs::create_dir_all(&addon).expect("create fixture addon");
    std::fs::write(
        addon.join("DelayFixture.toc"),
        format!(
            "## Interface: {}\nDelayFixture.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    )
    .expect("write fixture TOC");
    std::fs::write(
        addon.join("DelayFixture.lua"),
        r#"
        DelayFixtureElapsed = 0
        DelayFixtureFired = false
        DelayFixtureScheduled = false
        -- Headless startup dispatches six updates after FIRST_FRAME_RENDERED.
        -- Schedule only on a subsequent update, never during startup.
        local ticks = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("FIRST_FRAME_RENDERED")
        frame:SetScript("OnEvent", function()
            frame:SetScript("OnUpdate", function(_, elapsed)
                ticks = ticks + 1
                if ticks <= 6 then return end
                DelayFixtureElapsed = DelayFixtureElapsed + elapsed
                if not DelayFixtureScheduled then
                    DelayFixtureScheduled = true
                    C_Timer.After(0.5, function() DelayFixtureFired = true end)
                end
            end)
        end)
    "#,
    )
    .expect("write fixture Lua");

    for (delay, scheduled, fired) in [
        (Some(1500), "true", "true"),
        (Some(20), "true", "false"),
        (Some(0), "false", "false"),
        (None, "false", "false"),
    ] {
        let data = root.path().join(format!("data-{delay:?}"));
        copy_current_cvars(&data);
        let mut command = Command::new("timeout");
        command
            .args([
                "90",
                env!("CARGO_BIN_EXE_wow-sim"),
                "--no-saved-vars",
                "--addons-path",
            ])
            .arg(&addons);
        if let Some(delay) = delay {
            command.args(["--delay", &delay.to_string()]);
        }
        let output = command.args([
                "--exec-lua",
                "print('DELAY_RESULT', tostring(DelayFixtureScheduled), tostring(DelayFixtureFired), DelayFixtureElapsed)",
                "dump-tree",
                "--filter",
                "UIParent",
            ])
            .env("XDG_DATA_HOME", data)
            .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
            .output()
            .expect("run dump-tree fixture");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let text = format!("{stdout}\n{stderr}");
        assert!(output.status.success(), "dump-tree failed: {text}");
        let line = text
            .lines()
            .find(|line| line.contains("DELAY_RESULT"))
            .unwrap_or_else(|| panic!("missing result: {text}"));
        let values: Vec<_> = line.split_whitespace().collect();
        let index = values
            .iter()
            .position(|value| *value == "DELAY_RESULT")
            .unwrap();
        assert_eq!(values[index + 2], fired, "delay {delay:?}: {line}");
        assert_eq!(values[index + 1], scheduled, "delay {delay:?}: {line}");
        let elapsed: f64 = values[index + 3].parse().expect("numeric elapsed");
        if delay.is_some_and(|delay| delay > 0) {
            assert!(elapsed > 0.0, "delay {delay:?}: {line}");
        }
        println!("delay {delay:?}: {line}");
    }
}

fn assert_failure(output: &Output, name: &str, error: &str, summary: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let captured = format!("{stdout}\n{stderr}");
    assert_eq!(output.status.code(), Some(1), "{captured}");
    for required in [FILE, name, error] {
        assert!(
            captured.contains(required),
            "missing {required}: {captured}"
        );
    }
    let plain = stderr
        .replace("\x1b[31m", "")
        .replace("\x1b[32m", "")
        .replace("\x1b[0m", "");
    let failure = format!("  ✗ {FILE} > {name}\n    {error}\n");
    assert_eq!(plain.matches(&failure).count(), 1, "{captured}");
    assert!(plain.contains(summary), "{captured}");
    assert!(
        plain.find(&failure).unwrap() < plain.find(summary).unwrap(),
        "{captured}"
    );
}

#[test]
fn sync_failure_reaches_process_output() {
    let output = run_fixture(
        "test('passing sync', function() assertEquals(42, 6 * 7) end)\n\
         test('failing sync', function() error('sync failure detail', 0) end)",
    );
    assert_failure(
        &output,
        "failing sync",
        "sync failure detail",
        "2 tests, 1 passed, 1 failed",
    );
}

#[test]
fn async_failure_reaches_process_output() {
    let output = run_fixture(
        "async_test('passing async', function(done) done() end)\n\
         async_test('failing async', function(done)\n\
             C_Timer.After(0, function()\n\
                 done(function() error('async failure detail', 0) end)\n\
             end)\n\
         end)",
    );
    assert_failure(
        &output,
        "failing async",
        "async failure detail",
        "2 tests, 1 passed, 1 failed",
    );
}

#[test]
fn sync_failure_after_async_registration_reaches_process_output() {
    let output = run_fixture(
        "async_test('passing async', function(done) done() end)\n\
         test('failing mixed sync', function() error('mixed sync failure detail', 0) end)",
    );
    assert_failure(
        &output,
        "failing mixed sync",
        "mixed sync failure detail",
        "2 tests, 1 passed, 1 failed",
    );
}

#[test]
fn passing_tests_keep_success_summary_and_exit_code() {
    let output = run_fixture(
        "test('passing sync', function() assertEquals(42, 6 * 7) end)\n\
         async_test('passing async', function(done) done() end)",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(
        stderr.contains(&format!("  \x1b[32m✓\x1b[0m {FILE} (2 tests)")),
        "{stderr}"
    );
    assert!(
        stderr.contains("2 tests, \x1b[32m2 passed\x1b[0m, \x1b[31m0 failed\x1b[0m"),
        "{stderr}"
    );
}
