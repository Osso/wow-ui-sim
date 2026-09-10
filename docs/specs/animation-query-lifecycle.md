# Animation query lifecycle

Ordinary simulator query behavior for `SimpleAnim` and `SimpleAnimGroup`, exercised through real playback and ticks. The pinned PTR register adds `QueryAnimationProgress` access checks to sixteen query occurrences; these tests do not establish that security contract or native WoW equivalence. See [event dispatch](../event-system.md).

## What it must do

- [ ] Animation `IsPlaying`, `IsPaused`, and `IsDone` follow their owning group through playback, pause/resume, restart, stop, and completion; an independent idle group stays unchanged.
- [ ] Pausing freezes elapsed time. Restart and Stop reset elapsed/progress. The modeled animation `IsStopped` is true whenever its owner is not playing, including pause.
- [ ] `Finish` remains pending until a tick; completion callbacks observe settled state and fire once. Natural completion after restart also settles owner/child queries.
- [ ] Reverse playback and repeating loops update elapsed/progress while `IsReverse` and `GetLoopState` identify direction and configured loop mode. Bounce callbacks observe direction changes.
- [ ] Delayed animations expose local active elapsed/progress independently of the group's total timeline; animation progress clamps during end delay. `GetSmoothProgress` currently equals unsmoothed progress even with `IN` smoothing.

These are simulator-model requirements, not native lifecycle or smoothing claims.

## How it works

- [Frame/script data flow](../frame-data-flow.md)
- [Event and update dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs`: shared FrameRef method registration; animation objects can call group-state query methods.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations.rs`: owner-state and local elapsed/progress queries.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/runtime.rs`: playback advancement and completion/loop callbacks.

## Tests asserting this spec

`tests/animation_query_lifecycle.rs`, auto-discovered into the existing `integration` Cargo target:

| Test | Queries/boundary |
|---|---|
| `owner_queries_follow_pause_resume_restart_and_stop` | Group/animation `IsPlaying`, `IsPaused`, `IsDone`; animation `IsStopped`, `GetElapsed`, `GetProgress`, `GetSmoothProgress`; group `GetElapsed`, `GetProgress`; paused ticks and independent owner |
| `pending_finish_queries_are_settled_before_callbacks` | Above state/progress queries plus group `IsPendingFinish`; explicit Finish and natural completion callback visibility |
| `reverse_and_repeat_queries_follow_playback_direction` | Group `IsReverse`, `GetLoopState`, elapsed/progress; child elapsed/progress across reverse and repeat |
| `bounce_loop_callback_observes_direction_change` | Loop/direction/state queries inside real `OnLoop` callbacks |
| `delayed_animation_queries_use_local_active_elapsed` | Animation `IsDelaying`, elapsed/progress/smooth progress; group elapsed/progress and completion across start/end delays |

Existing complementary coverage: `tests/animation_group_state.rs`, `tests/animation_group.rs`, and `tests/animation_anim.rs`. No new Cargo target or API-publication absence assertions are needed.

## Known gaps (current cycle)

- [ ] Investigate delay-flag consistency: the getter compares already delay-adjusted local elapsed against start delay. The test records the intermediate flag diagnostically rather than requiring a suspected inconsistency.
- [ ] Native owner-versus-child state, delay boundaries, smoothing, reverse/bounce geometry, and callback timing are not established by this model coverage.

## Out of scope

- `SetParent` and `CreateAnimation` mutation contracts: no production changes in this test-only slice.
- Forbidden-aspect, protected, taint, and secret enforcement: ordinary lifecycle tests do not exercise those boundaries.
- Audit manifest/checklist changes, broad suites, rendering proof, and final verification gates: separate work.
