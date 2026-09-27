# EditBox text position and selection

Shared EditBox cursor and highlighted-range methods expose bounded text-position behavior. This spec covers valid UTF-8 byte boundaries, selected-range `Insert`, and changed programmatic text replacement, not the entire input system.

## What it must do

- [x] `SetCursorPosition` accepts valid UTF-8 byte offsets; `GetCursorPosition` returns the byte offset and `GetUTF8CursorPosition` returns the Unicode scalar count before the cursor.
- [x] Cursor positions past the end clamp to the end; `Insert` at the cursor preserves UTF-8 and advances both public getters consistently.
- [x] `HighlightText` with valid byte endpoints selects a range; `Insert` replaces that range (including empty-string deletion), collapses the cursor after inserted text, and clears selection for the next insertion.
- [x] ASCII cursor and selected-range editing retain their original units; renderer-facing stripped text remains synchronized with inserted text.
- [x] `Insert` derives its resulting cursor from the actual clamped edit range, including when earlier text replacement left cursor or selection indices beyond the new text.
- [x] Keyboard printable input replaces a highlighted range; Backspace and Delete remove it. Each consumes selection and commits text/caret/render caches before existing callbacks. Rejected numeric input preserves the pending selection.
- [x] Keyboard character callbacks precede `OnTextChanged(self, true)`; deletion emits only the changed callback. Unselected editing and forbidden scripted-input cursor guards remain intact.
- [x] XML `letters` and `bytes` populate the existing `MaxLetters` and `MaxBytes` fields before `OnLoad` in ordinary, inherited, and runtime-template construction, including nested runtime-template EditBoxes.
- [x] Each explicit XML value, including zero, overrides the most-derived inherited value independently. Preserve the declared integer verbatim; do not add terminator arithmetic.
- [x] Keyboard input respects positive `MaxLetters` (Unicode scalars) and `MaxBytes` (UTF-8 bytes) against the proposed selected-range replacement. Zero defaults are unlimited.
- [x] An overflowing input fragment is rejected entirely, preserving existing text, caret, selection and render caches. It emits no `OnChar` or `OnTextChanged`; accepted input retains the existing callback sequence.

- [ ] Positive `MaxLetters` truncates programmatic EditBox `SetText` (including the shared `SetFormattedText` path) to the configured Unicode-scalar prefix without splitting UTF-8. Non-EditBox text and unlimited values retain their existing behavior.
- [ ] Derive render text, clamp cursor/selection, and decide text callbacks from the final limited value. Different inputs that normalize to the same stored text retain the existing no-op lifecycle.
- [x] A changed EditBox `SetText` (including `SetFormattedText`) commits text, then clamps stored scalar cursor and selection endpoints to its length before callbacks; an already in-bounds cursor stays in place. Clipping is a simulator model invariant, not a native precedence claim.
- [x] Changed programmatic text dispatches `OnTextSet(self)` before `OnTextChanged(self, false)` through normal scripts and hooks. Callbacks see committed text and coherent byte/scalar cursor getters; handler errors reach the error handler and do not prevent later handlers or `OnTextChanged`.
- [x] Same-value EditBox text assignment is a simulator no-op: it does not dispatch this lifecycle, so a callback assigning its current text cannot recurse. It emits no `OnChar`.
- [x] Shortening text leaves subsequent `Insert` and Left/Backspace edits within bounds; non-EditBox text setters retain their existing text behavior.

Length overflow rejection is the user-selected simulator policy, not a native-client claim. No-edit callback suppression follows the existing numeric-input rejection policy; tests establish simulator consistency, not native callback equivalence.

## How it works

