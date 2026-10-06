# Patch 11.1.7 publication sweep

## Contract

Probe all 48 inventory occurrences from page 628473, revision 6726774 in unmodified cached Game UI. Default retail carries 12.1.0; no 11.x feature exists. Apply later 11.2.0, 11.2.5, 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 registers chronologically. Latest add/remove wins; changes preserve publication. Preserve source direction and supersession IDs.

Shared publication sweep requires exact reviewed gap IDs. P1117_SWEEP_OUT writes all observations before assertion; P1117_SWEEP_REGISTER permits a full same-sized negative control. Publication/absence only, not signature, output, security, behavior or native parity. Cached Blizzard deprecated wrappers remain intact.

## Acceptance

- [ ] Review all gaps and fix bounded model-backed defects.
- [ ] Retain exact fixture, non-inventory extract and exhaustive page ledger.
- [ ] Run nine isolated publication sweeps and one-row negative control.
- [ ] Run new behavioral tests, formatting, Mists test check and startup [].
