# Patch 1.1.0 literal SOURCE audit

Frozen original historical Retail audit on branch `p110-page`, base `b02b9f544ada14ee5d229b74f4f819c6ab4d7f5f`, owned cwd `/home/osso/.worktrees/wow-ui-sim-p110-page`. Frozen page271516/revision5913060/timestamp2023-12-27T16:05:57Z; revision timestamp is not release date. Canonical own-base1.6/1.7/1.8 templates provide methodology only, not transplanted proof.

## Identity and coverage

701-byte body SHA256 `d584c6fd1208de8669c85333f63c3da63bebdd21bd0b3a3e3387a36f3ddf8625`; response `9ad4deff959b721b3561d017b1e3613be2b6b35e7d5d27d195639b96af1b7dde`; frozen manifest `b07fc204377842c8d5303f9cd4597acba02290bbe6874efe4ab160143c27d6ba`; manifest-linked registry `e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c` (101 pages, ends1.0.0). Verified before deriving own ledger. These immutable copies preserve upstream bytes, including source newline behavior.

| Feature | Count | Proof / precise limit |
|---|---:|---|
| Physical/nonblank rows | 17/15 | Exact literal occurrence accounting |
| Global API additions/signature limits | 11/11 | Names listed as added; no argument/return/arity/type/default declaration |
| Links/prose/headers/templates | 12/1/2/1 | API links and Global_functions oldid4997032 unexpanded; two headings without numeric counts; navigation preserved |
| UNPROVEN contracts/defaults/count claims | 13/0/0 | 11 API contracts plus attribution and navigation; defaults/errors/events/transitions/security/native absent |
| Meaningful historical model/runtime/native proof | 0/0/0 | No historical contract or executed model subset |

Names: CheckTalentMasterDist, ConfirmSummon, ConfirmTalentWipe, GetAuctionItemLink, GetCVarDefault, GetResSicknessDuration, GetSummonConfirmAreaName, GetSummonConfirmSummoner, GetSummonConfirmTimeLeft, TutorialsEnabled, UnitRangedAttack. Explicit source publication does not prove historical behavior.

## Current backing-model assessment

[Candidate review](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/model-candidate-review.json) retains exact own-base source/test hashes. `GetCVarDefault` reads `CVarStorage.defaults`/`registered_defaults`, not `overrides` (`src/cvars.rs:38,102,119,160`; global path `set_cvar_verb.rs:208`). Existing `register_cvar_numeric_default_preserves_existing_override_and_first_default` concretely asserts current1.25/default0, then re-registration6 preserving both. Genuine current state/default-separation candidate, not executed here; no historical/native credit. Main owns runtime proof.

`UnitRangedAttack` (`unit_stats.rs:392`) derives max(level*5,0) and fixed modifier0. Not a modeled independent ranged-skill/modifier lifecycle; the existing Mists numeric/positive test provides no transitions. Reject meaningful ranged-attack credit. Global summon stubs are not equivalent to `C_SummonInfo`. No API aliases/defaults/shims added.

## History and proof boundaries

Navigation literally says prev1.0.0/next1.2.0; registry has no1.2.0, so no fabricated successor pin. Newer1.3.0 active `p130-page`,1.4.0/1.5.0 pending remain separately queued/unapplied. Original historical Retail2.0.1+ references stay separate; Era1.13.x+, TBC2.5.x, Wrath3.4.x, Cataclysm4.4.x, Mists5.5.x and Forever1.60.1 cannot supply missing historical contracts.

Own SOURCE RED7/portable RED3 at base retained. Historical own-base generator default has zero entries, unlike literal11; default extractor106 bytes. Preserve both default outputs rather than changing shared defaults. Two malformed-input error identities/messages retained. First SOURCE GREEN attempt atf5b1991fe retained one test-order failure: registry naturally queues1.5/1.4/1.3, while the fixture assumed ascending order. Test compares the exact version set without assigning application order; frozen ledger/queue unchanged. Corrected SOURCE GREEN and portable epochs pending; receipts record actual execution revisions and file hash scopes, not later documenting HEAD. No proof credit from setup/help/formatter commands.

## Sources

- [Frozen body](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/source.wikitext), [response](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/source-response.json), [pin](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/source-pin.json).
- [Literal ledger](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/ledger.json), [proof ledger](../../../data/patch-api/evidence/1.1.0-session-2026-10-09/proof-ledger.json), [spec](../../specs/patch-1-1-0-source-accounting.md).
- [Governing handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md); explicit bounded-child restrictions override broader operations.

## See Also

- [[patch-1-6-0-api-audit]], [[patch-1-7-0-api-audit]], [[patch-1-8-0-api-audit]] — original frozen SOURCE methodology.
