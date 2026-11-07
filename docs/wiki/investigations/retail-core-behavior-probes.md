# Retail Core Behavior Probes

Retail core-frame probes captured several places where simulator and reference-client behavior diverged outside Blizzard addon loading.

## Content

`CoreBehaviorProbe` is a small live-client addon used to capture SavedVariables for core UIObject/frame APIs. It stores scalar summaries instead of raw userdata/table results so the data survives logout/reload cleanly.

Real retail `12.0.5.67823` showed these behaviors:

- `CreateForbiddenFrame` is nil on the current retail client.
- `CreateFrame("Frame"):SetForbidden(true)` succeeds but does not mark a normal frame forbidden; `IsForbidden()` stays false.
- `RegisterUnitEvent("UNIT_HEALTH", "player")` registers and `IsEventRegistered("UNIT_HEALTH")` reports `true, "player"`.
- `RegisterUnitEvent("UNIT_HEALTH", "not_a_unit")` also registers, but `IsEventRegistered("UNIT_HEALTH")` reports `true, nil`; invalid filters fall back to an event-only registration rather than failing.
- `SetAttribute("*type1", false)` must preserve explicit false through wildcard `GetAttribute("help", "type", "1")`.

The simulator now matches the confirmed retail behavior for invalid `RegisterUnitEvent` filters, retail `SetForbidden` on addon-created normal frames, wildcard attribute lookup preserving explicit false, and Lua-visible `GetRaisedFrameLevel()` for the simple Raise/Lower sibling probe. The insecure `SetForbidden` no-op is compiled for `profile-retail` and `client-ptr`; every other profile keeps the unrestricted behaviour for classic-profile compatibility shims that create forbidden frames. PTR carries the same restricted declaration, and its secure environment guards `MakeForbidden` with `issecure()`. WowForever publishes that declaration too, but its gate is not enabled. See [set-forbidden-restriction](../../specs/set-forbidden-restriction.md).

The first Raise/Lower probe was too weak because both frames reported `GetRaisedFrameLevel() == 0` before and after. The improved probe creates paired low/high siblings under both a private parent and `UIParent`, snapshots `GetFrameLevel()`, `GetRaisedFrameLevel()`, strata, show/visible state before/after `Raise()` and `Lower()`, and records whether the relative raised levels changed.

The fresh retail capture from `2026-06-10T16:26:09` kept the same values in both cases: low frame level `1`, high frame level `10`, both raised-frame levels `0`, and both frames visible/shown before and after `Raise()` and `Lower()`. The calls succeeded, but the probe found no Lua-visible ordering change for simple shown sibling frames.

The follow-up hit-order capture from `2026-06-10T16:52:38` kept mouse focus on the high frame before `low:Raise()`, after `low:Raise()`, and after `high:Lower()`. Both frames were full-screen, mouse-enabled, visible, shown, and in `DIALOG` strata; low had frame level `1`, high had frame level `10`, and both reported raised-frame level `0`. The simulator now keeps internal `raise_order` only as a same-raw-level tie-breaker, so Raise/Lower no longer lets lower frame levels overtake higher ones.

## Sources

- [CoreBehaviorProbe](../../../docs/addons/CoreBehaviorProbe) - live-client addon and install/readback notes
- [events.rs](../../../src/lua_api/frame/methods/text_attribute_event/events.rs) - `RegisterUnitEvent` implementation
- [attributes.rs](../../../src/lua_api/frame/methods/text_attribute_event/attributes.rs) - `SetForbidden` / `IsForbidden` implementation
- [admin_event_api.rs](../../../tests/admin_event_api.rs) - invalid unit-filter regression coverage
- [protected_frame_enforcement.rs](../../../tests/protected_frame_enforcement.rs) - Retail and PTR `SetForbidden` no-op regression coverage
- [frame_level.rs](../../../tests/frame_level.rs) - `GetRaisedFrameLevel()` Raise/Lower regression coverage
- [protected_attribute_enforcement.rs](../../../tests/protected_attribute_enforcement.rs) - wildcard `GetAttribute` explicit-false regression coverage

## See Also

- [[protected-frames]] - protected-frame enforcement and remaining gaps
- [[frame-surrogate-identity-slot]] - related live-client frame identity probe workflow
- [[lua-api]] - frame method dispatch and Lua API surface
