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
        && !stderr.contains("[exec-lua] error");

    assert!(
        clean,
        "{context} flow emitted Lua errors\n\
         status: {status:?}\n\
         unique Lua errors: {unique:?}\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn summary_field<'a>(stderr: &'a str, prefix: &str) -> Option<&'a str> {
    stderr
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::trim)
}
