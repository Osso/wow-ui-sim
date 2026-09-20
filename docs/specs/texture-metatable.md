# Texture metatable access

## Contract

- Forever exposes `GetTextureMetatable()` before addon loading.
- It returns the actual per-type metatable obtained from a Texture created through the existing frame factory, not a fabricated method table.
- Like the existing FontString helper, an optional Texture argument selects that object's metatable.
- Repeated calls and calls with different ordinary textures return the same per-type metatable. Its `__index` methods operate on real Texture objects.
- Other profiles retain their prior global surface.

## Consumer and proof

Forever 1.60.1.69913 `Blizzard_UnitFrame/Mainline/UnitFrameUtil.lua:67` copies `GetTextureMetatable().__index` and uses its `IsObjectType` method. `tests/wowforever_texture_metatable.rs` loads that unmodified source and exercises metatable identity, type queries, and texture mutation through the real methods.

This preserves simulator metatable-helper behavior; optional-argument validation and native metatable mutation/security semantics are not established by this test.
