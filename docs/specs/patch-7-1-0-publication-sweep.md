# Patch 7.1.0 publication sweep

Audit the [pinned Warcraft Wiki page](../../data/patch-api/sources/7.1.0-api-changes.wikitext), revision 3742660. Current publication is not historical/native parity; see [audit](../wiki/investigations/patch-7-1-0-api-audit.md).

## What it must do

- [x] Account for every registered identity and retained plaintext statement without dropping XML or qualified mouse-bug prose.
- [x] Probe raw publication/lookup and current superseded absences after unmodified cached retail startup; require the exact known-gap set.
- [x] Preserve existing source inputs and extraction outcomes; reproduce every register with its recorded/inherited flags.
- [ ] Bound independent clipping state, inherited `clipChildren` before OnLoad and explicit false overrides; do not credit custom intrinsic factories or pixel rendering from flag queries.
- [x] Bound physical screen dimensions and existing item catalog queries through current successor APIs, without crediting constant item-set/reagent defaults as modeled metadata.
- [ ] Compile Mists tests with zero non-vendor warnings and validate portable, revision-scoped evidence.

## How it works

- [Audit and proof ledger](../wiki/investigations/patch-7-1-0-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).
- [XML template system](../xml-template-system.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: existing opt-in top-level API bullets plus additive `--legacy-widget-cvar-bullets` parser.
- `tests/patch_7_1_0_publication_sweep.rs`: cached publication; 7.2.0 placeholder precedes 7.2.5 and all merged later registers.
- `tests/patch_7_1_0_behavior.rs`: inherited clipping state and real catalog binding/expansion/base item level.
- `data/patch-api/sources/7.1.0-page-coverage.json`: occurrence ledger, bounded credit and explicit pending contracts.
- `data/patch-api/evidence/7.1.0-session-2026-10-08/validate.py`: read-only historical scope and hashed proof validation.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py`: method/CVar summaries and underscore API links; default parsing remains unchanged.
- `tests/patch_7_1_0_publication_sweep.rs`: exact gap accounting and one-row negative control.
- `tests/patch_7_1_0_behavior.rs`: clipping state and item successor metadata.
- `tests/screen_mode.rs`, `tests/c_item_api/`, `tests/intrinsic_types.rs`: bounded existing-model regressions.

## Known gaps (current cycle)

- [ ] Arbitrary registered custom intrinsic types: runtime tag mapping rejects the retained custom-template probe; known aliases are not general support.
- [ ] Complete item four-return semantics: itemSetID/reagent outputs lack backing metadata; legacy globals are superseded.
- [ ] Final historical mouse-entry policy: source explicitly qualifies this as a pre-release bug.
- [ ] Historical ScrollingMessageFrame migration parity: current Lua-template mapping does not prove the pinned old implementation.
- [ ] TitleRegion native removal contract: bare object name supplies no factory/method contract; no guessed retirements.

## Out of scope

Native 7.1.0 client reconstruction, unexpanded linked pages, guessed metadata or factory shims, vendor Lua edits, pixel-clipping/native protected-policy claims. Historical personal-nameplate behavior is not credited from later CVar removal. No runtime paths are changed by this audit.
