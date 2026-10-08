# Patch 7.3.0 API audit

Page 553643 refetched on 2026-10-08 and pinned to revision **5335723** (2017-10-26). Source is a ten-line summary: three named table additions, addon/debug-tool announcements and a PlaySound input change. Audit in progress; no completion claim.

## Source and extraction

The default extractor retains every statement. Existing inventory options require template references, nested API sections or caption tables; none fits these bare table summaries. New `--legacy-summary-tables` opts into a separate parser for `New global table:` and `New API tables:` lines only. Behavioral fixture retains SOUNDKIT and both namespace names, excludes addon/debug prose and ignores similarly shaped lines outside New. All previous default behavior remains unchanged.

## Bounded proof and remaining work

Prefork discovery probes the three table additions, with the pending 7.3.2 placeholder first in later registers. Sound transition test exercises the existing `last_sound_kit_requested` backing state, concrete ID 861, rejection of the old string name and a subsequent ID 839. It does not mock PlaySound or change Blizzard Lua. Actual Table Inspector focus/navigation/close is also probed after loading Blizzard_DebugTools. Initial Cargo discovery stopped at an unsupported Rust `u32` result conversion; fixed to the supported signed result with checked conversion. This compile failure is not a runtime gap. Discovery, gap accounting, validator and targeted Cargo verification remain pending.

## Retirement policy

No removal statements occur on this page; no retirement is authorized. Complete whole-word `/usr/bin/grep -R -n -w -F` scans cover the named tables and PlaySound in cached retail AddOns (excluding Documentation) and src/tests. Bare-name caller scans include `pcall(Name, ...)` and `and Name then` without syntactic filtering. Later-register snapshots read master and p732-page Git blobs only; no other worktree is touched.

## Sources

- [Pinned source](../../../data/patch-api/sources/7.3.0-api-changes.wikitext).
- [Fetch and evidence](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/).
- [Spec](../../specs/patch-7-3-0-publication-sweep.md).

## See Also

- [[patch-8-0-1-api-audit]] — prior-page template and source reproduction recipes.
- [[patch-audit-validator-portability]] — fixed historical scope, no checkout path gates.
