# FontString Default Anchors

Unanchored XML `FontString` layer children do not always default to the parent center. Retail WoW gives them one implicit anchor selected from `justifyH`: `LEFT`, `CENTER`, or `RIGHT`; `justifyV` does not affect that implicit point.

## Content

The simulator previously generated `fs:SetPoint("CENTER", parent, "CENTER", 0, 0)` for every XML `FontString` without explicit `<Anchors>` and without `setAllPoints="true"`. A real-client addon test using nine `justifyH` x `justifyV` permutations showed the actual behavior:

- `justifyH="LEFT"` -> one point: `LEFT` relative to parent `LEFT`, offset `0, 0`
- `justifyH="CENTER"` -> one point: `CENTER` relative to parent `CENTER`, offset `0, 0`
- `justifyH="RIGHT"` -> one point: `RIGHT` relative to parent `RIGHT`, offset `0, 0`
- `justifyV="TOP"`, `"MIDDLE"`, and `"BOTTOM"` leave the implicit anchor unchanged

This is not theoretical. The retail Blizzard UI manifest contains many unanchored XML `FontString`s with direct `justifyH="LEFT"` or `justifyH="RIGHT"` attributes, including Catalog Shop product text, Encounter Timeline labels, UI widget text templates, and shared tooltip/text templates. Anchoring all of those at center shifts real Blizzard text.

The XML code generator now chooses the default `SetPoint` point from the parsed `justify_h` field and falls back to `CENTER` when `justifyH` is absent or unrecognized.

Follow-up retail probes showed the same rule applies to a `Button`'s XML `ButtonText` child. The simulator must not pre-seed unanchored `ButtonText` with a hard-coded `CENTER` anchor; it should flow through the same XML `FontString` default-anchor generation so `justifyH="LEFT"` and `justifyH="RIGHT"` produce `LEFT` and `RIGHT` implicit points.

The rule is based on the text region having no anchors at all, not on whether it lacks a horizontal anchor. Retail cases with only `TOP` or only `BOTTOM` anchors kept exactly that one explicit point and did not receive an extra `LEFT`/`CENTER`/`RIGHT` point from `justifyH`.

EditBox internals are different. Retail `EditBox:GetRegions()` exposes a backing `FontString`, but that backing region remains unanchored (`GetNumPoints() == 0`) even when its XML child uses `justifyH="RIGHT"`. XML `TextInsets` apply to the EditBox owner and do not anchor the backing FontString. In the simulator, EditBox's synthetic `Text` region must therefore remain unanchored, while XML `TextInsets` still need to populate the frame's `editbox_text_insets`.

MessageFrame and ScrollingMessageFrame probes did not expose FontString regions through `GetRegions()` for this default-anchor path. Owner frames kept `GetNumPoints() == 0`; no matching default-anchor behavior was visible there.

## Sources

- [xml_fontstring.rs](../../../src/loader/xml_fontstring.rs) - default XML `FontString` anchor code generation
- [button.rs](../../../src/loader/button.rs) - `ButtonText` creation path now relies on generic `FontString` default-anchor generation
- [helpers_shared.rs](../../../src/lua_api/globals/create_frame/helpers_shared.rs) - EditBox synthetic `Text` region is created without slider-label anchors
- [direct.rs](../../../src/lua_api/globals/template/direct.rs) - XML `TextInsets` application to frame state
- [xml_basics_extra.rs](../../../src/loader/tests/xml_basics_extra.rs) - regression test covering all nine `justifyH` x `justifyV` permutations
- [xml_text_region_defaults.rs](../../../src/loader/tests/xml_text_region_defaults.rs) - regression tests for `ButtonText`, explicit anchor preservation, and EditBox `TextInsets`
- [data/blizzard-ui-files/retail.txt](../../../data/blizzard-ui-files/retail.txt) - retail Blizzard UI manifest used to confirm real usage

## See Also

- [[xml-template-system]] - XML parsing and code generation overview
- [[layout-system]] - anchor resolution behavior
- [[fontstring-anchor-sizing]] - inferred width from opposite-edge anchors
