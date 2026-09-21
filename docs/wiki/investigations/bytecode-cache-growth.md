# Lua Bytecode Cache Growth Bound

Commit `39caf2662` bounded the packed Lua bytecode cache after a valid cache grew beyond available memory during isolated addon tests. The fix rejects oversized packs before payload allocation and enforces the serialized size limit on every store, while preserving warm-cache reuse when compaction can fit.

## Content

### Symptoms and forensic evidence

An isolated `Blizzard_AchievementUI` smoke test stalled while loading the first baseline addon, `Blizzard_SharedXMLBase`. The cache pack for the clean proof checkout was **32,316,662,045 bytes** (~30.1 GiB), with a valid `WOWBC002` header and whitelist version. A bounded scan found **25,256,275 entries** and **25,256,274 unique hashes**; the pack parsed through exact EOF with no torn entry or trailing garbage. Growth was therefore millions of distinct appended entries, not ordinary duplicate accumulation.

### Root cause

`MAX_PACK_SIZE` was enforced only when a process first loaded an existing pack. The loader called `read_to_end` before checking the file size, so an oversized valid pack could be materialized in memory before compaction or deletion. During the process lifetime, `put()` and legacy-key promotion appended entries without checking the serialized pack size, and mutated in-memory state before persistence succeeded. Repeated generated chunks could therefore grow both the file and `CacheState` without a bound.

### Size-bound fix

`39caf2662` applies the bound at both load and store boundaries:

- inspect file metadata and perform a bounded read before parsing an existing pack;
- calculate serialized header/entry size before append;
- compact live entries before adding a replacement when the append would exceed the limit;
- rebuild with only the new entry when the compacted live set still cannot fit;
- reject entries larger than the limit;
- persist replacement data before replacing `CacheState`, and roll back failed appends;
- route legacy-key promotion through the same bounded store path.

This keeps the warm full-addon cache path while preventing unbounded in-process growth. It does not claim a broader cache eviction policy or a new generated-chunk persistence policy.

### Compiler identity

`8ddf0908d` binds persisted chunks to the exact 40-hex Rilua Git revision resolved from `Cargo.lock`. The revision is part of both the pack header and each content key, alongside the existing whitelist ABI. A compiler correction therefore invalidates stale bytecode without a manual cache-version bump or user-cache deletion.

The build fails when the lock record has no uniquely identified Git-sourced Rilua revision. `WOWBC003` rejects older/unidentified packs before replay. Loose `.luac` artifacts and legacy/unversioned keys are ignored instead of migrated or promoted. Exact-revision packs retain normal replay, bounded storage, read-only behavior, and prefork modes.

This closes the implementation gap only. Real Ellesmere cold/rejected-stale/warm replay remains parent-owned acceptance work.

### Proof

The `39caf2662` focused proof covered bounded oversized-pack rejection, compaction, rebuild, oversized entries, and failed-append rollback in `src/loader/bytecode_cache.rs`. Its legacy-promotion case is historical: `8ddf0908d` deliberately removes that behavior.

Current cache tests cover exact compiler-key replay, foreign-compiler pack rejection, ignored loose legacy files, read-only snapshots, pack bounds, torn entries, and prefork modes. `tests/locked_rilua_identity.rs` separately proves exact lock-record extraction and rejects missing or ambiguous compiler identity. The failing isolated loader boundary was reproduced with `WOW_SIM_TRACE_LOAD_ADDON=1`; parent-owned real Ellesmere replay remains the acceptance boundary.

## Sources

- [bytecode_cache.rs](../../../src/loader/bytecode_cache.rs) — packed cache format, bounded load/store paths, compiler identity, and focused regression tests
- [compiler bytecode cache spec](../../specs/compiler-bytecode-cache.md) — compiler-identity contract and pending real-addon acceptance
- [Track 3 global-slot ABI](../design/track-3-global-slot-abi.md) — bytecode-cache versioning and slot-ABI invalidation context

## See Also

- [[track-3-global-slot-abi]] — cache versioning protects the global-slot ABI; this page covers size, compiler identity, and persistence bounds
- [[ellesmereui-forever]] — compiler correction consumer with pending cold/stale/warm replay acceptance
- [[talent-performance]] — earlier startup profiling identified cold bytecode-cache reuse as a performance concern
