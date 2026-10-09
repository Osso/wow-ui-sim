# Retail Patch 3.0.2 frozen source audit

Frozen page 482353/revision 4638841 (`2020-02-23T21:55:29Z`) is historical retail, not Wrath Classic. Source input lives in `data/patch-api/source-cache/legacy-2026-10-09`. [Audit](../wiki/investigations/patch-3-0-2-api-audit.md).

## What it must do
- [x] Opt-in launch parser retains labeled NEW/UPDATED/MODIFIED/REMOVED, bare unknowns, template-linked calls, missing bullet, handlers and literal summary references; default bytes remain unchanged.
- [ ] Account every nonblank source row, header, inventory occurrence, signature and substantive prose/qualification independently, without repairing literal typos or inconsistent returns.
- [ ] Preserve original ledger/gaps/source/code/input/log archives separately from closure claims, with byte seals and Git-free fresh-process replay.
- [ ] Reject serialized source, ledger, log and tool tampering; restore exact bytes before continuing replay.
- [ ] Restrict successor interpretation to retail; actual 3.3.0/3.3.3/3.3.5/4.0.1 inputs and ordered 3.0.3/3.0.8/3.1.0/3.2.0 placeholders do not prove runtime/native parity.

## How it works
- [Audit and coverage matrix](../wiki/investigations/patch-3-0-2-api-audit.md)
- [Bounded handoff](../../data/patch-api/evidence/3.0.2-session-2026-10-09/handoff.md)

## Implementation inventory
- `tools/gen_patch_wikitext_register.py`: opt-in `--wrath-launch-inventory`.
- `data/patch-api/sources/3.0.2-*`: convenient frozen register, extract, coverage ledger and provenance.
- `data/patch-api/evidence/3.0.2-session-2026-10-09/accounting.py`: deterministic source ledger and precise limits.
- Evidence `validate.py`: sealed original replay, independent of mutable sources/target/Git.
- Evidence `original/` and `closures/`: distinct source-accounting and empty behavioral-claim archives.

## Tests asserting this spec
- `tools/test_patch_3_0_2_source.py`: concrete serialized inventory, 346 labeled rows, occurrence preservation and unchanged defaults.
- Evidence `test_accounting.py`: complete omission controls, false-credit/foreign-history rejection, relocated serialized tamper/restoration controls.

## Known gaps (current cycle)
- [ ] All 373 inventory, 80 prose/qualification and 367 signature contracts remain UNPROVEN behaviorally; runtime publication and native absence were not measured.
- [ ] No historical 3.0.2 profile/runtime/native capture; existing modern-retail names do not establish old return or storage contracts.
- [ ] Main replaces queued placeholders with actual ordered registers and owns native/integration acceptance.

## Out of scope
Classic 3.4.x/TBC/Era supersession, linked-source expansion, guessed contracts, shims/fallbacks, vendor/cache/Wowless edits, push/merge/deploy/delegation and broad/final gates. No runtime changes or retirement claims.
