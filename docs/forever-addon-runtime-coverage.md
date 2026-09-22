# Downloaded Forever addon runtime coverage

Overall compatibility remains **unverified**. The [per-project matrix](../data/forever-addon-audit/runtime-coverage.json) covers all 269 projects in the [cached archive inventory](../data/forever-addon-audit/cached-comparisons.json), not just previously selected candidates. The earlier [comparison audit](forever-addon-comparison.md) records static evidence; it is not runtime acceptance.

## Inventory — 2026-09-22

All 268 successfully downloaded Forever packages exist and match their indexed sizes and SHA-256 hashes. ConsumableTracker's selected Forever file `8924598` is unavailable; its older comparison archive is not silently substituted. Missing cached dependencies remain separate blockers. No further downloads are authorized by this audit.

The matrix separates current startup and interaction results from `priorEvidence`. A `not-run` current result does not erase earlier scoped proof: it means that proof has not yet been reconciled or rerun for this inventory-wide pass. Current bounded batches include Abattis PugBoard open/close, Abgesattelt named remote-death count updates, one intentional LoadOnDemand-not-requested module, one profile-excluded package, and ActionBarAuras duration create/countdown/removal evidence. Ellesmere's existing six-group proof is retained. EpicDamageMeter also has historical bounded interaction proof; CooldownMaster and DragonGuildMaster have startup-only evidence. Carbonite and Baganator have recorded failures or dependency blockers.

## Baseline isolation correction — 2026-09-22

All 268 available packages were attempted in package-only batches. Those runs omitted `XDG_DATA_HOME`: `--no-saved-vars` and isolated WTF paths do not isolate simulator CVar overrides. Addons wrote to shared `/home/osso/.local/share/wow-sim/cvars.json`; enabling combo-point UI exposed missing `GetComboPoints`, after which unrelated packages repeated the native error. A no-addons shared-data control fails; the same control with fresh `XDG_DATA_HOME` returns `[]`.

The polluted baseline remains historical evidence only. The corrected harness uses per-run data storage and explicit cached-provider composition. Cross-process tests prove writer persistence, independent reader defaults, and an unchanged shared host CVar hash. The post-batch shared CVar file is snapshotted, not reset: no exact pre-batch snapshot exists, so original values cannot be claimed restored.

## Isolated startup checkpoint

The initial isolated checkpoint at binary build `c1e830ffa` recorded 161 clean startups, 32 runtime failures, 72 unloaded/partial loads, and three unresolved dependency cases. The unavailable archive is the 269th entry. Every isolated batch preserved the shared host CVar hash. These are startup observations, not full compatibility acceptance; 14 of the 72 unloaded cases contain some successfully loaded roots, and intentionally excluded or deferred modules must not be mistaken for failures.

`GetComboPoints` and whitespace enum/configuration regressions pass 6/6 and 1/1 respectively; format/default compile checks pass. Classic UI Forever now starts without its prior combo-point failure. Focused proof through `ebff90517` passes native formatter 7/7, configuration 7/7, binding 11/11, numeric 8/8, public base-spell 3/3, finite constants/events 10/10, input style 4/4, named duplicates 5/5, legacy identity 1/1, macro verbs 14/14, and placeholder migration 21/21. `cargo fmt --check` and default `cargo check --offline` pass. ActionBarAuras clean-starts under `ebff90517`. At frozen scheduler build `748e3668`, its real player-buff duration workflow observes `7s` then `6s`, removes the aura button, and completes with zero Lua errors; scope remains bounded. Full per-addon major workflows remain mostly untested.

A separate DBM-GUI replay at `55ee20d7` observes metadata, dependency and LoD status before execution, then explicitly loads the module and observes its table/options frame with zero Lua errors. This is loader evidence only—not DBM encounter, timer, alert or rendered-GUI acceptance. The replay requires the explicit out-of-date interface switch; its ledger is `/tmp/forever-addon-runtime/dbm-lod-native-e4hvi488/ledger.json`.

## Explicit out-of-date replay of unloaded packages

Replayed exactly the 72 previously unloaded/partial projects with `WOW_SIM_LOAD_OUT_OF_DATE_ADDONS=1` and the `55ee20d7` binary, which includes deferred-addon metadata registration. Result: 48 clean startups, 12 runtime failures, 12 still unloaded. Per-run data/WTF isolation preserved the shared host CVar hash. Default-interface attempts remain in each project's `priorDefaultInterfaceAttempt`.

