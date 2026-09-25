# Final unit-frame click and aura proof

A bounded local Forever release run confirms PlayerFrame targeting, Aura tooltip hover, ComboFrame reset, expected panel toggling, and hidden status-bar animation textures. It does not establish native-client conformance or suite-wide green status.

## Source and build boundary

The inspected release was rebuilt from simulator source `a2fd85382`; the supplied later snapshot identifier `21310772e3b9d8b2a203205a78f77b2cbe2ee111aed5329ee25f64fee94a85e6` was described as docs/tests-only. `/tmp/wow-unit-frame-bug/release-build.log` records exit `0` for the Forever GUI release build. The first diagnostic invoked a probe with `@file` incorrectly; it was superseded and is excluded from passing evidence.

## Owned runtime observations

`/tmp/wow-unit-frame-bug/final-gui-observed/observer.json` records PID `1218515`:

- Physical click selected `PlayerFrame`; target changed from `Unknown` to `Uther`, with matching `Player-1-00000001` target GUID.
- `C_ClickBindings` reported `target`; ComboFrame CVar `1` reported `0`; `CharCustomizeFrame` was absent.
- Both XP/reputation flare and level textures remained hidden.
- Panel state was open `true/true`, closed `false/false`, then reopened `true/true`.
- Hook and final error counts were zero.

Task-owner inspection of `final-gui/after-click.png` and `final-gui/panels.png` found the rendered states consistent with those observations.

`/tmp/wow-unit-frame-bug/aura-hover-final/observer.json` records PID `1220915`: hovering Aura ID `1` produced a visible tooltip named `Arcane Intellect`.

## Independent and inherited proof

Independent verifier logs report `cargo fmt --check` and Forever GUI `cargo check` exit `0` with no warnings at source `30971450a`: `/tmp/wow-unit-frame-bug/sim-final-fmt-30971450a-20260925.log` and `/tmp/wow-unit-frame-bug/sim-final-check-30971450a-20260925.log`.

Within the accidentally broad 78-test filtered batch, required click-profile, secure-click, full-click-chain, and ComboFrame cases passed. This is not a suite-green result: 68 passed and 10 distinct tests failed in `/tmp/wow-unit-frame-bug/sim-final-click-30971450a-20260925.log`:

1. `blizzard_ui_blizzard_accountstore::behavior_fullscreen_escape::on_key_down_with_escape_does_not_invoke_leave_store_button_click`
2. `blizzard_ui_blizzard_accountstore::behavior_category_selected::account_store_category_mixin_on_click_triggers_account_store_category_selected_event_with_category_id`
3. `test_keybindings::backpack_button_click_opens_backpack`
4. `test_showuipanel_toggles::character_reputation_tab_click_selects_reputation_panel`
5. `click_targeting::cast_spell_book_item_blocked_while_casting`
6. `workarounds_professions::clicking_blacksmithing_button_in_professions_book_opens_panel`
7. `test_keybindings_panels::premade_group_category_buttons_can_be_clicked`
8. `housing_dashboard::plot_pin_click_selects_plot_and_shows_info`
9. `test_keybindings_spellbook_talents::specialization_escape_after_micro_button_and_tab_click_closes_panel`
10. `housing_dashboard::neighborhood_selector_populates_and_click_loads_selected_map`

The cast-spell-book blocked case expected numeric slot `7` but received a string; it is outside click-profile methods and was not changed in this slice. No failure is labeled baseline or pre-existing here.

Earlier focused evidence remains: aura/combo 2/2, atlas visibility 6/6, bootstrap dependency selection 1/1, and rilua 21/21 plus format/check. The rilua check retains its inherited `strlen` warning.

## Sources

- [click-binding interaction profile](../../specs/click-binding-interaction-profile.md) — click profile contract and focused test scope.
- [script-object environment crossings](../../specs/script-object-environments.md) — private Count and ComboFrame contract.
- [[target-aura-private-count]] — Count root cause and original-boundary regressions.
- [[secret-number-ordering]] — guarded ordering required by the aura layout path.

## See Also

- [[taint-system]] — secure click-binding model.
- [[animated-status-bar-atlas-visibility]] — hidden flare/level texture behavior.
- [[addon-loading]] — bootstrap-only dependency selection.
