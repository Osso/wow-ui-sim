# Party-map runtime receipts — final PASS

Verified 2026-10-10. Read `/home/osso/AgentConfig/skills/verify/SKILL.md`. Read-only receipt/source/artifact inspection; no builds, tests, checks, delegation, operations, commits, or working-directory changes. Only this report written.

**PASS for the requested default/retail party-map receipt gate:** 81 actual named tests passed; subsequently supplied fmt/default-check receipts both exit 0 with source equality. This is not a claim that the original GREEN controller completed successfully.

## Evidence locations

- GREEN: `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z`
- RED: `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z`
- Repository: `/home/osso/Projects/wow/wow-ui-sim`
- Separate correction: `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/controller-count-correction.json`
- Original controller error: `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/controller.stdout`

## Completed tests and controller boundary

| Receipt | Actual named successes | Result | Execution exit |
|---|---:|---|---:|
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/map-probes.stdout` | 30 unique `c_map_probes::` names | 30 passed; 0 failed; 0 ignored; 0 measured; 10673 filtered out | 0 |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/map-api.stdout` | 51 unique `c_map_api::` names | 51 passed; 0 failed; 0 ignored; 0 measured; 10652 filtered out | 0 |

Independently compared all actual `test NAME ... ok` lines against the named `#[test] fn` declarations in `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_probes.rs`, `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_api.rs`, and `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_api/texture.rs`. Exact set equality for both selections: no missing, unexpected, or duplicate names. Total **81/81 actual cases**. Full runtime stderr logs inspected: startup diagnostics, no panic/error lines.

Both the unchanged original positive case `c_map_probes::get_player_map_position_uses_independent_active_party_member_input` and separate control case `c_map_probes::get_player_map_position_party_input_controls_and_roster_reset` occur as actual `ok` entries, not merely declarations.

Original `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/execution-results.json` still records API `expected_tests: 49`, `reached_tests: 51`, execution exit 0. Controller records `Error: expectedexactdefinedCMapcontrolcount`; it aborted before fmt/default-check and final outcome writing. No original successful worker-outcome claim is warranted. Current `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/worker.py` expects 51; that current script is not evidence of the originally executed expectation.

### Exact two additional API cases

`/home/osso/Projects/wow/wow-ui-sim/tests/c_map_api.rs:5-6` explicitly declares `#[path = "c_map_api/texture.rs"] mod texture;`. Parent file has 49 lexical `#[test]` markers; child file has two ordinary named tests:

- `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_api/texture.rs:4`: `c_map_api::texture::test_create_texture_inherits_template_size` — actual `ok`.
- `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_api/texture.rs:43`: `c_map_api::texture::test_create_texture_applies_sublevel_argument` — actual `ok`.

These are child-module tests selected by the `c_map_api::` prefix. No parameterized/macro expansion mechanism is needed or asserted. Correction JSON's wording “expandedRusttestcases” is less precise than the inspected source evidence. Original count receipts remain separate from correction JSON.

## Source, artifact, and profile binding

GREEN compile-result and cargo-result record exit 0, source equality true, empty stream errors. Before/after source maps contain **3853 identical entries**; freshly recomputed hashes of all 3853 current captured files equal GREEN source-after exactly. Observed HEAD and GREEN submission/check revision: `d25c1fdb444c2dd1c8b0c45dd4f8032b1ca75289`.

Read-only git comparison against producer `40f3801705e2d930dc34c2716e8011234dd67295` exits 0 with empty diff for `/home/osso/Projects/wow/wow-ui-sim/src/c_api/c_map.rs`, `/home/osso/Projects/wow/wow-ui-sim/src/lua_api/game_data.rs`, and the three test-source paths above.

Independently recomputed sealed-executable SHA256:

- GREEN `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/integration-sealed`: `6bf26583ca0c623df5a0cdc66044b21ea8553c9bfd2e38358d79b56b8d9d66df`.
- RED `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/integration-sealed`: `6c356fff5eda73aad513483222c2c9a425aa7341832f7372bb32e6d2c8127475`.

