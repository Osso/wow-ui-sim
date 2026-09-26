# SetParent parent cycles

Public Lua `SetParent` rejects a proposed self or descendant parent before changing frame hierarchy. `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` invokes the shared `methods_hierarchy::would_create_parent_cycle` helper before animation reparenting or `apply_parent_change`.

## What it must do

- [x] Reject self-parenting and parenting to a descendant before mutation; the tested simulator error contains `cycle`.
- [x] Preserve existing parent links, child enumeration, and visibility after rejection.
- [x] Allow ordinary reparenting, nil parent, and same-parent calls without duplicating child links.
- [x] Preserve existing protection and forbidden-aspect validation before cycle validation.

## How it works

- [Widget system](../widget-system.md)
- [Layout system](../layout-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/methods_hierarchy.rs` — shared `would_create_parent_cycle` walks the proposed parent's `parent_id` ancestry; `reparent_widget` remains the mutation helper.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — public Lua `SetParent` retains its protected-state and forbidden-aspect guards, then calls the shared validator before animation reparenting and hierarchy mutation.

## Tests asserting this spec

- `tests/methods_hierarchy.rs` — public self/descendant rejection, preserved hierarchy and visibility, and valid controls.

## Known gaps (current cycle)

- [ ] Native WoW error wording remains unverified; only a simulator error containing `cycle` is tested.
- [ ] The helper assumes the existing `parent_id` chain is acyclic. It does not validate or repair every malformed model graph.
- [ ] Other parent writers are not covered by this public Lua `SetParent` boundary.

## Out of scope

Anchor-cycle policy, other parent writers, internal trusted reparenting call sites, and repairs of preexisting malformed hierarchy graphs.
