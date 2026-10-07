# Patch 11.1.0 publication sweep

## Contract

Probe every inventory occurrence from Warcraft Wiki page 616105, revision 6726776, against unmodified cached Game UI. Default retail carries 12.1.0, not historical 11.1.0. Apply all ten later registers, 11.1.5 through 12.1.0, chronologically. Latest add/remove wins; changes preserve publication. Preserve original direction and supersession IDs. Require exact reviewed gap IDs; write all observations before asserting the fixture. P1110_SWEEP_OUT selects results; P1110_SWEEP_REGISTER selects a same-sized negative-control register.

Publication/absence only: no signature, output, security, behavior or native parity claim. Explicit registration may still be a placeholder. Generic namespace autostub lookup is not explicit publication. Never delete cached Blizzard deprecation wrappers.

## Bounded behavior

- Removed C_BarberShop.GetCustomizationScope and C_TransmogCollection.CanAppearanceBeDisplayedOnPlayer stay absent on repeated ordinary/raw namespace lookup. Current cached retail Lua has no consumers. Classic registration remains unchanged.
- GetSpecializationNameForSpecID returns the existing English specialization catalog name for valid IDs (70 Retribution, 65 Holy, 577 Havoc), and nil for unknown IDs. Gender does not alter this catalog's names; localized gender-specific names remain unproven.
- SetSpecialization stays published: cached Blizzard_TalentUI/Mists/Blizzard_TalentUI.lua:70 still calls it. Retain the historical removal as a gap, not deprecated-alias credit.
- Multiply indented changed API rows must be retained with their own annotation; C_PlayerInfo.GetSex is a distinct row, not an annotation on the preceding mount API.

## Acceptance

- [ ] Exact gap fixture and exhaustive inventory/non-inventory ledger.
- [ ] Every publication sweep alone, new behavioral tests and one-row negative control.
- [ ] Relevant isolated prefork cases, formatting, Mists test check without non-vendor warnings and startup [].

## Proof

See data/patch-api/evidence/11.1.0-session-2026-10-06/p1110-proof.json for revision-scoped local commands and results. No independent/native acceptance; no full suite.
