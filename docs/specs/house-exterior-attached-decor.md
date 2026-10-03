# House exterior attached-decor actions

Batch70 bounds EXACT272/274/276: `C_HouseExterior.SelectFixtureOption`, `SetHouseExteriorSize`, and `SetHouseExteriorType` with `attachedDecorAction`. Explicit inputs live in `src/c_api/c_housing/exterior.rs`; runtime transitions and read snapshots live under `src/c_api/c_housing/exterior/`. The [existing housing catalog](housing-catalog-variants.md) remains storage identity/count SSOT. Compiled RED observed 38 failures and one pass before runtime implementation; independent566 accepts bounded simulator behavior, not native parity. B73 adds unchecked removal inputs for EXACT267/268; B70 acceptance does not establish removal behavior.

## What it must do

Checked requirement boxes record bounded simulator proof under explicit inferred policies, not native parity.

### Selections and attachments

- [x] Modern behavior applies only with `retail-12-0-5` and `profile-retail` or `client-ptr`. Earlier/other profiles retain original no-op mutators and seeded queries; inverse controls receive no modern credit. Data structs may exist unconditionally.
- [x] Default exterior state has no selections, options, fixture point, or placements. Existing `inside_owned_plot` and `active_house_editor_mode == 6` supply availability; no extra ownership framework.
- [x] Explicit unlocked size/type/fixture options authorize changes. Type and fixture options also require `is_invalid == false`. Missing selection/options, unknown IDs, unavailable host, and locked/invalid targets reject atomically without inventory changes. Native error versus failure-response mapping is unproved; tests accept an explicit error or one non-Success response, never silent success.
- [x] `Store = 0` removes affected attached placement instances and increments existing full-key catalog variant `num_stored` by removed instance count. No fabricated variants, catalog records, IDs, or aggregate totals. Preserve destroyable counts and all dye fields. Explicit known base totals change by stored +delta / placed -delta; `None` remains an unmodeled aggregate gap.
- [x] `Detach = 1` clears affected `fixture_point_owner_hash` only. Preserve every position coordinate, placement identity, variant identity and storage count. Detached placements float at the same position and never automatically reattach.
- [x] Fixture selection affects only attachments whose owner hash equals the selected point's hash. Size/type changes affect all attached exterior placements. Already-floating placements remain unchanged. Nonaffected point selection/options and storage remain intact.
- [x] Omitted action defaults to Store. Explicit nil also defaults to Store as an **inferred simulator policy**, not observed native coercion.
- [x] Same-size/type requests leave attachments/inventory untouched and synchronously respond AlreadySize42/AlreadyType43 (**inferred emission policy**). Same-fixture request is not replacement, preserves attachments, and responds Success0 (**inferred policy**).
- [x] Missing variant, negative/overflowed stored count, known aggregate overflow or placed-count underflow reject Store before any mutation. Invalid trusted-host inventory handling is simulator policy, not native acquisition parity.

### Security and arguments

- [x] Authenticate both original arguments before type/domain parsing or model access. All three cached declarations specify `SecretArguments = "AllowedWhenUntainted"`: actual typed secret NUM selectors/actions work in secure callers; addon callers cannot use secrets. Public addon arguments work without clearing taint. Do not substitute secure-secret rejection, declassification, fake callbacks, or security overrides.
- [x] Targets are strict positive integral `u32` IDs; size must be a declared option. Action accepts numeric 0/1 only. No string coercion, fractional truncation, nonfinite values or undeclared target acceptance. Wrong-type/domain arguments fail with API/argument-position context, no private payload, no mutation and no event. Exact native errors/coercion remain unproved.
- [x] Arg2 secret authentication wins over arg1 wrong type or absent model target. Authentication failures remain atomic, do not emit responses/storage events, and do not expose secret numeric payloads.
- [x] Publish `Enum.HousingFixtureDecorAction.Store = 0` and `.Detach = 1`. Mutation fixtures use literal numbers so missing enum publication cannot mask actual call-boundary RED.

### Queries and callbacks

