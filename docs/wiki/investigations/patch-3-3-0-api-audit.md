# Retail Patch 3.3.0 API audit

Frozen historical Retail source, verified 2026-10-09: page 522376/revision 6055853/time 2024-06-04T04:47:16Z. Manifest response/wikitext hashes match; response content equals raw source. Owned base `9ff195d19`; main owns integration and acceptance.

## Literal accounting
| Scope | Exact coverage | Proof / limits |
|---|---|---|
| Inventory | 11 occurrences: eight added, one removed, two changed references | Nine current-retail publication/absence matches; two Texture dimension gaps. No callable/default behavior credit. |
| Full extract | 29 rows: 13 metadata, 16 original prose gaps | Attribution, inline Button XML, namespace, packed NPC GUID example, macro target/vehicle conditionals and unexpanded diff retained. |
| Signatures | Seven explicit NEW call signatures | Argument/output text retained separately; all seven historical/native contracts remain pending. |
| Existing-model closure | One XML `motionScriptsWhileDisabled` row | Actual loader/template true/false, inheritance, pre-OnLoad and Lua mutation pass; existing disabled-button pointer dispatch passes. |
| Original/current ledgers | 47 IDs each; original 9 bounded/25 pending/13 metadata, current 10/24/13 | Original sealed independently; closure updates only `prose-undated-024`. No native 2009 client parity. |

`GetObjectType` is a named successor reference, not a new 3.3.0 API. `MouseIsOver` is a replacement reference, not an inferred 3.3.0 removal. Quest completion response event is an explicit separate occurrence. No transcluded or linked page expansion: Special:Diff/4997637/4997649 and Iriel's forum post remain source boundaries.

## Retail ordering
Current sweep uses actual ordered 3.3.3, 3.3.5, 4.0.1 inputs before the 68 Retail 4.1+ registers. Main added these after rebase onto `7a7a29519`; none has section/symbol overlap with the 11 own occurrences. Original placeholder capture, archives and seals remain unchanged. Extractor conflict resolution preserves both independent `cataclysm_labeled_inventory` and `wrath_summary_markup` opt-ins; default/current-byte replay is an integration proof obligation. Wrath Classic 3.4.x never supersedes this historical Retail page. Parent registry continues through 1.0.0.

Four actual later removals govern current publication: QueryQuestsCompleted and QUEST_QUERY_COMPLETE by 5.0.4, GetQuestsCompleted by 9.0.1, MouseIsOver by 12.1.0. Do not restore these APIs to satisfy historical prose. GetFrameType already absent; no retirement or source removal performed, so Classic API preservation is untouched. No cached/vendor/Wowless edits.

## Established model closure
`FrameXml` silently discarded `motionScriptsWhileDisabled`. Deserializing the literal bool and applying it to the existing Frame motion flag fixes actual XML loading and Lua CreateFrame template application before scripts. Derived/instance false overrides inherited true. No new model, shim or fallback.

Texture:GetFileWidth/GetFileHeight require real decoded-file metadata and inaccessible-file semantics; layout or atlas display size is not actual source-file size. `registerForClicks` remains pending because this source establishes no token/delimiter grammar. Packed historical GUID formatting, server query throttling/transport, target-specific vehicle macro state and seven signature contracts remain unsupported at this proof boundary. Reasons are occurrence-specific in the ledger.

## Development proof
- Source parser RED → GREEN 2/2; current fixture 2/2. Default parser outputs/errors match base on all 77 retained raw inputs; no claim that previously unsupported default parsing now succeeds.
- Own publication RED exposes two texture gaps → known-gap GREEN 1/1. Impossible-method serialized negative changes two gaps to three and fails exactly as required.
- XML real-loader RED → GREEN 1/1; existing real pointer disabled enter/leave model test 1/1. Changed Rust files formatted before commits. Six inherited iced-wgpu manifest deprecations remain unmodified.
- Portable fresh copied process passes 1/1 fixture; five serialized source/own-log/original-ledger/archive/manifest tamper controls fail, restored replay matches. No Git, target, current source files or network needed.
- Original and closure manifests seal 24 files and 166 archived code/source files across separate archives. Counts, register sets, gap sets, signatures and receipt summaries are derived dynamically. No broad/profile/startup/full-suite/final acceptance gates run; no push/merge/deploy/delegation/rebase.

## Sources
- [Frozen source](../../../data/patch-api/sources/3.3.0-api-changes.wikitext)
- [Spec](../../specs/patch-3-3-0-publication-sweep.md)
- [Handoff and reproduction](../../../data/patch-api/evidence/3.3.0-session-2026-10-09/handoff.md)
- [Original ledger](../../../data/patch-api/evidence/3.3.0-session-2026-10-09/historical-page-coverage.json)
- [Current ledger](../../../data/patch-api/sources/3.3.0-page-coverage.json)
- [Portable summary](../../../data/patch-api/evidence/3.3.0-session-2026-10-09/validator-summary.json)

## See Also
- [[patch-4-1-0-api-audit]] — actual later Retail source.
