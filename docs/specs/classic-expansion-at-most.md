# Classic expansion upper-bound predicate

`ClassicExpansionAtMost(expansionLevel)` is declared in the Retail and Forever `ExpansionInfoDocumentation.lua` files; Mists keeps its existing compatibility-bootstrap implementation. Retail's declared argument is a required number and its result is a non-nil boolean. The original UnifiedProfileManager LibDualSpec consumer calls it while building options. The Forever profile also uses it in LibDualSpec.

## Contract

- [x] Retail and Forever compare the numeric argument, without integer truncation, against the same temporary Classic level `10` already returned by `GetClassicExpansionLevel()` and used by `ClassicExpansionAtLeast()`; return one boolean.
- [x] Keep the independently inferred Forever `GetExpansionLevel()` and `LE_EXPANSION_LEVEL_CURRENT` values at `0`. Preserve the existing Classic getter and AtLeast argument policy, Mists bootstrap behavior, and an already registered global.
- [x] Reject missing or nonnumeric arguments. Accept a secret number from an untainted caller; reject a secret argument from a tainted caller without declassification.
- [x] The unchanged BigWigs LibDualSpec thresholds at Cataclysm (`4`) and Shadowlands (`8`) evaluate false against the temporary Classic level.

## Implementation inventory

- `src/lua_api/workarounds/temporary/client_info_defaults.rs` — one shared temporary Classic level compatibility policy with a retirement condition, separate from current-expansion identity.
- `src/lua_api/globals/stubs/global_stubs.rs` — Classic getter/AtLeast behavior and Retail/Forever upper-bound predicate consult the same policy.
- `tests/classic_expansion_at_most.rs` — Forever numeric boundaries, fractional threshold, BigWigs-shaped branch, argument security and bootstrap retention.
- `tests/retail_classic_expansion_at_most.rs` — Retail function availability and the LibDualSpec-style conditional consumer.

## Known gaps

- [ ] Native Forever Classic level and predicate results are unverified. The `10` policy is a compatibility default, not proof of native metadata; replace it when authoritative per-client Classic expansion metadata is available.
- [x] Independent verification reuses the hash-matched 3/3 focused proof, formatting, default compilation, frozen binary, exact 442-file archive, and isolated root replay: `/tmp/forever-addon-audit/verify-classic-expansion-at-most-ledger.json`.
- [x] The unchanged 442-file BigWigs archive loads its root with zero Lua errors on frozen `fbcd9bb51`; `/tmp/forever-addon-runtime/bigwigs-classic-upper-final-2k58pia8/ledger.json` records the exact binary, roots, and isolated state.
- [ ] BigWigs Core/Options/Plugins remain deferred LoadOnDemand. All eight encounter TOCs use `AllowLoadGameType: standard`, which Forever excludes; they are not only Midnight roots. `/bw` now reaches deferred plugin activation but fails at `BigWigs_Plugins/Auras.lua:2408` on missing `CanBeAccessedInContext`. Major workflows remain untested. The package is partial/unloaded, not a clean all-root startup.
