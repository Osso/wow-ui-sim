# Patch 11.2.5 publication sweep

## Contract

Probe all 163 consolidated inventory occurrences from page 641912, revision 6726772, in unmodified cached Game UI. Publication/absence only; no signatures, outputs, security, behavior or native parity claim. No 11.x Cargo feature exists. Default retail carries 12.1.0; apply 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 chronologically. Latest later add/remove wins; changed rows preserve publication. Source directions and supersession IDs remain recorded. 11.2.7 already excludes older registers.

Use `tests/common/publication_sweep.rs`; exact observed gap IDs equal `tests/data/patch_11_2_5_sweep_known_gaps.json`. `P1125_SWEEP_OUT` writes every observation before assertion; `P1125_SWEEP_REGISTER` accepts a full same-sized negative-control register. One altered unsuperseded row must add exactly one gap. Retail-only file gating; any retirement must preserve classic contracts and supported retail epochs. No vendor rewrite, fabricated models, or concurrent 11.2.7 producer edits.

## Acceptance

- [ ] Review every non-OK row; fix cheap root causes and retain explicit model gaps.
- [ ] Exact gap baseline and one-row negative control.
- [ ] Unique inventory/extract ledger and complete extract scout.
- [ ] Seven isolated sweeps, local debug retail.
- [ ] Format, retail check, Mists test check without non-vendor warnings, startup `[]`.

## Location preference publication fix

Both location visibility globals already have a state-backed implementation, synchronous event notifications, strict boolean/secret-caller validation and per-environment isolation. Their module and registration were gated to Forever only despite the 11.2.5 introduction. Publish that unchanged model from `retail-12-0-0` as well; classic builds retain absence, Forever retains its existing contract. Reuse the five behavioral tests under the same widened gate. RED: all five fail on missing globals. No recent-allies namespace producer changes.
