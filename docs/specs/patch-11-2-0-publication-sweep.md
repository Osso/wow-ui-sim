# Patch 11.2.0 publication sweep

## Contract

Probe all 162 consolidated inventory occurrences from page 636685, revision 6726773, in unmodified cached Game UI. Publication/absence only; no signatures, outputs, security, behavior or native parity claim. No 11.x Cargo feature exists. Default retail carries 12.1.0; apply every later retained register chronologically (11.2.5 through 12.1.0). Latest later add/remove wins; changed rows preserve publication. Retain source direction and supersession IDs.

Use `tests/common/publication_sweep.rs`; exact observed gap IDs must equal `tests/data/patch_11_2_0_sweep_known_gaps.json`. `P1120_SWEEP_OUT` writes all observations before assertion; `P1120_SWEEP_REGISTER` accepts a full same-sized negative-control register. One altered unsuperseded row must add exactly one gap. Retail-only file gating. No Blizzard/vendor modifications, fabricated models, catalog shop changes or 11.2.7 fixture edits.

## Acceptance

- [ ] Review every non-OK row; fix cheap root causes, retain explicit model gaps.
- [ ] Exact gap fixture and one-row negative control.
- [ ] Unique inventory/extract coverage ledger and complete ranked scout.
- [ ] Eight sweeps run alone on local debug retail; later observations unchanged.
- [ ] Formatting, retail check, Mists test check without non-vendor warnings, startup `[]`.

## Bounded simulator fixes

Initial cached sweep reports 38 gaps, including four missing fog CVars/defaults, four retired members fabricated by namespace autostubs, a removed Browser method, and missing combat/gradient readbacks. Five new behavioral tests reproduce all five defect groups before implementation. Register the existing live player combat field, read FontString gradient slot 0, add documented zero fog defaults, mark four removed members using existing C API retirement machinery and exclude Browser NavigateTo on supported retail epochs. Classic contracts stay unchanged; new helpers/registrations use `retail-12-0-0`. Gradient cleared/default `(0, 0)` is simulator policy inferred from empty state, not native proof.
