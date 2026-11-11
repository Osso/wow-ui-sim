# Mists engraving inputs

The [bounded contract](../../specs/mists-engraving-inputs.md) covers the seven normal Mists bag/settings members with supplied state, not invented native inactive values. This page is the source of truth for shared availability and the merged Mists settings proof.

## Shared availability and input supply

Verified: 2026-10-26, merged master `35dedb8b6` (including Era lane `f8736cc49`). `src/c_api/c_engraving.rs::register` publishes `C_Engraving.IsEngravingEnabled` on Mists, Era and Anniversary only. Retail, PTR, Wrath and Forever do not receive this modeled API. The other six modeled members are Mists-only.

The reader returns one environment-local supplied boolean from `EngravingInputs.enabled: Option<bool>` in `src/c_api/engraving_inputs.rs`. `None` raises exactly `C_Engraving.IsEngravingEnabled: unsupplied input`; `Some(false)` is a supplied value, not an unset value. Rust callers supply it through simulator state, for example `env.state().borrow_mut().engraving.enabled = Some(true)`, as in `tests/mists_engraving_settings.rs` and `tests/era_engraving.rs`. No Lua admin setter exists. No startup seed, inactive default or seasonal policy is inferred. Native availability values remain unknown on all three profiles.

### Merged settings proof and stock effects

`tests/mists_engraving_settings.rs` exercises the selected original `Classic/InterfaceOverrides.lua:138` through complete `AdjustDisplaySettings` construction. GREEN `3cb3c664f` and producer-subtracted RED `f20300e2d` have byte-identical tests. Both opposite-order selectors fail RED at the missing member and pass GREEN with supplied true/false, real rune-checkbox counts, tail preview-control construction, typed CVar reads/writes/rejection and environment isolation. A third GREEN selector proves the named unset error at original138 and supplied recovery. These observations reject either constant boolean producer. The earlier missing-transmog display-settings blocker is resolved; full SettingsRegistrar and visible-panel interaction remain unproved.

