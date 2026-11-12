# EditBox text font, insets and justification

A `SearchBoxTemplate` box drew its text yellow, centered and against its left edge. The client draws it white, at the left, 16 px in. Three loader gaps caused it: an EditBox's `<FontString>` element was never applied, a new EditBox started at `CENTER`, and `<TextInsets>` were read only from the frame's own XML. One client quirk around the box's font file and size remains unmodeled.

## Content

### Client behavior

Checked in the WoW Forever client (1.60.1, no addons loaded, running under Wine). Forever and retail ship the same `SearchBoxTemplate` and `InputBoxVisualTemplate`.

| Call | Result |
|---|---|
| `CreateFrame("EditBox"):GetJustifyH()` | `LEFT` |
| `CreateFrame("EditBox"):GetFont()` | `nil, 0` |
| `CreateFrame("EditBox"):GetTextColor()` | `1, 1, 1, 1` |
| `CreateFrame("EditBox", nil, UIParent, "InputBoxTemplate"):GetFont()` | `Fonts\ARIALN.TTF, 14, ""` (ChatFontNormal) |
| `CreateFrame("EditBox", nil, UIParent, "InputBoxInstructionsTemplate"):GetFont()` | `Fonts\FRIZQT__.TTF, 10, ""` |
| `CreateFrame("EditBox", nil, UIParent, "SearchBoxTemplate")` | `GetJustifyH()` `LEFT`, `GetTextColor()` `1, 1, 1, 1`, `GetTextInsets()` `16, 20, 0, 0`, `GetFont()` `Fonts\FRIZQT__.TTF, 10, ""`, `GetFontObject():GetName()` `nil` |
| `InputBoxTemplate` box plus `CreateFontString(nil, "ARTWORK", "GameFontDisableSmall")` | `GetFont()` stays `Fonts\ARIALN.TTF, 14, ""` |
| `ChatFontNormal:GetFont()` | `Fonts\ARIALN.TTF, 14, ""` |

### Root causes and fix

- `FrameXml::font_string_child()` parsed an EditBox's `<FontString>` element, but nothing applied it. `src/loader/editbox_font.rs` now applies the element of the box or its nearest template on both the XML path (`xml_frame/finalize.rs`) and the runtime template path (`runtime_loader_effects.rs`): the font object it inherits, and the `justifyH`, `justifyV` and `<Color>` it names. The font object leaves the justification as it was unless the element names one, because font objects default to `CENTER` while the client keeps a `SearchBoxTemplate` box at `LEFT`.
- `Frame::new` gave an EditBox the FontString default `CENTER`; it now starts at `LEFT`.
- `apply_xml_text_insets` read only the frame's own XML. It now falls back to the nearest template in the chain, and the runtime template path applies it too.

### Open: the instructions region's font

`InputBoxTemplate` takes ChatFontNormal's font from its element, as modeled. `InputBoxInstructionsTemplate` adds an `Instructions` FontString region (`GameFontDisableSmall`, Friz Quadrata 10). With it, the client reports Friz Quadrata 10 for the box, but keeps the box's white color. A FontString created at runtime does not have this effect. The rule behind it (declaration order, region type, or something else) is unknown. The simulator keeps ChatFontNormal, so a search box measures its text at 14 instead of 10. The client's font object is unnamed; the simulator's is ChatFontNormal itself.

## Sources

- [editbox_font.rs](../../../src/loader/editbox_font.rs) — applies an EditBox's `<FontString>` element
- [direct.rs](../../../src/lua_api/globals/template/direct.rs) — `apply_xml_text_insets` with template fallback
- [editbox_text_defaults.rs](../../../tests/editbox_text_defaults.rs) — tests for the default, the element and the insets on both paths
- In-game `/dump` and `/run` checks in the WoW Forever client 1.60.1, listed above

## See Also

- [[xml-template-system]] — template chains the element and insets are resolved from
- [[widget-system]] — EditBox widget defaults
- [[rendering-pipeline]] — font sources
