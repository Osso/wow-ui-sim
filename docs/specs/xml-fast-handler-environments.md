# XML fast-handler environments

Generated fast XML handlers must use the same loading environment as the XML scripts they replace. This covers Retail secure-addon and scoped XML loading; it does not change Blizzard Lua.

## What it must do

- [ ] A fast handler created while loading a secure addon resolves globals from its secure environment.
- [ ] A scoped script environment, when supplied, takes precedence over the addon environment.
- [ ] Public-addon handlers retain the public environment.

## How it works

- [XML template system](../xml-template-system.md)

## Implementation inventory

- `src/lua_api/globals/create_frame/template_chain/builders.rs` prepares generated handler factories.

## Tests asserting this spec

- `tests/store_tree.rs::store_product_card_buy_button_click_completes_without_store_upvalue_error` exercises the original secure Store notice handler.

## Known gaps (current cycle)

- [ ] Deferred template instantiation and per-file environment overrides need independent proof.
- [ ] Scoped and public environment preservation need separate behavioral tests.

## Out of scope

Changes to Blizzard Store code and unrelated secure-environment policy are excluded.
