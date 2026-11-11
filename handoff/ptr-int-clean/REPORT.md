# PTR integration cleanup

## Active goal

Reduce the 69 PTR integration failures at `aa33b409c` with no new failure names. Scope: source-backed PTR fixtures and simulator root causes exposed by these failures. Preserve Retail and all other profile behavior. Exclude vendor edits, placeholder APIs, Cargo manifest/lock changes, push and rebase. Required verification: user-requested per-profile all-target builds and immediate tests, PTR full integration/lib, touched-file filters on Retail/Forever/Mists, Retail lib, Era/Anniversary builds, Wrath compile and manifest bin test; zero warnings. Test runs stay below 9 GiB.

## Baseline and method

VERIFIED: `/var/tmp/osso/prship/m29c-p-int-all.log` ends with 10,573 passed, 69 failed and 17 ignored. Names below use regex `^    ([A-Za-z0-9_:]+)$` after its last `\nfailures:\n`. Failure count is not root-cause count. Original per-test panic output is retained outside git in `~/.local/state/wow-ui-sim/handoff/ptr-int-clean/errors.json`. Classifications identify the next evidence/action, not claims that all causes are already proven.

## Cluster table (initial triage, refined after pinned-source inspection)

| Cluster | Names | Decision | Evidence and next action |
|---|---:|---|---|
| Pinned source expectations | 35 | (b) | PTR 12.1.7 TOCs, selected variants, declarations and Lua wrappers contradict old expectations; corrections retain other profile expectations. |
| StaticPopup virtual-template materialization | 1 | (a; R24, resolved) | VERIFIED: PTR `GameDialogBaseMixin:OnLoad` fails on `self.GamepadSequence` after earlier mutations; R24 verifies protected dispatch, installed-handler routing, parentKey timing and post-OnLoad visibility/OnShow. INFERRED: native `CreateFrame` returns the partial frame without propagating the error. Simulator commit `7b4dc8b0a` reports the error and continues finalization; the exact PTR regression passes with partial state, parentKeys, handler capture and visible `OnShow`. |
| Action-bar startup and presentation | 7 | (b; 1 master-matching failure) | VERIFIED: current PTR fixtures include `Blizzard_RaidWarning`, `Blizzard_StatusTrackingBar`, `Blizzard_GamepadSharedUtility` and its TOC dependencies; all seven exact selectors were rerun at `45b5dc284`, and six pass. `action_bar::test_bag_bar_layout_locked` alone fails with `toggle_missing_rect`; the exact same name failed on master. VERIFIED (simulator source): the bag toggle's parent is the unanchored `BagSlotCluster`, and `GetRect` rejects frames with any ancestor lacking a render-layout anchor. UNKNOWN: native `GetRect` behavior for this grouping frame. This does not add a PTR-only failure name. |
| Bag-frame array and template source expectations | 3 | (b; expectation corrections) | VERIFIED: selected Mainline XML yields array lengths 6 on Retail and 7 on PTR/Forever; the two array tests pass 2/2 on all three. Mists has no Mainline `ContainerFrameContainer` contract and these tests select 0. The path fixture now searches XML files from the selected TOC and passes on PTR/Retail. No runtime/vendor behavior changed. |
| PTR bag-frame rectangle query | 1 | (a; blocked on R48) | VERIFIED: commit `47ec5a397` fails on `CharacterBag0Slot` with `expected numeric result, got nil`. VERIFIED source: the frame is under unanchored `BagSlotCluster`; simulator rect queries reject a frame with any unanchored ancestor. INFERRED: this ancestor causes the empty result. Native rect validity in this hierarchy is UNKNOWN; do not change `frame_has_render_layout` before R48. |
| Auction-house token namespace | 4 | (a; R22) | VERIFIED: producer commit `ba7df3919` registers the PTR-only `C_WowTokenUI.ShouldDisplayInAuctionHouse()` query; all four original auction-house consumer tests pass. INFERRED from R22: its result is true before and after server status updates. |
| Profile publication expectations | 1 | (a; R23) | VERIFIED: both PTR tests for `GetLootMethodStyle()` pass, including live rule-157 rereads after `GAME_RULES_CHANGED`; the getter is independent of group and loot-method state. |
| Profession startup and geometry | 3 | (a) | Blacksmithing book button absent; ProfessionsFrame width 1; crafting ValidateControls transaction nil. Trace selected profession model/input before geometry changes. |
| Game-menu Store API and source gate | 1 | (a; R38, resolved) | VERIFIED: PTR `GameMenuFrame.lua:198-200` uses AND; Retail `:148-154` uses OR. R38 backs the separate queries and the profile suites pass 13/13 each. INFERRED: the defaults hide PTR's button and show Retail's. |
| Game-menu load smoke | 1 | (b; not reproduced) | VERIFIED: `ace3::test_load_game_menu` passes 1/1 on PTR at `93dc97c1f`; its original full-suite failure is not reproduced. Cause UNKNOWN until the full-suite comparison. |
| LOD shard 10 errors | 2 | (a; R25) | VERIFIED: both failures came from one `Blizzard_Commentator` error, `C_Commentator.SetMouseDisabled` nil. PTR native registration remains INFERRED from its implementation, docs and consumer; simulator commit `49f01d132` now registers the existing behavior on PTR. The original consumer test passes on PTR and Retail; LOD shards 10 and 14 pass on PTR, with `Blizzard_Commentator` runtime errors `{}`. Keep the baseline strict. |
| Achievement comparison fixture | 2 | (b; fixture corrected) | VERIFIED: original Mainline `Blizzard_AchievementUI.lua:418` calls `DateCompleted:SetShown`; the fixture now supplies that collaborator and preserves completed/incomplete assertions. Both tests pass 2/2 on PTR and Retail. |
| Modified-click mutation | 2 | (a) | R22 proves immediate read-after-write for accepted mutations and says not to model the undocumented PTR context checks; trace simulator state and preserve Retail behavior. |
| Combo-frame consumer selection | 1 | (b; source scope corrected) | VERIFIED: PTR's selected `Blizzard_UnitFrame.toc` does not load the Mainline ComboFrame consumer; the test is excluded on PTR. Retail retains the consumer and its exact test passes 1/1. |
| Missing cooldown duration policy | 1 | (a) | R17 and R23 distinguish an unknown spell (zero Lua results) from a known spell without active cooldown (zero `LuaDurationObject`); verify the original failure at this exact consumer and preserve profile-specific behavior. |
| PTR eager load-order snapshot | 1 | (b) | PTR 12.1.7 selected-addon order differs from the 12.1.5 fixture; retain a separate source-bound PTR snapshot. |
| Quest tracker block count | 1 | (a) | Expected at least three visible blocks; observed two. Trace seeded quest and original tracker filtering before changing count. |
| Spellbook geometry | 1 | (a) | Observed book_bg_gap=1612. Trace named background, parent and selected template before changing lock. |
| Store free upgrade API | 1 | (a; R22, resolved) | VERIFIED: the function is absent from `C_StoreSecure` and exists under `C_SharedCharacterServices`; the corrected exact test passes 1/1 on PTR and Retail, without changing `C_StoreSecure`. |

