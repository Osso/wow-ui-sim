# Evidence compaction

Original mapping: `a82b8eb1c:data/patch-api/evidence/5.5.4-session-2026-10-08/integrated/rebase-mapping.json` (retained in Git history).

9 recorded/rebased scopes retain their exact input names and content. Full reachable rebased tree IDs plus sparse exclusions and recorded blob overrides reconstruct directory inventories; projected input tree IDs preserve the original blob pins. Standalone and historical-exception blob IDs/SHA-256 seals remain explicit. Recorded tree IDs are recomputed from reachable rebased entries, not looked up through pre-rebase commits.

Some originally rebased commits were rebased again before master. Their labels now resolve only through master-ancestor tree snapshots; no original or stale rebased commit/tree object is needed. Where needed, unreachable wiki blobs and changed historical patches are retained as small, hash-checked gzip artifacts (five patches across all sessions). Master-equivalent patch IDs are also checked.

Both the integrated validator and historical replay expand and verify these pins before running their unchanged invariants. Compaction checked every original SHA-256/blob pin and compared expanded inputs, inventories and diff blob maps exactly with the original mapping. Commit/patch-ID checks, conflict exceptions, accounting, receipts and historical artifact checks are unchanged. Only seals for changed files were refreshed; the new helper is hash-pinned by both entrypoints.
