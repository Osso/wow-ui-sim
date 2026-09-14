# AuraDispelCurveProbe

Manual native recorder for `C_UnitAuras.GetAuraDispelTypeColor`. It captures addon-accessible player-aura observations only; it does not change auras, invoke protected actions, deploy itself, or infer a dispel-type ID.

## Target and install

The TOC uses interface **120100** because the pinned local retail target is **12.1.0.69497**, not the observed desktop retail client `12.0.5.67823` (interface `120005`). No deployment or native capture has happened.

Manually copy `AuraDispelCurveProbe` into the matching client's `Interface/AddOns/` directory, enable it, then `/reload`. Do not use a deployment script: none exists for this probe.

## Native capture

1. Log in on the matching client with one or more real player buffs/debuffs visible. The recorder scans both `HELPFUL` and `HARMFUL` auras, up to 40 entries per filter.
2. Run:

   ```text
   /auradispelcurve
   ```

3. Repeat after a deliberately different real-aura scenario—for example, a visible dispellable helpful aura and a visible harmful aura. Do not manufacture IDs or edit SavedVariables in-game.
4. Run `/reload` or log out to flush SavedVariables.
5. Retain the raw capture from:

   ```text
   WTF/Account/<ACCOUNT>/SavedVariables/AuraDispelCurveProbe.lua
   ```

The database retains at most ten captures; later command uses increment `dropped`.

## Recorded evidence

Each manual capture records client metadata, raw accessible aura name/dispel name/instance ID, filter/index, failures, and two explicitly linear color-curve definitions:

- points `0 → 32`: green to red, with blue `0.25`;
- points `-16 → 48`: blue to orange.

For each accessible aura, it calls `C_UnitAuras.GetAuraDispelTypeColor("player", auraInstanceID, curve)` with both curves and records accessible RGBA components. Compare raw results across both curves and real aura scenarios. Do not infer dispel IDs, default interpolation, native validation, security semantics, or a global curve-input scale from those results. A rejected or restricted addon-tainted call is **inconclusive**; do not attempt a bypass.

The local fixture command is:

```text
luajit docs/addons/AuraDispelCurveProbe/tests/harness.lua docs/addons/AuraDispelCurveProbe
```

It proves recorder control flow, redaction, bounded capture storage, and unavailable-access handling. It is not native evidence and does not establish dispel IDs.

See [probe spec](../../specs/aura-dispel-curve-probe.md) and [investigation](../../wiki/investigations/aura-dispel-curve-probe.md).
