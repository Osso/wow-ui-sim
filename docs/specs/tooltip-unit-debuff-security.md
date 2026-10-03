# Indexed unit-debuff tooltip inputs — B81 exact347

Bounded Retail 12.0.5 contract for `C_TooltipInfo.GetUnitDebuff`, source347's argument1 `NeverSecret` removal. The modern producer now shares B79's indexed Buff boundary in `src/c_api/c_tooltip_info_indexed_aura.rs`. The older `probes.rs` provider still parses/discards unit/index and returns an empty UnitAura DTO below the modern feature boundary. Inputs authored 2026-10-03; compiled behavioral RED inspected before production edits. Producer implementation is not GREEN or acceptance proof; main owns asynchronous GREEN and integration. Source347 remains pending, with no requirement-completion or native parity credit.

## What it must do

### Live harmful selection — inferred simulator contract

- [ ] Read mixed live `player.buffs` and requested `party_members[N].debuffs`, never party buffs or constant player/catalog content. Skip helpful sentinels even inside party debuffs. Select a positive 1-based harmful filtered index, not raw position or instance ID.
- [ ] Omitted/public nil filter defaults to HARMFUL. `HARMFUL|PLAYER` applies existing `is_from_player_or_player_pet` source classification before indexing. Per-unit blocked instances are removed before indexing; identical instance IDs on another unit remain visible.
- [ ] Conflicting `HELPFUL` and `HELPFUL|PLAYER`, and bare unknown `UNKNOWN`/`UNKNOWN|PLAYER`, produce no matches. This follows the established helper's helpful interpretation without HARMFUL, intersected with Debuff's harmful polarity. Mixed `HELPFUL|HARMFUL` and unknown tokens combined with HARMFUL are not covered.
- [ ] Missing rows, out-of-range and nonpositive integral indices return exactly one fresh line-empty public UnitAura DTO with only `type` and `lines`. Unsupported `target`, `pet`, `focus`, `raid1`, `party3` in the two-member fixture, unknown and empty tokens miss without player fallback. Target's fixed helper fixture is explicitly outside this chosen domain.
- [ ] Queries preserve every host aura field in all player/party stores. Later name mutation, removal and clear refresh new results while previous DTOs remain snapshots; stores and separate environments remain isolated. Caller DTO mutation cannot alter subsequent results or host content.

### Original secret arguments — cached contract boundary

Cached Retail `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua` GetUnitDebuff block is recorded in `/tmp/patch-12.0.5-debuff-indexed-boundary-map.md` as `SecretArguments="AllowedWhenUntainted"`, `RequiresUnitAuraAccess`, `SecretWhenUnitAuraRestricted`, and `MayReturnNothing`, with required unit/index and nullable filter. The local temporary map is investigation input, not native execution evidence or a durable specification dependency. The declarations do not prove selected payload, miss/error policy, access or output secrecy.

- [ ] Authenticate **all three original positions before any parsing or lookup**. Secure authentic host-secret unit string, numeric index, filter string and nil filter must drive concrete selection/default/miss behavior without modifying wrappers or caller trust. Combined originals must select and genuinely miss filtered rows.
- [ ] Tainted authentic secret inputs at each position are denied before malformed earlier public arguments, unsupported-unit/out-of-range lookup, or malformed secret payload parsing. Include secretNil at unit/index/filter. Public tainted calls still select concrete content and preserve caller taint; outer secure context is restored.
- [ ] Secure malformed secret BOOL/table/actual Frame payloads at every position, nil required arguments and secret string index cause ordinary validation errors. Public wrong types, omitted required arguments, string index, fractional/nonfinite/out-of-i32 numeric index cause errors, including on unsupported units. These strict parsing/error policies are **inferred**, not native-verified.
- [ ] Errors identify `C_TooltipInfo.GetUnitDebuff`, never disclose private payloads, preserve caller taint, and allow public recovery. Distinguish existing VM `requires an untainted caller` authorization denial from ordinary errors; no native wording claim.
- [ ] Root authentic wrappers in permanent Lua globals and a permanent global list before API calls/allocation/GC. After secure acceptance, tainted denial, ordinary errors and allocation/forced collection, assert host userdata identity/allocation sequence, list identity, secrecy and original table/Frame properties. No tainted Lua equality comparison of authentic secret BOOLs.

