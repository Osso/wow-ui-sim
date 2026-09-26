# SetParent parent cycles

Public `SetParent` rejects a proposed self or descendant parent before changing frame hierarchy. The source is `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs`.

## What it must do

- [x] Reject self-parenting and parenting to any descendant with a meaningful Lua error before mutation.
- [x] Preserve existing parent links, child enumeration, and visibility after rejection.
- [x] Allow ordinary reparenting, nil parent, and same-parent calls without duplicating child links.
- [x] Preserve existing protection and forbidden-aspect validation before cycle validation.

## How it works

- [Widget system](../widget-system.md)
- [Layout system](../layout-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/methods_hierarchy.rs` — checks the proposed parent's ancestry.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — rejects invalid public `SetParent` calls before mutation.

## Tests asserting this spec

- `tests/methods_hierarchy.rs` — public self/descendant rejection, preserved hierarchy and visibility, and valid controls.

## Known gaps (current cycle)

- [ ] Native WoW error wording remains unverified; the error text is simulator-defined.

## Out of scope

Anchor-cycle policy, internal trusted reparenting call sites, and repairs of preexisting malformed hierarchy graphs.
