# EditBox text position and selection

Shared EditBox cursor and highlighted-range methods expose bounded text-position behavior. This spec covers valid UTF-8 byte boundaries and selected-range `Insert`, not the entire input system.

## What it must do

- [x] `SetCursorPosition` accepts valid UTF-8 byte offsets; `GetCursorPosition` returns the byte offset and `GetUTF8CursorPosition` returns the Unicode scalar count before the cursor.
- [x] Cursor positions past the end clamp to the end; `Insert` at the cursor preserves UTF-8 and advances both public getters consistently.
- [x] `HighlightText` with valid byte endpoints selects a range; `Insert` replaces that range (including empty-string deletion), collapses the cursor after inserted text, and clears selection for the next insertion.
- [x] ASCII cursor and selected-range editing retain their original units; renderer-facing stripped text remains synchronized with inserted text.
- [x] `Insert` derives its resulting cursor from the actual clamped edit range, including when earlier text replacement left cursor or selection indices beyond the new text.
- [x] Existing keyboard typing and forbidden scripted-input cursor guard remain intact.

## How it works

- [Widget system](../widget-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/editbox.rs` — public cursor conversions and `Insert`.
- `src/lua_api/frame/methods/widgets/editbox/selection.rs` — highlighted byte endpoints and character-range storage.
- `src/lua_api/key_dispatch.rs` — existing character-based keyboard editing (unchanged).

## Tests asserting this spec

- `tests/editbox_stub_family.rs` — cursor units, selected replacement/deletion, render text.
- `tests/key_dispatch.rs` — keyboard editing controls.
- `tests/forbidden_aspect_creation.rs` — scripted-input cursor restrictions.

## Evidence boundary

- The supported public boundary is valid UTF-8 byte offsets. The simulator retains character indices internally for editing and selected ranges.
- Initial verification added `editbox_insert_clamps_cursor_after_text_shortens`, which was RED 0/1 in `/tmp/editbox-shortening-red.log`: after `SetText` shortened text without resetting stored cursor or selection indices, `Insert` clamped its byte edit range but derived the resulting scalar cursor from the stale logical index. `3b5ee8d55` instead derives that cursor from the actual clamped text prefix plus inserted scalar count. Independent verification at `3b5ee8d55` passes seven EditBox family cases plus four focus controls; 19 unchanged key-dispatch cases retain their prior proof. Format/check pass. Exact logs and source boundaries are in `/tmp/cross-version-editbox-text-position-proof.md`.
- Cached Blizzard consumers corroborate byte-facing use: `Blizzard_AutoComplete/AutoComplete.lua:405` and `Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua:510` pass Lua `strlen` results to `HighlightText`; Classic `ChatFrameUtilOverrides` passes `GetCursorPosition()` to byte-indexed Lua `string.sub`. This is source-consumer corroboration, not a native probe.

## Out of scope

- Full input-system behavior, limits, IME, clipboard, keyboard selection mutation, and rendering selection highlight.
- This slice does not normalize intermediate cursor/selection state at `SetText` itself or establish its full native lifecycle. It corrects `Insert`'s resulting cursor after clamped editing.
