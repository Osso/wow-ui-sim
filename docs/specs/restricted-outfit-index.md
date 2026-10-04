# Restricted-environment outfit index

Retained `prose-2026-03-12-032`: "Added GetTransmogOutfitIndex to the restricted environment. Values returned from this can be used with the new /outfit slash command." The actual cached retail declaration is `Blizzard_RestrictedAddOnEnvironment/RestrictedEnvironment.lua:216–224`, not a standalone generated API. Its exported wrapper queries `C_TransmogOutfitInfo.GetOutfitInfo(outfitID)` and returns the record's player-facing index, or nil. Reuse [outfit catalog records](outfit-catalog-lookups.md) and the separately integrated [outfit action/command](outfit-action-command.md).

## What it must do

All boxes remain unchecked: authored tests, not executed proof.

- [ ] Load the real cached RestrictedAddOnEnvironment dependency closure. Execute actual secure snippets using GetTransmogOutfitIndex; no copied helper, patched vendor code or fallback secure-handler substitute.
- [ ] Resolve outfit IDs91/305 to sparse explicit indices7/42, not IDs or list positions. Unknown ID returns nil. Host catalog changes/deletions are reflected by cached snippets, and independent environments keep separate catalogs.
- [ ] Use real snippet-produced macro text with the existing C_Macro.RunMacroText `/outfit` consumer, selecting the intended catalog outfit and exercising bang/nonbang toggle behavior. No duplicate consumer/provider or index map.
- [ ] Load the entire unchanged cached RestrictedEnvironment.lua in its secure environment with its addon-table export to test the real helper's original getter boundary directly: secure secret IDs resolve, addon secret IDs reject, ordinary addon IDs resolve. This is not proof that insecure CallRestrictedClosure entry is allowed; it tests the helper/getter boundary separately from snippet infrastructure.
- [ ] Preserve actual restricted closure rejection of direct table creation and successful subsequent helper invocation.

## How it works

- [C API and environment boundary](../lua-api.md)
- [Catalog lookup contract](outfit-catalog-lookups.md)
- [Existing outfit consumer contract](outfit-action-command.md)

## Implementation inventory

- `tests/restricted_outfit_index.rs`: four authored real-cache/real-snippet behavioral cases.
- Existing `src/c_api/c_transmog_outfit_info/catalog.rs`: sole catalog getter; no changes proposed.
- Existing `src/c_api/c_transmog_outfit_info/actions.rs`: active selection and macro consumer; no changes proposed.
- Cached vendor RestrictedEnvironment.lua/RestrictedExecution.lua/SecureHandlers.lua: unchanged target.

## Tests asserting this spec

`tests/restricted_outfit_index.rs` loads the actual cached TOC through the existing closure harness. Existing `tests/patch_12_0_5_outfit_catalog.rs` and `tests/outfit_action_command.rs` remain unchanged; their passing results cannot substitute for executing these helper tests.

## Development proof and independent bounded acceptance — 2026-10-03 (restricted-outfit-index)

Commit `4d142e325`. RED: none: 4/4 passed before any change (existing capability). GREEN: 4/4. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-housing-bars.md) SHA256 `6cc70517c54675b995a442bc47d2fa681c21e457ea32e5dd0d9d582baa6fc65d`. Existing behavior proven by new tests; no producer change. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-12-032 bounded-coverage under capability `restricted-outfit-index`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Execute four cases against the current integrated producer. They may already pass. No missing simulator exposure has been demonstrated; therefore none is authored.
- [ ] Validate secure-environment export, addon loading and getter-boundary behavior in the pinned VM; formatting alone does not establish runtime behavior.

## Out of scope

Fallback secure-handler helper exposure, new outfit records/allocation/mutation/persistence, duplicate `/outfit` parsing, native secret/scrub or protected-action parity and full-UI/profile verification. Existing catalog miss/validation/matching policies and command toggle/clear/index policies remain INFERRED as documented in their owning specs; helper forwarding adds no new native claims. Macro invocation uses the actual consumer but does not simulate a protected macro button click.