| Profile | Bound stock unique keys / occurrences | Evidence scope |
|---|---|---|
| Mists | 8/14 before and after; raw key/count maps identical | Paired settings-lane binaries; not clean startup |
| Era | 0/0 → 2/2 | [Original OnLoad unset error and reporter echo](era-engraving-availability.md#evidence-and-limits) |
| Anniversary | 0/0 → 2/2 | Same two individually attributed keys as Era |

The Mists comparison inherits two ScenarioFinder missing-GetMinLevel keys replaced by named unsupplied-input keys relative to the older global-strings reference, with totals unchanged; this is not an engraving delta. The seven final settings-lane all-target checks pass with zero JSON/plain compiler warnings/errors; they prove compilation, not seven-profile runtime parity. No new runtime run is claimed for this documentation reconciliation.

## Selected consumers

In the pinned Mists UI, `Blizzard_UIPanels_Game_Classic.toc:67–69` selects `Mists/ContainerFrame.lua`, `Classic/ContainerFrame_Shared.lua` and `Classic/ContainerFrame.xml`. The shared Lua calls:

| Member | Call lines | Consumed result |
|---|---|---|
| GetEngravingModeEnabled | 54,95,315 | Enables rune overlay branch |
| GetCurrentRuneCast | 316 | Nonnil enables targeting desaturation |
| RefreshRunesList | 1078 | No result used; conditional on alwaysShowRuneIcons |
| IsEngravingEnabled | 1088 | Gates overlay and slot lookup |
| IsInventorySlotEngravable | 1088 | Gates per-slot rune projection |
| GetRuneForInventorySlot | 1089 | Optional iconTexture; nil uses original empty-slot texture |
| IsInventorySlotEngravableByCurrentRuneCast | 1111 | Negated into item-icon desaturation |

`Blizzard_SettingsDefinitions_Frame_Classic.toc:8` selects `Classic/InterfaceOverrides.lua`; line138 tests `IsEngravingEnabled` to register the rune-icon checkbox. All seven members and the complete rune structure are declared in pinned generated `EngravingInfoDocumentation.lua`; declaration lines are linked from the contract.

## Excluded cached files

Mists `Blizzard_CharacterFrame.toc:18–21` selects Cata/PaperDollFrame.lua, not Vanilla/PaperDollFrame.lua. The latter contains equipment queries and the normal `ToggleEngravingFrame` entry. Era/Anniversary select that Vanilla file; both also select the same shared bag/settings consumers. Their generated engraving documentation is byte-identical to Mists. Wrath's complete pinned profile tree has no C_Engraving references.

`Blizzard_EngravingUI` is LoadOnDemand and explicitly loadable on Mists, but has no normal Mists UI entry. Its fourteen additional query/filter/cast members are outside this normal-flow slice. Their declarations do not establish native catalog filtering, cast transitions or default values. [Era/Anniversary availability](era-engraving-availability.md) now has a separate supplied-state slice; the former temporary false policy was not native evidence.

## Supplied input boundary

Absence of input is not supplied false/nil. Tests supply state before loading addons, and unsupplied query inputs must fail explicitly; the native-backed mode flag is a non-nilable boolean. `SimState::seed_default_game_state` seeds the scenario values reached by the bag flow: availability false, mode false and current rune nil. The generated declarations describe a zero-argument bool availability getter (:179–185, no setter), a zero-argument bool mode getter plus setter (:73–79, :273–281), and a nilable `EngravingData` current-rune getter (:64–70). **VERIFIED** native Mists R21 evidence establishes the mode flag starts false and the setter stores its boolean argument, then synchronously fires `ENGRAVING_MODE_CHANGED` on every call, including unchanged values; see [R21 Classic API contracts](mists-r21-classic-apis.md#c_engraving-mode). **INFERRED** native defaults are still availability false and current rune nil. The LoadOnDemand `Blizzard_EngravingUI` also toggles mode in OnShow/OnHide (:86,99), but its seasonal-content TOC note and lack of a normal Mists UI entry do not establish the native default. Supplied availability/current-rune state still overrides, and clearing either seeded field still raises its named missing-input error; the mode getter always returns the stored boolean. Refresh requests have an observable simulator count, not inferred native refresh side effects. Separate supplied tests establish actual overlay and targeting behavior. The original bag lane did not prove settings construction. The later [merged settings proof](#merged-settings-proof-and-stock-effects) closes that bounded constructor path without copied vendor excerpts or adjacent shims.

## Historical bag-lane proof

Producer `f7a9227b8` registers exactly seven members only when `client_profile::ACTIVE` is Mists. Corrected tests are byte-identical in RED `e97b841e7` and GREEN `eb3dca9dc`; their whole-tree difference contains only the producer module and registrar file. Each exact list selects one test, followed by one exact run. Both bag selectors reject the missing namespace at the original `ContainerFrame_OnShow` line315. All three GREEN cases pass: original overlay, original targeting, and direct projection/missing-input/isolation.

The overlay exercises true/false mode and availability, per-slot eligibility, present/nil rune textures and the original CVar-driven refresh request. Targeting exercises nil/present/nil current cast with opposite per-slot results. Direct proof checks six named missing-input errors, all record fields, one nil result versus no refresh results, independent tables, repeated requests and independent environments. Distinct observations reject constant/no-op behavior; no separately compiled constant-mutant binaries are claimed.

The unchanged old storage test now passes: the native mode default removes the previous `C_Engraving.GetEngravingModeEnabled: unsupplied input` boundary failure at original315.

At this earlier bag-lane checkpoint, Mists stock maps were6 keys/12 occurrences, with no added, removed or count-changed raw keys; the [merged settings checkpoint](#merged-settings-proof-and-stock-effects) supersedes these totals. BEFORE `c50e6386a` is base `e98282ab8` plus scaffold types and the default-initialized environment field, with no producer; it is not a separately rebuilt exact master. AFTER `f7a9227b8` adds the producer. Later changes are test/documentation-only. The existing generic `(string):1` pair remains unattributed; no new key needs a masking-predecessor trace.

All seven full-feature all-target checks pass with zero JSON/plain warnings: Mists at `eb3dca9dc`, the other six at documentation-only successor `81d59b62c`. Their Rust/test inputs are identical. This proves compilation, not runtime parity on the other six profiles. Formatting and manual changed-Rust readability pass. Runtime invocations use fresh byte-exact child CVar copies and O_NOATIME source snapshots; originals are not reset/restored.

## Fixture fidelity

Initial fixture failures were not producer RED: missing original `CanOpenPanels`, then missing `MainMenuBarBackpackButton` before the engraving call. Loading the original panel-manager and bag-button closures lets original OnShow reach315. No vendor function is replaced.

Original GenerateFrame assigns reversed button IDs at953–955. Runtime observation gives Item1/Item2 IDs16/15, while inventory slots1/2 are Item16/Item15. Fixtures now locate original controls by GetID instead of equating visual index and bag slot.

The inherited empty-texture assertion expected a path string, contrary to the established `methods_texture.rs` contract. Guarded diagnostics return numeric134400 for the supplied icon and numeric136509 for the original empty-slot path. Main authorized those exact, distinct expectations and a renewed original315 RED. Neither texture semantics nor the engraving producer changed. This is existing simulator-contract evidence, not a new native-client texture probe.

## Sources

- [Feature contract](../../specs/mists-engraving-inputs.md)
- [Original consumer and direct tests](../../../tests/mists_engraving.rs)
- [Merged settings tests](../../../tests/mists_engraving_settings.rs)
- [Shared registrar](../../../src/c_api/c_engraving.rs) and [supplied input](../../../src/c_api/engraving_inputs.rs)
- [Established texture round-trip contract](../../../tests/methods_texture.rs)
- [Selected consumer proof rules](consumer-selection-and-red-proof.md)
- [Original storage-flow predecessor](invented-query-defaults.md#legacy-tutorial-request-boundary)

## See Also

- [[mists-r21-classic-apis]] — native Mists mode default and setter behavior.
- [[era-engraving-availability]] — original XML availability proof and unset-stock attribution.
- [[mists-global-strings]] — preceding display-settings label boundary.
- [[mists-transmog-enabled]] — preceding supplied display branch.
- [Addon loading pipeline](../../addon-loading-pipeline.md)
