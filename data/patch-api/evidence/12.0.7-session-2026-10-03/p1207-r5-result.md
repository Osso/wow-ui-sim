# Retail 12.0.7 r5 integration

Active goal: integrate staged B11/B25/B26 party roles, leadership/uninvite, solo GROUP_FORMED and /tm ~marker in isolated p1207-r5. Completion: exact anchors, behavioral RED/GREEN, intersecting targeted controls, fmt/check and committed slice. Exclusions: vendor/cache edits, coverage JSON, agents/models, push/merge/deploy/PR, unfiltered/alternate-profile/startup runs.

## Progress
- Step 1: full handoff and author helpers read; helpers reference canonical repo, so not executed. Worktree clean at 63b32995d0a7d5336fac89866dc73f4902161297.
- Step 2: all E01–E25 anchors matched exactly once, disjoint; no master-movement adaptation. RED includes E01/E02 state plus staged tests/spec only. E03 declarations and existing test-support deferred to GREEN: tests use public APIs, so unwired modules are unnecessary and would generate dead-code warnings. All producers withheld.
- Step 3 RED proof: requested local integration command, party_1207_audit::, base63b32995d + state/tests only: 0 passed / 11 behavioral failures, exit101; log r5-RED-party_1207_audit.log. No compile/link failure. Temporary unread latch warning expected until producer is wired; no suppression. RED proof intentionally superseded by GREEN producers.
- Step 4: applied remaining E03–E25 against unique anchors; copied staged role and solo modules. All-argument authentication precedes validation; inferred policies labeled in producers/spec. No retired raw native reintroduced.
- Step 5: cargo fmt exit0; coherent implementation committed as 68572cce2 before GREEN verification. No source changes since commit; RED no longer covers current producer revision.
- Step 6 GREEN at68572cce2: 10 passed / 1 failed. Macro assertions passed; full cached SecureTemplates executes, but test assumed SECURE_ACTIONS is global (nil). Investigating vendor lexical dispatch; no shim/substitute. Warning: parent namespace borrow imports now unused under 12.0.7, will cfg-gate them. Log r5-GREEN-party_1207_audit.log.
- Step 7 adaptation: cached SECURE_ACTIONS is local (SecureTemplates.lua:260); changed test to public SecureActionButton_OnClick with type/unit/useOnKeyDown attributes, exercising original lexical raidtarget handler through its real dispatcher. No vendor changes/approximations. Unused borrow imports cfg-gated to older epochs; formatted and committed before re-verification.
- Step 8 GREEN at74e6e9067: 11 passed / 0 failed, no warnings; actual unmodified cached secure dispatcher/action passed. Log r5-GREEN-party_1207_audit-dispatch.log. Narrow controls below are first invocations on unchanged producer revision.
- Control group_verbs:: at74e6e9067: exit0; test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10373 filtered out; finished in 1.57s; log r5-GREEN-group_verbs.log.
- Control admin_party_api:: at74e6e9067: exit0; test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 10356 filtered out; finished in 3.61s; log r5-GREEN-admin_party_api.log.
- Control patch_12_0_7_removed_native_surface:: at74e6e9067: exit0; test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 10354 filtered out; finished in 11.63s; log r5-GREEN-patch_12_0_7_removed_native_surface.log.
- Control unit_relation_probes:: at74e6e9067: exit0; test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 10365 filtered out; finished in 2.34s; log r5-GREEN-unit_relation_probes.log.
- Control mouse_tm_commands:: at74e6e9067: exit0; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10379 filtered out; finished in 0.64s; log r5-GREEN-mouse_tm_commands.log.
- Control c_party_info_probes:: at74e6e9067: exit0; test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10373 filtered out; finished in 1.33s; log r5-GREEN-c_party_info_probes.log.
- Control delve_instance_state:: at74e6e9067: exit0; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10379 filtered out; finished in 0.82s; log r5-GREEN-delve_instance_state.log.
- Control integration chat_lockdown_ready_checks:: at74e6e9067: exit0; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10378 filtered out; finished in 2.69s; log r5-GREEN-chat_lockdown_ready_checks.log.
- Control lib startup_globals::test_patch_12_0_7_safe_global_bridges at74e6e9067: exit0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1974 filtered out; finished in 0.14s; log r5-GREEN-startup_globals-test_patch_12_0_7_safe_global_bridges.log.
- Step 9: all intersecting integration controls passed (120 cases); changed safe-bridge lib test passed (1). Spec and existing wiki updated with bounded scope, actual dispatcher adaptation and native/server gaps. Only Rust change after proof is test module documentation; behavior unchanged.
- Step 10: documentation/comment-only commit df5e59a0a; prior behavioral proof remains valid (producer bytes unchanged). Manual Rust readability review: shared authenticate/validate/mutate helpers, short borrow scopes, no suppression; token resolver and conditional-marker dispatcher exceed preferred 30-line guidance but stay shallow. No speculative refactor added. Final fmt/check is first invocation for this final integration scope.
- Final fmt-check atdf5e59a0a: exit0; log r5-GREEN-fmt-check.log.
- Final cargo-check atdf5e59a0a: exit0; log r5-GREEN-cargo-check.log.

