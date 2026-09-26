# EditBox text position and selection

Shared EditBox cursor and highlighted-range methods expose bounded text-position behavior. This spec covers valid UTF-8 byte boundaries, selected-range `Insert`, and changed programmatic text replacement, not the entire input system.

## What it must do

- [x] `SetCursorPosition` accepts valid UTF-8 byte offsets; `GetCursorPosition` returns the byte offset and `GetUTF8CursorPosition` returns the Unicode scalar count before the cursor.
- [x] Cursor positions past the end clamp to the end; `Insert` at the cursor preserves UTF-8 and advances both public getters consistently.
- [x] `HighlightText` with valid byte endpoints selects a range; `Insert` replaces that range (including empty-string deletion), collapses the cursor after inserted text, and clears selection for the next insertion.
- [x] ASCII cursor and selected-range editing retain their original units; renderer-facing stripped text remains synchronized with inserted text.
- [x] `Insert` derives its resulting cursor from the actual clamped edit range, including when earlier text replacement left cursor or selection indices beyond the new text.
- [x] Existing keyboard typing and forbidden scripted-input cursor guard remain intact.
- [x] Changed EditBox `SetText` (including `SetFormattedText`) commits the new text and clamps stored scalar cursor and selection endpoints to its length before callbacks; an already in-bounds cursor stays in place. Clipping a selection is a simulator model invariant, not a native precedence claim.
- [x] Changed programmatic text fires ordered `OnTextSet(self)` then `OnTextChanged(self, false)` through script bindings and error routing; callbacks see the committed text and coherent byte/scalar cursor getters. Same-text reentry does not recurse indefinitely; no `OnChar` is emitted.
- [x] Shortening text leaves subsequent `Insert` and Left/Backspace edits within bounds; non-EditBox text setters retain their existing text behavior.

## How it works

- [Widget system](../widget-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/editbox.rs` — public cursor conversions and `Insert`.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — shared text mutation and changed EditBox callback lifecycle.
- `src/lua_api/frame/methods/widgets/editbox/selection.rs` — highlighted byte endpoints and character-range storage.
- `src/lua_api/key_dispatch.rs` — existing character-based keyboard editing (unchanged).

## Tests asserting this spec

- `tests/editbox_stub_family.rs` — cursor units, selected replacement/deletion, render text, programmatic mutation and callbacks.
- `tests/key_dispatch.rs` — keyboard editing controls.
- `tests/forbidden_aspect_creation.rs` — scripted-input cursor restrictions.

## Evidence boundary

- The supported public boundary is valid UTF-8 byte offsets. The simulator retains character indices internally for editing and selected ranges.
- Initial verification added `editbox_insert_clamps_cursor_after_text_shortens`, which was RED 0/1 in `/tmp/editbox-shortening-red.log`: after `SetText` shortened text without resetting stored cursor or selection indices, `Insert` clamped its byte edit range but derived the resulting scalar cursor from the stale logical index. `3b5ee8d55` instead derives that cursor from the actual clamped text prefix plus inserted scalar count. Independent verification at `3b5ee8d55` passes seven EditBox family cases plus four focus controls; 19 unchanged key-dispatch cases retain their prior proof. Format/check pass. Exact logs and source boundaries are in `/tmp/cross-version-editbox-text-position-proof.md`.
- The bounded SetText lifecycle tests were RED 2 failed / 2 passed before the fix and GREEN 11/11 in the EditBox family after implementation; `/tmp/cross-version-editbox-settext-proof.md` records exact commands. Warcraft Wiki's `OnTextChanged` and `EditBox:GetText` pages document changed programmatic text, handler order and new text visibility, but are external documentation, not native execution.
- Cached Blizzard consumers corroborate byte-facing use: `Blizzard_AutoComplete/AutoComplete.lua:405` and `Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua:510` pass Lua `strlen` results to `HighlightText`; Classic `ChatFrameUtilOverrides` passes `GetCursorPosition()` to byte-indexed Lua `string.sub`. This is source-consumer corroboration, not a native probe.

## Out of scope

- Full input-system behavior, limits, IME, clipboard, keyboard selection mutation, and rendering selection highlight.
- Same-text native callback policy, native caret placement/reset, `OnCursorChanged`, and full native EditBox lifecycle remain unverified. Changed-text callbacks and clipping are bounded simulator behavior, not a native-client execution claim.
