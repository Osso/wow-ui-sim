# Forever addon comparison audit

A complete public CurseForge catalog capture provides the comparison corpus; it does not prove addon compatibility. The first evidence-backed correction is Forever-only `Enum.BagIndex` publication, motivated by BetterBags’ contiguous bank-tab enumeration and confirmed by the pinned 1.60.1.69913 API documentation.

## Catalog coverage

Commit `7eb74d91e` records the public CurseForge `1.60.1` / Forever catalog in [the comparison audit](../../forever-addon-comparison.md): 875 unique projects across all 44 rendered A–Z pages, with no duplicate or missing rows. Every captured page reported 875 projects and the ending page-one check retained that count and ordering. The live catalog was previously observed at 871, so this capture is a dated 875-project observation rather than a stable total.

Commit `6397eb7a3` freezes this pass to offline use of the already acquired archive set after bulk acquisition stopped. [The comparison audit](../../forever-addon-comparison.md) is the single source for the acquisition boundary, cached-pair/triage coverage, parked remainder, and candidate dispositions. A Forever tag is a declaration, not an API contract or a passing simulator result.

## Cached consumer follow-up

`31ac46b3a` corrects TOC selection so dash/underscore Camelot variants precede generic TOCs. The focused selector suite passes 44 targets; cached Carbonite providers and dependents now load. Its later `OnUpdate` reaches map 2521 and first uses the modern map-art API path. Map-art metadata is absent, so the legacy path fails; this is not a simple addon filename defect. Exact-build map-art records and decoder inputs are unavailable, therefore no metadata, asset name, or fallback is invented.

