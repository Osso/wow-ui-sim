# Rilua secret-string transform

Active goal: implement Rust-only authenticated secret-string transform with unchanged caller taint, byte preservation, fresh wrapper identity, and GC safety; add existing-style behavioral tests and sibling docs; commit without push.

Scope: only `/home/osso-test/.worktrees/rilua-secret-string-transform` and this scratchpad report/logs. No consumer changes, agents, model CLIs, or deployment.

Initial state: clean branch `secret-string-transform`, HEAD `6044544b960cd68b4b0c58bb3373412757c2caee`, tracking origin/host-secret-bool. No repo CLAUDE.md or AGENTS.md exists. README read. Existing tests live in helpers grouped into `tests/integration.rs`; `table_security` is already a public module.

Proof ledger: no tests run yet.

Baseline `rilua-secret-string-transform` at 6044544: `cargo test -j 4 --test integration table_security` exit 0. Log: `baseline-table_security.log`.
iling[0m same-file v1.0.6
[1m[92m   Compiling[0m cast v0.3.0
[1m[92m   Compiling[0m bit-vec v0.8.0
[1m[92m   Compiling[0m str_stack v0.1.0
[1m[92m   Compiling[0m quick-error v1.2.3
[1m[92m   Compiling[0m inferno v0.12.6
[1m[92m   Compiling[0m bit-set v0.8.0
[1m[92m   Compiling[0m rusty-fork v0.3.1
[1m[92m   Compiling[0m criterion-plot v0.5.0
[1m[92m   Compiling[0m walkdir v2.5.0
[1m[92m   Compiling[0m indicatif v0.18.4
[1m[92m   Compiling[0m opener v0.8.4
[1m[92m   Compiling[0m cargo_metadata v0.23.1
[1m[92m   Compiling[0m rand_chacha v0.9.0
[1m[92m   Compiling[0m rayon v1.11.0
[1m[92m   Compiling[0m ciborium v0.2.2
[1m[92m   Compiling[0m plotters v0.3.7
[1m[92m   Compiling[0m clap_complete v4.6.1
[1m[92m   Compiling[0m tinytemplate v1.2.1
[1m[92m   Compiling[0m rand v0.9.3
[1m[92m   Compiling[0m rand_xorshift v0.4.0
[1m[92m   Compiling[0m regex v1.12.3
[1m[92m   Compiling[0m is-terminal v0.4.17
[1m[92m   Compiling[0m shlex v1.3.0
[1m[92m   Compiling[0m unarray v0.1.4
[1m[92m   Compiling[0m anes v0.1.6
[1m[92m   Compiling[0m oorandom v11.1.5
[1m[92m   Compiling[0m rustc-demangle v0.1.27
[1m[92m   Compiling[0m proptest v1.11.0
[1m[92m   Compiling[0m criterion v0.5.1
[1m[92m   Compiling[0m flamegraph v0.6.12
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 26.63s
[1m[92m     Running[0m tests/integration.rs (target/debug/deps/integration-6ca9a24e5be5ea00)


Baseline `rilua-secret-string-transform` at 6044544: `cargo test -j 4 --test integration secret_string_formatting` exit 0. Log: `baseline-secret_string_formatting.log`.

running 6 tests
test secret_string_formatting::secret_format_precision_preserves_full_payload ... ok
test secret_string_formatting::secret_format_width_and_alignment_add_no_padding ... ok
test secret_string_formatting::secret_format_public_controls_keep_width_and_precision ... ok
test secret_string_formatting::secret_format_mixed_arguments_and_gc_retain_typed_result ... ok
test secret_string_formatting::secret_format_inferred_tainted_operation_preserves_guards_and_stack_taint ... ok
test secret_string_formatting::secret_format_unused_secret_input_still_marks_output ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 473 filtered out; finished in 0.00s

[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 0.12s
[1m[92m     Running[0m tests/integration.rs (target/debug/deps/integration-6ca9a24e5be5ea00)


RED: `cargo test -j 4 --test integration secret_string_transform` on baseline plus new tests: exit 101, unresolved import of missing transform_host_secret_string (expected missing API).

Implementation and five behavioral tests added, formatted with `cargo fmt`, committed before verification. Public module export already exists; no Lua global added. Input bytes copied before callback; payload temporarily pushed while allocating wrapper, stack top restored. Docs updated in API reference, table-security spec, and changelog.

First commit attempt failed: no configured Git author. Retried with per-command `-c user.name=Alessio Deiana -c user.email=adeiana@gmail.com` (no persistent configuration changed). Commit retry exit 0.

At 70625a96e1556c45845fb4d030f37af3ef896877: `cargo test -j 4 --test integration secret_string_transform` exit 0.

running 5 tests
test secret_string_transform::host_secret_transform_preserves_binary_bytes_empty_strings_and_identity ... ok
test secret_string_transform::host_secret_transform_returns_fresh_wrapper_without_changing_input_or_stack ... ok
test secret_string_transform::host_secret_transform_result_survives_full_gc_without_input ... ok
test secret_string_transform::host_secret_transform_rejects_nonsecret_and_nonstring_values ... ok
test secret_string_transform::host_secret_transform_tainted_call_keeps_taint_and_lua_opacity ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 479 filtered out; finished in 0.00s

[1m[92m   Compiling[0m rilua v0.1.21 (/home/osso-test/.worktrees/rilua-secret-string-transform)
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 2.48s
[1m[92m     Running[0m tests/integration.rs (target/debug/deps/integration-6ca9a24e5be5ea00)


At 70625a96e1556c45845fb4d030f37af3ef896877: `cargo test -j 4 --test integration table_security` exit 0.

running 24 tests
test table_security::secret_boolean_stdlib_truth_checks_guard_tainted_callers ... ok
test table_security::secret_nil_equality_and_table_reads_reject_tainted_callers ... ok
test table_security::secret_boolean_comparator_results_and_ordinals_enforce_caller_security ... ok
test table_security::host_secret_booleans_keep_taint_and_secure_boolean_control_flow ... ok
test table_security::secret_table_reads_use_lua_indexing_and_return_plain_fields ... ok
test table_security::host_secret_number_and_string_preserve_taint_and_payload_guards ... ok
test table_security::secret_numbers_order_securely_without_exposing_other_operations ... ok
test table_security::table_security_core_rejects_tainted_and_secret_key_access_cumulatively ... ok
test table_security::table_security_reused_arena_slots_drop_restrictions_and_secret_payload ... ok
test table_security::table_security_flags_do_not_survive_collected_table_slot_reuse ... ok
test table_security::table_security_frozen_wrapper_graph_keeps_private_payload ... ok
test table_security::table_security_registration_is_opt_in ... ok
test table_security::table_security_gc_keeps_payload_alive_only_while_wrapper_is_rooted ... ok
test table_security::table_security_options_validate_and_return_no_values ... ok
test table_security::table_security_secret_wrappers_preserve_unwrapped_keys_and_graph ... ok
test table_security::tainted_closure_cannot_order_secret_numbers_even_through_secure_call ... ok
test table_security_stdlib::captured_iterators_recheck_tainted_callers ... ok
test table_security_stdlib::host_functions_using_handles_cannot_bypass_caller_security ... ok
test table_security_stdlib::raw_secret_keys_are_rejected_but_unwrapped_keys_work ... ok
test table_security_vm::vm_security_checks_proxy_before_metamethod_and_redirected_target ... ok
test table_security_vm::vm_security_rejects_tainted_existing_missing_and_numeric_access ... ok
test table_security::tainted_code_cannot_inspect_secret_booleans_even_by_alias_identity ... ok
test table_security_stdlib::protected_tables_reject_tainted_stdlib_access ... ok
test table_security_vm::vm_security_rejects_secret_keys_and_allows_unwrapping_proxy ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 460 filtered out; finished in 0.01s

[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 0.10s
[1m[92m     Running[0m tests/integration.rs (target/debug/deps/integration-6ca9a24e5be5ea00)


At 70625a96e1556c45845fb4d030f37af3ef896877: `cargo test -j 4 --test integration secret_string_formatting` exit 0.

running 6 tests
test secret_string_formatting::secret_format_precision_preserves_full_payload ... ok
test secret_string_formatting::secret_format_width_and_alignment_add_no_padding ... ok
test secret_string_formatting::secret_format_public_controls_keep_width_and_precision ... ok
test secret_string_formatting::secret_format_inferred_tainted_operation_preserves_guards_and_stack_taint ... ok
test secret_string_formatting::secret_format_mixed_arguments_and_gc_retain_typed_result ... ok
test secret_string_formatting::secret_format_unused_secret_input_still_marks_output ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 478 filtered out; finished in 0.00s

[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 0.10s
[1m[92m     Running[0m tests/integration.rs (target/debug/deps/integration-6ca9a24e5be5ea00)


`cargo fmt --check`: exit 0. `cargo build -j 4`: exit 0.
Manual changed-code readability and contract review: owned callback bytes; authenticated private payload validation; no guard bypass added to unwrap; single temporary payload root; caller stack top restored. No findings. Existing table-security tests: 24/24 before and after. Existing secret-formatting tests: 6/6 before and after. No existing-suite failures observed in this targeted scope. Full suite not run.

## Final report

Commit: `70625a96e1556c45845fb4d030f37af3ef896877` on `secret-string-transform`. Required co-author trailer included. Not pushed; worktree clean.

Files changed:
- `src/table_security.rs`
- `tests/helpers/secret_string_transform.rs`
- `tests/integration.rs`
- `docs/src/api.md`
- `docs/specs/table-security.md`
- `CHANGELOG.md`

Verification at committed revision:
- `cargo test -j 4 --test integration secret_string_transform`: 5 passed, 0 failed.
- `cargo test -j 4 --test integration table_security`: 24 passed, 0 failed.
- `cargo test -j 4 --test integration secret_string_formatting`: 6 passed, 0 failed.
- `cargo fmt`: exit 0; `cargo fmt --check`: exit 0.
- `cargo build -j 4`: exit 0, no warnings.

Baseline 6044544 ran the identical existing table_security and secret_string_formatting commands: 24 and 6 passed respectively. No existing failures with or without the change in this targeted scope. Full suite not run. RED invocation of the new tests failed because the requested public API did not yet exist; final invocation passes.

Covered: transformed bytes via trusted untainted unwrap, fresh result wrapper and unchanged input identity/secrecy, tainted host invocation preserving exact Lua stack taint and stack top, tainted output opacity, rejected ordinary strings/secret numbers/non-userdata/ordinary userdata, invalid-input callback not invoked, binary bytes/NUL/empty/identity transforms, full-GC survival after releasing input. No Lua global registered.
