# Classic consumer fixture applicability

This bounded lane repairs test fixture selection before assigning classic integration failures to simulator APIs. It starts at `75038d409` and covers Era, Anniversary and Mists. No vendor behavior or native API contract changes are authorized by fixture failures alone.

## Historical failure ranking

VERIFIED: The inventory's final integration failure lists contain 1,003 Era names, 1,003 Anniversary names and 1,076 Mists names. These are older captures, not this lane's required fresh master baseline. Parse names only after the last `\nfailures:\n`, with `^    ([A-Za-z0-9_:]+)$`. For classification, select the first actual panic inside each named stdout block; ignore the outer timeout-reexecution panic and deduplicate repeated child headings.

| Rank | Era first-error names | Decision | Scope |
|---|---:|---|---|
| 1 | 462 | (b) Wrong shared panel roots | `common::panel_fixtures` and its startup/smoke consumers |
| 2 | 110, plus 7 micro-menu TOC assertions | (b) Retail dependency arrays | Shared keybinding/game-menu fixtures and action/guild/panel loaders |
| 3 | 41 explicit routing failures; 3 further composite assertions need confirmation | (b) Wrong native blocked-action consumer | `protected_method_coverage` |
| Later | 18 generic missing-TOC panics, 12 PlayerSpells loads, 12 ClickBinding bootstrap reads, other unique errors | Not one proven root cause | Separate consumer-specific triage |

Mists and Anniversary have the same 462/110 first-error clusters. Mists also has 50 explicit `C_ArtifactUI` boolean-index failures, which are separate from this batch.

## Decided rules

### 1. Native classic startup, not Retail panel roots

VERIFIED: In all three classic caches, `Blizzard_TimerunningUtil` contains two files and its TOC line 5 declares `## AllowLoadGameType: mainline`. `Blizzard_POIButton`, `Blizzard_GameMenuEsc`, `Blizzard_ClickBindingUI` and `Blizzard_Game` have no directory/files in these pins. This does not mean classic panels, action bars or menus are absent.

VERIFIED: `Blizzard_ActionBar_Classic.toc:4-6` names classic dependencies, declares `classic` and allows Game. Each classic cache contains 46 ActionBar files. `Blizzard_GameMenu_Classic.toc:4-6` selects GameTooltip/UIPanelTemplates and `classic`; its directory contains six files. The profile's selected UIParent TOC loads `Vanilla/UIParent.lua` for Era/Anniversary and `Mists/UIParent.lua` for Mists.

Use the existing production-backed `common::load_game_startup_inventory` for these three classic fixtures. Preserve Retail/PTR/Forever loading and Wrath's native FrameXML path. Do not make `load_required_blizzard_addon` silently skip missing roots, and do not filter an arbitrary Retail array until it happens to load. Retain dependency ordering from the selected startup inventory.

`load_panel_addons` must work independently for closure harnesses. `setup_env` must not continue into absent Retail bootstrap reads after loading the classic inventory. Fixture-specific real LoD roots remain explicit, subject to native selection; missing applicable consumers are not replaced by stubs.

### 2. Apply the same root choice to POI-blocked fixture families

Use the same classic startup decision in keybinding, game-menu, action-bar and guild fixtures, then the direct POI-dependent variants where necessary. Keep existing test intent and startup events. Do not gate action/button/keyboard tests merely because their current fixture demands POIButton: their classic consumers exist.

Gating is permitted only for independently verified absent features or different selected XML/Lua contracts. For example, Era/Anniversary ship nine AchievementUI files but have no vanilla-eligible AchievementUI TOC: the three TOCs allow cata, tbc/wrath or mists. Mists does have a native achievement panel, so a classic panel must not be removed simply because Retail coordinates differ. Inspect the specific selected XML before using existing profile-gate macros for Retail-only geometry or PlayerSpells assertions. Any such exclusions must be listed in the report.

### 3. Observe the original classic blocked-action handler

VERIFIED: `Blizzard_UIParent/Classic/UIParent.xml:23` binds `UIParent_OnEvent`; Era/Anniversary `Vanilla/UIParent.lua:92` register `ADDON_ACTION_BLOCKED`, and `:657-658` call `DisplayInterfaceActionBlockedMessage`. Mists registers at `Mists/UIParent.lua:96` and calls the same function at `:842-843`. No `Blizzard_Game` addon exists in these three cache directories.

