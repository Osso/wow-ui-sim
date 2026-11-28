# Perks activities state

Retail and WoW Forever expose a host-supplied Traveler's Log snapshot through `C_PerksActivities`. The Rust model lives in `src/c_api/perks_activities.rs`; see the [Lua API architecture](../lua-api.md) for the API boundary.

## What it must do

- [x] Accept a complete JSON snapshot through `A_Admin.SetPerksActivities`, reject malformed or invalid input, and keep the prior snapshot after rejection.
- [x] Publish supplied activity rows through the documented `C_PerksActivities` getters.
- [x] Let the selected original ObjectiveTracker display two distinct supplied activity names and remove one through the public tracking API.
- [ ] Keep native snapshot defaults, activity update notifications, and account data outside the host-supplied model until evidence defines them.

## How it works

- [Lua API architecture](../lua-api.md)
- [Admin input](../admin-api/perks-activities.md)

## Implementation inventory

- `src/c_api/perks_activities.rs` — snapshot types and per-profile model availability.
- `src/c_api/perks_activities/admin.rs` — validated host snapshot input.
- `src/c_api/perks_activities/lua.rs` — documented API publication.
- `src/lua_api/globals/admin.rs` — A_Admin registration.
- `src/lua_api/workarounds/temporary/perks_activities_state.rs` — legacy-profile compatibility state.

## Tests asserting this spec

- `tests/perks_activities_original_tracker.rs::original_tracker_displays_and_selectively_untracks_two_activities`

## Known gaps (current cycle)

- [ ] Native account data, absent-snapshot results, and activity-refresh event timing remain unknown.
- [ ] The input contract is simulator-only and does not model a server request or response.

## Out of scope

Live Traveler's Log rotation, server networking, and native event lifecycle are not modeled.
