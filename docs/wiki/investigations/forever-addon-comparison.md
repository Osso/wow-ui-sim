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

## DinoUnitFrames curve producer follow-up

The unchanged cached DinoUnitFrames `8936218` startup failed in `modules/basecombopoints.lua` because `UnitPowerPercent("player", 4, true, curve)` returned a number and the addon called `color:GetRGBA()`. Forever’s generated `UnitDocumentation.lua` documents an optional `LuaCurveObjectBase` and an evaluated result; the simulator’s existing typed evaluator already returns scalar or color results. The defect was profile gating: Forever skipped optional curve evaluation entirely.

`17abf7071` adds focused Forever regressions; `d23cfe65c` evaluates supplied health and power curves on Forever. Curve input is normalized `current / max` only for Forever. This is an explicit inference, grounded in DinoUnitFrames’ unchanged `id / 5` color-curve thresholds and cached Blizzard `CurveConstants.ScaleTo100` mapping `[0, 1]` to `[0, 100]`, not a native-runtime proof. Omitted/nil curves still return the existing `0..100` numeric percentage, Retail retains its existing `0..100` curve-input policy, and other profiles are unchanged.

Independent verification at `d23cfe65c` reuses the fresh grouped 8/8 target (six Forever curve cases plus two numeric regressions), and passes formatting plus default offline checking. Frozen `wow-sim-d23cfe65` (SHA-256 `f7319afdfc06df39f9224d09e0e72111295202653ae9ae2e3726f75a5fca6b8c`) clean-starts unchanged archive `8936218` in isolated data; its optional options package remains unloaded on demand. A no-addons control returns `[]`. The unchanged addon's GUI workflow drives modeled combo power `0 → 2 → 5 → 1 → 0`, observing the corresponding segment alpha transitions and `DONE` before timeout 124 with no Lua errors and unchanged host CVars.

This is injected modeled-power/event evidence, not native gameplay production, secret semantics, rendered-pixel proof, native scale/security semantics, or whole-inventory compatibility.

## ClassicCastBar empty-CVar producer follow-up

Cached ClassicCastBarForever `8909724` calls `C_CVar.RegisterCVar(name, "")` only after an unknown read, then treats an empty read as unavailable so its authored `scale = 1` default survives. Before `d1e2487f6`, simulator registration filtered the explicit empty string into an omitted default; storage then substituted `"0"`. The unchanged addon read that value, converted it with `tonumber`, and passed `0` to the correctly strict `PlayerCastingBarFrame:SetScale`, causing startup failure.

`2530fcf56` supplies the targeted regression boundary; `d1e2487f6` preserves explicit empty defaults while retaining existing omitted-default `"0"` behavior. Independent verification reuses the matching grouped 15/15 target and passes formatting plus default offline checking: `/tmp/forever-addon-audit/verify-empty-cvar-ledger.json`. Frozen `wow-sim-d1e2487f` clean-starts unchanged archive `8909724` in isolated data; the matching no-addons control returns `[]`.

No addon code, scale validation, or generic fallback changed. The empty-string contract is inferred from cached API signatures and Wowless behavior, not a Forever-client probe. The separate settings workflow had failed at the simulator's missing Slider `SetValue` → `OnValueChanged` dispatch; its committed callback implementation remains pending GREEN verification and replay. It neither invalidates the CVar startup proof nor establishes settings, persistence, or inventory acceptance.

## Slider value callback follow-up

The unchanged ClassicCastBar settings slider installed `OnValueChanged`, but `SetValue(1.35)` previously changed only the simulator slider field: addon database scale, cast-bar scale, and persisted CVar stayed `1`. The defect was simulator-side: the Slider arm stored a changed clamped value then stopped at an explicit dispatch TODO.

`20318baf7` adds focused regressions and `3d6017fe3` synchronously dispatches existing pre/normal/post `OnValueChanged` bindings after releasing widget-state borrowing. Handlers receive the frame, clamped value, and documented `treatAsMouseEvent` boolean; same clamped values remain suppressed. Handler failures use the established error route and do not stop later bindings. StatusBars continue through their existing value path.

This is a bounded callback binding model, not a new mouse-drag producer or native timing/security claim. Development GREEN, independent verification, and unchanged ClassicCastBar scale/icon/reset workflow replay remain pending.

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

## Sources

- [Forever comparison audit](../../forever-addon-comparison.md) — catalog provenance, scope, and incomplete comparison matrix
- [Forever finite constants spec](../../specs/forever-finite-constants.md) — BagIndex and finite-event boundaries
- [Public base-spell lookup](../../specs/spell-base.md) — ActionBarAuras failure boundary and model limits
- [Forever expansion identity](../../specs/forever-expansion-identity.md) — Angleur consumer basis and inferred policy limit
- [UnitPowerPercent curves](../../specs/unit-power-percent-curves.md) — profile-specific curve input and result contract
- [UnitHealthPercent curves](../../specs/unit-health-percent-curves.md) — matching Forever health-curve boundary
- [CVar registration](../../specs/cvar-registration.md) — explicit-empty versus omitted registration boundary
- [Slider value callbacks](../../specs/slider-value-callback.md) — changed-value script delivery scope and pending proof
- [Cursor transfer spec](../../specs/cursor-item-transfer.md) — cached EasyFishing transfer contract and limits
- [Forever running report](../../wowforever-1.60.1.md) — profile-wide committed behavior and proof boundaries
- `Blizzard_APIDocumentationGenerated/BagIndexConstantsDocumentation.lua` in the pinned Forever 1.60.1.69913 cache — authoritative enum values
- BetterBags `411a6f6ee1ea40eca8ac96927ccdd49a6aab3941` — motivating consumer loop
- `/tmp/forever-bag-index-development-ledger.json` — RED/GREEN command and revision evidence
- `data/forever-addon-audit/deep-dispositions.json` — offline Carbonite, EpicDamageMeter, and parked-runtime dispositions
- [Forever running report](../../wowforever-1.60.1.md) — current cached-consumer boundaries and final-gate status

## See Also

- [[forever-clean-startup]] — distinct sustained Blizzard-runtime proof
- [[client-profiles]] — Camelot/Forever profile selection and inferred expansion identity
- [[lua-api]] — bounded `C_Spell.GetBaseSpell` model
- [[event-system]] — finite Forever event registration
- [Forever UI/API delta notes](../../wowforever-1.60.1-ui-api-deltas.md) — separately published native-POV report (`362b65c7f`, branch `forever-ui-api-report`)
