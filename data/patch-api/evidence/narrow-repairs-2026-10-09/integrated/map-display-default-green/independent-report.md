# Independent MapDisplay saved-receipt final audit

Date: 2026-10-10. **OVERALL: bounded GREEN for saved MapDisplay producer `0979952073b64581903283c87d38ba2eae29f2b5`: 86/86 targeted executions PASS; publication row PASS. Not whole-P801 GREEN.**

Read `[private-path]` first. Independently read saved receipts/source, recomputed SHA-256 hashes, and manually audited Rust readability and literal security declarations. No command, test, build, check, delegation, operational action, or repository edit executed. Only this exact report was written; no alternative report created. This replaces the prior pending observation, preserving historical failures and acceptance limits.

## 1. Exact evidence root and actual execution

All current receipt names below are relative to EXACT root:

`[private-green-epoch]`

`submission.json` records repository `[repository]`, revision `0979952073b64581903283c87d38ba2eae29f2b5`, and `/usr/bin/cargo test --no-run --test integration --test prefork_full_ui --offline --locked --message-format=json --timings -j 12`. `cargo-result.json`: exit 0, stream_errors []; `compile-result.json`: exit 0, source_equal true. Cargo JSON ends with build-finished success true. `outcome.json`: build_performed true, compile_exit 0, source_equal true, selected_execution_count 2.

| Evidence | Actual result | Boundary |
|---|---|---|
| map-probes.stdout + execution-results.json | 35 PASS / 0 FAIL; reached_tests 35; exit 0 | Five MapDisplay cases and 30 probe controls, same sealed integration executable |
| map-api-controls.stdout + execution-results.json | 51 PASS / 0 FAIL; reached_tests 51; exit 0 | C_Map API controls, same executable |
| lua-errors-result.json + lua-errors.stdout/stderr | exit 0; stdout `[]`; stderr `Lua errors: 0 unique, 0 occurrence(s)` | Stock no-addons/no-saved-vars stored-error CLI; CASC disabled by this mode |
| checks/fmt.json | exit 0; source_equal true | cargo fmt --check at saved revision |
| checks/default-check.json | exit 0; source_equal true | default cargo check --offline --locked -j 12 |
| checks/ptr-check.json | exit 0; source_equal true | cargo check --no-default-features --features sound,gui,casc,client-ptr --offline --locked -j 12; **PTR check, not PTR runtime** |
| p801-sweep-result.json + stdout/stderr | exit 1; 0 PASS / 1 FAIL | Only assertion difference: new gaps []; resolved/stale gaps ["wt-global-api-C_Map.GetMapDisplayInfo-30"] |

Both targeted execution receipts record artifact_unchanged true. Each saved execution argv uses `/usr/bin/timeout 90`, the exact sealed executable, its selector, `--nocapture --test-threads=1`. P801 and stock CLI receipts likewise record artifact_unchanged true. Counts are actual test executions, not static inventory or zero-test success. Exact targeted summaries:

```text
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 10673 filtered out; finished in 2.79s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 10657 filtered out; finished in 3.40s
```

All five actual MapDisplay names report `... ok`: `get_map_display_info_returns_one_supplied_bool`, `get_map_display_info_tracks_updates_with_map_and_environment_isolation`, `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy`, `get_map_display_info_untainted_secret_returns_one_supplied_bool`, and `get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret`.

Compile/default/PTR stderr retain six iced_wgpu manifest deprecation warnings. Successful exit is not warning-free completion; warnings were neither suppressed nor repaired here.

## 2. Independent source/artifact hashes and unchanged-test boundary

Parsed both complete 3,853-entry source manifests. Before equals after; both manifest-file SHA-256 hashes independently recompute to `b32953d0b862b3894b9c49436f90cf49a525b18f869f2c27925abb8d7de1d224`.

Rehashed every current file named in the saved after-manifest: **3,852/3,853 match**. Sole changed path is `tests/data/patch_8_0_1_sweep_known_gaps.json`; no current whole-checkout equality claim. Saved fixture SHA-256: `bad5e7e4e77494f4e506684fb63b97d5b0d8ff9dd8279f714fbf00ad32bac55a`. Current fixture SHA-256: `618f9a1c663f16ce1df4613ff953e69b322e2cd04fe2868ad099189ab669e1b1`; read current JSON contains 16 IDs, excludes MapDisplay, and equals the row JSON's residual non-ok set. Saved checks prove the saved boundary, not this later fixture edit or its subsequent integration acceptance. Main owns fixture closure after independently admitting observed row ok; this audit neither edited it nor ran post-edit P801.

