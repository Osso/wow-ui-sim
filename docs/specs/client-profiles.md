# Client Profiles

Client profile bundles select the runtime cache and API epoch exposed by wow-ui-sim. Source lives in `Cargo.toml` and `src/client_profile.rs`; see [Client Profiles](../wiki/systems/client-profiles.md) for architecture and loader details.

## What it must do

- [x] The public `client-retail` bundle selects the retail profile and current retail 12.1.0 API epoch (`120100`).
- [x] Retail API epoch features remain cumulative, so 12.1.0 includes the modeled 12.0.0, 12.0.5, and 12.0.7 surfaces.
- [x] Historical retail epochs remain selectable through `profile-retail` without enabling the current-retail bundle.
- [x] `client-ptr` remains a distinct profile/cache while selecting the cumulative 12.1.5 API epoch (`120105`).
- [x] PTR CASC resolves the official `wowxptr` product; 12.1.5.69594 uses BuildConfig `4a9973f37906f8cfb344f8a9fe6777e0` and Gethe `ptr2` source commit `49b69918fcdc77e109813281e4f537d45ec7dcbf`.
- [x] PTR `GetBuildInfo()` publishes `12.1.5`, build `69594`, and interface `120105` from the pinned source identity; non-PTR builds retain temporary interface `120100` with `retail-12-1-0`, otherwise `120007`.
- [x] PTR 12.1.5 publishes `Enum.CurioRarity.EpicTier2 = 5` and metadata through 5; earlier retail epochs retain the four-value contract.
- [x] Same-epoch profiles may have source-proven post-startup removals: retail 12.1 keeps `C_RecruitAFriend.IsEnabled`, while PTR hides it after startup.
- [x] Default-retail Lua initialization publishes the probe-backed retail 12.1 global-string contract.
- [x] `client-wowforever` selects a distinct `WowForever` profile with interface `16001`, cache `wowforever`, CASC product `wow_classic_beta`, and `_classic_beta_` install paths.
- [x] Forever resolves `[Family]` to `Mainline` and `[Game]` to `Camelot`, accepting `mainline`/`camelot` annotations, not `classic`, `standard`, or `vanilla`; matching exclusion annotations suppress files and dependencies. This follows the authenticated 1.60.1.69913 source tree, not the product name: ActionBar's mainline annotations select present `AssistedCombatManager.lua` and `ActionButtonOverrides.lua`, while its classic annotation selects absent `Mainline/MainMenuBar.lua`.
- [x] Forever selects Camelot variants, generic `.toc`, then Mainline variants. Each flavor tier supports underscore then hyphen spellings; retaining underscore precedence is simulator policy, not a claim about native tie-order when both exist. Mainline-only load-on-demand dependencies participate in discovery and dependency ordering; no WorldMap-specific exception is used.
- [x] Supported TOC names and `.toc` extensions match case-insensitively without demoting active-profile variants below generic. Exact spelling wins within a separator variant; remaining case collisions resolve lexically, independently of directory order. Unrelated folder names, foreign flavors, and unknown suffixes are not selected by an unrestricted scan.
- [x] Retail/PTR retain Mainline then generic then Standard; Wrath/Mists/Era/Anniversary retain their primary flavor then generic then Classic-family variants. Named Mists GameMenu/Mainline and UIParentPanelManager/Classic compatibility variants retain their priority.
- [x] A Carbonite-shaped package with incompatible Retail `Carbonite.toc` and compatible `Carbonite-Camelot.toc` admits the provider through discovery and the startup interface gate before its Info/Notes/Warehouse dependents load.
- [x] Forever reports version `1.60.1`, build `69913`, and interface `16001` without enabling a retail API epoch or legacy compatibility bootstrap.

## How it works

- [Client profile architecture](../wiki/systems/client-profiles.md)
- [Lua API registration](../wiki/systems/lua-api.md)

## Implementation inventory

- `Cargo.toml` — cumulative retail epoch features and public client bundles; `retail-12-1-5` extends `retail-12-1-0` for PTR.
- `src/client_profile.rs` — active profile/epoch selection and interface constants.
- `src/asset_resolver_config.rs` — profile-to-CASC-product mapping (`client-ptr` uses `wowxptr`; Forever uses `wow_classic_beta`).
- `data/blizzard-ui-files/wowforever.txt` — Forever manifest generated from `/home/osso/.cache/wow-ui-sim/wow-ui-source/forever`; source branch `forever` and runtime cache profile `wowforever` are intentionally distinct.
- `src/blizzard_ui_sync/pinned.rs` — reads the pinned PTR version/build identity.
- `src/lua_api/workarounds/temporary/client_info_defaults.rs` — exposes the modeled `GetBuildInfo()` identity.
- `src/ptr/strict_removals.lua` — PTR-only post-startup removals, including `C_RecruitAFriend.IsEnabled`.
- `src/lua_api/globals/strings/mod.rs` — epoch-gated retail string registration.
- `src/lua_api/globals/enum_data/addon_system.rs` and `src/lua_api/env_init/enums.rs` — CurioRarity values and profile-specific metadata.
- `src/lua_api/globals/strings/string_data/more_strings.rs` — probe-backed retail 12.1 values.

