# Saved partial Retail runtime verification — FAIL

Source: `e37f5426a1da8d721b0ab3ff0341877f59100c02` only.
Epoch: `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-producer-green-20261014T173458823763Z`.
Read-only artifact audit under verify/SKILL.md; no builds/tests/reruns/delegation, binary reads/hashes, or canonical edits. Existing digest receipts reused, not freshly rehashed. No current HEAD/native/parent-goal claim.

## Runtime coverage

| Boundary | Evidence | Result |
|---|---|---|
| Eight subtext cases | Complete saved stdout | 7 PASS, 1 FAIL |
| Three unchanged lifecycle controls | Complete saved stdout | 3 PASS |
| Main aggregate | `10 passed; 1 failed; 0 ignored; 0 measured; 11055 filtered out`; launcher exit 101 | FAIL, expected 11 |
| Initial type25 activation | Archived `consumer.rs:39-50`; initial assertions `consumer_assertions.lua:33-43` complete before failed later synchronous phase | PASS |
| In-place update | `consumer.rs:52-65`; panic `tests/widget_two_records/helpers.rs:6:33` | FAIL |
| Hide/reacquire, membership removal, unregister live container | Sequential calls `consumer.rs:155-157` follow failed activation/update call | NOT REACHED / NOT PROVED |

Failing case: `ui_widget_text_with_subtext::consumer::text_with_subtext_cached_type25_normal_activation_update_hidden_shown_remove_unregister`.

Exact saved error: `two-record fixture [in-place update of both visible lines]: Lua error: assertion failed!`
Child exit 101 is propagated by `tests/common/timeout_reexec.rs:184:9`; not evidence of an elapsed timeout.
Update block checks frame identity, updated visible lines/style/geometry/metadata, retained snapshot, and current getter. Error supplies no individual assertion location or values. Exact subassertion and failure cause UNKNOWN; no guess. Passing sibling controls do not prove later type25 lifecycle phases.

## Authentication correction — PASS, metadata/receipt scope

Original producer seals select `artifacts/deps/integration-9f07e4a55f2bbbf7` and `artifacts/wow-sim`; preserved original child receipts instead select the extra copies, previously 0600. Original pairing was path-incoherent, not silently treated as valid.
Correction is recorded in `auth-copy-path-permission-correction.json`. Explicit new `seal-auth-*`/`receipt-auth-*` select exactly:

- `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-producer-green-20261014T173458823763Z/auth-copies-20261014T174115425769Z/integration`
- `<private-state>/verification/widget-discrete-progress-current/widget-subtext-retail-producer-green-20261014T173458823763Z/auth-copies-20261014T174115425769Z/wow-sim`

Both runtime argv arrays use their respective exact authenticated paths. Source/profile/features and recorded digest chains agree with retained original seals/receipts. New seals, receipts and runtime reused stats agree; current stat metadata also agrees. Permission transition is 0600 → 0700; changed fields only `st_mode` and expected `st_ctime_ns`. Device, inode, link count, UID/GID, size and mtime remain stable. No extra binary content/hash read. This proves recorded control coherence and stable metadata, not independent fresh byte authentication or source/cache/library authentication.

## Normal Retail observation — PASS, limited

Saved stdout exactly `[]\n`; JSON `[]`; launcher exit 0.
Full stderr: 219 lines, classified 219 informational/summary lines, 0 warning diagnostics, 0 error diagnostics. Final summary: `Status: CLEAN`; `Lua errors: 0 unique, 0 occurrence(s)`; attributed owners 0, unattributed errors 0. Summary heading containing “error” is not an error diagnostic.

Information includes startup/timing/resource limits, font fallback, read-only WTF EditMode cache, 291 Blizzard addons, 1569/1569 bytecode cache hits, applied post-load workarounds and GC restart. CASC and sound disabled; `--no-addons --no-saved-vars` used. SavedVariables disabled does not mean WTF EditMode cache was unused. No all-addon, rendering, native or global clean-startup claim. Normal zero-Lua-error observation does not make the failing subtext gate GREEN.

`proof.json` records exact paths, controls, argv, all eleven case outcomes, full saved runtime stdout/stderr, and coverage limits.
