# Tooltip unit-buff input security

B79 **exact342** removes arg1 `NeverSecret` from `C_TooltipInfo.GetUnitBuff` in [the Retail12.0.5 source](../../data/patch-api/sources/12.0.5-api-changes.txt). This inputs-only slice requires meaningful existing-model lookup and the pinned VM's secure secret-input boundary. Source currently lives in `src/lua_api/globals/missing_surface/tooltip_info/`; [Lua API architecture](../lua-api.md) describes the runtime boundary. **Exact342 stays pending.** Tests are authored, not compiled or executed; main owns compiled RED, producer implementation and acceptance.

## What it must do

### Registered API and bounded live lookup

- [ ] Call the real registered `C_TooltipInfo.GetUnitBuff(unitToken, index, filter?)`; no function replacement, fake namespace, or new production fixture.
- [ ] Resolve exact `player`, `party1`, and `party2` against current `player.buffs` and `party_members[idx].buffs`. Use an explicit host fixture, not seeded roster/catalog assumptions. A party query must not return player content.
- [ ] Select a positive **1-based filtered index**, not a raw-vector offset or aura instance ID. Helpful selection must skip a harmful record interleaved in `player.buffs`.
- [ ] Omitted or public nil filter defaults to helpful. `HELPFUL|PLAYER` excludes records whose existing `is_from_player_or_player_pet` is false before indexing; public and secret filters must affect meaningful results, not remain ignored.
- [ ] Honor the existing per-unit blocked-aura state before indexing. Blocking party1 instance201 shifts its remaining record to index1; party2 instance201 remains visible. Blocking all eligible records yields a true miss.
- [ ] Missing selected records, out-of-range indices, and nonpositive indices return exactly one fresh line-empty UnitAura DTO with only `type` and `lines`, preserving current `emptyTooltip` shape.
- [ ] Unsupported `target`, `pet`, `focus`, `raid1`, `party3` in this two-member fixture, unknown and empty tokens must not fall back to player. General unit resolution and target identity are not credited.
- [ ] Subsequent queries observe live host mutation, removal and clear. Player/party stores and separate environments stay isolated; previously returned DTOs remain snapshots. Calls do not mutate any fixture aura field. DTO, lines and line tables are fresh; caller mutation cannot affect other results or host records.

### Secret-input boundary — chosen simulator contract

Cached retail `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua`, full `GetUnitBuff` block (lines1238–1257), declares `MayReturnNothing`, `RequiresUnitAuraAccess`, `SecretWhenUnitAuraRestricted`, and `SecretArguments="AllowedWhenUntainted"`. Arguments are required `unitToken:UnitTokenRestrictedForAddOns`, required `index:luaIndex`, and nullable `filter:AuraFilters`; return is `TooltipData`. These declarations do not establish native lookup, miss/error policy, output secrecy, or payload parity.

- [ ] Authenticate genuine host-secret values at **all three original positions before parsing or state lookup**, using existing `unwrap_secret` authorization. A secure secret unit must select party1's concrete content, a secure secret index must select the concrete filtered row, and a secure secret filter must enforce PLAYER selection. Secure secret NIL filter defaults to helpful. Combined secrets must support both meaningful matches and actual filtered misses.
- [ ] Tainted secret use is denied at every position, even with invalid public arguments, unsupported units, missing indices, or malformed secret payloads. Never silently declassify or permit a lookup miss to short-circuit authentication. Ordinary public calls from tainted callers continue to work.
- [ ] Secure malformed secret BOOL/table/actual Frame inputs and public wrong-type inputs receive ordinary type errors, not VM authorization denial. Required index secret STRING is invalid. Error messages identify `C_TooltipInfo.GetUnitBuff`, do not expose private payloads, and preserve caller taint. Tests distinguish the existing rilua `requires an untainted caller` diagnostic from type errors; no native error-wording claim.
- [ ] After secure acceptance, tainted denial, ordinary errors, public recovery and forced GC, rooted input wrappers retain secrecy, host userdata identity/allocation sequence and rooted-list identity. Original table/Frame properties, all host aura fields, caller taint and outer secure context remain unchanged. No authentic secret BOOL identity comparison from tainted Lua.

Exact342 can receive only arg1 NeverSecret-removal credit after acceptance. Arg2/arg3 and pre-parse denial tests constrain the chosen existing AllowedWhenUntainted boundary; they are not additional source-row removals or a new SecretArguments-transition claim.

### Current public payload convention — not secrecy proof

- [ ] A hit returns three existing builder lines: SpellName with the selected host name, SpellName `1 hr`, and wrapped nonempty SpellDescription. Public numeric RGBA channels are checked semantically, not by color-object identity. The fixture's duration/expiration3600 agrees with the retained **hardcoded `1 hr` builder limitation**; dynamic duration is not modeled by this slice.
- [ ] DTO fields/keys are public under the **current simulator model convention only**. `RequiresUnitAuraAccess` and `SecretWhenUnitAuraRestricted` output behavior remain **UNMODELED**, not native secrecy or parity credit.

### Inferred domain, polarity and miss policies

These requirements are **inferred/chosen**, not native-verified: exact player/party host domain, helpful polarity for this Buff slice, default helpful filter, filtered 1-based indexing, unsupported-unit misses, nonpositive/out-of-range misses, strict malformed-input errors, and public DTO shape. Existing visibility/filter helpers provide simulator evidence for blocked and PLAYER behavior, not native proof. PLAYER means the current `is_from_player_or_player_pet` flag, not source-token reinterpretation.

The contract covers helpful filters only. HARMFUL, conflicting/unknown filter tokens and broader filter grammar receive no behavior credit; they must not be inferred from these tests. Target's separate fixed aura fixture is not part of this contract.

