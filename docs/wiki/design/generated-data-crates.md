# Generated data crate boundary

The [contract](../../specs/generated-data-crates.md) separates generated production catalogs from simulator code.

## Boundary

`sim-data` has no client features and depends only on phf. Common tables therefore share a Cargo unit across profiles when toolchain, compilation flags and dependencies match. Items and profession overrides stay together because their lookups refer to each other.

`sim-data-profile` contains atlas variants, global-string variants, the aura attribute table, CVar metadata and profile-pinned item-effect catalogs. Root feature bundles forward only the predicates used by these tables. Original cfg expressions remain on the selected modules. Root `client_profile` still rejects missing or conflicting profiles; leaf crates do not choose a runtime profile. Splitting the selected tables prevents forwarded features from invalidating the large common crate.

Root re-exports preserve existing paths and type identity. Generators write to each crate’s `data/` directory; non-Rust source data and provenance stay in root `data/`. Docker copies both crates before Cargo dependency resolution. The original crate extraction did not regenerate catalog contents.

CVar metadata uses generated PHF maps selected at compile time, with an empty map
for profiles without a pinned native snapshot. These maps contain only captured
flags; current/default values remain in `CVarStorage`. The Python generator
`tools/gen_cvar_info.py` uses only the standard library to read immutable Wowless native-query
expectations and preserves provenance/license files in `data/cvars-info/`.
Its `--check` gate compares generated outputs without runtime JSON loading. See
[CVar metadata](../investigations/cvar-metadata.md)
for snapshot pins and behavior requirements.

Item effects now follow this boundary for all seven profiles. The former shared
mapping and the separate preselected use-spell catalogs are replaced by one
profile-specific effect mapping. Its flat core rows retain item/effect IDs,
signed spell IDs, triggers and legacy slots; sparse effect-ID-keyed metadata
retains nonzero specialization/condition fields, and spell names remain a separate
table. Source-availability flags distinguish an absent metadata column from a
captured zero. Keeping core rows flat avoids another lookup for the common query
path; the sparse metadata avoids repeating mostly zero fields.

`wow-cli generate items --profile PROFILE --build VERSION` owns these outputs and
their provenance in `data/item-effects/`. The no-profile command continues to
generate common item data but no longer emits the moved effect mapping. The four
official Classic profiles read associations from `ItemEffect.ParentItemID`, while
Retail, PTR and Forever use `ItemXItemEffect`. Retail generation merges the pinned
historical mapping with its CSV source to retain existing rows and name bytes;
only the resulting profile catalog participates in runtime lookup. See the
[item-spell contract](../../specs/item-spell-query.md) for source pins, sparse
metadata semantics and reproducible generation options.

## Proof status

Measured on 2026-10-24 in one reused slot with `CARGO_INCREMENTAL=0`, `-j1`, identical toolchain/configuration, 9 GiB memory cap and no swap. These are single warm-dependency samples, not cold builds or a full profile matrix.

| Measurement | Pinned master 08b423c5c | Extracted source |
|---|---:|---:|
| Initial build wall | 438.51 s | 335.44 s |
| Root library unit | 435.61 s | 260.06 s |
| Touched-source rebuild wall | 413.80 s | 258.52 s |
| Rebuild frontend / codegen | 60.78 / 352.00 s | 54.43 / 198.63 s |
| Rebuild maximum child RSS | 6,189,056 KiB | 2,717,272 KiB |

Initial common/selected data units took 50.20/7.36 s; both were reused on the touched-root rebuild. Wrath check explicitly reported common `sim-data` Fresh after Retail check. Rebuild wall decreased 37.5%; peak child RSS decreased 56.1%, from 5.902 → 2.591 GiB. Python `resource.RUSAGE_CHILDREN` records RSS; cgroup MemoryPeak is retained separately and is not RSS. `/usr/bin/time` was unavailable; its failed launch preceded Cargo and is excluded.

Retail and Wrath checks passed without warnings. All 94 bounded Rust tests passed: 53 integration cases plus 37 common catalog and four selected global-string unit cases. Six generator Python tests passed. The Lua refresh-manifest case preserves generated Lua evidence; it does not itself test the moved Rust catalogs. All 23 moved Rust files retain original SHA-256. Root exactly-one-profile validation is byte-identical. Docker COPY wiring and seven-profile feature resolution were checked statically; no image or seven-profile runtime suite was run.

Data unit tests now belong to their owning packages; run `cargo test -p sim-data -p sim-data-profile` with the selected-data feature set appropriate to the desired variant. Root integration discovery is unchanged. The lane handoff retains the full commands, source bindings, HTML timings, output and proof ledger. The measured Rust source is unchanged through 91a4e437e; intervening lock commits only normalize serialization.

## Sources

- [Contract](../../specs/generated-data-crates.md)
- [Workspace manifest](../../../Cargo.toml)
- [Root facade](../../../src/lib.rs)

## See Also

- [[client-profiles]] — runtime profile and epoch selection.