`9862dc7b3` removes the public Forever `C_CombatLog.GetCurrentEventInfo` exposure while retaining `C_CombatLogInternal` and `C_CombatLogSecure`. Three new and four existing namespace checks pass. Cached EpicDamageMeter starts cleanly and its modern path completes 60 updates on one instance with two named rows. This is seeded render-path evidence, not native combat evidence. The combined final gate passes at `572f1c23c`; [the shared proof record](../../forever-addon-comparison.md#follow-up-verification) records exact reuse, fresh checks, and remaining baseline warnings/blockers.

## Current producer follow-up

Three current Forever corrections are committed but have no GREEN verification at this audit point. `232bc7e72` adds only the generated-documentation events `CHAT_MSG_COLLECTED_APPEARANCE` and `UNIT_AURA_BLOCKED` to the finite Forever registration list; the latter was retail-only before this change, not already deployed to Forever. `f1c0a19a8` models public `C_Spell.GetBaseSpell` identity for unconfigured relationships after cached ActionBarAuras passed a nil result into a table key. It intentionally ships no live override data and rejects secret identifiers. `374c2c6c7` assigns Forever current expansion `0` as an inferred policy so cached Angleur's `WOW_PROJECT_MAINLINE` predicate selects its Camelot branch instead of consumer code whose retail producers are outside the Camelot TOC.

These are bounded simulator-model changes. They do not certify ActionBarAuras, Angleur, event delivery, native override/secret behavior, native expansion identity, or the inventory.

## BigWigs Classic-expansion predicate follow-up

The unchanged BigWigs `8931513` archive failed during bundled LibDualSpec initialization at line 372 because `ClassicExpansionAtMost` was absent. Cached Forever API documentation requires a numeric argument, boolean result, and untainted caller for secret arguments. The focused `e11c0d792` tests, implementation `ae045495a`, and policy placement `fbcd9bb51` add only Forever's upper-bound comparison against the existing temporary Classic level `10`; the separate Forever current-expansion value remains the inferred `0`. The library itself notes its Classic AtLeast query is effectively true on Forever and uses build range to select dual specialization, so the two expansion identities are not conflated.

Independent verification reuses the hash-matched 3/3 focused proof, formatting, default compilation, frozen binary, and exact 442-file archive. The BigWigs root loads without Lua errors, but Core/Options/Plugins remain deferred LoadOnDemand and all eight encounter TOCs use `AllowLoadGameType: standard`, which Forever excludes; they are not only Midnight roots. The matrix changes the previous failure to **partial/unloaded**, not clean all-root startup. A real `/bw` route now reaches deferred plugin activation but fails at `BigWigs_Plugins/Auras.lua:2408` on missing `CanBeAccessedInContext`; options and `DONE` are not reached. Native Classic expansion metadata is unavailable; level `10` remains a temporary compatibility policy. See [Classic expansion upper-bound predicate](../../specs/classic-expansion-at-most.md) and [the startup record](../../forever-addon-runtime-coverage.md#bigwigs-partial-startup-after-classic-expansion-comparison).

## Script-object context-access follow-up

`d4b0401f3` records the missing `CanBeAccessedInContext()` boundary reached by an unchanged BigWigs `/bw` deferred-plugin workflow. `ec6c3c9f7` adds the bounded Forever query: it uses actual caller taint, the frame forbidden flag, and `DenyTaintedAccessWhenAurasAreSecret` only when explicit per-environment `auras_secret_in_context` is active. The context defaults inactive and is not inferred from combat or static aura-secrecy metadata. The method returns an ObjectSecurity-secret boolean when the existing frame aspect mask requires it, using the pinned rilua host-secret-boolean API; it does not add general access enforcement to other methods.

The mixed-source `283029e6` binary is excluded. Frozen `e3cafcc11` proof covers 17 unique integration tests across 18 executions (one overlapping filter). After readability-only refactors, final `0bee9e939` revalidates context 4/4, parser 3/3, formatting, and default offline checking; unchanged-path proof is reused. Exact unchanged BigWigs (442 staged files) runs `/bw`, loads Core, Plugins, and Options, reaches `options-open` and `DONE`, and records `[]`; five sound-chat warnings occur after that array. Rilua `a5edc4c` is approved, published, and pinned: 19 focused secret-boolean tests pass, while its 462/463 integration result retains one baseline-confirmed nil-diagnostic mismatch. No full-suite, native, sound, test-bar, raid, pixel, or whole-inventory claim follows. See [script-object context access](../../specs/script-object-context-access.md).

## Template child OnLoad lexical-self follow-up

The final BigWigs replay exposed four post-`DONE` AceGUI child-OnUpdate errors after the first context-query correction. Its XML child OnLoad assignment used lexical `self`; the fast template path had classified bare and dotted `self` roots as `_G.self` globals. `103e6422c` and `e11037217` send those roots through authoritative Lua evaluation while preserving genuine dotted globals. The three bounded template-parenting tests pass. This is a simulator parser correction, not a Blizzard/vendor patch; other local-expression forms remain unmodeled. See [template child OnLoad self](../../specs/template-fast-path-self.md).

## BigWigs Create Test Bar workflow

Fixture `e826e801d` drives unchanged BigWigs `8931513` through the real `/bw` options route, requires the visible AceGUI TreeGroup, clicks its `general\u0001Bars` tree node, then clicks the enabled AceGUI Create Test Bar button. It does not invoke a saved callback or a BigWigs bar producer directly. Frozen `0bee9e939` with the exact 442-file archive records a new localized bar at `21` seconds, progress to `20.80755216` seconds remaining, expiry/hide, and `DONE` before the external timeout. The read-only audit confirms raw output contains zero Lua errors and the host CVar hash is unchanged: `/tmp/forever-addon-audit/verify-bigwigs-testbar-ledger.json`.

This is a second bounded BigWigs interaction inside the unchanged 18-pass / 251-not-run matrix, not a startup reclassification. It does not prove pixels, texture correctness, audio playback, raid or encounter behavior, native Forever behavior, persistence, or all-root loading. CASC was disabled with 73 missing textures. The five historical custom-sound warnings are resolved by the separate loose-asset replay below. See [runtime coverage](../../forever-addon-runtime-coverage.md#bigwigs-partial-startup-after-classic-expansion-comparison).

## BigWigs loose sound-asset root cause

Frozen `0bee9e939` queried all five existing BigWigs sound paths and returned `IsKnownFile=false`, `IsLooseFile=false`, and `GetFileID=nil`; the unchanged package then emitted five custom-sound reset warnings. The cached `C_UIFileAsset` documentation says known files include loose files, while the pre-fix simulator only consulted its shipped listfile and hard-coded `IsLooseFile=false`. Commits `77513cbed` and `a9afe0231` retain the loader-selected TOC directory and add selected-root loose-file recognition; the 3/3 development proof covers load-time/later `.ogg` queries, selected-root isolation, extensionless textures, and traversal/symlink rejection.

Frozen `963a3b791` (SHA `2f24d4…`) then ran unchanged BigWigs through `/bw` and the deferred Sounds plugin. All five staged `.ogg` paths were known/loose with `nil` IDs, each registered at its exact path, extensionless `Otravi` stayed valid, `DONE` and `[]` occurred, reset warnings were zero, and host CVars were unchanged: `/tmp/forever-addon-runtime/bigwigs-sound-assets-public-_wheduwi/ledger.json`. Independent audit passes focused loader/API checks 3/3, warning-free formatting/default checks, readability, and the exact replay: `/tmp/forever-addon-audit/verify-ui-file-assets-ledger.json`. Final source `5913de433` only removes an unused classification payload, so the frozen `963a3b791` behavior proof is validly reused. The simulator's regular-file existence probe and `nil` loose-file ID are bounded inferred policies: cached native docs explicitly say loose-file existence/openability is not verified and do not specify loose IDs. No audio-playback or native-filesystem claim follows. See [UI file asset spec](../../specs/ui-file-assets.md).

## Recent-allies location-preference follow-up

`561943dd0` supplies five focused Forever regressions and `37da0f132` adds the per-environment `GetAllowRecentAlliesSeeLocation` / `SetAllowRecentAlliesSeeLocation` state. Cached Forever UI metadata documents the boolean setter and synchronous payload-free `LET_RECENT_ALLIES_SEE_LOCATION_SETTING_UPDATED` event; cached Settings metadata supports the default `true`. Existing VM secret-argument validation remains the enforcement route, rather than a new security mechanism.

A changed value updates state before synchronously dispatching the event. Same-value suppression is a simulator inference: the cached Settings listener writes the getter value back, so duplicate notification would recurse. Independent verification reuses the 5/5 target, passes formatting/default offline checking/readability, and validates the frozen `wow-sim-37da0f13` artifact. The unchanged Account-wide UI archive later completes its bounded self-cast save/change/load route after the neighborhood and guild preference follow-ups; this does not prove persistence, network visibility, native coercion parity, other-profile behavior or all-settings fidelity. See [location-preference spec](../../specs/recent-allies-location-preference.md).

## Guild-invite preference follow-up

`7529257de` adds five focused Forever regressions and `6632d6373` adds only `SetAutoDeclineGuildInvites`, reusing the existing `SimState.auto_decline_guild_invites` getter/state. Cached Forever documentation supplies the optional boolean argument default `false` and `AllowedWhenUntainted` contract. Existing stored initial `false`, VM secret validation, getter behavior, and no-event policy are preserved; no native initial-state claim is made. Independent verification reuses the 5/5 proof and passes formatting/default offline checking/readability/security: `/tmp/forever-addon-audit/verify-guild-preference-ledger.json`. The exact unchanged 81-file Account-wide UI archive reaches `saved-zero → restored-zero → DONE` with empty Lua errors and unchanged host CVars: `/tmp/forever-addon-runtime/account-guild-workflow-hgl3rw6d/ledger.json`. This credits only the self-cast round trip through complete handlers—not all-settings, bag preferences, persistence, native behavior or rendered UI. See [guild invite preference](../../specs/guild-invite-preference.md).

## Neighborhood-invite preference follow-up

`9733a87a8` adds five focused Forever regressions and `6eccec458` supplies the independent `GetAutoDeclineNeighborhoodInvites` / `SetAutoDeclineNeighborhoodInvites` state. Cached Forever documentation declares an `AllowedWhenUntainted` boolean setter with optional-argument default `false`. It does not establish the stored initial value; the per-environment initial `false` is an explicit simulator guess. The globals retain VM secret-argument validation and introduce no event because cached sources do not document one.

Independent verification reuses the 5/5 targeted GREEN proof, passes formatting, default offline checking, security, and readability, and authenticates frozen `wow-sim-6eccec45`. The later guild-setter follow-up completes Account-wide UI's bounded self-cast save/load route; neighborhood preference semantics beyond that route remain unproven. See [neighborhood-invite-preference spec](../../specs/neighborhood-invite-preference.md).

## Camelot stable-read follow-up

`9a8e2a841` and `fffb25ae4` first publish only the two required Forever `Constants.PetConsts` fields: `MAX_STABLE_SLOTS = 2` and `NUM_PET_SLOTS_HUNTER = 3`. Their frozen replay reached the next boundary: `Blizzard_StableUI.lua:219` compared `id - 2` with a nil `C_StableInfo.GetNumStableSlots()` result. Cached StableInfo documentation and the unchanged Camelot consumer identify four required reads: `GetNumStableSlots`, `GetNextStableSlotCost`, `GetNumStablePets`, and `GetStablePetInfo`.

`e6f5e5792` adds the targeted read-state regression and `a6fbf432e` models those four calls only for Forever. Its explicit default is two owned stable slots, empty pet storage, and unavailable next purchase at cost `0`; that fully unlocked state is a simulator guess, not native evidence. Configured pet records expose documented fields as snapshot tables on public one-based slots, while the inferred pet count includes the current-pet slot. Existing `IsAtPetStable` behavior, the open stable-bonus-slot probe, and other-profile publication remain unchanged.

Independent verification reuses 4/4 focused tests, passes formatting and default offline checking, and verifies wiring/readability. Frozen `wow-sim-a6fbf432` passes the no-addons `PLAYER_MONEY` control and exact cached Aurarium `8915742` plus ArcaneWizardLibrary `8915500` replay: both exit 0 with empty Lua-error JSON; Aurarium records money `12345 → 54321`, overview open/close, and `DONE`. The verifier authenticated the frozen binary, both archives (109 and 28 members), staged fixture, and unchanged host CVar hash. Matrix `ac3063db3` changes Aurarium from failed to bounded-pass, making the current interaction totals 14 bounded passes, one failure, and 254 not run; startup totals are unchanged. This is neither native pet behavior nor inventory acceptance: purchases, swaps, favorites, food, gameplay producers, persistence, security, rendering, and other-profile runtime behavior remain out of scope. See [stable read state](../../specs/forever-stable-read-state.md).

## Accountant and BagMeter workflow follow-up

The unchanged cached Accountant Classic `8919183` package receives actual `PLAYER_MONEY` events through its registered handler. Its bounded fixture primes the current balance, then verifies a `+250` income and `-250` expense in `OTHER` Session, Day, and Total records, restores saved balance, and reaches `DONE` with empty Lua-error JSON. Independent verification authenticates all 92 staged archive files, the frozen binary, fixture markers, empty errors, and matrix totals. This is simulator-injected money-event evidence only; accounting rollover, other categories, persistence, settings, rendering, and native behavior remain unproven.

BagMeter `8917130` is different: Forever's loader selects generic `Bagmeter.toc`, not its unrecognized `_Forever.toc`; explicit out-of-date loading admits that generic TOC. The actual generic `Bagmeter.lua` path updates bag text through `A_Admin.ClearBags`, `AddBagItem`, and `BAG_UPDATE`: `15/16 → 14/16 → 15/16` and aggregate `79/80 → 78/80 → 79/80`, reaching `DONE`. Two earlier fixture prechecks derived from the unselected Classic source were discarded. The shared merchant read family now supplies the needed closed-merchant queries, so both the exact 23-file generic-TOC replay and a no-addons control exit 0 with empty Lua-error JSON. Independent verification authenticates the 11/11 focused source proof, fresh format/default checks, changed-Rust readability, frozen artifact, archive, outputs, and CVar isolation. BagMeter receives bounded generic-TOC/OOD workflow credit; current loader policy, not native addon-author intent, establishes that selection, so `_Forever.toc` remains untested. Matrix `2e39c97ba` records 17 bounded passes, zero failed workflows, and 252 not run; startup totals are unchanged.

## Merchant repair and buyback read follow-up

`4024a6444` and `9acd62a82` add a Forever-only `CanMerchantRepair` predicate: existing merchant-open state and a distinct configured repair capability must both be true. Default capability `false` and this conjunction are simulator policy, not native observation. Existing merchant inventory, `CanMerchant`, open/close behavior, and other profiles remain unchanged; no repair service, costs, durability, guild-bank repair API, or repairing-merchant UI is modeled.

`cc6567131` and `88d84bc0` then add distinct per-environment legacy buyback snapshots and `GetNumBuybackItems`, `GetBuybackItemInfo`, and `GetBuybackItemLink`. The initial empty collection, one-based ordering, empty indexed results, snapshot transaction fields, and metadata availability policies are simulator inferences; no buyback transaction or sale mutation is modeled. Four focused buyback tests pass.

`67b32cddd` and `58b0072d1` add only the required `C_MerchantFrame.GetNumJunkItems` read, then `8260074a3` records the targeted proof. Eleven focused tests cover required merchant reads, buyback state, junk count, and the cached BAG_UPDATE route; that route is clean. Positive junk metadata exists only in test fixtures because the live catalog has no quality-0 records. Eligibility and stack-unit rules are simulator inferences; no item-catalog data, sell transaction, full repair flow, or native junk behavior is claimed. Independent verification reuses the hash-matched 11/11 target, passes formatting/default offline checking and changed-Rust readability, and authenticates the frozen binary, no-addons control, and exact generic-TOC BagMeter replay. See [merchant repair capability](../../specs/merchant-repair-capability.md), [merchant buyback reads](../../specs/merchant-buyback-reads.md), and [merchant junk count](../../specs/merchant-junk-count.md).

## DinoUnitFrames curve producer follow-up

The unchanged cached DinoUnitFrames `8936218` startup failed in `modules/basecombopoints.lua` because `UnitPowerPercent("player", 4, true, curve)` returned a number and the addon called `color:GetRGBA()`. Forever’s generated `UnitDocumentation.lua` documents an optional `LuaCurveObjectBase` and an evaluated result; the simulator’s existing typed evaluator already returns scalar or color results. The defect was profile gating: Forever skipped optional curve evaluation entirely.

`17abf7071` adds focused Forever regressions; `d23cfe65c` evaluates supplied health and power curves on Forever. Curve input is normalized `current / max` only for Forever. This is an explicit inference, grounded in DinoUnitFrames’ unchanged `id / 5` color-curve thresholds and cached Blizzard `CurveConstants.ScaleTo100` mapping `[0, 1]` to `[0, 100]`, not a native-runtime proof. Omitted/nil curves still return the existing `0..100` numeric percentage, Retail retains its existing `0..100` curve-input policy, and other profiles are unchanged.

Independent verification at `d23cfe65c` reuses the fresh grouped 8/8 target (six Forever curve cases plus two numeric regressions), and passes formatting plus default offline checking. Frozen `wow-sim-d23cfe65` (SHA-256 `f7319afdfc06df39f9224d09e0e72111295202653ae9ae2e3726f75a5fca6b8c`) clean-starts unchanged archive `8936218` in isolated data; its optional options package remains unloaded on demand. A no-addons control returns `[]`. The unchanged addon's GUI workflow drives modeled combo power `0 → 2 → 5 → 1 → 0`, observing the corresponding segment alpha transitions and `DONE` before timeout 124 with no Lua errors and unchanged host CVars.

This is injected modeled-power/event evidence, not native gameplay production, secret semantics, rendered-pixel proof, native scale/security semantics, or whole-inventory compatibility.

## ClassicCastBar empty-CVar producer follow-up

Cached ClassicCastBarForever `8909724` calls `C_CVar.RegisterCVar(name, "")` only after an unknown read, then treats an empty read as unavailable so its authored `scale = 1` default survives. Before `d1e2487f6`, simulator registration filtered the explicit empty string into an omitted default; storage then substituted `"0"`. The unchanged addon read that value, converted it with `tonumber`, and passed `0` to the correctly strict `PlayerCastingBarFrame:SetScale`, causing startup failure.

`2530fcf56` supplies the targeted regression boundary; `d1e2487f6` preserves explicit empty defaults while retaining existing omitted-default `"0"` behavior. Independent verification reuses the matching grouped 15/15 target and passes formatting plus default offline checking: `/tmp/forever-addon-audit/verify-empty-cvar-ledger.json`. Frozen `wow-sim-d1e2487f` clean-starts unchanged archive `8909724` in isolated data; the matching no-addons control returns `[]`.

No addon code, scale validation, or generic fallback changed. The empty-string contract is inferred from cached API signatures and Wowless behavior, not a Forever-client probe. The separate Slider callback correction now has independent verification and a bounded unchanged-ClassicCastBar replay; it neither invalidates the CVar startup proof nor establishes mouse interaction, settings navigation, restart persistence, or inventory acceptance.

`19c11e551` additionally accepts finite Lua numeric registration defaults through both CVar surfaces, retaining the existing first-default and override behavior while rejecting non-finite and unsupported values. This follows unchanged Datamine `8936714` registering `debugTargetInfo` with numeric `0`; it is an inferred conversion policy, not native evidence. Development RED recorded two expected failures and GREEN records 18/18 grouped tests: `/tmp/forever-addon-audit/register-cvar-numeric-{red,green}/ledger.json`. Final independent verification and an unchanged Datamine replay remain pending. Datamine's separate missing global `ConsoleGetAllCommands` correction is in progress, so this does not change startup status, interaction coverage, or the matrix.

## Slider value callback follow-up

The unchanged ClassicCastBar settings slider installed `OnValueChanged`, but `SetValue(1.35)` previously changed only the simulator slider field: addon database scale, cast-bar scale, and persisted CVar stayed `1`. The defect was simulator-side: the Slider arm stored a changed clamped value then stopped at an explicit dispatch TODO.

`20318baf7` adds focused regressions and `3d6017fe3` synchronously dispatches existing pre/normal/post `OnValueChanged` bindings after releasing widget-state borrowing. Handlers receive the frame, clamped value, and documented `treatAsMouseEvent` boolean; same clamped values remain suppressed. Handler failures use the established error route and do not stop later bindings. StatusBars continue through their existing value path.

This is a bounded callback binding model, not a new mouse-drag producer or native timing/security claim. Independent proof reuses 9/9 focused slider tests, passes formatting/default offline checking, and includes a direct StatusBar regression: `/tmp/forever-addon-audit/verify-slider-value-ledger.json`. Frozen `wow-sim-3d6017fe` runs unchanged ClassicCastBar `8909724` through scale `1 → 1.35 → 1`, icon toggles, reset and `DONE` with no Lua errors; the matching no-addons control returns `[]`: `/tmp/forever-addon-runtime/classic-slider-workflow-q675457h/ledger.json` and `/tmp/forever-addon-runtime/slider-control-0uwemvbq/ledger.json`. Mouse dragging, native timing/security, rendered pixels, settings navigation and restart persistence remain unproven; the inventory stays open.

## Automatic duration-binding follow-up

The September 22, 2026 ActionBarAuras interaction replay disproved a startup-only explanation: its player container, matching aura candidate, assigned AuraButton, and visibility state all existed after `A_Admin.AddBuff(19750, ...)`, yet the button's duration text remained nil. Manual binding coverage therefore did not establish automatic countdown behavior.

`bf073db38` plus `d6a3859e4` add engine-tick scheduling for enabled duration text bindings. At frozen build `748e3668`, a fresh isolated ActionBarAuras replay now observes `7s` → `6s`, then a hidden aura button after `RemoveBuff`, with the probe completion marker, no collected Lua errors, and an unchanged host CVar hash. Its timeout `124` follows completion. Evidence: `/tmp/forever-addon-runtime/aba-duration-automatic-4rgzhxxk/{ledger.json,stdout}`.

This is a bounded helpful-player-buff path, not full ActionBarAuras or native conformance. Target debuffs, colors/rendering, timing parity, and final scheduler formatter-error isolation/independent verification remain pending. See [duration text binding](../../specs/duration-text-binding.md).

## Cursor transfer correction

Cached EasyFishing packages identified an exact existing-API sequence for returning a fishing pole: `C_Container.PickupContainerItem(bag, slot)`, `PickupInventoryItem(MAINHAND)`, then `C_Container.PickupContainerItem(bag, slot)` when the cursor still holds the displaced weapon. Commit `215a4080a` moves namespaced and legacy bag pickup through one simulator-side transfer model, removing the namespace no-op and reusing the existing auto-equip swap path. `5dbd06ec8` names the extracted helpers by their operations only. `d40397025` splits the touched finite-constant registration phases and registers `C_Container.PickupContainerItem` through its existing `c_container` owner rather than global inventory registration; no transfer contract changed.

The four new exact-sequence regressions were RED before production changes; the focused `inventory_verbs` module passed 25/25 in development afterward. This proves the bounded ID/count transfer behavior. It does not load EasyFishing, establish native inventory conformance, preserve hyperlinks/enchants/gems, implement stack merging or eligibility, model bank state, or add security/combat/event behavior. Independent final verification at `d40397025` passes; see the [shared verification record](../../forever-addon-comparison.md#independent-verification).

See the [cursor transfer contract](../../specs/cursor-item-transfer.md) for pinned archive identities, authored Camelot consumer evidence, and representation limits.

## BagIndex correction

BetterBags commit `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` walks consecutive `Enum.BagIndex.CharacterBankTab_N` and `AccountBankTab_N` members. Pinned `BagIndexConstantsDocumentation.lua` confirms Forever’s character IDs `6..14`, account IDs `15..23`, and `BagIndexMeta {-3, 23, 27}`; shared publication instead left account IDs at Retail’s earlier positions.

Commit `bb83a4c0a` publishes the corrected values and metadata only under `client-wowforever`. Its two exact consumer-loop enum-shape tests were RED before the producer change and GREEN after it. This proves enum names, values, boundaries, disjointness, and metadata only. It does not load BetterBags, model bank state, prove purchased tabs or account-bank availability, add Warbank behavior, or establish native conformance.

## CVarsBackup scoreboard producer gap

Unchanged CVarsBackup `8925285` remains a startup failure on frozen `963a3b791`: its save, restore, and reload button `OnUpdate` handlers call `IsInBG()`, whose `CVarsBackup.lua:150` invokes missing legacy global `GetNumBattlefieldScores()`. The isolated exact 15-file archive exits `1`; raw errors identify that call and host CVars remain unchanged: `/tmp/forever-addon-runtime/cvarsbackup-current-k5wyhwr5/ledger.json`.

The simulator has queue/active-battlefield state but no modeled scoreboard-row producer. That flag cannot establish a score count, so no zero-return shim or active-flag substitution is proposed. This remains an unresolved scoreboard producer gap, not a native contract claim or a matrix change.

## Datamine CVar and console startup follow-up

Datamine `8936714` previously failed at a local alias of missing global `ConsoleGetAllCommands` and at numeric `C_CVar.RegisterCVar("debugTargetInfo", 0)`. `19c11e551` accepts finite numeric registration defaults while retaining first-default/override and invalid-type behavior; `2c78bff73` derives fresh CVar-only command records from supported storage for both global and `C_Console` calls. RED/GREEN proof is numeric 16/18→18/18 and console 0/2→2/2; the independent audit also passes format/default checking and Rust readability: `/tmp/forever-addon-audit/verify-datamine-catalog-ledger.json`.

The earlier `e511053b2` clean-startup classification is invalid. It observed 863 byte-identical members, a fully loaded root, zero warnings and `[]`, but diagnostic load trace records recovery from `Failed to create frame __Datamine_45253: Missing FileName or FilePath`; `src/loader/xml_frame/setup.rs` then silently discarded that recovered deferred child. The `[]` result is therefore a false-negative observation, not startup health: `/tmp/forever-addon-runtime/datamine-recovery-trace-2do7026r/stderr`.

The immediate producer is now identified: `resolve_texture_inheritance` copied texture visual/mixin fields but omitted instance `KeyValues`. Datamine's inherited custom-atlas control texture therefore reached `ApplyAtlas()` without `Icon.FileName`, raised the reported error during nested workspace construction, and prevented later initialization. `dd52ef974` preserves inherited plus instance KeyValues; `3f2cf7c26` propagates partial XML creation errors instead of treating them as healthy loading. The valid RED had two new failures and two existing lifecycle controls passing, while GREEN passes all four: `/tmp/forever-addon-audit/datamine-texture-error-{red,green}-*/ledger.json`. The texture specification's three test-backed requirements are checked.

The clean `f095f24c6` replay supersedes that warning-only observation. The exact 863-member archive exits `1`; `lua-errors` now contains the nested XML `CreateFrame` error from `MovieFrame.lua:270` calling missing `EnableSubtitles`, plus the Blizzard ScriptErrors mirror. The collector also retains the addon warning, and host CVars are unchanged: `/tmp/forever-addon-runtime/datamine-collected-error-mzafz9ai/ledger.json`; build provenance is `/tmp/forever-addon-audit/xml-collector-clean-build-2e1thpb8/ledger.json`. The narrow `xml_create_error_reporting` GREEN proves a temporary nested failure reaches both one load warning and one canonical collector entry; it does not prove subtitle behavior or a healthy startup. The normal `ChatFrame1EditBox:SendText("/dm ui")` replay remains failed before opening at nil `DatamineUnifiedFrame.ShowAnim`, with no `DONE`: `/tmp/forever-addon-runtime/datamine-keyvalues-ui-zcycbmnp/ledger.json`. Earlier open/close/DONE evidence belongs to a prior source and is not acceptance proof. Startup and workflow counts remain failed. No MovieFrame implementation, 3D/model, rendered-pixel, settings, persistence, audio, native-client, or full-catalog claim follows. See [texture KeyValue inheritance](../../specs/texture-keyvalue-inheritance.md), [XML creation error reporting](../../specs/xml-create-error-reporting.md), [console catalog](../../specs/console-command-catalog.md), [CVar registration](../../specs/cvar-registration.md), and [runtime coverage](../../forever-addon-runtime-coverage.md).

## Cached RestrictedExecution state and rejection coverage

Tests-only `6e2f65346` exercises the cached Forever `Blizzard_RestrictedAddOnEnvironment` through the normal cached-addon harness and public `SecureHandlerExecute` path. Two protected headers retain separate working state: the first records `10 → 11 → 12`, while the second records `20` without changing the first. Public `CallRestrictedClosure` rejects `return {}` with `Direct table creation is not permitted`; the subsequent first-header increment proves execution state is released after rejection. The test also requires an empty collected Lua-error list. No Blizzard patch, compiler shim, or initialization bypass is used.

Source proof is **1/1 passed**, exit `0`, with unchanged before/after hashes for the test and Cargo manifests: `/home/osso/.local/state/wow-ui-sim-proof/forever-restricted-execution-2026-09-30/{ledger.json,stdout,stderr}`. Independent audit `20328` remains pending. This bounded available-consumer test does not reproduce the historical DisenchantHelper nil-`loadstring_untainted` failure, but the archived addon is unavailable: no exact-addon replay, matrix status change, native security parity, or full restricted-execution claim follows. Datamine remains blocked at its separately documented startup/workflow boundaries; that blocked case does not close or pause the broader inventory goal.

## BugCapture selected-TOC source boundary

The first current-local BugCapture test failed before its workflow: `!BugGrabber did not load any Lua files`. Persistent `trace.stderr` rejects both `BugGrabber.lua` and `locales.lua`: relative `Interface/AddOns/!BugGrabber/...` module paths are outside the absolute selected TOC root. The trace's `loaded !BugGrabber` bookkeeping marker does not prove Lua execution. An earlier explorer assertion that it was already loaded lacked observation and is rejected by these warnings.

Pre-fix `resolve_addon_file_path` chose a repo-relative overlay whenever that file existed, overriding the selected TOC source. `dd77368b7` removes that implicit alternate-source choice and passes `toc.file_paths()` directly to loading; the selected-root module guard remains intact. This honors the user's NoFallbacks constraint without vendor edits. Post-fix bounded GREEN and independent final acceptance are recorded below.

| Boundary | Observed proof | Remaining limit |
| --- | --- | --- |
| Cached RestrictedExecution, updated rilua pin | Build ledger revision `3e40b526b`; combined test ledger: 2 tests, 1 baseline pass / 1 loading failure, exit `101` | Improves bounded producer proof only; independent/full security acceptance unclaimed |
| Current-local BugGrabber/BugSack | 29-file before/after identity unchanged; first test fails loading, diagnostic trace confirms root mismatch | Not an archive-release identity; subsequent bounded workflow GREEN recorded below |
| Selected-TOC correction `dd77368b7` | Source diff removes existence-based substitution | Independent bounded workflow/final checks pass below |

No archival matrix status changes follow. Historical DisenchantHelper remains unavailable; Datamine's separate blockers remain open. Evidence directory: `/home/osso/.local/state/wow-ui-sim-proof/forever-bug-capture-2026-10-01/` (`build-ledger.json`, `test-ledger.json`, `tests.stdout`, `trace.stderr`, and local addon before/after hashes).

## Current-local messaging and error capture — observed GREEN

Main-owned artifacts in `/home/osso/.local/state/wow-ui-sim-proof/addon-messages-2026-10-01/` record unchanged build source `255adcbde`, build exit 0 and run **36/36 passed**, exit 0: 15 messaging cases including one legacy-global contract, seven additional legacy controls, twelve module cases, one cached RestrictedExecution case and one current-local BugGrabber/BugSack workflow. The selected-TOC correction now has bounded workflow GREEN, superseding the earlier loading failure, not erasing it.

The real local TOCs and embedded libraries initialize through normal ADDON_LOADED/PLAYER_LOGIN dispatch. Two normal OnUpdate error dispatches store one error, increment its counter `1 → 2`, keep broker count `1`, switch broker icon to red, and update the actual minimap texture property. The intentional marker appears twice in stderr (second occurrence suppression reported); the deliberate failed-module control also emits its expected diagnostic. Independent inspection confirms one expected `Broken/Bad.lua` module-failure diagnostic and two intentional `FOREVER_BUG_CAPTURE_MARKER` occurrences, with zero unexpected BugSack errors. This is not zero-diagnostics or rendered-pixel proof.

`green-ledger.json` retains exact before/after hashes for all **29 available local files**, unchanged; binary SHA-256 `6aa6c19ea007c13bad1e02b4425f6169ea3088c06d34791d03c5c4bc62202271` and shared host CVar hash are unchanged. These local sources are **not the historical BugSack zip** and establish no new archive identity or credit against the 269-project inventory. No archive matrix counters change.

Earlier BugSack `MissingRequirements` reflects nil-symbol-access diagnostics, not `RequiresAPI` TOC metadata. Runtime fallback synthesizes nil-returning callables; RED's thirteen behavior assertions failed after function-type checks passed. See [outbound contract](../../specs/addon-messages.md) for the policy and proof limits. `verify-artifacts.json` is source-audit evidence predating main GREEN. `verify-green.json` independently confirms bounded 36/36 exit 0, one default `cargo fmt --check` and one default `cargo check --offline`, each exit 0 with zero warnings at `255adcbde`. Docs-only `54c6bfaf2` preserves all 3252 recorded build inputs. Current reconciliation rechecks those inputs, all 29 local vendor hashes, host CVar hash and retained artifact hashes without mismatches; Cargo manifest/lock and rilua pin `e47c1fedc59dfa3859bdc522ea454792112d8316` remain unchanged. Relevant cached vendor hashes provide current source attribution only: no whole-cache before/after snapshot exists, so full-cache preservation is not claimed. The source audit's dense numeric predicate and six positional record arguments are two nonblocking readability findings, not correctness blockers. AutoRoll tests require separate proof/goal extension. Binary stringView, logged nil outcomes, native parity, network delivery and rendered pixels remain unsupported or unverified.

## Sources

- [Selected-TOC loader](../../../src/loader/addon.rs) — `dd77368b7` removes implicit local overlays, preserves module-root guard
- `/home/osso/.local/state/wow-ui-sim-proof/forever-bug-capture-2026-10-01/test-ledger.json` and sibling build/trace/identity artifacts — pre-fix baseline pass and BugCapture loading failure, not post-fix acceptance
- [RestrictedExecution regression](../../../tests/forever_restricted_execution.rs) — tests-only `6e2f65346`, cached consumer state and public rejection boundary
- `/home/osso/.local/state/wow-ui-sim-proof/forever-restricted-execution-2026-09-30/ledger.json` and sibling `stdout`/`stderr` — bounded 1/1 source proof; independent audit pending
- [Forever runtime coverage](../../forever-addon-runtime-coverage.md) — historical DisenchantHelper classification remains unchanged without its archive
- [Forever comparison audit](../../forever-addon-comparison.md) — catalog provenance, scope, and incomplete comparison matrix
- [Forever finite constants spec](../../specs/forever-finite-constants.md) — BagIndex and finite-event boundaries
- [Public base-spell lookup](../../specs/spell-base.md) — ActionBarAuras failure boundary and model limits
- [Forever expansion identity](../../specs/forever-expansion-identity.md) — Angleur consumer basis and inferred policy limit
- [UnitPowerPercent curves](../../specs/unit-power-percent-curves.md) — profile-specific curve input and result contract
- [UnitHealthPercent curves](../../specs/unit-health-percent-curves.md) — matching Forever health-curve boundary
- [CVar registration](../../specs/cvar-registration.md) — explicit-empty, numeric and omitted registration boundaries
- [Console command catalog](../../specs/console-command-catalog.md) — modeled CVar-only console records and limits
- [Slider value callbacks](../../specs/slider-value-callback.md) — changed-value script delivery scope and bounded proof
- [Cursor transfer spec](../../specs/cursor-item-transfer.md) — cached EasyFishing transfer contract and limits
- [Forever running report](../../wowforever-1.60.1.md) — profile-wide committed behavior and proof boundaries
- `Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua` in the pinned Forever 1.60.1.69913 cache — authoritative enum values
- BetterBags `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` — motivating consumer loop
- `/tmp/forever-bag-index-development-ledger.json` — RED/GREEN command and revision evidence
- `data/forever-addon-audit/deep-dispositions.json` — offline Carbonite, EpicDamageMeter, and parked-runtime dispositions
- `/tmp/forever-addon-runtime/cvarsbackup-current-k5wyhwr5/ledger.json` — frozen CVarsBackup `GetNumBattlefieldScores` failure
- `/tmp/forever-addon-audit/verify-datamine-catalog-ledger.json` — independent Datamine CVar/catalog proof
- `/tmp/forever-addon-runtime/datamine-catalog-startup-b4hktcva/ledger.json` — invalidated false-negative `[]` startup observation
- `/tmp/forever-addon-runtime/datamine-recovery-trace-2do7026r/stderr` — recovered/discarded Datamine frame diagnostic
- `/tmp/forever-addon-audit/datamine-texture-error-red-n7c75m2n/ledger.json` — valid texture-KeyValue and partial-create-reporting RED
- `/tmp/forever-addon-audit/datamine-texture-error-green-4jqizar3/ledger.json` — four focused texture/error-reporting tests pass
- `/tmp/forever-addon-audit/verify-datamine-keyvalues-ledger.json` — independent focused/fmt/default-check/readability proof after diagnostic source restoration
- `/tmp/forever-addon-runtime/datamine-keyvalues-startup-8oof2qa6/ledger.json` — superseded warning-only Datamine observation
- `/tmp/forever-addon-audit/xml-error-collector-green-ftxo0ffp/ledger.json` — focused canonical XML-error collection proof
- `/tmp/forever-addon-audit/xml-collector-clean-build-2e1thpb8/ledger.json` — clean `f095f24c6` build provenance
- `/tmp/forever-addon-runtime/datamine-collected-error-mzafz9ai/ledger.json` — exact Datamine collected `EnableSubtitles` failure
- `/tmp/forever-addon-runtime/datamine-warning-trace-cvydydyd/stderr` — diagnostic source of remaining `C_XMLUtil.GetTemplateInfo` nil result
- `/tmp/forever-addon-runtime/datamine-keyvalues-ui-zcycbmnp/ledger.json` — current failed chat-dispatched UI attempt
- `/tmp/forever-addon-audit/verify-datamine-ui-failure-ledger.json` — independent failure classification and provenance audit
- [Forever running report](../../wowforever-1.60.1.md) — current cached-consumer boundaries and final-gate status

## See Also

- [[forever-clean-startup]] — distinct sustained Blizzard-runtime proof
- [[client-profiles]] — Camelot/Forever profile selection and inferred expansion identity
- [[lua-api]] — bounded `C_Spell.GetBaseSpell` model
- [[event-system]] — finite Forever event registration
- [Forever UI/API delta notes](../../wowforever-1.60.1-ui-api-deltas.md) — separately published native-POV report (`362b65c7f`, branch `forever-ui-api-report`)
