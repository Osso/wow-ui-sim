# GUI CASC cache preparation

The library regression for GUI CASC cache preparation now uses a test-owned local install. A separate `optional-tests` smoke test uses a live install and preserves startup's explicit error path.

## Install evidence

VERIFIED: On November 8, 2026, the installed Era `.product.db` selected build `5604ce35a8644bdcf0e26c1daf979c15` (`1.15.9.70576`), matching the Era row in `.build.info`. Its build config selected encoding key `6f1851167dd66bfc2e85297b68ef2afd`.

VERIFIED: That key's first nine bytes were absent from all 28 local `.idx` files, including older generations. The inspected indices used the expected v7 format with 9-byte keys, 5-byte archive locations and 4-byte sizes; each sorted-entry region was complete and divisible into 18-byte entries.

INFERRED: The selected local index set could not locate the selected build's encoding file, so cold resolution-cache preparation correctly reported the missing archive location. The observations do not establish whether a partial transfer, content removal, or another update process caused the absence. No new index format or resolver parsing defect was established.

## Test boundary

The ordinary library regression builds a synthetic product database, build config, root file, encoding file, local archive and index. It checks cold cache creation, two FDID mappings, warm reuse, disabled CASC, a missing bootstrap index entry and corrupt indexed content. It does not skip a resolver error or depend on the current game installation. The optional live-install smoke test reports its selected product and path; it skips only when no install is discoverable and fails when preparation fails.

VERIFIED: The `optional-tests` smoke test ran on November 8, 2026, against the selected Era install and failed with the same missing encoding-key index location. This confirms the live test does not hide that resolver error. A successful live smoke against an install with complete local bootstrap content remains unverified.

## Sources

- [CASC loading contract](../../specs/casc-loading.md#gui-resolution-cache-preparation) — preparation behavior and test coverage.
- [Preparation entry point](../../../src/asset_resolver_config.rs) — production behavior and regression tests.
- [Synthetic install fixture](../../../tests/common/gui_casc_cache.rs) — local fixture bytes and expected resolution rows.

## See Also

- [[casc-asset-cache]] — cache ownership and GUI startup sequence.