## Tests asserting this spec

- `src/client_profile.rs` — current retail, historical retail, PTR, and interface-version contracts.
- `tests/wowforever_profile.rs` — Forever identity, manifest/cache isolation, game-type filters, source TOC substitution, and Carbonite-shaped provider/dependent loading through the interface gate.
- `src/loader/tests/toc_discovery.rs` — separator/case priority, foreign/unknown exclusion, folder identity, retained family aliases, and named Mists compatibility variants.
- `src/paths.rs` and `src/asset_resolver_config.rs` — Forever install-root and CASC product selection.
- `src/loader/tests/wow_api_globals/startup_globals.rs` — post-startup strict-removal contract, including PTR-only `C_RecruitAFriend.IsEnabled` removal.
- `tests/blizzard_recruit_a_friend_loads.rs` — retail `C_RecruitAFriend.IsEnabled` availability and behavior.
- `src/lua_api/globals/register.rs` — exact retail 12.1 string values and intentional nil globals.
- `src/loader/tests/wow_api_globals/patch_12_1_service_payloads.rs` — PTR 12.1.5 CurioRarity, vendor mapping, and build-identity contracts.

### Carbonite TOC development proof

Loader source `31ac46b3a`: RED 4/9 TOC tests and 0/1 Carbonite provider/consumer test; targeted offline GREEN 10/10 Forever TOC, 8/8 Forever profile, 12/12 Mists TOC, 3/3 existing Mists screen-selection, and 11/11 Retail TOC tests. These 44 tests cover the selected profiles, not a fresh runtime test of every client or the entire Carbonite addon. Eight pre-existing unrelated lib-test warnings occur under Forever/Mists, also present in RED. Commands, revision scopes, and complete logs are recorded in `/tmp/forever-carbonite-toc-development-ledger.json`; final checks and actual-package startup remain integration-owned.

## Forever event coverage

- Forever accepts and dispatches `PET_STATS_UPDATE`, `SHARD_TRANSFER`, `SHARD_TRANSFER_IMMINENT`, `GUILD_PREFERRED_PLAY_SETTINGS_UPDATED`, and `HIDDEN_GROUP_BUFFS_CHANGED` through ordinary frame event handlers. Empty and invented event names remain rejected.
- Forever also accepts its documented `INPUT_DEVICE_INTERFACE_TRANSITION`, `DIEL_CYCLE_CHANGED`, `DISCORD_LINK_UPDATE`, `CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW`, `EXTERNAL_EVENT_LAUNCH_URL_FAILED`, `GROUP_BUFF_VISUAL_ALERTS_CHANGED`, `GUILD_RANKS_UPDATE_ACTIVE_PLAYER`, `LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY`, `UNIT_HAPPINESS`, and `UNIT_PING_PIN_ADDED` events. Acceptance does not imply every event has a modeled producer.
- Source: Forever 1.60.1.69913 `Blizzard_APIDocumentationGenerated/{PaperDollInfo,System,GuildInfo,UnitAura}Documentation.lua`. This finite addition does not enable permissive legacy validation or imply modeled event producers.
- Behavioral coverage: `tests/startup_api_events.rs`, `forever_source_events_*`.

## Known gaps (current cycle)

- [ ] Full Forever UI/data compatibility is not established by profile selection. Profile tests and separate startup checks pass, but optional panels, game data and third-party dependencies retain explicit coverage limits; see the [Forever report](../wowforever-1.60.1.md). Event acceptance remains a finite, source-backed surface rather than accepting arbitrary names.
- [ ] Forever `GetBuildInfo()` date and trailing fields retain existing temporary defaults, not a native build-date claim.

- [ ] PTR 12.1.5 source synchronization completes from the pinned Blizzard CDN index, but the current startup baseline has six pixel-rounding error records; profile selection is not startup acceptance.
- [ ] Representative PTR panel interactions remain unproven.
- [ ] PTR `GetBuildInfo()` date and trailing return slots remain temporary defaults until the live PTR capture establishes their contract.

## Out of scope

- Removing historical retail API epochs.
- Generalizing the pinned PTR 12.1.5 content-index route into a universal TVFS resolver without a new requirement.
