# Input interface style test input

Forever tests can set the current input-interface style through A_Admin:

```lua
A_Admin.SetInputInterfaceStyle(Enum.InputDeviceInterfaceType.Mkb)
A_Admin.SetInputInterfaceStyle(Enum.InputDeviceInterfaceType.Gamepad)
```

The function accepts only `Mkb` (0) or `Gamepad` (1). It updates the environment's input-interface style through the normal transition path, which queues `INPUT_DEVICE_INTERFACE_TRANSITION` when the value changes. It is simulator-only; it does not detect or control physical input devices.
