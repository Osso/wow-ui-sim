# Historical retail Patch 3.1.0 API source audit

Frozen page 233195/revision 2259594/timestamp 2013-05-22T17:34:42Z describes 2009 retail builds 9614/9626. The later wiki revision timestamp is not the game-release date. Supplied October 9, 2026; literal manifest response/body hashes checked before copying. Wrath Classic 3.4.x is a different client history.

## Source format and capability boundary

`--legacy-function-labels` is opt-in. It retains plain NEW/UPDATED/REMOVED rows and explicit addenda with original line numbers/annotations. Default parser behavior is unchanged; default full-page extraction remains unmodified. No invented owner for the prose-only secure `SetUpAnimation` method, no `GetTargetFacing` publication from a question, and no argument expansion from linked API pages. The page explicitly says its PTR list is incomplete and hover implementation buggy.

| Scope | Accounted | Still UNPROVEN | Proof |
|---|---|---|---|
| Publication occurrences | 110: 87 additions, 12 changes, 11 removals | Current callable/absent mismatches; historical semantics regardless of match | Own discovery in progress |
| Nonblank original lines/full extract | 189 / 189, including source context and PTR caveats | Talent-group/preview state, aura legacy caster/controller token and owned-first order; secure restoration/hover lifecycle; macro spec selection; item-location table mutation | Literal source accounting |
| Signature fragments | 33, including heading/question/empty-call/code fragments | Historical coercion, arity, default selection, outputs and security; identity-only inventory has no invented signature | Literal parenthetical source retention |
| Cheap modeled closures | Original archive: zero. Real retail player-orientation activation pending proof | Native PTR correction, unavailable-state behavior, movement production, other-unit facing and security | Test RED: retail backing field was feature-gated; concrete 0/π÷2/π state fixture added |
| Queued successors | Retail 3.2.0, 3.3.0, 3.3.3, 3.3.5, 4.0.1 | Ordered main integration and final runtime/CI gates | Read-only overlaps, no placeholder credit |

100 labeled rows plus ten addenda include both `GetPlayerFacing` statement and code occurrence. Raw-line, full-extract, signature and publication IDs remain separate so current symbol presence cannot erase prose limits. Talent/glyph groups are historically 1-based, but modern specializations do not establish preview allocation/learning behavior. Hover prose retains expiration/reset, explicit-hide and movement cancellation, child-rectangle and noncombat requirements rather than implementing a guessed driver.

## Targeted development evidence

Parser RED at `1fcfa05b4` fails on the missing opt-in flag. First GREEN attempt at `768d579f4` exposes a fixture omission: the source contains both player-facing statement and example. Corrected fixture at `1a19167bc` passes 2/2; parser unchanged. The source itself is not changed to fit the fixture.

Initial headless prefork invocation at `c7f6e49ae` exits 101 on pre-existing GUI-gated registry references; it provides no publication observations. Failure log/exit retained, warnings unsuppressed. The bounded default-feature case is running independently; no all-publication, check/lint/profile/full suite or final acceptance claimed.

## Bounded radians model

Existing `player.facing: Option<f64>` and real getter now reused for retail; common player state retains unknown initial orientation as simulator policy. Only getter publication changes on retail; WowForever admin setter and Classic API exposure remain unchanged. No movement controller, UI input, normalization, fallback, other-unit facing or native-default behavior invented. Concrete state-transition test checks 0, π÷2 and π without degree conversion and exactly one return; proof pending. The archived original source audit remains independent and earns no retroactive model credit.

## Historical replay boundary

Own historical source/register/extract/ledger/gaps and command receipts are sealed separately from current/future closures. Fresh-process replay proves source accounting and retained targeted results, not full runtime/native reconstruction. Main owns integration/final gates. Only real retail getter/backing-state activation; no cache/vendor writes, shims, retirements, delegation, push, merge or deployment.

## Sources

- [Spec](../../specs/patch-3-1-0-publication-sweep.md).
- [Frozen source](../../../data/patch-api/sources/3.1.0-api-changes.wikitext).
- [Literal response and manifest receipt](../../../data/patch-api/evidence/3.1.0-session-2026-10-09/source-pin.json).
- [Read-only consumer scans](../../../data/patch-api/evidence/3.1.0-session-2026-10-09/consumer-scans.json).
- [Queued successor overlaps](../../../data/patch-api/evidence/3.1.0-session-2026-10-09/queued-successor-overlaps.json).

## See Also

- [[patch-4-0-1-api-audit]] — separately integrated source format and historical evidence boundary.
- [[patch-4-1-0-api-audit]] — oldest actual successor applied in this slice.
