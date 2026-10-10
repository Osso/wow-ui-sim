# Resumed XML click-registration audit — bounded GREEN verified

2026-10-10. Read-only audit of retained receipts and current source; only this private report written. Read `/home/osso/AgentConfig/skills/verify/SKILL.md`, full rust-readability skill, original `independent-audit.md`, feature diff, literal spec, fixture source and relevant callers. No builds/tests/checks rerun, delegation, operations, commits or vendor/repository edits. Git inspection and hashing only.

## Verdict

**PASS for the bounded default-Retail XML click-registration contract and retained command gates; not warning-free, all-profile, full-suite, startup or native-client acceptance.** Earlier audit's GREEN-blocked finding is superseded by completed `20261010T163159Z` receipts, not by the resource-blocked `20261010T162935Z` attempt. Original audit and original RED remain unchanged.

Repository `/home/osso/Projects/wow/wow-ui-sim`: independently observed HEAD `c6bc87c120e43199dadeb299691511b03a81408a`; implementation `33d62d70546326a1ced19715bd64e47476314f10`. No src/tests/Cargo/build/config differences between implementation and HEAD in inspected scope. Git status contains pre-existing untracked `.code-index.db`; no tracked modification observed. This file is outside the receipts' scope.

## Exact execution counts / original RED comparison

GREEN root: `20261010T163159Z/` beside this report. One module invocation, **nine actual Rust tests**, not one test and not three total. `execution-results.json` records exit 0, reached_tests 9 and artifact_unchanged true. Full stdout independently contains nine named `... ok` records and:

```text
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1987 filtered out; finished in 0.86s
```

All names have prefix `iced_app::mouse::mouse_test_modules::registration_tests::`:

| Test suffix | GREEN | Original exact RED |
|---|---|---|
| xml_register_for_clicks_applies_literal_edges_preserves_default_and_allows_lua_mutation | PASS | FAIL, exit 101; missing right-button receipt, `mouse_registration_tests.rs:60:5` |
| xml_click_templates_replace_inherited_edges_before_onload_and_physical_dispatch | PASS | FAIL, exit 101; base registration absent during instance OnLoad, `:193:10` |
| lua_create_frame_applies_xml_template_click_edges_before_onload_and_dispatch | PASS | FAIL, exit 101; base registration absent during Lua-template OnLoad, `:229:10` |
| left_button_up_fires_mouse_up_before_click | PASS | Not executed in original three-case RED |
| register_for_clicks_any_up_matches_addon_lowercase_spelling | PASS | Not executed in original three-case RED |
| register_for_clicks_left_button_down_fires_click_on_mouse_down_only | PASS | Not executed in original three-case RED |
| register_for_mouse_matches_addon_lowercase_spelling | PASS | Not executed in original three-case RED |
| register_for_mouse_restricts_physical_mouse_button_events | PASS | Not executed in original three-case RED |
| right_click_handlers_see_held_alt_modifier | PASS | Not executed in original three-case RED |

RED root: `/home/osso/.local/state/wow-ui-sim/verification/xml-click-red-current/20261010T162259Z/exact-red/`. Read all three stdout/stderr streams and results.json: each reached exactly one test, `0 passed; 1 failed; 0 ignored; 0 measured; 1995 filtered out`, exit 101. Thus original RED is **three reached failures**, GREEN is **three repaired cases plus six existing controls**. Earlier wrong-selector zero-test records have no behavioral credit.

RED submission revision is `fe04144cf4c9f5d246f3b7947873e7c1e6dcffa1`, not the spec's `fbb502900` label. Inspected git diff establishes equal captured input scope between those revisions; revision identities remain distinct. RED/GREEN snapshots differ only in the four production implementation files below; test bytes did not change. Original RED's first failure stops each case before its later assertions; GREEN completes those assertions too.

## Source / artifact equality — PASS within declared scope

