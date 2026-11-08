# Main source/evidence corrections — 2026-10-09

## UnitFrame enum metadata

Agent496's incomplete report is not authority for an epoch fix. Main independently read exact retained96 panic and revision96 Cargo.toml and compared test bytes with current:
- Panic tests/edit_mode_api/enums.rs:44:5 is assert_eq!(max_value,21), not BigDefensiveIconSize. BigDefensiveIconSize21 and min0 assertions precede it and passed; count22 assertion not reached.
- Revision96 default client-retail maps profile-retail +retail-12-1-0, NOT12.1.5. Current firsttrace compiler artifact likewise lists retail-12-1-0, no12.1.5.
- Actual publisher path init_enum_globals in src/lua_api/env_init/enums.rs seeds static enum tables/missing_enums then invokes patch_12_0_5_enums::register when its cumulative feature is active. src/c_api/patch_12_0_5_enums.rs:122–127 adds DebuffIconSize19, BigDefensiveIconSize21, BuffIconSize22. This is an actual publisher missed by the report, not dynamic cached-document enum loading.
- Cached Retail provenance exists beside completion marker under AddOns/.wow-ui-sim-blizzard-ui-provenance: schema1/profile retail/product wow/version12.1.0.69933/build_key dcfc90fffd79ba00406ae46f5f657592/manifest_sha256 aa7dfec3fb3bc9440a8c737c2274363502cb3d22c7939b8a570a58e717202edd/source casc-local-or-cdn/fallback none. This is recorded source identity, not per-file cryptographic/native-runtime authentication. Generic missing_enums metadata21/22 is earlier data; final publisher must be considered.
- Candidate fixture correction: preserve BigDefensiveIconSize21/min0, assert sourced active BuffIconSize22 and updated metadata22/23. Explicit12.0.0 removes BigDefensiveIconSize; moving this test there remains wrong. Not implemented yet; queued controlled rebuild's source epoch is unchanged.

## Render-group boundary

Agent499 misidentified lines63/64 as registry-name lookups. Main read exact revision96/current identical test and line numbers:
- Name lookup unwraps are lines58/62. Panic63:64 is `order.iter().position(|&id|id==red).unwrap()`. Named red and blue IDs were found; red is absent from returned bucket order. Missing-widget-name diagnosis is rejected.
- Main traced CreateTexture ->Frame::new(name)->register_child_with_strata ->WidgetRegistry::register, which indexes widget.name; no missing name-registration defect was identified.
- Fixture case5 creates a GameTooltip, custom color texture, then SetOwner(blue,ANCHOR_NONE) and never calls Show afterward. Simulator owner.rs record_tooltip_owner clears tooltip lines and calls set_frame_visible(false); this gives a concrete candidate for missing visible-bucket ID, not an ordering regression. Exact missing case not printed by current panic, so candidate remains unexecuted/unproven.
- Native controlled-layer spec requires visible controls. Linked original native Lua probe is absent on this host (`docs/local/private/probes/UnitFrameLayerProbe-controls-2026-09-08.lua`), so no claim that its exact Show ordering was independently reproduced. A visibility-focused fixture observation/explicit visible control setup is the next small step; preserve ordering assertions and avoid renderer changes. Not implemented yet.

No runtime/model/vendor edits, tests, or native execution performed for these corrections.
