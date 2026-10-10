# Forever finite UI constants

## Contract

Forever 1.60.1.69913 publishes these additions from its generated API documentation, without changing other client profiles:

- `MinimapTrackingFilter.TrainerClass = 8388608`, `VendorAmmo = 16777216`; metadata: 26 values, range 0–16777216.
- `PingResult.FailedSilent = 8`; metadata: nine values, range 0–8.
- `Constants.Transmog.NoTransmogID = 0`.
- `GamepadPossessBarOverride`: SpecialPageTopBar=1; Page1LeftBar/RightBar/BottomBar=2/3/4; Page2TopBar/LeftBar/RightBar/BottomBar=5/6/7/8; Page3TopBar/LeftBar/RightBar/BottomBar=9/10/11/12. Metadata: twelve values, range 1–12.

Sources: authenticated Forever `MinimapConstantsDocumentation.lua`, `PingConstantsDocumentation.lua`, `TransmogConstantsDocumentation.lua`, and `GamepadUIDocumentation.lua` under `Blizzard_APIDocumentationGenerated`.

## Stance override enum

Forever publishes `Enum.GamepadStanceBarOverride`: `None=1`, Page1 Left/Right/Bottom=2/3/4, Page2 Top/Left/Right/Bottom=5/6/7/8, Page3 Top/Left/Right/Bottom=9/10/11/12. `GamepadStanceBarOverrideMeta` has MinValue=1, MaxValue=12, NumValues=12. Publication remains Forever-only.

Evidence: cached `GamepadUIDocumentation.lua` and `Resources/LuaEnum.lua` in Ketho/BlizzardInterfaceResources revision `659e8042049df854c114714f8ecd640823a1cd5c`, whose README identifies build 1.60.1.69913. Unlike possession, the first stance value means no override.

The grouped regression executes unchanged `StanceBar.lua` and `GamepadOverrideBarMixin.lua` against frame-backed fixture anchors. It exercises all twelve mappings, default right-anchor positioning, linked-bar identity, and unparented `None` behavior. Page-owner getters are fixture inputs; this is not full PageUnit startup proof.

## Legacy and level constants

Forever `LegacyConstantsDocumentation.lua` publishes `Constants.LegacyConsts`: `LEGACY_REWARD_TRACK_FACTION_ID=2802`, `LEGACY_POINTS_TRAIT_CURRENCY_ID=4225`, `LEGACY_TREE_PROFESSIONS_ID=1187`, `LEGACY_TREE_ADVENTURE_ID=1188`, `LEGACY_TREE_PROGRESSION_ID=1189`, and `LEGACY_TREE_ADVENTURE_TALENTED_NODE_ID=110298`.

Forever `LevelConstantsDocumentation.lua` publishes `Constants.LevelConstsExposed.MIN_RES_SICKNESS_LEVEL`, `MIN_ACHIEVEMENT_LEVEL`, and `MIN_TALENT_LEVEL`, each `10`.

The actual Camelot `PlayerSpellsMicroButtonMixin:GetTalentUnlockLevel()` returns `10` when adventure tree `1188` has no config. An unknown tree returning nil remains valid; this publication does not change the trait model or claim configured-tree behavior. Grouped tests assert all nine values and execute the unchanged Camelot override file against the real `C_Traits` API.

## Quest log limit

Forever `QuestConstantsDocumentation.lua` publishes `Constants.QuestLogConsts.MAXIMUM_NUM_QUESTS_LOG_CAN_ACCEPT = 40`. The Camelot quest-map count refresh compares the current quest count against this value while `WorldMapFrame:Show()` initializes the map. It remains Forever-only.

The world-map runtime regression shows the real failure boundary: without the constant, `QuestLogQuests_ShowQuestCount()` aborts `WorldMapMixin:OnShow()` before `SetMapID()`. The scroll container consequently retains a nil `targetScale`, and every later `OnUpdate` fails in `IsZoomingOut()`. The test shows the map and runs sixty GUI-style updates, asserting a positive target scale and no collected errors; no vendor guard or scale fallback is added.

