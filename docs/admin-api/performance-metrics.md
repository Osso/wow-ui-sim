# Performance metrics

Retail and WoW Forever expose simulator inputs for the legacy performance globals used by Blizzard UI. These are supplied scenario values, not live measurements.

## SetFramerate(fps)

Supplies the already-smoothed number returned by `GetFramerate()`. **R88fg VERIFIED:** the default is `60` FPS on Retail and `240` FPS on WoW Forever.

```lua
A_Admin.SetFramerate(47.5)
```

## SetAddOnUsage(name, memoryKB, cpuMilliseconds)

Supplies per-addon memory and CPU values. **INFERRED (R88fg, native evidence pending):** memory is in KB, CPU is in milliseconds, and each Update function publishes a snapshot of the supplied values. **R88fg VERIFIED:** `GetAddOnMemoryUsage` reads stored per-addon values rather than measuring live; `GetAddOnCPUUsage` returns `0` when an addon has no record; both Update functions return no values. `ResetCPUUsage()` clears the supplied and published per-addon CPU values.

```lua
A_Admin.SetAddOnUsage("MyAddon", 240, 7.5)
UpdateAddOnMemoryUsage()
UpdateAddOnCPUUsage()
print(GetAddOnMemoryUsage("MyAddon"), GetAddOnCPUUsage("MyAddon"))
```

The sample units and snapshot publication behavior are **INFERRED (R88fg, native evidence pending)**. **R88fg VERIFIED:** `GetFrameCPUUsage` returns time then count; it remains an unmodeled compatibility default because the audited runtime-selected Retail and Forever UI consumers do not call it.
