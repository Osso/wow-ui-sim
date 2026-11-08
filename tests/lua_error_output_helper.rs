//! Behavioural tests for the shared `lua-errors` output assertion.

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use lua_error_output::{assert_no_lua_errors, assert_no_lua_sink_errors};

const CLEAN_STDERR: &str = "\
=== Final observed Lua error summary ===
Status: CLEAN
Lua errors: 0 unique, 0 occurrence(s)
Attributed owners: 0
Unattributed Lua errors: 0 occurrence(s)
";

const FAILED_STDERR: &str = "\
=== Final observed Lua error summary ===
Status: FAILED
Lua errors: 1 unique, 1 occurrence(s)
Attributed owners: 1
  Foo: 1 occurrence(s)
Unattributed Lua errors: 0 occurrence(s)
";

fn assert_rejected(context: &str, stdout: &str, stderr: &str) {
    let result = std::panic::catch_unwind(|| assert_no_lua_errors(context, stdout, stderr));
    assert!(
        result.is_err(),
        "helper accepted output that reports Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

#[test]
fn accepts_clean_summary() {
    assert_no_lua_errors("clean", "[]", CLEAN_STDERR);
}

#[test]
fn rejects_failed_summary() {
    assert_rejected(
        "failed",
        r#"[{"message":"boom","count":1}]"#,
        FAILED_STDERR,
    );
}

#[test]
fn rejects_nonempty_stdout_with_clean_summary() {
    assert_rejected(
        "nonempty",
        r#"[{"message":"boom","count":1}]"#,
        CLEAN_STDERR,
    );
}

#[test]
fn rejects_missing_summary() {
    assert_rejected("missing", "[]", "no summary here\n");
}

#[test]
fn rejects_exec_lua_error() {
    assert_rejected("exec", "[]", &format!("{CLEAN_STDERR}[exec-lua] error: boom\n"));
}

#[test]
fn rejects_lua_error_text_on_stdout() {
    assert_rejected("stdout-lua-error", "Lua error: boom\n[]", CLEAN_STDERR);
}

#[test]
fn rejects_exec_lua_error_on_stdout() {
    assert_rejected(
        "stdout-exec",
        "[exec-lua] error: boom\n[]",
        CLEAN_STDERR,
    );
}

#[test]
fn accepts_sink_output_without_summary() {
    assert_no_lua_sink_errors("sink-clean", "frame tree\n", "[Startup] complete\n");
}

#[test]
fn sink_rejects_lua_error_text() {
    let result = std::panic::catch_unwind(|| {
        assert_no_lua_sink_errors("sink-dirty", "", "Lua error: boom\n")
    });
    assert!(result.is_err(), "sink helper accepted a Lua error line");
}

#[test]
fn sink_rejects_exec_lua_error_text() {
    let result = std::panic::catch_unwind(|| {
        assert_no_lua_sink_errors("sink-exec", "", "[exec-lua] error: boom\n")
    });
    assert!(
        result.is_err(),
        "sink helper accepted an exec-lua error line"
    );
}