- [x] Supporting reads reflect the same explicit state: `GetCurrentHouseExteriorSize`; `GetCurrentHouseExteriorType` returns ID and explicit matching option name; size/type option DTOs; selected fixture point DTO; selection and exterior/selected-point attachment predicates. Nil/missing selections must not reproduce old seeded data. Snapshot mutations/GC must not change host state.
- [x] DTO fields match cached primary structures, including `isLocked`, `isInvalid`, `reasonString`, fixture `typeID/typeName/colorID`, point `ownerHash/selectedFixtureID/fixtureOptions/canSelectionBeRemoved`. Do not confuse current `reasonString` with historical `lockReasonString`.
- [x] Valid changed calls return zero Lua values and synchronously publish the documented one-argument response event with Success0 after all state commits: `HOUSING_SET_FIXTURE_RESPONSE`, `HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE`, or `HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE`.
- [x] Store publishes existing `HOUSING_STORAGE_ENTRY_UPDATED` once per affected full variant before response; callbacks observe all committed counts and attachment queries. Storage synchrony/order is **inferred**, not established by its UniqueEvent declaration. No queued duplicate.
- [x] Response handlers can synchronously query and reenter real mutators. No retained borrow, post-callback overwrite, or callback bypass. Environments keep independent state/listeners.

### B73 removal requirements — bounded acceptance, 2026-10-03

Scope: `RemoveFixtureFromSelectedPoint(attachedDecorAction)` only, EXACT267/268. All policy, validation order/domain, mutation boundaries, live eligibility, failure handling and event ordering below are **inferred simulator requirements**, not native-verified semantics. The declaration grounds AllowedWhenUntainted and omitted Store; explicit nil is inferred despite `Nilable = false`.

- [x] Authenticate the original arg1 before type/domain/model access: secure actual secret NUM0/1 work; addon secret NUMs, BOOLs and strings reject without private payload disclosure, mutation or events. Public addon0/1 retains taint. Numeric0/1 only; wrong types, strings, fractions, nonfinite and unknown enum values reject atomically. Omitted/nil select Store0 (nil inferred).
- [x] Require owned plot, exterior customization mode6, selected point, current fixture and raw host `can_remove`. Missing/nonremovable/ineligible state and repeated removal after the fixture is absent produce explicit atomic errors and no events; no native error/result-code mapping claimed. Clear **only** `selected_fixture_id`; retain point owner/options and raw `can_remove`. Public `canSelectionBeRemoved` becomes false while the fixture is absent and automatically returns to the raw eligibility on existing `SelectFixtureOption` reselection (inferred dynamic eligibility). Selected point remains present; attachment query becomes false. Size/type/options and other exterior state remain unchanged.
- [x] Store only matching-owner attached placements into existing full-key inventory, updating known base counts atomically before clearing/publishing. Preserve variant/base identity, destroyability, all dye metadata, unrelated-owner and floating placements; unknown aggregates remain unknown. Prevalidate mixed valid/invalid variants (missing, negative, overflow) and known aggregate overflow/underflow, including combined deltas: no partial inventory/placement/selection mutation or event.
- [x] Detach only matching-owner placements, preserving positions, full-key IDs and storage even with missing catalog records; reselection never reattaches them. No attachments still clears the fixture and emits one Success0 response. Success returns zero Lua values. After full commit, publish one rooted storage event per affected full variant, then synchronous `HOUSING_SET_FIXTURE_RESPONSE` with exactly one HousingResult0 payload. Storage order and Success0 policy inferred. GC and storage/response reentry must observe committed counts/selection, preserve original payload, and allow existing fixture reselection without outer overwrite or queued duplicates.
- [x] Preserve optional pending request and environment isolation. Retain all41 historical B70 modern tests; extend only the existing inverse control to call Remove and retain seeded/no-op behavior. No SelectCore/3D/new no-ops, production data structures or Cargo targets.

Grounding read for B73: patch source EXACT267/268 adds arg1 and AllowedWhenUntainted; cached `HouseExteriorUIDocumentation.lua:150–157` declares `attachedDecorAction`, `Default = "Store"`, `Nilable = false`; `HouseExteriorConstantsDocumentation.lua:77–79` declares nullable `selectedFixtureID`, fixture options and public removal flag. Existing enum documentation at `PlayerHousingConstantsDocumentation.lua:141–150` grounds Store0/Detach1 meanings; exterior documentation324–330 grounds synchronous response/one-result shape, not per-call success/failure mapping.

Historical input baseline correction: `/tmp/patch-12.0.5-remove-fixture-boundary.md`/580's “method missing” claim was wrong. Remove was published as `__wow_noop` in the main Lua namespace, without legacy capture or a modeled modern callback. B73 producers replace that modern path; the no-op now belongs only to the legacy initializer. Compiled RED exercises the published API, not a shared query setup failure.

### Grounding inspected 2026-10-02

