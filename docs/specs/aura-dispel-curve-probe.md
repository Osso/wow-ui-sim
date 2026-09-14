# Aura dispel curve probe

`AuraDispelCurveProbe` prepares a manual native observation of `C_UnitAuras.GetAuraDispelTypeColor` without claiming an aura dispel-ID mapping. Its recorder is `docs/addons/AuraDispelCurveProbe/AuraDispelCurveProbe.lua`. See [investigation](../wiki/investigations/aura-dispel-curve-probe.md).

## What it must do

- [x] Expose only the manual `/auradispelcurve` command; never capture automatically.
- [x] Scan accessible player `HELPFUL` and `HARMFUL` aura records, bounded to 40 entries per filter.
- [x] Record only accessible scalar aura fields and RGBA output from two supplied, explicitly linear color curves.
- [x] Record restricted, invalid, enumeration-error, missing-API, and curve-construction outcomes without serializing inaccessible values.
- [x] Retain at most ten SavedVariables captures and count dropped requests.
- [ ] Obtain and review a native capture from the pinned retail `12.1.0.69497` / interface `120100` target.

## How it works

- [Aura dispel curve probe investigation](../wiki/investigations/aura-dispel-curve-probe.md)
- [Base spell aura secrecy](spell-aura-secrecy.md)

## Implementation inventory

- `docs/addons/AuraDispelCurveProbe/AuraDispelCurveProbe.toc`: pinned interface and SavedVariables declaration.
- `docs/addons/AuraDispelCurveProbe/AuraDispelCurveProbe.lua`: manual bounded recorder.
- `docs/addons/AuraDispelCurveProbe/tests/harness.lua`: controlled recorder fixture.

## Tests asserting this spec

At `261b4cbe6`, exact-byte reuse confirms the original harness; a supplemental fixture passes 18/18 for real TOC/slash/SavedVariables wiring, secret/access redaction, opaque errors, curve-construction failures, and the 80-scan/10-capture limits. This is recorder proof only, not native execution or dispel-ID evidence. See `/tmp/verify-aura-dispel-probe-ledger.json`.

## Known gaps (current cycle)

- [ ] Native real-aura capture has not been installed, run, or reviewed.
- [ ] Native dispel-ID mapping, interpolation semantics, validation/coercion, unknown-aura behavior, secrecy/security, and full-LoD availability remain unverified.

## Out of scope

Runtime changes, API occurrence credits, SavedVariables parsing, security bypasses, synthetic secret values, deployment, and interpreting fixture IDs as native IDs.