## Targeted integration proof — 2026-11-10

- VERIFIED: commit `ba7df3919c656789e8abf6281aacbb4770d33b9e` built the PTR `integration` target in slot 1; the exact artifact is `integration-4927c2ef62e6c1de`, and each of 13 selectors listed exactly once.
- VERIFIED: the four auction-house tests and two loot-style tests passed; all seven action-bar tests remained failing. Per-test receipts and logs are in `/var/tmp/osso/prship/ptr-int-clean-targeted-ba7df3919/summary.json` and its sibling directories.
- VERIFIED: all 26 fresh CVar before/after receipts preserve the original file's stat fields, symlink resolution and SHA-256 under O_NOATIME. This is targeted evidence, not full-integration or final-profile-gate evidence.
- VERIFIED: R25 commit `49f01d132b77395686b9e5ec98f182c9571dcbe9` passes `commentator_consumer_updates_shared_mouse_availability` on PTR and Retail plus PTR LOD shards 10 and 14. Build receipts are in slot 1; test receipts are in `/var/tmp/osso/prship/ptr-int-clean-commentator-green-49f01d132/` and `/var/tmp/osso/prship/ptr-int-clean-commentator-retail-green-49f01d132/`. All eight CVar receipts preserve the original.
- VERIFIED: StaticPopup acceptance test commit `d6cf3b78e` failed before the fix with `CreateFrame propagated the template OnLoad error`. Producer commit `7b4dc8b0a` built the PTR integration target successfully with no compiler warnings; the exact selector passed 1/1 using `integration-4927c2ef62e6c1de`. Build receipt: `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-staticpopup-green-7b4dc8b0a.json`; test log: `/var/tmp/osso/prship/ptr-int-clean-staticpopup-greenrun-7b4dc8b0a.log`.
- VERIFIED: The StaticPopup run used a fresh CVar copy under `/var/tmp/osso/ptr-int-clean-staticpopup-green-4a3pm6lp`; the original CVar SHA-256 and all recorded stat fields were identical before and after O_NOATIME reads. This is targeted evidence only; full PTR failure-name comparison remains open.
- DEFERRED: R24 verifies native same-slot script rules (no `inherit` replaces; `prepend` runs inherited then new; `append` runs new then inherited), a first-handler error skips the second handler but not independent slots, mixins do not add automatic `OnLoad`, and duplicate chain functions are not removed. The simulator XML chain wrapper was not changed or tested here. Base `OnLoad` running once is INFERRED only for this StaticPopup case from shipped Lua; no general guarantee is claimed.

## Pinned-source citations

All paths below are relative to `~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns/`; no cache file is modified.

