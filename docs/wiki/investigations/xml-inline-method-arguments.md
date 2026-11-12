# Inline XML method operand loss

VERIFIED: an inline tooltip test exposed a parser defect and a separate fixture mismatch. The defect was in simulator XML script installation, before GameTooltip API dispatch. The [contract](../../specs/xml-inline-method-arguments.md) separates Lua operand preservation from native tooltip argument validation.

## Root cause

VERIFIED: the old `is_fast_passthrough_args` accepted every identifier, including `true`, `false`, and `nil`. `GameTooltip:AddLine(ALL_ASSIST_NOT_LEADER_ERROR, RED_FONT_COLOR, true)` therefore entered a generated passthrough handler instead of executing the three explicit expressions. The handler called the Lua spy with its own callback varargs: zero values for the tested OnEnter invocation. A direct Lua invocation of the same call supplied the string, the color table, and the boolean.

VERIFIED: the test receiver was a plain Lua table. Its AddLine function recorded its arguments without color expansion. INFERRED: expecting five expanded values from that three-operand call was a fixture error independent of native tooltip coercion.

## Source fidelity

VERIFIED: pinned Retail `Blizzard_RaidFrame/Mainline/RaidFrame.xml:175` supplies `ALL_ASSIST_NOT_LEADER_ERROR, RED_FONT_COLOR.r, RED_FONT_COLOR.g, RED_FONT_COLOR.b, true` explicitly. The addon TOC permits mainline and selects this XML file. VERIFIED: `Blizzard_SharedXML/SharedTooltipTemplates.lua:166-171` expands `color:GetRGB()` in `GameTooltip_AddColoredLine` before invoking `tooltip:AddLine(text, r, g, b, wrap, leftOffset)`.

VERIFIED: no `AddLine` entry was found in the pinned Retail `Blizzard_APIDocumentationGenerated` Lua files, including `FrameAPITooltipDocumentation.lua`. UNKNOWN: native handling of a color-table overload or invalid operands. Those native details are not needed to establish what the plain Lua spy must receive.

## Correction

The catch-all predicate no longer accepts arbitrary identifier operands. Simple parameter-prefix calls retain fast frame installation through a cached handler compiled with the same callback parameters and implicit local bindings as normal XML. The original expression list is retained, so explicit self, count, and varargs remain distinct. Reordered or unrelated operands use the existing Lua compilation path.

The coordinator approved a separate master-assertion correction after the nil cause was established: keep the original test name and attribute, and use the five operands from the pinned RaidFrame consumer. Execution results are recorded in the lane handoff report, not inferred from source inspection.

## Sources

- [Parser](../../../src/lua_api/globals/create_frame/template_chain/parser.rs)
- [Fast handler installation](../../../src/lua_api/globals/create_frame/template_chain/builders.rs)
- [Regression tests](../../../tests/xml_templates/inline_flow/tooltip_sequences.rs)
- [XML callback parameters](../../../src/loader/helpers.rs)

## See Also

- [XML template system](../../xml-template-system.md)
- [XML local self contract](../../specs/template-fast-path-self.md)