Cached sources under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `HouseExteriorUIDocumentation.lua:171–210`: three signatures, both selectors, Store defaults and AllowedWhenUntainted. Lines1–140 ground supporting queries and owned-plot/exterior-mode attachment availability; lines215–334 declare synchronous response events and one HousingResult argument.
- `PlayerHousingConstantsDocumentation.lua:141–151`: Store returns attached decor to storage; Detach leaves decor floating at its current position and explicitly does not reattach. Lines300–315 ground result identifiers AlreadySize42/AlreadyType43, not exact emission/failure mapping.
- `HouseExteriorConstantsDocumentation.lua:1–85`: option and fixture-point DTO fields. This is current cached declaration evidence, not a native mutation probe or pinned historical behavioral proof.
- Existing `housing_catalog_state.lua:908–959` publishes seeded reads and three no-op mutators. Historical inputs left runtime untouched; modern registration now replaces it, while inverse profiles retain it.

## How it works

- [Housing catalog variant identity and storage](housing-catalog-variants.md).
- [Existing storage update event contract](housing-storage-entry-updated.md).
- [Existing event dispatch](../event-system.md).

## Implementation inventory

- `src/c_api/c_housing/exterior.rs`: empty-default data structs and single modern availability decision.
- `src/c_api/c_housing/exterior/`: authenticated callbacks, live rooted read snapshots, atomic selection/placement changes, and prevalidated catalog storage deltas.
- `src/c_api/c_housing.rs`: registers exterior alongside catalog; `src/lua_api/state/support_types.rs` holds `HousingState.exterior`.
- Legacy bootstrap returns its original seeded exterior initializer; only inverse profiles invoke it. Modern registration never falls back to missing Lua methods.
- `tests/house_exterior_attached_decor.rs`: 41 retained B70 modern tests plus19 B73 removal inputs and one extended inverse legacy control, automatically discovered by existing grouped `integration` target; no Cargo target added. B73 execution proof pending.
- This spec: contract and inference/native-evidence limits; bounded simulator acceptance is recorded below; native gaps remain.

## Tests asserting this spec

All in `tests/house_exterior_attached_decor.rs`:

| Case names/group | Contract |
|---|---|
| `{fixture,size,type}_{store,detach,omitted}_changes_only_affected_placements`; `nil_action_is_inferred_store_for_each_mutator` | Actual namespace calls first, then exact host selection/position/identity/count delta; no shared pre-call seeded-query assertions |
| `{fixture,size,type}_secure_actual_secret_numbers_are_accepted`; `_public_addon_call_preserves_taint`; `_both_original_selectors_authenticate_before_types_or_model` | Real rooted host-secret NUMs, secure acceptance, addon denial, auth-before-type/model and private payload protection |
| `{fixture,size,type}_invalid_types_and_domains_reject_atomically`; `_unknown_locked_or_invalid_option_has_no_inventory_effect` | Error domains versus model failure; atomicity, no fabricated inventory |
| `{fixture,size,type}_synchronous_storage_then_response_observe_committed_state`; `store_emits_once_per_affected_full_variant_after_all_counts_commit` | Real frame listeners, payload arity, full-key variant identity, committed storage/selections/attachment queries, no queued duplicates |
| `decor_action_enum_matches_primary_store_and_detach_values`; `current_selection_and_primary_dtos_are_live_read_only_snapshots`; `selected_point_and_availability_queries_use_explicit_state` | Publication plus separate live/copy/GC DTO and availability proof |
| `default_has_no_exterior_records_and_no_fabricated_query_selection`; `same_size_type_and_fixture_never_touch_existing_attachments`; `response_listener_can_query_and_reenter_without_outer_overwrite`; `environments_do_not_share_exterior_placements_or_storage` | Empty default, same-value policies, synchronous reentry, isolation |
| `missing_selection_options_and_unavailable_host_reject_without_inventory_effect`; `store_preserves_unknown_base_totals_without_synthesizing_aggregates`; `invalid_inventory_host_state_rejects_store_atomically` | Host gaps, unknown aggregate preservation, missing/negative/overflow inventory atomicity |
| `storage_listener_gc_and_reentry_preserve_committed_state_and_original_payload` | Original storage payload survives forced GC/nested mutation; committed counts, event order/arity, final nested state and no queued duplicates |
| `mixed_valid_and_invalid_affected_variants_reject_store_atomically` | Fixture/size × missing/overflow variant; exterior, both variants/base records and pending state unchanged, zero events |
| `legacy_inverse_control_preserves_seeded_queries_and_noop_mutators` | Earlier/other profile behavior retained; no modern-profile credit |

