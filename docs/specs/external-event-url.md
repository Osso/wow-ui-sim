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

- [ ] On Retail, `HasURL()` returns one boolean and is true only when the URL
  text is nonempty.
- [ ] On Retail, `IsNew()` returns one boolean by comparing the current URL
  text with the saved last-launched URL text.
- [ ] On Retail, `LaunchURL()` with an empty URL logs
  `No external event URL to launch.`, emits `EXTERNAL_EVENT_LAUNCH_URL_FAILED`
  synchronously with no payload, and returns no values.
- `LaunchURL()` with a supplied URL records it in `SimState.last_launched_url`
  as simulator policy; no browser opens.
- PTR `CanShow()` returns one boolean from the supplied visibility flag. The
  other four profiles do not register it because their pins do not declare it.
- Non-Retail `IsNew()` keeps the existing supplied newness state; R44-Q13 does
  not establish other-profile parity.
- The simulator scenario starts with no URL and false PTR visibility. These
  values are not evidence of native defaults.

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

- Retail URL writers, initial values, nonempty launch success, and restricted-
  action early-exit inputs are unknown. PTR `EXTERNAL_EVENT_LAUNCH_URL_RESULT`
  behavior is also unknown.
- URL persistence and the relation between PTR `CanShow` and `HasURL` are not
  modeled; they remain separate inputs.

## Tests asserting this spec

- `tests/external_event_url.rs` — the GameMenu click consumer launches only a
  supplied URL, the Retail empty-URL failure reaches the original popup handler
  synchronously, `IsNew` follows the supplied flag on non-Retail profiles, and
  the PTR minimap consumer uses `can_show`.
