# Forever Classic expansion upper-bound predicate

`ClassicExpansionAtMost(expansionLevel)` is exposed for the Forever profile. Cached `ExpansionInfoDocumentation.lua` specifies a required number, boolean result, and secret arguments accepted only from untainted callers. BigWigs `8931513` invokes it during LibDualSpec initialization.

## Contract

- [x] Compare the numeric argument, without integer truncation, against the same temporary Classic level `10` already returned by `GetClassicExpansionLevel()` and used by `ClassicExpansionAtLeast()`; return one boolean.
- [x] Keep the independently inferred Forever `GetExpansionLevel()` and `LE_EXPANSION_LEVEL_CURRENT` values at `0`. Preserve the existing Classic getter and AtLeast argument policy, other client profiles, and an already registered global.
- [x] Reject missing or nonnumeric arguments. Accept a secret number from an untainted caller; reject a secret argument from a tainted caller without declassification.
- [x] The unchanged BigWigs LibDualSpec thresholds at Cataclysm (`4`) and Shadowlands (`8`) evaluate false against the temporary Classic level.

## Implementation inventory

- `src/lua_api/globals/stubs/global_stubs.rs` — one shared temporary Classic level policy for existing helpers and the Forever-only upper-bound predicate. No replacement for current-expansion identity or other-profile behavior.
- `tests/classic_expansion_at_most.rs` — numeric boundaries, fractional threshold, BigWigs-shaped branch, argument security and bootstrap retention in the grouped integration target.

## Known gaps

- [ ] Native Forever Classic level and predicate results are unverified. The `10` policy is a compatibility default, not proof of native metadata; replace it when authoritative per-client Classic expansion metadata is available.
- [ ] Unchanged BigWigs startup, deferred roots, and major workflows require separate evidence. Predicate tests alone are not addon compatibility.
