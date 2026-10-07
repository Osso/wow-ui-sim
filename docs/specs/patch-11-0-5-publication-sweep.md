# Patch 11.0.5 publication sweep

Probe retained Warcraft Wiki page 601519, revision 6726778, against unmodified cached Game UI. Default retail carries 12.1.0, not historical 11.0.5. [Audit](../wiki/investigations/patch-11-0-5-api-audit.md) owns evidence and boundaries.

## What it must do

- [x] Probe all 48 inventory occurrences, applying twelve later registers (11.0.7 through 12.1.0) chronologically. Latest add/remove wins; retain original direction and supersession ID.
- [x] Persist observations before exact gap-ID comparison. P1105_SWEEP_OUT selects results; P1105_SWEEP_REGISTER selects a same-sized negative-control register.
- [x] Keep unused C_AuctionHouse.RequestFavorites and C_MajorFactions.GetCovenantIDForMajorFaction absent after repeated raw/ordinary lookup on supported retail epochs, without touching classic profiles or cached deprecation wrappers.
- [x] C_BarberShop.HasAlteredForm returns whether the current host character snapshot contains an alternate-form race. Viewing selection does not change availability. Missing snapshot or missing alternate race returns false. This is a bounded simulator policy, not native form-eligibility parity.
- [ ] Retain exact known gaps and account for every inventory and non-inventory source row.
- [ ] Run each of thirteen publication sweeps alone, new behavior tests, one-row negative control, relevant isolated prefork cases, formatting, Mists test check with zero non-vendor warnings, and bounded startup returning [].

## Tests asserting this spec

- `tests/patch_11_0_5_publication_sweep.rs` / `tests/common/publication_sweep.rs` — publication, absence, chronological supersession and exact gap accounting.
- `tests/patch_11_0_5_publication_fixes.rs` — repeated retirement lookup, retained neighboring APIs, alternate-form snapshot transitions and independence from viewing selection.

## Out of scope

Historical 11.0.5 emulation, 11.x epoch features, native signature/output/security/behavior parity, external linked-page expansion, full suite, agents/models, push and merge. Generic namespace autostubs are not explicit publication; explicitly registered functions may still be placeholders. Active cached consumers must not be retired. Blizzard deprecation wrappers remain untouched.

## Known gaps

Per-ID producer/model/policy boundaries and all local proof will be linked from the audit. Publication accounting is not behavioral completion.
