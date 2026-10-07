# Patch 11.1.7 publication sweep

## Contract

Probe all 48 inventory occurrences from page 628473, revision 6726774 in unmodified cached Game UI. Default retail carries 12.1.0; no 11.x feature exists. Apply later 11.2.0, 11.2.5, 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 registers chronologically. Latest add/remove wins; changes preserve publication. Preserve source direction and supersession IDs.

Shared publication sweep requires exact reviewed gap IDs. P1117_SWEEP_OUT writes all observations before assertion; P1117_SWEEP_REGISTER permits a full same-sized negative control. Publication/absence only, not signature, output, security, behavior or native parity. Cached Blizzard deprecated wrappers remain intact.

## Acceptance

- [ ] Review all gaps and fix bounded model-backed defects.
- [ ] Retain exact fixture, non-inventory extract and exhaustive page ledger.
- [ ] Run nine isolated publication sweeps and one-row negative control.
- [ ] Run new behavioral tests, formatting, Mists test check and startup [].

## Bounded fixes

Two removed C_Debug members were registered unconditionally. Move the retained pre-retirement implementation under src/c_api/c_debug.rs and register only outside retail-12-0-0; mark removed keys to prevent namespace autostub fabrication. Never remove cached Blizzard deprecated wrappers. Existing legacy test runs only before retirement. Two documented graphics commands join the existing typed console catalog, not CVar storage; execution and native metadata remain unmodeled.

Two behavioral tests reproduce all four defects before implementation. An earlier assisted-slot candidate did not compile: AssistedCombatState contains only next_cast_spell_id, not an assisted action spell identity. Next-cast recommendation is not the rotation action spell; defer that producer rather than fabricate it.