- VERIFIED: `Blizzard_MacroUI/Blizzard_MacroUI.toc:4-11` lists eight selected files, including explicit Lua before XML. `Blizzard_PTRFeedbackGlue/Blizzard_PTRFeedbackGlue.toc:8-14` adds `Blizzard_PTRFeedback_Gamepad.lua`. `Blizzard_PTRFeedback/Blizzard_PTRFeedback.toc:2,8-14` is PTR-only and has seven files.
- VERIFIED: `Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc:4-45` lists 42 entries. `Blizzard_StableUI/Blizzard_StableUI.toc:4-6` lists shared Lua, game Lua and game XML. `Blizzard_StaticPopup/Blizzard_StaticPopup.toc:4-7` includes `StaticPopupGamepad.lua`.
- VERIFIED: `Blizzard_StaticPopup_Game/Blizzard_StaticPopup_Game.toc:2` adds `Blizzard_GamepadSharedUtility`; `:9-13` selects `GameDialog.xml` and Mainline `StaticPopupSpecial.xml`. `Blizzard_FrameXML/Blizzard_FrameXML.toc:19` adds `Blizzard_GamepadSoftCursor`. `Blizzard_SharedTalentUI/Blizzard_SharedTalentUI.toc:3` adds `Blizzard_SpellTypeIconOverlay`.
- VERIFIED: `Blizzard_SubscriptionInterstitialUI/Blizzard_SubscriptionInterstitialUI.toc:3-6` explicitly lists bootstrap/Lua/XML. `Blizzard_TrainerUI/Blizzard_TrainerUI.toc:5-9` explicitly lists bootstrap/Mainline Lua/Mainline XML/localization. `Blizzard_TokenUI/Blizzard_TokenUI.toc:4-11` uses Mainline TokenUI paths for PTR.
- VERIFIED: `Blizzard_UIParent/Blizzard_UIParent.toc:1-3` is a unified TOC and selects only UIParent.lua. `Blizzard_UnitPopup/Blizzard_UnitPopup.toc:11,14` selects Mainline buttons/menus through `[Family]`; `:12,15` filter other `[Game]` flavors. `Blizzard_Settings_Shared/Blizzard_Settings_Shared.toc:38` adds Mainline LanguagesOverrides.lua.
- VERIFIED: PTR `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua:276-281` declares `C_GameRules.IsHardcoreActive` as a non-nil boolean function; the test's PTR absence assertion is stale. PTR `SystemDocumentation.lua:85-90`, `PvpInfoDocumentation.lua:1385-1390`, `SocialRestrictionsDocumentation.lua:120-125`, and `VoiceChatDocumentation.lua:918-923,1397-1402` declare the tested Forever-known events.
- VERIFIED: Lua defines the wrappers previously asserted absent: `Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUI_Bootstrap.lua:13`, `Blizzard_GuildBankUI/Blizzard_GuildBankUI_Bootstrap.lua:16`, `Blizzard_ProfessionsCustomerOrders/Blizzard_ProfessionsCustomerOrders_Bootstrap.lua:13`, `Blizzard_GarrisonUI/Mainline/Blizzard_GarrisonUI_Bootstrap.lua:44`, `Blizzard_BlackMarketUI/Blizzard_BlackMarketUI_Bootstrap.lua:13`.
- VERIFIED: `Blizzard_StaticPopup_Game/GameDialog.xml:3,51,78` declares `StaticPopupButtonTemplate`, `StaticPopupBaseTemplate`, and `StaticPopupTemplate` as virtual templates. The PTR TOC selects `GameDialog.xml` at `Blizzard_StaticPopup_Game.toc:11`; the remaining failure is simulator-side registration, not a stale expectation.
- VERIFIED: `Blizzard_RaidUI.toc:5-6` lists Mainline Lua before XML on PTR; the Mists TOC selects Mists XML at `Blizzard_RaidUI.toc:8-9`.
- VERIFIED: PTR `Blizzard_GameMenu/Shared/GameMenuFrame.lua:198-200` uses an AND gate; Retail `:148-154` uses an OR gate. R38 backs both queries and records their defaults and update behavior in the section below.
- VERIFIED: R22 native evidence is in `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md#r22`: PTR publishes `C_WowTokenUI.ShouldDisplayInAuctionHouse`, the `C_StoreSecure` member is absent, and accepted modified-click changes are immediately readable.

Further fixture changes require citations at the point of change; the initial table alone does not authorize weaker checks.

## Store API behavior — R38 evidence and implementation verification

- VERIFIED (R38, PTR 12.1.7.70668): `C_StorePublic.IsEnabled()` returns one boolean from hidden server mirror `bpayStoreEnable`, default `false`; `C_CatalogShop.IsShop2Enabled()` returns one boolean from independent hidden mirror `shop2Enabled`, default `true`. Retail 12.1.5.70688 has the same results in a limited comparison only.
- VERIFIED (R38): `C_CVar.GetCVar` refuses both hidden values. INFERRED: the Lua result is one `nil`; exact result shape is not asserted. UNKNOWN: legacy global `GetCVar` parity and real-account values.
- VERIFIED (R38): a changed `bpayStoreEnable` posts one payload-free `STORE_STATUS_CHANGED` per server update and no `CVAR_UPDATE`. On initialized paths, a `shop2Enabled` change posts no Lua event; first-use behavior is UNKNOWN. The two queries do not read each other's value.
- VERIFIED (shipped source): PTR `GameMenuFrame.lua:198-200` gates the Store button with AND; Retail `:148-154` uses OR. INFERRED: defaults `(false, true)` hide the PTR button and show the Retail button.
- VERIFIED (simulator source before the change): temporary fallbacks returned `true` for `C_StorePublic.IsEnabled` and `false` for `C_CatalogShop.IsShop2Enabled`. VERIFIED (commit `08169d88e`): Retail/PTR now register state-backed queries and keep hidden server mirrors separate from Lua-visible CVar state; compatibility defaults remain for uncovered profiles.
- VERIFIED: the duplicate GameMenu shop-button block was removed; one state-backed version remains.
- UNKNOWN: 70493 behavior, exact `C_CVar.GetCVar` result shape, and first-use `shop2Enabled` event behavior remain unasserted.

