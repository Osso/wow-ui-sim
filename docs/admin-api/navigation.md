# Navigation target

`A_Admin.SetNavigationTarget(record)` supplies one navigation target for the currently super-tracked target. Select the target with `C_SuperTrack` first. The navigation record alone does not activate navigation. Pass `nil` to clear the record.

```lua
A_Admin.SetNavigationTarget({
    state = Enum.NavigationState.InRange,
    distance = 125,
    clamped = false,
    screenX = 320,
    screenY = 240,
})
```

`state` must be an integer from 0 through 3. `distance` must be a finite, nonnegative number. `clamped` must be a Boolean. `screenX` and `screenY` must be finite numbers.

## R88ff VERIFIED

`GetTargetState` publishes one integer-valued number. `GetDistance` publishes one float-valued number. `WasClampedToScreen` returns a stored Boolean. `HasValidScreenPosition` is true exactly when `abs(screenX) >= 2^-22` or `abs(screenY) >= 2^-22`. When `GetNextWaypointForMap` has no waypoint, it returns zero values.