Both GREEN source JSON snapshots have **3,852 entries**, parsed maps equal. Rehashed every recorded current file: **0 mismatches**. Current git-tracked file membership under worker scope also equals the snapshot: **0 additions/removals**. Scope: `src`, `tests`, Cargo.toml/Cargo.lock, build.rs, `.cargo/config.toml`, profile Blizzard manifests and the two specified listfile CSVs. Worker captures before/after compilation; fmt/check receipts also record source_equal true after each command. This is endpoint equality, not continuous mutation monitoring.

Current/test/RED/GREEN `src/iced_app/mouse_registration_tests.rs` SHA256 is identical:
`17d8e5b5864f9502c68da40397b8afddab40e42a4d866970a5edda9c7b34449f`.
Git diff from `da944ee94` to HEAD for this file is empty.

GREEN sealed binary independently rehashed:
`05a02118e5b0933c5f4bc57a3cd6904ebdf3191cb3edb8e68559e3f0aa280362`.
It equals artifact.json; current original `target/debug/deps/wow_ui_sim-ea7724947ed01504` also has that hash. Execution receipt records post-execution equality. RED sealed hash independently matches its original receipt:
`e69d0813d609bbd15c4ab9275178af1e84b21fa57fa0d3f6734b6e2b61394aba`.
These are different RED/GREEN artifacts as expected.

Parsed every line of GREEN compile.stdout: 656 compiler-artifact, 74 build-script-executed, one build-started and one build-finished record; **0 compiler-message records**, build-finished success true. The library test artifact is non-fresh (`fresh:false`), test profile true, and matches the original executable path and feature list in artifact.json. Features explicitly include `default`, `client-retail`, `profile-retail`, `gui`, `casc`, `sound` and Retail version gates through 12.1.0. No other profile runtime credit.

Source equality and binary identity are separately established; this is retained Cargo provenance, not an independently reproducible/hermetic source-to-artifact proof. External dependency source, untracked/index state, inherited environment and non-fixture runtime assets remain excluded.

## Retained compile / fmt / check — PASS with warnings

| Retained command | Result | Diagnostics |
|---|---|---|
| cargo test --no-run --lib --offline --locked --message-format=json --timings -j 12 | exit 0; cargo-result stream_errors [] | Finished test profile in 1m 07s; six manifest deprecations |
| cargo fmt --check | exit 0; revision c6bc87c120; source_equal true | Both streams empty (0 bytes); no failures/warnings |
| cargo check --offline --locked -j 12 | exit 0; revision c6bc87c120; source_equal true | Finished dev profile in 56.24s; six manifest deprecations |
| sealed module execution, timeout 90, --nocapture --test-threads=1 | exit 0; 9/9 | Full stderr contains startup/font timing records; no warning/error/panic diagnostics |

Read complete compile.stderr and default-check.stderr. Each emits the same **six distinct** `iced-wgpu-patched/Cargo.toml` Clippy-key deprecations plus one `generated 6 warnings` summary (not a seventh distinct warning). Deprecated keys: `large-enum-variant`, `map-entry`, `match-wildcard-for-single-variants`, `redundant-closure-for-method-calls`, `trivially-copy-pass-by-ref`, `type-complexity`; suggested replacements use underscores. **Twelve warning-detail emissions across two commands, six unique issues.** No compile/check errors, and no new Rust compiler warnings shown. Manifest untouched by this feature; warnings retained, not suppressed or fixed during this audit.

## Literal requirement coverage

Contract: `docs/specs/xml-button-click-registration.md`, five literal requirements. Its unchecked GREEN/pending prose is stale relative to these private receipts; no tracked documentation acceptance update authorized here.

