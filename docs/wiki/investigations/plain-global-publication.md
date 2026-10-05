# Retail plain-global publication attribution

The [closure contract](../../specs/patch-12-0-0-publication-sweep.md#plain-global-closure-contract) owns state behavior, epoch boundaries, VM blockers and inferred choices for the assigned 37 globals.

## Root cause

Direct Blizzard aliases keep the destination function's debug source. A classifier checking only whether that source contains `Deprecated` misclassifies legitimate cached deprecation publications. Simulator bootstrap aliases also pre-published retired globals before Blizzard loaded. Gate simulator publication, then attribute loaded cached assignments by exact destination identity; do not rewrite Blizzard Lua.

## Sources

- [Shared sweep](../../../tests/common/publication_sweep.rs)
- [Behavioral proof](../../../tests/publication_deprecated_alias.rs)
- [Closure contract and proof ledger](../../specs/patch-12-0-0-publication-sweep.md#plain-global-closure-contract)

## See Also

- [[patch-12-0-0-api-audit]] — historical source-row classifications
