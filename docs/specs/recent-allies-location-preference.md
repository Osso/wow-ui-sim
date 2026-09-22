# Recent allies location preference

Forever exposes a per-environment location preference through two real Lua globals in `src/lua_api/globals/real/recent_allies_location.rs`. See [event dispatch](../event-system.md) for delivery infrastructure.

## What it must do

- [x] Expose `GetAllowRecentAlliesSeeLocation()` returning a boolean and `SetAllowRecentAlliesSeeLocation(bool)` returning no values, only under Forever.
- [x] Start at `true`; preserve false/true changes across bootstrap restoration and isolate separate simulator environments. The default follows cached `Blizzard_SettingsDefinitions_Frame/Social.lua`, not native observation.
- [x] Commit changed state before synchronously delivering `LET_RECENT_ALLIES_SEE_LOCATION_SETTING_UPDATED` with no payload, through ordered existing frame handlers.
- [x] Suppress same-value notifications, including Settings-style feedback. This is an inferred policy motivated by the cached listener writing the getter value back through `setting:SetValue`.
- [x] Require a boolean using the existing strict typed-value convention. Reuse VM secret-argument validation: untainted callers can provide a wrapped boolean; tainted callers cannot. Public booleans remain usable by tainted addon code; rejected input changes neither state nor event count. Exact native coercions/errors remain unverified.

Cached Forever `PlayerScriptDocumentation.lua` documents the getter and required boolean setter (`SecretArguments = AllowedWhenUntainted`). `RecentAlliesDocumentation.lua` declares the event synchronous without a payload. Cached Account-wide UI `8935141` saves the getter at `SaveFunction.lua:457` and conditionally restores false as well as true at `LoadFunction.lua:786`.

## How it works

- [Event system](../event-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/recent_allies_location.rs`: getter, validation, setter and synchronous notification.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only module and registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment boolean and initial value.

## Tests asserting this spec

`tests/recent_allies_location_preference.rs`, included by the grouped integration target: round trip; synchronous ordered payload-free events and feedback; environment/bootstrap retention; malformed and secure/tainted arguments; Account-wide UI-shaped save/change/load.

## Known gaps (current cycle)

Targeted development proof: `561943dd0` tests and `37da0f132` runtime pass 5/5 grouped tests after two identical Lua probes failed on the frozen pre-change simulator. Commands, source hashes, build features, and frozen executable hash: `/tmp/forever-addon-audit/location-preference-development-ledger.json`.
- [ ] Parent-owned unchanged-addon full save/load replay and independent verification.

## Out of scope

- Cross-process persistence, networking and actual ally location visibility: no persistence or world-sharing contract established.
- Other client profiles and unrelated preferences: existing behavior preserved.
- Native conformance, generic declassification, or addon option changes: unsupported by available evidence.