## Final integration state

- Clean branch `p1207-r5`, HEAD `df5e59a0a43adf161b9eeb46e4c06532c61062c4`; base `63b32995d0a7d5336fac89866dc73f4902161297`.
- No push, merge, deploy, PR, model CLI or subagent. Canonical checkout untouched. Vendor paths and page coverage JSON unchanged. Cached Lua read only.
- Final `cargo fmt --check` and `CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --jobs 4`: exit0, no warnings. No linking cleanup needed.
- Correction to Step9 arithmetic: existing integration controls total **115**, not120; plus11 new +1 lib = **127 final passing cases**. Preliminary GREEN10/1 is historical, not current failure.

### Commits

- `df5e59a0a43adf161b9eeb46e4c06532c61062c4 Record bounded party slice proof and native gaps`
- `74e6e906734e015acb66fa52fd5941d3de59dd6d Exercise cached raidtarget action through secure click dispatcher`
- `68572cce22c8ed35ddb47f8ba30fdeb197bbbabf Model 12.0.7 party roles, solo formation and conditional markers`

All three commit messages end with requested Opus co-author attribution.

### Changed files

- `docs/specs/party-12-0-7-audit.md`
- `docs/wiki/index.md`
- `docs/wiki/investigations/patch-12-0-7-api-audit.md`
- `docs/wiki/log.md`
- `src/c_api/c_party_info.rs`
- `src/c_api/c_party_info/roles_1207.rs`
- `src/c_api/c_party_info/solo_1207.rs`
- `src/loader/tests/wow_api_globals/startup_globals.rs`
- `src/lua_api/globals/group_queries_relationships.rs`
- `src/lua_api/globals/group_verbs.rs`
- `src/lua_api/globals/spell_macro_verbs.rs`
- `src/lua_api/on_update.rs`
- `src/lua_api/state.rs`
- `src/lua_api/state/sim_state.rs`
- `tests/group_verbs.rs`
- `tests/party_1207_audit.rs`
- `tests/unit_relation_probes.rs`

## Proof ledger

Every test used explicit worktree cwd, `CARGO_BUILD_JOBS=4`, `BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts`, default features, exclusive b100 target and serial test execution. One filter per invocation; commands captured once then saved logs inspected. No concurrent builds.

Exact integration command template:
`python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b100 --test integration FILTER -- --test-threads=1`

Lib invocation substitutes `--lib FILTER` for `--test integration FILTER`.

