# Pending transmog cost: exact row357

`C_TransmogOutfitInfo.GetPendingTransmogCost()` exposes an optional explicit host snapshot as positional `cost, modifierFlags`. This is an **inferred simulator contract**, not native-verified behavior. Input types live in `src/c_api/c_transmog_outfit_info/pending_cost_info.rs`; backing state lives in `src/lua_api/state/sim_state.rs`. See [C API architecture](../wiki/systems/c-api.md).

## What it must do

Producer follows compiled RED. Saved GREEN and startup evidence below are parent-observed; independent acceptance remains pending.

- [ ] Under `retail-12-0-5`, accept an explicit optional host snapshot with `cost: u64` and `modifier_flags: u32`; default to `None`, without fabricated production values.
- [ ] **Guessed absence policy:** a no-argument query returns zero values for `None`. `Some` returns exactly two ordinary public Lua numbers, including `(0, 0)`, zero cost with nonzero flags, and positive cost with zero flags.
- [ ] Return concrete test data `123450, 14` unchanged. `14 = 8 | 4 | 2` uses existing enum assertions as fixture context, not native enum chronology or discount calculation evidence. Preserve all u32 bits, including unknown bits and `4294967295`.
- [ ] **Chosen representation limit, not a native cost limit:** return cost exactly through `9007199254740991`; larger host costs raise an explicit error instead of rounding. Failed reads retain the snapshot; later valid replacement or clearing recovers normally.
- [ ] Repeated reads leave snapshot, catalog, locks, viewed-slot, viewed-outfit, situation setting, and pending sheathe state unchanged. Those adjacent inputs must not synthesize a price. Observe live host replacement/clearing without sticky results; isolate environments.
- [ ] Ordinary secure and tainted no-argument calls return the same public pair and preserve caller context. This policy makes no native output-secrecy claim and introduces no secret-argument or extra-argument policy.

## How it works