The historical explicit-interface checkpoint totals **209 clean startups, 44 failed, 12 unloaded, three dependency-blocked, one unavailable archive**. Later producer replays at `b8f0982be` and exact-binary `ebff90517` reached **216 clean startups, 37 failed, 12 unloaded, three dependency-blocked, one unavailable archive**. A frozen `2c5bf78c7` replay added DRaidFrames after its registered-template query. The later frozen `5a1cc851` replay re-used 24 valid prior outcomes and retried only 12 launcher failures caused by invalid `env` argument ordering; exit 127 was never counted as an addon failure. After the Dino curve and ClassicCastBar CVar replays, the current mixed-provenance matrix is **222 clean startups, 30 failed, 13 unloaded, three dependency-blocked, one unavailable archive**. These observations are not a single current-build certification. Historical batches: `/tmp/forever-addon-runtime/unloaded-ood-55ee20d7-batch0.json` and `batch1.json`; later replay evidence: `/tmp/forever-addon-runtime/failed-replay-5a1-ledger.json` and `/tmp/forever-addon-runtime/failed-replay-5a1-env-corrected-ledger.json`.

Ellesmere's expanded startup replay exposes a taint error in `AuraKit.RunJob`; an identical isolated replay with frozen `c1e830ffa` reproduces the same 56 occurrences. Thus the error is not introduced by the formatter/LoD commits. Prior six-workflow evidence remains scoped historical proof, not acceptance of this expanded configuration.

Current residuals include Buffalo's unavailable local spell record 9910 and unresolved downstream configuration data. The successful producer replays do not establish their major workflows. These are observed simulator gaps, not addon patches or exclusions.

## DRaidFrames registered-template startup

DRaidFrames `8922652` now starts cleanly after `DoesTemplateExist` was modeled as a query of the existing registered virtual-template registry. The predicate does not scan disk, force XML/addon loading, or treat arbitrary global frame names as templates. The unchanged package root loads with zero collected Lua errors at frozen `2c5bf78c7`: `/tmp/forever-addon-runtime/draidframes-template-gqfplb1k/ledger.json`. This is startup evidence only; DRaidFrames layout, aura display, and user workflows remain untested. The frozen `2c5bf78c7` predicate lifecycle target passes 3/3; independent final verification also confirms `cargo fmt --check`, default offline `cargo check`, registry/source/readability review, and immutable replay provenance. The test and replay use the recorded wow-sim SHA-256 `7d0ff153e1bf2149f670e09ba1eea253460bf58b6bb9d2f754c64f857f8ccfbf`; see `/tmp/forever-addon-audit/template-existence-green-emlklwik/{build-ledger.json,test-ledger.json}` and the replay ledger above.

## Cached LibStub composition and bounded library interactions

C_Everywhere `8912546`, CustomSearch `8912549`, and CustomTutorials `8912550` start with the unchanged, TOC-bearing LibStub subtree from cached DBM `8925922`. The explicit `WOW_SIM_LOAD_OUT_OF_DATE_ADDONS=1` switch is required by these observations; it does not waive other loader restrictions. Original dependency-only attempts remain in the matrix. Provider provenance: `/tmp/forever-addon-audit/cached-libstub-subtree-provenance.json`.

At the frozen `c1e830ffa` binary, isolated interaction probes observed:

| Package | Covered behavior | Still untested |
| --- | --- | --- |
| CustomSearch | 16 positive/negative accent, AND/OR/NOT, tag and numeric query checks | Consumer integration; native-client comparison |
| CustomTutorials | Progress 1→2→3, callback delivery, already-seen no-op, reset/hide, retrigger1 | Button clicks, rendered appearance, images/shine, restart persistence |
| C_Everywhere | CVar namespace/call parity; occupied item6948 stack3, five empty slots, removal | Legacy scalar packing; remaining namespaces |

Each probe returned its completion marker and zero recorded Lua errors; shared host CVars remained unchanged. Fixtures live in `tools/forever-addon-fixtures/`; exact commands, binary hashes, and outputs are linked per project. These add bounded evidence, not whole-addon certification. The matrix now records 14 bounded interaction passes, one failed workflow and 254 not-run entries; historical evidence remains separate.

## Abgesattelt bounded interaction

Simulated guild addon messages create a named player's count at three, reject a lower count, advance it to five, and retain a second player's count at two. Timestamps are recorded. Nine assertions and the completion marker pass with zero Lua errors and unchanged host CVars: `/tmp/forever-addon-runtime/abgesattelt-major-0gezxdzq/ledger.json`. This replaces the current initialization-only classification, not its preserved historical evidence. Local death detection, real transport/audio, synchronization negotiation, displayed rankings and restart persistence remain untested.

## Alias bounded interaction

Alias `8260596` opens its manager, creates an alias through its real Add button, expands a slash invocation into simulated chat output, removes its saved entry and dispatch registration through the Remove button, then closes its manager. Eight assertions pass with zero Lua errors and unchanged host CVars: `/tmp/forever-addon-runtime/alias-ui-contract-j1qikr18/ledger.json`.