## Current targeted proof — 2026-11-11

- VERIFIED: PTR integration target at `45b5dc284da969b3d142c9dbec92f02cec39ef15` built with zero warnings. Receipt: `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-game-menu-nostore-preflight-45b5dc284.json`.
- VERIFIED: GameMenu `test_game_menu_buttons_created_on_show` and `test_game_menu_hide` each passed 1/1 after removal of the store-path assertions.
- VERIFIED: six of seven ActionBar selectors passed: `action_bar::test_main_action_bar_end_caps_visible_after_startup`, `action_bar::test_status_tracking_xp_and_reputation_bars_layout_locked`, `action_bar_drag::action_button_drag_round_trip_keeps_spell_visible`, `action_bar_tree::main_action_bar_layout_is_locked`, `blizzard_ui_blizzard_actionbar::behavior_main_bar_endcaps::main_bar_endcaps_follow_hide_bar_art_setting`, and `blizzard_ui_blizzard_actionbar::behavior_spell_flyout::spell_flyout_toggle_shows_populates_and_hides_from_same_button`.
- VERIFIED: the remaining selector, `action_bar::test_bag_bar_layout_locked`, failed with `toggle_missing_rect:barLayout=function:size=10x16:initial=10x16:parent=BagSlotCluster:points=RIGHT->MainMenuBarBackpackButton:LEFT(0.00,0.00)`; master exact-run evidence also records this selector as failed. The native `GetRect` result for this unanchored grouping frame is UNKNOWN, so no simulator or vendor behavior was changed.
- VERIFIED: all nine earlier targeted runs preserved the original CVar SHA-256 `8d1709049bc9acdbed0465889701a0e370667ef18cf06b71db479b3972aa350c` and recorded stat fields. Receipts and logs: `/home/osso/.local/state/wow-ui-sim/verification/ptr-int-clean-game-menu-nostore-45b5dc284/` and `/home/osso/.local/state/wow-ui-sim/verification/ptr-int-clean-actionbar-current-45b5dc284/`.
- VERIFIED: the R38 targeted tests pass. PTR: GameMenu 13/13, Store API 1/1, StorePublic 2/2, CatalogShop product structures 19/19, RewardsShop 2/2, StoreFree 1/1, and `ace3::test_load_game_menu` 1/1. Retail: GameMenu 13/13, Store API 1/1, StorePublic 2/2, CatalogShop product structures 19/19, full-UI CatalogShop 4/4, StoreFree 1/1, Achievement comparison 2/2, and ComboFrame consumer 1/1. All these targeted build receipts report zero warnings; receipt files are under `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/` with prefixes `ptr-int-clean-r38-ptr-` and `ptr-int-clean-93dc97c1-`.
- VERIFIED: the earlier broad PTR `store` filter's two in-scope expectation failures pass after `bde1f76b6`; the full PTR failure-name comparison remains pending.
- VERIFIED: selected `Mainline/ContainerFrame.xml` declares `ContainerFrame1`-`ContainerFrame6` on Retail and adds `ContainerFrame7` on PTR and Forever under a `parentArray="ContainerFrames"` template. The post-`b7c4794b1` array tests pass 2/2 on PTR, Retail and Forever; observed lengths are 7, 6 and 7. Receipts: `ptr-int-clean-b7c4794b1-bag-frames-{ptr,retail,forever}.json`.
- VERIFIED (Mists): selected Classic `ContainerFrame.xml` declares `ContainerFrame1`-`ContainerFrame7` as standalone frames but has no `ContainerFrameContainer` or `parentArray="ContainerFrames"`; the earlier runtime array was empty. After the Mainline-only scope correction, the two array tests select 0. The Mainline path fixture also selects 0 on Mists. Receipts: `ptr-int-clean-b7c4794b1-bag-frames-mists.json` and `ptr-int-clean-b7c4794b1-path-control-mists.json`.
- VERIFIED: the PTR selected bag-button TOC loads `Mainline/MainMenuBarBagButtonTemplates.xml`, which declares `BaseBagSlotButtonTemplate`; Retail's selected TOC has the declaration in `Mainline/MainMenuBarBagButtons.xml`. The TOC-driven path fixture passes 1/1 on PTR and Retail. The same filter selects 0 on Forever because `path_control_point.rs` is gated by `retail-12-0-0`; WowForever's selected TOC still lists the separate template file. Receipts: `ptr-int-clean-b7c4794b1-path-control-{ptr,retail,forever}.json`.
- VERIFIED: the PTR bag-rect test fails at `CharacterBag0Slot` on commit `47ec5a397`; Retail passed 1/1 on `b7c4794b1`, and Mists passed 1/1 on `93dc97c1f` before this diagnostic-only edit. Retail, Forever and Mists reruns after `47ec5a397` remain pending.
- VERIFIED (PTR source): `Mainline/MainMenuBarBagButtons.xml:12,28` parents `BagSlotCluster` under `BagsBar`; `Mainline/MainMenuBarBagButtonTemplates.xml:61,90-97` declares the grouping template without anchors and anchors `CharacterBag0Slot` directly to sibling `MainMenuBarBackpackButton`.
- VERIFIED (simulator source): `layout_render_eligibility.rs:15-27` requires each ancestor to have a layout anchor; `rect_geometry.rs:44-82,88-100,106-117` returns zero values when that check rejects a frame. INFERRED: the unanchored `BagSlotCluster` causes the empty `CharacterBag0Slot` result.
- UNKNOWN (native PTR): whether `GetRect`, `GetScaledRect` or `GetCenter` returns values in this hierarchy. Parent requested R48 native evidence; do not change `frame_has_render_layout` before the answer.

