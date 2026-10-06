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
- [x] Run all five publication sweeps separately under debug retail, local build host.
- [x] Startup Lua errors `[]`; formatting and local check.

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

## Final local verification — 2026-10-06

Verified code/data revision `a69dcfd66b08b853452fbe273c52735c545ba6dd`; subsequent evidence/docs-only commits do not invalidate runtime proof. No agents/models or independent review. All tests used default debug retail and the existing target directory.

| Isolated filter | Rows | OK | Exact known gaps | Result |
|---|---:|---:|---:|---|
| patch_12_0_0_publication_sweep | 1010 | 983 | 27 | PASS |
| patch_12_0_1_publication_sweep | 225 | 174 | 51 | PASS |
| patch_12_0_5_publication_sweep | 363 | 352 | 11 | PASS |
| patch_12_0_7_publication_sweep | 174 | 171 | 3 | PASS |
| patch_12_1_0_publication_sweep | 778 | 773 | 5 | PASS |

Each ran as its own command: `python3 /home/osso/.worktrees/wow-ui-sim-p1201-page/scripts/build-host.py --build-host local --test --test integration FILTER -- --nocapture --test-threads=1`. Output environment variables write `p1200/p1201/p1205/p1207/p1210-sweep-result.json` under [session evidence](../../data/patch-api/evidence/12.0.1-session-2026-10-06/).

`patch_12_0_1_retirement`: 1 PASS. `audio_`: 11 PASS across the direct and cached-prefork phases. Four Python extract fixtures PASS (old plaintext, new templates/API identities, comparison operators, nested contracts). Artifact validation PASS: 477 unique IDs, exact seed/occurrence union, all hashes/header counts, 201 pending scout assignments, four unchanged old ledgers. All four pre-existing registers regenerate byte-identically with the unmodified register generator.

`cargo fmt --manifest-path /home/osso/.worktrees/wow-ui-sim-p1201-page/Cargo.toml -- --check` PASS. Local helper `--check` PASS. Startup was built separately without a timeout, then run with `timeout 90 python3 /home/osso/.worktrees/wow-ui-sim-p1201-page/scripts/build-host.py --build-host local --no-build --run -- --no-addons --no-saved-vars lua-errors`: exit 0, Lua-error JSON `[]`, stderr CLEAN / zero errors. The helper's stdout artifact announcement was parsed separately from the JSON; saved output was inspected, not rerun.

Six pre-existing `iced-wgpu-patched/Cargo.toml` deprecated manifest-key warnings match the initial RED build exactly; no warning suppression or vendor modification. Manual changed-Rust readability/scope audit found no added suppression, deep control flow or forbidden-file changes. [Proof ledger](../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-proof.json) retains commands, revisions, outputs and proof limits; [477-row validation](../../data/patch-api/evidence/12.0.1-session-2026-10-06/p1201-page-validation.json).

## Inputs and tests

- `data/patch-api/sources/12.0.1-api-changes.wikitext` and provenance: source capture.
- `data/patch-api/sources/12.0.1-wikitext-register.json`: inventory occurrences; all eight added/removed header counts match.
- `tests/patch_12_0_1_publication_sweep.rs`: shared cached publication sweep.
- [Shared sweep contract](patch-12-0-7-publication-sweep.md).

## Removal follow-up — 2026-10-05

Five removed CVars are filtered from defaults, overrides, registration and console enumeration from `retail-12-0-5`. Encounter chat registration is rejected; arena cooldown registration is accepted. `SetPreventSecretValues` is no longer registered in later retail epochs. Cached retail consumers contain none of these seven removed symbols. Strict 12.0.0 publishers remain gated separately. Security-query tests now inject host frame state instead of calling the retired setter. Eight publication IDs close; behavioral retirement regression covers case-insensitive resurrection attempts.

## Settings producers

Category voice/volume settings use independent category-keyed maps under `c_api`, shared across specs. Encounter warning visibility and hidden custom-sound settings roundtrip through `WarningSettings`. These do not simulate audio playback, routing, persistence or native secret-argument enforcement. INFERRED: unconfigured values are zero/false; finite numeric writes succeed. Eight publication rows close with concrete category isolation and boolean transition tests.

## Prediction object producers

Move the existing prediction value/configuration object out of temporary proxy defaults into `globals/real`. Twelve 12.0.1 methods query those values, mode-dependent clamp maxima, fractions and supplied curves. `UnitGetDetailedHealPrediction` still supplies health/vitals to the same object; native absorb/heal and secrecy inputs remain unmodeled by that producer. Explicit `SetPredictedValues` provides concrete absorb/heal inputs. INFERRED: zero-max fractions return zero; overflow multiplies the maximum-health boundary. The documented WithAbsorbs mode adds total damage absorbs. Tests cover nonzero values, three clamp modes, reset and curve evaluation.

## Outfit successors

`SetOutfitToOutfit` stages a copy of source saved slots into the viewed outfit pending overlay, emits slot refresh and rejects unknown sources/no viewed target. INFERRED merge policy follows existing set imports; source contents are unchanged. Three event/discount queries read explicit host policy fields, defaulting false rather than inferring event participation from saved outfits. Discount query publication does not claim a native discount-pricing or redemption model. Tests cover nonempty source copying, source preservation and independent event/discount inputs.
