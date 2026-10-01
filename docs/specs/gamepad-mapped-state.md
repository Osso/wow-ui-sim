# Forever mapped-stick queries

Forever exposes the two `C_GamePad` queries consumed by cached `Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua`. The backing model lives in `src/c_api/c_game_pad.rs`; [system notes](../wiki/systems/gamepad-mapped-state.md) describe its bounded implementation. This is a **source-observed subset**, not a native mapped-state DTO or hardware model.

## What it must do

- [x] Forever direct calls to `GetDeviceMappedState()` and `StickIndexToConfigName(index)` expose the modeled queries.
- [ ] Publication is Forever-only (source-gated; no executed other-profile absence proof). The actual cached initializer calls the getter eagerly twice, without arguments, and passes ordinary zero-based numbers to the selector.
- [x] No configured snapshot returns nil. Defaulting to no configured snapshot is simulator policy, not observed hardware detection.
- [x] Actual cached initialization marks both listener sides centered.
- [x] A configured ordered stick list publishes coherent `stickCount`, one-based `sticks`, and per-stick `len`. Selector index zero names the first configured stick. The actual cached initializer compares names with `Movement`/`Camera` and checks `len == 0`.
- [x] Concrete nonzero Movement/zero Camera initializes left uncentered/right centered. Reordered Camera/Movement inputs initialize the opposite sides from names rather than fixed positions.
- [x] **Inferred simulator policy:** finite coordinate fixtures produce Euclidean length; Rust x/y are input fields, not claims about native Lua fields. Each getter returns independent tables; replacement, empty snapshots and clearing are visible on subsequent queries.
- [x] Each environment owns its optional snapshot independently of logical UI input style and frame gamepad flags.
- [x] **Inferred simulator policy:** absent/out-of-range/negative/fractional numeric selectors return nil; string and opaque secret selectors error, with opaque input remaining secret.
- [ ] Other nonnumeric selectors and payload-free error reporting lack observable assertions. No native coercion or secret contract is claimed.
- [x] Ordinary addon-tainted calls preserve caller taint; opaque selector input remains secret after rejection.

Checked bullets denote bounded observable model/initializer proof, not overall acceptance or native parity. Current evidence under `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/` is `hover-green-ledger.json`, `hover-green.stdout` and `hover-green.stderr`: source-scoped `a956dfdd3`, rilua `6044544b`, 75 cases / 71 pass / 4 fail (exit 101). All eleven mapped/hover cases pass: three direct-model, three actual cached initializer and five hover cases. Prior messaging/module/restricted/BugCapture cases pass 36/36; loot/instance controls pass 24/24. Four AutoRoll cases stop before local lifecycle decisions at `Blizzard_StaticPopup_Game` requirements for `C_Club.GetInvitationCandidates` and `C_GameRules.IsHardcoreActive`. Eager-call versus lookup classification is recorded below; the separate [AutoRoll fixture contract](forever-auto-roll-fixture.md) owns the approved consumed-path scope and unexecuted revision. No native or AutoRoll acceptance follows. Historical `model-v2` 63/70 proof remains retained, superseded only for these current boundaries.

## Free-look-hover policy

Cached Forever `GamePadDocumentation.lua:135-141,231-238,305-314` declares a non-nil bool getter, a bool setter with `SecretArguments = "AllowedWhenUntainted"`, and `GAME_PAD_ALLOW_HOVER_EVENTS_WITH_FREE_LOOK_CHANGED` with `SynchronousEvent = true` and one bool payload. Cached `FrameControlsManager.lua:830` consumes the setter. These declarations establish surface/security/event shape, not native initial value or change detection.

- [x] Forever getter returns an ordinary boolean and setter is callable; Forever-only publication remains source-gated, with no executed other-profile absence proof.
- [x] Require an actual boolean after VM secret access checks, without truthiness/numeric/string coercion. Untainted callers may pass opaque booleans; tainted callers may pass ordinary booleans but must be rejected on opaque arguments. Preserve caller taint and opaque argument secrecy; invalid input must not mutate state or emit events.
- [x] **Inferred simulator guesses:** initialize each environment false, ignore repeated values, and write changed state before dispatching synchronous callbacks and returning. Emit exactly one ordinary bool payload per transition.
- [x] Observed environment-local policy is independent of mapped sticks and logical UI input style.
- [ ] Frame mouse/gamepad-flag independence and exact Forever-only event publication remain source-only; other-profile absence is not executed.

Five hover cases now pass in the current run; checked criteria reflect their bounded assertions. Existing tests commit `0ff9d1c5a` has executed RED: `hover-red-ledger.json`, `hover-red.stdout`, and `hover-red.stderr` under `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/` record 3 pass / 12 fail. All five hover cases stop registering the unknown event, before getter/transition/security assertions. Collect-all diagnostics at `ee3e27172` find only the setter unexpected within the failed GamepadSharedUtility TOC in seven cached cases; later dependency closure remains unknown. Only the exact known Reveal setup diagnostic is retained. The current run establishes bounded hover and cached initializer GREEN, not native semantics or AutoRoll behavior. Parent owns independent verification gates.

## How it works

- [Environment-local input and staged result tables](../wiki/systems/gamepad-mapped-state.md)
- [Separate logical input style](forever-input-interface-style.md)