### Unchanged public builder convention — not access/secrecy parity

- [ ] A selected row uses existing builder output: UnitAura type, selected host name, three lines (SpellName, hardcoded `1 hr`, wrapped nonempty SpellDescription), semantic public RGBA channels. DTO keys/values remain public under the current simulator convention only. Fresh DTO/lines/line tables do not alias other results.
- [ ] Preserve host icon135987 through read-only calls. The unchanged builder publishes **no icon field**; tests assert its absence, not invented icon output. All records use spell19750 and duration/expiration3600; no dynamic duration or description parity credit.

Explicit host fixture (all records share spell19750/icon135987):

| Store | Ordered IDs/names | Helpful / player-source |
|---|---|---|
| `player.buffs` | 101 Player helpful sentinel; 102 Player outside source; 103 Player own source | true/true; false/false; false/true |
| `party_members[0].debuffs` | 200 Party one helpful sentinel; 201 Party one outside source; 202 Party one player source | true/true; false/false; false/true |
| `party_members[1].debuffs` | 200 Party two helpful sentinel; 201 Party two player source | true/true; false/true |
| Each party member's `buffs` | 901 Wrong party buffs store | false/true; never an indexed Debuff hit |

Shared instance201 across units and distinct names distinguish requested state from plausible fallback content. Explicit host records are test inputs, not aura-acquisition evidence. No instance-query API supplies expected indexed results.

## How it works

- [Lua API boundary](../lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)
- [Adjacent B79 contract and VM guards](tooltip-unit-buff-security.md)
- [Aura filter-query contract](unit-aura-filter-query.md)

## Implementation inventory

