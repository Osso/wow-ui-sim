# Patch 10.2.0 API page audit

Page 12983, revision 6473470 (September 15, 2025, 16:50:03 UTC), retrieved October 7, 2026 UTC. Branch `p1020-page` starts from master `e403fd39e`. Default retail carries 12.1.0, not reconstructed 10.2.0.

## Source and accounting

150 inventory occurrences: 79 added, 64 removed, seven changed. All eight headers match: global API 46/46, widgets 15/5, events 4/9, CVars 14/4. Eighteen later registers, 10.2.5 through 12.1.0, supersede chronologically (10.2.5 added at integration; the thirty exact gaps are unchanged); three expectations reverse direction. The independent 10.2.5 register is deliberately not an input yet. Adding it before 10.2.6 requires one later-register list entry.

284 unique source IDs: 150 inventory + 134 extract. Ledger counts: 64 bounded-coverage, 51 partial-development-green, 151 audit-pending, 18 metadata-only. Inventory pending comprises 30 exact publication gaps and five serialized-default mismatches. All 116 substantive extract occurrences remain pending: 33 enumeration, four constant, 28 structure, 19 deprecated API, 21 texture-slicing/example and 11 prose rows. Eighteen editorial/source rows grant no runtime credit. Raw wikitext is authoritative; normalized plaintext is not executable Lua and does not preserve Lua long-string delimiters.

Parser/extractor extensions have RED/GREEN fixtures: skip the `Widget Scripts` category label without dropping handler occurrences, retain `widget-script` kind, preserve XML example tags, and retain reference-list context. All eighteen registers reproduce byte-identically. Initial wrong `--inventory-only` attempts for 12.0.5/12.0.7 are recorded, then superseded by correct reproduction. All 102 previously retained source/register/coverage/fixture inputs remain unchanged.

## Bounded closures

Discovery: 110 OK / 40 failures. Final: 120 OK / 30 exact gaps. Ten closures comprise eight unused retirements and two corrected interface probes; parent-key behavior is separately bounded:

- Retail/PTR no longer fabricate `C_Console.GetFontHeight`, `PrintAllMatchingCommands` or `SetFontHeight`. Qualified cached retail searches find no callers. Bare-name matches identify current global Console* successors or unrelated font methods, not removed namespace callers. Global console successors stay published.
- Retail/PTR no longer register legacy `GetNumAddOns`, `IsAddOnLoaded`, `GetAddOnEnableState`, `IsAddOnLoadOnDemand` or `LoadAddOn`. Token-boundary global-call searches find no cached retail callers; broad bare searches are disambiguated from C_AddOns members and helpers. C_AddOns successors and `GetAddOnMetadata` remain. Classic retains original registrations. Cached Blizzard deprecation files are untouched.
- `Object` is an interface, not a creatable frame type. The classifier now uses a Frame implementing that interface for `Object:ClearParentKey` and `Object:SetParentKey`.
- ClearParentKey was a no-op. It now clears referring Rust `children_keys`/`parent_key` state and matching Lua parent properties. Concrete Frame/Texture fixtures cover forced key replacement, repeated clears and preservation of another child's mapping. Parentless objects, all object kinds, native protection/errors and ambiguous multiple-alias policy remain unproved.
- Removed movie subtitle scripts previously passed because constructing a frame named after the script failed. The register now identifies scripts, and the classifier constructs MovieFrame and queries HasScript. Both absent-script rows pass this direct support probe; no handler dispatch or subtitle rendering credit.

No 10.x/11.x epoch, new placeholder, vendor patch or Blizzard monkey-patch introduced. Retirement namespace module remains disabled for classic; addon global registration uses the existing active client profile.

## Retained boundaries

