# Widget button-state and scroll-offset secret aspects — B37–B38

Eight Retail 12.0.7 widget changes add `ButtonState`/`ScrollOffset` input and output annotations in the [retained API source](../../data/patch-api/sources/12.0.7-api-changes.txt). Producers must read live per-widget state and authenticate actual VM secrets. The later retail cache supplies declaration context, not native historical proof. This is an authored, unintegrated contract; no test has run. See [Lua API](../lua-api.md) for runtime architecture.

| Source row | API | Required delta | Live input |
|---|---|---|---|
| widgets-Button-GetButtonState-129 | `Button:GetButtonState` | `SecretReturnsForAspect(ButtonState)` | `Frame.button_state`; state/enabled origins |
| widgets-Button-IsEnabled-130 | `Button:IsEnabled` | `SecretReturnsForAspect(ButtonState)` | `Frame.attributes["__enabled"]`; state/enabled origins |
| widgets-Button-SetButtonState-131 | `Button:SetButtonState` | `SecretArgumentsAddAspect(ButtonState)` | authenticated token/lock; state origin |
| widgets-Button-SetEnabled-132 | `Button:SetEnabled` | `SecretArgumentsAddAspect(ButtonState)` | authenticated enabled; enabled origin |
| widgets-ScrollFrame-GetHorizontalScroll-138 | `ScrollFrame:GetHorizontalScroll` | `SecretReturnsForAspect(ScrollOffset)` | horizontal offset; both axis origins |
| widgets-ScrollFrame-GetVerticalScroll-139 | `ScrollFrame:GetVerticalScroll` | `SecretReturnsForAspect(ScrollOffset)` | vertical offset; both axis origins |
| widgets-ScrollFrame-SetHorizontalScroll-140 | `ScrollFrame:SetHorizontalScroll` | `SecretArgumentsAddAspect(ScrollOffset)` | authenticated horizontal offset; horizontal origin |
| widgets-ScrollFrame-SetVerticalScroll-141 | `ScrollFrame:SetVerticalScroll` | `SecretArgumentsAddAspect(ScrollOffset)` | authenticated vertical offset; vertical origin |

## What it must do

### Publication and live state

- [ ] New behavior is gated directly on `retail-12-0-7`, not the separately inherited `forbidden-aspects` feature. With that feature absent, even host-configured origin bits do not wrap getter outputs.
- [ ] Default getters return plain `NORMAL`, true, zero and zero, one result each. Setters keep zero-result arity and ordinary state transitions.
- [ ] All four getters read live state on every call. Changing token, enabled attribute, either offset or any origin bit after creation changes the next result; no captured/wrapped constants.
- [ ] Inputs and origins are per frame and per `WowLuaEnv`; same names in separate environments share nothing.

### Secret authentication and output policy

- [ ] Each setter calls `rilua::table_security::unwrap_secret` on receiver, every supplied named argument and every extra argument before receiver lookup, argument validation or mutation. Authentication neither clears taint nor modifies original wrappers.
- [ ] Secure callers may set actual VM secret token, bool, lock and offset inputs. Tainted callers cannot; denial leaves offsets/token/enabled/origins unchanged and sends no callbacks.
- [ ] Secret origin on either button input makes both button getter results secret; secret origin on either scroll axis makes both offset getter results secret. Getters still succeed for tainted callers and preserve exact result count.
- [ ] Trusted host bool/string/number wrappers preserve live payload values. Tainted callers cannot unwrap outputs, branch on enabled, order offsets or perform offset arithmetic. Caller taint stays unchanged.
- [ ] INFERRED: an accepted extra secret argument contributes to the shared aspect; secrets in lock contribute even when the existing lock behavior ignores that argument.
- [ ] INFERRED: origins are tracked per input. Public overwrite clears only its own input origin, even for an unchanged value; another secret input keeps the shared aspect secret. No automatic inheritance or permanent accumulation is claimed.

### Script ordering and B36 prerequisites

- [ ] Existing OnEnable/OnDisable and scroll callback ordering remains: actual value change commits value and origin before dispatch, unchanged values suppress dispatch. Public payload/order behavior remains intact.
- [ ] `EditBox`, `Font`, `FontString`, `MessageFrame` and `SimpleHTML` SetFont paths authenticate every argument and extra under the same feature gate before existing decoding and early returns. Secure secret path/height/flags work with the existing storage; tainted input denial is atomic. This is an authentication prerequisite only, not `RequiresValidFontHeight` or `RequiresValidFontAsset` coverage.