Current relevant file hashes match the saved GREEN manifest:

| Repository-relative file | Recomputed SHA-256 |
|---|---|
| src/c_api/c_map.rs | 2cd9313d7540696840e5b1c90813ea52d85e54bb13fd07859ba8c127fa0bd95c |
| src/lua_api/state.rs | 798a0c52f09e0968e6f6ef02839d598786dc5c8d06d0dceb14e4c74a408ef473 |
| src/lua_api/state/sim_state.rs | 5da06c389ae6441ffbbe45c9acea4bba0f10bf70c710e21b03d62b6553f4d5cf |
| tests/c_map_probes.rs | 72f14ed9a1ed27d1a46b36175f9ba70629d0976ac958e7bf1788a39011cb8021 |
| tests/c_map_api.rs | 65c326cf6d2c0938f28bf55970fde11000bea8428a35601e696b65c8d4a8ef23 |
| tests/c_map_api/texture.rs | a889bec4d49f7f216351411f3619b81642c7626681ad28838252fa0e041b3e46 |
| Cargo.toml | f439cb619c2bff21706e7c0f175cfb63701a0383c1dac732b2308ef759285961 |
| Cargo.lock | 859f258fa21eb92f7f0c83c77d526ea0ce3e7535b0970df2e39c459023c0b9cf |

All three behavioral test-file hashes additionally match the complete saved `map-display-security-red-current/20261010T193717Z/source-after.json`. Thus the security RED-to-GREEN correction did not weaken/change these tests. The prior report's successful comparison to `d80ee1b7f` is retained as prior inspection evidence, not represented as a new Git command. Producer source differs from the security RED snapshot, as expected.

Independently streamed SHA-256 over each saved sealed executable; each matches its artifact receipt:

| Sealed artifact under EXACT root | Recomputed SHA-256 |
|---|---|
| integration-sealed | 69b8729f6594ba8607859c58f5fb5ee8037cdcf8302a2b3999d48c95aeda32d4 |
| prefork_full_ui-sealed | de2baed5988c35baab675a7ea50db22d446c4e562785291fb0c3335c0b75c242 |
| wow-sim-sealed | fc0d2267a91fe28cfc8d1c1ac0e0ad1062aa3336ca51609e52e65b0bf2ca0600 |

Original paths in receipts and Cargo JSON agree: `target/debug/deps/integration-6fa7ee1f035abc08`, `target/debug/deps/prefork_full_ui-d84a52f78dca94c5`, and `target/debug/wow-sim`. Mutable originals were not executed or credited in this audit.

All three Cargo artifact records have fresh false; profile opt_level "1", debuginfo "line-tables-only", debug_assertions true, overflow_checks true. Integration/prefork test true; wow-sim test false. Feature receipts include default, client-retail, profile-retail, retail-12-1-0, sound, gui, casc, prefork-full-ui. Runtime proof is Retail native test-binary execution, **not authenticated native WoW client execution**. Cargo.toml:121-122,151-152 shows Retail reaches retail-12-1-0 and PTR reaches retail-12-1-5, which includes it. Prior epochs alone do not enable this getter.

Toolchain boundary: saved argv uses local `/usr/bin/cargo`, not desktop build-host/rustup. **No actual cargo -V or rustc -Vv receipt exists in this epoch or worker. Exact executed compiler version/host identity remains unsealed.** Environment documentation/version expectations are not substituted for measured toolchain receipts. No toolchain command rerun. External dependencies/source-to-artifact provenance, untracked files/index, inherited environment, and runtime assets outside fixture inputs are explicit submission exclusions.

## 3. Source, wiring, literal declaration/security, manual readability

[EXIST] PASS — actual implementation and behavioral fixtures read; source hashes verified.
[SUBSTANTIVE] PASS — `src/lua_api/state/sim_state.rs:575` represents input as `map_display_hide_icons: HashMap<i32, bool>`; state.rs:494 initializes empty. `src/c_api/c_map.rs:191-227` unwraps an authorized secret before numeric validation, reads supplied input, returns zero values on absence, and one `Val::Bool` on supplied true/false. No DTO, catalog/default derivation, or new Lua setter.
[WIRED] PASS — `c_map.rs:75-76` registers GetMapDisplayInfo; lines 96-101 install its method table; external `src/c_api/registration.rs:115-116` calls register_c_map_surface.
[ANTI-PATTERN] PASS for new parser/helper — no TODO/FIXME/HACK/XXX, suppression, placeholder, fallback, or commented-out implementation.

