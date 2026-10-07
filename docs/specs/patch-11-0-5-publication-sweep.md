# Patch 11.0.5 publication sweep

Probe retained Warcraft Wiki page 601519, revision 6726778, against unmodified cached Game UI. Default retail carries 12.1.0, not historical 11.0.5. [Audit](../wiki/investigations/patch-11-0-5-api-audit.md) owns evidence and boundaries.

## What it must do

- [x] Probe all 48 inventory occurrences, applying twelve later registers (11.0.7 through 12.1.0) chronologically. Latest add/remove wins; retain original direction and supersession ID.
- [x] Persist observations before exact gap-ID comparison. P1105_SWEEP_OUT selects results; P1105_SWEEP_REGISTER selects a same-sized negative-control register.
- [x] Keep unused C_AuctionHouse.RequestFavorites and C_MajorFactions.GetCovenantIDForMajorFaction absent after repeated raw/ordinary lookup on supported retail epochs, without touching classic profiles or cached deprecation wrappers.
- [x] C_BarberShop.HasAlteredForm returns whether the current host character snapshot contains an alternate-form race. Viewing selection does not change availability. Missing snapshot or missing alternate race returns false. This is a bounded simulator policy, not native form-eligibility parity.
- [x] ChromaEffectsEnable and ChromaEffectsFactionColor have registry default `1`, mutable case-insensitive values, and immutable defaults through global and C_CVar queries. This models configuration storage only, not physical peripheral lighting.
- [x] Retain ten exact known gaps and account for all 48 inventory / 34 extract source rows.
- [x] Run each of thirteen publication sweeps alone, new behavior tests, one-row negative control, relevant isolated prefork cases, formatting, Mists test check with zero non-vendor warnings, and bounded startup returning [].

## Tests asserting this spec

- `tests/patch_11_0_5_publication_sweep.rs` / `tests/common/publication_sweep.rs` — publication, absence, chronological supersession and exact gap accounting.
- `tests/patch_11_0_5_publication_fixes.rs` — repeated retirement lookup, retained neighboring APIs, alternate-form snapshot transitions, independence from viewing selection and Chroma CVar value/default behavior.

## Out of scope

Historical 11.0.5 emulation, 11.x epoch features, native signature/output/security/behavior parity, external linked-page expansion, full suite, agents/models, push and merge. Generic namespace autostubs are not explicit publication; explicitly registered functions may still be placeholders. Active cached consumers must not be retired. Blizzard deprecation wrappers remain untouched.

## Known gaps

- [ ] Ten exact publication/compatibility gaps: [per-ID review](../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-gap-review.json).
- [ ] 24 non-inventory enum/structure contracts: [exhaustive scout](../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-extract-scout.md); candidates only, no behavior credit.
- [ ] Native altered-form eligibility and peripheral lighting behavior. Snapshot availability and CVar configuration storage are bounded simulator contracts.

## Local proof

[Proof ledger](../../data/patch-api/evidence/11.0.5-session-2026-10-06/p1105-proof.json) records command/cwd/environment/revision/outcome scopes. Runtime revision `5fb610fa3`; earlier retirement/barber revision `9ebd57076`. Later sweeps ran at `b3113d0f7`; subsequent Chroma-only registry/test/11.0.5-fixture change does not intersect their symbols/expectations. No later audit inputs changed. Local development proof, not independent/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.0.5 | 48 | 38 | 10 | PASS |
| 11.0.7 | 98 | 70 | 28 | PASS |
| 11.1.0 | 116 | 97 | 19 | PASS |
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Every sweep ran alone through `cargo test --test integration <filter> -- --nocapture --test-threads=1`. Negative control changes only C_BarberShop.HasAlteredForm added → removed: exactly one new gap, none resolved, 10 → 11, expected exit 101. Three behavior tests GREEN after RED; three isolated prefork cases GREEN (cached new surfaces, deprecated Glue boolean, specialization alias identity). Seventeen extractor/register fixtures pass. Mists test check passes with zero non-vendor warnings; six existing iced manifest deprecations plus vendor summary unsuppressed. Separate retail build and bounded exit-0 startup return `[]`.

`cargo fmt` / `cargo fmt --check`, deterministic extract reproduction, all thirteen byte-identical register regenerations and portable artifact validation pass at `2bf022deb` with newly generated reproduction/proof inputs. Later docs/evidence records do not change validated capability/source scopes. Changed Rust lines manually reviewed for readability. Local ignored Cargo logs are not required by the portable validator; tracked summaries retain test outcomes and warning boundaries.

82 IDs: 30 partial-development-green, eight bounded-coverage, 34 audit-pending, ten metadata-only. Seven strict removals and one cached alias acceptance; no effective supersession reversal. Page ledger stays in-progress. No full suite, agents/models, push or merge.
