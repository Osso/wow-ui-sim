# Retail 12.1.0 events — C06

Bounded simulator proof for [prose-2026-06-23-083; prose-2026-06-23-088](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L83: EventRegistrations rejects seven mutation operations for secure and tainted callers; existing individual/unit/all listeners and callbacks retain queries and actual event delivery; unrestricted controls continue to work.
- [ ] L88: real AuraContainer/CustomAuraContainerTemplate has forbidden mask 0, failing intrinsic EventRegistrations assertion. Failing test excluded from commit; full reproduction and logs in persistent result file. Recommend modelable / audit-pending, not bounded.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/events.rs — event registration restriction`
- `cached Blizzard_AuraContainer/Blizzard_AuraContainer.xml — intrinsic template lacks EventRegistrations declaration`

## Tests asserting this spec

- `tests/forbidden_aspect_creation.rs — five event_registration_aspect tests`

## Known gaps (current cycle)

- [ ] L88: real AuraContainer/CustomAuraContainerTemplate has forbidden mask 0, failing intrinsic EventRegistrations assertion. Failing test excluded from commit; full reproduction and logs in persistent result file. Recommend modelable / audit-pending, not bounded.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
