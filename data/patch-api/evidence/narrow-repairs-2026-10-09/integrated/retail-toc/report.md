# Retail TOC independent bounded proof — PASS

## Scope and revision

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Requested correction: `c5124f4c9366063f4096fa61e72bf2c0a3e31b38` (three test fixture corrections plus documentation/formatting; no production loader/runtime/vendor edits). Initial inspection observed that HEAD, then concurrent docs/evidence-only commit `41f1abbeb83d508b323f51d1f371a3bc506b63b5` landed before proof. Actual formatting, compile, and executions ran at **41f1abbeb**, not immutable c5124f4c9. `concurrent-head-diff.stdout` lists six docs/evidence paths; no code/build inputs changed between those commits. All three changed test files match c5124f4c9 byte-for-byte (`changed-test-commit-comparison.json`).

Proof window: 2026-10-09 17:36:27–17:37:51 UTC. Before/after hashes cover 23,161 tracked existing files and 4,044 Retail cache files: zero changed hashes; HEAD unchanged during proof. Untracked `.code-index.db` existed before proof; no ownership/cleanup claim. `before.json`, `after.json`, and `scope-changes.json` retain status and exact scope. No code/cache/vendor writes, commits, pushes, deploys, network, delegation, alternate profiles, prefork, startup, broad tests, production check, or RED rerun. Saved 3de full-suite RED is caller-owned historical evidence, not independently rerun or credited as fresh proof.

## Commands and results

| Proof | Invocations | Exit | Observed result |
|---|---:|---:|---|
| `cargo fmt --check` | 1 | 0 | Empty stdout/stderr |
| `cargo test --offline --locked --test integration --no-run --message-format=json` | 1 | 0 | Default-feature integration compiled; build-finished success=true |
| `blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain` | 1 | 0 | Exactly 1/1 passed |
| `blizzard_frame_xml_loads::blizzard_frame_xml_toc_is_load_first_with_current_dependencies` | 1 | 0 | Exactly 1/1 passed |
| `blizzard_reforging_ui_loads::retail_toc_selection_excludes_classic_only_reforging` | 1 | 0 | Exactly 1/1 passed |

Each case executed the artifact directly with `--exact --nocapture`, canonical cwd, a 120-second CommandBuilder timeout, and no explicit environment overrides. Each reported 10,697 filtered-out cases; none executed. Actual durations ~0.016 seconds each. Compile took 71.168 seconds; formatting 12.323 seconds. No proof command repeated.

Exact artifact: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`

SHA256: `3395a9d9307b65f28b7cb51f423e7d4a315c3793ecbd3e0ac214497780f8356b` (unchanged after executions). `binary-artifact.json` retains full Cargo artifact, features, filenames and hash.

Full stdout/stderr retained separately for every proof command, with hashes, exact argv/cwd, UTC times, revision, inherited environment key names and scope references in `command-ledger.json`. Environment keys inspected first; no environment values/secret snapshots retained. Build stdout (572,411 bytes) fully parsed: 663 compiler-artifact records, 74 build-script-executed records, one successful build-finished record, zero unparsed lines and zero compiler-message records. Full compile stderr (1,275 bytes) inspected; six manifest deprecations preserved below. `full-stream-inspection.json` records complete stderr and parser totals; `compiler-diagnostics.json` is empty, not a claim that manifest warnings vanished.

## Source expectations and actual cache

Actual FrameXML cache file: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_FrameXML/Blizzard_FrameXML.toc` (generic name; no Mainline-suffixed TOC at the initially guessed path). It contains 17 `## Dep:` declarations; UnitPopup and MirrorTimer have `[AllowLoadGameType classic]`. Retail active types are mainline/standard (`src/toc/mod.rs:59–64`).

Insertion chain: `TocFile::from_file` reads bytes (`src/toc/mod.rs:322`); `parse` sends each `##` line to `insert_metadata` (`:290–300`); `insert_metadata` invokes `is_allowed_game_type` **before** splitting/storing (`:145–148`), strips annotations only for accepted entries, and appends repeated Dep values in encountered order (`:160–167`). `dependencies()` collects those already-filtered metadata lists (`:348`). Thus 17 raw declarations become 15 applicable dependencies in original order. Reading dependencies() alone misses the filtering boundary; the objection attributed to `/tmp/loader-fixture-falsification-current.md` is not applicable to this actual insertion chain.

