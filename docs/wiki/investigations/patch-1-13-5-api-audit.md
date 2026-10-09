# Patch 1.13.5 frozen Classic SOURCE audit

Bounded literal audit at base `045e396b0c6f717c2d962b97cdf46b736a3328ce`. Frozen Warcraft Wiki page398847/revision3835002/timestamp2021-05-06T13:20:38Z; retrieved2026-10-09T08:51:36.400324+00:00. Original source versus current proof receipts remain distinct. No runtime/model/native credit.

## Coverage matrix

| Literal feature | Accounting | Proof boundary |
|---|---|---|
| Source rows |1690bytes;36physical/31nonblank;22metadata/9UNPROVEN|Lossless literal SOURCE only|
| API/event inventory |3globals/2events;5 unspecified signatures|No arguments/returns/payload/default/security declarations|
| Structure |5headings/2numeric headers/1caption|Counts3/2 match raw inventory; default generator omits toggle counts, ledger retains both|
| Prose |2rows/3substantive changes plus rationale|Threat reinstatement; increased dungeon/raid combat-log range; four additional authenticator-conditioned backpack slots remain UNPROVEN|
| References |7links/7templates/2named-ref occurrences|All unexpanded;14 UNPROVEN identity/prose/link contracts; no foreign/linked credit|

RawSHA256 `f8a5356cff1fc8c2188b68aa77ab60969c102ea64767a4f4721fd07532f85ac2`; responseSHA256 `eaa85552784310bd4cf62fab378f6863123fe1843c7826537495499f18d9f831`. Manifest-linked registry101 ends1.0.0. Caption1.13.4/build34600→1.13.5/build34713; TOC11305; navigation1.13.4→1.13.5→1.13.6. Reference title contains `WoW Classic`; citation date2020-06-12 is not the revision date or configured native identity.

## Backing-model assessment first

Retained `unit_misc.rs` reads two optional token strings for UnitThreatSituation but always returns nil; that existing signature is not imported into the historical page. `unit_threat_defaults.rs` explicitly lacks a real threat table and returns false/zeros for detailed/lead queries. No meaningful threat-model closure or native equivalence. Existing factory/name availability would establish neither.

Threat event identities appear in valid_events_c; Classic event registration is permissive. No grounded payload, threat mutation, producer lifecycle or delivery-order contract. Source says combat-log range increased in dungeons/raids, but supplies no before/after distance; complete owned source scan finds no range declaration to test. No fabricated numerical threshold/filter model.

`C_Container.GetContainerNumSlots` reads bag_info-backed capacity, whose baseline fixture is16 for bag0. No authenticator matches in the retained complete src rs/lua scan. That unrelated fixture is not the historical default/final capacity or proof of four conditional extra slots. Entitlement state, activation, persistence and native transition remain unmodeled/UNPROVEN. No runtime change or synthetic behavior test justified.

## Current configuration and successor limits

Retained Era/Anniversary configuration11507 differs from source11305/build34713. Same-Era canonical1.13.7–1.15.9 pins/responses/bodies retained as inputs, not applied registers. Concurrent1.13.6 recorded separately in queued-successor.json, no integrated register. Main must integrate1.13.6 before1.13.5; possible supersessions unknown until actual register is applied. `later_registers=[]`; no foreign Retail/TBC/Wrath/Mists/Forever credit.

## Development proof

SOURCE RED8 assertion failures at `c33717e54` retained; GREEN8/8 at `b1a35a773`. Literal implementation includes103 per-occurrence omission controls and fabricated signature/default/alias/history/native-credit rejection. Portable RED3 retained at the same source revision; GREEN3/3 at `b6efab100` runs copied SOURCE8/8 and validator in fresh processes with empty PATH, no copied Git/target/current tools or source-checkout lookup. Generator bytes match; default extractor failure reproduces exactly. Both serialized ledger omission and fabricated GREEN log reject by seal; exact original bytes restored, all70 original seals and post-restoration validator pass. No native/model/runtime/final acceptance.

Historical generator defaults flags[] produce all5 inventory entries byte-exact. Default extractor flags[] raises existing `ValueError: unhandled template` at the threat prose's ref-web template; failure retained, no extracted output or fallback. All raw prose and references remain in ledger. Shared tools unchanged; no runtime/vendor/Wowless changes, Cargo/build/cache changes, network or final gates.

## Historical evidence versus later receipts

Original70 seals/975379bytes and mapSHA256 `872ffcab1e3a5464378607c4e50ca810c2940319e38d6fff927a9c32e859a71b` remain unchanged. Archive71members/157330bytes copies all originals and the map. Seven separately sealed current receipt/archive/map files record later portable GREEN, not backfill of original proof. Derived receipts contain8 copied process runs and2 exact disk restorations. [Historical proof ledger](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/proof-ledger.json), [current proof ledger](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/current-proof-ledger.json), [portable receipts](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/portable-proof.json), [replay](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/REPLAY.md).

Python targeted tests emitted no warnings; expected default extractor error is retained, not suppressed. Cargo/native/model tests not run; no Rust warning-free claim. Formatting used Python AST serialization on only the three own Python files before implementation commits. No final checks/lint/readability/coverage/broad suites. Source accounting and controls closed; all14 substantive/reference contracts remain UNPROVEN. Parent goal, actual successor supersession and final/native gates remain main-owned.

## Sources

- [Spec](../../specs/patch-1-13-5-source-accounting.md).
- [Frozen pin](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/source-pin.json), [raw source](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/source.wikitext), [ledger](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/ledger.json): Warcraft Wiki, CC BY-SA4.0; exact request/retrieval retained.
- [Own model review](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/model-review.json), [static state scan](../../../data/patch-api/evidence/1.13.5-session-2026-10-09/state-scan.json): current code observations only.

## See Also

- [[patch-1-13-7-api-audit]], [[patch-1-14-0-api-audit]], [[patch-1-14-1-api-audit]] — canonical accounting templates, no borrowed runtime/model receipts.
- [[client-profiles]] — configured profile versus native historical identity.
