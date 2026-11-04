# Forever ordinary talents

## Contract and evidence

The [query contract](../../specs/forever-talent-info.md) follows WM90
(`native-evidence-70688/WM90-forever-get-talent-info.md`). The pinned Forever
70707 `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:316-329`
declares the query-table API. `Blizzard_FrameXML/Blizzard_FrameXML.toc:96`
selects `[Family]/TalentFrameBase.lua` for mainline. Forever's active game tags
are camelot/mainline (`src/toc/mod.rs`), and Family resolves to Mainline.
The original `Mainline/TalentFrameBase.lua:63-122` queries and displays cells.

## Supplied character model

Verified 2026-11-04: neither repository data nor `game-engine/data/db2/`
contains the required Forever 70707 table exports (older Forever directories
and Retail 70391 exports are not substituted). `TalentState.forever` is therefore
an **EMULATED supplied-state seam**, populated through `WowLuaEnv::state()`.
It defaults to an empty snapshot, not a fabricated populated character.
Tests supply explicitly synthetic records and two distinct group selections;
their IDs, names and icons are not claimed as native Forever data.

`rows` preserves provider order for Talent ID/ClassID/TierID/ColumnIndex/SpecID/
SpellID selection. `spell_names` supplies SpellName joins, including empty names.
Removing a row removes its grid candidate; removing the spell name makes a
chosen candidate absent. `spell_icons` supplies an already resolved
SpellXSpellVisual icon and SpellMisc icon; visual provider/difficulty selection
is UNKNOWN. Zero icons publish nil, not a missing record.

Groups keep specialization, selected IDs and tier mask separate. Direct stored
allowance takes precedence; otherwise supplied NumTalentsAtLevel records limit
effective NumTalents by the base-level class cap (classes 6 and 12 use their
named columns). Effective/base lookup inputs are EMULATED because WM90 only
infers those level meanings. Aura grants are pre-extracted enabled type-224
talent IDs; explicit aura subjects use a supplied GUID-keyed grant map. Inspection uses a GUID-matched supplied group snapshot; token
resolution uses the existing unit identity model. Other target descriptor
representations remain UNKNOWN.

Missing coordinates raise diagnostics using the exact WM90 GetTalent literals.
WM90 does not establish whether the protected native wrapper raises or returns
nil; retaining explicit simulator errors is an EMULATED policy, not native proof.
Numeric fields accept the existing numeric range and subtract before truncation;
string coercion and unusual native conversions remain UNKNOWN.

## Loaded combat configurations

The [spec-group configuration query](../../specs/forever-spec-group-config.md)
uses a separate ordered `loaded_configs` vector in the same Forever state. Each
record supplies its ID, combat-kind predicate and combat flags. WM99 establishes
first-match selection with flag 1 for group 1 and flag 8 for other accepted
indices. Zero IDs remain present; no match returns nil. This is independent of
legacy talent groups and the separately stored active configuration ID. The
unseeded vector is empty; live native population and update timing remain unknown.

## Trait-group display rows

[WM101 display projection](../../specs/forever-trait-group-display.md) stores
ordered relations separately from supplied SkillLine names/icons under
`group_display`. Missing joins retain their relation row with empty text and zero
icon. There is no implicit populated catalog and no sorting by orderIndex. One
named inferred seam maps absent tree relations to an empty provider. Native
relation acquisition and live provider behavior remain unknown.

## Trait-group currency snapshots

[WM102 group currency](../../specs/forever-trait-group-currency.md) selects a
supplied configuration's positive-rank node memberships and checks requested
known groups for a common tree. Separate supplied accumulator and tree-currency
orders preserve provider traversal without sorting or echoing requests. Resolved
cost/holdings snapshots are an explicit UNKNOWN acquisition boundary; the model
does not derive unknown native per-node merges or source eligibility. Missing
populated sources raise named diagnostics. Native empty cases use the separately
labelled inferred public empty-array result. The unchanged original Camelot
ClassTalents tab-switch tooltip regression passes with this query.

## Event delivery seam

`A_Admin.FireEvent` is the single host-controlled notification seam. Reading
GetTalentInfo or replacing a supplied snapshot emits no event. WM90 establishes
conditional PLAYER_TALENT_UPDATE and ACTIVE_TALENT_GROUP_CHANGED emitters but
not payloads, timing, suppression rules or mutation transitions. No fabricated
refresh notifications are added; those unknown transitions remain outside this
read-only query.

## Legacy globals

`Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc:3`
allows classic/standard, not camelot. Its Standard wrapper at lines 36-52 is
therefore not runtime-selected for Forever. The simulator's legacy nil globals
remain untouched: WM90 establishes only the C namespace query-table contract,
not GetTalentInfoBySpecialization or positional legacy-global semantics.

## Sources

- WM90 handoff evidence, Forever 70707 original-image behavior and explicit UNKNOWN limits.
- [Producer](../../../src/c_api/forever_talents/query.rs) and [tests](../../../tests/forever_talent_info.rs).

## See Also

- [[forever-addon-startup-compatibility]] — the icon provider that first exposed the stub.