| Phase/filter | Revision/scope | Pass | Fail | Log |
|---|---|---:|---:|---|
| RED `party_1207_audit::` | base +E01/E02 state +tests; all producers withheld | 0 | 11 | r5-RED-party_1207_audit.log |
| first GREEN `party_1207_audit::` | 68572cce2 | 10 | 1 | r5-GREEN-party_1207_audit.log |
| final GREEN `party_1207_audit::` | 74e6e9067 | 11 | 0 | r5-GREEN-party_1207_audit-dispatch.log |
| `group_verbs::` | 74e6e9067 | 11 | 0 | r5-GREEN-group_verbs.log |
| `admin_party_api::` | 74e6e9067 | 28 | 0 | r5-GREEN-admin_party_api.log |
| `patch_12_0_7_removed_native_surface::` | 74e6e9067 | 30 | 0 | r5-GREEN-patch_12_0_7_removed_native_surface.log |
| `unit_relation_probes::` | 74e6e9067 | 19 | 0 | r5-GREEN-unit_relation_probes.log |
| `mouse_tm_commands::` | 74e6e9067 | 5 | 0 | r5-GREEN-mouse_tm_commands.log |
| `c_party_info_probes::` | 74e6e9067 | 11 | 0 | r5-GREEN-c_party_info_probes.log |
| `delve_instance_state::` | 74e6e9067 | 5 | 0 | r5-GREEN-delve_instance_state.log |
| `chat_lockdown_ready_checks::` | 74e6e9067 | 6 | 0 | r5-GREEN-chat_lockdown_ready_checks.log |
| lib `startup_globals::test_patch_12_0_7_safe_global_bridges` | 74e6e9067 | 1 | 0 | r5-GREEN-startup_globals-test_patch_12_0_7_safe_global_bridges.log |

Final documentation/test-comment commit did not invalidate behavioral scopes above. Final fmt/check ran on df5e59a0a. Known pre-existing lib duration, c_api_surface classification and c_system_api console failures were not invoked or modified; no full-suite claim.

## Adaptations

1. Every E01–E25 OLD anchor matched original worktree exactly once and disjoint; no master-drift adaptation.
2. RED omitted module declarations/unwired producer files and old test-support changes because public tests only need fields; avoids unnecessary unwired functions. E01/E02 state supplied compilation; remaining anchors applied GREEN.
3. Staged secure-action test assumed global SECURE_ACTIONS. Real cached file declares it local; switched to its public SecureActionButton_OnClick dispatcher with actual frame attributes. Full file remains unmodified and loaded in its entirety, no snippet/shim.
4. Producer routing made parent borrow imports unused under current epoch; cfg-gated imports for older epochs, not warning suppression.
5. Spec/test authorship-only notes replaced with actual bounded proof and wiki links. No source-row ledger promotion.

## Exclusions / empty-default compatibility

No candidate excluded because an empty default was observed to break cached consumers. Static call-site inspection in handoff: empty role sets preserve false defaults; false restrictions permit normal cached calls; home membership retains existing roster rather than substituting empty category map; missing solo payload emits nothing rather than invented nil/GUID events. Full cached marker consumer executes in test. Other whole-cache/startup acceptance remains unproved and was not authorized to run.

Excluded: native/historical build68182 parity; automatic permissions, exact native errors/timing, stable GUID identities, non-home roster service, ! marker syntax, malformed-prefix native coercions, secure-click protected authorization, SetRaidTarget secret-policy changes, arbitrary aliases/casefold, all-profile checks and server join integration. Ready-check producers unchanged; only inherited bounded lockdown controls rerun. No new fallback/shim or invented secret policy. INFERRED policies remain labeled in spec and producers.

## Per-source-row recommendations

Exact IDs verified against `data/patch-api/sources/12.0.7-page-coverage.json`; JSON unchanged. Recommendations describe integration/development evidence, not independent/native approval. New eight rows remain partial-development-green because unresolved obligations are explicit; ready checks and migration stay audit-pending with bounded inherited/control observations.