An earlier probe incorrectly required normal `SlashCmdList` lookup to return nil after removal. Unchanged Blizzard `ChatFrameSetup.lua` and `ChatFrameUtil.lua:ImportListToHash` deliberately retain handlers in the table's `__index` cache. That assertion was invalid, not a simulator defect; the corrected probe checks removed command spelling/raw registration/dispatch hash. Native chat delivery and restart persistence remain untested.

## ActionBarAuras bounded duration interaction

The named-container lifecycle fix retains separate player and target AuraContainers. Frozen build `748e3668775274cf26facb6b5587237effb36935` observes the real player-container workflow after `A_Admin.AddBuff`: duration text changes from `7s` to `6s`, `RemoveBuff` hides the aura button, and the probe emits `DONE`. The 20-second process timeout (`124`) is teardown after completion, not a test failure; collected Lua errors are empty and the shared host CVar hash is unchanged. Evidence: `/tmp/forever-addon-runtime/aba-duration-automatic-4rgzhxxk/{ledger.json,stdout}`.

This covers only the modeled helpful player-buff create/countdown/removal path. Target debuffs, color behavior, rendered pixels, broader ActionBarAuras settings, and native timing parity remain open.

## CustomMinimapArrow bounded rotation interaction

Frozen `5a1cc851` supplies nullable player facing to the unchanged addon. In a 15-second GUI-style replay, its arrow rotates from `0.5` to `2.5` radians; after player facing becomes unknown, the arrow retains `2.5`. The probe emits `DONE`, records zero Lua errors, and leaves host CVars unchanged. Exit `124` is bounded teardown after completion: `/tmp/forever-addon-runtime/minimap-facing-gui-h7afvjws/{ledger.json,stdout}`.

This covers two modeled finite angles and unknown-state retention only. Default/native facing behavior, player movement, minimap instance rotation, visual pixels, and broader addon settings remain untested.

## AppelSwingsForever bounded swing interaction

AppelSwingsForever `8925606` now clean-starts with the documented swing events registered; a matching `--no-addons` control also returns `[]`. Independent verification passes 13 targeted event/enum tests, formatting, and default compilation: `/tmp/forever-addon-audit/verify-player-swing-ledger.json`. In an isolated 22-second GUI-style replay, injected `PLAYER_SWING` events for a four-second main-hand swing and six-second ranged swing activate both real fills, advance both, expire the main hand while ranged remains active, then clear both fills while leaving the idle bar tracks shown. The probe emits `DONE`, collects zero Lua errors, and preserves the host CVar hash. Exit `124` is timeout teardown after completion: `/tmp/forever-addon-runtime/player-swing-startup-controls-ledger.json` and `/tmp/forever-addon-runtime/appel-swing-final-gui-olh1svgs/ledger.json`.

This is injected-event coverage, not a modeled gameplay swing producer or range-check implementation. Off-hand visuals, rendered pixels, range behavior, and native timing remain untested. An earlier timer-based assertion that ran before the addon’s first frame update is excluded as timing-fragile.

## DinoUnitFrames bounded combo interaction

DinoUnitFrames `8936218` clean-starts at frozen `d23cfe65c`; its options root remains intentionally deferred. Matching no-addons startup returns `[]`. Forever now evaluates supplied health/power curves through the existing typed evaluator. Normalized curve input is inferred from the unchanged addon's `id / 5` thresholds and Blizzard's shared curve constants, not native-verified; uncurved numeric results and other-profile behavior remain unchanged. Independent proof passes eight focused tests, formatting, default compilation and source/readability review: `/tmp/forever-addon-audit/verify-forever-percent-ledger.json`.

The actual GUI target unit-watch and addon event handlers display injected combo counts **0→2→5→1→0**, checking each of five blocks' alpha and transparent retained extras. All markers through `DONE` occur before 20-second timeout teardown, with zero Lua errors and unchanged host CVars. Startup: `/tmp/forever-addon-runtime/dino-curve-startup-ppa3m3fy/ledger.json`; workflow: `/tmp/forever-addon-runtime/dino-combo-alpha-icywlldv/ledger.json`. Earlier headless assertions ran before visibility updates; a later hidden-state assertion contradicted the addon's explicit Show/alpha behavior and was corrected with user approval. Those attempts are not credited. Options, other unit modules, secret gameplay production and rendered pixels remain untested.

## ClassicCastBar startup and bounded settings workflow

