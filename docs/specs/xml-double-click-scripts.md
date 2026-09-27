# XML double-click script registration

XML `OnDoubleClick` declarations must survive optimized script installation. Source: `src/lua_api/globals/create_frame/template_chain.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [x] Install method-bound and inline `OnDoubleClick` handlers for ordinary XML instances and runtime-created template instances.
- [x] Return callable handlers through public `GetScript`; invoking those handlers forwards supplied arguments and produces the declared side effects.
- [x] Preserve other script/mixin registration in the same declaration.

## How it works

- [XML template system](../xml-template-system.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/create_frame/template_chain.rs`: includes `OnDoubleClick` when collecting handlers for optimized installation and determining whether general compilation is needed.
- `src/loader/helpers.rs`: existing general `OnDoubleClick` handler generation, unchanged.

## Tests asserting this spec

`tests/xml_templates/inline_advanced/double_click_scripts.rs`, inside the existing grouped integration target, checks ordinary method/inline declarations, runtime templates, callable effects and an `OnEnter` mixin control.

## Known gaps (current cycle)

Tests-only `0d052b4b2` is RED in three cases; the independent `OnEnter` control works. The collector omitted `OnDoubleClick` and could report successful installation with an empty list, preventing general generation from running. `/tmp/cross-version-doubleclick-xml-proof.md` records exact commands and results. Independent verification of `4d42cb803` passes 11 focused cases: three new registration/invocation cases, three XML/mixin/taint controls, and five shared ScrollFrame-binding controls. Format/check and grouped integration compilation pass without warnings. The scroll error-continuation sentinel remains an intentional diagnostic. `/tmp/cross-version-doubleclick-xml-verification-ledger.md` records exact scope; the existing template-chain file-size cap remains unchanged in classification.

## Out of scope

Physical double-click detection/input dispatch, native event argument conventions, all-profile/native parity, other handler expansion, and Blizzard/vendor changes. Calling a returned Lua handler is registration/invocation proof, not mouse-input proof.
