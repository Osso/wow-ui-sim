# Tooltip owner lifecycle

`GameTooltip` ownership is separate from its lines and visibility. The shared visibility model releases ownership on an explicit tooltip hide, while `ClearLines` preserves the owner. This is documented-source corroboration, **not native execution**: [GameTooltip API reference](https://warcraft.wiki.gg/wiki/UIOBJECT_GameTooltip) states `Region:Hide` clears the owner and recommends `ClearLines` when retaining owner/anchor.

## What it must do

- [x] `Hide` and `SetShown(false)` on a shown, owned tooltip hide it, release `GetOwner`/`IsOwned` ownership and keep both simulator owner fields synchronized without clearing lines.
- [x] `ClearLines` empties lines while retaining ownership.
- [x] `FadeOut` still hides and releases the owner; hiding a normal frame does not release another tooltip's owner.

## How it works

- [Lua API architecture](../lua-api.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/core_state/visibility.rs` — shared explicit visibility transitions and tooltip owner release.
- `src/lua_api/frame/methods/widgets/tooltip/owner.rs` — tooltip ownership and `FadeOut` delegation.

## Tests asserting this spec

- `tests/tooltip_basic.rs` — Hide, SetShown(false), ClearLines, FadeOut and ordinary-frame controls.

## Known gaps (current cycle)

- [ ] Native execution and callback observation order remain unverified.

## Out of scope

- Parent-effective hiding, automatic line clearing, SetOwner visibility, SetText, empty Show, and profile conversion are not changed by this slice.
