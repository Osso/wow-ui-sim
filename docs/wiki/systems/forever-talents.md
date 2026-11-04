# Forever ordinary talents

## Contract and evidence

The [query contract](../../specs/forever-talent-info.md) follows WM90
(`native-evidence-70688/WM90-forever-get-talent-info.md`). The pinned Forever
70707 `Blizzard_APIDocumentationGenerated/SpecializationInfoDocumentation.lua:316-329`
declares the query-table API. `Blizzard_FrameXML/Blizzard_FrameXML.toc:98`
selects `[Family]/TalentFrameBase.lua` for mainline. Forever's active game tags
are camelot/mainline (`src/toc/mod.rs`), and Family resolves to Mainline.
The original `Mainline/TalentFrameBase.lua:63-122` queries and displays cells.

## Supplied character model

Verified 2026-11-04: neither repository data nor `game-engine/data/db2/`
contains the required Forever table exports. `TalentState.forever` is therefore
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
talent IDs. Inspection uses a GUID-matched supplied group snapshot; token
resolution uses the existing unit identity model. Other target descriptor
representations remain UNKNOWN.

Missing coordinates raise diagnostics using the exact WM90 GetTalent literals.
WM90 does not establish whether the protected native wrapper raises or returns
nil; retaining explicit simulator errors is an EMULATED policy, not native proof.
Numeric fields accept the existing numeric range and subtract before truncation;
string coercion and unusual native conversions remain UNKNOWN.

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
