# PvP talent tooltip

VERIFIED: Retail `C_TooltipInfo.GetPvpTalent` uses the PvP talent row to build spell content and then adds conditional talent annotations. See the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for shared tooltip construction.

## What it must do

### Registration and inputs

- [ ] VERIFIED: Register `GetPvpTalent` on Retail only. PTR and WowForever were not inspected.
- [ ] VERIFIED: Read required signed `talentID`, optional boolean `isInspect` (default `false`), optional checked one-based `groupIndex`, and optional signed `talentIndex` (default `0`). Use `Usage: local data = C_TooltipInfo.GetPvpTalent(talentID [, isInspect, groupIndex, talentIndex])`.
- [ ] VERIFIED: Read `SpellID` and `OverridesSpellID` from the host-supplied talent row. The inspected CSV export had no `PvpTalent` table; do not create built-in talent records.

### Spell content and identity

- [ ] VERIFIED: Use `SpellID` as the primary spell input. The host-supplied resolver map gives its resolved outcome; an absent entry means identity.
- [ ] VERIFIED: Publish spell tooltip type `1` and the resolved spell ID only after the player and resolved spell-name row are available. Omit `id` when it is zero.
- [ ] VERIFIED: Use nonzero `OverridesSpellID` only for `REPLACES_SPELL`. Resolve it separately and exclude the original `SpellID` as a candidate. Require the resolved name row and the original override row; suppress the line when the original row carries attribute mask `32768`. Append the type-`0` line after ordinary spell lines.
- [ ] VERIFIED: Stamp populated tooltip data with the shared data-instance counter.

### Local and inspect annotations

- [ ] VERIFIED: The five talent messages are alternative outcomes, not a fixed suffix. `REPLACES_SPELL` is inside ordinary spell content and precedes the selected talent message.
- [ ] VERIFIED: For a local query, require the player and current local group. An explicit non-current or unknown group has no local annotation context. Omitted `groupIndex` selects the active group.
- [ ] VERIFIED: Emit the early `UNLOCKED_AT_LEVEL` line when the player's signed level is below `LevelRequired`; format it with that level, use `HOSTILE_STATUS_COLOR`, and do not wrap it.
- [ ] VERIFIED: Suppress eligibility annotations when the talent is present in any of the active group's four positions.
- [ ] VERIFIED: Evaluate only a supported zero-based talent position `0..=3` allowed by the supplied position mask. The external eligibility-helper result is host supplied; an absent result emits no eligibility message.
- [ ] VERIFIED: On supplied eligibility success, append `TALENT_TOOLTIP_ADDPREVIEWPOINT` with `FRIENDLY_STATUS_COLOR`, line type `0`, and wrapping enabled.
- [ ] VERIFIED: On a supplied later level failure, append `UNLOCKED_AT_LEVEL` with `HOSTILE_STATUS_COLOR`, line type `0`, and wrapping enabled.
- [ ] VERIFIED: On a supplied item-grant rejection, use the host item candidate only when its item row exists. Add the blank rows, then either the item-name `TALENT_GRANTED_BY_ITEM` line or, when the host confirms an essence lookup, `PVP_TALENT_GRANTED_BY_AZERITE_ESSENCE`. Both use `REFUND_TOOLTIP_COLOR`, line type `0`, and wrapping.
- [ ] VERIFIED: An item-grant rejection takes priority over the combat flag. Otherwise, combat selects `TALENT_TOOLTIP_ADD_COMBAT_ERROR` with `HOSTILE_STATUS_COLOR`, line type `0`, and wrapping.
- [ ] VERIFIED: When the rest predicate fails, append `TALENT_TOOLTIP_ADD_REST_ERROR_PVP` with `HOSTILE_STATUS_COLOR`, line type `0`, and wrapping. INFERRED: The simulator maps the native signed state and player flags to `PlayerXpState.rest_state`, `PlayerState.in_combat`, and `PlayerState.is_resting`.
- [ ] UNKNOWN: The rest-error message is also the default for other eligibility failures, but their predicates are not fully known. This implementation leaves those outcomes unmodelled.
- [ ] VERIFIED: In inspect mode, bypass local group, selection, level and eligibility checks. Read the granting item from the current inspect GUID context, not local PvP state; require a nonzero item ID and item row, then append `TALENT_GRANTED_BY_ITEM` with `ITEM_LEGENDARY_COLOR`, line type `0`, and wrapping.

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

- [ ] UNKNOWN: The final Lua result for a missing talent row, zero talent ID, failed base-spell presence check, missing player, or missing resolved spell-name row. Tests do not assert these paths.
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
