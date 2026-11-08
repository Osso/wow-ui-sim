# Map display info

`C_Map.GetMapDisplayInfo` exposes supplied per-map `hideIcons` input from `SimState`. Bounded default simulator GREEN is admitted at producer `0979952073b64581903283c87d38ba2eae29f2b5`, epoch `20261010T194147Z`: 86 targeted tests PASS. Later exact P801 simulator publication passes 1/1; PTR finalized independent audit admits 84 actual simulator tests PASS. Neither is complete handoff or authenticated native WoW proof. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Required numeric `uiMapID` selects supplied input; a supplied result is exactly one non-nil boolean `hideIcons`, never a DTO. Current parser accepts finite integral i32 values; native fractional/out-of-range parity is unestablished.
- [x] Explicit inputs for maps 84/85 return supplied true/false, including false as one result; updates remain independent across maps and environments.
- [x] Simulator-input policy: empty input/removal returns zero values. Map catalog presence alone supplies no display input.
- [x] Canonical input starts empty, without guessed defaults, map-flag derivation, or a new Lua setter.
- [x] Tested simulator security: untainted authentic-secret input succeeds; tainted ordinary input succeeds and tainted authentic-secret input rejects.

## Mists original group-provider contract

Pinned Mists `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:168–180` declares `GetMapDisplayInfo(uiMapID: number) -> hideIcons: bool` with `MayReturnNothing = true`. There is no return struct or struct field list. The existing scalar boolean/zero-return projection matches this declaration; no profile-specific return adapter is needed.

- [x] Publish the existing producer on Mists through the same ACTIVE registration selection used by Era/Anniversary, without a second store or a Retail epoch.
- [x] Supplied hide/show inputs reach the provider and XML pin installed by the normally selected Mists WorldMap OnLoad, including map revisits, live updates and independent environments.
- [x] Preserve absent/removed zero results and exactly one supplied false boolean. Native input population and malformed-number parity remain excluded.

`tests/mists_map_display.rs` selects `Blizzard_WorldMap_Mists.toc` and `Blizzard_SharedMapDataProviders_Mists.toc`, loads stock startup order, then finds the actual installed group provider and pin. It invokes the original provider `OnMapChanged:22–23`, reaching original pin `OnMapChanged:112–123` and the query at line114. No alternate consumer, fabricated provider, replacement method or vendor edit is used. The populated selector rejects constant true, false, nil and no-op producers on its own. Corrected RED at `f12339876` reaches the original missing call at line114 after identity/provenance assertions pass. GREEN at `29b030439` passes the populated selector and the separate exact-arity preservation selector. Both first list exactly one case. The retained producer-only RED sibling has byte-identical tests.

The selected boundary is after normal Blizzard startup addon loading, before `apply_post_load_workarounds`. Stock CLI startup runs that pass after Blizzard addons and again after third-party addons (`src/bin/wow_sim/startup.rs:424–444`); WorldMap XML OnLoad runs earlier during loading. A later, existing fog workaround overwrites multiple provider pins, including this group pin. That separate defect is excluded; this test does not claim post-workaround group-pin parity.

Mists stock at the executed GREEN is 8 keys/14 occurrences versus the current base 10/16, with only the query/ScriptErrors pair removed and no additions or common-key count changes. The older 12/18 reference predates social-squelch. Mists/Retail/Era/Anniversary `cargo check --all-targets` at `3923a7699` all pass with zero warnings in Cargo JSON and plain output. Workspace formatting, explicit test formatting and changed-Rust readability pass; later documentation-only updates preserve those Rust inputs.

## How it works

- [Lua API and simulator state](../lua-api.md)

## Implementation inventory

`src/lua_api/state/sim_state.rs` stores `map_display_hide_icons: HashMap<i32, bool>`; `src/lua_api/state.rs` initializes it empty. `src/c_api/c_map.rs` registers the getter, unwraps authorized secrets before numeric validation, and returns zero values for absent input or one boolean for supplied input. Current 097 parser requires finite integral i32 range. Literal declarations say number; these tests do not establish malformed-number native parity.