Fixtures explicitly supply types101/102, fixtures301/302, owner hashes11/22, sizes3/4, variant(7101,1,0) stored2, known base stored2/placed3, three attachments across two points and one floating placement at distinct coordinates. Multiple-variant case explicitly adds variant1 stored6 and corresponding known base stored8. Callback observations are asserted outside handlers so swallowed callback errors cannot count as success.

### B73 tests asserting unchecked removal requirements

All19 new tests are in the existing modern module of `tests/house_exterior_attached_decor.rs`; helpers reuse `fixture_env`, `placements`, full-key records and existing fixture selection. API calls precede post-removal query assertions. Callback results are asserted outside handlers.

| Exact test names | Requirement |
|---|---|
| `remove_store_clears_only_fixture_and_returns_selected_owner_attachments`; `remove_detach_preserves_full_identity_position_and_inventory`; `remove_omitted_action_defaults_to_store`; `remove_nil_action_is_inferred_store` | Exact state delta, preservation, declared/default and inferred nil behavior |
| `remove_secure_actual_secret_store_and_detach_are_accepted`; `remove_public_addon_actions_retain_taint`; `remove_addon_secret_authentication_precedes_domain_and_model`; `remove_actual_secret_wrong_types_authenticate_before_type_validation`; `remove_wrong_types_and_malformed_enum_reject_atomically` | Real rooted secrets, secure acceptance/addon denial, validation order, domains and atomic rejection |
| `remove_missing_fixture_point_and_ineligible_host_fail_without_events`; `remove_no_attachments_clears_fixture_with_one_success_response`; `remove_detach_needs_no_catalog_and_never_reattaches_after_selection`; `remove_live_eligibility_restores_on_selection_and_repeat_remove_errors` | State failure matrix, empty attachments, missing catalog, raw flag retention/live eligibility and repeat rejection |
| `remove_store_unknown_aggregate_totals_remain_unknown`; `remove_store_mixed_valid_invalid_variants_and_aggregates_are_atomic`; `remove_store_combined_known_aggregate_limits_reject_before_any_commit` | Unknown totals, valid+invalid full-key matrix, combined aggregate overflow/underflow atomicity |
| `remove_store_full_variant_events_follow_complete_commit_and_survive_gc`; `remove_storage_and_response_gc_reentry_keep_new_fixture_and_original_payload`; `remove_preserves_pending_request_and_environment_isolation` | Committed variant/base counts, rooted payloads, exact response shape, storage/response GC reentry, no queued duplicates, pending/isolation |

### Proof ledger

#### B73 independent bounded acceptance — 2026-10-03

Saved `/tmp/patch-12.0.5-remove-fixture-final-proof.{md,json}` supersedes the runtime-pending checkpoint in `/tmp/patch-12.0.5-remove-fixture-independent-proof.{md,json}` only for scoped EXACT267/268 simulator acceptance. Producer `71973af4b`, documentation revision `7aa5fbfb6`; seven-path equivalence inherited from the independent audit, not whole-tree or current-HEAD equivalence.

**60/60 modern PASS = 19 new removal + 41 retained modern tests**, matched by exact name without duplicates: 51 in partition0089 and 9 in partition0090. Original-argument authentication/taint/default/domain, owner-only atomic Store, identity/position-preserving Detach, live reselection, pending/isolation, rooted full-key events and synchronous GC/storage/response reentry have bounded runtime proof. Separate Forever inverse **1 PASS**, not a 61st modern case or full-profile suite. Retail startup exit0, stdout `[]`, with third-party addons and SavedVariables disabled. Scopedfmt exit0; default check exit0, zero diagnostics (239.962097s including lock wait). Root test compilation and separate Forever compilation exit0 with zero diagnostics; compilation is not additional execution credit.

The later `/tmp/patch-12.0.5-full-suite-independent-proof.md` summary corrects the final proof's provisional 12,021/60/18, four-unobserved and pending-doctest status: **12,022 PASS / 60 FAIL / 18 ignored** across 12,100 ordinary identities after raw parent-result recovery. Separate custom runners retain **11 failures** (frame_positions: 27 PASS/1 FAIL; prefork_full_ui: 2,006 PASS/10 FAIL). Doctests finished exit0 with **0 PASS / 3 ignored**, not three passing examples. Full suite is **FAIL**, not blanket GREEN. Four seeded exterior-query failures in partition0089 remain open; their cause, age and unrelatedness are not established. The EXDEV launcher incident remains historical, no longer a blocker to the selected 60 results.