## Implementation inventory

- `src/c_api/c_game_pad.rs` — public concrete stick inputs, mapped queries, and secret-aware hover getter/setter using synchronous `script_helpers::fire_named_event_state` after releasing the state borrow.
- `src/c_api/mod.rs`, `src/c_api/registration.rs` — Forever-only publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — optional mapped input and private crate-visible Forever hover boolean, initialized false.
- `src/event/valid_events.rs` — exact hover-changed event in the Forever-only sorted extension.

## Tests asserting this spec

`tests/gamepad_mapped_state.rs` is autodiscovered inside the existing grouped integration target. Three cases load the actual cached GamepadSharedUtility TOC dependency closure without overrides or callback bypasses, with fixture state installed before initialization. Other cases cover replacements/counts/names/table independence, environment isolation and taint/opaque-selector policy. The exact existing SharedXML `C_Reveal` debug-setup diagnostic is retained and reported; its truthy stub changes setup behavior and is not native absence or an innocent probe.

Five `hover_*` cases in `tests/gamepad_mapped_state.rs` assert synchronous state-before-event transitions, isolation from sticks/style, invalid ordinary arguments, untainted opaque acceptance, and tainted opaque rejection. Their recorded RED remains historical; current `hover-green.stdout` records all five passing.

## Final independent proof and limits

SSOT: `/home/osso/.local/state/wow-ui-sim-proof/forever-auto-roll-2026-10-01/independent-hover-report.md` (read fully) and `independent-hover-command-ledger.json`, with referenced invocation/ledger files. Independent source, wiring, security and readability audits completed; readability advisories remain, not edit authorization. Runtime at `a956dfdd3`: **overall FAIL**, 71/75 pass (11/11 mapped/hover, 60/60 prior controls, 0/4 AutoRoll blocked before local loading/decisions). All 4393 cached sources, 205 addon files, host CVars and executable remain unchanged; no hash drift or stale-cache claim.

Default `cargo check --offline` exits 0 with zero warnings. First `cargo fmt --check` fails on concurrent `CastTargetSnapshot` re-export wrapping; original failed-fmt and restart/lost-output proof remain preserved. Corrected fmt passes at `8d2c7ccff42f4dd4293e42f1c5fdc5ea2ecbd50c` with unchanged source/config input scope. `independent-hover-check-input-reconciliation.json` classifies during-check input drift as whitespace-only plus uncompiled `tests/unit_spell_target_name.rs`; `independent-hover-concurrency-classification.json` retains full diffs. No check/runtime rerun. Casting representation changes preceding default gates occurred after the runtime: current root casting representation is not blanket-covered by the 75-case run.

Forever `[Family]` resolves Mainline. Loaded `Mainline/GameDialogDefs.lua:1387` eagerly captures `C_Club.GetInvitationCandidates` as `autoCompleteSource`; invocation at :1357 is only inside the `InviteToClub` callback. Base `GameDialogDefs.lua:3412` eagerly calls `C_GameRules.IsHardcoreActive` to gate popup definitions. Basename/:1371 diagnostic attribution is not the physical Mainline-family line. Neither local AutoRoll source invokes these queries. No code exemption, masking or vendor modification follows.

Production edits stopped after three dependency iterations. User approved consumed-path-only AutoRoll testing; its [separate fixture contract](forever-auto-roll-fixture.md) records exact retained setup diagnostics, observer calibration and pending GREEN. No AutoRoll decision, native parity, inventory credit or overall completion follows; existing native/hardware/Reveal/profile limits remain.

## Known gaps (current cycle)

- [x] Retry compilation exits 0 with unchanged recorded inputs; five hover and three cached initializer cases pass.
- [ ] Four AutoRoll workflows historically stop at the StaticPopup requirements above; [approved fixture revision](forever-auto-roll-fixture.md) is unexecuted.
- [x] Independent audits and scoped default gates completed as recorded above; overall acceptance remains FAIL.

Original GREEN build runner was interrupted by Pi restart, proven a terminated zombie with no surviving compiler. Original lost-output proof is retained, not silently replaced by retry success. `hover-green-retry-build-ledger.json` records exit 0 at unchanged `a956dfdd3` inputs; runtime artifact names remain `hover-green-*`, not `hover-green-retry-*`. Runtime ledger records unchanged binary, addon and host CVar hashes. Later commits are not blanket-covered by this source-scoped proof.

Independent historical audit `20366` remains complete for 63/70. Default fmt/check in `/tmp/patch-12.0.5-batch11-rust-gates.json` remain historical. This docs audit runs no tests/builds/checks. Native hover/controller and Reveal behavior remain unmodeled; retained exact Reveal setup diagnostic changes debug setup, not native absence. No AutoRoll or whole-inventory credit.

## Out of scope

Full native fields, optional getter arguments, selector coercion/secrecy parity, physical devices, enumeration/configuration/vibration, additional `C_GamePad` methods beyond the two mapped queries and two hover-policy methods, native cursor/hover filtering, host free-look/controller support, host events, Admin producers, Retail, Reveal modeling and AutoRoll workflow integration. Source only establishes the consumed fields and call forms; native probes are unavailable. No vendor edits, no-op masks, fallback paths or diagnostic-classification patches.
