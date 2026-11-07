# Tooltip owner lifecycle

## Profile scope

The historical contract below records legacy-profile behavior, with Wrath
simulator proof. Retail/PTR follow the newer WM63/WM95 behavior instead:
`Hide` clears rows and releases ownership; same-owner `SetOwner` clears
without an explicit hide, while changing owners hides first. A non-table
owner is rejected before mutation. Rejected empty/ownerless `Show` also runs
hiding. See the [current timing and admission contract](tooltip-layout-timing.md).
Native execution, callback re-entry and owner-auto-hide remain unverified.

## Historical legacy-profile contract

`GameTooltip` ownership is separate from its lines and visibility. Explicit tooltip hiding releases ownership; `SetOwner` hides and clears content while retaining its newly assigned owner. This is documented-source corroboration, **not native execution**: [GameTooltip API reference](https://warcraft.wiki.gg/wiki/UIOBJECT_GameTooltip) describes owner, text, and append visibility behavior.

## Current Retail/PTR lifecycle

The WM63 O17 Retail 12.1.5 / 70688 owner/hide contract supersedes the legacy
requirements below for Retail/PTR. Forever shares only the owner reassignment
visibility lifecycle described below. Shared Lua consumers do not establish
native anchor geometry, default anchors, or Hide-clears-lines behavior.

- `SetOwner` clears content for the same or a different owner. Same-owner
  assignment does not hide; reassignment hides through the ordinary lifecycle
  before recording the new owner. Forever also uses this visibility lifecycle
  (**INFERRED**, not native Forever proof), but keeps its existing anchor modes.
  Retail/PTR anchor modes follow the WM63 path.
- `Hide` clears content even when already hidden, releases the owner and resets
  its mode/displacements, without removing its existing points. This requirement
  applies only to Retail/PTR. Forever retains lines and its anchor mode on Hide.
- Clear notification (WM63 O17 §7, Retail 12.1.5 / 70688): a clear dispatches
  its notification only when the prior line count was nonzero, after the
  per-line resets and before a new owner is recorded. Native `VERIFIED` order;
  the public script name stays inferred, so tests assert dispatch count and
  order rather than the name. The clear also empties and hides the used pooled
  line FontStrings and keeps their allocation for reuse.
- The pinned `Blizzard_GameTooltip/Mainline/GameTooltip.xml` binds that
  notification with an `<OnTooltipCleared>` element to `GameTooltip_ClearMoney`
  and `SharedTooltip_ClearInsertedFrames`. XML tooltip script elements
  (`OnTooltipCleared`, `OnTooltipAddMoney`, `OnTooltipSetDefaultAnchor`,
  `OnTooltipSetFramestack`, `OnTooltipSetQuest`, `OnTooltipSetSpell`,
  `OnTooltipSetItem`, `OnTooltipSetUnit`) therefore bind for a GameTooltip,
  using the same name set `SetScript` accepts (cached consumer, build
  unverified for the exact element set beyond those pinned declarations).
- Forever's original `Blizzard_GameTooltip/Mainline/GameTooltip.xml:27` binds
  `OnHide`; `GameTooltip.lua:395-431` clears handler information.
  `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:264-270` hides on a
  missing non-appended lookup, and `:353-355` clears `infoList`/`processingInfo`.
  Silently hiding during same-owner `SetOwner` bypasses that later cleanup and
  retains a previous successful tooltip's data.

The `tooltip_bag_child` original-consumer regression stays unchanged. It rejects
an unmatched equip slot after two successful distinct item selections; no
fixture `Hide` or replacement consumer masks the lifecycle failure. Sizing,
attachments, other profiles and getter selection are unchanged by this extension.

## Legacy requirements (other profiles, including Forever anchors and Hide)

- [x] `Hide` and `SetShown(false)` on a shown, owned tooltip hide it, release `GetOwner`/`IsOwned` ownership and keep both simulator owner fields synchronized without clearing lines.
- [x] Explicit `Hide`, `SetShown(false)`, and `FadeOut` release ownership even if `SetOwner` already hid the tooltip; public protected-state checks still apply.
- [x] `SetOwner` clears lines and hides without discarding the newly assigned owner or changing its anchor. Forever instead keeps same-owner visibility, as specified above.
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
- `tests/tooltip_lifecycle_owner_lane.rs` — same-owner clear, notification count, and reassignment through the shared tooltip templates.
- `tests/tooltip_owner_clear_consumer.rs` — clear notification count and order through the cached `GameTooltip.xml` handlers and `TooltipDataHandler.lua` reprocessing; the original `OnTooltipCleared` cleanup must release inserted frames; Forever owner-reassignment lifecycle.

## Known gaps (current cycle)

- [ ] Native execution and callback observation order remain unverified: the
      evidence is static native analysis, and no live-client observation exists
      for the notification's registered-callback lifetime.
- [ ] Forever does not reset its pooled line FontStrings on clear. Its line
      FontStrings come from tooltip XML and are recorded for clear-time reset
      only on Wrath/Retail/PTR, so no Forever native evidence covers the reset.

## Out of scope

- Parent-effective hiding, empty Show, profile conversion, and line-registration changes are not changed by this slice.
