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
- [x] Keyboard input respects positive `MaxLetters` (Unicode scalars) and `MaxBytes` (UTF-8 bytes) against the proposed selected-range replacement. Nonpositive limits are unlimited.
- [x] An overflowing input fragment is rejected entirely, preserving existing text, caret, selection and render caches. It emits no `OnChar` or `OnTextChanged`; accepted input retains the existing callback sequence.

Length overflow rejection is the user-selected simulator policy, not a native-client claim. No-edit callback suppression follows the existing numeric-input rejection policy; tests establish simulator consistency, not native callback equivalence.
- [x] A changed EditBox `SetText` (including `SetFormattedText`) commits text, then clamps stored scalar cursor and selection endpoints to its length before callbacks; an already in-bounds cursor stays in place. Clipping is a simulator model invariant, not a native precedence claim.
- [x] Changed programmatic text dispatches `OnTextSet(self)` before `OnTextChanged(self, false)` through normal scripts and hooks. Callbacks see committed text and coherent byte/scalar cursor getters; handler errors reach the error handler and do not prevent later handlers or `OnTextChanged`.
- [x] Same-value EditBox text assignment is a simulator no-op: it does not dispatch this lifecycle, so a callback assigning its current text cannot recurse. It emits no `OnChar`.
- [x] Shortening text leaves subsequent `Insert` and Left/Backspace edits within bounds; non-EditBox text setters retain their existing text behavior.

## How it works

- [Widget system](../widget-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/editbox.rs` — public cursor conversions and `Insert`.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — shared text mutation and changed EditBox callback lifecycle.
- `src/lua_api/frame/methods/widgets/editbox/selection.rs` — highlighted byte endpoints and character-range storage.
- `src/lua_api/key_dispatch.rs` — selected-range keyboard replacement/deletion, pre-mutation length validation, and input callbacks.
- `src/widget/frame.rs` — selection consumption shared by public `Insert` and keyboard edits.

## Tests asserting this spec

- `tests/editbox_stub_family.rs` — cursor units, selected replacement/deletion, render text, programmatic mutation and callbacks.
- `tests/key_dispatch.rs` — ASCII/Unicode selected typing, selection deletion, subsequent public Insert, callback observations, numeric rejection, and unselected controls.
- `tests/forbidden_aspect_creation.rs` — scripted-input cursor restrictions.

## Evidence boundary

- Keyboard-limit tests at `e114cafe0` are RED in three cases: append overflow, selected oversized replacement with no callbacks, and combined scalar limit. One zero-default control passes. `/tmp/cross-version-editbox-limits-proof.md` records commands and revision. Independent post-change verification is pending. This change is entirely simulator-side Rust; Blizzard Lua/XML and existing UI definitions are untouched.

- Selected keyboard editing was RED in five behavioral cases at `67beac8c0`; four existing unselected controls passed. Independent bounded verification of code `0c5d5f047` then passed 45 scoped invocations: 24 `key_dispatch::`, 11 EditBox-family, six focus, and four scripted-focus guards. `cargo fmt --check`, `cargo check`, and `cargo test --test integration --no-run` passed with zero compiler warnings. `/tmp/cross-version-editbox-key-selection-verification-ledger.md` records exact commands, source scope, and logs. The scoped test logs contain three expected existing Lua diagnostics: a bare-environment ESCAPE-to-`ToggleGameMenu` nil call, plus explicit text-set and focus callback-error controls; this is not a zero-Lua-error claim. Solarity's `edit_box_replacement_range` corroborates the replacement model, not native-client behavior.

- The supported public boundary is valid UTF-8 byte offsets. The simulator retains character indices internally for editing and selected ranges.
- Initial verification added `editbox_insert_clamps_cursor_after_text_shortens`, which was RED 0/1 in `/tmp/editbox-shortening-red.log`: after `SetText` shortened text without resetting stored cursor or selection indices, `Insert` clamped its byte edit range but derived the resulting scalar cursor from the stale logical index. `3b5ee8d55` instead derives that cursor from the actual clamped text prefix plus inserted scalar count. Independent verification at `3b5ee8d55` passes seven EditBox family cases plus four focus controls; 19 unchanged key-dispatch cases retain their prior proof. Format/check pass. Exact logs and source boundaries are in `/tmp/cross-version-editbox-text-position-proof.md`.
- The bounded SetText lifecycle tests were RED 2 failed / 2 passed before the fix and GREEN 11/11 in the EditBox family after implementation. Independent verification also passes 19 keyboard, four focus and three non-EditBox text controls, plus format/check without warnings; `/tmp/cross-version-editbox-settext-proof.md` records exact commands and source scope. Warcraft Wiki's `UIHANDLER_OnTextChanged` and `API:EditBox_GetText` are API-reference sources for this audit, not native execution; native callback, caret, selection, and IME lifecycle remains unverified.
- Cached Blizzard consumers corroborate byte-facing use: `Blizzard_AutoComplete/AutoComplete.lua:405` and `Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua:510` pass Lua `strlen` results to `HighlightText`; Classic `ChatFrameUtilOverrides` passes `GetCursorPosition()` to byte-indexed Lua `string.sub`. This is source-consumer corroboration, not a native probe.

## Out of scope

- Full input-system behavior, public `Insert`/`SetText` limit enforcement, IME, clipboard, keyboard selection creation/navigation, and rendering selection highlight.
- Native same-value callback policy, caret placement/reset (including any always-end behavior), `OnCursorChanged`, IME lifecycle, and full native EditBox lifecycle remain unverified. The changed-value callbacks, no-op policy, and clipping above are bounded simulator behavior, not a native-client execution claim.
