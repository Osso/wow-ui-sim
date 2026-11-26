# PvP talent tooltip

VERIFIED: Retail `C_TooltipInfo.GetPvpTalent` uses the PvP talent row to build spell content and then adds conditional talent annotations. The cached Retail API declaration is at `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:726-743`; the `SetPvpTalent` mapping is at `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:567`, and a consumer call is at `Blizzard_PlayerSpells/PvPTalents/Blizzard_PvPTalentSlotTemplates.lua:157`. **VERIFIED (Forever 1.60.1.70707, R88n.8):** Forever declares the getter at `TooltipInfoDocumentation.lua:726-743`, uses Lua truth conversion for `isInspect`, and returns zero Lua values when its PvP talent record is missing. See the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for shared tooltip construction.

## What it must do

### Registration and inputs

- [x] VERIFIED: Retail registers `GetPvpTalent`; PTR remains unchanged. Retail's `GameTooltip:SetPvpTalent` reaches it through Blizzard's original accessor.
- [x] VERIFIED: Forever registers `GetPvpTalent`; its missing-record result and argument conversion are covered by the optional Forever test.
- [x] VERIFIED: Forever requires a signed-32 talent ID; numeric strings are accepted and fractions truncate toward zero after range validation. Omitted/nil optional inputs are accepted, `isInspect` uses Lua truth conversion, and the supplied group index uses Forever's `luaIndex` reader.
- [ ] UNKNOWN (Forever): R88n.8 does not define the result when the decremented, truncated `groupIndex` is outside signed-32 range. The Rust saturating cast is only a neutral placeholder, not native-parity evidence.
- [x] VERIFIED: Forever Usage is `Usage: local data = C_TooltipInfo.GetPvpTalent(talentID [, isInspect, groupIndex, talentIndex])`.
- [x] VERIFIED: A missing Forever `PvpTalent` record returns zero Lua values. The simulator has no Forever catalog and does not publish Retail rows.
- [x] INFERRED (Forever simulator input): `SimState.forever_pvp_talent_spell_ids` is an empty-by-default host map from `talentID` to `SpellID` and uses the shared spell-tooltip producer. R88n.8 does not prove this field mapping, a native writer or populated native-record parity.
- [ ] UNKNOWN: Secret-value errors, all invalid argument diagnostics and the complete wrapped Usage error. The optional test checks that the Usage text is present for missing required input.
- [x] VERIFIED: Retail starts with generated `SpellID`, `OverridesSpellID`, `LevelRequired`, and category `TalentSlotMask` rows; tests check actual default rows and neutral host-only grant/eligibility inputs. The local export is installed build 12.1.5.70783, not the selected 12.1.5.70575 pin.

### Spell content and identity

- [x] VERIFIED: Use `SpellID` as the primary spell input. The host-supplied resolver map gives its resolved outcome; an absent entry means identity. R53 confirms `OverridesSpellID` is only used by the conditional replacement annotation and is not a fallback.
- [x] VERIFIED: A successful tooltip uses type `1` and the resolved nonzero spell ID; the tests do not cover zero-ID publication or miss paths.
- [x] VERIFIED: Zero `OverridesSpellID` adds no replacement line. A nonzero override can add a type-`0` `REPLACES_SPELL` line after ordinary spell content, and the original row's attribute mask `32768` suppresses it.
- [ ] VERIFIED: Missing resolved override-name or original override rows suppress `REPLACES_SPELL`; getter-level tests cover both row-miss branches, but no selected original PvP talent consumer test proves them.
- [x] VERIFIED: Populated tooltip data uses the shared data-instance counter, which increments between consecutive calls.

### Local and inspect annotations

