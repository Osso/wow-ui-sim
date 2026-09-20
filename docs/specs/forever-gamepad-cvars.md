# Forever gamepad override CVars

Forever registers `GamepadPossessBarOverride` with default `"1"` and `GamepadStanceBarOverride` with default `"3"` in the native CVar store. Other profiles retain their existing registry.

Source: Ketho/BlizzardInterfaceResources revision `659e8042049df854c114714f8ecd640823a1cd5c`, `Resources/CVars.lua`; README identifies Forever 1.60.1.69913. Capture: `/tmp/forever-wiki-research/`. The source marks these character-scoped, not account-scoped or secure. This change uses existing simulator persistence; it does not add multi-character storage or metadata APIs.

Reads are case-insensitive. Writes update current values without changing defaults and retain `CVAR_UPDATE`. Changed numeric values also emit the documented `GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED` or `GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED` with old/new numeric arguments, as consumed by `GamepadOverrideBarMixin`. Non-numeric writes fail before mutation. Enum range/clamping behavior is not established by this capture.

Unmodified vendor initial positioning must resolve registered defaults to populated override mappings. Tests exercise initial anchor selection, changed anchor selection, defaults preserved after writes, event payloads, and profile publication. Native execution and full startup are outside this slice.
