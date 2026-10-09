# Wrath 3.4.1 current factory measurement

Measure literal frozen-source inventory in current bare `WowLuaEnv::new`, `client-wrath`, configured interface 38001. Source Classic TOC 30401 is not the tested client. Historical [source accounting](patch-3-4-1-source-accounting.md) remains immutable. [Audit](../wiki/investigations/patch-3-4-1-api-audit.md).

## What it must do

- [x] Measure all 333 literal occurrences: 211 globals, 44 events, 77 CVars and one Command-kind row; retain raw/lookup and CVar value/default details.
- [x] Pin exact current publication mismatch IDs separately from source assertions and published CVar default differences.
- [x] Apply only actual same-line Wrath successor identities; no foreign-history supersession.
- [x] Expose nondiscriminating event acceptance and console catalog scope, without native behavior credit.
- [ ] Retain exact argv/environment/revision/exit/full streams and input hashes in fresh evidence; preserve all 19 historical seals and ledger counts.

## How it works

- [Audit and limitations](../wiki/investigations/patch-3-4-1-api-audit.md).

## Implementation inventory

- `tests/patch_3_4_1_factory.rs` — whole-file Wrath-gated standalone factory cases; Cargo requires `client-wrath`.
- `tests/common/publication_sweep.rs` — existing classifier, unchanged.
- `data/patch-api/evidence/3.4.1-factory-2026-10-09/` — separately derived inventory and fresh current receipts, never historical observations.

## Tests asserting this spec

`cargo test --no-default-features --features client-wrath --test patch_3_4_1_factory -- --nocapture --test-threads=1`: discovery RED exit 101 at `467acfa6e`; targeted GREEN 6/6 exit 0 at `da31b9ed1`. [Exact proof ledger and full streams](../../data/patch-api/evidence/3.4.1-factory-2026-10-09/proof-ledger.json). Historical 19-seal preservation checked directly, not by rewriting or rerunning historical validators. Receipt retention requirement above is administrative, not a native parity test.

## Known gaps (current cycle)

- [ ] Native Classic publication/behavior, 211 signatures, event payload/dispatch, CVar persistence/effects and Command execution remain UNPROVEN.
- [ ] Both source summaries and linked widget equivalence remain UNPROVEN.

## Out of scope

Production API/model/shim/vendor/cache changes, parser defaults, historical validator rewrites, loaded UI/startup/full Game, other profiles, broad checks/readability/final gates and operations. Models added: NONE. Main owns integration and native final acceptance.
