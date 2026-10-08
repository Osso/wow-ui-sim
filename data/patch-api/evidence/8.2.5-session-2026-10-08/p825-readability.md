# Changed Rust readability — October 8, 2026

Manual changed-line audit; `rust-code-analysis-cli` is unavailable on this host. No metric execution or numeric metric claim is fabricated.

Reviewed:

- `src/c_api/patch_retired_members.rs`: existing retail-only registration path gains one named list/call; member arrays have no decision branches. Existing long list-dispatch function is unchanged except one call; no scope-expanding refactor.
- `tests/patch_8_2_5_publication_sweep.rs`: real 8.3.0 register replaces placeholder. Literal data remains chronological; no new control flow.
- `tests/patch_8_2_5_publication_fixes.rs`: concrete member identities, explicit raw/lookup/repeat assertions, single loop in shared absence helper. Classic test asserts actual lookup separately. No shape-only behavior test or arbitrary producer value.
- `tests/patch_8_2_5_cached_surfaces.rs`: same behavioral assertions after real cached startup; observable Lua-error count preserved.

No changed-line complexity, excessive nesting, opaque state accumulation, misleading side-effect name, warning suppression, duplicate production logic or parameter-overload violation found. Existing generic retirement machinery is reused; no fallback or vendor mutation added. Cargo format, retail/default and Mists compilation proof retained separately.
