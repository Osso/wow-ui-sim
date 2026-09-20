# Forever gamepad override CVars

Forever registers `GamepadPossessBarOverride` with default `"4"` and `GamepadStanceBarOverride` with default `"3"` in the native CVar store. Other profiles retain their existing registry.

## Native evidence correction — 2026-09-20

User-provided chat screenshot `pi-clipboard-c51eab0c-c0fc-40a4-8f7c-20ad3bf40d49.png` identifies build `1.60.1`, `69913`, `Sep 17 2026`, interface `16001`. Probe labels report `Possess 4 4` and `Stance 3 3`, in current/default order. This direct native observation supersedes the conflicting dump's possess default `"1"`; stance remains `"3"`. The private screenshot is not committed. `DefaultLayout 0` is an observation only, not evidence of layout-selection policy; no pet-slot base was captured.

The unmodified possess initializer must select `Page1BottomBar` (value `4`, page `1`), not `SpecialPageTopBar` (value `1`, page `4`). Explicit writes to other values remain supported independently of the default.

Earlier source: Ketho/BlizzardInterfaceResources revision `659e8042049df854c114714f8ecd640823a1cd5c`, `Resources/CVars.lua`; README identifies Forever 1.60.1.69913. Capture: `/tmp/forever-wiki-research/`. The source marks these character-scoped, not account-scoped or secure. This change uses existing simulator persistence; it does not add multi-character storage or metadata APIs.

Reads are case-insensitive. Writes update current values without changing defaults and retain `CVAR_UPDATE`. Changed numeric values also emit the documented `GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED` or `GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED` with old/new numeric arguments, as consumed by `GamepadOverrideBarMixin`. Non-numeric writes fail before mutation. Enum range/clamping behavior is not established by this capture.

Unmodified vendor initial positioning must resolve registered defaults to populated override mappings. Tests exercise initial anchor selection, changed anchor selection, defaults preserved after writes, event payloads, and profile publication. Agent-run native execution and full startup are outside this slice.
