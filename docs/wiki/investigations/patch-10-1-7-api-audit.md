# Patch 10.1.7 API page audit

Page 442982, revision 6473483 (September 15, 2025, 16:55:36 UTC), retrieved October 7, 2026. Branch p1017-page starts from master b9c4da976. Default retail carries 12.1.0, not reconstructed 10.1.7.

## Source and accounting

48 inventory occurrences: 45 added, three removed, zero changed. The page explicitly warns its API list is out of date. Header counts remain verbatim: global additions declare 19 but list 28; events declare eight additions but list three. Other six counts match. Do not synthesize nine missing/five extra identities from header arithmetic.

103 unique source IDs: 48 inventory + 55 extract. Ledger initially distinguishes 29 partial-development-green, five bounded-coverage, 59 audit-pending and ten metadata-only. Publication/absence/event registration/default presence is not signatures, populated DTOs, security or historical/native parity. All 55 extract statements are assigned exactly once, including every Lua example line and all enum occurrences.

Eighteen later registers (10.2.5 through 12.1.0) supersede chronologically. C_Ping.GetContextualPingTypeForUnit and ScriptRegion:SetProtected are expected absent due to later removals. The one-line insertion point for the concurrent 10.2.0 register is explicit. Read-only sibling comparison found no matching gap symbol; no predicted gap supersessions at the retained 10.2.0 register fingerprint.

## Gaps and bounded fix

Discovery: 34 OK / 14 exact publication gaps. All fourteen retained with [individual reasons](../../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-gap-review.json). Existing four-valued SetRestrictPings state is not a historical raid boolean contract. Membership does not imply unread stream markers; UI manifests do not imply art-file manifests. Ping callbacks and no-op CreateFrame do not implement pending interaction, hit testing, forbidden sending or 3D world targeting. No new placeholder publication.

Removed tradeskill events and useCompactPartyFrames are already absent; [qualified and bare cached searches](../../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-retirement-consumers.json) find no consumers. No retirement/wrapper/vendor/classic changes required.

Full cached PingReceiverAttributeTemplate construction exposed omitted `<Attributes>` in runtime CreateFrame inheritance. Direct XML loading already emits SetAttribute calls; runtime template chain and direct runtime child paths omitted them. Reuse the existing typed emitter through ordinary SetAttribute dispatch. Concrete boolean/number/string/nil overrides and parent/child OnLoad ordering reproduce before fixing. This closes a bounded template primitive, not any of the fourteen publication gaps or historical ping/security claims. Current cached mixin uses GetIsPingable/GetTargetInfo rather than the complete historical example API.

## Verification

Twenty-one parser/extractor fixtures pass. All eighteen existing registers regenerate byte-identically. Attribute regression RED and cached diagnostic failure retained. Final targeted gates pending; no acceptance claim yet. All commands use explicit cwd p1017-page and its own target; worktree creation used the prescribed canonical Git metadata operation from empty destination. No canonical working-file or sibling edits, no vendor/cache/Wowless edits, agents/models, push or merge.

## Sources

- [Provenance](../../../data/patch-api/sources/10.1.7-api-changes.provenance.json).
- [Register](../../../data/patch-api/sources/10.1.7-wikitext-register.json).
- [Coverage ledger](../../../data/patch-api/sources/10.1.7-page-coverage.json).
- [Extract scout](../../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-extract-scout.json).
- [Contract](../../specs/patch-10-1-7-publication-sweep.md).
- [Proof ledger](../../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-proof.json).
- [Possible 10.2.0 supersessions](../../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-possible-1020-supersessions.json).

## See Also

- [[patch-10-2-5-api-audit]] — verbatim example and publication accounting conventions.
- [[patch-10-2-6-api-audit]] — cached consumers and chronological supersession.
- [[client-profiles]] — classic boundaries.
