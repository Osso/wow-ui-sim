# Pending transmog cost: exact row357

`C_TransmogOutfitInfo.GetPendingTransmogCost()` exposes an optional explicit host snapshot as positional `cost, modifierFlags`. This is an **inferred simulator contract**, not native-verified behavior. Input types live in `src/c_api/c_transmog_outfit_info/pending_cost_info.rs`; backing state lives in `src/lua_api/state/sim_state.rs`. See [C API architecture](../wiki/systems/c-api.md).

## What it must do

The inferred bounded requirements below are independently accepted. Native parity and excluded systems remain unproved.

- [x] Under `retail-12-0-5`, accept an explicit optional host snapshot with `cost: u64` and `modifier_flags: u32`; default to `None`, without fabricated production values.
- [x] **Guessed absence policy:** a no-argument query returns zero values for `None`. `Some` returns exactly two ordinary public Lua numbers, including `(0, 0)`, zero cost with nonzero flags, and positive cost with zero flags.
- [x] Return concrete test data `123450, 14` unchanged. `14 = 8 | 4 | 2` uses existing enum assertions as fixture context, not native enum chronology or discount calculation evidence. Preserve all u32 bits, including unknown bits and `4294967295`.
- [x] **Chosen representation limit, not a native cost limit:** return cost exactly through `9007199254740991`; larger host costs raise an explicit error instead of rounding. Failed reads retain the snapshot; later valid replacement or clearing recovers normally.
- [x] Repeated reads leave snapshot, catalog, locks, viewed-slot, viewed-outfit, situation setting, and pending sheathe state unchanged. Those adjacent inputs must not synthesize a price. Observe live host replacement/clearing without sticky results; isolate environments.
- [x] Ordinary secure and tainted no-argument calls return the same public pair and preserve caller context. This policy makes no native output-secrecy claim and introduces no secret-argument or extra-argument policy.

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

Tests call the real namespace function, without fake registrations or replaced callbacks. Fixtures are test data only. All ten tests remained unchanged through the producer GREEN; later readability-only assertion splits preserve identical predicates. Compiled RED at the real callable placeholder established wrong arity/payload, not an absent callable; compilation failure alone would not qualify. The out-of-domain test includes valid recovery assertions.

### Actual compiled RED — 2026-10-02

At `b584e84f657f5a6b2481f39fc5c24be1674e22bc`, saved `/tmp/patch-12.0.5-batch60-red-build-result.json` records `cargo test --test integration --no-run --message-format=json`: exit0, 332.1104654330993s compilation including unmeasured shared-lock time. `/tmp/patch-12.0.5-batch60-red-run.json` and full `.stdout`/`.stderr` record the selected `pending_transmog_cost::` execution: ten selected, zero PASS, ten genuine FAIL, exit101, 2.2938987109810114s execution. Build/run share integration executable SHA256 `6c08ecd2ee9549d2007535ceb8b9363f437c414984a1de116656519322da7594`. Provenance is dirty-combined; recorded dirty diff hash was supplied, not recomputed.

The final-publication trace correctly identifies `runtime_surface_bootstrap.lua:64–78`, `__wow_namespace_mt.__index`; the original arity explanation was wrong and the trace now corrects it: literal `function() return nil end` returns **exactly one nil**, not zero values. This cached generic callable explains default zero-arity and positive two-arity failures. No old named getter or modeled provider existed. Direct registration now supplies the modeled key; no compatibility fallback is retained for this getter when epoch125 is enabled.

### Saved parent GREEN — 2026-10-02

Producer `84088ea57dd54c247084d516cd293aa62fd8dba7` compiled the default integration target successfully in 152.6985794439679s, including any unmeasured shared-lock time. Full compiler output and executable hashes: `/tmp/patch-12.0.5-batch60-green-build.stdout.jsonl`, `.stderr`, and `-result.json`.

`/tmp/patch-12.0.5-batch60-green-runs.json` binds four finite executions to integration SHA256 `274414f2cd5a5452c03134c9e2a3e293a5fdfd4781c9b3585d6ac66efafa14bb`: ten focused tests, sixteen illusion-category controls, sixty-seven transmog/heirloom controls, and five outfit-catalog controls, **98 distinct PASS**, all exits0. Execution totals 17.81266000075266s; no duplicate test names. Startup separately returned `[]`, exit0, 6.025275836000219s; `green-startup-run.json` binds binary SHA256 `578f459da46570328b371b34ba3b4ebd914c20d797e6d78579cd9e6fd34d1d3d` and full output paths.

Proof is dirty-combined, not a clean-revision or native-client claim. Protected dirty hash remains supplied, not recomputed. Independent security, wiring, readability, scoped formatting and Rust checks remain pending; global formatting has the preserved unrelated unowned failure. Do not rerun applicable build/runtime solely for documentation or accounting.

