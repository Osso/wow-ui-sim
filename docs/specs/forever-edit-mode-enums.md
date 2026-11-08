# Forever Edit Mode enums

Forever publishes finite additions from its `EditModeManagerConstantsDocumentation.lua` through `src/c_api/forever_edit_mode_enums.rs`. Existing 1.60.1.69913 proof is historical; the bounded 1.60.1.70374 gamepad proof is tracked below. See [client profiles](../wiki/systems/client-profiles.md#forever-gamepad-edit-mode--2026-10-15).

## What it must do

- [x] Publish the six missing GroupFinder, MainActionBarEndCap and SwingTimer setting/index/visibility namespaces and their source metadata.
- [x] Publish documented LossOfControl and RaidWarning settings, Gamepad preset 2, systems 24–29, account settings 33–37, minimap IconScale, micro-menu DeprecatedEyeSize and unit-frame DebuffIconSize/BuffIconSize with updated metadata.
- [x] Reuse the existing RaidDispelOverlayType values and metadata.
- [x] Execute the historical 69913 Forever preset and setting-display consumers without enum lookup failures; this does not establish 70374 compatibility.
- [x] Under `client-wowforever`, publish 70374 `EditModeSystem.GamepadMainActionBar=30` and system metadata `MinValue=0`, `MaxValue=30`, `NumValues=31`.
- [x] Publish `EditModeGamepadMainActionBarSetting`: `Compact=0`, `ShowEmptyBars=1`, `ShowButtonPrompts=2`, `ShowHighlight=3`, `ShouldExpand=4`; metadata `MinValue=0`, `MaxValue=4`, `NumValues=5`.
- [x] Execute actual 70374 Camelot preset constants, Mainline presets and shared setting-display consumers: `EDIT_MODE_GAMEPAD_SYSTEM_MAP[30].settings` has exactly five entries, values `0/1/0/1/1` at keys `0..4`, BOTTOM/UIParent anchor with offsetY 110, and five checkbox descriptors mapping settings `0..4`.
- [x] Keep Forever GamepadMainActionBar, its settings/metadata, Gamepad preset and RecentAlliesInteractionCategoryFilter/metadata absent on Retail, Wrath, Mists, Era and Anniversary; preserve base ActionBar=0. PTR 12.1.7.70493 declares them natively (`Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:782`, `:498–509`, `:648`; `RecentAlliesConstantsDocumentation.lua:6–18`), so PTR asserts that declaration instead. Bounded runtime proof below, not exhaustive other-profile enum preservation.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)
- [Running patch report](../wowforever-1.60.1.md)

## Implementation inventory

- `src/c_api/forever_edit_mode_enums.rs` — finite documented additions and metadata.
- `src/c_api/mod.rs` — profile-scoped compilation.
- `src/lua_api/env_init/enums.rs` — publication after shared enum initialization.
- `src/lua_api/globals/enum_data/combat_system.rs` — reused raid dispel definitions.

## Tests asserting this spec

- `tests/wowforever_edit_mode_enums.rs` — existing exact source values, metadata, actual preset and display consumers. Includes executed `forever_edit_mode_gamepad_consumers`; bounded PASS on runtime source136, not the new import revision.
- `tests/enum_isolation.rs` — earlier-profile absence cases, the PTR native gamepad/preset/RecentAllies declaration case, and original-consumer preservation cases; receipts below.

### Earlier-profile isolation receipt — 2026-10-18

Executed `cargo test --locked -j1 --no-default-features --features sound,gui,casc,client-<profile> --test integration enum_isolation:: -- --nocapture --test-threads=1`, sequentially in reusable slot-0, systemd user units with MemoryMax=9G and ADD admission.

| Profile | Result | Passed / failed | Source revision |
|---|---|---|---|
| Retail | GREEN | 5 / 0 | `31f9eee9c` |
| PTR | GREEN | 5 / 0 | `31f9eee9c` |
| Wrath | GREEN | 3 / 0 | `31f9eee9c` |
| Mists | GREEN | 3 / 0 | `31f9eee9c` |
| Era | GREEN | 3 / 0 | `31f9eee9c` |
| Anniversary | GREEN | 3 / 0 | `d28922a21` |

All six executed `forever_gamepad_publication_is_absent_from_earlier_profiles` at the time; the PTR row is historical. The PTR Edit Mode declaration lane `8a23c562e` now publishes `EditModeSystem.GamepadMainActionBar=30` from the pinned PTR document, so current PTR runs `ptr_edit_mode_gamepad_publication_matches_pinned_declarations` instead. Retail/PTR also execute `original_communities_sort_preserves_profile_stream_ids`, `original_account_consumer_keeps_friend_levels_and_ranks`, `original_sound_data_keeps_numeric_sound_ids_and_kits` and `original_visual_alert_lookup_keeps_numeric_style_ids`. Classic profiles execute `classic_profiles_keep_shared_modern_enum_group_absent`; Wrath adds `wrath_keeps_original_stream_numbers_without_discord`, while Mists/Era/Anniversary add the original Communities sort case. These are 22 executed cases, not empty cfg-filtered passes. See [shared enum contract](forever-shared-enums.md).

First five rows precede the requested rebase onto `032a3e7d8`; Anniversary follows it. Intervening changes concern timed-signal-map behavior and unrelated tests/docs, not enum publication or these fixtures. No earlier row is represented as rerun on the later revision. Logs and exact commands: `/home/osso/.local/state/wow-ui-sim/handoff/enum-isolation-run/{report.md,proof-ledger.md,<profile>.log}`. Original consumer files came from the active profile cache without edits; cache bytes were not independently resealed. No production changes, full-startup runs, native-client parity or full enum-surface proof.

## Known gaps (current cycle)

### Verified 70374 gamepad scope — 2026-10-15

[Central receipts](../baselines/README.md#verified-scoped-receipts--2026-10-15) establish actual compiled enum, preset, default-layout, display and gamepad-consumer PASS at runtime source136. The gamepad regression loads real cached TableUtil/preset/display Lua and asserts the concrete values above; external vendor cache inputs were not independently resealed.

Historical source9a sealed-binary probe `FOREVER_GAMEPAD_RED system=nil settings=nil` and preset line1272 failure remain RED history, not current results. Candidate applicability/source review alone was not runtime proof; the saved executed tests now close that bounded model gap.

- [x] Owning enum/preset/default-layout/display/gamepad model gate executed.
- [x] Actual post-enum startup captured; [baseline owner](../baselines/README.md#forever-lua-errors) records removed preset/display cascade and remaining FAIL, not clean startup.
- [ ] Full Edit Mode UI interactions, native parity, isolation beyond the named cases and parent acceptance remain open.

Locale acquisition remains separate; see [locale owner](casc-loading.md#locale-checkpoint--october-15-2026).

## Out of scope

Native-game behavior, arbitrary enum generation, retail epoch activation, and vendor changes.