Keep all existing protected-method state, taint, owner and event assertions. On classic, the test observer wraps the existing UIParent `OnEvent` script, records the owner, calls the original handler, and records that the hooked original display function runs; it does not synthesize GameEvent. On Retail/PTR/Forever it retains the GameEvent observer. A listener alone is insufficient proof that the Blizzard consumer ran. Unrelated roleset availability is deferred unless the pinned declarations prove a gate; do not add missing methods to satisfy this fixture.

## Implementation: native blocked-action consumer

The 41 direct RED cases now select their observer by profile surface. Classic tests call the existing UIParent handler and observe `DisplayInterfaceActionBlockedMessage`; Mainline tests keep the `GameEvent.HandleActionBlocked` observer. Existing owner, action, event-count, combat, and taint assertions remain in place. This changes the test's native consumer, not simulator or vendor behavior.

## Implementation: native startup roots

The shared panel fixture now calls `common::load_game_startup_inventory` on Mists, Era and Anniversary. This uses the same profile-selected startup TOCs and dependency ordering as simulator startup. It does not silently skip missing roots or fabricate TimerunningUtil/POIButton. Retail, PTR, WowForever and the existing Wrath branch keep their prior paths. The first lane-owned RED reproduced TimerunningUtil at `tests/common/panel_fixtures.rs:128:9`; the GREEN run and post-baseline profile counts are recorded in the untracked handoff report.

## Implementation: POIButton-dependent fixtures

The same native-startup selector now replaces the hard-coded Retail addon arrays in action-bar, game-menu, keybinding, guild, reputation-panel and micro-menu fixtures for Mists, Era and Anniversary. It retains their explicit arrays and bootstrap behavior on other profiles. This also avoids loading `Mainline/TargetFrame.lua` into a classic fixture. GuildControlUI remains an explicit LoD load at the guild consumer. Classic TOCs prove that ActionBar, GameMenu, MicroMenu and UnitFrame have native classic files; these consumers are not gated merely because the Retail-only POIButton root is absent.

The lane-owned RED for `action_bar::test_action_bar_env_can_bootstrap_twice_in_same_process` reproduced the missing POIButton TOC at `tests/common/mod.rs:244:9`; the committed game-menu acceptance RED reproduced the same boundary. Both exact tests now pass on Era. The guild panel, guild micro-menu, target-frame visibility and character-keybind consumers also pass targeted Era runs.

### Mainline-only bag helpers

VERIFIED: Each Era/Anniversary/Mists cache has 142 files under `Blizzard_UIPanels_Game`, but zero copies of `Mainline/ContainerFrame.lua` and `Mainline/ReputationFrame.lua`. `Blizzard_UIPanels_Game_Classic.toc:7` is the selected TOC; it selects one container implementation (`Vanilla/ContainerFrame.lua:65` for Era/Anniversary, `Mists/ContainerFrame.lua:67` for Mists) and `Classic/ContainerFrame_Shared.lua:68`. Each of those source paths has one file in the cache. The selected Classic/Mists Lua files do not define `ContainerFrameSettingsManager` or `EnumerateValidItems`. Retail `Blizzard_UIPanels_Game_Mainline.toc:6,37` selects `Mainline/ContainerFrame.lua`, where those members and token ownership are implemented. `C_Container.SetItemSearch` is separately declared in every pinned `ContainerDocumentation.lua` (`:393`), and the shared `ItemButtonTemplate.lua` supplies `GetMatchesSearch`; the profile gate applies only to the test's Mainline frame enumeration, not those shared API members. Each Classic cache has eight `Blizzard_TokenUI` files; the selected Era/Anniversary `Blizzard_TokenUI.toc` and Mists `Blizzard_TokenUI_Mists.toc:5-10` load legacy sources that publish `BackpackTokenFrame` without a `ContainerFrameSettingsManager` ownership contract.

Gate only the manager-ownership and Mainline frame-enumeration tests to Retail/PTR/WowForever with the existing `retail_ptr_forever_toc_fixture!` profile set. Keep classic TokenUI loading and legacy bag-icon tests; skip only the modern combined-bag toggle when the active Classic client has no manager. Do not treat the panel/fixture failure as evidence that bag frames or backpack tokens are absent. The legacy token helper now asserts the manager relationship only when that manager exists; it still requires the original `BackpackTokenFrame` on every profile.

VERIFIED: Each Classic cache has 30 `Blizzard_CharacterFrame` files, including one `Vanilla/CharacterFrame.lua` and one `Cata/CharacterFrame.lua`. The addon TOC selects the Vanilla file for Era/Anniversary, where `RefreshDisplay` is absent, and the Cata file for Mists, where it is defined. The layout test is therefore not registered for Era/Anniversary; this is a selected-file contract gate, not a panel-availability gate. The filter-dropdown probe uses `retail_ptr_forever_toc_fixture!`: each Classic cache has zero `Blizzard_UIPanels_Game/Mainline/ReputationFrame.lua` files; its `filterDropdown` assertion belongs to the Mainline/Camelot consumer, not the selected Classic sources.

