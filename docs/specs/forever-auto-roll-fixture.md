# Forever AutoRoll consumed-path fixture

`tests/forever_auto_roll.rs` tests the current local AutoRoll TOC against the unchanged complete cached Settings dependency closure. User-approved scope: **“Test only consumed AutoRoll paths.”** This is not whole-closure cleanliness, archive identity or native parity. [Forever investigation](../wiki/investigations/forever-addon-comparison.md#mapped-stick-and-autoroll--current-pin-boundary) records historical runtime evidence; [gamepad contract](gamepad-mapped-state.md) owns mapped/hover behavior separately.

## What it must do

### Loading and diagnostic boundaries

- [ ] Load the real complete Game-screen `Blizzard_Settings_Shared` closure and local `AutoRoll.toc` normally, without overrides, narrowed dependencies, preseeded AutoRoll state or callback bypasses. Configure raid world state before local TOC/lifecycle, preserving existing ordering.
- [ ] Retain and print separately exactly these cached setup requirements, matching kind, addon, Public environment, source basename and diagnostic line: SharedXML `CNamespace C_Reveal`, `DebugBarManager.lua:106`; StaticPopup_Game `CMethod C_Club.GetInvitationCandidates`, `GameDialogDefs.lua:1371`; StaticPopup_Game `CMethod C_GameRules.IsHardcoreActive`, `GameDialogDefs.lua:3412`. All other cached/local requirements and all warnings/errors remain fatal. Never clear or mask diagnostic history.
- [ ] Reject all Reveal method manufacture throughout full history and all callable entries in its raw namespace table. The existing truthy namespace changes debug setup; it is neither native absence nor an innocent nil probe.
- [ ] Compare the entire C-requirement access history before/after local loading, lifecycle/Settings publication/query, world query and produced loot decisions. No new requirement is tolerated on consumed paths, even an otherwise allowed setup requirement.

### Execution isolation and observable outcomes

- [ ] Install a supported VM main-thread call observer without replacing any API, callback or existing hook. Capture already-cached club/hardcore function identities by raw lookup. Calibrate actual club autocomplete-reference and hardcore calls, plus Lua/native witnesses and caller-taint preservation, before world/local TOC/lifecycle. Calibration must return existing nil results without creating AutoRoll state, rolls, choices or log output; retain calibration counts/history.
- [ ] At consumed-path checkpoints require the observer still installed, live Lua/native witnesses, unchanged function identities, invite autocomplete reference and hardcore-death absence, and zero additional club/hardcore calls after calibration. The hook itself must never invoke monitored functions. No construction-shape assertions substitute for outcomes.
- [ ] Real `ADDON_LOADED('AutoRoll')` initializes defaults, publishes Settings values and enables real event handling. The complete TOC publishes its slash command; slash invocation and Settings UI interaction are not claimed by these four cases.
- [ ] Real `A_Admin.StartLootRoll` synchronously produces `START_LOOT_ROLL`; unchanged addon handling calls real `RollOnLoot`. Enabled Vault chooses Greed, records model/API choice, emits the addon item/Greed log and removes the roll. Unsupported raid and default-disabled Nerub-ar make no choice/log and retain the full produced roll unchanged.
- [ ] Separate environments isolate database, event registration, choices, rolls and logs; guard diagnostic history and observer independently in each environment. Preserve first-environment mutations while the second initializes and consumes a roll.

## How it works

- [Current-pin source and runtime boundary](../wiki/investigations/forever-addon-comparison.md#mapped-stick-and-autoroll--current-pin-boundary)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `tests/forever_auto_roll.rs` — four grouped integration cases, exact retained diagnostics, full-history guards and calibrated identity-based VM observer.
- `Interface/AddOns/AutoRoll/{AutoRoll.lua,OptionPanel.lua,SlashCommand.lua,AutoRoll.toc}` — unchanged real local sources; decision handler, Settings registration and slash publication.

## Tests asserting this spec

Four existing `forever_auto_roll_*` cases in `tests/forever_auto_roll.rs`; no additional Cargo target.

Existing RED: `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/hover-green-ledger.json`, `hover-green.stdout`, `hover-green.stderr`, source-scoped `a956dfdd3`, rilua `6044544b`: 71/75 pass, four AutoRoll cases fail at the pre-local-addon StaticPopup requirement gate; all eleven mapped/hover and sixty prior controls pass. This does not prove AutoRoll outcomes. Baseline not rerun.

Revised fixture and observer calibration are **unexecuted**. Implementation phase formatted and committed; parent-owned GREEN and independent verification follow. All criteria remain unchecked pending GREEN. Historical default gates do not cover the changed fixture.

## Known gaps (current cycle)

- [ ] Execute revised four-case fixture and calibration before awarding any AutoRoll decision credit. Unsupported observer, identity resolution or taint preservation must fail explicitly, not disable observation or weaken assertions.
- [ ] Observer covers synchronous main-thread calls while installed, not other coroutines, callbacks run with hooks suppressed or deferred timer/UI execution. Nil-access logging alone records first lookup/manufacture and cannot detect later calls through cached functions.

Source/path classification: Forever `[Family]` selects Mainline. Physical `Mainline/GameDialogDefs.lua:1387` captures club autocomplete; :1357 calls it only inside `InviteToClub`. Diagnostic :1371 is loader attribution, not that physical line. Base :3412 eagerly gates hardcore popup definitions. Local OptionPanel registers Settings, AutoRoll handles lifecycle/loot and SlashCommand publishes its command; none directly invokes either query. Their nonparticipation in consumed outcomes remains a source-grounded inference until calibrated observation and behavioral assertions execute. Setup itself remains altered by existing nil-returning functions; no full-closure innocence claim.

## Out of scope

Club/hardcore models, Reveal/native semantics, full Settings closure cleanliness, full UI/Settings callbacks/slash invocation, timer-driven loot-history closure, public raid entry, archive/inventory credit, production/vendor/API/Cargo changes and whole-addon acceptance. Retire exact setup exceptions when their missing models/classification are corrected; never broaden them to namespace exemptions.
