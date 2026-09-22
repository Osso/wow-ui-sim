# Merchant repair capability

Forever exposes `CanMerchantRepair` from `src/lua_api/globals/real/merchant_repair.rs` as a query of current merchant service state. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [x] Return exactly one boolean: `merchant_frame_open && merchant_repair_capable`. Closed merchants and open nonrepair merchants return false; a configured repair-capable open merchant returns true.
- [x] Default the distinct capability to false, an inferred simulator scenario rather than a native observed default. Opening a merchant does not imply repair capability.
- [x] Preserve existing merchant inventory, `CanMerchant`, open/close behavior, and environment independence. Closing the merchant makes the query false without changing the configured capability.
- [x] Retain query behavior after normal bootstrap restoration.
- [x] Actual cached Blizzard startup followed by `ClearBags`, `AddBagItem`, and `BAG_UPDATE` completes without Lua errors and leaves all three repair buttons hidden when no merchant is open.

Cached Forever `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:988` consumes `CanMerchantRepair()` as a predicate. Local cached legacy API documentation is absent; the conjunction and default are explicitly inferred policy, not native-verified semantics. The observed global is absent (`type == nil`); misleading stack labels do not establish a callable wrapper.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/merchant_repair.rs`: read-only capability predicate.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: distinct per-environment capability and inferred default.
- `tests/merchant_repair_capability.rs`: grouped behavioral and cached full-startup regression tests, discovered by the existing integration harness.

## Tests asserting this spec

- `tests/merchant_repair_capability.rs`: closed/open/configured/closed transitions, inventory and close-event preservation, environment isolation, bootstrap retention, and actual cached bag-event execution.

## Known gaps (current cycle)

- [x] Combined family verification reuses 11/11 focused tests, passes formatting/default offline checking and changed-Rust readability, and verifies the frozen artifact: `/tmp/forever-addon-audit/verify-merchant-read-family-ledger.json`.
- [x] The unchanged generic-TOC BagMeter replay and no-addons `BAG_UPDATE` control complete with empty Lua-error JSON after the required buyback and junk reads are added.
- [ ] A repairing merchant UI remains unproven: the default capability is false and no fixture exercises a configured repair-capable merchant through its other unmodeled UI calls.

## Out of scope

Repair actions, costs, durability, guild-bank repair APIs, events beyond existing close behavior, other-profile publication, native conformance, and full repairing-merchant UI. True capability is tested only as a query because other APIs needed by that UI remain unmodeled.
