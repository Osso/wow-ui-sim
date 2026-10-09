# Patch 4.0.1 API audit

Pinned historical retail page 129645, revision 1271877, timestamp 2012-09-06T23:25:23Z; supplied HTTP 200 receipt on October 9, 2026. No network requested or used.

## Source coverage

The page contains 416 labeled inventory occurrences plus three API-linked breaking references: 258 added/83 removed globals, 60 added/15 removed events, one changed global and two changed events. Eight retained non-inventory rows cover navigation, heading, four breaking statements and two automatically generated build contexts. No explicit structure or enumeration sections. Only explicit signature assertion: GetItemCooldown numeric item IDs; no inferred signatures for inventory-only names.

Existing colon/indented-list and combat-restriction flags do not parse NEW/REMOVED inventories under commented spaced headings with correctly typed event references. New `--cataclysm-labeled-inventory` handling is opt-in in both tools; no existing source flags change. Targeted parser/extractor fixtures pass 75/75 at 6c041621f. Default-feature cached retail RED sweep recorded all 419 observations and 119 mismatches. Headless build failed on 17 pre-existing generated registry references to GUI-gated modules; no unrelated registry fix. Exact command/revision/log receipts are retained.

## Bounded runtime change and remaining limits

Complete 197-symbol expected-absence scans cover 2,551 cached retail Lua files plus complete src/tests Rust/Lua/XML, including Lua strings. Fully qualified and bare spellings coincide for these global/event names. `rg` is unavailable; recorded Python regex uses ASCII whole-word boundaries and retains all matching lines, file hashes and paths in `retirement-scans.json` (1.1 MB).

Only CollapseSkillHeader/ExpandSkillHeader are retired: explicit source removals, no cached retail/test callers, only two no-op registrations. Factory RED proves both were callable. A separate 4.0.1 list gates modern-retail registration exclusion; Classic unchanged. GREEN proof is pending this implementation commit.

The remaining 117 publication gaps have individual observations and domain/consumer reasons in the page ledger. Current consumers preserve GetQuestLogRewardHonor and later-retired CVar bitfield aliases; existing state APIs with tests are not swept into the unused-no-op change. Inventory identities alone provide no primary call signatures or state contracts for adding missing gameplay models. No new constants, aliases, shims or fallback behavior.

Four historical breaking statements remain separate native limits: implicit script globals; legacy per-power event non-emission and replacement payload/order; numeric-only GetItemCooldown; protected buff/weapon-enchant cancellation. C_Item.GetItemCooldown currently ignores its itemInfo and returns constants: no item cooldown backing model exists here, so narrowing its current retail input based on a 2010 statement would be unsound. Dedicated signature row preserves this gap. No native client, structure or enum parity claim.

## Successor boundary

Four explicit pending 4.1.0/4.2.0/4.3.0/4.3.4 placeholders precede actual 5.0.1 and later retail registers. Main replaces placeholders on integration. Classic 5.5.x is excluded. No native historical behavioral credit follows from modern retail publication or deprecation wrappers.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.0.1-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-4-0-1-publication-sweep.md).

## See Also

- [[patch-5-0-1-api-audit]] — actual next integrated retail register.
- [[patch-5-0-4-api-audit]] — substantive later retail publication.