- [x] VERIFIED: The talent messages are alternative outcomes, not a fixed suffix. `REPLACES_SPELL` precedes the selected talent message.
- [x] VERIFIED: For local calls, omitted `groupIndex` selects the active group; an explicit current group is accepted, while non-current and unknown groups have no local annotation context.
- [x] VERIFIED: The early `UNLOCKED_AT_LEVEL` line uses `LevelRequired` and `HOSTILE_STATUS_COLOR` without wrapping.
- [x] VERIFIED: Eligibility annotations are suppressed when the talent is present in each of the active group's four positions.
- [x] VERIFIED (R73): The category mask tests the requested index shifted by its low five bits; current-slot access is guarded to `0..=3`. Tests cover supported positions and masking. UNKNOWN: general out-of-range requested-position results are not asserted. Absent helper and permission inputs retain neutral output.
- [x] VERIFIED: A supplied eligibility pass after required/later level and progression checks, grant rejection, combat and permission checks appends `TALENT_TOOLTIP_ADDPREVIEWPOINT` with `FRIENDLY_STATUS_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A supplied later level failure appends `UNLOCKED_AT_LEVEL` with `HOSTILE_STATUS_COLOR`, type `0`, and wrapping.
- [x] VERIFIED: A supplied item-grant rejection uses a catalogued item row; it adds blank rows and either `TALENT_GRANTED_BY_ITEM` or the host-confirmed essence message. The lines use `REFUND_TOOLTIP_COLOR`, type `0`, and wrapping.
- [x] VERIFIED (R88at.1/at3): Eligibility selects one failure in order: level/progression, grant rejection, combat, rest-like condition, then final player-state restriction. The later display dispatcher checks combat before grant and late unlock, but this does not reorder the earlier eligibility result. Combat selects `TALENT_TOOLTIP_ADD_COMBAT_ERROR` with `HOSTILE_STATUS_COLOR`, type `0`, and wrapping when combat is the selected status.
- [x] INFERRED (simulator mapping): Available supplied inputs model late unlock before grant rejection before combat; the tests assert each combined case.
- [x] VERIFIED (R88at.1/at3): The display dispatcher uses `TALENT_TOOLTIP_ADD_REST_ERROR_PVP` for rest and other selected failure statuses not handled by its combat, grant or late-unlock branches.
- [x] INFERRED (simulator mapping): Host `rest_permission: Option<bool>` and `eligibility_by_position` supply outcomes without mapping native XP/resting fields. `None` with no explicit eligibility result preserves neutral output; supplied permission permits preview or rejects with a rest error. This host seam is not a native default.
- [ ] UNKNOWN: The progression gate, talent/GUID rejection predicate, indirect Flags-bit reads and final player-state helper remain unmodelled; tests supply eligibility outcomes, not guessed predicates.
- [x] VERIFIED: Inspect mode bypasses local group, selection, level and eligibility checks. It reads the granting item from current inspect GUID context, not local PvP state; a supplied item result appends `TALENT_GRANTED_BY_ITEM` with `ITEM_LEGENDARY_COLOR`, type `0`, and wrapping. An absent inspect granting-item result adds no line.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) — shared data tables, lines and instance IDs.

## Implementation inventory

- `src/c_api/c_tooltip_info_pvp_talent.rs` — argument parsing, Retail catalog/producer, and Forever's supplied spell-tooltip path.
- `src/c_api/c_tooltip_info_pvp_talent_data.rs` — generated Retail catalog rows.
- `tools/gen_pvp_talent_catalog.py` and `data/pvp-talents/retail-12.1.5.70783/` — projection generator, CSV and source provenance.
- `src/c_api/tooltip_getter_args.rs` — shared numeric readers and Forever's numeric-string `luaIndex` conversion.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs` — C_TooltipInfo registration and existing spell-tooltip bridge.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs` — shared spell tooltip producer.
- `src/c_api/c_spec.rs` — active PvP talent group and Retail slot-info registration.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — preserves only the old no-host slot result pending native defaults; the modeled Retail slot producer replaces its Lua slot registration.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — Retail catalog inputs and the separate empty-by-default Forever spell-ID map.

## Tests asserting this spec

- `tests/tooltip_pvp_talent.rs` — optional R73 tests drive original slot hover and the original list template hover (the panel harness does not realize virtualized list rows), with a pre-load pass-through spy, visible lines, mutually exclusive outcomes and primary getter metadata; supplied slot fields share group selection and preserve no-host defaults. Retail registration; a `TestGetPvpTalent` getter-level wrapper around `GameTooltip:SetPvpTalent` with a pass-through getter spy; generated default catalog values and neutral grant/eligibility inputs; spell identity, replacement line and suppression; getter-level missing resolved replacement-name and original replacement-row cases (not yet proved through the selected PvP talent consumer); local level, group, position, eligibility, combat and rest branches; item, essence and inspect grant branches.
- `tests/tooltip_full_env_helpers.rs` — optional pre-addon setup hook installs the getter spy before `TooltipDataHandler` copies the namespace.
- `tests/tooltip_forever_four_getters.rs` — optional Forever registration, argument acceptance, Usage text, two supplied-hit cases and a missing-record case.
- `tools/test_gen_pvp_talent_catalog.py` — projected-column, row validation, and generated Rust output.

## Known gaps (current cycle)

- [ ] UNKNOWN (Retail): The native final Lua result for zero talent ID, failed base-spell presence, missing player or missing resolved spell-name row. The simulator uses the shared EMULATED zero-value seam for these Retail paths.
- [ ] INFERRED (Forever): The simulator's supplied `talentID`-to-`SpellID` map does not establish current-player selection, native writers, inspect behavior or native populated-record parity. It remains empty by default; no talent rows are seeded.
- [ ] UNKNOWN: Whether the native base-spell presence provider can report `SpellID` zero as present. This path is not modeled or tested.
- [ ] UNKNOWN: The native spell override ranking walk's exact ranked IDs, map writers and precedence among talent, aura and SpellReplace sources. The host supplies the resolved ID; this implementation does not infer the walk.
- [ ] UNKNOWN: Native alignment of the installed 12.1.5.70783 rows with selected build 12.1.5.70575, the effect-parameter decoder, essence lookup, inspect lifecycle, additional progression restrictions and remaining eligibility failure reasons. Grant and eligibility outcomes remain host-only; absent inputs do not fabricate lines.
- [x] VERIFIED (R73): Active-group membership uses four shared selected positions. The original slot hover supplies selected talents; the original list hover supplies unselected candidates.
- [x] VERIFIED (cached declaration): `GetPvpTalentSlotInfo` uses the shape at `SpecializationInfoDocumentation.lua:153-165,491-499`. Supplied slot records contain enabled/level/available IDs; selectedTalentID comes only from shared groups. UNKNOWN: native default values. The old empty default remains isolated as a temporary workaround, unchanged for calls without supplied records.
- [ ] VERIFIED: When the primary spell producer misses because the local player or spell-name row is unavailable, the native PvP caller still runs talent annotations. UNKNOWN: Its final result and table shape; this failure path stays unmodelled and untested here.
- [ ] VERIFIED: Native `isInspect` also changes the ordinary spell request. UNKNOWN: The existing simulator producer has no matching observable request field, so this effect stays unmodelled.
- [ ] UNKNOWN: Numeric RGB defaults, the complete ordinary spell line and field set, and a getter-specific event contract.
- [ ] INFERRED: The simulator maps native attribute mask `32768` to the current spell row's `attributes_9` value.
- [ ] INFERRED: The existing ordinary spell producer cannot represent the native inspect flag in its request. No extra request field is added until it has an observable simulator contract.

## Out of scope

- PTR registration and behavior. Forever native catalog population, player-state writers and selected PvP UI consumers are not modeled; only explicit simulator inputs are available.
- Native learning, progression and effect-list internals; host records expose only supplied outcomes.
- Invented PvP talent, aura, essence, item-grant or resolved-spell rows.
- Native miss-result parity and UNKNOWN eligibility failure messages.

## Evidence boundary

VERIFIED: R73 and R88at describe Retail 12.1.5.70688 builder and eligibility decisions; the catalog export remains labeled 12.1.5.70783. INFERRED: these decisions apply to the 70783 runtime cache. INFERRED: native Retail field-name mapping (field 7 selects category, field 8 supplies the level, fields 3/4 feed ordinary spell/replacement content). The separate Forever talent-to-spell input mapping is also INFERRED and is not seeded. UNKNOWN: native final miss results and inspect-unit details remain outside this change. The existing Retail inspect seam is preserved, not extended.
