# Patch 12.0.1 publication sweep

## Contract

Probe all 225 consolidated inventory occurrences from retained page 659762, revision 6747895, in the unmodified cached Game environment. Publication/absence only: no signature, result, behavior, security or native parity credit. Non-inventory statements require separate behavioral evidence.

Use the shared classifier in `tests/common/publication_sweep.rs`, including exact cached deprecation-alias attribution. Apply later add/remove inventories from 12.0.5, 12.0.7 and 12.1.0 in order; changed rows alone do not reverse publication. The 12.0.0 sweep includes 12.0.1 as its earliest later register.

Require the exact observed non-OK ID set to equal `tests/data/patch_12_0_1_sweep_known_gaps.json`. `P1201_SWEEP_OUT` writes every observation before assertion; `P1201_SWEEP_REGISTER` accepts a full scratch register with the same row count. One flipped unsuperseded row must introduce exactly one new gap.

## Retirement epoch

`Cargo.toml` and `src/client_profile.rs` expose 12.0.0, 12.0.5, 12.0.7 and later epochs; no 12.0.1 feature exists. The source explicitly compares 12.0.0 (65655) to 12.0.1 (66838). `C_NamePlate.GetTargetClampingInsets` and `SetTargetClampingInsets` were observed raw-absent but fabricated by ordinary namespace lookup. Mark them retired at `retail-12-0-5`, the first supported epoch after removal. A 12.0.0 gate would incorrectly retire APIs on the preceding surface. Earlier-epoch preservation is source-gate inspected, not runtime-proven under this retail-only task. The same gate stops the explicit `C_CombatAudioAlert.GetSpeakerVolume`/`SetSpeakerVolume` publishers and marks their keys retired. Earlier-only volume tests remain epoch-gated; current format/speed tests no longer invoke retired volume APIs. Category voice/volume successors remain real model gaps; no replacement stubs were invented.

## Verification

- [x] Review every non-OK row and baseline exact IDs.
- [x] Fix cheap root-cause publication defects without fabricated successor behavior.
- [x] Prove one-row negative control.
- [ ] Run all five publication sweeps separately under debug retail, local build host.
- [ ] Startup Lua errors `[]`; formatting and local check.

## Reviewed outcomes — 2026-10-06

Development sweep at `3ddcc019c`: 225 rows, 174 OK, 51 exact reviewed gaps. Initial sweep: 170 OK / 55 gaps; retirement closes four, with no fabricated successor publication. Sixteen OK rows carry later-patch supersession. [Per-ID review](../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-gap-review.json) retains every non-OK observation.

| Gap class | Rows | Remaining boundary |
|---|---:|---|
| Namespace members | 27 | Raw-absent lookup autostubs; backing models required |
| Object methods | 13 | Twelve missing heal-prediction methods; one removed frame method still present |
| CVars | 5 | Removed names still queryable, including case-insensitive aliases |
| Plain globals | 3 | Totem count/duration and player spell target absent |
| Events | 2 | Arena event unknown; encounter event remains registerable |
| Unprobeable NamePlate | 1 | `CreateFrame('NamePlate')` unsupported; supported acquisition fixture required |

The 12.0.0 baseline changes from 23 to 27 IDs: minimapTrackedInfov2, useCompactPartyFrames, CHAT_MSG_ENCOUNTER_EVENT and FrameScriptObject:SetPreventSecretValues now inherit 12.0.1 removal expectations and remain published. Two speaker-volume removal gaps exposed by supersession were fixed rather than baselined. No old page-coverage ledger is changed.

Negative control flips unsuperseded `C_DamageMeter.GetSessionDurationSeconds` from added to removed: 51 → 52 gaps, exactly one new ID and exactly one changed observation; expected exit 101. [Control proof](../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-negative-control.json).

## Inputs and tests

- `data/patch-api/sources/12.0.1-api-changes.wikitext` and provenance: source capture.
- `data/patch-api/sources/12.0.1-wikitext-register.json`: inventory occurrences; all eight added/removed header counts match.
- `tests/patch_12_0_1_publication_sweep.rs`: shared cached publication sweep.
- [Shared sweep contract](patch-12-0-7-publication-sweep.md).