| Requirement | Evidence / proof level | Remaining boundary |
|---|---|---|
| Direct XML Button registration, comma tokens, whitespace trim, matching physical edges | `src/xml/types.rs:183-185` adds an Option<String> field with serde `@registerForClicks`; `direct.rs:534-557` splits commas, trims and collects into existing registration set. First GREEN test loads real temporary TOC/Lua/XML, hit-tests and calls App left/right down/up handlers; asserts emitted and suppressed OnClick receipts. | Comma forms including space after comma have behavioral proof; single token/arbitrary surrounding whitespace have source support, not dedicated XML cases here. Complete grammar not proved. |
| Inheritance, derived/instance replacement, omission/default, Lua CreateFrame templates | Second GREEN case covers base, derived, omitted leaf/instance and explicit instance replacement; third covers Lua base/derived template construction. First covers no-declaration left-up default and rejected right edges. `xml/template.rs:219-246` establishes base-to-derived chain; helper's last present declaration wins, then instance overrides. | No comprehensive multiple-inheritance or runtime nested-child test matrix; shared callers inspected statically. |
| Registration before OnLoad | Second/third tests observe all four edge predicates during OnLoad. XML `xml_frame.rs:61-71` setup precedes finalize; setup applies properties at `setup.rs:54`, setter call `:363`, runtime OnLoad suppressed at `:131-148`; lifecycle fires `finalize.rs:72`. Lua path applies runtime properties before OnLoad at `template_chain.rs:280-291`; runtime setter wired `runtime.rs:609-611`, child properties before deferred OnLoad at `:157-180`. | Predicate observations are not physical clicks inside OnLoad or new global parent/child lifecycle guarantees. |
| Later Lua RegisterForClicks replaces normally; no persistent XML override | First GREEN case replaces mixed registration with LeftButtonUp then asserts only left-up receipt. Lua setter `buttons.rs:159-166` replaces the same field. XML helper does initialization assignment only. | No exhaustive later-mutation sequence matrix. |
| No vendor/input-policy/public-test-API/fallback change required | Implementation diff: four production files plus spec only. RED→HEAD diff for mouse dispatch/test-module wiring and Interface is empty. Existing registration predicate/default policy remains `mouse.rs:715-755`. Test-only `__xml_click_edges` registered in private fixture, not public production API. | Vendor caches/assets outside fixture scope were not execution-time sealed. No native token-validation policy inferred. |

## Artifact/source and readability audit

[EXIST] PASS — all four production files and real fixture file present.
[SUBSTANTIVE] PASS — parsed declarative field, actual inherited/instance resolution and registration assignment; three behavioral fixture tests, not placeholders.
[WIRED] PASS — XML setup and Lua runtime callers outside helper module shown above; test module reachable at `src/iced_app/mouse_test_modules.rs:16-17` and nine named executed tests prove it ran.
[ANTI-PATTERN] PASS — production added lines and both test commits scanned: 0 TODO/FIXME/HACK/XXX/#[allow] markers; no empty/stub implementation or commented-out replacement code introduced.

Manual changed-line readability: no introduced violation identified. New helper has 17 body lines (540-556), four meaningfully named parameters, shallow nesting, explicit mutation and an absent-declaration early return. Short token transformation and last-present inheritance accumulation are understandable; existing long orchestration functions receive named calls, not new nested logic. Fixture loading exposes filesystem effects; test functions remain under the 200-body-line test threshold. `rust-code-analysis-cli` unavailable on PATH: **no automated complexity-metric claim**. No readability repair or suppression performed.

## Remaining limits

- Bounded default-Retail module proof only: 1,987 library tests filtered out; no full suite, other profiles, full Blizzard/addon startup, live GUI/rendering or deployment proof.
- No native WoW/historical-client parity, exhaustive grammar, empty/invalid-token contract or new mouse-policy claim. Empty declaration currently produces an empty registration set and thereby existing default behavior; source fact, not native acceptance.
- Tracked endpoint snapshots and retained Cargo receipt do not seal external dependency source, inherited environment, untracked inputs or runtime caches; no execution-time whole-input seal or reproducible build attestation.
- Six unique manifest warnings remain. Tracked spec acceptance boxes remain unchanged/pending; earlier resource-blocked epoch remains non-evidence for GREEN.

**Conclusion:** all three unchanged intentional RED cases now reach PASS, six existing controls also PASS, and retained fmt/check gates PASS at source-equal c6bc87c120. Bounded acceptance established; broader/native/whole-input acceptance remains unproven.