## Current feature applicability

The `retail-12-1-0` method-table entry preserves existing epoch publication. `client-retail` includes it; `client-ptr` includes `retail-12-1-5`, which includes it. A separate runtime `client_profile::ACTIVE` decision publishes the same producer on Mists, Era and Anniversary without enabling a Retail epoch. Earlier epoch features alone do not enable the getter on other profiles. Cached Retail/PTR declarations establish source applicability; separate PTR runtime receipts below establish bounded simulator execution, not earlier-profile/native parity.

## CF-2 D — static Classic producer (2026-10-24)

- [x] Source publication for Era and Anniversary reuses the existing per-map projection without enabling a Retail epoch or adding publication on unrelated profiles. The Classic decision uses `client_profile::ACTIVE`. Runtime acceptance remains pending.
- [ ] Their normally selected original `GroupMembersPinTemplate` must hide on supplied map84=true and show/bind map85=false; revisits, independent live changes, and opposite inputs in another environment must remain observable through original `GroupMembersPinMixin:OnMapChanged`.
- [ ] Preserve zero returns for absent/removed input, exactly one supplied false boolean, and existing Retail/PTR numeric/security behavior. Do not add Classic coercion claims, defaults, map-flag heuristics, catalogs, Lua setters, or another store.

`tests/cf2_group_map_icons.rs` stages one populated consumer selector, registered only for Era/Anniversary. It loads normal `Blizzard_UIPanels_Game` and `Blizzard_WorldMap` TOC closures without overrides, uses the original XML WorldMapFrame and its `AcquirePin('GroupMembersPinTemplate', provider)`, and prints selected TOCs/metadata/files plus original Lua function provenance and initial loading diagnostics. No surrogate map/frame, replacement consumer, vendor edit, or forced cached-file execution is used.

Selection to reproduce independently in both profiles: `Blizzard_WorldMap_Vanilla.toc` (game, vanilla; RequiredDep MapCanvas + SharedMapDataProviders) → `Blizzard_SharedMapDataProviders_Classic.toc` (game, vanilla/tbc) → `GroupMembersDataProvider.xml` → original Lua `OnMapChanged` lines112–123, target call114. `Blizzard_UIPanels_Game_Classic.toc` supplies `Classic/UnitPositionFrameTemplates.xml`; the pin is the selected XML-derived UnitPositionFrame, not a hand-mixed fixture. The provider's supplied player pin size0 disables only unrelated ping work; original `ShouldShowUnit` is retained.

The consumer selector alone observes hidden/shown states, both selected map IDs when shown, revisits, per-map changes, restoration, and two environments with opposing inputs. Constant true fails shown85; constant false/nil/zero returns fail hidden84; stale or wrong-map input fails opposing-map/live-update observations. These are authored counterexamples, **not executed mutation or RED/GREEN proof**. A setup error before call114 is a blocker, not RED. Separate arity coverage is preservation, not consumer acceptance.

Static producer work removes only the parser/getter compile gates and registers that same backing-state projection for Era and Anniversary. The existing Retail-epoch registration, parser validation/security, per-environment store and return policy are unchanged. Tests remain byte-identical to merged staging. No compilation, runtime selection, RED/GREEN, stock delta, full-suite, prefork, or native parity is claimed; the two unchecked requirements above remain execution obligations.

## Tests asserting this spec

`tests/c_map_probes.rs`, gated by `retail-12-1-0`:

- `get_map_display_info_returns_one_supplied_bool`
- `get_map_display_info_tracks_updates_with_map_and_environment_isolation`
- `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy`
- `get_map_display_info_untainted_secret_returns_one_supplied_bool`
- `get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret`

Staged Classic selectors (not yet compiled/executed):

- `cf2_group_map_icons::original_group_pin_tracks_map_display_visibility_and_live_updates` — populated original-consumer candidate; must independently RED/GREEN on Era and Anniversary.
- `cf2_group_map_icons::map_display_absence_removal_and_supplied_false_preserve_return_arity` — simulator-input preservation, not original-consumer RED.