- [C API architecture](../wiki/systems/c-api.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_transmog_outfit_info/pending_cost_info.rs`: public scalar snapshot type only; no getter or calculation.
- `src/c_api/c_transmog_outfit_info/pending_cost.rs`: direct snapshot handler/registration, pure cost validation against named `MAX_EXACT_LUA_INTEGER`, and primitive public-number pushes. Copies the snapshot and releases the state borrow before validation/push; errors occur before either push and retain host state.
- `src/c_api/c_transmog_outfit_info.rs`: epoch125 type/module export and getter registration immediately after `ensure_namespace`. The existing later direct C API registration populates this exact key after generic bootstrap, replacing its fallback lookup without changing the metatable or earlier-profile wiring.
- `src/lua_api/state/sim_state.rs`: epoch125 optional host input field.
- `src/lua_api/state.rs`: epoch125 absent default.

## Tests asserting this spec

`tests/pending_transmog_cost.rs` contains ten concrete tests, grouped by existing integration auto-discovery, not a new Cargo target:

- `pending_transmog_cost_default_has_zero_returns`
- `pending_transmog_cost_positive_cost_and_combined_flags_are_public_numbers`
- `pending_transmog_cost_zero_values_never_mean_absence`
- `pending_transmog_cost_preserves_full_u32_flags_and_unknown_bits`
- `pending_transmog_cost_repeated_reads_do_not_derive_or_mutate_owned_state`
- `pending_transmog_cost_live_replacement_and_clear_have_no_sticky_result`
- `pending_transmog_cost_environments_do_not_share_snapshots`
- `pending_transmog_cost_safe_integer_upper_boundary_is_exact`
- `pending_transmog_cost_out_of_domain_errors_without_rounding_and_recovers`
- `pending_transmog_cost_secure_and_tainted_public_reads_preserve_context`

Tests call the real namespace function, without fake registrations or replaced callbacks. Fixtures are test data only. All ten tests remain unchanged for this producer cycle. Compiled RED at the real callable placeholder established wrong arity/payload, not an absent callable; compilation failure alone would not qualify. The out-of-domain test includes valid recovery assertions.

### Actual compiled RED — 2026-10-02

At `b584e84f657f5a6b2481f39fc5c24be1674e22bc`, saved `/tmp/patch-12.0.5-batch60-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`: exit0, 332.1104654330993s compilation including unmeasured shared-lock time. `/tmp/patch-12.0.5-batch60-red-run.json` and full `.stdout`/`.stderr` record the selected `pending_transmog_cost::` execution: ten selected, zero PASS, ten genuine FAIL, exit101, 2.2938987109810114s execution. Build/run share integration executable SHA256 `6c08ecd2ee9549d2007535ceb8b9363f437c414984a1de116656519322da7594`. Provenance is dirty-combined; recorded dirty diff hash was supplied, not recomputed.

The final-publication trace correctly identifies `runtime_surface_bootstrap.lua:64–78`, `__wow_namespace_mt.__index`, the original arity explanation was wrong and the trace now corrects it: literal `function() return nil end` returns **exactly one nil**, not zero values. This cached generic callable explains default zero-arity and positive two-arity failures. No old named getter or modeled provider existed. Direct registration now supplies the modeled key; no compatibility fallback is retained for this getter when epoch125 is enabled.

### Saved parent GREEN — 2026-10-02

Producer `84088ea57dd54c247084d516cd293aa62fd8dba7` compiled the default integration target successfully in 152.6985794439679s, including any unmeasured shared-lock time. Full compiler output and executable hashes: `/tmp/patch-12.0.5-batch60-green-build.stdout.jsonl`, `.stderr`, and `-result.json`.

`/tmp/patch-12.0.5-batch60-green-runs.json` binds four finite executions to integration SHA256 `274414f2cd5a5452c03134c9e2a3e293a5fdfd4781c9b3585d6ac66efafa14bb`: ten focused tests, sixteen illusion-category controls, sixty-seven transmog/heirloom controls, and five outfit-catalog controls, **98 distinct PASS**, all exits0. Execution totals 17.81266000075266s; no duplicate test names. Startup separately returned `[]`, exit0, 6.025275836000219s; `green-startup-run.json` binds binary SHA256 `578f459da46570328b371b34ba3b4ebd914c20d797e6d78579cd9e6fd34d1d3d` and full output paths.

Proof is dirty-combined, not a clean-revision or native-client claim. Protected dirty hash remains supplied, not recomputed. Independent security, wiring, readability, scoped formatting and Rust checks remain pending; global formatting has the preserved unrelated unowned failure. Do not rerun applicable build/runtime solely for documentation or accounting.

## Known gaps (current cycle)

- [x] Observe saved genuine compiled RED and implement the separately authorized snapshot getter/registration.
- [x] Parent observed focused GREEN, adjacent controls and startup as recorded above.
- [ ] Independent checks, readability, acceptance, and native parity remain unproved. Row357 receives no coverage/accounting credit here. Inventory remains 206 pending / 141 bounded / 14 partial / 1 metadata, 362 IDs and 66 capabilities.

### Version evidence and correction

Retained `data/patch-api/sources/12.0.0-register.json` declares `cost:number` with `MayReturnNothing`; the exact 12.0.5 row357 delta adds **only** `ret2 = modifierFlags`, not a cost-type change.

The supplied `/tmp/patch-12.0.5-pending-cost-modifier-boundary.md` incorrectly identifies a vendor/mists path as the later documentation authority. Current cached retail `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:377–386` declares `MayReturnNothing`, `cost:BigUInteger`, and `modifierFlags:number`. This is current-cache evidence, not proof of 12.0.5 width or native enum chronology.

Current cached retail `AddOns/Blizzard_Transmog/Blizzard_Transmog.lua:278+` consumes numeric costs with `cost == 0` and numeric comparisons, and uses `FlagsUtil.IsSet` for masks 8, 4, and 2. That consumer supports bounded scalar context and meaningful small fixtures; it does not establish native charges, discount eligibility, selected-patch enum history, or working UI.

## Out of scope

- Transaction creation/editing/commit/revert, catalog-derived cost, price calculations, discounts, events, persistence, affordability, currencies, and automatic slot/sheathe/viewed-outfit synthesis: no backing model authorized.
- Large BigUInteger compatibility, native cost-width limits, native output secrecy, and exact 12.0.5 enum chronology: evidence unavailable; later documentation cannot establish earlier semantics.
- Argument parsing, new secret/extra-argument contracts, generic declassification/security bypasses, and allocation/root machinery beyond primitive pushes: excluded.
- Tests/types/state changes, `c_api/mod.rs`, other collections, protected `aura_duration`, generic metatable, WowlessData/vendor edits, broader audit accounting, batch59 acceptance, shared specs/data/wiki, operations, push, delegation, and gates: excluded from this producer cycle.
