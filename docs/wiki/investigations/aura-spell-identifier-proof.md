# Aura spell-identifier parent proof

Batch37 saved parent proof establishes bounded simulator query behavior at `4b98920f0`; independent310 bounded acceptance is now accepted. [Contract and exact artifact provenance](../../specs/aura-spell-identifier.md#reconciled-batch37-parent-proof--2026-10-01) own the proof ledger.

## Failure and cause

Input `28393b01b` compiled successfully (130.04s), then query fixtures produced 3 PASS / 9 FAIL. Existing numeric/legacy controls passed; missing unit lookup and identifier/alias behavior failed. Existing numeric aura records and explicitly seeded aliases were sufficient backing state; no generic spell catalog or target store was required.

## Producer and saved results

C API-owned epoch queries share the existing C_Spell resolver and aura collector/DTO helper; legacy numeric player global remains unchanged. Producer `4b98920f0` compiled successfully (378.90s): 12 query + 29 aura + 18 admin + 14 C_Spell controls = **73 PASS**. Saved normal no-addon/no-SavedVariables startup exited0 with `[]`. This reconciliation executes no tests/builds; saved parent proof alone was not independent acceptance.

## Inferences and open coverage

Seeded-label resolver reuse, alias precedence, helpful-then-harmful first-match order, missing-unit handling and strict/secret rejection are inferred simulator policies, not native semantics. Player helpful-only/unblocked legacy behavior is preserved; target coverage is the existing harmful fixture only. No native visibility/access/name/link/secret parity, generic spell catalog, refresh duration or all-profile coverage. Parent accepts independent310: 73 selected PASS, saved startup0 `[]`, fresh default fmt/check0 at `6755f0e6c`, no new readability violations. [Exact acceptance](../../specs/aura-spell-identifier.md#independent-bounded-acceptance--2026-10-01) owns bounded coverage matrix, inferences and exclusions.

Accounting **260/88/14 → 258 pending / 90 bounded / 14 partial = 362**; only rows392/396 promoted. Retained IDs/register/plaintext hash and unrelated rows unchanged, verified in explicit before/after artifact linked in contract. Row394/new duration314 separate; future315 outfit verdict unaccepted, row673 pending. PLAN remains ignored local accounting, never staged.

## Sources

- [Aura spell-identifier contract](../../specs/aura-spell-identifier.md) — requirements, implementation inventory, exact saved artifact provenance and limits.

## See Also

- [[lua-api]] — C API ownership and registration boundary.
- [[patch-12-0-5-api-audit]] — retained source accounting.
