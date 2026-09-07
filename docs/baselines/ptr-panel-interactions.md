# PTR 12.1.5 panel interactions

Verified runtime source: pinned `wowxptr` build `12.1.5.69594`, interface `120105`. Source provenance and the 4,025-file index are documented in [PTR source pinning](../specs/ptr-blizzard-ui-source.md).

## Runtime coverage

After an explicit PTR build at `9d96dbba0`, `wow-sim --no-addons --no-saved-vars lua-errors` with a build/bootstrap assertion reported `12.1.5 / 69594 / 120105`, `InClickBindingMode` as a function while ClickBinding remained `loaded=false, finished=false`, and an empty error array.

The combined post-startup Lua sequence opened and closed these panels through their normal helpers:

| Panel | Operation | Result |
|---|---|---|
| Character | PaperDoll tab via `ToggleCharacter`, then `HideUIPanel` | Visible/closed assertions pass |
| Spellbook | `PlayerSpellsUtil.ToggleSpellBookFrame` twice | Visible/closed assertions pass |
| Talents | `PlayerSpellsUtil.ToggleClassTalentFrame` twice | Visible/closed assertions pass |
| World Map | `ToggleWorldMap` twice, positive map ID | Visible/closed assertions pass |
| Settings | `SettingsPanel:Open`, `Close(true)` | Closed without opening GameMenu |
| Collections | Mount tab via `ToggleCollectionsJournal` twice | Mount tab visible, then closed |

All six transitions passed in one process; the trailing Lua error array was `[]`. Exit status alone is not this evidence: assertions, identity, and the error array were inspected. Runs were bounded by `timeout 90`, compilation was separate.

Artifacts: `/tmp/pi-ptr125-metadata-{build,startup,panels}.{stdout,stderr}.log`; script `/tmp/pi-ptr125-panels-smoke.lua`. The earlier six ClassNameplate startup errors and Spellbook ClickBinding errors no longer occur in this sequence. [The committed startup baseline](ptr-lua-errors.json) is empty.

## Final bounded verification

`/tmp/pi-ptr125-final-acceptance.md` records acceptance at `53b741bc32f053df31eba14db9f78ee589de6416`: 27 selected PTR integration cases and five PTR binary cases passed; default coverage is 24 selected integration cases and four binary cases, with one ignored dump helper. `cargo check` and `cargo fmt --check` completed with zero warnings. The default snapshot exact test was rerun after its metadata-driven order correction.

## Limits

This is headless runtime interaction coverage, not visual parity or proof that every panel/API works. Personal addons and SavedVariables were disabled for these runs; existing simulator EditMode cache selection remains part of startup. GetBuildInfo date/trailing slots remain compatibility defaults. Native rounding has separate [captured geometry coverage](../specs/pixel-layout-rounding.md). Private-table identity and SavedVariables timing across bootstrap/full loads remain unprobed. This bounded scope does not execute classic profiles, the full project suite, or establish full API coverage.
