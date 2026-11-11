# PvP talent tooltip

VERIFIED: Retail `C_TooltipInfo.GetPvpTalent` uses the PvP talent row to build spell content and then adds conditional talent annotations. The cached Retail API declaration is at `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:726-743`; the `SetPvpTalent` mapping is at `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:567`, and a consumer call is at `Blizzard_PlayerSpells/PvPTalents/Blizzard_PvPTalentSlotTemplates.lua:157`. See the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for shared tooltip construction.

## What it must do

### Registration and inputs

- [x] VERIFIED: Retail registers `GetPvpTalent`, and `GameTooltip:SetPvpTalent` reaches it through Blizzard's original tooltip accessor. UNKNOWN: PTR and WowForever registration were not inspected.
- [x] VERIFIED: Successful calls use the required talent ID, default `isInspect` to `false`, default local group to the active group, default `talentIndex` to `0`, and preserve the one-based group and zero-based position inputs.
- [ ] VERIFIED: Invalid argument types, secret-value failures, checked-index errors and the complete usage-error text. The tests cover successful inputs only.
- [ ] VERIFIED: Read `SpellID` and `OverridesSpellID` from the host-supplied talent row without seeding built-in talent records. The tests provide rows but do not query the empty default catalog.

### Spell content and identity

- [x] VERIFIED: Use `SpellID` as the primary spell input. The host-supplied resolver map gives its resolved outcome; an absent entry means identity.
- [x] VERIFIED: A successful tooltip uses type `1` and the resolved nonzero spell ID; the tests do not cover zero-ID publication or miss paths.
- [x] VERIFIED: Zero `OverridesSpellID` adds no replacement line. A nonzero override can add a type-`0` `REPLACES_SPELL` line after ordinary spell content, and the original row's attribute mask `32768` suppresses it.
- [ ] VERIFIED: Missing resolved override-name or original override rows suppress `REPLACES_SPELL`; these row-miss branches are not tested.
- [x] VERIFIED: Populated tooltip data uses the shared data-instance counter, which increments between consecutive calls.

### Local and inspect annotations

- [x] VERIFIED: The talent messages are alternative outcomes, not a fixed suffix. `REPLACES_SPELL` precedes the selected talent message.
- [x] VERIFIED: For local calls, omitted `groupIndex` selects the active group; an explicit current group is accepted, while non-current and unknown groups have no local annotation context.
- [x] VERIFIED: The early `UNLOCKED_AT_LEVEL` line uses `LevelRequired` and `HOSTILE_STATUS_COLOR` without wrapping.
- [x] VERIFIED: Eligibility annotations are suppressed when the talent is present in each of the active group's four positions.
- [x] VERIFIED: Eligibility checks use supported zero-based positions `0..=3` and the supplied position mask. A supplied absent helper result adds no line.
- [x] VERIFIED: Supplied eligibility success appends `TALENT_TOOLTIP_ADDPREVIEWPOINT` with `FRIENDLY_STATUS_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A supplied later level failure appends `UNLOCKED_AT_LEVEL` with `HOSTILE_STATUS_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A supplied item-grant rejection uses a catalogued item row; it adds blank rows and either `TALENT_GRANTED_BY_ITEM` or the host-confirmed essence message. The lines use `REFUND_TOOLTIP_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A valid item-grant result takes priority over combat. Otherwise, combat selects `TALENT_TOOLTIP_ADD_COMBAT_ERROR` with `HOSTILE_STATUS_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A failed rest predicate appends `TALENT_TOOLTIP_ADD_REST_ERROR_PVP`; the rest state and explicit resting-player paths both prevent this line. The failure line uses `HOSTILE_STATUS_COLOR`, type `0`, and wrapping. INFERRED: The simulator maps native state to `PlayerXpState.rest_state`, `PlayerState.in_combat`, and `PlayerState.is_resting`.
- [ ] UNKNOWN: The rest-error message is also the default for other eligibility failures, but their predicates are not fully known. This implementation leaves those outcomes unmodelled.
- [x] VERIFIED: Inspect mode bypasses local group, selection, level and eligibility checks. It reads the granting item from current inspect GUID context, not local PvP state; a supplied item result appends `TALENT_GRANTED_BY_ITEM` with `ITEM_LEGENDARY_COLOR`, type `0`, and wrapping. An absent inspect granting-item result adds no line.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) — shared data tables, lines and instance IDs.

## Implementation inventory

- `src/c_api/c_tooltip_info_pvp_talent.rs` — Retail registration, host inputs, resolved spell selection and annotation composition.
- `src/c_api/tooltip_getter_args.rs` — shared secret-aware numeric, boolean and index readers.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — C_TooltipInfo registration and existing spell-tooltip bridge.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs` — shared spell tooltip producer.
- `src/c_api/c_spec.rs` — active PvP talent group publication.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — empty-by-default host state.

## Tests asserting this spec

- `tests/tooltip_pvp_talent.rs` — Retail registration; original `GameTooltip:SetPvpTalent` consumer; spell identity, replacement line and suppression; local level, group, position, eligibility, combat and rest branches; item, essence and inspect grant branches.

## Known gaps (current cycle)

- [ ] UNKNOWN: The native final Lua result for a missing talent row, zero talent ID, failed base-spell presence check, missing player, or missing resolved spell-name row. The simulator uses the shared EMULATED zero-value `unknown_getter_failure` seam; this is not a claim of native parity. Tests do not assert these paths.
- [ ] UNKNOWN: Whether the native base-spell presence provider can report `SpellID` zero as present. This path is not modeled or tested.
- [ ] UNKNOWN: The native spell override ranking walk's exact ranked IDs, map writers and precedence among talent, aura and SpellReplace sources. The host supplies the resolved ID; this implementation does not infer the walk.
- [ ] UNKNOWN: The complete PvP talent catalog, effect-parameter decoder, essence lookup, inspect lifecycle, additional progression restrictions and remaining eligibility failure reasons. Hosts must supply rows and observed helper results; absent inputs do not fabricate lines.
- [ ] VERIFIED: When the primary spell producer misses because the local player or spell-name row is unavailable, the native PvP caller still runs talent annotations. UNKNOWN: Its final result and table shape; this failure path stays unmodelled and untested here.
- [ ] VERIFIED: Native `isInspect` also changes the ordinary spell request. UNKNOWN: The existing simulator producer has no matching observable request field, so this effect stays unmodelled.
- [ ] UNKNOWN: Numeric RGB defaults, the complete ordinary spell line and field set, and a getter-specific event contract.
- [ ] INFERRED: The simulator maps native attribute mask `32768` to the current spell row's `attributes_9` value.
- [ ] INFERRED: The existing ordinary spell producer cannot represent the native inspect flag in its request. No extra request field is added until it has an observable simulator contract.

## Out of scope

- UNKNOWN: PTR and WowForever registration and behavior.
- Invented PvP talent, aura, essence, item-grant or resolved-spell rows.
- Native miss-result parity and UNKNOWN eligibility failure messages.
