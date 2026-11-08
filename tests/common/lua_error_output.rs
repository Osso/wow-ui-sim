//! Shared assertion for `wow-sim ... lua-errors` output.
//!
//! `src/lua_errors.rs::run_lua_errors` prints the unique-error JSON array to
//! stdout and `final_error_summary_lines` writes the structured summary to
//! stderr:
//!
//! ```text
//! === Final observed Lua error summary ===
//! Status: CLEAN
//! Lua errors: 0 unique, 0 occurrence(s)
//! ```
//!
//! The summary line contains the substring "Lua error", so a plain
//! `stderr.contains("Lua error")` check rejects a clean run. Parse the
//! structured fields instead.
//!
//! The stdout substring guards preserve the checks the per-file helpers used
//! before conversion: user output from `--exec-lua` can print before the JSON
//! array, so a clean summary alone does not prove those substrings are absent.

/// Assert that a `lua-errors` run reported no Lua errors.
///
/// `context` names the flow in the panic message.
pub fn assert_no_lua_errors(context: &str, stdout: &str, stderr: &str) {
    let status = summary_field(stderr, "Status: ");
    let unique = summary_field(stderr, "Lua errors: ")
        .and_then(|value| value.split_whitespace().next())
        .and_then(|count| count.parse::<usize>().ok());

    let clean = status.as_deref() == Some("CLEAN")
        && unique == Some(0)
        && stdout.trim_end().ends_with("[]")
        && !stdout.contains("Lua error")
        && !stdout.contains("[exec-lua] error")
        && !stderr.contains("[exec-lua] error");

    assert!(
        clean,
        "{context} flow emitted Lua errors\n\
         status: {status:?}\n\
         unique Lua errors: {unique:?}\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

/// Assert that a run reported no Lua errors for subcommands that print no
/// summary banner (`dump-tree`, `screenshot`, `run-tests`).
///
/// Those commands report errors through the canonical error sink
/// (`Lua error: ...` on stderr) and through `Lua error suppressed ...` lines
/// printed when the environment is dropped. There is no `Status:` line to
/// parse, so judge the error text itself.
pub fn assert_no_lua_sink_errors(context: &str, stdout: &str, stderr: &str) {
    let clean = !stdout.contains("Lua error")
        && !stdout.contains("[exec-lua] error")
        && !stderr.contains("Lua error")
        && !stderr.contains("[exec-lua] error");

    assert!(
        clean,
        "{context} flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn summary_field<'a>(stderr: &'a str, prefix: &str) -> Option<&'a str> {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::trim)
}
