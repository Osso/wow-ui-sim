# PTR Edit Mode declaration families

PTR publishes declaration-backed Edit Mode enums through `src/c_api/ptr_edit_mode_enums.rs`. The [investigation](../wiki/investigations/ptr-edit-mode-declarations.md) explains the original consumer boundaries and initialization limits.

## What it must do

- [x] Publish PTR `Enum.BossFrameGrowDirection` as Left=0, Center=1, Right=2, with NumValues=3, MinValue=0, MaxValue=2 metadata.
- [x] Preserve those three distinct values through the original boss-growth dropdown constructor.
- [x] Publish PTR `Enum.EditModePresetLayouts` as Modern=0, Classic=1, Gamepad=2, with NumValues=3, MinValue=0, MaxValue=2 metadata.
- [x] Let the original preset manager index supplied Modern, Classic and Gamepad maps separately, preserve Gamepad system data and return independent anchor copies.

- [x] Publish PTR `Enum.EditModeSwingTimerSetting` as Scale=0, Opacity=1, Visibility=2, Width=3, Height=4, ShowBarTitle=5, ShowTime=6, with NumValues=7, MinValue=0, MaxValue=6 metadata.
- [x] Publish PTR `Enum.EditModeSwingTimerVisibility` as Always=0, InCombat=1, Hidden=2, with NumValues=3, MinValue=0, MaxValue=2 metadata.
- [x] Preserve all seven setting identifiers and three visibility choices through the complete original SwingTimer constructor.

## Complete PTR display-info initialization

- Extend existing enum tables without replacing their earlier members, only when the active profile is PTR.
- Publish the pinned members read by the complete original display-info file: system MainActionBarEndCap=26, GroupFinder=27, LossOfControl=28, SwingTimer=29 and GamepadMainActionBar=30; Gamepad settings Compact=0, ShowEmptyBars=1, ShowButtonPrompts=2, ShowHighlight=3 and ShouldExpand=4; GroupFinder Size=0; EndCap Hidden=0; PersonalResourceDisplay AllowMouseoverThrough=15; UnitFrame DebuffIconSize=19, UseHorizontalLayout=23 and GrowDirection=24.
- The unchanged whole `EditModeSettingDisplayInfo.lua` must complete and publish `GetSystemSettingDisplayInfoMap`, with the original per-system keys and setting values.
- Do not publish unread account settings or metadata for this boundary. Stop at a non-declaration failure rather than replacing Blizzard functions or strings.

## Complete PTR Mainline preset index declarations

- Publish only `Enum.EditModeMainActionBarEndCapSystemIndices` as EndCapLeft=1 and EndCapRight=2, with `Enum.EditModeMainActionBarEndCapSystemIndicesMeta` NumValues=2, MinValue=1, MaxValue=2. The pinned PTR 12.1.7.70493 `Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:560–568` declares this exact name, metadata and values. Do not publish a shortened alias.
- Keep publication in the existing C API module and runtime `client_profile::ACTIVE` PTR gate. Preserve other profiles and unrelated members.
- The unchanged selected Mainline preset endcap constructor at `EditModePresetLayouts.lua:630–655` must retain distinct records at indices1/2, each with Hidden=0, and its original anchors: RIGHT/MainActionBar/LEFT/(30,5) and LEFT/BagsBar/RIGHT/(-30,5).
- Publish `Enum.EditModeSwingTimerSystemIndices` as MainHand=1, OffHand=2, Ranged=3, with `Enum.EditModeSwingTimerSystemIndicesMeta` NumValues=3, MinValue=1, MaxValue=3. The same pinned PTR document at719–728 declares the exact name, metadata and values.
- The whole unchanged selected Mainline `EditModePresetLayouts.lua` must complete. All three preset maps must preserve the original two EndCap records and three distinct SwingTimer records, settings and exact per-index anchors, including Gamepad's original Width/Height overrides. Producer-subtracted RED must stop at the first EndCap read (physical631); byte-identical GREEN must reach the end of the file.
- This is complete preset construction, not a native default-layout policy or full Edit Mode lifecycle claim. No other declaration family is included.

## PTR library baseline reconciliation

The 70493 library baseline additionally requires `EditModeSystemMeta` to span
0–30 (31 values), `EditModeAccountSetting.ShowGroupFinder=35`,
`ShowLossOfControl=36`, `ShowSwingTimer=37`, and account metadata spanning
0–37 (38 values). These declarations are in the pinned
`Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:215–260,745–782`.
The original account-setting consumer reads ShowLossOfControl in
`Blizzard_EditMode/Shared/EditModeManager.lua:971,2954`.
This extends the earlier display-info-only boundary above; Retail keeps its
existing identifiers and metadata. The focused library test asserts both profiles'
respective loss-of-control identifiers and metadata.

UnitFrame `BuffIconSize` must be 22, with metadata spanning 0–24 (25 values),
as declared in the same pinned document at797–828. Publish it before the PTR
compatibility bootstrap rather than appending it after GrowDirection as 25.
The original display-info constructor at
`Blizzard_EditMode/Shared/EditModeSettingDisplayInfo.lua:427–438` must retain
its buff slider at numeric key22 (range50–200, step10), with no entry at25.
The original consumer regression rejects the former appended-key producer.
Retail retains its 0–22 metadata and unchanged buff identifier.

## How it works

- [Declaration and consumer evidence](../wiki/investigations/ptr-edit-mode-declarations.md).

## Implementation inventory

- `src/c_api/ptr_edit_mode_enums.rs`: declaration-backed tables and metadata.
- `src/c_api/mod.rs`: unconditional module declaration, compiled on every profile.
- `src/lua_api/env_init/enums.rs`: unconditional hook after generated and compatibility enum initialization; the module returns without changes unless `ACTIVE == ClientProfile::Ptr`.

## Tests asserting this spec

- `tests/ptr_edit_mode_enums.rs`: original selected dropdowns, complete SwingTimer settings constructor, independently reached original preset manager, unchanged whole-file display-info initialization and original LossOfControl XML OnLoad completion, and complete unchanged Mainline preset construction with distinct EndCap and SwingTimer records across all three maps.

## Known gaps (current cycle)

- [x] Complete original display-info initialization and the original LossOfControl OnLoad both pass byte-identical producer-only RED/GREEN selectors.
- [ ] Native preset defaults and other Edit Mode lifecycle behavior remain outside this contract. Full Edit Mode lifecycle and native default-layout behavior are not covered by complete preset construction. Its complete constructor has the separate two-family contract above; display-info does not read either index family.

## Out of scope

Other declaration families, GraphicsOverrides and its preset, GameRules, replacement Edit Mode Lua, frame-method patches, and native preset defaults. Other profiles retain their existing registration paths.