Observed preserved order: ObjectAPI → FrameXMLBase → UIErrorsFrame → UIParentPanelManager → SettingsDefinitions_Frame → ItemButton → FrameXMLUtil → RaidWarning → UIPanelTemplates → GameTooltip → MoneyFrame → Colors → TransmogShared → LFGUtil → ManagedFrameSystem (each prefixed `Blizzard_`). Both exact dependency tests passed against actual cache bytes.

Reforging directory has only `Blizzard_ReforgingUI_Classic.toc` plus `Classic/`. Actual TOC declares `AllowLoadGameType: classic`. Retail filename tiers are Mainline, generic, Standard (`src/loader/mod.rs:189–192`); `find_toc_file` considers only supported variant names (`:258–265`), not arbitrary Classic fallthrough. Exact finder case observed None. Diff leaves explicit-load and metadata tests unchanged; neither was executed, so no new explicit-load/metadata behavioral claim. Read-only cache/source excerpts and hashes retained in `source-cache-evidence.json`.

## Changed Rust readability and cfg limits

Read verify and rust-readability instructions before proof; manual audit used because `rust-code-analysis-cli` and `readability-audit` are unavailable on PATH (also ripgrep unavailable; no substitute broad audit/check run). Audited all c5124f4c9 changed Rust lines and surrounding functions: literal ordered expectations remain legible, array→string-vector conversion is short, Reforging assertion names its observable policy, and formatting-only edits introduce no behavior. No changed-line readability violations found. No new warning suppressions, opaque conditionals, deep nesting, parameter overload, or mutable state accumulation. Repeated dependency lists belong to independent existing fixtures; this correction adds no duplicated helper implementation. Manual inspection is not measured cognitive-complexity proof.

FrameXML module is client-retail-gated. Reforging finder test newly has `#[cfg(feature = "client-retail")]`; under other profiles this test is omitted, not proof of their selector behavior. Core lane has no module-level Retail cfg in inspected source; this proof nevertheless uses only default Retail. The default-feature integration compile supplies changed-test type proof and all three exact executions establish presence in that artifact. No alternative-profile compile/execution or universal/native loader parity claim.

## Full exact-case outputs

### Case 1

```text

running 1 test
test blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10697 filtered out; finished in 0.00s

```

Stderr: empty.

### Case 2

```text

running 1 test
test blizzard_frame_xml_loads::blizzard_frame_xml_toc_is_load_first_with_current_dependencies ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10697 filtered out; finished in 0.00s

```

Stderr: empty.

### Case 3

```text

running 1 test
test blizzard_reforging_ui_loads::retail_toc_selection_excludes_classic_only_reforging ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10697 filtered out; finished in 0.00s

```

Stderr: empty.

## Full compile stderr (warnings preserved)

```text
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.large-enum-variant` is deprecated in favor of `lints.clippy.large_enum_variant` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.map-entry` is deprecated in favor of `lints.clippy.map_entry` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.match-wildcard-for-single-variants` is deprecated in favor of `lints.clippy.match_wildcard_for_single_variants` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.redundant-closure-for-method-calls` is deprecated in favor of `lints.clippy.redundant_closure_for_method_calls` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.trivially-copy-pass-by-ref` is deprecated in favor of `lints.clippy.trivially_copy_pass_by_ref` and will not work in a future edition
warning: iced-wgpu-patched/Cargo.toml: `lints.clippy.type-complexity` is deprecated in favor of `lints.clippy.type_complexity` and will not work in a future edition
warning: `iced_wgpu` (manifest) generated 6 warnings
   Compiling wow-ui-sim v0.1.0 (/home/osso/Projects/wow/wow-ui-sim)
    Finished `test` profile [optimized + debuginfo] target(s) in 1m 11s
```

**Bounded conclusion:** format and changed-test compile PASS; three exact Retail cases PASS (1/1 each). Main owns evidence retention, full parent goal and acceptance.
