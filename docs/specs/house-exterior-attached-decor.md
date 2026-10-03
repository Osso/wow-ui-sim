# House exterior attached-decor actions

Batch70 bounds EXACT272/274/276: `C_HouseExterior.SelectFixtureOption`, `SetHouseExteriorSize`, and `SetHouseExteriorType` with `attachedDecorAction`. Inputs live in `src/c_api/c_housing/exterior.rs`; behavior is pending main-owned compiled RED and production implementation. The [existing housing catalog](housing-catalog-variants.md) remains storage identity/count SSOT. No behavioral or profile acceptance is claimed by this inputs commit.

## What it must do

### Selections and attachments

- [ ] Modern behavior applies only with `retail-12-0-5` and `profile-retail` or `client-ptr`. Earlier/other profiles retain original no-op mutators and seeded queries; inverse controls receive no modern credit. Data structs may exist unconditionally.
- [ ] Default exterior state has no selections, options, fixture point, or placements. Existing `inside_owned_plot` and `active_house_editor_mode == 6` supply availability; no extra ownership framework.
- [ ] Explicit unlocked size/type/fixture options authorize changes. Type and fixture options also require `is_invalid == false`. Missing selection/options, unknown IDs, unavailable host, and locked/invalid targets reject atomically without inventory changes. Native error versus failure-response mapping is unproved; tests accept an explicit error or one non-Success response, never silent success.
- [ ] `Store = 0` removes affected attached placement instances and increments existing full-key catalog variant `num_stored` by removed instance count. No fabricated variants, catalog records, IDs, or aggregate totals. Preserve destroyable counts and all dye fields. Explicit known base totals change by stored +delta / placed -delta; `None` remains an unmodeled aggregate gap.
- [ ] `Detach = 1` clears affected `fixture_point_owner_hash` only. Preserve every position coordinate, placement identity, variant identity and storage count. Detached placements float at the same position and never automatically reattach.
- [ ] Fixture selection affects only attachments whose owner hash equals the selected point's hash. Size/type changes affect all attached exterior placements. Already-floating placements remain unchanged. Nonaffected point selection/options and storage remain intact.
- [ ] Omitted action defaults to Store. Explicit nil also defaults to Store as an **inferred simulator policy**, not observed native coercion.
- [ ] Same-size/type requests leave attachments/inventory untouched and synchronously respond AlreadySize42/AlreadyType43 (**inferred emission policy**). Same-fixture request is not replacement, preserves attachments, and responds Success0 (**inferred policy**).
- [ ] Missing variant, negative/overflowed stored count, known aggregate overflow or placed-count underflow reject Store before any mutation. Invalid trusted-host inventory handling is simulator policy, not native acquisition parity.

### Security and arguments

- [ ] Authenticate both original arguments before type/domain parsing or model access. All three cached declarations specify `SecretArguments = "AllowedWhenUntainted"`: actual typed secret NUM selectors/actions work in secure callers; addon callers cannot use secrets. Public addon arguments work without clearing taint. Do not substitute secure-secret rejection, declassification, fake callbacks, or security overrides.
- [ ] Targets are strict positive integral `u32` IDs; size must be a declared option. Action accepts numeric 0/1 only. No string coercion, fractional truncation, nonfinite values or undeclared target acceptance. Wrong-type/domain arguments fail with API/argument-position context, no private payload, no mutation and no event. Exact native errors/coercion remain unproved.
- [ ] Arg2 secret authentication wins over arg1 wrong type or absent model target. Authentication failures remain atomic, do not emit responses/storage events, and do not expose secret numeric payloads.
- [ ] Publish `Enum.HousingFixtureDecorAction.Store = 0` and `.Detach = 1`. Mutation fixtures use literal numbers so missing enum publication cannot mask actual call-boundary RED.

### Queries and callbacks