[Per-ID review](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-gap-review.json) assigns every discovery failure, ten closures and thirty precise remaining reasons. Three Button highlight owner-removal rows remain: current cached callers use inherited Frame methods, so removing them from Buttons would break current UI. Exact file:line citations and qualified/bare searches are retained. Two C_Console real producers remain registered pending migration of existing namespace consumers/tests; they are exact removal gaps, not attributed to Blizzard wrappers. Who queries, addon reset/dependency metadata, CVar metadata, LFG availability, party item-level aggregation, perks freeze state, display/transformation identity, brawl identity, atlas record IDs, staged trait cascades and weekly activity history need models/contracts rather than inert functions. Line hit thickness requires input hit-region behavior. Tabard/actor 3D behavior remains intentionally unsupported.

Five CVars pass publication but differ in formatting: page `5.0`, `8.0`, `2.5`, `5.0`, `20.0` versus current six-decimal strings. Values are numerically equal; classifier reports literal mismatch separately. Ledger grants no historical serialized-default parity.

[Extract scout](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-extract-scout.json) assigns every non-inventory ID once. Numeric enum values/renames, constant identities, populated DTO additions/nilability, all legacy call migrations, texture slicing/render/XML examples, portrait argument restrictions, early addon-load ordering, new-character defaults, NaN keys and combat protection remain explicitly pending. Publication and source candidates are not behavioral proof.

Read-only inspection of the concurrently retained [10.2.5 register](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-possible-1025-supersessions.json) finds no add/remove-symbol intersection with the final thirty gap IDs. This snapshot does not include future changes or effects of that branch's runtime fixes. Recompute exact fixture after integration, as required by the handoff.

## Verification

Current runtime/test revision `06326ad3e`. Three new behavioral/probe tests and full cached Game prefork pass; parent-key and eight retirement boundaries have RED/GREEN evidence. Four existing SetParentKey cases and one cached groupfinder parent-key case pass at the earlier, unchanged parent-key implementation scope. Eighteen isolated publication sweeps pass exact fixtures; [table](../../specs/patch-10-2-0-publication-sweep.md#local-proof). Negative control changes only TextureBase:GetTextureSliceMargins added → removed: exactly one new failure, no resolved failures, 30 → 31, expected exit 101.

Cargo fmt --check and Mists tests check pass with zero non-vendor warnings. Six iced vendor manifest deprecations plus their summary remain unsuppressed. Separate current-retail binary build passes; bounded startup exits 0 with JSON `[]`. Twenty parser/extractor fixtures pass. Saved outputs are searched instead of rerunning commands for logs. Earlier proof invalidated by subsequent addon-global changes is explicitly superseded by final runs; unchanged parser/parent-key scopes retain their proof.

Changed Rust manually audited against the readability checklist: short state-clearing function, data-only markers, bounded profile registration and test probes; no changed-line violations found. All commands use explicit cwd `p1020-page` and its own target. No sibling target reuse, canonical working-file edits, other-worktree edits, full suite, agents/models/CLIs, push or merge. Worktree creation used only the prescribed canonical Git metadata operation, with cwd in the empty destination.

Portable artifact validator checks hashes, every source ID, all chronological expectations/fixtures, ten closures, thirty gaps, five numeric-equal default mismatches, exact negative control, eighteen reproduced registers and 102 preserved inputs. Artifact validation passes at `18797a33c`: [result](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-validation-result.json). Subsequent result/documentation-only changes preserve runtime and accounting scopes. Proof remains local targeted development evidence, not native or independent acceptance.

## Sources

- [Provenance](../../../data/patch-api/sources/10.2.0-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.2.0-page-coverage.json)
- [Publication contract](../../specs/patch-10-2-0-publication-sweep.md)
- [Qualified/bare retirement searches](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-retirement-consumers.json)
- [Disambiguated legacy global calls](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-legacy-global-consumers.json)
- [Register reproduction](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-register-reproduction.json)
- [Proof ledger](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-proof.json)
- [Artifact validator](../../../data/patch-api/evidence/10.2.0-session-2026-10-07/p1020-validate.py)

## See Also

- [[patch-10-2-6-api-audit]] — audit template and first integrated later register.
- [[patch-10-2-7-api-audit]] — occurrence/proof accounting conventions.
- [[client-profiles]] — retail/classic boundaries.