Checked B73 requirements mean bounded simulator proof only. Nil/domain/error mapping, live eligibility, storage ordering and Success0 remain inferred, not native-verified. The independent LENGTH advisory for the 48-line `assert_removed_fixture` helper remains nonblocking, not repaired. No native/full-UI/acquisition/core/3D/all-profile/global-privacy/whole-repo-formatting or whole-goal completion. Main source-accounting commit `cbf001ba5` promotes only EXACT267/268 and adds `house-exterior-fixture-removal`; independent accounting603 PASS33/33 ([proof](../../../../../../../../tmp/patch-12.0.5-batch73-accounting-validation.md)). Current checkpoint: **175 pending / 166 bounded / 14 partial / 7 metadata,362 ordered IDs/78 capabilities**. Historical177/164/14/7 remains a prior checkpoint.

#### Test-only exterior fixture repair — 2026-10-03

Main-supplied `365643a96` repairs four exterior assertions by explicitly seeding modern host state while keeping the assertions for all profiles. No production change. Saved async 64-case execution exits0; raw evidence: `/tmp/patch-12.0.5-exterior-fixture-proof`. Main now accepts independent609 for scoped **64 exterior cases + 4 pinned-child CLI passes** only. Check interruption and dirty-combined limits remain. This is fixture-repair acceptance, not new B73 contract acceptance, current-whole-HEAD proof or full-suite GREEN. The original suite ledger stays red; this checkpoint does not establish failure age or native parity.

#### Historical B73 implementation and compiled RED — 2026-10-03

Main-supplied producers `036036e7c` / `8502a2ced` / `71973af4b` implement modern Remove state mutation, original-argument input authentication and public registration; the DTO removal flag is live. Lua Remove no-op moves only to legacy initialization; existing methods remain preserved. At this historical checkpoint, implementation was not accepted behavior: all five removal requirement boxes were unchecked, with GREEN, independent verifier and Forever inverse proof **pending**. Current scoped acceptance is recorded above. Declaration-grounded signature/default/security and inferred nil/domain/eligibility/failure/Success0/storage-order policies remain distinct; no native parity claim.

Actual RED at `710674128`: compile **140.106230s, zero diagnostics**; **19 FAIL / 0 PASS, 2.63s**. First `loader()` compilation failure is not behavioral RED. Saved artifacts are `batch73-red-fixed-build-result.json` and `red-run.*` under `/tmp`, as supplied by the brief. This proves the pre-producer removal boundary, not downstream GC/reentry success or producer GREEN. Existing B70 41-case and acceptance proof below remain historical, not refreshed B73 proof.

Initial full-suite snapshot710 compilation failed with **E0063**, missing `addon_dir` in an old test fixture; no suite pass. User approved one-field repair `9d9015e1e`. New snapshot71973 full suite is asynchronous at `/tmp/patch-12.0.5-all-tests-async-71973`; outcome pending, **no full-suite-pass claim**. No job inspection or execution gates in this docs-only update.

EXACT267/268 remain pending; **177 pending / 164 bounded / 14 partial / 7 metadata,362 ordered IDs/77 capabilities** unchanged. No accounting promotion or core/native/UI/all-profile credit.

#### Historical B73 input checkpoint — 2026-10-03

Tests/spec only. Owned test formatting with `rustfmt --edition 2024 --config skip_children=true`; no delegation, builds, runtime, tests, checks or verification gates. Main must establish compiled RED **before** producers, then GREEN and producer proof gates. Authored inputs are not behavioral evidence. B70 acceptance below remains historical and intact; B73 requirements remain unchecked. EXACT267/268 stay pending; **177 pending/164 bounded/14 partial/7 metadata,362 ordered IDs/77 capabilities** unchanged. No new/native/profile/UI/acquisition/core credit.

#### Independent bounded acceptance — 2026-10-03

Main accepted EXACT272/274/276 after independent566 final PASS, recorded in `/tmp/patch-12.0.5-house-exterior-final-proof.md` and `.json`. Accepted scope: runtime `41904efbf`, queries `c3f0c7f4c`, publisher visibility `e7750b17d`, refactor `62f3629b8`, new tests `d4dd4386b` and final `361642437`.

**104 distinct Retail PASS =41 fresh exterior +63 unchanged housing regressions**; historical39 overlaps41, not additional credit. Secure actual NUM selectors/authentication-first, live DTOs, synchronous storage/response GC reentry and mixed-valid/missing-or-overflow atomicity pass. Separate Forever inverse1 PASS, not105 Retail cases/full-profile proof.

