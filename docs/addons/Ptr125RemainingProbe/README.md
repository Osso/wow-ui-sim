# PTR 12.1.5 remaining-contract probe

**Prepared only. Not installed or executed in WoW.** Local fixture tests validate the recorder, not native behavior. Audit rows stay evidence-required until captured results are reviewed.

Target: **12.1.5.69594**, interface **120105**. Every run records `GetBuildInfo()` and whether it matches this target. Results from another build are not pinned-build proof.

## Coverage

| Unresolved rows | Experiment | Limit |
|---|---|---|
| `CreateRegionParams` + four fields | Documented positional region creation versus candidate options-table calls; observe name, layer/sublevel, and template size | A rejected table call does not prove no other consumer exists |
| `TimedSignalMapEntry` + two fields | Capture ordinary `GetNextSignal()` arity/types and candidate `key`/`time` fields | Does not assume a table or global type exists |
| `RequiresTimedSignalMapAccess` | Observe direct and native `securecallfunction` attempts on an addon-owned map containing an actually observed secret timestamp | Without a verified secret-bearing fixture, result is inconclusive |
| `SecretWhenLuaTableHasSecretKeys` | Observe count/count-info/empty results for an addon-owned table with an actually observed secret key | Does not manufacture secrets or assume a privileged caller |

## Manual capture

1. Copy this folder to the actual PTR client's `Interface/AddOns/Ptr125RemainingProbe/`, keeping the TOC directly inside that folder. See [addon placement](../create-and-install-wow-addon.md); Desktop staging is not installation.
2. Enable the addon and run `/ptr125probe structures`.
3. Run `/ptr125probe secrets`. Repeat only when your normal player/target state supplies suitable secret values; the addon does not target, cast, enter combat, or modify auras.
4. Run `/ptr125probe status`. Later, when safe, `/reload` or log out to flush SavedVariables.
5. Retain `WTF/Account/<ACCOUNT>/SavedVariables/Ptr125RemainingProbe.lua` with the capture conditions. Review it locally before sharing or committing it.

No tests run automatically on login. Commands are `structures`, `secrets`, and `status`; there is no automatic gameplay loop or reset command.

## Safety and interpretation

- Only addon-owned hidden frames, tables, and signal maps are created. Cleanup hides owned frames and cancels owned signals. Reloading after capture removes remaining UI objects; existing UI is not modified.
- Raw secret values, tables, userdata, functions, and threads are never deliberately stored in SavedVariables. Secret or unknown-access results are described only by type/access metadata. The probe does not stringify secret values.
- `securecallfunction` is an **attempt label**, not proof of Blizzard-private or untainted execution. Compare the recorded `issecure` markers and fixture-verification observations; do not infer privilege from the wrapper's name or `pcall` success alone.
- Missing APIs, missing secret sources, denied fixture construction, and inaccessible results remain unavailable/inconclusive—not successful conformance tests. Native wrappers may handle errors internally; inspect the resulting fixture, not just the call status.
- No `forceinsecure`, secure-environment writes, global error-handler replacement, protected gameplay actions, security bypass, or throttling/VM work is included.

The two XML templates are probe-owned controls. Their dimensions let the capture distinguish a processed `templateName` from an ignored argument. No generated-documentation structure is fabricated as a global.

## Local preparation checks

From the repository root:

```text
luajit docs/addons/Ptr125RemainingProbe/tests/harness.lua docs/addons/Ptr125RemainingProbe
luajit docs/addons/Ptr125RemainingProbe/tests/structures.lua docs/addons/Ptr125RemainingProbe
luajit docs/addons/Ptr125RemainingProbe/tests/secrets.lua docs/addons/Ptr125RemainingProbe
```

These use controlled fake APIs and secret markers **only in tests**. They prove redaction, recording, cleanup, and inconclusive-case handling—not WoW secrecy, timing, structure transport, or native return values.

## Evidence boundary

Source inventory: [`12.1.5-register.json`](../../../data/patch-api/sources/12.1.5-register.json). Current dispositions: [PTR occurrence inventory](../../wiki/investigations/patch-12-1-5-occurrence-inventory.md).

Do not upgrade the ten rows from this preparation work. Interpret a native capture per operation, client build, execution context, and whether the required secret fixture was actually established.
