# Independent bounded controller verification — PASS

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Verified revision: `96e88494a6844833b79a574d37a2115dc03b44eb`. Implementation: `6e6f8cb5b`; correction/build: `2de36e2b36a7643c53864b1bfddb317e206b9528`. No source edits, rebuilds, delegation, network, operational changes, cwd switching, or async-full-suite polling.

## Fresh command proof

Exact executable was independently selected from the final development compile's `compiler-artifact` JSON in `/tmp/prefork-group-controller-dev/green-fix-build.stdout`, not inferred from a filename or selected from a directory listing. Compile result receipt exits 0 and identifies correction revision and input hashes.

Command: `timeout 90 /home/osso/Projects/wow/wow-ui-sim/target/debug/deps/prefork_full_ui-21dd5c807fe7a7b6 conformance::exact_group_ --test-threads=1`.
Explicit env only: `PREFORK_CONFORMANCE_SUITE=1`; other environment inherited, not dumped. UTC start `2026-10-09T21:02:34.640102+00:00`; end `2026-10-09T21:02:39.214120+00:00`. Exit 0; elapsed receipt interval 4.574018 seconds.

Full stdout:

```text
running 3 tests
test conformance::exact_group_selection ... ok
test conformance::exact_group_failure ... ok
test conformance::exact_group_timeout ... ok

test result: ok. 3 passed; 0 failed; 3 total
```

Full stderr: empty. Streams: `suite.stdout`, `suite.stderr`. `start.json` and `result.json` capture cwd, argv, required explicit env, revision before/after, UTC, exit, complete compiler-artifact message, input hashes and artifact/source hashes before/after.

Executable SHA256 before/after: `98130da61450a476eb81dcd353a2ce3839d63fa97730d25cd1cc421f65717547`.
Source SHA256 before/after: `d3414b70ad02c2cf5cd73e971cfdfc8053b9e786c20df645ee2e30414ef85df0`.
Both match supplied development identities. Cargo.toml, Cargo.lock and build.rs also match final compile receipt. `git diff 2de36e2b3 96e88494a -- :(exclude)docs/**` is empty; `git diff --exit-code 96e88494a --` exits 0, empty streams. Correction-to-current differences are documentation only. HEAD unchanged across suite. Initial status contained only untracked `.code-index.db`; no tracked changes.

## Behavioral coverage matrix

All three enabled groups are compiled: chat (`gui`), cast-bar, spellbook (`retail-12-1-0`); features independently read from final compiler-artifact message.

| Contract | Observed proof | Coverage |
|---|---|---|
| Exact selection | Exactly one selected deterministic pass line; only selected group records sealed setup | chat/cast-bar/spellbook |
| Skip/unmatched, no setup | Successful zero selections; no group setup marker and no isolated cache directory | each group, both variants |
| Panic aggregation/continuation | Deliberate panic text, captured stdout/stderr, nonzero controller; all nonfailing groups succeed, all setup PIDs distinct | each group chosen as failing |
| Timeout tree cleanup | 120ms timeout failure; direct prefork child and descendant PIDs both become ESRCH | each group |
| Disk preservation/fresh process | Sentinel tree snapshot equal after timeout and fresh success; new setup PID, seal result zero, successful selected child | each group |

Three outer tests exercise 18 controlled controller subprocess invocations: 9 selection/zero-selection, 3 failure, 6 timeout/fresh-success. These counts follow inspected loops, not independently persisted inner-command receipts.

## Actual wiring and preservation

`tests/prefork_full_ui.rs:280-330`: controlled commands remove suite/driver/exact dispatch flags and enter real main -> shared workload lock -> `run_full_ui_and_exact_fixtures`. Its eagerly evaluated results array executes all groups before aggregation. Each uses unchanged `run_exact_fixture_subprocess` (same executable, original argv, exact-group env).

`:346-399`: only private controlled branch replaces group cases/constructor and shortens controlled timeout. Normal chat/cast-bar/spellbook constructor mapping and feature gates remain unchanged. `:432-463`: real `run_exact_fixture` -> lazy `prefork::run_with_setup` -> parent bypass -> constructor -> real cache sealing; controlled mode skips recursive conformance and writes PID/seal markers. Normal mode retains recursive conformance, 120-second config and original fixture construction.

`tests/common/prefork.rs:139-166` selects before setup and returns early for zero selections. `:464-467` invokes the normal child read-only cache hook before the case body; `:595-607` uses original TERM/KILL timeout enforcement. Cache sealing remains actual `src/loader/bytecode_cache.rs:70-95`, rejecting populated/initialized state and sealing empty bypass state.

Diff against `cbef795b2` changes only target root and two docs. Entire chat_frame.rs, spell_casting.rs, blizzard_player_spells_loads.rs, runner and bytecode_cache.rs are unchanged. Inspected original constructors and all four bodies/wrappers: chat manual startup without UPDATE_CHAT_WINDOWS, SAY/white assertions; cast startup and unlocked/locked anchor transitions; spellbook profile-aware loader and S-toggle assertions. Existing non-Retail wrapper gates remain unchanged. Saved full target-root diff: `source.diff`.

## Manual readability / format

Read both required skills, spec, wiki index entries/system section, development report, changed Rust lines and relevant actual callers/cache/runner/fixture bodies. Manual semantic readability audit found no actionable violations in changed code: effectful boundaries explicit, small helpers, deterministic case names, named env constants, test functions below 200 lines, no deep new nesting or new warning suppressions. Added-line TODO/FIXME/HACK/XXX/suppression scan: zero. Existing intentional generic fixture no-op is reused, not a new placeholder implementation.

Retained valid format proof `/tmp/prefork-group-controller-dev/fix-format.result.json`: rustfmt exit 0 with exact current source hash. No new format command needed; this is retained formatter evidence, not a new whole-project fmt/check claim.

## Limitations / open gates

PASS applies only to bounded controlled public-controller integration. Constructor is bare WowLuaEnv, not original Blizzard fixture loading. Child inherits the parent boolean; this suite does not exercise actual addon bytecode misses/stores, warm valid packs, or all cache API mutation paths. Disk sentinel is arbitrary bytes, not a parsed production pack. Snapshot includes entries, bytes, length, mtime nanoseconds, inode, mode and directory/file identity; excludes root directory metadata, atime/ctime and ownership. Read-only hook is verified by wiring, not an independent runtime mode assertion.

The four original migrated fixture-body executions remain separate retained proof at `3de874658`; none rerun here. Non-Retail execution remains OPEN. Generic 21-case conformance, Cargo check, broad/native/profile/fixture reruns and full-suite acceptance receive no new credit. Main's newly submitted asynchronous full suite was neither waited on nor polled. No parent-goal completion or performance claim.