## Exact name coverage

### Pinned source expectations (35 stale assertions)

- `blizzard_auction_house_show_hide::ptr_auction_house_does_not_publish_snapshot_only_hide_wrapper`
- `blizzard_black_market_ui_loads::ptr_black_market_does_not_publish_reversed_hide_wrapper`
- `blizzard_core_frame_lane::lane_dep_edges_pin_canonical_chain`
- `blizzard_garrison_ui_loads::ptr_garrison_ui_does_not_publish_snapshot_only_hide_wrappers`
- `blizzard_guild_bank_ui_loads::ptr_guild_bank_does_not_publish_snapshot_only_hide_wrapper`
- `blizzard_interaction_lane::popup_cluster_dep_edges_pin_load_floor_for_popup_flow`
- `blizzard_macro_ui_loads::blizzard_macro_ui_directory_holds_nine_entries_with_bootstrap`
- `blizzard_macro_ui_loads::ptr_macro_save_placeholder_is_replaced_by_lod_delegate`
- `blizzard_professions_customer_orders_loads::ptr_customer_orders_does_not_publish_snapshot_only_hide_wrapper`
- `blizzard_progression_lane::each_lane_addon_directory_exists_on_disk`
- `blizzard_progression_lane::lane_dep_edges_classify_each_addons_load_floor`
- `blizzard_ptr_feedback_glue_loads::blizzard_ptr_feedback_glue_does_not_appear_in_eager_discovery`
- `blizzard_ptr_feedback_glue_loads::blizzard_ptr_feedback_glue_lists_five_files_with_cross_addon_parent_paths`
- `blizzard_ptr_feedback_loads::blizzard_ptr_feedback_does_not_appear_in_eager_discovery`
- `blizzard_raid_ui_loads::blizzard_raid_ui_xml_pulls_lua_via_script_directive`
- `blizzard_reforging_ui_loads::classic_subdirectory_holds_bootstrap_lua_xml_and_localization`
- `blizzard_reforging_ui_loads::is_game_type_restricted_returns_true_for_classic_only_addon`
- `blizzard_settings_shared_loads::family_placeholder_resolves_to_mainline_subdir_in_three_entries`
- `blizzard_shared_xml_base_loads::body_count_matches_filesystem_layout`
- `blizzard_shared_xml_loads::ptr_smooth_progress_helper_is_global_not_interpolator_method`
- `blizzard_stable_ui_loads::body_lists_lua_before_xml_for_mixin_resolution`
- `blizzard_static_popup_game_loads::dependencies_chain_pulls_six_blizzard_addons_via_plural_dependencies_key`
- `blizzard_static_popup_loads::body_lists_three_lua_files_in_dependency_order`
- `blizzard_subscription_interstitial_ui_loads::body_resolves_bootstrap_then_xml_entry`
- `blizzard_subscription_interstitial_ui_loads::xml_chains_lua_via_script_file_directive`
- `blizzard_talent_ui_loads::allow_load_absent_restricts_to_game_screen_only`
- `blizzard_talent_ui_loads::body_resolves_two_bootstraps_and_five_entries_with_normalized_slashes`
- `blizzard_talent_ui_loads::flavor_subdirs_present_classic_subdir_absent`
- `blizzard_token_ui_loads::body_resolves_to_five_entries_in_declared_order`
- `blizzard_trainer_ui_loads::body_resolves_to_bootstrap_mainline_xml_and_localization_lua`
- `blizzard_ui_parent_loads::ptr_mainline_excludes_mists_time_string_helper`
- `blizzard_unit_frame_loads::classic_companion_is_restricted_on_mainline_build`
- `game_rules::game_rules_hardcore_flag_defaults_to_false`
- `startup_api_events::forever_known_startup_names_preserve_other_profile_registration_policy`
- `blizzard_unit_popup_loads::game_placeholder_resolves_to_standard_directory`

### StaticPopup virtual-template materialization

- `blizzard_static_popup_game_loads::xml_registers_three_virtual_templates`

### Action-bar startup and presentation

- `action_bar::test_bag_bar_layout_locked`
- `action_bar::test_main_action_bar_end_caps_visible_after_startup`
- `action_bar::test_status_tracking_xp_and_reputation_bars_layout_locked`
- `action_bar_drag::action_button_drag_round_trip_keeps_spell_visible`
- `action_bar_tree::main_action_bar_layout_is_locked`
- `blizzard_ui_blizzard_actionbar::behavior_main_bar_endcaps::main_bar_endcaps_follow_hide_bar_art_setting`
- `blizzard_ui_blizzard_actionbar::behavior_spell_flyout::spell_flyout_toggle_shows_populates_and_hides_from_same_button`

