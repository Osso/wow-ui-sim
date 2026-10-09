# Exact startup migration independent proof

## Result: BLOCKED / FAIL at compilation

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Requested commits: `29e48f22501c11df8ed792a1035e644903ed1314` and `911c980fc0eceece074b8ce0c3167a49381fc57c`.

Initial HEAD: `911c980fc0eceece074b8ce0c3167a49381fc57c`. Final observed HEAD: `e119ffa3256a71e0c86f9727d1c108769e29ac93`. Concurrent SOURCE integration changed Cargo.toml only by adding `patch_1_13_6_factory`, requiring `client-era`, within the inspected runtime/build scope. No changes to the five migration Rust files, build.rs, Cargo.lock, prefork runner, or preload. This addition does not select a different default Retail prefork/integration execution scope. Before/after hashes match for every recorded relevant path; the initial working Cargo.toml already contained the concurrent addition. No automatic rerun.

## Commands and execution boundary

Every executed CLI used explicit `.cwd(repo)`, argv-style Pyrun `cli.command`, captured stdout/stderr, and no environment overrides. Exact argv, cwd, timestamps with timezone, exit statuses, and overrides: `commands.json`. Inherited environment: `environment.json`. Full separate streams: `<label>.stdout` and `<label>.stderr`. Relevant hashes: `hashes-before.json`, `hashes-after.json`.

1. `cargo test --offline --locked --test prefork_full_ui --no-run --message-format=json`: exit 101. Compiled the requested default target once; failed before producing a scope-valid executable. Compiler JSON fully parsed; rendered diagnostic saved in `compiler-messages.json`.
2. `cargo fmt --check`: exit 0, no output; run once.
3. `cargo nextest list --offline --locked --test integration`: exit 101. Bounded listing, not suite execution; it failed at the same compilation boundary.

Exact compiler cause, `tests/common/mod.rs:129`:

```text
error: macro-expanded `macro_export` macros from the current crate cannot be referred to by absolute paths
129 | $crate::prefork_full_ui_case! { $($case)* }
= note: #[deny(macro_expanded_macro_exports_accessed_by_absolute_paths)]
(part of #[deny(future_incompatible)]) on by default
```

Diagnostic points to `tests/blizzard_player_spells_loads.rs:209` marker invocation and macro definition at `tests/common/mod.rs:115`. Cargo reports “due to 2 previous errors”; compiler JSON contains one rendered diagnostic for this denied macro condition. Both commands also emit six existing iced_wgpu manifest lint-key deprecation warnings. No suppression or source edit attempted.

## Case observations / proof matrix

| Case | Original body preserved | Actual postfork assertions | Runtime listing |
|---|---|---|---|
| `chat_frame::test_chat_editbox_click_type_and_submit` | Exact text match | NOT EXECUTED | BLOCKED |
| `chat_frame::test_chat_editbox_text_color_after_activation` | Exact text match | NOT EXECUTED | BLOCKED |
| `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix` | Exact text match | NOT EXECUTED | BLOCKED |
| `blizzard_player_spells_loads::mainline_spellbook_keybind_opens_and_closes_without_runtime_errors` | Exact text match | NOT EXECUTED | BLOCKED |

0/4 runtime cases executed; 0 asserted runtime passes. No resulting binary full `--list`, no proven exactly-once prefork names, and no proven absence from Retail integration. No scope-valid old binary substituted. Chat filter scope could not be established through executable listing; no candidate execution launched. Conformance (including new `conformance::exact_fixture_groups_list_each_case_once`) was not executed. No timeout/thread/cache runtime assertions reached. After-process observation contains no `prefork_full_ui-*` processes or prefork zombies; this is not cleanup conformance proof because execution never began. Full process snapshot: `process-after.stdout`.

Non-Retail execution, including Forever aggregate iced builds: unsupported/unverified in this proof; not attempted.

## Artifact and preservation inspection

Read requested spec and wiki system page, migration code, old/new diff, affected fixture constructors/startup helpers, and build registry reachability. Git inventory establishes **five changed Rust files**, not six: `tests/chat_frame.rs`, `tests/blizzard_player_spells_loads.rs`, `tests/common/mod.rs`, `tests/prefork_full_ui.rs`, `tests/spell_casting.rs`; plus two Markdown files. `build.rs` inspected as the sixth Rust boundary dependency, but is not changed by these commits. Evidence: `commit-file-inventory.stdout`, `changes.stdout`, `scope-delta.stdout`.

`preservation.json` records exact byte-for-byte equality of all four original assertion/body ranges after excluding only original outer timeout/constructor scaffolding and new marker/function wrappers. Individual `.old-body`/`.new-body` files preserve compared ranges and hashes. Original source streams: `old-chat_frame.stdout`, `old-spell_casting.stdout`, `old-blizzard_player_spells_loads.stdout`.

All three fixture constructor/startup ranges match old text exactly after ignoring the added `pub(crate)` visibility prefix. Chat manual event sequence omits `UPDATE_CHAT_WINDOWS` and retains original SAY/white assertions. Cast-bar retains original loading, startup events, unlocked initial parent and both anchor transitions. PlayerSpells retains profile-specific FrameXML startup, cleanup restoration, screen-specific events, error clearing and both S dispatches. Error clearing was already present in the original spellbook case, not introduced as normalization. No state reset or weakened assertion added.

Wiring inspected: `prefork_fixture_case!` forwards to the existing emitter; explicit arrays at `tests/prefork_full_ui.rs:195-217` reference the four run functions; `listed_full_ui_cases` combines registries. Execution maps chat/cast-bar/spellbook to unchanged constructors in `run_exact_fixture_group`; separate executable invocations carry `PREFORK_EXACT_FIXTURE_GROUP`. `run_exact_fixture` enters bypass, constructs original fixture, seals memory and supplies inherited read-only child setup through existing runner config. Default generated registry discovery uses `prefork_full_ui_case!`, not the new fixture marker. These are source observations, not runtime proof.

Non-Retail chat/cast-bar wrappers preserve their original constructors and timeout wrappers under `not(client-retail)`; chat retains its enclosing gui module gate. Spellbook wrapper adds Retail exclusion while retaining the original `any(retail-12-1-0, client-wowforever)` applicability gate and original constructor. No non-Retail gates modified during verification.

Changed-Rust readability skill read and applied to changed lines only: no separate readability violations identified. New named group orchestration/functions have bounded responsibilities and explicit subprocess/cache side effects; no new lint/dead-code suppressions. The denied macro path is the actual compilation defect above, not a readability claim.

## Limitations and integrity

No repo edits, commits, reset, operational changes, network, delegation, broad suite, cargo check or reviewer. Two initial Pyrun context persistence attempts failed before executing their intended CLI calls; durable proof helper then used `/tmp/prefork-exact-independent/proof.py`. All actual CLI outputs remain captured without reruns for logs.

Static preservation and format success do not complete the parent migration or its gates. Compile repair and fresh runtime/listing/conformance verification remain required outside this bounded failed proof.
