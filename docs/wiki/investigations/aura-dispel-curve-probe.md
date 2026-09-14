# Aura dispel curve probe

`AuraDispelCurveProbe` is a prepared manual recorder for addon-visible `C_UnitAuras.GetAuraDispelTypeColor` behavior. No native installation, deployment, run, or API-credit change has occurred.

## Protocol

The probe targets pinned local retail `12.1.0.69497`, interface `120100`. That pin—not the observed desktop retail `12.0.5.67823` / `120005` client—sets the TOC interface. A manual run scans real accessible player `HELPFUL` and `HARMFUL` auras, recording name, dispel name, aura instance ID, filter/index, and supplied-curve RGBA results. It must be flushed with `/reload` or logout before reading `WTF/Account/<ACCOUNT>/SavedVariables/AuraDispelCurveProbe.lua`.

The only capture command is `/auradispelcurve`. It supplies two explicitly linear, two-point color curves with different domains: `0..32` and `-16..48`. Comparing raw outputs across both curves and real aura scenarios may constrain observed client behavior. It does not justify an assumed dispel ID, default interpolation, input scale, validation rule, or security contract.

The recorder rejects inaccessible values from persisted output and labels API errors/restricted results. An addon-tainted rejection or restricted value is inconclusive. No bypass, privileged caller, or synthetic secret source is authorized.

## Proof level

Commits `f2d1e94b9` and `97b770c54` pass the local Lua fixture recorded in `/tmp/aura-dispel-probe-development-ledger.json`. The fixture proves manual-command registration, bounded recording, access redaction, and unavailable-access handling only. Its `101`/`202` fixture IDs and interpolation are not native evidence.

`C_UnitAuras.GetAuraDispelTypeColor` remains evidence-required in the 12.0.0 occurrence inventory. A reviewed native capture can support only the demonstrated operation, build, aura scenario, and access context.

## Sources

- [AuraDispelCurveProbe README](../../addons/AuraDispelCurveProbe/README.md) — install and capture procedure.
- [AuraDispelCurveProbe recorder](../../addons/AuraDispelCurveProbe/AuraDispelCurveProbe.lua) — stored fields, curves, and access handling.
- [Aura dispel curve probe spec](../../specs/aura-dispel-curve-probe.md) — bounded contract.
- [12.0.0 occurrence inventory](patch-12-0-0-occurrence-inventory.md) — current evidence-required status.

## See Also

- [[patch-12-0-0-api-audit]] — audit disposition.
- [[patch-api-blocker-inventory]] — evidence planning conventions.