UNRESOLVED — native-evidence gap for main: the selected `Blizzard_UIPanels_Game/Vanilla/ReputationFrame.lua:24` (Era/Anniversary) and `Classic/ReputationFrame.lua:33` (Mists) call the legacy global `GetNumFactions`; those consumers also call the legacy global `GetFactionInfo` (`Vanilla/ReputationFrame.lua:40`, `Classic/ReputationFrame.lua:65`). No global declaration for either function appears in the pinned generated-documentation trees (535 files for Era/Anniversary, 534 for Mists). The same-named `C_CreatureInfo.GetFactionInfo` at `CreatureInfoDocumentation.lua:85` is a different namespace. Native return shapes and backing state remain unknown. Do not add a shim or infer a faction count in this lane.

## Acceptance and execution order

Commit `tests/classic_consumer_applicability.rs` before fixture changes. Its two cases execute original classic UIParent/action-button and menu consumers through the broken shared fixtures. Expected initial boundaries are TimerunningUtil and POIButton, respectively. Existing `protected_method_coverage::protected_input_clamped` is the third boundary. Historical logs already show these causes; fresh lane-owned RED runs remain required and must not be replaced by the historical evidence.

VERIFIED historical RED excerpts from the inventory's Era log (source `183edc381`):

- `achievement_panel_layout::achievement_frame_layout_stays_locked`, `tests/common/panel_fixtures.rs:128:9`: ``required panel addon `Blizzard_TimerunningUtil` has no eligible Game TOC under /home/osso/.cache/wow-ui-sim/blizzard-ui/era/AddOns``.
- `action_bar::test_action_bar_env_can_bootstrap_twice_in_same_process`, `tests/common/mod.rs:244:9`: ``required Blizzard addon `Blizzard_POIButton` has no compatible TOC under /home/osso/.cache/wow-ui-sim/blizzard-ui/era/AddOns/Blizzard_POIButton``.
- `protected_method_coverage::protected_input_clamped`, `tests/protected_method_coverage.rs:98:10`: `attempt to index global 'GameEvent' (a nil value)`.

1. Claim one slot through `/var/tmp/osso/slot-claim.py`, retrying every two minutes. Keep a detached baseline checkout pinned to `75038d409`. Build/run each touched profile's full integration baseline in this lane's slot before switching that profile to changed sources. Include `optional-tests` so the historical modules remain covered. Run tests through `/var/tmp/osso/runtests.sh`.
2. Commit each of the three fixes separately, formatted, with focused RED/GREEN receipts. Do not accumulate implementations while running checks.
3. Run final full integration once for Era, Anniversary and Mists. Each failure set must be a strict subset of this lane's fresh master set, not just a lower count. New acceptance tests must pass.
4. Run affected shared test files on Retail/PTR/Forever against fresh master comparisons. Run six `cargo check --all-targets` profiles with zero warnings and successful build-finished cargo JSON; compile Wrath. Rehash any edited file pinned by `data/patch-api/*.json` and run the manifest-artifact test if applicable.
5. Record remaining clusters, native unknowns, SHAs and before/after names in the untracked lane report. No push, rebase, squash or stash; only no-ff merges if integration is requested.

## Open native questions and limits

- Roleset methods, incidental `FlashClientIcon` errors and other API gaps are not established producer defects by the routing fixture failure.
- Classic selected panel geometry and generated Retail snapshot shapes need individual contracts; unblocking a fixture does not prove those tests applicable or passing.
- Counts above classify the first observed boundary. Secondary failures may remain under the same names after fixture repair.

## Sources

- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md) — applicability and original-consumer standards.
- [Client profiles](../systems/client-profiles.md) — selected caches and native startup inventory.
- [Classic startup cascades](../investigations/classic-startup-producer-cascades.md) — secondary failures and producer attribution.
- `~/.local/state/wow-ui-sim/handoff/master-failure-inventory/REPORT.md`, cluster 2 and its three raw integration logs — historical first errors.
- `~/.local/state/wow-ui-sim/handoff/classic-consumer-applicability/native-cache-inventory.tsv` and `*-ranked-clusters.md` — exact local file counts, TOC metadata and all historical names.

## See Also

- [[prefork-test-harness]] — full-UI and grouped fixture boundaries.