Each equals its artifact receipt; each execution receipt records `artifact_unchanged: true`. Both compiler-artifact records match their receipt's original executable path and exact feature array. Shared compiler test profile: opt_level `1`, debuginfo `line-tables-only`, debug_assertions true, overflow_checks true, test true. Feature arrays include `default`, `client-retail`, `profile-retail`, `gui`, `sound`, `casc`; no alternate-client claim. Hash recomputation establishes present artifact identity; execution-time before/after equality is supplied by the preserved execution receipts.

## Original RED bytes and genuine failure

RED submission revision: `439260ee585b44f1d07c79ed4ce2ea77649f1ead`; compile exit 0, source-before/source-after equal across 3853 entries. Exact positive-test execution exits 101: `running 1 test`, `0 passed; 1 failed`; stderr identifies `/home/osso/Projects/wow/wow-ui-sim/tests/c_map_probes.rs:236:6`, `active party1 position must be present`.

Read original test-file bytes using git show at the RED submission revision. SHA256 `41420ba945568c0833a3e20fd7836220586f1980d4de06df694fb5ec8097d3e2` equals the preserved RED source-map entry. Original positive function, signature through closing brace excluding attributes, is byte-identical to current source; SHA256 `6326ed808797d1fdc0d8f9650da6e36e7295dd2869c9e0125728546e68c507c0`. No positive-test weakening between RED and GREEN.

Present original RED receipt bytes fingerprinted, not rewritten:

| Absolute path | SHA256 |
|---|---|
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/module.stdout` | `01e6651049fd59c2c2caea6850fe6a59b8cc293424b4aca3b44372c2fffd3c3c` |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/module.stderr` | `30b6f4ba2ab4ad6c245d57d63c478c8bc8b20d589ca89ae89b374cd0dc7b60da` |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/execution-results.json` | `9487eca993c90e57a019e04578abbb1366bfa3383c96e21661d3ffe8afcab142` |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/source-before.json` | `4a9b69487f394be4569fa155ce0857fd71b4318c63115eccebe1b33efe70cc91` |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z/source-after.json` | `4a9b69487f394be4569fa155ce0857fd71b4318c63115eccebe1b33efe70cc91` |

Historical immutability of every receipt cannot be independently established without prior receipt fingerprints; this inspection establishes current consistency, original stage-test bytes, and makes no changes to originals.

## Previously missing checks — now available

| Absolute receipt path | Recorded command | Exit | Source equality |
|---|---|---:|---|
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/checks/fmt.json` | `/usr/bin/cargo fmt --check` | 0 | true |
| `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/20261010T181344Z/checks/default-check.json` | `/usr/bin/cargo check --offline --locked -j 12` | 0 | true |

Both check revisions equal GREEN submission. Full stdout/stderr inspected. Fmt output empty. Default check finishes `dev` profile in 28.50s. `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/checks-controller.stdout` records completion, with initial source equality assertion and per-check equality receipts. These are later missing-check receipts, not rerun test receipts or retroactive original-controller success.

Default check and both compilation logs contain six `iced-wgpu-patched/Cargo.toml` deprecated hyphenated Clippy-key warnings, plus the summary `iced_wgpu` manifest warning. **Not warning-free.** No warnings suppressed or files changed here.

## Scope conclusion

**Final PASS:** requested 30 probe + 51 API actual tests, genuine original RED, unchanged original positive-test body, sealed artifact/profile correspondence, captured-source equality, and later missing fmt/default-check receipts. No checks pending at inspection.

Receipt exclusions retained: external dependency/source-to-artifact provenance, untracked files/index, inherited environment, and runtime assets outside fixture inputs. No native coordinate acquisition, projection, all-profile, full-suite, or whole-HEAD acceptance claim. Earlier `/home/osso/.local/state/wow-ui-sim/verification/party-map-green-current/independent-report.md` remains an untouched historical pending report; this report supplies the later runtime evidence.