### Bag-frame array and template source expectations

- `bags::test_container_frames_array_contains_only_real_container_frames`
- `bags::test_container_frames_registered`
- `path_control_point::path_actual_cached_smooth_xml_configures_points_and_restores_playback`

### PTR bag-frame rectangle query

- `workarounds_bags::bag_frame_rect_queries_match_render_layout_rects`

### Auction-house token namespace

- `blizzard_auction_house_categories_list::auction_house_categories_list_refresh_count_matches_categories`
- `blizzard_auction_house_event_registrations::auction_house_frame_registers_runtime_events_after_show`
- `blizzard_auction_house_show_hide::auction_house_show_hide_round_trip_has_no_lua_errors`
- `blizzard_auction_house_tab_switch::auction_house_tabs_and_token_modes_show_matching_subframes`

### Profile publication expectations

- `party_loot_method::loot_method_style_reads_rule_157_and_ignores_loot_method`
- `party_loot_method::ptr_loot_method_style_rereads_rule_157_on_game_rules_changed`

### Profession startup and geometry

- `micro_menu::micro_menu_professions_button_loads_and_opens_panel`
- `workarounds_professions::casting_blacksmithing_opens_professions_frame`
- `workarounds_professions::clicking_blacksmithing_button_in_professions_book_opens_panel`

### Game-menu source/API revision

- `ace3::test_load_game_menu`
- `game_menu::test_game_menu_shop_button_opens_store_and_menu_can_reopen_after_store_closes`

### LOD shard 10 errors

- `addon_coverage_lod::load_on_demand_blizzard_addons_shard_10_stays_within_known_error_baseline_after_startup`
- `addon_coverage_lod::shard_14_runtime_load_survives_prior_runtime_shards_in_process`

### Achievement comparison fixture

- `blizzard_ui_blizzard_achievementui::behavior_comparison_friend_completed::comparison_template_init_completed_writes_format_short_date_and_saturates_friend_card`
- `blizzard_ui_blizzard_achievementui::behavior_comparison_friend_completed::comparison_template_init_incomplete_writes_incomplete_token_and_short_circuits_on_repeat_id`

### Modified-click mutation

- `keybindings::modified_clicks_round_trip_through_runtime_surface`
- `startup_api_stubs::startup_client_time_camera_and_channel_globals_are_callable`

### Combo-frame consumer selection

- `click_targeting::click_targeting_forever_regressions::blizzard_player_world_entry_updates_combo_frame_from_target_power`

### Missing cooldown duration policy

- `cooldown_restriction::ssp1::missing_and_no_state_secret_selectors_preserve_existing_empty_results`

### PTR eager load-order snapshot

- `load_order::test_blizzard_addon_load_order_snapshot`

### Quest tracker block count

- `objective_tracker_tree::objective_tracker_frame_layout_is_locked`

### Spellbook geometry

- `spellbook_layout::spellbook_layout_stays_locked`

### Store free upgrade API

- `store_tree::store_free_upgrade_check_handles_account_without_upgrade`

## R22 evidence received

VERIFIED (PTR 12.1.7.70668; reconstructed native behavior in `pr46-49-decomp-answers.md#r22`): `C_WowTokenUI` has `ShouldDisplayInAuctionHouse()` and `StartTokenSell`; the consumer calls the zero-argument boolean getter, whose result is always true in this build. Retail 12.1.5.70688 does not publish the getter.

VERIFIED (PTR and Retail; same answer as R20): `C_StoreSecure.HasFreePromotionalUpgrade` is nil; the function exists only as `C_SharedCharacterServices.HasFreePromotionalUpgrade()` and returns false without a qualifying promotion.

VERIFIED (PTR): accepted `SetModifiedClick(action, binding)` updates the next `GetModifiedClick(action)` immediately. Known unbound actions return `NONE`, unknown actions return nil, and the setter rejects bad action/binding inputs. INFERRED (PTR): the native setter silently does nothing when any of three additional PTR context checks fail; their meanings are UNKNOWN, so do not model them. Retail has no extra PTR checks.

## R23 evidence and implementation decisions

VERIFIED (PTR 12.1.7.70668, native evidence in `pr46-49-decomp-answers.md#r23`): `C_PartyInfo.GetLootMethodStyle()` reads game rule 157 (`LootMethodStyle`) with default 0. It does not read group membership, the selected loot method, or a CVar. R23 also verifies the same expression on Retail 12.1.5.70688. The simulator getter must read the live game-rule value on each call and return 0 when the rule is absent. `SetLootMethod` must not change this value.

VERIFIED (PTR shipped Lua): `GAME_RULES_CHANGED` is synchronous and carries no payload when the game-rule collection is replaced. INFERRED: a consumer that displays loot style must re-read `GetLootMethodStyle()` on that event; there is no style-specific event. Do not make `A_Admin.SetGameRule` synthesize collection-replacement events.

VERIFIED (R17 parsers on Forever 1.60.1.70707 and Retail 12.1.5.70688; PTR 12.1.7.70668 matches per R23): the spell identifier is required and accepts number/string forms; invalid or missing identifiers error. INFERRED (R17/R23 normal results): a valid unknown identifier returns zero Lua values, while a supported spell without an active cooldown returns one zero `LuaDurationObject`, including with `ignoreGCD=true`. Keep cooldown/GCD timing selection unchanged unless another pinned answer requires it.

