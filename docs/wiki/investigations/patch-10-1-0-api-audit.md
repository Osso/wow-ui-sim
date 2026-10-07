# Patch 10.1.0 API page audit

Page 230704, revision 2236681 (June 15, 2023, 22:35:50 UTC). Current retail carries 12.1.0; no historical epoch reconstruction.

## Source accounting

129 inventory occurrences; source header counts match. Existing extractor retains 184 non-inventory occurrences, including verbatim Lua/XML examples. Discovery: 83 OK / 46 gaps. Ten unused namespace autostubs retired using the existing retail-12-0-0 module gate; classic registers and cached deprecation wrappers unchanged. Exact retained fixture has 36 gaps, awaiting final GREEN. Qualified and bare cached scans and whole src/tests scans are retained in the evidence directory. No direct retail callers require migration; Mists CanMasterLoot remains classic-only.

## Sources

- [Provenance](../../../data/patch-api/sources/10.1.0-api-changes.provenance.json)
- [Publication contract](../../specs/patch-10-1-0-publication-sweep.md)

## See Also

- [[patch-10-1-5-api-audit]] — audit template.
- [[patch-10-1-7-api-audit]] — publication boundaries.