| Source ID | Proven scope | Unproved scope | Recommended status |
|---|---|---|---|
| `global api-C_PartyInfo-ConfirmReadyCheck-038` | Inherited lockdown atomic rejection/unlock and existing ready lifecycle; 6 ready controls + safe bridge. | AllowedWhenUntainted authentication/extras, native restrictions/errors, authenticated historical cache. | audit-pending |
| `global api-C_PartyInfo-DemoteAssistant-039` | Public Lua individual removal/exclusion, live role queries, all declared/extra secret positions, taint/root preservation, host restriction rejection/recovery. | Native permission derivation, role/event policies, name/case/realm semantics, historical epoch and alternate profiles. | partial-development-green |
| `global api-C_PartyInfo-DoReadyCheck-040` | Inherited lockdown atomic rejection/unlock and existing ready lifecycle; no new producer. | Native restrictions/errors and argument policy; no new whole-row authentication claim. | audit-pending |
| `global api-C_PartyInfo-IsGUIDInGroup-041` | One public bool; existing home roster; false empty category-2 map then live insertion/removal; category validation, secrets/extras and environment locality. | Stable home GUIDs (existing positional synthesis renumbers), other categories, GUID syntax, native output secrecy/historical identity semantics. | partial-development-green |
| `global api-C_PartyInfo-PromoteToAssistant-042` | Resolved individual promotion; unknown/exact/ambiguous target behavior, live role sets, secrets/extras and explicit restriction input. | Native permissions, name/case/realm/token parity and historical epoch. | partial-development-green |
| `global api-C_PartyInfo-PromoteToLeader-043` | Live existing-member resolution; unknown no-op; public listener sees leader changed before synchronous notification; duplicate no event; secret/extras handling. | Native event timing/authorization, automatic permission derivation, stable identity and unsupported aliases. | partial-development-green |
| `global api-C_PartyInfo-SetEveryoneIsAssistant-044` | Exactly one public updated bool, active toggle/duplicate/solo cases; exclusions clear on change; explicit roles retained; secrets/extras and restrictions. | Native updated/role exclusion semantics, output restriction/permission and historical epoch. | partial-development-green |
| `global api-C_PartyInfo-UninviteUnit-045` | Validate reason/exact selector; one live member removed; cleanup/rebase/local leader; synchronous listener sees new roster; secret/extras and restrictions; raw native remains absent. | Native self-removal, permissions/timing/errors, durable GUID identity and category-2 roster mutation. | partial-development-green |
| `prose-undated-015` | /tm ~n reads actual selected-unit marker map, preserves marked units with no notification; numeric control; full unchanged cached SecureTemplates dispatches set-unmarked twice through SecureActionButton_OnClick; macro input/extras authentication. | Historical-client evidence; native secure-click authorization; SetRaidTarget AllowedWhenUntainted overhaul; !, malformed-prefix/coercion parity and slash UI registration. | partial-development-green |
| `prose-undated-017` | Explicit empty/false/absent host inputs; shared fire_on_update producer; solo Delve/follower with host category/GUID payload; live public listener sees instance state; identical entry once, changed identity/observed exit/reentry, grouped suppression and environment isolation. | Actual server join pipeline, stable formation epoch, GUID syntax/secrecy and unobserved between-tick transitions; invalid category/empty GUID branches inspected but not separately behavioral-tested. | partial-development-green |
| `deprecated-api-177` | Context only: existing removal filter 30/0 and group verbs11/0 prove retired natives stay absent while namespace operations work. | All forwarding-wrapper/loadDeprecationFallbacks migration behavior, strict native removal timing and historical consumer parity. | audit-pending |

## Merge risk

**Moderate, bounded model change; not native parity.** Existing controls and new Lua behavior pass, fmt/check clean, native removals preserved. Intentional semantics change: individual demotion no longer disables everyone mode, unknown leader no longer resets to player, inactive roster mutations no-op, macro text/button validation stricter. Cached call signatures inspected; only marker consumer exercised whole. Role identity is name-based and existing home GUIDs remain positional; solo formation requires host payload and misses unobserved between-tick transitions. A source shared across later Retail profiles receives the 12.0.7-gated policies; alternate epochs and whole startup unverified by instruction. No CI/merge readiness claim.

- Final step: result ledger completed; clean isolated worktree and requested exclusions verified directly. Integration complete within requested bounded scope.