Design note: implement the loot-style getter from `SimState.game_rules.rules["LootMethodStyle"].as_int`, defaulting to 0, without caching or changing loot-method mutation. Apply R17 `GetSpellCooldownDuration` parser/result behavior only where `uses_r17_duration_behavior()` is true; preserve earlier-profile behavior. Acceptance tests were committed as `56ff76723`.

## R29/R33 cursor and ActionBar verification — 2026-11-10

- VERIFIED: R33 (`pr46-49-decomp-answers.md#r33`) specifies the PTR Game-only `C_ActionBar.SpellMatchesActionSpellTypeRestriction` contract, including its usage/range errors, zero-result slots outside 1–408, normal-mode true result, and WoW Hack label groups. `C_GameRules.IsWoWHack` was already modeled; the implementation reuses that predicate and an empty-by-default host-supplied SpellLabel map.
- VERIFIED: the R29 producer is `d8f3a0956`. R33 tests were committed first as `5d2084323` and failed on the missing member; the logs are `/var/tmp/osso/jcol/ptr-int-clean-r33-red-native-mode-run.log` and `/var/tmp/osso/jcol/ptr-int-clean-r33-red-native-errors-run.log`. The query, host label input, and spec are in `6601f0c09`. Its first build exposed E0502 because the `SimState` borrow outlived the result calculation; `a628368cb` scopes that borrow before pushing the Lua result. The EditMode grid fixture correction is `8520962f5`.
- VERIFIED: PTR build receipts `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-r33-green-a628368cb.json` and `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-r33-gridfix-8520962f5.json` succeeded with zero compiler warnings; the final PTR artifact is `integration-4927c2ef62e6c1de`. Retail receipt `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-r33-retail-8520962f5-r1.json` also succeeded with zero warnings; its artifact is `integration-b852c2890d72c5da`.
- VERIFIED: exact PTR selectors passed 1/1: `action_bar_drag::ptr_action_spell_restriction_matches_native_mode_and_slot_rules`, `action_bar_drag::ptr_action_spell_restriction_reports_native_argument_errors`, `action_bar_drag::ptr_action_button_drag_matches_r29_cursor_and_event_order`, `action_bar_drag::ptr_action_button_drag_swaps_occupied_spell_without_cursor_change`, `action_bar_drag::ptr_action_bar_lock_is_checked_by_the_lua_drag_consumer_only`, `action_bar_drag::ptr_tainted_combat_pickup_is_blocked_and_place_clears_the_cursor`, `action_bar_drag::action_slot_limits_match_the_client_profile`, `action_bar_drag::pickup_action_accepts_ignore_removal_arg_and_place_restores_slot`, and `action_bar_drag::cursor_info_projects_existing_payloads_with_profile_deltas`.
- VERIFIED: exact Retail selectors passed 1/1: `action_bar_drag::cursor_info_projects_existing_payloads_with_profile_deltas`, `action_bar_drag::action_slot_limits_match_the_client_profile`, `action_bar_drag::pickup_action_accepts_ignore_removal_arg_and_place_restores_slot`, and `action_bar_drag::retail_action_spell_type_restriction_is_absent`.
- VERIFIED: each selector listed exactly one test and ran alone through `/var/tmp/osso/jcol-run.py`; its fresh CVar copy and O_NOATIME before/after guard preserved source SHA-256 `8d1709049bc9acdbed0465889701a0e370667ef18cf06b71db479b3972aa350c`. Logs are under `/var/tmp/osso/jcol/`.
- VERIFIED: `default_action_bars()` in `src/lua_api/game_data.rs` seeds slot 2 with spell 31935, so the tainted-place test had a pre-populated target; its blocked trace had no slot-change event. The fixture now clears slot 2 and passes. The ActionButton drag test also resets `showgrid` to zero before checking the drag-event bit, because EditMode seeds `REASON_CVAR=1`; `tests/blizzard_ui/blizzard_actionbar/behavior_show_grid.rs` documents this fixture requirement.
- VERIFIED: `cargo fmt --all` and `git diff --check` passed; no Blizzard/vendor or Cargo files changed.
- UNVERIFIED: the separate PTR startup/presentation failure cluster and full profile failure-name comparison were not rerun as part of this R29/R33 slice.

## R37 merchant cursor

