# Patch 10.2.5 API page audit

Page 564286, revision 5993852 (March 24, 2024, 16:20:32 UTC), retrieved October 7, 2026 UTC. Branch p1025-page starts from master 9d1a36178. Default retail carries 12.1.0, not reconstructed 10.2.5.

## Source and accounting

59 inventory occurrences: 43 added, 13 removed, three changed. All eight header counts match. Sixteen later registers, 10.2.7 through 12.1.0, supersede chronologically. Independent 10.2.6 is deliberately excluded; its later-register include is a one-line integration change. Read-only retained 10.2.6 register has no matching symbol among the fourteen 10.2.5 gaps, so none is predicted to be superseded by it.

415 unique source IDs: 59 inventory + 356 extract. Ledger: 35 partial-development-green, ten bounded-coverage, 96 audit-pending and 274 metadata-only. Extract scout assigns all 356 IDs exactly once: 82 substantive contracts pending, 274 editorial/documentary occurrences. The 253-row Blizzard Docs section records documentation backfill, not newly added runtime functions; every identity remains retained with no runtime credit. No inventory publication probe implies populated DTOs, enum values, signatures, security or native behavior.

Original extractor erased XML and rewrote Lua long-bracket literals as wiki links. Opt-in `--preserve-examples` retains both fenced examples verbatim. Default mode is unchanged so existing capture reproduction is unaffected. Added fixture fails before this option and passes afterward. Page register parser unchanged.

## Gap review

Discovery: 45 OK / 14 failures. Four VertexColor failures used an invalid frame factory. VertexColor is a texture animation; corrected factory reaches actual missing endpoints rather than reporting an unknown frame kind. All four remain real publication gaps. No runtime closure, placeholder or vendor edit.

[Per-ID review](../../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-gap-review.json) distinguishes absent explicit namespace producers from lookup autostubs. Item effective stats/deltas/set bonuses need item-instance metadata and specialization relations. Ping cooldown dispatch and measured texture-kit data are missing. Voice transcription needs session/provider activity. UnitSetRoleEnum needs assigned group-role state, not conversion over the existing unmodeled role getter. VertexColor needs endpoints and interpolation/restoration across Lua/XML, not merely its enum variant.

UnitAura, UnitBuff and UnitDebuff remain simulator-native legacy functions, not cached deprecation wrappers. Current qualified cached searches find no actual retail global consumer; retirement requires coordinated simulator-owned Admin/test migration and classic-preserving gating, not deleting a shared registration. `src/lua_api/globals/admin_buffs.rs:116` uses UnitDebuff; `src/mists/compat_bootstrap.lua:853-855` wraps the three legacy functions. This is a deferred migration, not a claim Blizzard still consumes them. Existing cached wrappers untouched. FillLocalizedClassList cached hits are flavor files and its deprecation wrapper; UnitAuraSlots has only a comment hit. [Exact searches](../../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-retirement-consumers.json).

[Extract scout](../../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-extract-scout.json) separates endpoint animation, color-picker migration, unit-token ping routing, insecure-call permission, secure-CVar hotfix, enum literals, populated DTO fields and legacy deprecation semantics. Current cached color-picker/hex consumer is tested separately; identical historical optional parameters/native behavior remain pending.

## Verification

Requested targeted gates pending. [Proof ledger](../../../data/patch-api/evidence/10.2.5-session-2026-10-07/p1025-proof.json) retains command, exact revision, scope, results and invalidated RED runs. [Contract](../../specs/patch-10-2-5-publication-sweep.md) lists required tests. No native or independent acceptance claim.

Every command uses explicit cwd p1025-page and own target. Worktree creation used prescribed canonical Git metadata operation with cwd in empty destination. No canonical working files, sibling worktrees, cache/vendor Lua or Wowless modified; no full suite, agents/models, push or merge.

## Sources

- [Provenance](../../../data/patch-api/sources/10.2.5-api-changes.provenance.json).
- [Coverage ledger](../../../data/patch-api/sources/10.2.5-page-coverage.json).
- [Publication contract](../../specs/patch-10-2-5-publication-sweep.md).
- [Register](../../../data/patch-api/sources/10.2.5-wikitext-register.json).
- [Verbatim-code extract](../../../data/patch-api/sources/10.2.5-api-changes.txt).

## See Also

- [[patch-10-2-7-api-audit]] — accounting and current-retail chronological supersession.
- [[patch-11-0-0-api-audit]] — proof and exact-fixture conventions.
- [[client-profiles]] — classic boundaries.
