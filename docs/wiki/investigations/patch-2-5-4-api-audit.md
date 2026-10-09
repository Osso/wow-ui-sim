# TBC Classic Patch 2.5.4 SOURCE/contract audit

Frozen page 515131/revision 4967736, timestamp 2023-06-20T22:06:32Z. Own isolated `p254-source` from canonical master `acf7fbfe9b07dc342b8078cb1c54b702151a23d6`. Bounded source proof only; no current-runtime or native acceptance.

## Coverage matrix

| Literal surface | Accounting | Proof level / missing contract |
|---|---|---|
| Complete original source | 23,192 bytes; 466 physical / 461 nonblank rows | Exact response/content/hash identity; 48 metadata, 413 UNPROVEN rows |
| Global API | 187 added, zero removed | Names only; 187 unspecified signatures; publication/arguments/returns/errors/state/security UNPROVEN |
| Widgets | `Frame:SetAttributeNoHandler`, one added | No literal signature or handler/state/security semantics; UNPROVEN, not a model specification |
| Events | 200 added, zero removed | Emitter, ordering, payloads, lifecycle and security UNPROVEN |
| CVars | 15 added, nine removed | Literal defaults/scopes/descriptions retained per row; actual storage/mutation/effects/removal absence UNPROVEN |
| Headers | Eight Added/Removed counts, all reconciled | Source-count evidence only, not API/native parity |
| Other literal contracts | Two linked diff commits; 414 contracts total | Linked contents not retrieved; both UNPROVEN; no fabricated diff expansion |
| Summary/signature/transclusion | Zero local summary contracts, zero explicit signatures, zero content transclusions; 188 unspecified callable signatures | Navigation template retained; no linked-page inventory or transclusion reconstruction |
| Models/measurement | Zero changed models; zero runtime/native observations | Unavailable matching configured profile is not an unsupported-API diagnosis |

Every API occurrence has its original line and a precise UNPROVEN contract. Every nonblank row retains full literal text, including hidden CVar metadata, build caption, table syntax and navigation. All rows have empty capability credit. The ordinary non-inventory extractor retains four context rows and drops tables: the full literal ledger therefore accounts for all 461 original rows separately; four plaintext rows are not the inventory count.

## Source provenance

[Original manifest](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/frozen-manifest.json) pins response SHA-256 `e72b07ad6e287743b2f2d17b1d2ad200442cd893cbe6bfa83ba71203a272ffbc` and wikitext SHA-256 `c4e57ba61e46af02c1f0fb48d6b6048d540898f2eea08694b95b5321f2b47766`. Response content matches committed wikitext byte-for-byte. [Frozen registry](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/frozen-registry.json) has 101 pages through 1.0.0; manifest's registry hash validates. This registry is the full remaining-page inventory below 8.3.0 from the handoff, not an assertion of full historical API coverage.

Literal source: `TOC: 20504`; build caption `2.5.3 (41812) → 2.5.4 (44833) Jul 25 2022`. Caption date and response revision timestamp are different facts. The two diff links pin wow-ui-source `bc779043df19164bbe2478ab47305231e60c8899` and BlizzardInterfaceResources `c1c8511423580a0e606258726d889a7c51a55ae9`; referenced payloads remain unexamined.

## Configured interfaces, separately observed

[Historical configuration observation](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/profile-observation.json) derives values from copied source Rust arms/Cargo feature graph and manifest bytes at the base revision: Retail 120100, PTR 120105, Wrath 38001, Mists 50504, Era/Anniversary 11507, Forever 16001. Manifests list paths, not loaded TOC content. No configured 205xx profile exists in this snapshot; no cache inspection, sync, native probe, startup, build or profile test performed. Anniversary naming cannot justify TBC205xx measurements. No shared classifier/profile/runtime edits made to force correspondence. Contracts remain unproven, not unsupported.

## Client history and main ownership

Separate `tbc-classic` SOURCE label, not a new runtime client classifier. Only registry-pinned 2.5.5 and 2.5.6 are pending same-TBC successors, with zero supersession credit until main integrates them. No dependency on sibling files. Literal navigation `next=3.4.0` points into Wrath Classic history and is not TBC supersession. Retail3.x/Cata4.x/WrathClassic3.4/Era1.x cannot retire these contracts. Main owns integration and native/publication/state gates; source-only replay cannot discharge them.

## Targeted proof

Own source-accounting RED failed `0 != 461` on the seed ledger at `c7c560d4f`. Own GREEN: eight tests, all pass; every row/contract omission rejected, precise field/default/scope/header mutations rejected, wrong-client/foreign-successor and fabricated native/profile/model credit rejected. [RED scope](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/red-scope.json), [GREEN receipt](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/green.log). Code/data are committed before historical replay verification. Sealed copied replay and serialized controls are recorded below once executed. No broad/check/lint/profile/startup/full-suite/final/native gates run.

## Sources
- [Spec](../../specs/patch-2-5-4-source-accounting.md) — scope and testable source contract.
- [Literal ledger](../../../data/patch-api/sources/2.5.4-page-coverage.json) — complete rows/API/header/contract accounting.
- [Frozen response](../../../data/patch-api/evidence/2.5.4-session-2026-10-09/source-response.json) — original revision payload.
- [Handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md) — historical registry/integration context; this bounded request excludes its broad gates/operations.

## See Also
- [[patch-3-4-3-api-audit]] — source-only accounting template, not same-client supersession.
- [[client-profiles]] — simulator configuration boundary.
