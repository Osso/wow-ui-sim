# Map display info

`C_Map.GetMapDisplayInfo` must expose supplied per-map `hideIcons` input from `SimState`. Preparation revision `1ee814f895d5b3f262bd4576745d41add0da47cb` has admitted behavioral RED. Getter work in `src/c_api/c_map.rs` is in progress separately; no GREEN evidence is admitted here. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Current Retail/PTR API accepts required non-nil numeric `uiMapID`; a supplied result is exactly one non-nil boolean `hideIcons`, never a DTO.
- [ ] Explicit inputs for maps 84/85 return supplied true/false, including false as one result rather than no result.
- [ ] Queries reflect input updates independently across maps and simulator environments.
- [ ] Simulator-input policy: empty input or removal returns zero values, not one nil or false. Map catalog presence alone supplies no display input.
- [ ] Canonical input starts empty; no guessed defaults, derivation from map flags, or new Lua setter.

## How it works

- [Lua API and simulator state](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: explicit `map_display_hide_icons: HashMap<i32, bool>` input.
- `src/lua_api/state.rs`: empty canonical initialization.
- `src/c_api/c_map.rs`: input-backed getter/registration implemented at `e8c8ced9c`, not verified GREEN.

## Current feature applicability

`cfg(feature = "retail-12-1-0")` is the current capability gate. In `Cargo.toml`, `client-retail` includes `retail-12-1-0`; `client-ptr` includes `retail-12-1-5`, which includes `retail-12-1-0`. That closure includes `retail-12-0-7` and earlier cumulative capabilities. Earlier epoch features alone do not enable this gate; any build explicitly enabling it also qualifies. Cached Retail/PTR declarations establish source applicability, not earlier-profile or native-build parity.

## Tests asserting this spec

`tests/c_map_probes.rs`, gated by `retail-12-1-0`:

- `get_map_display_info_returns_one_supplied_bool`
- `get_map_display_info_tracks_updates_with_map_and_environment_isolation`
- `get_map_display_info_absence_and_removal_return_zero_values_sim_input_policy`
- `get_map_display_info_untainted_secret_returns_one_supplied_bool`
- `get_map_display_info_tainted_caller_accepts_ordinary_and_rejects_secret`

The last two tests use an authentic host-secret numeric map ID and stamped addon closure. Their compiled RED/GREEN remains pending; expected failures are not proof.

Admitted RED, 2026-10-10: `/home/osso/.local/state/wow-ui-sim/verification/map-display-red-current/20261010T193141Z`. `submission.json` pins preparation revision `1ee814f895d5b3f262bd4576745d41add0da47cb`; `outcome.json` records compile exit 0 and `source_equal = true`. `execution-results.json` records all three selected tests reached, unchanged sealed artifact, and execution exit 101. `map-display.stdout` reports three actual FAILED tests, zero passed.

- Absence/removal test failed at its initial empty-input boundary: actual arity 1, expected 0 (`map-display.stderr`, line assertion at `tests/c_map_probes.rs:111`). Removal behavior was not reached/proved.
- Both positive tests failed at the supplied map-84 type assertion: actual `"nil"`, expected `"boolean"` (`tests/c_map_probes.rs:51`). Later false/update/isolation assertions were not reached/proved.

No GREEN execution is admitted. Map 85 is explicitly seeded as a test-only catalog fixture; fixture metadata and supplied booleans are not native map facts. Zero returns for empty/removed input is simulator-input policy, not a native unknown-ID claim.

## Known gaps (current cycle)

- [ ] Finish getter/registration and establish behavioral GREEN plus applicable controls/checks; admitted RED does not satisfy runtime requirements.
- [ ] **NOT VERIFIED — security behavior:** cached Retail and PTR `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:216` declare `SecretArguments = "AllowedWhenUntainted"`. This source annotation is separate from supplied-input behavior. Simulator enforcement and native ordinary/secret-argument behavior under tainted/untainted calls are unverified. No `SecretReturns` annotation appears; that absence does not prove runtime return secrecy. Typed numeric extraction alone is not security proof.

Security/feature evidence: `/home/osso/.local/state/wow-ui-sim/handoff/map-display-security-recovered.md` (2026-10-10). No native security result is claimed.

## Out of scope

Native unknown-map-ID behavior is unestablished. Cached Retail/PTR `MapDocumentation.lua` declarations pin `MayReturnNothing = true` and one `bool hideIcons`, but do not identify which IDs yield zero results. Missing simulator input is not equivalent to an unknown native map. Current cached consumer truthiness does not establish that boundary.

Earlier historical/profile applicability and publication reconciliation remain separate. No native catalog-completeness prerequisite is imposed on this supplied-input slice.
