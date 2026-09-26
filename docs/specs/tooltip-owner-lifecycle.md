# Tooltip owner lifecycle

`GameTooltip` ownership is separate from its lines and visibility. Explicit tooltip hiding releases ownership; `SetOwner` hides and clears content while retaining its newly assigned owner. This is documented-source corroboration, **not native execution**: [GameTooltip API reference](https://warcraft.wiki.gg/wiki/UIOBJECT_GameTooltip) describes owner, text, and append visibility behavior.

## What it must do

- [x] `Hide` and `SetShown(false)` on a shown, owned tooltip hide it, release `GetOwner`/`IsOwned` ownership and keep both simulator owner fields synchronized without clearing lines.
- [x] Explicit `Hide`, `SetShown(false)`, and `FadeOut` release ownership even if `SetOwner` already hid the tooltip; public protected-state checks still apply.
- [x] `SetOwner` clears lines and hides without discarding the newly assigned owner or changing its anchor.
- [x] Nonempty `SetText` shows an owned tooltip with replacement text; `AddLine` and `AddDoubleLine` append without showing, until `Show` is called.
- [x] Spell tooltip payloads still show after `SetOwner`; `ClearLines` empties lines while retaining ownership.
- [x] Hiding a normal frame does not release another tooltip's owner.

## How it works

- [Lua API architecture](../lua-api.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/core_state/visibility.rs` — shared explicit visibility transitions and tooltip owner release.
- `src/lua_api/frame/methods/widgets/tooltip/owner.rs` — tooltip ownership, internal hiding, and `FadeOut` delegation.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — owned tooltip replacement text visibility.
- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs` and `content.rs` — append-without-show and payload-driven visibility.

## Tests asserting this spec

- `tests/tooltip_basic.rs` — owner replacement, hidden explicit hide, SetText, append, spell payload, and existing ownership controls.

## Known gaps (current cycle)

- [ ] Native execution and callback observation order remain unverified.

## Out of scope

- Parent-effective hiding, empty Show, profile conversion, line-registration changes, and `OnTooltipCleared` callback ordering are not changed by this slice.