- VERIFIED (R37; PTR 12.1.7.70668 and Retail 12.1.5.70688): a successful merchant pickup projects exactly `"merchant", index`; R37 also verifies invalid-index clearing, fraction truncation, overflow errors, and successful-pickup `CURSOR_CHANGED(false, 5, 0, …)` with no `ACTIONBAR_SHOWGRID` on PTR.
- INFERRED (R37): no available merchant row clears the cursor. INFERRED (R37): normal PTR `PlaceAction` leaves a merchant cursor and target unchanged with no events; its tainted/insecure exception is verified to block and clear the cursor.
- INFERRED (R37; PTR 12.1.7.70668): when the cursor holds an owned item, the wrapper takes a separate route that is probably a sale. UNKNOWN (R37; PTR 12.1.7.70668): its transaction, final cursor state and events are not established. Existing simulator behavior remains unchanged; no native-parity assertion is added.
- UNKNOWN (R37; PTR 12.1.7.70668): the same-item guard's final cursor state and events are not established. Existing simulator behavior remains unchanged; no native-parity assertion is added.
- UNKNOWN (R37; PTR 12.1.7.70668): `CursorHasItem()` on a merchant cursor is not specified. WM46's mode-1-only predicate evidence is Retail 12.1.5.70688; no PTR assertion is made.
- VERIFIED (simulator source): both UNKNOWN paths continue through the existing synthetic merchant-cursor path. `tests/merchant_cursor.rs` has no assertions for either path.
- VERIFIED: the PTR integration target built at `022249c5ca87a1c4053125fa7da72cc65db633d1` with zero warnings; artifact `integration-63be1c5fb8e6d7af`, receipt `/home/osso/.cache/wow-ui-sim-targets/slot-1/receipts/ptr-int-clean-r37-green-022249c5c.json`.
- VERIFIED: all four `merchant_cursor::*` tests passed; `inventory_verbs::pickup_merchant_item_uses_the_supplied_offer` and `action_bar_drag::ptr_tainted_place_merchant_cursor_reports_block_and_clears` each passed 1/1.
- VERIFIED: the three test services used byte-identical CVar copies and left the original unchanged: SHA-256 `8d1709049bc9acdbed0465889701a0e370667ef18cf06b71db479b3972aa350c`, size 811, with matching before/after stat fields. Logs and JSON receipt: `/home/osso/.local/state/wow-ui-sim/verification/ptr-int-clean-r37/`.
- VERIFIED (source/evidence scope): `CursorHasItem()`'s mode-1 predicate is established by WM46 for Retail 12.1.5.70688 only; R37 does not establish its PTR merchant result. The unsupported PTR assertion is removed and recorded as UNKNOWN.
- UNVERIFIED: the PTR full-suite run, current-failure comparison, targeted Retail tests and remaining profile builds are still pending.

## Current-master comparison — baseline PTR failures

VERIFIED (master `aeb3ce50ecf3351f29675a1134370fc9361df2ea`): the PTR integration/library targets built with zero warnings, and all 26 names from the prior PTR failure set were run once with exact filters. Sixteen also failed on master:

- `action_bar::test_bag_bar_layout_locked`
- `bags::test_container_frames_array_contains_only_real_container_frames`
- `bags::test_container_frames_registered`
- `blizzard_ui_blizzard_achievementui::behavior_comparison_friend_completed::comparison_template_init_completed_writes_format_short_date_and_saturates_friend_card`
- `blizzard_ui_blizzard_achievementui::behavior_comparison_friend_completed::comparison_template_init_incomplete_writes_incomplete_token_and_short_circuits_on_repeat_id`
- `blizzard_ui_parent_loads::ptr_mainline_excludes_mists_time_string_helper`
- `click_targeting::click_targeting_forever_regressions::blizzard_player_world_entry_updates_combo_frame_from_target_power`
- `keybindings::modified_clicks_round_trip_through_runtime_surface`
- `micro_menu::micro_menu_professions_button_loads_and_opens_panel`
- `objective_tracker_tree::objective_tracker_frame_layout_is_locked`
- `path_control_point::path_actual_cached_smooth_xml_configures_points_and_restores_playback`
- `spellbook_layout::spellbook_layout_stays_locked`
- `startup_api_stubs::startup_client_time_camera_and_channel_globals_are_callable`
- `workarounds_bags::bag_frame_rect_queries_match_render_layout_rects`
- `workarounds_professions::casting_blacksmithing_opens_professions_frame`
- `workarounds_professions::clicking_blacksmithing_button_in_professions_book_opens_panel`

VERIFIED: the other ten exact selectors passed on master: `action_text::action_text_empty_spell_and_unplaceable_item_controls`; `container_split_use::b1_invalid_amounts_are_inert_then_valid_split_succeeds`; `container_split_use::b1_occupied_cursor_and_missing_source_are_inert`; `container_split_use::b1_split_moves_supplied_quantity_and_reports_cursor_count`; `game_menu::test_game_menu_shop_button_opens_store_and_menu_can_reopen_after_store_closes`; `inventory_verbs::place_action_writes_spell_id_to_action_bar_slot`; `store_tree::store_free_upgrade_check_handles_account_without_upgrade`; `timeout_reexec::timeout_children_are_limited_to_two_concurrent_processes`; `transmog_outfit_catalog_inputs::transmog_outfit_catalog_inputs_pickup_sets_cursor_and_clear_works`; and `xml_create_error_reporting::partially_created_parent_key_and_array_child_reports_original_nested_failure`.

VERIFIED: the 26 master runs used isolated byte-exact CVar copies and preserved the original CVar SHA-256 `8d1709049bc9acdbed0465889701a0e370667ef18cf06b71db479b3972aa350c` and recorded stat fields. Build receipt and per-selector logs: `/home/osso/.local/state/wow-ui-sim/verification/ptr-int-clean-master-aeb3ce50e/`.
UNVERIFIED: these results classify the earlier PTR failure set only; compare against the current PTR full-suite failures after that run completes.
