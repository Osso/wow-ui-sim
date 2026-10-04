# Secure aura headers and ClassTalentHelper delegation

Retail 12.0.5 helper commands must drive the cached Blizzard UI without tainting fields they actually change. Simulator producers live in `src/c_api/class_talent_commands.rs`; shared timed completion lives in `src/lua_api/cast_completion.rs`. Aura ordering remains vendor-owned. See [event architecture](../event-system.md).

## What it must do

- [x] Real cached aura headers order finite expirations before permanent auras under TIME ascending, independent of input order; reuse positional children and hide removed entries.
- [x] All four helper commands dispatch the corresponding documented callback event through an untainted native boundary; addon callers and addon observers retain their own taint.
- [x] Specialization helpers initiate the existing activation cast, leave active specialization/loadout unchanged before its deadline, and complete through the same producer used by the GUI.
- [x] Completion clears the cast and pending selection, updates player and talent/loadout/hero state coherently, and publishes specialization notifications whose observers see completed state.
- [x] Existing seeded instant loadout behavior remains reachable through real UI callbacks and publishes `ACTIVE_COMBAT_CONFIG_CHANGED`; UI fields actually written remain untainted.
- [x] Callback errors restore caller taint. Invalid specialization commands do not start casts.

## How it works

- [Event system](../event-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/class_talent_commands.rs`: four retail command-event producers.
- `src/lua_api/globals/missing_surface/traits/class_talents.rs`: epoch-selected registration; existing LoadConfig provider retained.
- `src/lua_api/cast_completion.rs`: existing timed lifecycle shared with headless callers; retail talent-state synchronization and active-player specialization refresh.
- `src/iced_app/update.rs`: GUI calls shared lifecycle, not a second completion implementation.

## Tests asserting this spec

- `tests/secure_aura_header_helpers.rs`: actual cached Lua/XML and PlayerSpells callbacks, taint controls, aura ordering and timed specialization completion.
- `tests/hero_talents.rs::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state`: retained mapping contract through real helper initiation/completion for retail; earlier-epoch expectations unchanged.

## Development proof — round 101

The test/refactor-only checkpoint compiled: both aura tests passed, four helper tests failed, and the rewritten hero lifecycle test failed. With producers enabled, the helper module passed 6/6, hero talents 16/16, admin specialization/talent APIs 23/23, secure group headers 2/2, and relocated cast-completion lib tests 12/12. Expanded casting/specialization regression modules passed after three stale identity observers were adapted to the existing GUID/castBarID contract; unit filtering, monotonic numeric identity, and blocked-cast identity assertions were preserved.

These are simulator development tests against the cached retail UI, not native-client comparison or independent full-row acceptance. Audit classification remains unchanged.

## Development proof and independent bounded acceptance — 2026-10-03

Commits `a9f6b0a26` (aura ordering tests) and `d62a972eb` (helper delegation and shared cast completion). Worktree RED: aura 2 PASS, helper 0 PASS / 4 FAIL, rewritten lifecycle test 0/1. GREEN there 381 integration + 12 lib, `cargo check` clean; on master 144/144 with controls, startup `lua-errors` `[]`, `cargo fmt --check` exit0.

Main accepts an independent GPT-6.1-sol source review: **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/round-101-review.md) SHA256 `108ffe0e872180e50f2193eb78916633d1410913d2ed078f1d39ee5d83cec2bc`. Qualifications: the retail TOC excludes `SecureAuraHeader` (classic-only), so the ordering proof is isolated component proof; in an environment with no helper callbacks registered the four switch commands are now a silent no-op instead of mutating state; delayed loadout commit is unmodeled; two cast tests select the payload slot by epoch feature where the producer selects by `player-cast-durations`.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose 03-25-112, 03-31-151, 03-25-120 and 03-31-177 partial; **133 capabilities/362 IDs; 24 pending /270 bounded /33 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Startup was not executed: the required build helper invocation rejected `--run --target-dir` with an unrecognized-arguments error.
- [ ] Loadout `LoadInProgress` commit/castbar lifecycle is not modeled by this slice. Existing Ready-plus-instant-mutation provider is retained, not asserted as universal native behavior.
- [ ] Cold lazy loading, combat/secret arguments, native-client comparison, and automatic retail aura-header exposure are not covered.

## Out of scope

- Vendor Lua changes and simulator-side aura sorting: cached comparator already maps zero expiration to infinity.
- Invented loadout cast duration or changing Ready to LoadInProgress merely to exercise a UI branch: cached declarations/consumers do not establish such policy for every loadout.
- Other aura filters, sort directions, grouping/consolidation and permanent-aura tie ordering.
