# Forever specialization visibility

Forever exposes specialization queries through `C_SpecializationInfo`, without the legacy `GetSpecialization`/`GetSpecializationInfo` capability pair. The producer is `src/lua_api/globals/state_backed_queries.rs`.

## What it must do

- [ ] Leave both legacy globals absent on Forever, including ordinary global lookup and bootstrap replay.
- [ ] Let EllesmereUI's existing reminder, nameplate and profile guards skip legacy specialization calls without errors.
- [ ] Preserve the modeled namespace's active-index and specialization-identity queries.
- [ ] Leave other profiles' existing `GetSpecialization` registration unchanged.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md) — active profile selection.
- [Ellesmere investigation](../wiki/investigations/ellesmereui-forever.md) — parent-owned runtime investigation.

## Implementation inventory

- `src/lua_api/globals/state_backed_queries.rs` — excludes the legacy getter's producer on Forever; no post-load deletion.
- `tests/wowforever_profile.rs` — exact guard, namespace-state and bootstrap-replay regressions in the grouped integration target.

## Tests asserting this spec

The three `wowforever_specialization_visibility_*` tests in `tests/wowforever_profile.rs` reproduce the legacy guards and retain concrete namespace output for modeled Paladin specialization fixtures. These fixtures do not establish native Forever specialization IDs.

## Evidence

Pinned Forever 1.60.1.69913 `Blizzard_DeprecatedSpecialization/Blizzard_DeprecatedSpecialization.toc` permits only `classic, standard`; Camelot does not load its alias publisher. The namespace remains defined by `SpecializationInfoDocumentation.lua`.

Cached EllesmereUI v9.2.2 (CurseForge file `8936131`) guards `GetSpecialization` before using `GetSpecializationInfo`: `EllesmereUIAuraBuffReminders.lua:114–116`, `EllesmereUINameplates.lua:9180–9182`, and `EllesmereUI_Profiles.lua:4006–4008`. Publishing only the first global falsely enables this legacy branch.

## Known gaps (current cycle)

- [ ] Targeted GREEN and parent-owned final verification pending.

## Out of scope

No missing alias is synthesized, no excluded vendor TOC is loaded, and no addon code or nil guard is changed. Namespace specialization data, other legacy APIs, and native specialization-content fidelity are unchanged.
