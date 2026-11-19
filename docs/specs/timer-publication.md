# Timer API publication

This spec defines the profile-specific publication of `C_Timer` functions and the Retail mixin factory bootstrap. Timer scheduling is described in the [event system](../event-system.md).

## What it must do

- [x] VERIFIED — Retail 12.1.5.70688 publishes `C_Timer.After`, `NewTicker`, and `NewTimer` with flag 3 to ordinary and secure environments; secure publication requires the shared secure environment to be a table. Evidence: `pr46-49-decomp-answers.md` §R88n.9. Runtime test: `src/c_api/helpers.rs`.
- [x] VERIFIED — Retail 12.1.5.70688 has no native `C_Timer.NewTimerID` entry. The simulator does not publish it on Retail. Evidence: `pr46-49-decomp-answers.md` §R88n.9. Runtime test: `tests/c_small_probes.rs`.
- [x] INFERRED — Non-Retail profiles retain `C_Timer.NewTimerID` as simulator preservation; this is not a claim of native support. Cache scans found no reference in Retail, WoW Forever, Mists, or Wrath. Test: `tests/c_small_probes.rs`.
- [x] INFERRED — Retail gets `CreateAndInitFromMixin` from `Blizzard_SharedXMLBase/Mixin.lua`, not a simulator bootstrap fallback. The pinned runtime-cache TOC lists `Mixin.lua` before dependent addons, and `Blizzard_AddOnPerformance.toc` depends on `Blizzard_SharedXML`; its Lua calls the factory. Runtime consumer test: `tests/blizzard_shared_xml_base_loads.rs`.

## How it works

- [Timer callback dispatch](../timer-after-callback.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/timer_layout.rs` — registers timer functions.
- `src/lua_api/globals/missing_surface/small_probes.rs` — retains the non-Retail `NewTimerID` preservation.
- `src/lua_api/env_init/shared_bootstrap.lua` — gates the `CreateAndInitFromMixin` fallback.
- `src/lua_api/env_init/bootstrap.rs` — supplies the active-profile bootstrap flag.
- `src/c_api/secure_publication_data.rs` — records verified C API publication flags.

## Tests asserting this spec

- `src/c_api/helpers.rs`
- `tests/c_small_probes.rs`
- `tests/blizzard_shared_xml_base_loads.rs`
- `tests/utility_api.rs`

## Known gaps (current cycle)

- [ ] UNKNOWN — The Retail 12.1.5.70688 native origin of `CreateAndInitFromMixin` is not proved. The cache-backed load test proves the current pinned runtime load path, not the exact 70688 client image.

## Out of scope

- Native support for `NewTimerID` on non-Retail profiles is not established. Its retained behavior is preservation only.
- Changes to timer scheduling, callback arguments, or timer identity are not part of this contract.
