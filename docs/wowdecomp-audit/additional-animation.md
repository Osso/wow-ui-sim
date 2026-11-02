# Additional animation discrepancies

Static source comparison at the [expanded-pass revisions and interpretation limits](additional-rendering-and-map.md). No animation was executed or rendered. Historical 3.3.5a behavior must be reconciled with each supported profile before implementation.

## A1 — Translation, rotation and scale configuration does not animate the target

**External:** `Engine/Source/Frame/CSimpleTranslationAnim.cpp:63-84`, stores offsets and applies progress-scaled translation to the region. `CSimpleRotationAnim.cpp:63-70`, applies radians around an origin. `CSimpleScaleAnim.cpp:58-84`, stores scale change and applies it to the region. Their corresponding `_Scripts.cpp` tables expose the class-specific setters/getters.

**Simulator:** `src/lua_api/frame/methods/button_anchor_hierarchy/mod.rs:548-574` routes `SetOffset`, `SetDegrees`, `SetOrigin`, `SetScaleFrom` and `SetScaleTo` to `animation_config_noop`. `animations/creation.rs:16-23` sends frame-backed `SetScale` to ordinary frame scaling, not a time-varying scale-animation target. `src/lua_api/animation.rs:80-109` has no translation offset/rotation angle/scale endpoint fields. `animations/runtime.rs:16-49` applies alpha/flipbook, vertex-colour and path outcomes, not those missing transform configurations. Existing Path movement does not implement a Translation animation's ignored setter.

**Counterexamples:** attach a Translation with offset `(40,0)` and duration 2 to a known rectangle; at midpoint the reconstructed path contributes 20 units of movement, while the simulator ignored the configured offset. Rotation by 90 degrees and Scale to `(2,1)` likewise need actual owner geometry changes, not changed scale on an animation-handle widget or a working progress getter.

**Rework:** model the supported transform properties, their origins and per-tick target application/composition/restoration. Keep ordinary frame `SetScale` separate from Scale-animation configuration. XML `radians` is parsed (`src/xml/types_animation.rs:111-114`) but `src/loader/helpers_anim.rs:339-365` emits only degrees; repair this input path as part of rotation behavior, not a separate inflated finding.

**Regression:** public API and equivalent XML fixtures for each transform; assert owner pixels/geometry at initial, midpoint, completion, pause, stop and looping boundaries. Preserve authored anchors/base scale and correct target identity. Respect the current target/profile contract for transform composition and modern from/to APIs rather than transplanting legacy internals.

## A2 — `GetLoopState` returns configured mode instead of live direction

**External:** `Engine/Source/Frame/CSimpleAnimGroup_Scripts.cpp:125-130`, returns `m_loopState`; `Engine/Source/FrameXML/LoadXML.cpp:104-108,343-352`, maps it to `NONE`, `FORWARD` or `REVERSE`. `CSimpleAnimGroup.cpp:889-906` sends that state to loop callbacks.

**Simulator:** `src/lua_api/frame/methods/button_anchor_hierarchy/animations.rs:358-374` aliases `animation_group_get_loop_state` to `animation_group_get_looping`, which returns configured `NONE`, `REPEAT` or `BOUNCE`. Playback direction and configured repetition policy are distinct facts.

**Counterexample:** start a non-looping group containing a timed animation: reconstructed startup selects live `FORWARD` (`CSimpleAnimGroup.cpp:563-574`), while simulator's configured-mode getter reports `NONE`. During reverse BOUNCE playback the source can report `REVERSE`, while simulator still reports `BOUNCE`.

**Contract conflict:** [the existing query-lifecycle spec](../specs/animation-query-lifecycle.md) explicitly defines the simulator getter as configured loop mode. This historical discrepancy does not authorize rewriting that tested contract without a supported-profile decision/evidence.

**Rework/regression:** if live state is the intended target contract, expose direction separately from loop mode; assert getter and `OnLoop` arguments while moving forward/reverse and on inactive/paused boundaries. Establish exact boundary ordering rather than inferring it solely from an elapsed-time formula.

## A3 — Invalid loop mode silently resets configuration

**External:** `Engine/Source/Frame/CSimpleAnimGroup_Scripts.cpp:96-114`, errors for unknown mode rather than changing it.

**Simulator:** `animations.rs:345-355` feeds any input to `LoopType::from_str`; `src/lua_api/animation.rs:63-71` maps everything except REPEAT/BOUNCE to NONE.

**Counterexample:** after `SetLooping("REPEAT")`, `SetLooping("INVALID")` errors/preserves mode in the reconstruction; simulator returns normally and its getter reports `NONE`. Exact downstream playback/callback consequences were not established.

**Rework/regression:** validate the target-profile vocabulary before mutation. Assert `pcall` failure plus unchanged prior mode and successful valid controls. Do not add a fallback-to-NONE policy unless it is actually the intended target contract.

## A4 — Animation order bounds are not enforced

**External:** `Engine/Source/Frame/CSimpleAnim_Scripts.cpp:231-251`, converts public order to/from zero-based storage. `CSimpleAnim.cpp:519-532`, clamps ordinary assignment to internal `[0,99]` (public `[1,100]`); its separate wrap behavior is not the same as clamping.

**Simulator:** `src/lua_api/frame/methods/button_anchor_hierarchy/animations/fields.rs:66-76` accepts an unbounded nonnegative integer order; `animations.rs:600-602` returns it directly.

**Counterexample:** `SetOrder(101)` returns order 100 in the reconstruction, 101 in simulator; zero maps to public one in the reconstruction, but can remain zero in simulator. Order also controls grouping/timeline scheduling, not just a getter.

**Rework/regression:** establish bounds for the supported profile, normalize public assignment consistently with scheduling, and test getters plus concrete order execution for low/high boundary cases. Existing duration arithmetic does not prove order-bound handling. Preserve an intentionally different modern contract if independently established.

## A5 — Group initial offsets are neither exposed nor applied

**External:** `Engine/Source/Frame/CSimpleAnimGroup_Scripts.cpp:141-180` exposes initial-offset accessors (). `CSimpleAnimGroup.cpp:210-218,446-463,984-996,1065-1066` for the setter, stores XML/API initial offsets and adds the translation to the owner during group updates.

**Simulator:** no `SetInitialOffset`/`GetInitialOffset` registration was found. `src/xml/types_animation.rs:9-39` group fields and `src/lua_api/animation.rs` group state omit the offsets; `src/loader/helpers_anim.rs` cannot forward them to playback.

**Counterexample:** otherwise identical XML animation groups with `initialOffsetX="20"` differ in reconstructed owner position while active; simulator discards that attribute. Public accessors are absent rather than roundtripping the offset.

**Rework/regression:** if this historical group feature belongs to the supported profile, parse/store/expose and apply it through the same target transform composition as A1. Test nonzero X/Y via XML and API plus zero/default and stop/reset behavior. Absence in a newer profile may be deliberate; no universal modern availability claim.

## Rejected alpha no-op claim

`SetChange` is **not** routed to a no-op: `button_anchor_hierarchy/mod.rs:569` selects `animations::animation_set_change`, and `animations.rs:591-597` changes the alpha endpoint. The broad explorer's contrary finding was rejected. Historical delta quantization and missing `GetChange` remain version-sensitive leads, not an assertion that alpha animations never work. Existing alpha, Path, VertexColor and FlipBook support is not erased by A1.
