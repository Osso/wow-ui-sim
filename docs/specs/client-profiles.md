# Client Profiles

Client profile bundles select the runtime cache and API epoch exposed by wow-ui-sim. Source lives in `Cargo.toml` and `src/client_profile.rs`; see [Client Profiles](../wiki/systems/client-profiles.md) for architecture and loader details.

## What it must do

- [x] TOC game-type headers and file annotations use case-insensitive space/comma-separated tokens and ignore unknown tokens. Zero masks impose no restriction; nonzero allow masks require overlap and exclude masks reject overlap. Game-type header values retain brackets verbatim rather than evaluating them as file conditions.
- [x] Retail recognizes the 70688 token table; PTR and Forever additionally recognize `camelot` and `forever` as the same bit. Forever's modeled active mask is mainline plus camelot; Retail/PTR retain mainline plus standard. Native basis and unknown client masks are recorded in [TOC game-type tokens](../wiki/reference/toc-game-type-tokens.md#native-game-type-masks).

- [x] The public `client-retail` bundle selects the retail profile and current retail 12.1.5 API epoch (`120105`), selected for installed Retail 12.1.5.70448.
- [x] Retail API epoch features remain cumulative, so 12.1.5 includes the modeled 12.0.0, 12.0.5, 12.0.7, and 12.1.0 surfaces.
- [x] Historical retail epochs remain selectable through `profile-retail` without enabling the current-retail bundle.
- [x] `client-ptr` remains a distinct profile/cache with TOC interface `120107`, retaining the cumulative 12.1.5 API surface. This interface selection does not claim inherited 12.1.7 API coverage or refresh the source/cache pin.
- [x] PTR 12.1.7 skips `##` metadata directives containing `[Family]`, `[Game]`, `[TextLocale]`, or `[RegionalTextLocale]`, preserving earlier values and dependency lists. Game-type headers are exempt and retain their values verbatim; other profiles retain existing metadata handling.
- [x] PTR and Forever expand `[RegionalTextLocale]` in TOC file paths to `enUS` for the simulator's current enUS text locale / US region. Existing `[Family]`, `[Game]`, and `[TextLocale]` file expansion is preserved; other profiles do not gain regional expansion. Alternate locale/region configuration is not modeled by this change.
- [x] PTR CASC resolves the official `wowxptr` product; 12.1.5.69594 uses BuildConfig `4a9973f37906f8cfb344f8a9fe6777e0` and Gethe `ptr2` source commit `49b69918fcdc77e109813281e4f537d45ec7dcbf`.
- [x] PTR `GetBuildInfo()` retains `12.1.5` and build `69594` from the pinned source identity but publishes selected TOC interface `120107`; non-PTR builds publish version `12.1.5` and interface `120105` with `retail-12-1-5`, `12.1.0`/`120100` with `retail-12-1-0`, otherwise `12.0.7`/`120007`. Retail build/date/trailing fields remain temporary defaults, not installed-build identity.
- [x] PTR 12.1.5 publishes `Enum.CurioRarity.EpicTier2 = 5` and metadata through 5; earlier retail epochs retain the four-value contract.
- [x] Same-epoch profiles may have source-proven post-startup removals: retail 12.1 keeps `C_RecruitAFriend.IsEnabled`, while PTR hides it after startup.
- [x] Default Retail 12.1.5 uses its separately generated shipped base GlobalStrings table (26,069 authenticated rows from 70448); 65 unreadable encrypted-section rows are omitted, not fabricated. Earlier Retail epochs, PTR and Classic retain the previous shared table; Forever retains its separate table. The original pulse-health command consumes Retail's shipped slash-command strings.
- [x] Retail 12.1.5 admits `ALERT_AGE_VERIFICATION_RESTRICTED`, published by 70448 `SocialRestrictionsDocumentation.lua` and registered by original `Blizzard_Game/Startup.lua`; invented suffixed names remain rejected.
- [x] `client-wowforever` selects a distinct `WowForever` profile with interface `16001`, cache `wowforever`, CASC product `wowf`, and unchanged `_classic_beta_` install-path discovery. Product selection does not establish addon-path parity.
- [x] Forever resolves `[Family]` to `Mainline` and `[Game]` to `Camelot`, accepting `mainline`/`camelot` annotations, not `classic`, `standard`, or `vanilla`; matching exclusion annotations suppress files and dependencies. This follows the authenticated 1.60.1.69913 source tree, not the product name: ActionBar's mainline annotations select present `AssistedCombatManager.lua` and `ActionButtonOverrides.lua`, while its classic annotation selects absent `Mainline/MainMenuBar.lua`.
- [x] Forever selects Camelot variants, generic `.toc`, then Mainline variants. Each flavor tier supports underscore then hyphen spellings; retaining underscore precedence is simulator policy, not a claim about native tie-order when both exist. Mainline-only load-on-demand dependencies participate in discovery and dependency ordering; no WorldMap-specific exception is used.
- [x] Supported TOC names and `.toc` extensions match case-insensitively without demoting active-profile variants below generic. Exact spelling wins within a separator variant; remaining case collisions resolve lexically, independently of directory order. Unrelated folder names, foreign flavors, and unknown suffixes are not selected by an unrestricted scan.
- [x] Retail/PTR retain Mainline then generic then Standard; Wrath/Mists/Era/Anniversary retain their primary flavor then generic then Classic-family variants. Named Mists GameMenu/Mainline and UIParentPanelManager/Classic compatibility variants retain their priority.
- [x] A Carbonite-shaped package with incompatible Retail `Carbonite.toc` and compatible `Carbonite-Camelot.toc` admits the provider through discovery and the startup interface gate before its Info/Notes/Warehouse dependents load.
- [ ] Forever reports version `1.60.1`, build `70532`, and interface `16001` without enabling a retail API epoch or legacy compatibility bootstrap. Metadata and regression expectations now match Gethe `forever` commit `ca8368f2f2380cbe696854f7f2646387c278a61b`'s `version.txt` (`1.60.1.70532`) and installed product `wowf`, build key `2582289d81e380163c7a13472df4cb81` (verified October 25, 2026). Desktop latest-entry-wins residency and the synchronized local root/encoding BLTE content hashes pass; see the [selected-target checkpoint](../updating-blizzard-ui-to-a-new-patch.md#current-selected-target--october-25-2026). Simulator execution is pending; this is a new acceptance epoch, not a 70374 rerun. `Jun 17 2026` and trailing fields remain temporary defaults, not sourced native metadata. Historical 70374 identity tests at runtime source136 retain their [central receipt](../baselines/README.md#verified-scoped-receipts--2026-10-15); they do not prove 70532. Cross-profile/native/full-parent gates remain open.

