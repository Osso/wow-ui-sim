# Historical retail Patch 3.3.5 source audit

Audit pinned page 25049/revision 247986 (2010-07-10T15:47:20Z). [Audit](../wiki/investigations/patch-3-3-5-api-audit.md).

## Requirements

- [x] Validate frozen manifest identity and literal response/wikitext hashes before copying.
- [x] Opt-in parser retains sectioned colon lists, changed prose identity and explicit signature/return fragments at original lines; original outputs remain byte-identical.
- [x] Account every inventory, retained prose/heading and explicit signature independently; publication never proves behavior.
- [x] Own retail sweep uses actual 4.1.0+ retail registers; 4.0.1 remains an integration placeholder. Wrath Classic 3.4.x cannot supersede historical retail.
- [ ] Portable historical replay derives totals from immutable archived ledger/gaps/receipts, independent of Git, target or future current closures; source/log/ledger tampering rejected.

## Implementation and tests

- `tools/gen_patch_wikitext_register.py`: `--legacy-section-lists` opt-in.
- `tools/test_patch_3_3_5_register.py`: concrete serialized inventory/signatures and default compatibility.
- `data/patch-api/sources/3.3.5-*`: frozen source, register, full-page extraction and 324-ID accounting.
- `tests/patch_3_3_5_publication_sweep.rs`: own cached retail publication/absence.
- `tools/test_patch_3_3_5_validator.py`: relocated replay/current-closure independence and source/log/ledger/gaps/bundle tamper controls.
- Own `data/patch-api/evidence/3.3.5-session-2026-10-09/validate.py`: sealed archived replay, no Git/target/current-head dependencies.

## Recorded development boundary

Own publication GREEN 1/1 (80 matches / 44 known gaps), parser 1/1, existing temporary chat state 1/1; negative rejects 44 → 45 gaps. Full ledger has 324 IDs and no new modeled closures. Portable validator GREEN remains a separate targeted development receipt; main owns current coordinator/final proof.

## Limits

No linked contract expansion, invented service/state model, compatibility shim, vendor/cache writes, Classic mutation, broad/final acceptance, push or merge. Existing saved chat geometry is temporary session-only behavior, not account persistence. Inspect throttling requires a server request/response model; no promise of an event on every request.