All five Retail-epoch probes executed PASS in the saved default integration artifact. Security tests use an authentic host-secret number and stamped addon closure; taint/wrapper preservation is asserted. Map 85 is a test-only catalog fixture, not a native map fact.

## Admitted evidence

Exact private report: `/home/osso/.local/state/wow-ui-sim/verification/map-display-prepublication-green-current/independent-report.md`.

Exact epoch: `/home/osso/.local/state/wow-ui-sim/verification/map-display-prepublication-green-current/20261010T194147Z`.

[Sanitized bounded report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-default-green/independent-report.md) and adjacent receipts/hashes retain default evidence without binaries, complete source manifests, or full sweep output.

| Boundary | Saved result | Limit |
|---|---|---|
| Map probes / C_Map controls | 35 / 51 PASS, exits 0 | 86 actual simulator test executions; sealed artifact unchanged |
| Stock stored-error CLI | Exit 0, stdout `[]` | No addons/saved variables; CASC disabled by mode |
| fmt / default / PTR check | Exits 0, source_equal true | Saved revision; PTR compile check only |
| P801 sweep | Exit 1, 0 PASS / 1 FAIL | Only stale MapDisplay gap; no new gaps |
| MapDisplay publication row | `ok: true`, raw/lookup function | Presence/lookup only, not behavior |

Saved checks retain six iced_wgpu manifest deprecation warnings; exit 0 does not mean warning-free. Exact executed compiler version is unsealed. Earlier ordinary/security RED epochs remain historical failures, superseded only for the bounded current behavior above.

## Later P801 and PTR receipts — 2026-10-10

Exact independent P801 report: `/home/osso/.local/state/wow-ui-sim/verification/p801-live-input-current/independent-green-report.md`. Saved native simulator test-binary epoch `20261010T195723Z` at `63f3a8a7373c8e52ffcc005bb443a9cf64b59fbf`: 1 PASS, exit 0; current publication 269 rows / 253 OK / 16 gaps. Historical validator PASS preserves frozen 269 / 252 OK / 17 gaps, sources and receipts unchanged; exact frozen/live digest pair retained in the [model investigation](../wiki/investigations/map-display-info-model.md). This supersedes current P801 pending status, not the older failed sweep receipt above.

[Sanitized PTR receipts/correction/hashes](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/README.md): exact epoch `/home/osso/.local/state/wow-ui-sim/verification/map-display-ptr-green-current/20261010T200242Z`, initial33 PASS + continuation51 PASS = **84 actual PASS**, all five MapDisplay targets PASS. Compile exit0/source equality true; unchanged artifacts; separate stored-error CLI `[]`, exit0, CASC disabled. Controller expected35 was wrong because two party tests are Retail-only; original error and correction JSON preserved. Final [independent PTR audit](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/independent-report.md) admits bounded **84/84 PASS**; earlier PENDING inspection is superseded, not rewritten.

## Known gaps (current cycle)

- [x] Final PTR independent audit admits 84 actual PASS, including all five MapDisplay targets.
- [x] Wrath/Mists/Era/Anniversary/Forever shared-state compile checks exit 0 with source equality; getter excluded in those saved checks. Era/Anniversary publication added later has no runtime or compile receipt yet. Six dependency manifest deprecations per profile remain. No runtime/native acceptance.
- [ ] Authenticated native WoW ordinary/secret behavior and runtime return secrecy unverified. `SecretArguments = "AllowedWhenUntainted"` is declaration evidence, not runtime secrecy proof.

## Out of scope

No complete handoff or broad-suite/profile readiness claim. Native unknown-ID/no-return selection, malformed-number and revoked-context parity remain unestablished. Missing simulator input is not an unknown native map. Cached declarations/content hashes do not authenticate cache origin or seal assets/dependency linkage. CASC feature presence and stock CLI `[]` do not establish CASC-backed CLI parity. No tests, checks, builds, or native execution were run for this retention update.