- `tests/tooltip_unit_debuff_security.rs`: new Retail12.0.5-gated public API fixtures/authentic VM roots; existing grouped integration automatic discovery, no Cargo target.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: existing empty indexed Debuff provider/body preserved below `retail-12-0-5`, excluded from modern builds.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: modern indexed provider registration and renamed `tooltip_for_selected_indexed_aura` bridge to the unchanged builder; old Debuff registration retired only at `retail-12-0-5`, not overridden or used as fallback. Older-profile registration/provider unchanged.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: unchanged selected-aura DTO builder, no icon output and hardcoded duration line.
- `src/lua_api/globals/auras.rs`: existing visibility/filter/source helpers, unchanged.
- `src/c_api/c_tooltip_info_indexed_aura.rs` (renamed from B79's `c_tooltip_info_unit_buff.rs`) and `src/c_api/mod.rs`: shared modern Buff/Debuff authentication, strict parsers and filtered positive 1-based selection, parameterized only by API name and existing `AuraFilter`. All original arguments authenticate before any parsing; all validation precedes lookup/domain misses. Debuff defaults nil to HARMFUL, reads harmful `player.buffs`/requested party `debuffs`, enforces polarity and existing blocked/source filters, and excludes fabricated target records. Other unsupported tokens miss via existing helpers. Buff's helpful default/polarity, lookup, validation/error context and builder are preserved; refreshed preservation proof is pending.

## Tests asserting this spec

`tests/tooltip_unit_debuff_security.rs`: **15 compiled behavioral RED cases** at `6aa7a391bf2ec40f1ff6f2924518a8a0bcd2d353`. Saved artifacts `/tmp/patch-12.0.5-debuff-indexed-red-ops/`: build exit0/89.594915484s/zero diagnostics; main-reported run exit101/1.898831219s, stdout reports1.86s, **15 FAIL / 0 PASS**. Full stdout and tests inspected before production edits. All new requirements remain unchecked until main acceptance; no producer-owned execution of GREEN/check/readability/coverage/startup/acceptance.

| Exact case | Observable contract | Proof |
|---|---|---|
| `distinct_live_units_select_harmful_rows_from_correct_stores` | Mixed player/party polarity, correct store, concrete unit and second filtered row | Authored only |
| `omitted_and_nil_filter_default_to_harmful` | Public omitted/nil defaults | Authored only |
| `player_filter_selects_source_before_one_based_index` | PLAYER selection precedes index; no instance-ID indexing | Authored only |
| `blocked_visibility_precedes_index_and_is_unit_local` | Blocked-before-index and cross-unit ID isolation | Authored only |
| `helpful_and_unknown_filters_do_not_select_harmful_or_helpful_sentinels` | Conflicting/bare unknown filter misses | Authored only |
| `unsupported_units_and_nonpositive_or_missing_rows_return_one_empty_dto` | Chosen domain, one public empty DTO | Authored only |
| `live_mutation_removal_and_clear_refresh_snapshots_without_cross_unit_leaks` | Live host refresh, old snapshot and environment isolation | Authored only |
| `dto_snapshots_are_fresh_and_caller_mutation_cannot_change_host_content` | Fresh results and read-only stores | Authored only |
| `public_tainted_calls_preserve_content_source_selection_and_caller_taint` | Public tainted compatibility | Authored only |
| `secure_original_secret_unit_index_filter_and_nil_drive_real_selection` | All original secure selectors and combined match/miss | Authored only |
| `tainted_secret_each_position_authenticates_before_earlier_malformed_parse` | Original-position authorization precedence, including secretNil | Authored only |
| `tainted_malformed_secret_payloads_are_denied_before_type_or_miss` | Malformed secret authorization, private payload protection | Authored only |
| `secure_and_public_invalid_types_are_ordinary_errors_without_payload_leaks` | Inferred type errors, recovery, original properties | Authored only |
| `fractional_nonfinite_and_out_of_i32_indexes_are_inferred_errors` | Strict numeric validation before lookup | Authored only |
| `allocation_and_gc_preserve_permanent_secret_roots_and_original_properties` | Permanent roots, identity/sequence/secrecy through API allocation and GC | Authored only |

Read-only probes compare all AuraInfo fields before/after, including icons and both party stores; secret probes also compare permanent global/list host identities and allocation sequences. Fixture mutation case intentionally changes only host records between public queries.

Observed RED boundaries: old probes provider's empty DTO fails `name/duration/description payload`; original secrets fail `expected string, got userdata at argument 1`; rejection/numeric probes fail `API error context`. RED proves these reached boundaries, not every downstream assertion in15 tests: GC/root preservation was not reached, and unsupported-target probe later failed selected recovery payload rather than an independently isolated target assertion. Target exclusion is an inferred chosen real-player/party domain policy. Test-table 'Authored only' entries retain no per-requirement PASS claim; compiled failure evidence does not upgrade them.

## Known gaps (current cycle)

- [x] Main compiled behavioral RED inspected at `6aa7a391b`; artifacts/reached-boundary qualifications above.
- [ ] Bounded modern producer GREEN/independent acceptance. Producer implemented here; all execution/acceptance remains main-owned and pending.
- [ ] Exact347 argument1 accounting after main acceptance; arg2/arg3 constraints are not additional source-row removals.

## Out of scope

- `RequiresUnitAuraAccess` and `SecretWhenUnitAuraRestricted` remain **UNMODELED**; public DTO checks confer no access/restricted-output parity.
- Native lookup/filter/error/miss/payload verification, aura acquisition, general unit resolution, broader filter grammar and dynamic duration are unverified.
- Old-profile behavior changes/verification, instance-API fallback, Buff behavior changes, aura helper/model changes, vendor changes, Cargo targets and audit/wiki accounting are excluded. Only shared provider/bridge extraction and modern Debuff wiring are authorized production changes; execution gates remain main-owned.
