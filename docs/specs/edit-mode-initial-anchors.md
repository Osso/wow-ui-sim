# Edit Mode initial anchors

Simulator Edit Mode startup replay in `src/lua_api/workarounds/editmode/apply_system_anchors.lua` must initialize registered system geometry before invoking cross-system callbacks, matching the pinned Blizzard manager's layout-apply ordering.

## What it must do

- [ ] Initialize registered system anchors before replaying system settings or final anchors.
- [ ] Let an earlier MicroMenu callback query an initially unanchored QueueStatus button without errors.
- [ ] Preserve the final saved QueueStatus anchor after initialization and replay.

## How it works

- [Ellesmere failure-time evidence](../ellesmereui-forever.md#queuestatus-initialization-ordering).
- [Layout system](../layout-system.md).

## Implementation inventory

- `src/lua_api/workarounds/editmode/apply_system_anchors.lua`: simulator startup replay uses native initial-anchor phase.

## Tests asserting this spec

- `tests/editmode_initial_anchors.rs`: actual Blizzard MicroMenu/QueueStatus consumers with registration order that reads QueueStatus before its own update.

## Known gaps (current cycle)

- [ ] Focused regression GREEN and integrated Ellesmere startup replay pending.

## Out of scope

- Fabricated `GetCenter()` results or addon/vendor source changes.
