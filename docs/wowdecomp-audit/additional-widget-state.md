# Additional widget-state discrepancies

Static source comparison at the [expanded-pass revisions](additional-rendering-and-map.md); no runtime, pixel or native verification. Each proposed adoption must preserve independently established supported-profile contracts.

## C7 — Button highlight/disabled fonts do not project into active text

**Historical baseline finding; reported implementation completed.** See the [exchange checklist](README.md#exchanged-implementation-checklist) for bounded Retail/Wrath results and unresolved native/live-font/nil-slot limits. The simulator trace below describes the audit baseline, not those later revisions.

**External:** `Engine/Source/Frame/CSimpleButton_Scripts.cpp:175-198,251-274` updates state after assigning font objects. `CSimpleButton.cpp:522-549`, `UpdateTextState`, chooses disabled/highlight/normal FontObject based on actual state and highlight layer, including pushed text offset. `LockHighlight` at `:64-69` also invokes text-state refresh.

**Simulator:** `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:379-394` applies a normal-font snapshot, but `:418-440` stores highlight/disabled references only in `__button_font_objects`. Source-wide searches find those slots only in their setter/getter paths. `src/lua_api/frame/methods/misc/highlight.rs:90-102` changes highlight lock/texture visibility without selecting the corresponding text FontObject. Correct highlight texture visibility does not establish correct text style.

**Counterexample:** a button has a real text FontString, normal font 12/red and highlight font 20/green. Activate highlight and assign/lock the highlight font: reconstructed text adopts the highlight style; simulator retains its prior snapshot despite getters returning the new highlight FontObject. Disabled-font transition has the same missing projection path.

**Rework/regression:** centralize the active button-text style projection and invoke it on state/highlight/enabled transitions and font-slot changes. Preserve explicit FontString and existing texture ownership rules. Test concrete font size/colour/path of rendered button text in normal, highlighted, locked, pushed and disabled states, including replacing the active font slot. Shared FontObject live mutation is a separate dependency question, not solved merely by copying a snapshot again.

## C8 — Slider range changes lack their script capability and value-transition lifecycle

**External:** `Engine/Source/Frame/CSimpleSlider.cpp:49-61` produces `OnMinMaxChanged(min,max)`; `:351-369`, `SetMinMaxValues`, commits the new range, invokes that callback and re-applies an initialized value through normal assignment. The existing value path can then produce `OnValueChanged` and applies C1 step rules.

**Simulator:** `src/lua_api/frame/methods/widgets/slider.rs:266-282,301-319` directly writes the range and clamps the value without either callback. `src/event/mod.rs` knows the `OnMinMaxChanged` name, but `src/lua_api/frame/methods/text_attribute_event/events.rs:397-435` has no Slider script-support case for it; the generic support predicate rejects an unrecognized handler. An enum entry alone does not make the public callback bindable.

**Counterexample:** bind range/value handlers, set a range/value and shrink the range across that value. The reconstruction reports the range transition, then normal value reapplication; simulator can reject the range handler and silently clamps the value in the setter.

**Rework/regression:** implement the target's script capability and range/value producer ordering, routing range-induced value changes through the same modeled assignment semantics rather than a silent field clamp. Test public handler binding, exact payload/order, callback-observed state, unchanged-range control and value clamp/step interaction. Existing [Slider callback contract](../specs/slider-value-callback.md) must be reconciled rather than replaced from a historical excerpt.

**Initialization lead:** source `SetValue` returns before any range was initialized (`CSimpleSlider.cpp:248-253`); simulator defaults to `[0,100]`. This is an additional version-sensitive initialization policy within the same lifecycle, not a separately inflated finding. Current profile default/range-initialization semantics remain unresolved.

## C9 — Scroll child-range refresh crosses a different callback boundary

**External:** `Engine/Source/Frame/CSimpleScrollFrame_Scripts.cpp:174-180` marks a deferred update. `CSimpleScrollFrame.cpp:225-265` computes the range and dispatches `OnScrollRangeChanged` during layer update when the deferred flag is consumed and the range changes.

**Simulator:** `src/lua_api/frame/methods/widgets/slider.rs:634-665` calls `refresh_scroll_child_rect` immediately and dispatches the changed-range callback inside the API call. This is not a claim that scroll range computation or ownership is absent.

**Counterexample:** change child bounds, call `UpdateScrollChildRect`, then record whether the range callback ran before returning or only after the next update tick. Reconstruction defers; simulator's current method is synchronous. No such timing fixture was executed here.

**Rework/regression:** establish supported-profile callback timing against the owning [scroll script contract](../specs/scrollframe-script-bindings.md), then model the correct dirty/update producer boundary while preserving range arithmetic, ownership and presentation. Test actual frame/update ordering with a valid child/viewport and a changed versus unchanged range; a getter-only unit test cannot prove the timing contract. If the existing synchronous policy is deliberate for a supported profile, record that instead of treating the old source as universally authoritative.

## StatusBar orientation false positive rejected

`SetOrientation`/`GetOrientation` are not absent merely because `widgets/statusbar.rs` does not list them. `widgets/slider.rs:672-676` registers their shared methods; the method index exposes shared methods, and `src/iced_app/statusbar.rs:35-48` consumes the same `slider_orientation` field that those methods set. The unused `statusbar_orientation` field is not proof of a missing API or renderer. The coverage ledger limits any match to the traced orientation aspect; dirty/layout and profile defaults are separate questions.
