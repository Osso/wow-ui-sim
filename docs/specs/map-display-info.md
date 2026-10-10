# Map display info

`C_Map.GetMapDisplayInfo` exposes supplied per-map `hideIcons` input from `SimState`. Bounded default simulator GREEN is admitted at producer `0979952073b64581903283c87d38ba2eae29f2b5`, epoch `20261010T194147Z`: 86 targeted tests PASS. Later exact P801 simulator publication passes 1/1; PTR finalized independent audit admits 84 actual simulator tests PASS. Neither is complete handoff or authenticated native WoW proof. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Required numeric `uiMapID` selects supplied input; a supplied result is exactly one non-nil boolean `hideIcons`, never a DTO. Current parser accepts finite integral i32 values; native fractional/out-of-range parity is unestablished.
- [x] Explicit inputs for maps 84/85 return supplied true/false, including false as one result; updates remain independent across maps and environments.
- [x] Simulator-input policy: empty input/removal returns zero values. Map catalog presence alone supplies no display input.
- [x] Canonical input starts empty, without guessed defaults, map-flag derivation, or a new Lua setter.
- [x] Tested simulator security: untainted authentic-secret input succeeds; tainted ordinary input succeeds and tainted authentic-secret input rejects.

## How it works

- [Lua API and simulator state](../lua-api.md)

## Implementation inventory

`src/lua_api/state/sim_state.rs` stores `map_display_hide_icons: HashMap<i32, bool>`; `src/lua_api/state.rs` initializes it empty. `src/c_api/c_map.rs` registers the getter, unwraps authorized secrets before numeric validation, and returns zero values for absent input or one boolean for supplied input. Current 097 parser requires finite integral i32 range. Literal declarations say number; these tests do not establish malformed-number native parity.

## Current feature applicability

`retail-12-1-0` gates this surface. `client-retail` includes it; `client-ptr` includes `retail-12-1-5`, which includes it. Earlier epoch features alone do not enable the getter. Cached Retail/PTR declarations establish source applicability; separate PTR runtime receipts below establish bounded simulator execution, not earlier-profile/native parity.

## Tests asserting this spec

`tests/c_map_probes.rs`, gated by `retail-12-1-0`:

- `get_map_display_info_returns_one_supplied_bool`
- `get_map_display_info_tracks_updates_with_map_and_environment_isolation`
- `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy`
- `get_map_display_info_untainted_secret_returns_one_supplied_bool`
- `get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret`

All five executed PASS in the saved default integration artifact. Security tests use an authentic host-secret number and stamped addon closure; taint/wrapper preservation is asserted. Map 85 is a test-only catalog fixture, not a native map fact.

## Admitted evidence

Exact private report: `/home/osso/.local/state/wow-ui-sim/verification/map-display-prepublication-green-current/independent-report.md`.

Exact epoch: `/home/osso/.local/state/wow-ui-sim/verification/map-display-prepublication-green-current/20261010T194147Z`.

[Sanitized bounded report](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-default-green/independent-report.md) and adjacent receipts/hashes retain default evidence without binaries, complete source manifests, or full sweep output.

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

[Sanitized PTR receipts/correction/hashes](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/README.md): exact epoch `/home/osso/.local/state/wow-ui-sim/verification/map-display-ptr-green-current/20261010T200242Z`, initial33 PASS + continuation51 PASS = **84 actual PASS**, all five MapDisplay targets PASS. Compile exit0/source equality true; unchanged artifacts; separate stored-error CLI `[]`, exit0, CASC disabled. Controller expected35 was wrong because two party tests are Retail-only; original error and correction JSON preserved. Final [independent PTR audit](../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/map-display-ptr-green/independent-report.md) admits bounded **84/84 PASS**; earlier PENDING inspection is superseded, not rewritten.

## Known gaps (current cycle)

- [x] Final PTR independent audit admits 84 actual PASS, including all five MapDisplay targets.
- [x] Wrath/Mists/Era/Anniversary/Forever shared-state compile checks exit 0 with source equality; current getter excluded. Six dependency manifest deprecations per profile remain. No runtime/native acceptance.
- [ ] Authenticated native WoW ordinary/secret behavior and runtime return secrecy unverified. `SecretArguments = "AllowedWhenUntainted"` is declaration evidence, not runtime secrecy proof.

## Out of scope

No complete handoff or broad-suite/profile readiness claim. Native unknown-ID/no-return selection, malformed-number and revoked-context parity remain unestablished. Missing simulator input is not an unknown native map. Cached declarations/content hashes do not authenticate cache origin or seal assets/dependency linkage. CASC feature presence and stock CLI `[]` do not establish CASC-backed CLI parity. No tests, checks, builds, or native execution were run for this retention update.