### Independent initial gate and readability follow-up — 2026-10-02

`/tmp/patch-12.0.5-pending-cost-independent-proof.md` and `batch60-independent-gates.json` record independent477 acceptance of saved98PASS/startup0[], security/wiring, owned rustfmt exit0 and default `cargo check` exit0 in27.168454498052597s with no warnings. Global `cargo fmt --check` exits1; potential protected-source diff was discarded unread. This is scoped dirty-combined proof, not unconditional whole-tree acceptance.

Three concrete test-readability findings were accepted after source inspection: split public-result secrecy checks in `assert_pair` and the caller-context probe, and split failure/type/nonempty checks in the out-of-domain test. The predicates remain identical; failure assertions retain short-circuiting through separate `assert` calls. Producer and inputs remain unchanged. Refreshed ten-test runtime and independent equivalence/format/readability follow-up remain pending; controls/startup/production check remain source-valid and must not be rerun solely for this test-only change.

### Independent bounded acceptance — 2026-10-02

Parent accepts independent477 security/wiring/production formatting/check, independent481 equivalent assertion splits/full-test readability/fresh test formatting, and independent483 final artifact audit. `/tmp/patch-12.0.5-pending-cost-final-artifact-proof.md` and `.json` own supplemental reconciliation. Refreshed ten focused tests PASS at compiled `6864b23eeed557b29b27c69fc14e2b1371c40999`, integration SHA256 `7834eb8ebc9a8ab06499ed03dfc38a1454c7829e2af195c772ae17dc528afa5a`, runtime2.6810877189273015s. Shared batch61 compilation114.38749977899715s is separate, not a new producer compile cost. Test SHA256 `36ac6ffc56c498927ed406ea60979984ba74143b7b6f9ed96279fdbc4a5b1665` binds the unchanged29cb readability version through independent481; manifests do not cryptographically embed that source hash.

Ten refreshed focused tests plus88 source-valid historical controls cover **98 distinct tests**, not fresh98 runtime or108 unique tests. Startup/security/wiring/check and production formatting reuse unchanged production; startup binary hash remained identical. Known globalfmt1 and dirty-combined provenance remain explicit. No native, earlier-profile execution, pricing/transaction lifecycle or UI parity claim.

Only `global api-C_TransmogOutfitInfo-GetPendingTransmogCost-357` promotes: **206 pending /141 bounded /14 partial /1 metadata →205 /142 /14 /1**,362 ordered unique IDs,67 capabilities. Prior66 capabilities,361 unrelated rows and retained register/plaintext hashes remain unchanged. `/tmp/patch-12.0.5-batch60-accounting-before.json` and postcommit `batch60-accounting-validation.json` retain exact reconciliation; parent owns validation.

## Known gaps

- [x] Genuine compiled RED, modeled producer, focused GREEN and independent scoped security/wiring/readability/Rust proof.
- [ ] Native absence, cost width, output secrecy, flag chronology, other-profile execution, pricing/lifecycle and consumer UI remain unproved. Broader selected-history goal remains open.

### Version evidence and correction

Retained `data/patch-api/sources/12.0.0-register.json` declares `cost:number` with `MayReturnNothing`; the exact 12.0.5 row357 delta adds **only** `ret2 = modifierFlags`, not a cost-type change.

The original `/tmp/patch-12.0.5-pending-cost-modifier-boundary.md` identified an incorrect vendor/mists authority; the corrected historical note now names the current cached retail source. Current cached retail `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TransmogOutfitInfoDocumentation.lua:377–386` declares `MayReturnNothing`, `cost:BigUInteger`, and `modifierFlags:number`. This is current-cache evidence, not proof of 12.0.5 width or native enum chronology.

Current cached retail `AddOns/Blizzard_Transmog/Blizzard_Transmog.lua:278+` consumes numeric costs with `cost == 0` and numeric comparisons, and uses `FlagsUtil.IsSet` for masks 8, 4, and 2. That consumer supports bounded scalar context and meaningful small fixtures; it does not establish native charges, discount eligibility, selected-patch enum history, or working UI.

## Out of scope

- Transaction creation/editing/commit/revert, catalog-derived cost, price calculations, discounts, events, persistence, affordability, currencies, and automatic slot/sheathe/viewed-outfit synthesis: no backing model authorized.
- Large BigUInteger compatibility, native cost-width limits, native output secrecy, and exact 12.0.5 enum chronology: evidence unavailable; later documentation cannot establish earlier semantics.
- Argument parsing, new secret/extra-argument contracts, generic declassification/security bypasses, and allocation/root machinery beyond primitive pushes: excluded.
- Tests/types/state changes, `c_api/mod.rs`, other collections, protected `aura_duration`, generic metatable, WowlessData/vendor edits, broader audit accounting, batch59 acceptance, shared specs/data/wiki, operations, push, delegation, and gates: excluded from this producer cycle.
