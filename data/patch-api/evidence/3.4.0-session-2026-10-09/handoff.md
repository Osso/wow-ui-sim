# Patch 3.4.0 bounded source/model-contract handoff

Owned worktree `/home/osso/.worktrees/wow-ui-sim-p340-source`, branch `p340-source`, base `34c303c108d55e6c9d4919878abbb9a0efc08c00`. No push/merge. Parent endpoint 1.0.0 unchanged.

## Counts and proof boundary

- Frozen page 16896 / revision 165668 / timestamp 2022-11-19T13:43:58Z, source TOC30400; manifest/response/literal hashes checked before copying. Raw 17530 bytes / response 18301 bytes.
- 386 nonblank raw rows = 331 inventory + four summary contracts + 51 metadata. 331 inventory = 255 globals + ten widget methods + 46 events + 19 CVars + command SetRaidDifficulty; 300 added / 31 removed. Eighteen literal defaults; bare TTSUseCharacterSettings removal retained.
- 266 signature-limit rows = 265 identity-only callable occurrences + UnitAura partial return contract. Zero complete signatures, runtime/native observations or closed runtime gaps. All 335 substantive rows UNPROVEN.
- `_Wrath`/`-WOTLKC` page contract retained. Static loader candidates omit -WOTLKC; configured Wrath38001 is 3.3.5-era architecture, not Classic30400 native parity. Candidate gap only. UnitAura has no consolidation input; no inferred boolean or numeric tuple position.
- Only Classic3.4.1/3.4.2/3.4.3 queued; no successor register applied. References 61fd58823/990dae19c/48ab8e1c3 are source-only historical proof. No copying/credit of Agent205's uncommitted shared classifier/factory. Parent owns actual current publication/factory measurement and integration after3.4.1.

## Exact owned proof paths

All paths below are relative to `data/patch-api/evidence/3.4.0-session-2026-10-09/` unless noted.

- `source-proof.json`: exact command/cwd/revision/content-scope ledger. `red.log`: expected None !=331 against temporary missing accounting. `green.log`: 8/8 at2497ec7a8, including 987 serialized row-omission controls and metadata/signature/profile/credit tampering.
- `validator.log` / `extractor.log`: source replay and recorded flags pass; limited extract nine rows (five metadata/four pending). `validate.py` derives own counts dynamically; no Git/global registers/cache/runtime dependencies.
- `portable-controls.log` / `portable-proof.json`: at30071d654b69f7377cfbe4a9690ff1a0c0fce167 source tamper exit1, GREEN-log tamper exit1, originals byte-restored/hash-checked; relocated Git-free replay exit0 with17 historical seals. Archive SHA256 af8cb1165ea0f7b79e42d5f48c5a60583b7ce770f74248ace9c8d8f23cc5f606. Evidence/source bytes at controls509611 (<5MB).
- `source-pin.json`, `source-response.json`, `model-context.json`, `profile-observation.json`, `seals.json`: immutable source/model/profile scope. Repository ledger path: `data/patch-api/sources/3.4.0-page-coverage.json`.
- `docs/specs/patch-3-4-0-source-accounting.md`; `docs/wiki/investigations/patch-3-4-0-api-audit.md`, wiki index/log updated coherently.

## Commits

1. `69c390450` — frozen source/model-contract accounting and tracked docs.
2. `2497ec7a8` — own dynamic validator, RED, partial-signature/serialized controls.
3. `30071d654` — GREEN/source replay receipts, seals and model-boundary docs.
4. Subsequent receipt/handoff commit adds portability evidence/docs/seals only; tested source/validator/test/historical-tool/model/profile/control bytes unchanged.

## Exclusions

No runtime changes/shim/fallback/retirement; no vendor/Wowless/cache writes. No all-publication/check/build/readability/startup/smoke/final gates, delegation, Bash, cwd switch, push/merge or provider/model/retry changes. Python manually formatted; no Rust edits. This handoff is not factory/native/API parity or integrated completion.
