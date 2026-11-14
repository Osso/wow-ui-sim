# Lua-owned fallback retirement (inventories O and J)

Original Blizzard Lua/XML owns the utility, registry and lifecycle behaviors in this investigation. Retiring simulator substitutes is not a native API implementation and must not be advertised as new decompiler evidence. Tracker lt-cbb9 contains the eleven-file synthesis and deferred native questions. The delivered scope separates original-consumer preservation from synthetic bootstrap retirement and residual follow-ups.

## Evidence boundaries

The selected Retail 12.1.5.70575 TOCs and cached files load through the ordinary addon loader, not translated test functions. Consumer probes exercise original AuctionHouseUtil formatting, Azerite model-scene orchestration, dropdown/slider templates, chat metadata/editbox transitions, panel lazy attributes and stack behavior, widget registrations, EventUtil callbacks, journal bootstraps and CodeOfConduct login delivery.

Some full-load consumers already overwrite bootstrap shims. Passing original behavior is preservation, not proof that removing the shim changed those successful full-load outputs. Separate bare-environment assertions establish retirement of synthetic pre-owner publication. Malformed-receiver probes distinguish original Lua call/error behavior from source patches; they do not establish that native clients normally create malformed frames.

## Delivered retirement tranche

- Dropdown template/style owns Text. UpdateText no longer creates substitute children or skips missing children.
- ChatFrameUtil uses original ChatTypeInfo and editbox/click-anywhere-button transitions, not fabricated yellow metadata or skipped lifecycle calls.
- SecondsFormatter and StaticModelInfo come from original SharedXML/SharedXMLGame. Removed bootstrap copies included divergent formatter enum/count/configuration behavior and static actor nil-result substitutions.
- RegisterUIPanel preserves the original attributes-table identity and lazy installation. Bootstrap no longer publishes a shadow registry or eager clone.
- Original panel stack/menu helpers replace synthetic ShowUIPanel, HideUIPanel, CloseAllWindows and CloseMenus publication. Probes exercise center-slot displacement, lazy attributes, special-window closure and actual dropdown XML menus, including original return arity. Original function provenance and pre-owner absence are distinct from those populated preservation checks.
- UIWidgetManager is the original XML frame/registry, not an empty table with no-op registration.
- Slider thumb updates and callable-table EventUtil callbacks retain original behavior instead of source-side suppression.
- Journal toggle owners remain the selected original Achievement/Collections/Encounter bootstraps after post-load and runtime-addon hooks, rather than being republished as simulator functions. Collections tests start bootstrap-loaded but not fully loaded, then exercise actual LoD loading, same-tab close, cross-tab selection, selected-tab CVar and original callback payloads. Achievement tests exercise regular/statistics transitions with already modeled earned rows.

## Login notice: source rewrite was not the only suppressor

Removing the CodeOfConduct source rewrite alone did not make the notice persist. The real PLAYER_ENTERING_WORLD callback ran, but the simulator subsequently called apply_post_event_workarounds, whose show-chat initialization unconditionally cleared ChatFrame1 history. The fix removes that destructive post-world clear rather than inventing a replacement notice or fake chat collector.

The original-notice test verifies selected addon loading, actual ChatFrame1.historyBuffer and original ChatFrameUtil/GetMessageInfo provenance. Diagnostic observers preserve and forward the original callback with its owner and unchanged arguments. The real initial-login event must preserve preexisting history and append ONLINE_SAFETY_NOTICE once; a reload must not append it again. The chat regression run after the history fix passed **171 tests, 0 failed** (stub-o/chat-regression.log). This count is that chat-focused run, not a full-suite or all-O certification.

## Achievement summary: retirement versus missing data

The summary lifecycle wrapper/veto is removed, leaving the original updater. Existing populated coverage uses modeled earned IDs 6,7,8,9,10 and verifies real visible rows with empty-text hidden.

The stronger empty-history/incomplete-default scenario remains data-blocked: the original default list requires IDs 6,503,116,545,1017, while 503/116/545/1017 lack modeled metadata. The candidate is retained outside executable tests; it is not a passing scenario or a proven target RED. Do not fabricate native rows, replace the original array/max count, skip a failing precondition, or infer default achievement records from display behavior. Authoritative data/model coverage is a follow-up.

## Startup navigation and cleanup ownership

Original selected actionbar XML produces the action buttons, container links and edit-mode registration. Original MarkAllSettingsDirty invalidates the Lua settings cache. Help XML supplies its OnUpdate timer, whose due/disabled scheduling is checked with a delegating GetWebTicket observer. This establishes Lua scheduling, not native ticket responses.

Original TutorialHelper opens all existing modeled bags. The original LFD link respects the toggle lock and selects the group-finder panel. GameEvent's roster consumer refreshes the compact manager from existing party/raid state. These selected-owner preservation checks pass without replacing native producers or installing the panel harness's status/grid substitutes.