Explicit fixture, all records duration/expiration3600, spell19750, icon135987:

| Store | Ordered records | Helpful / player-source |
|---|---|---|
| `player.buffs` | 101 Player harmful sentinel; 102 Player outside source; 103 Player own source | false/true; true/false; true/true |
| `party_members[0].buffs` | 201 Party one outside source; 202 Party one player source | true/false; true/true |
| `party_members[1].buffs` | 201 Party two player source | true/true |
| Each party member's `debuffs` | 901 Party harmful sentinel | false/true; never a helpful hit |

Names deliberately differ despite shared spell IDs and reused cross-unit instance201. Thus name assertions distinguish selected state from plausible catalog/player fallback content. Host values are test inputs, not aura acquisition evidence. Description assertions preserve the builder's nonempty-string contract; they do not prove native spell descriptions.

## How it works

- [Lua API boundary](../lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)
- [Adjacent instance-query security contract](tooltip-aura-instance-security.md)
- [Aura filter-query contract](unit-aura-filter-query.md)

## Implementation inventory

- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: current registered GetUnitBuff namespace entry.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: current provider parses then discards unit and calls player lookup; this inputs slice does not edit it.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: shared aura DTO builder and existing hardcoded duration line.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: current line-empty UnitAura DTO shape.
- `src/lua_api/globals/auras.rs`: existing `collect_visible_unit_auras` / `collect_filtered_unit_auras`, per-unit blocked state and PLAYER classification semantics; no aura producer changes authorized here.
- `src/lua_api/game_data.rs`: existing `AuraInfo`, `PartyMember.buffs` and `PartyMember.debuffs` fields, re-exported through `lua_api::state`.
- `tests/tooltip_unit_buff_security.rs`: explicit host fixtures, real API assertions and rooted authentic-wrapper probes, grouped through existing integration automatic discovery; no Cargo target added.

## Tests asserting this spec

`tests/tooltip_unit_buff_security.rs` defines **18 actual `#[test]` cases**. All are **AUTHORED / UNCOMPILED / UNEXECUTED** at this inputs-only checkpoint; no RED, GREEN, formatting-command, check, native or acceptance evidence is claimed.

| Case | Observable requirement |
|---|---|
| `distinct_player_party1_party2_content_uses_requested_live_unit` | Three distinct concrete host names and current duration/content shape |
| `positive_one_based_index_addresses_filtered_not_raw_player_vector` | Mixed polarity skip and filtered index2 |
| `omitted_and_public_nil_filter_default_to_helpful` | Real default-filter positive matches |
| `player_filter_excludes_other_sources_before_indexing` | Selected player-source names and genuine filtered misses |
| `blocked_filter_reindexes_and_same_instance_on_other_unit_remains_visible` | Real AddBlockedAura state, shifted index, unit isolation and all-blocked miss |
| `missing_or_nonpositive_filtered_index_returns_current_empty_shape` | Empty shape at zero, negative and absent filtered rows |
| `unsupported_units_never_fall_back_to_player` | Concrete unsupported tokens miss despite player content |
| `live_host_mutation_removal_and_environment_isolation` | Name mutation, removal, clear, old DTO and other-store/environment isolation |
| `fresh_payload_and_miss_tables_do_not_alias_host_or_other_results` | Fresh result identity and caller mutation isolation |
| `public_tainted_calls_keep_content_filters_and_caller_taint` | Ordinary addon calls, positive content/filter controls and misses |
| `secure_secret_unit_selects_actual_party_not_player` | Genuine secret party1 token yields party1 content |
| `secure_secret_index_selects_concrete_filtered_row` | Genuine numeric secret selects player/party indices |
| `secure_secret_filter_and_nil_apply_filter_and_default` | Genuine optional secret STRING/NIL controls |
| `secure_combined_secret_selectors_return_meaningful_content_and_true_miss` | All-position secret match/miss controls |
| `tainted_secret_each_position_denied_before_lookup_or_public_parse` | VM denial before missing state or invalid public parsing |
| `tainted_malformed_secret_payload_denied_before_type_or_state_short_circuit` | Malformed secret BOOL/table/Frame and private STRING denial |
| `secure_wrong_types_and_public_wrong_types_reject_with_recovery` | Ordinary type errors and concrete recovery controls |
| `rooted_host_secret_gc_preserves_identity_secrecy_trust_and_fixture_state` | Rooted GC, host wrapper identity/secrecy, original properties and trust |

## Known gaps (current cycle)

- [ ] Main must inspect/commit inputs and obtain genuine asynchronously compiled RED before producer implementation; authoring alone is not failing-test evidence.
- [ ] Current GetUnitBuff discards unit and bypasses filtered live-domain selection. Secure wrapper support and pre-parse authentication await implementation/proof; no outcome predicted as saved evidence.
- [ ] Independent bounded verification and exact342 acceptance/accounting remain pending. This slice changes no accounting, wiki, production code, Cargo target or aura producer.

## Out of scope

- Native client secrecy, unit-aura access/restricted output (**UNMODELED**), native payload/error/miss parity and general-unit/target-identity credit: unavailable evidence and outside this bounded model contract.
- Dynamic aura duration, acquisition/lifecycle producers, new fixtures/stores, aura mutation APIs and other tooltip query registrations: existing state and shared builder are the test boundary.
- HARMFUL/unknown/conflicting filter behavior, fractional/nonfinite/index-width policy, omitted required arguments and broader parser changes: not required by this slice.
- Other source rows, client profiles, full-suite proof, producer implementation, execution gates, delegation and commits: owned by main or explicitly excluded from this inputs-only assignment.