## How it works

- [Lua API](../lua-api.md)
- [Widget system](../widget-system.md)
- [Scroll script bindings](scrollframe-script-bindings.md)
- [Button enabled callbacks](button-enabled-callbacks.md)

## Implementation inventory

Proposed edits, not integrated:

- `src/widget/frame.rs`, `src/widget/frame_defaults.rs` — four explicit per-input origins, default false.
- `src/lua_api/frame/methods/secret_origin.rs` — whole-call VM authentication with a direct feature gate.
- `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs` — live button inputs, shared-aspect output wrapping and accepted-input origin writes.
- `src/lua_api/frame/methods/widgets/slider.rs` — live scroll inputs, shared-aspect output wrapping and accepted-input origin writes.
- `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs`, `src/lua_api/globals/font_strings_collection/fonts.rs` — authentication-only SetFont prerequisites; existing receiver storage and return shapes unchanged.

## Tests asserting this spec

`tests/patch_12_0_7_widget_secret_aspects.rs`: nine authored cases (eight with `retail-12-0-7`, one without), no execution proof:

| Case | Observable contract |
|---|---|
| `widget_aspects_defaults_and_public_round_trips` | Defaults, exact getter/setter arity, token/enabled and both offset transitions |
| `button_state_secret_token_lock_and_equal_public_overwrite` | Actual secret token/lock/extras, payload identity, other-frame control, origin clearing |
| `button_enabled_secret_origin_callbacks_and_shared_lifecycle` | Secret bool, change-only callbacks observing committed state/origin, shared input lifecycle |
| `widget_and_font_authentication_denies_tainted_inputs_before_mutation` | All four setter positions/extras/receiver denial, validation-order error, atomicity, ordinary tainted access; five SetFont receivers and secure secret round trips |
| `widget_secret_outputs_are_returned_but_opaque_to_tainted_callers` | Secret getter identity/arity, tainted opacity/taint preservation, secure live payload recovery |
| `scroll_offset_shared_origin_live_updates_and_equal_value_clearing` | Two-axis origins, payload round trips, extra-input inference, equal-value clearing and frame isolation |
| `scroll_callbacks_observe_committed_offset_and_origin_once` | Commit-before-callback and unchanged suppression; callbacks query current getters |
| `host_inputs_are_read_live_and_are_isolated_between_environments` | Direct host token/enabled/offset/origin updates, repeat reads, environment isolation |
| `earlier_epochs_keep_plain_widget_outputs` | Feature-off getter behavior with deliberately configured origin bits |

## Known gaps (current cycle)

- [ ] All nine tests are authored only. Compilation, RED/GREEN, startup Lua errors, broader regression checks and independent acceptance are not run; prohibited in this authoring task.
- [ ] B36 rows 133/134/135/136/142 remain blocked: no authoritative host font-validity model, height bounds or native failure semantics. The optional renderer/CASC cache and default-family substitution are not a validity oracle. FontString's later-cache `FontAsset` domain differs from the other receivers' `cstring`; receiver-specific return contracts require care. Authentication-only edits cannot close these rows.
- [ ] Native aspect clearing/aggregation, treatment of extra arguments and strict 68182 declaration epoch remain unverified; their proposed policies are INFERRED.
- [ ] Scroll callbacks still receive existing plain numeric event arguments; their secret payload policy is not declared in the inspected generated docs. Tainted callback access can expose offsets despite getter opacity. Do not claim end-to-end non-disclosure or quietly change script payloads without consumer/native evidence.
- [ ] Public scrolling, enabled bindings and cached UI consumers need integration-time regression proof. Shared ScrollUtil/InputBox consumers perform offset arithmetic; opaque secret inputs in those paths may fail. No vendor changes, broad secret-arithmetic changes or read-denial substitutes are authorized here.

## Out of scope

- Native font asset loading, height rules, clamping, invalid-font failure shape, hard-coded font allowlists and renderer fallback validity: unsupported evidence and unsafe consumer assumptions.
- Lock semantics, SetEnabled missing/nil argument coercion, Enable/Disable/Click-origin clearing, protected-frame access control and arbitrary aspect inheritance: existing behavior retained, not part of these added annotations.
- ScrollRange and script event secret-argument annotations: separate policy boundary, not proved by ScrollOffset getter tests.
- Native parity, older-profile full UI loading, changes to cached Blizzard/vendor Lua, source-substring tests, simulator/build execution and row promotion.
