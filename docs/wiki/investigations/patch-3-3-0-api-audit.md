# Retail Patch 3.3.0 API audit

Frozen historical Retail source, verified 2026-10-09: page 522376/revision 6055853/time 2024-06-04T04:47:16Z. Manifest response/wikitext hashes match; response content equals raw source. Owned base `9ff195d19`; main owns integration and acceptance.

## Literal accounting
| Scope | Exact coverage | Proof / limits |
|---|---|---|
| Inventory (saved accounting) | 11 occurrences: eight added, one removed, two changed references | Saved receipt records nine current-retail publication/absence matches and two Texture dimension gaps. Active-profile applicability/retirement of those methods remains under investigation; not proof of generic missing native APIs or callable behavior. |
| Full extract | 29 rows: 13 metadata, 16 original prose gaps | Attribution, inline Button XML, namespace, packed NPC GUID example, macro target/vehicle conditionals and unexpanded diff retained. |
| Signatures | Seven explicit NEW call signatures | Argument/output text retained separately; all seven historical/native contracts remain pending. |
| Existing-model closure | One XML `motionScriptsWhileDisabled` row | Actual loader/template true/false, inheritance, pre-OnLoad and Lua mutation pass; existing disabled-button pointer dispatch passes. |
| Saved original/closure ledgers | 47 IDs each; original 9 bounded/25 pending/13 metadata, closure 10/24/13 | Historical saved accounting, not today's model-completion count. Original sealed independently; saved closure updates only `prose-undated-024`. No native 2009 client parity. |

`GetObjectType` is a named successor reference, not a new 3.3.0 API. `MouseIsOver` is a replacement reference, not an inferred 3.3.0 removal. Quest completion response event is an explicit separate occurrence. No transcluded or linked page expansion: Special:Diff/4997637/4997649 and Iriel's forum post remain source boundaries.

## Retail ordering
Current sweep uses actual ordered 3.3.3, 3.3.5, 4.0.1 inputs before the 68 Retail 4.1+ registers. Main added these after rebase onto `7a7a29519`; none has section/symbol overlap with the 11 own occurrences. Original placeholder capture, archives and seals remain unchanged. Extractor conflict resolution preserves both independent `cataclysm_labeled_inventory` and `wrath_summary_markup` opt-ins; default/current-byte replay is an integration proof obligation. Wrath Classic 3.4.x never supersedes this historical Retail page. Parent registry continues through 1.0.0.

Four actual later removals govern current publication: QueryQuestsCompleted and QUEST_QUERY_COMPLETE by 5.0.4, GetQuestsCompleted by 9.0.1, MouseIsOver by 12.1.0. Do not restore these APIs to satisfy historical prose. GetFrameType already absent; no retirement or source removal performed, so Classic API preservation is untouched. No cached/vendor/Wowless edits.

## Established model closure
`FrameXml` silently discarded `motionScriptsWhileDisabled`. Deserializing the literal bool and applying it to the existing Frame motion flag fixes actual XML loading and Lua CreateFrame template application before scripts. Derived/instance false overrides inherited true. No new model, shim or fallback.

## Current bounded reconciliation
Read-only source/code inspection, 2026-10-10. Frozen source says the AddOn's Lua files gain a private table; `@` is a synonym for `target=`; XML gains `registerForClicks`; texture dimensions refer to the actual file, returning zero when inaccessible. It separately distinguishes player `vehicleui` from macro-target `unithasvehicleui`. No linked-page expansion or broad source count establishes model completion.

| Status | Exact statement / current coverage | Proof boundary |
|---|---|---|
| Handled; tests present | `@focus` / `target=focus` selection equivalence: `src/lua_api/globals/security/cmd_option.rs::parse_unit_override`; `tests/cmd_option_selected_unit.rs` checks selected unit and unchanged cached vendor handler; `tests/mouse_tm_commands.rs` checks `/tm` focus state changes. | Inspected assertions, not newly executed proof; no native 2009 parity or `/cast` execution claim. Target-specific vehicle state remains separate. |
| Handled at file-loading boundary | `src/loader/tests/lua_loading.rs::test_multi_file_closures` shares the private addon table across three Lua files and asserts the resulting closure-driven update. | Helper creates one table and calls `load_lua_file` with a shared `AddonContext`; not an actual TOC-level addon-loading test. No new execution. |
| Missing XML attribute; shared fix RED pending | Current `src/xml/types.rs::FrameXml` has no `registerForClicks` attribute. Cached PTR `Blizzard_ItemButton/Mainline/ItemButtonTemplate.xml:23` uses `LeftButtonUp, RightButtonUp`. | Comma-separated vendor example inspected; historical page alone specifies no grammar. Reported `AuctionHouseSharedTemplates.xml:34` could not be located in this local profile cache, so is not verified here. Shared fix remains at pending RED stage; no new PASS. |
| Handled; existing behavior proof | `motionScriptsWhileDisabled`: current `FrameXml` field feeds real loader and CreateFrame template application; `tests/patch_3_3_0_motion_xml.rs` covers true/false, inheritance, pre-OnLoad state and Lua mutation. | Existing saved execution receipts below remain the proof; current test inspected, not rerun. No new native parity claim. |
| Blocked / applicability unresolved | Historical `Texture:GetFileWidth/GetFileHeight`: actual-file dimensions and inaccessible-file zero semantics; active-profile publication/retirement under investigation. Packed historical GUID formatting, server query throttling/transport, target-specific vehicle state and seven signature contracts remain unproved at this boundary. | Do not substitute layout/atlas size for file metadata or label texture methods generically missing native APIs. Saved gap oracles remain unchanged. |

## Development proof (historical saved receipts; not rerun in this reconciliation)
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
