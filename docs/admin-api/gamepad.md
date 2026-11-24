# Gamepad Input

The gamepad cursor-control input is modeled on Retail and WoW Forever. Its behavior is **INFERRED (R88fo; native evidence pending)**.

## A_Admin.SetUsingGamepad(enabled)

Sets the scenario's input-device state through the existing `IsUsingGamepad()` model. Retail supplies a recent gamepad-input timestamp. WoW Forever supplies `C_InputInterfaceStyle` state. This is a simulator test input, not a native WoW API.

- **enabled** `boolean` -- Whether the scenario reports gamepad input as active
- **Affects:** `IsUsingGamepad()`, and therefore automatic cursor-control decisions

## A_Admin.GetGamePadCursorControlEnabled()

Returns the simulator's gamepad cursor-control enabled state.

- **Returns:** `boolean`
- **Affects:** observes changes made by `SetGamePadCursorControl(enabled)`

## Automatic control

`CanAutoSetGamePadCursorControl(enabled)` returns true when the existing `IsUsingGamepad()` model reports an active gamepad. The `GamePadCursorAutoEnable` CVar gates automatic enabling; automatic disabling remains available. GameMenu open and close handlers use this result before they call `SetGamePadCursorControl`.