- [ ] Supporting reads reflect the same explicit state: `GetCurrentHouseExteriorSize`; `GetCurrentHouseExteriorType` returns ID and explicit matching option name; size/type option DTOs; selected fixture point DTO; selection and exterior/selected-point attachment predicates. Nil/missing selections must not reproduce old seeded data. Snapshot mutations/GC must not change host state.
- [ ] DTO fields match cached primary structures, including `isLocked`, `isInvalid`, `reasonString`, fixture `typeID/typeName/colorID`, point `ownerHash/selectedFixtureID/fixtureOptions/canSelectionBeRemoved`. Do not confuse current `reasonString` with historical `lockReasonString`.
- [ ] Valid changed calls return zero Lua values and synchronously publish the documented one-argument response event with Success0 after all state commits: `HOUSING_SET_FIXTURE_RESPONSE`, `HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE`, or `HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE`.
- [ ] Store publishes existing `HOUSING_STORAGE_ENTRY_UPDATED` once per affected full variant before response; callbacks observe all committed counts and attachment queries. Storage synchrony/order is **inferred**, not established by its UniqueEvent declaration. No queued duplicate.
- [ ] Response handlers can synchronously query and reenter real mutators. No retained borrow, post-callback overwrite, or callback bypass. Environments keep independent state/listeners.

### Grounding inspected 2026-10-02

Cached sources under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`:

- `HouseExteriorUIDocumentation.lua:171–210`: three signatures, both selectors, Store defaults and AllowedWhenUntainted. Lines1–140 ground supporting queries and owned-plot/exterior-mode attachment availability; lines215–334 declare synchronous response events and one HousingResult argument.
- `PlayerHousingConstantsDocumentation.lua:141–151`: Store returns attached decor to storage; Detach leaves decor floating at its current position and explicitly does not reattach. Lines300–315 ground result identifiers AlreadySize42/AlreadyType43, not exact emission/failure mapping.
- `HouseExteriorConstantsDocumentation.lua:1–85`: option and fixture-point DTO fields. This is current cached declaration evidence, not a native mutation probe or pinned historical behavioral proof.
- Existing `housing_catalog_state.lua:908–959` publishes seeded reads and three no-op mutators. Inputs commit leaves this runtime untouched; main owns replacement after compiled RED.

## How it works

- [Housing catalog variant identity and storage](housing-catalog-variants.md).
- [Existing storage update event contract](housing-storage-entry-updated.md).
- [Existing event dispatch](../event-system.md).

## Implementation inventory

- `src/c_api/c_housing/exterior.rs`: data structs only, empty-default `HouseExteriorState`, explicit selection/options/placements; no callbacks or mutation helpers.
- `src/c_api/c_housing.rs`: `pub mod exterior` declaration only.
- `src/lua_api/state/support_types.rs`: empty-default `HousingState.exterior` field only.
- `tests/house_exterior_attached_decor.rs`: 39 modern behavioral inputs and one inverse legacy control, automatically discovered by existing grouped `integration` target; no Cargo target added.
- This spec: contract and inference/native-evidence limits; all behavioral boxes remain unverified.

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
| `legacy_inverse_control_preserves_seeded_queries_and_noop_mutators` | Earlier/other profile behavior retained; no modern-profile credit |

Fixtures explicitly supply types101/102, fixtures301/302, owner hashes11/22, sizes3/4, variant(7101,1,0) stored2, known base stored2/placed3, three attachments across two points and one floating placement at distinct coordinates. Multiple-variant case explicitly adds variant1 stored6 and corresponding known base stored8. Callback observations are asserted outside handlers so swallowed callback errors cannot count as success.

### Proof ledger

- Inputs only: no build, test, check, readability or integration gates run by this owner. No compiled RED/GREEN/native parity claim.
- `rustfmt --edition 2024 --config skip_children=true` on the four owned Rust files exited0 before inputs commit. Formatting evidence only; main must establish compiled RED before callback implementation.

## Known gaps (current cycle)

- [ ] Main-owned compiled RED, then callback/query implementation and bounded integration acceptance.
- [ ] Native failure/coercion/event mapping, storage-event ordering, nil/same-value policies and trusted-host invalid inventory handling remain unproved/inferred.
- [ ] Native UI, acquisition, pet behavior, global permissions/privacy and declaration dating remain unproved; they are evidence gaps, not waived requirements or completed coverage.

## Out of scope

- No semantics or credit for RemoveFixtureFromSelectedPoint, SelectCoreFixtureOption, door/core/hover/remove APIs in this exact-row slice.
- Inputs owner does not implement runtime publication, mutators, callbacks, legacy bootstrap replacement or integration gates; main owns those steps.