## Retail 70448 bounded execution

After rebase onto `9c0251fca`, default Retail startup exits 0 with exact `[]` against the isolated 4,026-file installed-build cache. Preservation passes 265/265 (approved batch132, quest82, label/widget51). Newly enabled epoch behavior passes 89 unit/API and 50 integration cases; atlas/aura controls pass 23/23. Default `cargo check --tests` and workspace formatting pass; seven pre-existing unused publication helper warnings remain. No consumer hashes were re-pinned. Historical absence assertions are retained on pre-12.1.5 Retail epochs, not applied to the new current bundle.

Runtime and preservation source `20398a9eb` is production-equivalent to corrected integration source `115761b56`; only unrelated historical test gates and subsequent docs differ. Other-profile/native/GPU/full-suite and the 65 encrypted GlobalStrings rows are not covered. [Profile implementation and isolation notes](../wiki/systems/client-profiles.md#retail-70448-source-and-producer-update--october-16-2026) own the source/cache details.

## Separate pinned Wrath source restoration

Wrath retains the historical Gethe 3.3.5 source pin in `data/blizzard-ui-builds/wrath.json`, not the installed Mists product. Its separate source snapshot is `~/.cache/wow-ui-sim/wow-ui-source/wrath`, containing `AddOns` and `FrameXML`; Mists/classic and Forever snapshots must not be overwritten.

`tools/gen_blizzard_ui_manifest.py wrath --no-refresh` validates that snapshot's `.wow-ui-sim-wrath-source.json` revision against the committed pin, then enumerates AddOns-relative paths plus `FrameXML/…` paths. Missing FrameXML or a mismatched pin rejects before replacing the manifest. Wrath has no moving branch refresh; other profiles retain their existing branch workflow.

Pinned archive acquisition and all 382 copied source files have independent byte-level verification. This earlier receipt is source-only; actual full-profile compilation and runtime publication are recorded below, separately from consumer and native acceptance. Saved generator execution at `e6fc26e77` passes five in-process CLI-entrypoint tests: two valid three-file snapshot fixtures and three rejection/preservation controls. Corrected RED rejected both valid fixtures; generic negative controls passed before implementation. The real pinned snapshot now regenerates a 382-entry Wrath manifest. The historical failing startup inventory is paired below; [bounded guard-retirement acceptance](editbox-text-position-selection.md#wrath-nil-field-guard-retirement--accepted-bounded) supersedes the earlier nil-cursor blocker, not the unrelated startup errors.

`src/blizzard_ui_sync/wrath_snapshot.rs` implements fresh-destination extraction of the two pinned roots, archive-path/link validation, required FrameXML TOC and provenance publication last. The Wrath-only acquisition route reuses a matching local snapshot, otherwise downloads that exact pinned archive when absent; invalid existing snapshots error unchanged. Transient requests receive bounded retries with backoff/jitter and Retry-After handling. No CASC UI-source fallback or asset-product retargeting is allowed. Standalone helper execution at `94b8593bc` passes four tests after the positive fixture failed at the Unsupported baseline; a separate real-archive test matches all 382 selected files against the verified local snapshot. Independent review accepts these component receipts, not remote-origin authentication. Later full-profile compilation and runtime publication are recorded below; the consumer owner distinguishes isolated execution from the original startup gap.

The isolated runtime-copy helper maps manifest `FrameXML/…` entries to the profile's sibling FrameXML tree; other entries remain AddOns-relative. It preflights selected source files and conflicting destination bytes before copying, preserves identical and unrelated files, and never publishes completion markers itself. The wrapper publishes the ordinary runtime text provenance and completion marker only after copying succeeds. Cache acceptance checks the pin/manifest identity and selected files, including sibling FrameXML, without requiring the source snapshot or a WoW install. Conflicting existing metadata errors without deletion. Wrath dispatches to this route before generic CASC synchronization; non-Wrath routes stay unchanged. Independent audit accepts nine standalone copy/publication fixtures at `5d795cd47` and seven acquisition/HTTP fixtures at unchanged source `f64559a14` (three local-source cases, four real localhost HTTP cases). These component receipts alone do not establish full-profile compilation or runtime publication; the separate actual-publication receipt below does. Live Gethe acquisition is not established.

`tests/wrath_scrolling_cursor.rs` exercises the complete unmodified pinned `UIPanelTemplates.lua` consumer through XML-wired callbacks, real layout ticks and real scroll widgets for cold SetText, Insert and keyboard movement. It requires the runtime cache; no private-source fallback is allowed. [Consumer history](editbox-text-position-selection.md#wrath-original-scrolling-consumer--2026-10-16) owns initial fixture failures and correction. The [retirement owner](editbox-text-position-selection.md#wrath-nil-field-guard-retirement--accepted-bounded) accepts bounded guard removal with independently verified genuine-consumer, ordering and preservation proofs. Earlier guard-retained statements describe saved checkpoints; native parity, clean full startup and whole-profile acceptance remain unproven.


### Actual Wrath publication — 2026-10-16

Independent saved-artifact verification accepts full Wrath compilation at `c06c84362`: 23m04s, 747 valid Cargo records, successful terminal result and four authenticated seals (integration, library tests, CLI and simulator). Two compiler messages duplicate the existing `UnitRole` dead-code warning; not warning-free. This is compilation, not test execution or current-fixture proof.

The sealed CLI actually published 382 files, exit0. All 382 current runtime files match the publication receipt and pinned source snapshot bytes; pin/manifest provenance and completion marker authenticate. Four other-profile source/runtime trees retain all 18,427 captured entries unchanged. Source/runtime acquisition is restored; native parity, full startup and continuous immutability are not established. Independent report retained privately; no host receipt paths published here.

The [historical failing Wrath startup inventory](../baselines/README.md#wrath-lua-errors) records 45 unique errors / 89 occurrences, exit1, including 34 original nil-cursor failures. Check remains pending. Later [bounded retirement acceptance](editbox-text-position-selection.md#wrath-nil-field-guard-retirement--accepted-bounded) supersedes that cursor blocker only; unrelated non-cursor startup Lua errors remain.

## Bounded shared-controls profile execution — 2026-10-16

[Existing Mists/Era/Anniversary owner matrix](../baselines/README.md#mists-shared-controls--2026-10-16) records the same39 logical behaviors separately executed per profile. [Era saved-artifact acceptance](../baselines/README.md#era-shared-controls--2026-10-16) and [Anniversary saved-artifact acceptance](../baselines/README.md#anniversary-shared-controls--2026-10-16) each cover their own normal library and separate test snapshot, not current HEAD/source. No startup/native/cache/all-profile or full-parent closure.

## How it works

- [Observed TOC game-type tokens](../wiki/reference/toc-game-type-tokens.md) — bounded source inventory, not a native acceptance contract; filename suffixes and filter strings remain separate.

- [Client profile architecture](../wiki/systems/client-profiles.md)
- [Lua API registration](../wiki/systems/lua-api.md)

## Implementation inventory

- `Cargo.toml` — cumulative retail epoch features and public client bundles; `retail-12-1-5` extends `retail-12-1-0` for PTR.
- `src/client_profile.rs` — active profile/epoch selection and interface constants.
- `src/asset_resolver_config.rs` — profile-to-CASC-product mapping (`client-ptr` uses `wowxptr`; Forever uses `wowf`).
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

## Startup warning test applicability

The Wrath cache is the flat historical FrameXML snapshot, not the modern split-addon UI. `tests/startup_warnings.rs` must retain applicable Wrath consumers and compile-exclude only tests whose required consumer is absent.

Verified 2026-10-19 against the actual Wrath cache: 382 source files plus two cache metadata files. `Blizzard_RestrictedAddOnEnvironment` is absent, as are all five files required by `test_secure_env_toc_annotations_are_exhaustive`: `Blizzard_ChatFrameBase/Shared/ChatFrameFiltersSecure.lua`, `Blizzard_CooldownViewer/CooldownViewerSecure.lua`, `Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua`, and `Blizzard_UnitFrame/Shared/{TargetFrameAuraButton,TargetFrameAuraContainer}.lua`. No Wrath Lua/XML/TOC contains `EssentialCooldownViewer`, `LoadIntoEnvironment`, `GetFrameMetatable`, or `PropagateForbiddenToReferencedFrames`.

Therefore `test_restricted_addon_environment_exposes_execution_surface`, `test_secure_env_annotated_files_load_cleanly`, and `test_secure_env_toc_annotations_are_exhaustive` exclude only `client-wrath`. Historical `FrameXML/SecureTemplates.xml` and `SecureHandlerTemplates.xml` do not establish the modern addon or annotated five-file contract. Other-profile eligibility and all assertions remain unchanged.

`test_no_warnings_on_startup` remains enabled: Wrath ships `AddOns/Blizzard_AchievementUI/Blizzard_AchievementUI.xml`. Its achievement/stats containers inherit `HybridScrollBarTemplate`, whose `$parentBG` texture is declared by `FrameXML/HybridScrollFrame.xml`. The broad fixture must load the required authentic `FrameXML/FrameXML.toc` before loading addons, matching product startup order in `src/bin/wow_sim/addon_loading.rs`. Previously it enumerated only AddOns, omitting these templates and failing at `Blizzard_AchievementUI.lua:628` and `:1634`. FrameXML failures, handler errors and runtime diagnostics use the same strict collector as addons; none are cleared or baselined. This repairs the fixture, not Blizzard Lua or an API default.

Static inventory at `fddc3bc0b`: [startup warning tests](../../tests/startup_warnings.rs) and [late fixtures](../../tests/startup_warnings/late.rs). After the three Wrath-only secure-environment exclusions, Wrath has 4 eligible tests (three of its former seven are gated), Retail 26 unchanged. These counts describe source eligibility, not passing runtime execution.

The late fixtures have consumer-scoped compile-time eligibility:

| Required consumer | Eligible public client profiles | Late tests |
|---|---|---:|
| Retained secure-action and zone-text consumers | All seven | 2 |
| Split-addon consumers absent from the flat Wrath tree | Retail, PTR, Forever, Mists, Era, Anniversary | 8 |
| AccountStore, BossBanner, EventUtil, FrameUtil, HousingTutorials or NewPlayerExperience consumers | Retail, PTR, Forever | 7 |

The two all-profile late tests use `load_profile_framexml()`: Wrath loads its manifest-backed `FrameXML/FrameXML.toc`; other profiles load `Blizzard_FrameXML`. This helper is limited to the secure-action protection tuple and zone-text fading-helper fixtures. It does not change production startup. The broad `test_no_warnings_on_startup` fixture has the separate Wrath FrameXML-before-addons ordering described above.

The parent module also restricts its widget-container test and three AccountStore tests to Retail, PTR and Forever. Its three secure-environment tests exclude only `client-wrath`, as detailed above. Static eligibility across both modules is:

| Public client profile | Parent-module tests | Late tests | Total eligible tests |
|---|---:|---:|---:|
| Retail (`client-retail`) | 9 | 17 | 26 |
| PTR (`client-ptr`) | 9 | 17 | 26 |
| Forever (`client-wowforever`) | 9 | 17 | 26 |
| Mists (`client-mists`) | 5 | 10 | 15 |
| Era (`client-era`) | 5 | 10 | 15 |
| Anniversary (`client-anniversary`) | 5 | 10 | 15 |
| Wrath (`client-wrath`) | 2 | 2 | 4 |

Counts assume exactly one named public `client-*` profile feature. The gates name `client-retail`, not the standalone historical `profile-retail` marker. Eligibility is not compiled discovery, runtime acceptance or a clean-startup claim.

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