## Bag indices

Forever alone publishes nine character-bank tabs (`CharacterBankTab_1..9 = 6..14`) and nine account-bank enum slots (`AccountBankTab_1..9 = 15..23`). `BagIndexMeta` has MinValue=-3, MaxValue=23, NumValues=27. The nine non-bank-tab members retain their shared values (-3 through 5); enumeration terminates at each missing tenth tab. Other profiles retain their existing publication.

Authoritative evidence: `Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua` in the Forever 1.60.1.69913 cache, sourced from Gethe/wow-ui-source revision `70ef1b2fd78061a73f886c4a1e79dc5b5cff6d5e`, lines 18–51. BetterBags [Forever introduction `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941`](https://github.com/Cidan/BetterBags/commit/411a6f6ee1ea40eca8ac96927ccdd49a6aab3941), `core/constants.lua`, motivated the audit: its `enumerateBagIndices` consumer walks contiguous named members. The regression uses that exact loop, asserting exact values, disjoint IDs, termination, all non-bank-tab values, member count, and metadata through `WowLuaEnv`.

Implementation: `src/c_api/forever_finite_constants.rs`, registered only for `client-wowforever`. Tests: `forever_bag_index_enumerates_disjoint_bank_tabs` and `forever_bag_index_preserves_non_bank_values_and_exact_metadata` in the existing grouped `tests/wowforever_finite_constants.rs` module. This is enum-publication coverage, not a full BetterBags load or proof of account-bank availability, purchased tabs, bank events, or money-display lifecycle.

## Finite event registration

Forever additionally accepts `CHAT_MSG_COLLECTED_APPEARANCE` and `UNIT_AURA_BLOCKED`, published by cached `ChatInfoDocumentation.lua:1348–1351` and `UnitAuraDocumentation.lua:641–645`. Both belong to the sorted Forever-only event list; arbitrary unknown names remain rejected. Other profiles and the undocumented `PLAYER_EQUIPED_SPELLS_CHANGED` name remain unchanged.

`forever_finite_events_register_deliver_and_reject_unknown` tests registration, injected callback delivery with payload preservation, unregistration, and unknown-name rejection in the existing grouped integration module. Injection proves simulator dispatch, not native event production or payload secrecy. The frozen pre-patch binary rejects both names (RED, isolated ledger `/tmp/forever-addon-audit/forever-finite-events-red-ww5p10a5/ledger.json`); compiled GREEN and addon replay remain pending.

### Aura block-list cleared registration — bounded independently verified GREEN

Source-inspected 2026-10-10: the Forever profile cache's `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:631–638` declares `UNIT_AURA_BLOCK_LIST_CLEARED`, synchronous, with one required `unitTarget: UnitTokenVariant` payload. This is a source declaration, not native firing or payload-parity proof.

- [x] Accept this exact event through the finite Forever registration override for both `RegisterEvent` and `RegisterUnitEvent`; do not make event validation permissive.
- [x] Preserve rejection of the tested unrelated unknown name through both registration methods, without retaining a failed registration. This is the known negative fixture, not exhaustive unknown-name testing.
- [x] Preserve default-mainline non-registerable event policy, including `COMBAT_LOG_EVENT` and `COMBAT_LOG_EVENT_UNFILTERED`, by source isolation: default tables and validator functions are unchanged. This is source-audit evidence, not a newly executed default-mainline runtime test.
- [x] In simulator-injected fixtures, deliver one unit payload, filter `RegisterUnitEvent` by its registered unit, and suppress delivery after unregistration. Injection is not native parity.

`src/event/valid_events.rs` now includes this name in the finite `FOREVER_REGISTERABLE_EVENTS` override; it already appears in the separately gated `PATCH_12_1_REGISTERABLE_EVENTS`. The current `NON_REGISTERABLE_EVENTS` contains only the two combat-log names above. The earlier private applicability handoff's claim that the cleared event is in that non-registerable table does not describe the inspected current source. The repair is limited to the finite Forever override: Retail 12.1 already registers this name, and the shared combat-log non-registerable policy is unchanged. No fallback, vendor patch, or event-dispatch implementation change is part of this slice; the source-declared synchronous payload does not establish simulator dispatch timing.

Fresh test-stage assertions in `tests/wowforever_finite_constants.rs`: `forever_aura_block_list_cleared_register_event_delivers_unit_payload`, `forever_aura_block_list_cleared_register_unit_event_filters_target`, and `forever_aura_block_list_cleared_unknown_event_control_rejects_both_methods`. These assert registration/injected delivery, unit filtering/unregistration, and actual unknown-name rejection respectively; source inspection alone does not establish their runtime results or default-mainline preservation.

Authentic test RED at `20e808b94`, epoch `20261010T190445Z`: compilation exit0/source equality; both registration/delivery positives fail at unknown `UNIT_AURA_BLOCK_LIST_CLEARED`, while the unrelated-unknown control passes (2FAIL/1PASS, execution101). Earlier epoch `20261010T190145Z` was resource-blocked, not behavioral RED.

Independently audited GREEN epoch `20261010T191909Z`, submission `ef55bf873ce7c9bcca3d88a3d690d447edf7a897`: the actual execution receipt and output report 18/18 PASS, zero failures, exit0, expected/reached18, and unchanged sealed artifact. Repair `d5dcc8b5953ef1e62a49f58845919077167a4a0a` adds only this literal to the finite Forever list. All fifteen original test bodies and the three new test bodies are unchanged between authentic RED and GREEN; source inventories differ only in `src/event/valid_events.rs`. Independent report: `/home/osso/.local/state/wow-ui-sim/verification/forever-aura-block-event-green-current/independent-report.md`. Main retains tracked reports at `data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-aura-block-event-{red,green}/independent-report.md`; these retained paths now exist in this checkout.

Main-owned formatting, default compile check, and Forever compile check receipts pass; these are not native event-firing proof or a full acceptance gate. Check/compile logs retain six dependency manifest-key deprecation warnings. Runtime-backed checkmarks cover only the known injected fixtures; default-mainline preservation is explicitly source-audited. No default-table code, vendor behavior, dispatch implementation, permissive validation, or fallback changes belong to this repair.

**Stock-startup acceptance: PENDING independent audit.** Prior baseline was19unique/36occurrences, including16unknown-event occurrences. Main observed post-repair epoch `20261010T192210Z`: actual `lua-errors.stdout` is `[]`; `runtime-result.json` records exit0, completion marker, unchanged artifact, source equality, and vendor-cache equality. This observation does not close the independent stock audit or whole-profile gate. Native firing, payload parity, secrecy, and whole-startup acceptance remain open.

Post-repair stock epoch `20261010T192210Z` is independently verified **PASS for bounded normal completion and distinct CLI error state only**: dump completion marker/exit0; separate CLI exact `[]`, exit0, 0 unique/0 occurrences. Normal dump `casc=true`, CLI `casc=false`; no shared in-memory pool claim. [Integrated proof SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#forever-post-event-repair-stock--bounded-normal-completion-and-distinct-cli-zero) owns artifact reuse, source/cache bounds, hashed retained receipts and exclusions. Historical19/36 and original422/514 remain preserved; attribute/pool disappearance has no established causal explanation. No native, GUI, all-profile, cold-state or third-party completion.

## Guild Discord event and aura sound trigger enum

Forever also accepts `CHAT_MSG_GUILD_DISCORD`, explicitly published by cached `ChatInfoDocumentation.lua:1676–1680` and consumed by Chattynator. Registration, injected delivery, unregister suppression, and arbitrary-name rejection are covered by the existing finite-event regression. Other documented residual events are not added by this change.

`UnitAuraConstantsDocumentation.lua:6–16` publishes `Enum.UnitAuraSoundTrigger`: `Added=0`, `ApplicationsIncreased=1`, `Removed=2`, with `UnitAuraSoundTriggerMeta` MinValue=0, MaxValue=2, NumValues=3. Forever shares the existing retail 12.1 enum producer; other profiles retain their previous publication policy. The grouped regression checks all three values, exact member count, metadata, and repeated restoration after cleanup removes both tables. This proves enum publication, not aura sound playback.

Frozen `b8f0982be` rejects the event and lacks both enum tables: `/tmp/forever-addon-audit/next-registration-red-fc0so85f/ledger.json`. The owned `319725248` registration slice passes the existing grouped integration filter 10/10, including repeated enum restoration and guild Discord delivery/unregistration. Other agents edited unrelated source during compilation; the executable is not proof of an exact whole-repository revision. Build and test ledgers: `/tmp/forever-addon-audit/next-registration-green-q_hgyboa/`. Focused integration now passes 10/10, including guild Discord delivery/unregistration and repeated enum restoration. An exact `ebff90517` replay clean-starts Chattynator and EnhanceQoL with the same binary hash; this proves registration/publication only, not native event production, aura sound playback, or whole-addon workflows.

## Player swing event

Forever accepts exactly the documented `PLAYER_SWING` event with payload order `(swingDuration: number, swingType: PlayerSwingType)`. `Enum.PlayerSwingType` publishes `MainHand=0`, `OffHand=1`, `Ranged=2`; `PlayerSwingTypeMeta` publishes MinValue=0, MaxValue=2, NumValues=3. Source: cached `Blizzard_APIDocumentationGenerated/SwingTimerDocumentation.lua:43–51, 72–84`. The C API-owned enum producer runs through shared enum bootstrap so cleanup restoration republishes the same values. Publication remains Forever-only and unknown event names remain rejected.

AppelSwingsForever `8925606`, `Swing.lua:23–60`, registers this event and routes its duration/type payload to independent main-hand, off-hand, and ranged state. It ignores nonpositive durations and clears completed progress rather than holding a full bar. This change does not modify that addon or synthesize gameplay swings. The initial exclusion of range-event registration was superseded by the actual GUI replay: Blizzard's main-hand, off-hand, and ranged SwingTimer frames reject `PLAYER_SWING_RANGE_UPDATE` during OnLoad.

Two grouped regressions assert exact two-value synchronous Admin-injected delivery for all three types, callback order, unregister suppression, unknown-name rejection, exact enum membership/metadata, and restoration after normal bootstrap cleanup. Injection is bounded simulator dispatch evidence, not real gameplay production or native conformance. Frozen `5a1cc851` RED rejects `PLAYER_SWING` and lacks the enum: `/tmp/forever-addon-audit/player-swing-red-corrected-fwmpzx12/ledger.json`. Targeted development GREEN at clean `c818f1b915ba811a8aee0d8cda9f29acc3d7eb39` passes the full finite-constants group 12/12, including both swing regressions. The earlier 11/12 result exposed missing enum restoration; moving the same producer into shared enum bootstrap fixed that boundary. Build/test ledgers: `/tmp/forever-addon-audit/player-swing-final-green-avb1bbsv/`. The build has unchanged source hashes and no warnings; `wow-sim` SHA-256 is `8536d33c7f603fafd2db4d155c262c4c61be4de4776ed7324a33295092e8abf2`.

### Swing range registration follow-up

Forever also accepts the documented synchronous `PLAYER_SWING_RANGE_UPDATE(swingType, isInRange, checksRange)` event (`SwingTimerDocumentation.lua:54–63`). Both boolean positions retain false/true values independently; `checksRange=false` does not mean an out-of-range observation. Registration survives bootstrap cleanup, and arbitrary unknown events remain rejected.

Frozen `c818f1b9` reproduces rejection in Blizzard SwingTimer OnLoad and the focused delivery probe: `/tmp/forever-addon-audit/swing-range-red-u9ot06pm/ledger.json`. At clean `ff19ecca24d398580584d08f258e9a4134b2abae`, the grouped finite-constants target passes 13/13, including range payload and cleanup tests; source hashes match before/after the build. Evidence: `/tmp/forever-addon-audit/swing-range-green-8s22zw34/{build,test}-ledger.json`.

The unchanged AppelSwingsForever package clean-starts at frozen `ff19ecca`; its matching `--no-addons` control also returns `[]`. An isolated GUI-style replay injects a four-second main-hand and six-second ranged `PLAYER_SWING`, observes fill activation and progress, then observes independent main-hand expiry before ranged expiry and idle tracks retained after both clear. It emits `DONE`, collects zero Lua errors, and preserves host CVars; timeout `124` occurs after completion. Evidence: `/tmp/forever-addon-runtime/player-swing-startup-controls-ledger.json` and `/tmp/forever-addon-runtime/appel-swing-final-gui-olh1svgs/ledger.json`.

This models registration and injected payload delivery only—not range detection, `C_SwingTimer` methods, a gameplay event producer, off-hand visuals, or native timing. Independent verification at `aad86deb7` passes the 13-targeted-test group, `cargo fmt --check`, and default `cargo check --offline`; it validates the frozen GUI evidence. Ledger: `/tmp/forever-addon-audit/verify-player-swing-ledger.json`.

## Camelot stable slot counts

Forever publishes `Constants.PetConsts.MAX_STABLE_SLOTS = 2` and `NUM_PET_SLOTS_HUNTER = 3`, merging into the existing common table. Cached `Blizzard_APIDocumentationGenerated/PetConstantsDocumentation.lua:46–54` specifies two stable slots plus one learned-spell slot; unchanged Camelot `Blizzard_StableUI.lua:75,150,249` consumes these two fields. The dependency value is source evidence for the sum, not an additional published field in this slice.

- [x] Publish both documented values without replacing common pet sentinels or `MAX_SUMMONABLE_PETS`.
- [x] Preserve table identity, addon fields and slot values through normal bootstrap restoration.

Implementation: `src/c_api/forever_finite_constants.rs`, behind its existing Forever-only registration. Tests: `forever_stable_slot_constants_preserve_common_pet_values` and `forever_stable_slot_constants_survive_bootstrap_restore` in the existing grouped integration module. Frozen `3d6017fe3` fails both initial Lua checks. At `fffb25ae4`, the grouped finite-constants target passes 15/15, including both new regressions and repeated normal bootstrap restoration; independent proof also passes `cargo fmt --check` and default offline `cargo check`. Ledger: `/tmp/forever-addon-audit/verify-stable-constants-ledger.json`.

- [x] Replay the no-addons `PLAYER_MONEY` boundary: hunter-slot count prints `3`, then Camelot reaches `Blizzard_StableUI.lua:219`, where callable `C_StableInfo.GetNumStableSlots()` returns nil.
- [x] Replay unchanged Aurarium and ArcaneWizardLibrary: both packages load, money history `12345 → 54321` plus overview open/close reach `DONE`, but the same StableUI nil result produces errors and exit `1`.
- [ ] Establish a state-backed `C_StableInfo.GetNumStableSlots` contract and replay cleanly; the two constant fields do not model stable state.

No pet ownership, stable purchasing, extra Camelot constants, other-profile publication or vendor behavior changes. Bootstrap already retains this table; no duplicate restoration producer is introduced.

## Verification

`tests/wowforever_finite_constants.rs` checks publication, actual Camelot minimap filter construction, and full PingManager/TransmogShared source loading in an initialized simulator environment. Initial tests reproduced missing PingResult data and the MinimapConstants nil table key (0/2); both passed after publication. TransmogShared loaded in the focused fixture, so its full-startup failure is not proven to arise solely from NoTransmogID. These additions do not establish complete Transmog initialization, gamepad possession behavior, or StableUI state; remaining consumer failures must be diagnosed independently.
