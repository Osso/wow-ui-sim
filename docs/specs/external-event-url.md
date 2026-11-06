# External event URL state

`C_ExternalEventURL.HasURL`, `IsNew` and `LaunchURL` read host-supplied
promotion state. The pinned
`Blizzard_APIDocumentationGenerated/ExternalEventURLDocumentation.lua:5` declares
the namespace on Retail, PTR, Mists, Era and Anniversary; `HasURL` is at `:11`,
`IsNew` at `:20` and `LaunchURL` at `:29` in each of those pins. PTR also
declares `CanShow` at `:11`. Wrath publishes no such documentation file, so the
namespace stays absent there. WowForever keeps its own modeled namespace from
[Forever startup inputs](forever-startup-inputs.md).

This replaces the temporary `external_event_url_default` workaround, which
returned `false`/`false`/no-op from Lua for the same five profiles.

## Contract

- `HasURL()` returns exactly one non-nil boolean. It is true only when the host
  supplied a promotion URL.
- `IsNew()` returns exactly one non-nil boolean from the supplied newness flag.
  Consumers use it only after `HasURL()`, so a newness flag without a URL has no
  consumer-visible effect.
- `LaunchURL()` returns no values. With a supplied URL it records that URL in
  `SimState.last_launched_url`, the same simulator modeling as the `LaunchURL`
  global; without one it records nothing. No browser opens.
- PTR `CanShow()` returns exactly one non-nil boolean from the supplied
  visibility flag. The other four profiles do not register it because their
  pins do not declare it.
- The simulator scenario starts with no URL, `is_new` false and `can_show`
  false. This preserves the previous scenario behavior; it is not evidence of a
  native default.

## Consumers and runtime selection

- Retail, Mists, Era and Anniversary: `Blizzard_GameMenu.toc` permits the Game
  screen and selects `Shared/GameMenuFrame.lua`.
  `GameMenuFrameMixin:ExternalEventClickCallback` calls `LaunchURL` only when
  `HasURL` is true, and `GameMenuFrameMixin:InitButtons` reads `IsNew` for the
  "new" badge.
- PTR: `Blizzard_Minimap.toc` permits the Game screen and selects
  `Shared/ExternalEventURLMinimapButton.lua`.
  `ExternalEventURLMinimapButtonMixin:Refresh` reads `CanShow`, `HasURL` and
  `IsNew`; `OnClick` calls `LaunchURL`.

## Known gaps

- Native writers, native initial values and launch success or failure are
  unknown. `EXTERNAL_EVENT_LAUNCH_URL_FAILED` (Retail) and
  `EXTERNAL_EVENT_LAUNCH_URL_RESULT` (PTR) are not emitted.
- URL validation, persistence and the relation between `CanShow` and `HasURL`
  are not modeled; both are independent host inputs.

## Tests asserting this spec

- `tests/external_event_url.rs` — the GameMenu click consumer launches only a
  supplied URL, `IsNew` follows the supplied flag, and the PTR minimap consumer
  hides or shows the promotion from `can_show`.