Final `361642437`: compile235.194071s exit0/zero diagnostics;41 run9.536627s; startup11.959878s exit0 `[]`; check30.825380s exit0/zero diagnostics. Scopedfmt/readability/security/rooting/atomicity/reentry/wiring/equivalence PASS. Forever compile325.970902s exit0/zero diagnostics; inverse0.411750s. Saved proof only; no execution gates in this reconciliation.

Accounting `0cf868024`: **177 pending/164 bounded/14 partial/7 metadata,362 ordered IDs/77 capabilities**; independent568 PASS30/30, preserving359 unrelated rows/76 existing capabilities ([accounting proof](../../../../../../../../tmp/patch-12.0.5-batch70-accounting-validation.md)). Only272/274/276 promote. Enum already passed RED through another publisher: no formerly-missing-enum/core/remove row credit. Prior B69 counters remain historical.

Native coercion/failure mapping, nil/same-value/storage-order policies remain inferred. UI/acquisition/pets/global permissions/privacy/nominal types/declaration dating/full-profile parity unproved. Original globalfmt1, Forever mask-quad FAIL and historical502 process FAIL remain explicit; protected source never inspected/searched/hashed, no blanket clearance.

#### Historical pre-acceptance ledger

- B70 implementation awaiting acceptance: main-authored runtime `41904efbf0fc6ea9ac4294847ea4dc2cd7255bdb`; supporting queries `c3f0c7f4c73b901a3c8ea2045db8219472bbb30e`. Commit existence/subjects inspected in this docs-only audit; runtime behavior has not been independently accepted.
- Supplied pre-runtime RED at `5ad54e5782d28b752a621d7fb7a3e915aa2f9d96`: default integration compile exit0, zero diagnostics, 264.585871s; modern39 cases, 38 FAIL/1 PASS, run8.704321s. This proves the pre-runtime failure boundary, not producer GREEN or downstream acceptance.
- GREEN, inverse-profile acceptance, check, readability/security gates and startup remain pending. All behavioral boxes remain unchecked; no accepted capability or core/remove/enum row credit. Current classification stays at the [accounting SSOT checkpoint](../wiki/investigations/patch-12-0-5-api-audit.md#six-source-occurrences--metadata-only-accounting-accepted).
- Cached declarations ground signatures, Store/Detach meanings, DTOs and synchronous response declarations only. Native failure/coercion/event mapping is unproved; nil/same-value handling, storage synchrony/order and invalid trusted-host inventory policies remain inferred. World acquisition, UI, global permissions/privacy and pets remain open. Implementation does not promote these policies to native evidence.

- Input owner ran no builds, tests, checks or verification gates. Main recorded compiled RED in `/tmp/patch-12.0.5-batch70-red-build-result.json` and `-red-run.json`. No native parity claim.
- `rustfmt --edition 2024 --config skip_children=true` on the four owned Rust files exited0 before inputs commit. Formatting evidence only; main must establish compiled RED before callback implementation.

## Known gaps (current cycle)

B70 and B73 bounded acceptance supersede their historical pending gates; native gaps and full-suite failures remain open.

- [x] B73 main-owned compiled RED at `710674128`: 19 FAIL/0 PASS; first loader compilation failure excluded.
- [x] B73 scoped modern60, independent audit, startup and separate Forever inverse1 accepted from saved artifacts; docs owner ran no execution gates.
- [ ] Full-suite clearance: ordinary60 FAIL plus separate custom11 FAIL remain; doctests0 PASS/3 ignored. No blanket GREEN.

- [x] Main-owned compiled RED at `5ad54e578`: default build exit0, zero compiler diagnostics; 38 failures / one pass in 8.7s.
- [x] Bounded GREEN, scoped Rust/security/readability gates, startup and separate Forever inverse acceptance.
- [ ] Native failure/coercion/event mapping, storage-event ordering, nil/same-value policies and trusted-host invalid inventory handling remain unproved/inferred.
- [ ] Native UI, acquisition, pet behavior, global permissions/privacy and declaration dating remain unproved; they are evidence gaps, not waived requirements or completed coverage.

## Out of scope

- B70 acceptance excludes RemoveFixtureFromSelectedPoint; B73 removal has bounded simulator acceptance and exact267/268 source-accounting credit only. SelectCoreFixtureOption, door/core/hover APIs, 3D and native parity remain excluded.
- Historical inputs ownership excluded runtime/integration gates; main implementation and acceptance are recorded above.
