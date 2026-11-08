# Mists engraving query inputs

The normal Mists bag and settings consumers read explicit environment-local engraving inputs. The C API implementation belongs in `src/c_api`; [the investigation](../wiki/investigations/mists-engraving-inputs.md) records consumer selection and proof limits.

## What it must do

- [x] Publish the seven members reached by the normal Mists bag/settings flow, gated by `client_profile::ACTIVE`. The six non-availability members remain Mists-only; [Era/Anniversary availability](era-engraving-availability.md) has a separate contract.
- [x] Project supplied availability, engraving mode, optional current rune, per-inventory-slot eligibility, optional per-slot rune, and current-cast per-slot eligibility. An unsupplied input must raise an explicit error naming the query; supplied false/nil is not missing input. Mists seeds the native default state at startup so the normal consumers see false/false/nil without host input (see below).
- [x] Return the pinned `EngravingData` fields and independent Lua tables. `RefreshRunesList` records an observable simulator refresh request with no Lua results; it does not invent native catalog refresh effects.
- [x] Drive the original selected `ContainerFrame_Shared.lua` overlay and desaturation functions, with input supplied before addon loading. Cover true/false, present/nil, slot discrimination, repeated refresh requests and environment isolation. The seeded defaults also let the unchanged bank and mail storage flows open bags without an engraving Lua error. The later settings lane proves complete original `AdjustDisplaySettings` construction with supplied true/false, typed CVar behavior, unset recovery and isolation.
- [x] Keep the old storage-flow tests unchanged. With the seeded native defaults they reach original `ContainerFrame_Shared.lua:315` and pass; clearing a supplied field still raises the named error.

## How it works

- [Consumer and declaration inventory](../wiki/investigations/mists-engraving-inputs.md)
- [Shared publication and Rust input supply; no Lua admin setter](../wiki/investigations/mists-engraving-inputs.md#shared-availability-and-input-supply)

## Implementation inventory

- `src/c_api/c_engraving.rs` — ACTIVE-gated Mists projections, shared Mists/Era/Anniversary availability, named missing-input errors, fresh rune tables and local refresh-request recording.
- `src/c_api/engraving_inputs.rs` — explicit state, the `mists_native_defaults()` seed and pinned record shape.
- `src/lua_api/state.rs` — Mists-only startup seed (`mists_native_defaults()`); `src/lua_api/state/sim_state.rs` holds environment-local state.

## Tests asserting this spec

- `tests/mists_engraving.rs` — supplied original consumers, direct projections, missing inputs and isolation.
- `tests/mists_engraving_settings.rs` — original settings line138, both availability branches, typed CVar behavior and named unset recovery.
- `tests/mists_bank_storage_panel.rs` — unchanged historical storage-flow boundary.

## Known gaps (current cycle)

- [x] Execute producer-subtracted RED and supplied GREEN with byte-identical tests: corrected RED `e97b841e7`, GREEN `eb3dca9dc`; all three exact GREEN cases pass.
- [x] Resolve the earlier display-settings blocker through the [merged original-consumer settings proof](../wiki/investigations/mists-engraving-inputs.md#merged-settings-proof-and-stock-effects). Full SettingsRegistrar and visible-panel interaction remain unproved.
- [ ] Establish native availability values; supplied observations do not establish defaults or seasonal policy.
- [x] Seed the INFERRED Mists native defaults reached by the bag/settings flow: availability false, engraving mode false, current rune nil. `EngravingInfoDocumentation.lua` declares a zero-argument bool availability getter (:179–185, no setter), a zero-argument bool mode getter plus setter (:73–79, :273–281), and a nilable `EngravingData` current-rune getter (:64–70). The only pinned producer is `Blizzard_EngravingUI/Blizzard_EngravingUI.lua:86,99`, which sets the mode true/false from the LoadOnDemand frame's OnShow/OnHide, and `Blizzard_EngravingUI.toc:2` says “Rune engraving for seasonal content”. Mists is not a seasonal realm and does not open that frame at startup, so the mode is false, availability is false and no rune cast exists. These values are INFERRED, not observed from a native client; supplied state still overrides and a cleared field still raises the named error.
- [x] Compare bound stock maps: the historical bag checkpoint `c50e6386a`/`f7a9227b8` had unchanged6/12. See the [current merged stock comparison](../wiki/investigations/mists-engraving-inputs.md#merged-settings-proof-and-stock-effects) for the later checkpoint.
- [x] All seven full-feature all-target checks pass with zero JSON/plain warnings. Mists checks `eb3dca9dc`; the other six check documentation-only successor `81d59b62c`, with identical Rust/test inputs.

## Out of scope

- LoD `Blizzard_EngravingUI`: loadable on Mists but no normal Mists UI entry.
- [Era/Anniversary availability](era-engraving-availability.md) has its own supplied-state contract; their remaining engraving system is excluded here. Wrath has no pinned engraving callers.
- Invented native inactive defaults, catalog/filter policy, casting transitions and refresh side effects. The three seeded values are INFERRED from the pinned declarations, the seasonal-content TOC note and the only pinned producer, not observed from a native client. Other engraving members remain supplied-only.

## Pinned declarations

Each citation is relative to Mists `Blizzard_APIDocumentationGenerated/EngravingInfoDocumentation.lua`: `GetEngravingModeEnabled` :73–79; `SetEngravingModeEnabled` :273–281; `GetCurrentRuneCast` :64–70; `IsEngravingEnabled` :179–185; `RefreshRunesList` :269–271; `IsInventorySlotEngravable` :211–223; `GetRuneForInventorySlot` :135–147; `IsInventorySlotEngravableByCurrentRuneCast` :226–238; `EngravingData` :329–340. These declare results and fields, not native defaults.