- [Widget system](../widget-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/editbox.rs` — public cursor conversions and `Insert`.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — shared text mutation and changed EditBox callback lifecycle.
- `src/lua_api/frame/methods/widgets/editbox/selection.rs` — highlighted byte endpoints and character-range storage.
- `src/lua_api/key_dispatch.rs` — selected-range keyboard replacement/deletion, pre-mutation length validation, and input callbacks.
- `src/widget/frame.rs` — selection consumption shared by public `Insert` and keyboard edits.
- `src/xml/types.rs` — optional EditBox `letters` and `bytes` declarations.
- `src/lua_api/globals/template/direct.rs` — shared independent `bytes`/`letters` resolution, invoked from ordinary XML setup and runtime-template property application before `OnLoad`.

## Tests asserting this spec

- `tests/editbox_stub_family.rs` — cursor units, selected replacement/deletion, render text, programmatic mutation and callbacks.
- `tests/editbox_stub_family/max_letters.rs` — recorded ASCII limit case, scalar/UTF-8 model consistency, clipped callback/cache observations and final-value no-op controls.
- `tests/key_dispatch.rs` — ASCII/Unicode selected typing, selection deletion, subsequent public Insert, callback observations, numeric rejection, and unselected controls.
- `tests/forbidden_aspect_creation.rs` — scripted-input cursor restrictions.
- `tests/xml_templates/inline_advanced/editbox_limits.rs` — XML/runtime-template propagation, independent overrides, OnLoad observations, and configured-limit keyboard rejection.

## Evidence boundary

- Programmatic MaxLetters tests `7542b1a56` are RED in three cases; unlimited/non-EditBox controls pass. `/tmp/cross-version-editbox-settext-limit-proof.md` records commands and revisions. `docs/wow-client-diff/WowBehaviorTest.lua` records `hello` and five letters on 2026-02-23; the matching installed fixture at `/syncthing/World of Warcraft/_retail_/Interface/Disabled Addons/WowBehaviorTest/tests_editbox.lua` sets MaxLetters to five before assigning `hello world`. No specific client-build or historical fixture-hash provenance was recovered. Unicode-scalar truncation extends the existing simulator `GetNumLetters` model beyond that ASCII observation; native markup/grapheme accounting remains unverified. Post-change verification is pending. Keyboard overflow still rejects rather than truncates, per the separate user-selected policy.

- XML-limit tests-only `5f94dd219` is RED in four cases; the ordinary `letters` control passes. Its pre-change evidence found unparsed `bytes` and missing runtime-template application for both limits. `547907429` now parses `bytes` and applies each explicit or inherited value independently in ordinary XML and runtime-template construction, including nested template children, before `OnLoad`. `/tmp/cross-version-editbox-xml-limits-proof.md` records the exact RED boundary. Independent verification passes 48 scoped cases: five XML-limit, 28 keyboard, 11 EditBox-family, one unchanged retail ColorPicker consumer, and three ColorSelect XML controls. Format/check and grouped integration compilation pass without warnings; changed-function readability has no introduced findings. `/tmp/cross-version-editbox-xml-limits-verification-ledger.md` records exact proof, the retained bare-environment ESCAPE nil-call diagnostic, and the deliberate text-set error control. Literal declared-value propagation is simulator behavior; native byte/terminator accounting remains unverified, with no adjustment inferred from the unchanged retail ColorPicker HexBox `bytes="7"` declaration.

- Keyboard-limit tests at `e114cafe0` are RED in three cases: append overflow, selected oversized replacement with no callbacks, and combined scalar limit. One zero-default control passes. `/tmp/cross-version-editbox-limits-proof.md` records commands and revision. Independent verification of `1933cb68c` passes 43 scoped tests (28 keyboard, 11 EditBox family, four scripted-focus guards), format/check and grouped integration compilation without compiler warnings; readability has no findings. `/tmp/cross-version-editbox-limits-verification-ledger.md` records exact proof. Render-cache preservation and negative-limit handling are source-inspected, not directly asserted by the new tests. Scoped logs retain the separate bare-environment ESCAPE nil-call diagnostic and deliberate text-set error control; no zero-Lua-error claim. This change is entirely simulator-side Rust; Blizzard Lua/XML and existing UI definitions are untouched.

- Selected keyboard editing was RED in five behavioral cases at `67beac8c0`; four existing unselected controls passed. Independent bounded verification of code `0c5d5f047` then passed 45 scoped invocations: 24 `key_dispatch::`, 11 EditBox-family, six focus, and four scripted-focus guards. `cargo fmt --check`, `cargo check`, and `cargo test --test integration --no-run` passed with zero compiler warnings. `/tmp/cross-version-editbox-key-selection-verification-ledger.md` records exact commands, source scope, and logs. The scoped test logs contain three expected existing Lua diagnostics: a bare-environment ESCAPE-to-`ToggleGameMenu` nil call, plus explicit text-set and focus callback-error controls; this is not a zero-Lua-error claim. Solarity's `edit_box_replacement_range` corroborates the replacement model, not native-client behavior.

- The supported public boundary is valid UTF-8 byte offsets. The simulator retains character indices internally for editing and selected ranges.
- Initial verification added `editbox_insert_clamps_cursor_after_text_shortens`, which was RED 0/1 in `/tmp/editbox-shortening-red.log`: after `SetText` shortened text without resetting stored cursor or selection indices, `Insert` clamped its byte edit range but derived the resulting scalar cursor from the stale logical index. `3b5ee8d55` instead derives that cursor from the actual clamped text prefix plus inserted scalar count. Independent verification at `3b5ee8d55` passes seven EditBox family cases plus four focus controls; 19 unchanged key-dispatch cases retain their prior proof. Format/check pass. Exact logs and source boundaries are in `/tmp/cross-version-editbox-text-position-proof.md`.
- The bounded SetText lifecycle tests were RED 2 failed / 2 passed before the fix and GREEN 11/11 in the EditBox family after implementation. Independent verification also passes 19 keyboard, four focus and three non-EditBox text controls, plus format/check without warnings; `/tmp/cross-version-editbox-settext-proof.md` records exact commands and source scope. Warcraft Wiki's `UIHANDLER_OnTextChanged` and `API:EditBox_GetText` are API-reference sources for this audit, not native execution; native callback, caret, selection, and IME lifecycle remains unverified.
- Cached Blizzard consumers corroborate byte-facing use: `Blizzard_AutoComplete/AutoComplete.lua:405` and `Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua:510` pass Lua `strlen` results to `HighlightText`; Classic `ChatFrameUtilOverrides` passes `GetCursorPosition()` to byte-indexed Lua `string.sub`. This is source-consumer corroboration, not a native probe.

## Out of scope

- Full input-system behavior, public `Insert` limit enforcement, `SetText` MaxBytes enforcement, native markup/grapheme counting, IME, clipboard, keyboard selection creation/navigation, and rendering selection highlight.
- Native same-value callback policy, caret placement/reset (including any always-end behavior), `OnCursorChanged`, IME lifecycle, and full native EditBox lifecycle remain unverified. The changed-value callbacks, no-op policy, and clipping above are bounded simulator behavior, not a native-client execution claim.
