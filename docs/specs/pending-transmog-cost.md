# Pending transmog cost: exact row357

`C_TransmogOutfitInfo.GetPendingTransmogCost()` exposes an optional explicit host snapshot as positional `cost, modifierFlags`. This is an **inferred simulator contract**, not native-verified behavior. Input types live in `src/c_api/c_transmog_outfit_info/pending_cost_info.rs`; backing state lives in `src/lua_api/state/sim_state.rs`. See [C API architecture](../wiki/systems/c-api.md).

## What it must do

All requirements remain unverified; this cycle supplies inputs and tests only.

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
- `src/c_api/c_transmog_outfit_info.rs`: epoch125 type/module export through the existing public outfit module; registration unchanged.
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

Tests call the real namespace function, without fake registrations or replaced callbacks. Fixtures are test data only. No tests executed in this inputs-only cycle. Expected genuine RED is a compiled test failure at a real call to the absent getter; compilation failure alone does not qualify. The out-of-domain test must also reach its valid recovery assertion, so an absent getter cannot satisfy the test through error assertions alone. Producer changes require actual compiled RED first.

## Known gaps (current cycle)

- [ ] Compile and observe genuine RED; subsequently implement the getter/registration in a separately authorized producer cycle.
- [ ] GREEN, checks, independent acceptance, and native parity remain unproved. Row357 receives no coverage/accounting credit here.

### Version evidence and correction

Retained `data/patch-api/sources/12.0.0-register.json` declares `cost:number` with `MayReturnNothing`; the exact 12.0.5 row357 delta adds **only** `ret2 = modifierFlags`, not a cost-type change.

The supplied `/tmp/patch-12.0.5-pending-cost-modifier-boundary.md` incorrectly identifies a vendor/mists path as the later documentation authority. Current cached retail `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:377–386` declares `MayReturnNothing`, `cost:BigUInteger`, and `modifierFlags:number`. This is current-cache evidence, not proof of 12.0.5 width or native enum chronology.

Current cached retail `AddOns/Blizzard_Transmog/Blizzard_Transmog.lua:278+` consumes numeric costs with `cost == 0` and numeric comparisons, and uses `FlagsUtil.IsSet` for masks 8, 4, and 2. That consumer supports meaningful small scalar fixtures; it does not establish native charges, discount eligibility, or selected-patch enum history.

## Out of scope

- Transaction creation/editing/commit/revert, catalog-derived cost, price calculations, discounts, events, persistence, affordability, currencies, and automatic slot/sheathe/viewed-outfit synthesis: no backing model authorized.
- Large BigUInteger compatibility, native cost-width limits, native output secrecy, and exact 12.0.5 enum chronology: evidence unavailable; later documentation cannot establish earlier semantics.
- Getter/registration and any other producer edits before actual compiled RED; broader audit accounting, batch59 acceptance, shared specs/data/wiki, operations, push, delegation, and gates: excluded from this cycle.
