# Inline XML method arguments

Inline XML method calls must preserve Lua operand values, order, identity, and argument count. This contract concerns simulator script installation, not the native GameTooltip API. See the [investigation](../wiki/investigations/xml-inline-method-arguments.md).

## What it must do

- [ ] Forward a global string, a color table, and a boolean as three distinct operands without expanding the table.
- [ ] Preserve reordered callback operands instead of substituting the callback's varargs.
- [ ] Preserve named parameter prefixes, including explicit `self` and optional varargs, during fast frame installation.

## How it works

- [Investigation](../wiki/investigations/xml-inline-method-arguments.md)
- [XML template system](../xml-template-system.md)

## Implementation inventory

- `src/lua_api/globals/create_frame/template_chain/parser.rs` — distinguishes parameter prefixes from unrelated identifiers.
- `src/lua_api/globals/create_frame/template_chain/builders.rs` — installs source-faithful prefix handlers.
- `src/loader/helpers.rs` — owns XML callback parameter names and implicit local bindings.

## Tests asserting this spec

- `tests/xml_templates/inline_flow/tooltip_sequences.rs::test_inline_tooltip_three_arguments_preserve_lua_values`
- `tests/xml_templates/inline_flow/tooltip_sequences.rs::test_inline_tooltip_reordered_handler_arguments`
- `tests/xml_templates/inline_flow/tooltip_sequences.rs::test_inline_tooltip_handler_parameter_prefixes`

## Known gaps (current cycle)

- [ ] Retail execution proof is pending.

## Out of scope

Native AddLine coercion and undocumented overloads are not established by a test that replaces GameTooltip with a plain Lua table. No Blizzard source changes are permitted.