Directly read both cached literal declarations, retail and ptr `AddOns/Blizzard_APIDocumentationGenerated/MapDocumentation.lua:216-229`: `MayReturnNothing = true`, `SecretArguments = "AllowedWhenUntainted"`, required non-nil number uiMapID, one non-nil bool hideIcons. Neither block declares SecretReturns. Absence of that declaration is not runtime proof of return secrecy.

Current external-source SHA-256, independently recomputed:

- Retail cached MapDocumentation.lua: `9aa2a98f5a02ae1bf047b0c01bf4861804273a35ba0b85a9963a8f8dd851321b`.
- PTR cached MapDocumentation.lua: `b73a2ed1d1552c1b38bc59df0a586a0986dfad54991efc905abc87799cecce57`.
- `[private-path]`: `cf03b2cb2043260233bf6363bbe2f641bc5fa970da66003d3d6b8ea4c56c486c`.

Actual VM source at table_security.rs:401-428: unwrap_secret passes ordinary values through; authentic secret payload requires can_access_secrets; that guard is secure state AND non-revoked context. Getter calls this real unwrap, not typed extraction of a wrapper. These inspected dependency bytes are not sealed dependency-to-executed-artifact provenance.

Read complete rust-readability skill and manually audited parser/getter lines. **No readability violations found**: short one-parameter parser, named exact-integer/range predicates, explicit early errors, shallow branching, no mutable accumulation, duplicated new logic, suppression, or hidden I/O. Getter exposes state read and stack mutation through its API; no unnecessary abstraction. Parser requires finite integral i32 range; literal declaration says number. Fractional/out-of-range native behavior remains unestablished, not proved by these five tests.

Behavioral assertions are substantive: supplied true/false and one boolean, update/map/environment isolation, empty/removal zero arity. Map 85 is expressly a test-only fixture. Both security cases use a rooted genuine wrap_host_secret_number value and authentic addon closure taint. Untainted secret succeeds; tainted ordinary succeeds and tainted secret rejects; taint and wrapper preservation are asserted. These are actual simulator VM executions, not mocked or expected-failure tests.

## 4. Publication-row admission versus whole P801

Read the complete 134,423-byte `p801-sweep-results.json`; recomputed SHA-256 `87c797c01e96de842d3ecd8a58b30bdc55e2aa44ad6aa4d9005f1ad3d46f7985`. Exactly 269 rows: 253 ok true, 16 non-ok.

Exact target entry `wt-global-api-C_Map.GetMapDisplayInfo-30`:

```json
{"expected":{"direction":"added","page_default":null,"publication":"published","section":"global-api","superseded_by":null,"symbol":"C_Map.GetMapDisplayInfo"},"observed":{"default":null,"default_mismatch":false,"detail":"raw=function; lookup=function","kind":"member","value":null},"ok":true}
```

**Row publication PASS establishes member presence/lookup, not its full behavior.** The five separately executed behavior/security cases supply the bounded behavior proof. Saved sweep fails at `tests/common/publication_sweep.rs:315` because frozen 17-gap fixture includes this now-ok row; no new gaps. Current 16-ID fixture matches observed residual set, but no subsequent sweep receipt was admitted. Whole saved P801 remains 0 PASS / 1 FAIL, exit 1. Main owns fixture closure; row PASS must not be relabeled whole-P801 GREEN.


## Retention limits

Sanitized bounded excerpt of independent report; CPU and other epochs are not retained. Historical RED remains historical. Whole saved P801 FAIL; post-fixture-change actual sweep PASS and PTR runtime were pending at this audit epoch. Later ../map-display-publication-green/independent-green-report.md admits1/1 P801 PASS; ../map-display-ptr-green/independent-report.md admits84/84 PTR PASS. Authenticated native WoW execution remains pending. No native unknown-ID, malformed-number, return-secrecy, revoked-context, cache-origin, dependency-linkage, or CASC CLI parity claim. No tests/checks/builds rerun for retention.
