# Performance metrics

Retail and WoW Forever expose simulator inputs for the legacy performance globals used by Blizzard UI. These are supplied scenario values, not live measurements.

## SetFramerate(fps)

Supplies the number returned by `GetFramerate()`. The scenario default is `60` FPS when no value is supplied.

```lua
A_Admin.SetFramerate(47.5)
```

## SetAddOnUsage(name, memoryKB, cpuMilliseconds)

Supplies per-addon memory and CPU values. Memory is in KB; CPU is in milliseconds. `UpdateAddOnMemoryUsage()` and `UpdateAddOnCPUUsage()` copy the supplied values into their respective published snapshots. Getters return the last published value, or `0` before the first update. `ResetCPUUsage()` clears the supplied and published CPU values.

```lua
A_Admin.SetAddOnUsage("MyAddon", 240, 7.5)
UpdateAddOnMemoryUsage()
UpdateAddOnCPUUsage()
print(GetAddOnMemoryUsage("MyAddon"), GetAddOnCPUUsage("MyAddon"))
```

The sampling and reset behavior is **INFERRED (R88fg, native evidence pending)**. `GetFrameCPUUsage` remains an unmodeled compatibility default because the audited runtime-selected Retail and Forever UI consumers do not call it.
