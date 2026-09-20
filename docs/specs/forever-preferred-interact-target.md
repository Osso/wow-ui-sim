# Forever preferred gamepad interaction target

Forever exposes `SetPreferredGamepadInteractTarget(unitOrNil)` as a modeled global. Its nullable `UnitToken` argument and no-return contract come from build 1.60.1.69913 `Blizzard_APIDocumentationGenerated/TargetScriptDocumentation.lua`.

The setter resolves existing unit tokens, including `softinteract`, and stores the selected unit GUID independently of target/focus aliases. Nil clears the preference. An unresolved token clears it under simulator policy; subsequent alias changes do not silently select another identity. It does not change target/focus, mutate unit interactions, or emit an invented event. Vendor `UpdateInteractIcons` emits its own `Gamepad.PreferredGamepadInteractTargetChanged` notification.

Only Forever registers this new global. Native restricted/secret argument enforcement and display/interaction consumption of the stored preference are not added by this slice. No undocumented Lua getter is introduced; host model inspection establishes selection identity.

Behavioral tests exercise the actual cached `MainActionBarFrame.lua` consumer selecting a lootable target, clearing when core bindings are inactive or the target disappears, and direct nil/soft-interaction selection without retargeting.
