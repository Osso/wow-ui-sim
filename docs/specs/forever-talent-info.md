# Forever ordinary talent query

Forever `C_SpecializationInfo.GetTalentInfo` reads the character's supplied legacy
seven-tier, three-column grid. [Implementation notes](../wiki/systems/forever-talents.md)
separate verified WM90 behavior from EMULATED scenario inputs.

## What it must do

- [x] Use the queried group's specialization, first exact candidate and last neutral candidate in provider order; never substitute Retail data.
- [x] Publish the 18 WM90 fields, keeping a present result when icon data is zero or absent. Rank, maxRank and previewRank are one.
- [x] Read selected state from the queried group, known and availability from the current group; selected always implies available.
- [x] Return nil for absent player, invalid group/grid/candidate, zero talent ID, missing spell name or unusable inspection context.
- [x] Read all eight named query fields; unused specializationIndex, talentIndex and isPet do not alter lookup. Report missing coordinates with WM90's GetTalent literals.

## How it works

- [Supplied model, data limits and events](../wiki/systems/forever-talents.md).

## Implementation inventory

- `src/c_api/forever_talents.rs`: supplied character state and provider rows.
- `src/c_api/forever_talents/query.rs`: parsing, selection and publication.
- `src/c_api/c_spec.rs`: Forever-only query registration and active group.
- `src/lua_api/talent_state.rs`: per-character state owner.

## Tests asserting this spec

- `tests/forever_talent_info.rs`: original TalentFrame_Update consumer and query contracts using explicit synthetic scenario records.

## Known gaps (current cycle)

- [ ] Exact Forever 70707 table exports and live populated native fixture remain unavailable.

## Out of scope

- Native synchronization/persistence and event timing/payloads: WM90 does not establish them.
- Deprecated global GetTalentInfo and GetTalentInfoBySpecialization: not covered by WM90; no new native behavior claimed.
- Retail query behavior: retained unchanged.
