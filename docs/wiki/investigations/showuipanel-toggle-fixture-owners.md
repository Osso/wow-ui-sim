# ShowUIPanel toggle fixture ownership

The Retail housing-dashboard and achievement toggles in the partial panel fixture require their original addon publishers. Retiring guessed fallbacks exposed omissions in that fixture, not a reason to restore synthetic behavior.

## Classification and owner loading

The achievement fixture omitted the original LoD bootstrap. Pinned Retail `Blizzard_AchievementUI.toc:2` is LoD, `:4` depends on FrameXMLUtil, `:5` admits mainline, and `:6` selects the Bootstrap file. `Blizzard_AchievementUI_Bootstrap.lua:13–17` loads the addon then calls `AchievementFrame_ToggleAchievementFrame`. The fixture now queries normal startup discovery, confirms that AchievementUI is present, and loads its original TOC bootstrap with `load_addon_bootstrap_from_toc` after its eager dependencies, preserving its actual addon argument, chunk provenance and bootstrap lifecycle. It does not copy/replace the original Lua source or restore the guessed toggle.

The housing fixture omitted eager `Blizzard_HousingTutorials`. Its TOC selects Game at line 2 and standard game type at line 3, declares Blizzard_Tutorials at line 5 and loads Data before Util at lines 7–8. `Blizzard_HousingTutorialsUtil.lua:52–54` supplies the original `BoughtHouseQuestComplete` function, which the dashboard's `Blizzard_HousingDashboardHouseInfoContent.lua:45` reads. Normal Retail startup discovery selects it as `Full` after Blizzard_Tutorials; the test asserts both selection and dependency order. Thus `standard` is not the cause: the partial fixture omitted the root. The fixture includes this root only on ACTIVE Retail in the existing dependency-ordered closure; it does not force the function true, alter the CVar or mark a quest completed.

Source-ownership assertions verify both original providers; the achievement panel implementation is verified after its original loader opens it. Existing assertion text and conditions remain unchanged. No producer, vendor source, fallback or event behavior changes. No quest/CVar input is altered and no completion value is asserted or synthesized; the original provider reads the existing host state.

## Out-of-scope diagnostics

The first Lua diagnostic for both target selectors is `TargetFrameAuraContainer.lua:242` missing `AuraContainerUtil`, also observed at the passing base. It is not the newly exposed missing publisher. ClubFinder.lua:475 DropdownButtonMixin occurs in other ShowUIPanel fixture logs, including passing selectors, but not in either target transcript. No ClubFinder fix is attempted.

Before the fixture fix, all 68 ShowUIPanel selectors yielded 63 passes and five failures. In addition to the two targets, open-all-bags and mail have tokenFrame failures, and professions specialization has a GetChildrenForPath failure. These remain separately reported; the task does not repair them. Canonical full startup is verified independently rather than treating partial-fixture diagnostics as full-startup regressions.

## Sources

- Tracker lt-900e and independent archive receipts (first-bad details appended after verification).
- `tests/test_showuipanel_toggles.rs` and `tests/blizzard_housing_dashboard_loads.rs:54–117`.
- Original pinned Retail owner TOCs and Lua files cited above.

## Independently rebuilt boundaries

- Achievement: `35c287515ce4214aae824c584fcbba6c5f794d85` PASS → `9003cf923b9d33187a61524aa34d6e97dea7a232` FAIL (`missing_toggle_achievement_frame`). Master first-parent introduction is merge `57b73744799b62a9ebf918462af0f0b0f6e31a2c`.
- Housing: `dad6933d9549817d25cc65ff41c1929d4dfed363` PASS → `9c50fd8eee00f75a2f262c20539a9a25848dde68` FAIL (`no_houses_dashboard_not_shown`). Master first-parent introduction is merge `7a694fa5baee411f56b1bc34549f1340cf6dc1c3`.
- Base `c36368269b70aae0f04ec2f75272ee035401c45d` passes both and current `964011f25022127fa7905f824d1eddb6e42b54c4` fails both. The other target passes at each independent boundary. These are history-guided adjacent-boundary proofs, not a claim every intervening revision was run.

The exact test file is byte-identical at all six revisions (SHA256 `ba4d54935e66372ef63649586fd2ae98cc3b71221ff8dae4b62de25125df2e78`). Independent archives refresh all build inputs, require relevant compiler artifacts fresh, and bind executable/actual loader dylib before/after each isolated runtime. Historical `9003cf923` has an unrelated lib-unit-test E0425 (removed `patch()` called by `close_menus.rs:31`); the preserved failed full build is followed by a fresh integration-only historical run without source workaround. Final gates still build both lib and integration.

Coordinator lt-900e comment 1275 accepts the fixture-omission classification and original owner routes. The initial return-to-current compile after historical target use exposed stale historical dependencies and was discarded without runtime; a complete current-input mtime refresh preceded successful current receipts.

Normal startup currently promotes AchievementUI to `Full` through `implicit_blizzard_startup_dependencies` (`loader/mod.rs:182`) and `collect_startup_prerequisites` (which pulls keys from the LoD pool). The explicit fixture closure (`loader/mod.rs:626–647`) uses only its roots and declared dependencies, not those implicit startup keys, so the owner was omitted. This is not evidence of a broken native bootstrap loader; the partial fixture needs only the real TOC bootstrap before lazy loading. A draft diagnostic wrongly assumed `BootstrapOnly`; observed startup kind was `Full`, and that new diagnostic was corrected without altering any original selector assertion.

## See Also

- [[master-test-fixture-triage]] — related false test-fixture failures and Mists profile gates.

## Retail bag tracker lifecycle (lt-c0e6)

**VERIFIED (Retail 12.1.5.70783, base aa1192ddc):** the unchanged
`open_all_bags_opens_a_bag_frame` fails at `ContainerFrame.lua:2464` because
`tokenFrame` is nil. Both original owners are already in the fixture closure.
`Blizzard_UIPanels_Game_Mainline.toc` admits Game/mainline and includes
`Mainline/ContainerFrame.lua`; its settings manager registers `ADDON_LOADED`
at lines 2160–2170 and creates `BackpackTokenFrame` only for
`Blizzard_TokenUI` at lines 2173–2178. The selected TokenUI TOC includes
`Blizzard_TokenUI.xml`, whose line 269 defines `BackpackTokenFrameTemplate`.

The partial fixture called low-level `load_addon` without dispatching each
owner's completion event. It now dispatches `ADDON_LOADED` after each loaded
owner, using the same argument helper as normal startup, before its existing
`VARIABLES_LOADED` event. The original owner creates the tracker; no substitute
frame, fallback, vendor edit or assertion change is needed. The unchanged bag
test passed after restoring these events. This is fixture lifecycle preservation,
not a new native-semantics claim.

The current fixture already selects `Blizzard_Menu`, whose `DropdownButton.lua`
defines `DropdownButtonMixin`. The baseline bag transcript does not contain the
historical ClubFinder/DropdownButtonMixin error, so no speculative second fix is
included. Other partial-fixture diagnostics remain outside this task.
