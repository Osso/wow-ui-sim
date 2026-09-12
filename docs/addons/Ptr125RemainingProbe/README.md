# PTR 12.1.5 remaining-contract probe

**Installed and captured once on desktop PTR; rerun pending after the absent-aura fix.** The first non-secret structure capture matched pinned build `12.1.5.69594`; its exact artifact and interpretation live in [the PTR audit](../../wiki/investigations/patch-12-1-5-api-audit.md#native-structure-capture-not-an-audit-upgrade). `secrets` is deferred, not native conformance. Local fixture tests validate the recorder, not native behavior. Audit rows stay evidence-required until reviewed native results justify a specific upgrade.

Target: **12.1.5.69594**, interface **120105**. Every run records available `GetBuildInfo()` fields and marks the pinned-build match `true`, `false`, or `unknown` when build metadata is unavailable. Results from another or unknown build are not pinned-build proof.

## Coverage

| Unresolved rows | Experiment | Limit |
|---|---|---|
| `CreateRegionParams` + four fields | Documented positional region creation versus candidate options-table calls; observe name, layer/sublevel, and template size | A rejected table call does not prove no other consumer exists |
| `TimedSignalMapEntry` + two fields | Capture ordinary `GetNextSignal()` arity/types and candidate `key`/`time` fields | Does not assume a table or global type exists |
| `RequiresTimedSignalMapAccess` | Observe direct and native `securecallfunction` attempts on an addon-owned map containing an actually observed secret timestamp | Without a verified secret-bearing fixture, result is inconclusive |
| `SecretWhenLuaTableHasSecretKeys` | Observe count/count-info/empty results for an addon-owned table with an actually observed secret key | Does not manufacture secrets or assume a privileged caller |

## Manual capture

1. The probe is installed at `C:/World of Warcraft/_xptr_/Interface/AddOns/Ptr125RemainingProbe/` on the desktop PTR. To reinstall it from the repository root, run `python3 docs/addons/Ptr125RemainingProbe/deploy.sh`.
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

The two XML templates are probe-owned controls. Their dimensions let the capture distinguish a processed `templateName` from an ignored argument. Compare a candidate table-form call only when its matching positional control succeeds. The clock must be a verified public, accessible, finite number before deadline arithmetic. A populated-map return shape is inspected only after `HasSignal` and `GetSignalTime` confirm the scheduled key and time; a successful call alone is insufficient. No generated-documentation structure is fabricated as a global.

The first structure capture recorded `matchesPinnedBuild = true`; its non-secret observations are kept only in the [audit artifact](../../baselines/ptr-12-1-5-remaining-structures.lua). `secrets` remains deferred. Commit `d48aa95da` skips absent or inaccessible aura records, but no post-fix native rerun has been reviewed.

For clean captures, record which other addons were enabled; they may wrap APIs. This probe does not change addon settings. If cleanup cannot confirm an empty map, the run records that limitation and retries once; reload after saving when safe.

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
