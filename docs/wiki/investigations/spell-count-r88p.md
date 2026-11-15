# Spell-count argument and inventory corrections

lt-67d2 separates proved Retail R88p/R88k behavior from obsolete assertions and unknown conversion details. The [contract](../../specs/spell-count-outputs.md) owns current API scope; tracker receipts own revision-specific build and runtime results.

## Three inherited failures

- `finite_threshold_and_utf8_nul_free_cstring_validate_without_source` first fails at `302d8767af7b34246d1c0d1e8199dfc68c260cbb`; immediate parent `8320404c2c6245522becc202296a6eaec9e5f2c2` passes. The producer began accepting numeric-string maxima, as VERIFIED by R88k.5 line11622. The old rejection of `"6"` was wrong. The same native line also contradicts rejecting ordinary numeric replacement text.
- The alias and authenticated-type selectors first fail at intentional test-RED commit `e8b46736ac26e5a30f5f62263ebcc24e2b2d0285`; parent `d69ee1397f74068495d87ad4d0f6478a36a4b3e7` passes. The fixture starts supplying reagent state before its producer exists, so its first positive check gets0 rather than7.
- `a1cfb4a2c60ba65627f6734e84df215f31681e3e` repairs that fixture/producer boundary and first exposes the current alias7-versus2 and authenticated-fraction Lua assertion failures. Its parent `596534f337904dbd506f89031a1433845ada1f38` still has the fixture failure. Numeric override rules and authenticated fractional conversion remain UNKNOWN (R88k.5 line11623), so those assertions were removed separately under coordinator #1226 rather than changing the producer to invent semantics.

All six adjacent historical boundaries were compiled from exact archives and the three exact selectors executed at each. Archive-preserved old mtimes initially contaminated Cargo cache reuse; those attempts were invalidated, preserved separately, and excluded. Corrected runs refresh every archived input timestamp without changing bytes, and verify `fresh:false` for the data library, simulator library, test dylib and integration executable. Linux integration is a thin executable: its hash alone is not standalone runtime identity. Historical linked-library hashes were not captured; exact source hashes, fresh rebuilds and exclusive sequential immediate runs establish the reported boundaries.

## Native-backed implementation

R88p.p3 supplies signed16 reagent quantities, first-eight-pair selection, sum-before-divide/minimum arithmetic, carried roots/containers and mask32 bank-tab admission. Explicit empty metadata and parent maps extend existing inventory backing; they do not acquire remote bank data or fabricate catalog rows. Equipment/container representations are deduplicated. Mask2 alone retains ordinary arithmetic because qualifying-property backing is absent; the alternate helper remains empty.

R88k.5 line11622 supplies numeric-string maximum acceptance and ordinary numeric replacement text. R88p.p2 supplies signed maximum range validation, truncation, negative thresholds and access-denial reasons. Exceptional numeric identifier outcomes depend on UNKNOWN FP configuration; the approved unresolved-record seam removes a contradicted range error without wrapping, clamping, or asserting the neutral result.

The four initial RED selectors each include two distinct backed results and absence: signed quantities/eight pairs, default bank exclusion, carried roots and numeric replacements. Additional traversal/deduplication/mask2 tests are further coverage, not retroactive base-RED claims. Detailed eligibility, native property decoding, active-record limits and dispatcher triggers remain UNKNOWN.

## Sources

- lt-67d2 #1207, #1215, #1226 and #1231 — native evidence and correction dispositions.
- lt-c3ed #1056 — R88k.5 publication/argument evidence, including line11622.
- [Spell count contract](../../specs/spell-count-outputs.md) — current and historical scope.
- [API tests](../../../tests/spell_count_outputs.rs) — independent backed inputs and absence controls.
- [Inventory model](../../../src/c_api/spell_reagent_inventory.rs) — metadata/containment seams and traversal.

## See Also

- [[lua-api]] — C_* boundary and state ownership.
- [[consumer-selection-and-red-proof]] — original-consumer applicability versus API proof.
