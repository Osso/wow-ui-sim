# EditBox text position and selection

Shared EditBox cursor and highlighted-range methods expose bounded text-position behavior. This spec covers valid UTF-8 byte boundaries and selected-range `Insert`, not the entire input system.

## What it must do

- [x] `SetCursorPosition` accepts valid UTF-8 byte offsets; `GetCursorPosition` returns the byte offset and `GetUTF8CursorPosition` returns the Unicode scalar count before the cursor.
- [x] Cursor positions past the end clamp to the end; `Insert` at the cursor preserves UTF-8 and advances both public getters consistently.
- [x] `HighlightText` with valid byte endpoints selects a range; `Insert` replaces that range (including empty-string deletion), collapses the cursor after inserted text, and clears selection for the next insertion.
- [x] ASCII cursor and selected-range editing retain their original units; renderer-facing stripped text remains synchronized with inserted text.
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

## Known gaps (current cycle)

- [ ] Native behavior for byte offsets inside a UTF-8 scalar and grapheme-cluster boundaries is unverified; simulator maps non-boundary offsets to the preceding scalar boundary.
- [ ] Native `Insert` callback timing and `OnTextChanged` userInput flag are unverified; this slice does not change callback dispatch.

## Out of scope

- Full `Insert` faithfulness, limits, IME, clipboard, keyboard selection mutation, and rendering selection highlight: no complete native contract or test in this slice.
- Native verification of UTF-8 selection endpoint units: cached Blizzard `Blizzard_AutoComplete/AutoComplete.lua:405` and `Blizzard_ChatFrameBase/Shared/ChatFrameEditBox.lua:510` pass `strlen` byte offsets to `HighlightText`, corroborating byte-facing endpoints without a native UTF-8 selection probe.