The remaining navigation module is removed from both initial bootstrap and EnvironmentCleanup restoration. Five actionbar placeholder frames and invented dirty methods no longer precede the original owners. SettingsPanel's injected dirty method was active during normal bootstrap; its absence is checked before and after cleanup restoration. Mail/Inbox/PVE injections could also activate on surviving frames during restoration. The independent SettingsPanel placeholder and main-menu click bridge remain outside this retirement.

The merchant consumer candidate stopped at unmodeled ResetSetMerchantFilter after opening bags; no native replacement was invented. The tutorial consumer avoids that unrelated prerequisite. Native denied group-finder eligibility and Help ticket response behavior remain unclaimed.

## Deferred boundaries and follow-ups

- Conditional residuals: arena parent/children, voice-transcription metadata, LocalizationMachinery deduplication, store/token/microbutton lifecycle (overlap G2/lt-8c9e), FloatingChatFrame fontstrings and TTS helper resurrection. The coordinator will create follow-up work after merge. No blanket claim that all eleven O files are removable or gone.
- Generic Frame IsInDefaultPosition returns false without systemInfo and interferes with managed templates. Native method discovery identifies the simulator edit-mode methods as extras; separate task lt-8a50 owns removal. No synthetic systemInfo, override or ignored test hides that boundary here.
- All seventeen tracking/trade/world-service native producers remain unchanged and queued for R88. Their empty/false/nil/zero fixtures are not native evidence. Classic-only and Forever-only questions must not inherit Retail answers without applicability proof.
- Exact texture-filename and zone-story producer questions remain in the O synthesis.

## J: original character labels and titles

Tracker lt-a655 reuses accepted synthesis #603 and authorization #628. The
Retail selected-name guard was retired from its registration/module, retaining
other profiles unchanged. Actual CharacterSelect discovery selects
`Blizzard_GlueXML_Mainline.toc`, whose original XML supplies the name label.
Default host-roster and explicit empty-host-roster consumers preserve original
formatting and absent-season tooltip clearing. Their behavior is PRESERVATION;
the separate original-source identity assertion is retirement RED (the wrapper
reported `=(string)` before removal). No missing label, callback substitution,
synthetic season or character-creation action is introduced.

The Retail final CharacterFrame title overwrite contradicted original
`CharacterFrameMixin:UpdateTitle`: selected `Blizzard_UIPanels_Game_Mainline.toc`
loads Mainline CharacterFrame Lua/XML, whose active Reputation/Token subframes
own REPUTATION/CURRENCY titles. Each independent test uses genuine
`ToggleCharacter`, verifies original source and title first, then fires the first
real PLAYER_ENTERING_WORLD without resetting the one-shot guard. The original
event legitimately hides the outer panel; the retained active-subframe title
must still remain authored. Before the fix both titles became the player name
(Uther in this host fixture). The narrow Retail policy removes that forced
name assignment; remaining OnShow/RefreshDisplay and slot-refresh bridges are
not retired without separate evidence. Non-Retail title assignment ordering
and failure behavior remain unchanged.

The five J selectors and three touched surface unit tests pass. Fourteen of
fifteen existing character-select/reputation regressions pass;
`show_ui_panel_locks_reputation_frame_layout` retains `character_not_shown`,
independently reproduced with the sealed pre-J35106b01b binary and matching
dylib. That baseline failure is not filtered or repaired by this tranche.
Retail startup reports CLEAN with zero Lua errors; this is not a full-suite claim.

No J native defaults were implemented. Kiosk, Perks, PrototypeDialog, legacy
session/video behavior, promotion inventory and store-service semantics remain
NATIVE/neutral-KEEP as synthesized. Creation/overlay/Enter World owners and
covered R facts remain untouched. Authentication challenge parenting and
secure/public binding remain unresolved, not blanket removable fallbacks.

- [Selected-name original consumers](../../../tests/stub_j_selected_name.rs)
- [CharacterFrame title consumers](../../../tests/stub_j_character_title.rs)
- [Remaining character surface bridge](../../../src/lua_api/workarounds/temporary/character_frame_surface_refresh.rs)

## Sources

- [Retirement consumer probes](../../../tests/stub_o_source_ownership.rs)
- [Original utility consumers](../../../tests/stub_o_lua_owned_helpers.rs)
- [Chat ownership](../../../tests/stub_o_chat_owner.rs)
- [Panel/widget ownership](../../../tests/stub_o_ui_ownership.rs)
- [Journal ownership/lifecycle](../../../tests/stub_o_panel_next.rs)
- [Panel stack/menu ownership](../../../tests/stub_o_panel_core.rs)
- [Login notice/history](../../../tests/stub_o_notice_owner.rs)
- [Original navigation consumers](../../../tests/stub_o_navigation.rs)
- Tracker lt-cbb9 comments 792–795 (synthesis), 796 (formatter correction), 803–804 (R88), 813 (managed-method deferral), 832 (finish navigation, deliver proven tranche, defer residual conditional patches).

## See Also

- [[consumer-selection-and-red-proof]] — source selection versus executed mutation rejection
- [[post-load-workaround-audit]] — remaining lifecycle ownership and retirement paths
