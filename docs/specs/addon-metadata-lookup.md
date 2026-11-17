# Addon metadata lookup

`C_AddOns.GetAddOnMetadata(addon, field)` and the legacy global `GetAddOnMetadata` read the `## Key: Value` lines of the addon's selected TOC. Large addon databases store rows as metadata: the QuestieDB Mists TOC has 408,495 keys, and Questie reads them one call at a time during login.

## What it must do

- [x] Match `field` against TOC keys ignoring ASCII case, for an addon given by folder name or by 1-based index. Both forms return the same value.
- [x] An exact repeated key keeps the last TOC line; repeated dependency headers (`Dependencies`, `RequiredDeps`, `OptionalDeps` and aliases) keep every line, joined with commas.
- [x] Keys that differ only in case (`X-Case`, `x-case`) resolve to the last such TOC line. Before the index, the result depended on random `HashMap` iteration order per parsed TOC (VERIFIED: 16 parses of one TOC returned all four spelling combinations, overlay proof below).
- [x] Special fields keep their results: `Title` returns the addon title, `Notes` returns nil for empty notes, `Group` returns the folder name, and a missing `Version` returns `@project-version@`.
- [x] A missing key returns nil. For an unknown addon only the exact field `Title` returns a value: the given name, or an empty string for an unknown index.
- [x] Exact-key readers (`AllowLoad`, `AllowAddOnTableAccess`, dependency lists, `Interface`) stay case-sensitive.
- [x] Each lookup costs O(field length), independent of the number of TOC keys. The index is built once when a TOC is parsed.

Native duplicate-key precedence is UNKNOWN; the rules above are simulator behavior. Native case-insensitive matching is the existing simulator contract.

## How it works

- `crates/sim-toc/src/metadata.rs` — `TocMetadata` holds the exact-key map and an ASCII-lowercased key → last-line spelling index. It dereferences to the exact map for exact readers.
- `src/c_api/c_addons.rs` — `addon_metadata` handles the special fields; other fields use `TocMetadata::get_ignore_ascii_case`.
- [Addon loading pipeline](../addon-loading-pipeline.md) — TOC parsing and `AddonInfo` registration.

## Tests asserting this spec

- `tests/addon_metadata_lookup.rs` — fixture TOC through the real loader; case variants, duplicates, dependency joining, locale-suffixed keys, `X-*` keys, empty and whitespace values, misses, unknown addons, name/index/legacy-global forms, and case-variant duplicates across 16 parses.
- `tests/addon_metadata_lookup_perf.rs` (`optional-tests`) — 10,000 lookups on a 400,000-key TOC under a 2 s budget. The previous linear scan took 32.04 s.
- `crates/sim-toc/src/metadata.rs` unit tests — exact reads, last-line spelling, appended lists.
- `tests/addon_api.rs::test_get_addon_metadata` — existing fixture lookups.
