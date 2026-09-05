# Client Profiles

Client profile bundles select the runtime cache and API epoch exposed by wow-ui-sim. Source lives in `Cargo.toml` and `src/client_profile.rs`; see [Client Profiles](../wiki/systems/client-profiles.md) for architecture and loader details.

## What it must do

- [x] The public `client-retail` bundle selects the retail profile and current retail 12.1.0 API epoch (`120100`).
- [x] Retail API epoch features remain cumulative, so 12.1.0 includes the modeled 12.0.0, 12.0.5, and 12.0.7 surfaces.
- [x] Historical retail epochs remain selectable through `profile-retail` without enabling the current-retail bundle.
- [x] `client-ptr` remains a distinct profile/cache while selecting the cumulative 12.1.5 API epoch (`120105`).
- [x] PTR CASC resolves the official `wowxptr` product; 12.1.5.69594 uses BuildConfig `4a9973f37906f8cfb344f8a9fe6777e0` and Gethe `ptr2` source commit `49b69918fcdc77e109813281e4f537d45ec7dcbf`.
- [x] Same-epoch profiles may have source-proven post-startup removals: retail 12.1 keeps `C_RecruitAFriend.IsEnabled`, while PTR hides it after startup.
- [x] Default-retail Lua initialization publishes the probe-backed retail 12.1 global-string contract.

## How it works

- [Client profile architecture](../wiki/systems/client-profiles.md)
- [Lua API registration](../wiki/systems/lua-api.md)

## Implementation inventory

- `Cargo.toml` — cumulative retail epoch features and public client bundles; `retail-12-1-5` extends `retail-12-1-0` for PTR.
- `src/client_profile.rs` — active profile/epoch selection and interface constants.
- `src/asset_resolver_config.rs` — profile-to-CASC-product mapping (`client-ptr` uses `wowxptr`).
- `src/ptr/strict_removals.lua` — PTR-only post-startup removals, including `C_RecruitAFriend.IsEnabled`.
- `src/lua_api/globals/strings/mod.rs` — epoch-gated retail string registration.
- `src/lua_api/globals/strings/string_data/more_strings.rs` — probe-backed retail 12.1 values.

## Tests asserting this spec

- `src/client_profile.rs` — current retail, historical retail, PTR, and interface-version contracts.
- `src/loader/tests/wow_api_globals/startup_globals.rs` — post-startup strict-removal contract, including PTR-only `C_RecruitAFriend.IsEnabled` removal.
- `tests/blizzard_recruit_a_friend_loads.rs` — retail `C_RecruitAFriend.IsEnabled` availability and behavior.
- `src/lua_api/globals/register.rs` — exact retail 12.1 string values and intentional nil globals.

## Known gaps (current cycle)

None.

## Out of scope

- Changing PTR or classic-profile cache selection, manifests, generators, or cache contents.
- Removing historical retail API epochs.
- Modifying Blizzard UI cache files or committed source manifests as part of the channel promotion.
