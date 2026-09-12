# Forbidden-aspect scripted input and focus queries

Patch 12.1 PTR generated API declarations annotate exactly six methods with `ChecksForbiddenAspects`. This specification defines a bounded simulator policy.

## Source annotation evidence

Cached PTR `Blizzard_APIDocumentationGenerated` declarations establish only these bindings:

- `SimpleButtonAPIDocumentation.lua:42-45` — `Button:Click` checks `ScriptedInput`.
- `SimpleEditBoxAPIDocumentation.lua:20-22`, `646-650`, and `668-670` — `EditBox:ClearFocus`, `SetCursorPosition`, and `SetFocus` check `ScriptedInput`.
- `SimpleEditBoxAPIDocumentation.lua:408-410` — `EditBox:HasFocus` checks `QueryFocus`.
- `SimpleScriptRegionAPIDocumentation.lua:469-471` — `Region:IsMouseMotionFocus` checks `QueryFocus`.

The same source does not annotate `Region:IsMouseOver`; no global focus query is included in this slice.

## What it must do

### Uniform policy assumption

- [x] Apply all six gates uniformly, without caller-taint, caller-identity, or privileged-caller exceptions. This is a simulator policy assumption, not native security conformance.
- [x] Reject a gated method when its receiver carries the relevant aspect.
- [x] Reject before any state mutation, click-depth transition, cursor change, focus-script callback, or click callback.
- [x] Preserve ordinary validation, return values, state changes, and callbacks for zero-mask receivers and receivers without the relevant aspect.

### `ScriptedInput`

- [x] Reject `Button:Click` before button state changes and `OnClick` runs.
- [x] Reject `EditBox:SetFocus` and `EditBox:ClearFocus` before focused-frame, per-frame focus, visual, or focus-script changes.
- [x] Reject `EditBox:SetCursorPosition` before cursor state changes.
- [x] Preserve physical GUI mouse clicking, edit-box click-to-focus, keyboard delivery, and their existing callbacks; this gate restricts only the annotated script methods.
- [x] Preserve unannotated text methods and their ordinary behavior.

### `QueryFocus`

- [x] Reject `EditBox:HasFocus` and `Region:IsMouseMotionFocus` before returning focus or hover state.
- [x] Leave global focus queries and `Region:IsMouseOver` unchanged because this slice has no local source annotation for them.

## Proof required

- [x] Focused tests demonstrate each annotated method rejects atomically, with observable state and callback non-mutation.
- [x] Focused tests demonstrate unaffected physical GUI mouse/keyboard paths and unannotated methods remain available.
- [x] Focused tests cover ordinary zero-mask controls.
- [x] Four earlier-12.0.7 focus/click controls pass with the changed runtime.

Seven retail tests cover the six gates and actual GUI/keyboard input. Independent verification also passed formatting, retail checking, and PTR build. The earlier-profile build reports unused `CastInfoSnapshot.delay_time`; its regression status remains unestablished.

## Out of scope

Native error wording/timing, native authority or taint semantics, secrets, VM/rilua changes, other forbidden aspects, global focus APIs, `IsMouseOver`, and text-method restrictions.