ClassicCastBarForever `8909724` now clean-starts after `d1e2487f6` preserves an explicitly empty `RegisterCVar` default. Previously the simulator changed empty to omitted, then substituted `"0"`; the unchanged addon's CVar fallback overwrote its authored scale `1` with zero. Omitted/nil default `"0"`, prior values and strict positive-scale validation remain unchanged. Independent verification passes 15 CVar tests, formatting and default compilation: `/tmp/forever-addon-audit/verify-empty-cvar-ledger.json`. Isolated addon startup and no-addons control return `[]`: `/tmp/forever-addon-runtime/classic-empty-startup-eqn0uw2u/ledger.json` and `/tmp/forever-addon-runtime/empty-cvar-control-ci9ls5w9/ledger.json`.

A second producer defect prevented `Slider:SetValue(1.35)` from dispatching `OnValueChanged`. `3d6017fe3` now dispatches ordered existing script bindings after storing the clamped value, with the documented mouse-event flag, unchanged-value suppression and existing error reporting. Nine focused tests pass. Frozen unchanged-addon replay reaches scale, icon, reset and `DONE` with zero Lua errors: database/bar/CVar/character scale changes **1→1.35→1**, icon settings toggle, and saved anchors clear. Evidence: `/tmp/forever-addon-runtime/classic-slider-workflow-q675457h/ledger.json`; matching no-addons control `/tmp/forever-addon-runtime/slider-control-0uwemvbq/ledger.json` returns `[]`. The prior failed callback diagnostic remains preserved in the matrix.

This covers programmatic slider input and real slash handlers, not mouse dragging, actual casting, settings navigation, rendered pixels or restart persistence. Re-enabling the icon checks the saved setting; the addon does not explicitly show it in that path.

## Account-wide UI save/load failure

The unchanged `8935141` package starts cleanly. Frozen `37da0f132` resolves the missing recent-allies location preference getter at `SaveFunction.lua:457`, but the real `/awi save` handler now fails at line894 calling missing `GetAutoDeclineNeighborhoodInvites`. The self-cast CVar save/change/load sequence remains incomplete: `/tmp/forever-addon-runtime/account-location-workflow-nmsc23tn/ledger.json`. Prior failure evidence remains recorded; no workflow pass is credited.

## AccentChat bounded pre-send interaction

AccentChat `8935580` registers its actual pre-send callback. With the explicit out-of-date switch, isolated `d1e2487f6` replay translates deterministic Dwarf text `you are and the` to `ye be an' tha` for PARTY and PARTY_LEADER, preserves slash input and disabled-channel/addon text, and resumes translation after re-enable. Six checks and `DONE` pass with zero Lua errors and unchanged host CVars: `/tmp/forever-addon-runtime/accentchat-presend-ood-qrzg720v/ledger.json`.

The fixture injects `ChatFrame.OnEditBoxPreSendText` with a synthetic editbox; it does not call the translator directly. Network send, settings UI, other accents, secret/lockdown handling and restart persistence remain untested. An initial run omitted the required interface switch and did not load the addon; it is excluded rather than credited as a workflow failure.

## Aurarium bounded money/overview interaction

Aurarium `8915742` with cached ArcaneWizardLibrary `8915500` records injected balances `12345→54321`, opens its overview through `/aurarium overview`, and closes it through the real close button. Frozen `a6fbf432e` reaches `DONE` with zero Lua errors; a fresh no-addons money-event control also returns `[]`. Documented pet-slot constants and the Forever StableInfo read model resolve the two previously reproduced shared Blizzard errors without changing either addon. The model's empty, fully unlocked two-slot default and count interpretation are explicit simulator guesses, not native evidence. Evidence: `/tmp/forever-addon-runtime/stable-read-aurarium-vwhnppg7/ledger.json` and `/tmp/forever-addon-runtime/stable-read-money-control-trow38aa/ledger.json`. Host CVars remain unchanged. Persistence, currencies, warband aggregation, pet mutations and rendered output remain untested.

## Acceptance discipline

- Verify that intended addon roots actually load; empty error JSON with nothing loaded is not a pass.
- Separate clean startup, LoadOnDemand/excluded roots, missing dependencies, runtime failures, and incomplete observations.
- Define addon-specific major actions and assert resulting state. Merely finding a slash command, frame, or library is not interaction proof.
- Preserve exact package hashes, simulator revision/binary identity, commands, complete outputs, and proof scope. Reuse prior evidence only when relevant behavior is unchanged.
- Keep goal open while required projects or major workflows fail or remain untested. No full native-conformance claim follows from bounded simulator tests.

Initial archive proof: `/tmp/forever-addon-audit/selected-archive-hash-proof.json`. Prior runtime evidence reconciliation: `/tmp/forever-addon-audit/runtime-proof-inventory.json`. Detailed per-run artifacts remain under `/tmp/forever-addon-runtime/`; the tracked matrix records their paths and dispositions.
