# Patch 4.0.1 API audit

Pinned historical retail page 129645, revision 1271877, timestamp 2012-09-06T23:25:23Z; supplied HTTP 200 receipt on October 9, 2026. No network requested or used.

## Source coverage

The page contains 416 labeled inventory occurrences plus three API-linked breaking references: 258 added/83 removed globals, 60 added/15 removed events, one changed global and two changed events. Eight retained non-inventory rows cover navigation, heading, four breaking statements and two automatically generated build contexts. No explicit structure or enumeration sections. Only explicit signature assertion: GetItemCooldown numeric item IDs; no inferred signatures for inventory-only names.

Existing colon/indented-list and combat-restriction flags do not parse NEW/REMOVED inventories under commented spaced headings with correctly typed event references. New `--cataclysm-labeled-inventory` handling is opt-in in both tools; no existing source flags change. Publication sweep and exact gap accounting are under development, not acceptance claims.

## Successor boundary

Four explicit pending 4.1.0/4.2.0/4.3.0/4.3.4 placeholders precede actual 5.0.1 and later retail registers. Main replaces placeholders on integration. Classic 5.5.x is excluded. No native historical behavioral credit follows from modern retail publication or deprecation wrappers.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.0.1-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-4-0-1-publication-sweep.md).

## See Also

- [[patch-5-0-1-api-audit]] — actual next integrated retail register.
- [[patch-5-0-4-api-audit]] — substantive later retail publication.
