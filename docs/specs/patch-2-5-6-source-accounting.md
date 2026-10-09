# Patch 2.5.6 Classic/TBC source accounting

Account for frozen page `685353`, revision `6778086`, timestamp `2026-07-22T05:42:00Z`; literal TOC **20506** defines the 205xx Classic/TBC source boundary, not historical retail. [Ledger](../../data/patch-api/sources/2.5.6-page-coverage.json); [audit](../wiki/investigations/patch-2-5-6-api-audit.md).

## What it must do

- [ ] Validate response identity/content/hash against the frozen manifest before copying; retain every nonblank source row and exact reproducible plaintext boundary.
- [ ] Keep unexpanded Blue posts transclusion UNPROVEN, with absent identity/signature/output/event/state/security/native-equivalence contracts explicit; no positive empty-inventory credit.
- [ ] Record actual configured feature/interface/cache/manifests separately from native source TOC; neither profile names, missing mappings nor manifest contents diagnose unsupported runtime behavior.
- [ ] Reject cross-client successors, omitted rows/contracts, fabricated API/signature/summary/header credit and measurements.
- [ ] Replay sealed historical tools/source/configuration without Git, target or current mutable repo/cache files; reject serialized ledger/log tampering and restore exact bytes/hashes.

## How it works

- [Literal boundary, coverage matrix and historical proof](../wiki/investigations/patch-2-5-6-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/2.5.6-api-changes.{wikitext,txt}` — raw source and limited plaintext.
- `data/patch-api/sources/2.5.6-page-coverage.json` — source ledger, not publication register.
- `data/patch-api/evidence/2.5.6-session-2026-10-09/` — frozen response/pin/registry/manifest, copied tools/configuration, observation, validator, tests, controls and seals.

## Tests asserting this spec

`python3 data/patch-api/evidence/2.5.6-session-2026-10-09/test_source_accounting.py` tests serialized SOURCE accounting. `validate.py` derives source/configuration totals from historical sealed inputs. `replay_controls.py` mutates/restores actual serialized artifacts and runs a fresh relocated validator from an own archive. Tests remain source-only; no simulator is invoked. Targeted RED retained; implementation GREEN/control execution pending at first commit.

## Known gaps (current cycle)

- [ ] Unexpanded Patch_1.15.9/API_changes blue-post content: member identities, arguments/returns, event payloads, state/security rules and TBC equivalence remain unspecified/unproven.
- [ ] Native/model/publication/full-UI measurements unperformed; coordinator retains ownership, not blocked or diagnosed by this source slice.

## Out of scope

Linked source reconstruction, other registry page audits, default-retail publication register, shared classifier expansion, runtime/profile/shim/retirement edits, vendor/cache/Wowless changes, broad/check/lint/build/startup/full-suite/final gates, delegation and operations. Main owns integration and native/final gates; this branch does not push/merge/deploy.
