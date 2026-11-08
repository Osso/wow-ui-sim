# Mists spellbook button size

The Mists spellbook's populated buttons collapsed because XML duplicate removal used indentation as a parent identity. The pinned `Blizzard_UIPanels_Game/Mists/SpellBookFrame.xml` declares a 37×37 `SpellButtonTemplate` size at line 102 and a nested 64×64 `NormalTexture` size at line 212. Both use two tabs of indentation. The old preprocessing pass treated them as duplicates and removed the button size.

`crates/sim-xml/src/parse.rs` now tracks XML parent elements when removing duplicate self-closing `Size` and `TexCoords` elements. It retains the last duplicate within one parent without removing a sibling or nested parent's element. A parser regression covers equal indentation; the original Mists spellbook consumer covers the runtime boundary.

The original panel test also encountered two stale expectations after the size fix: Blizzard texture assets can be returned by the simulator as numeric file IDs, and the pinned Mists `WHAT_HAS_CHANGED` string is `What has changed?`. The test accepts an asset ID or nonempty path and compares the tab with that localized string. These are simulator observations, not claims about undocumented native behavior.

## Sources

- [XML parser](../../../crates/sim-xml/src/parse.rs)
- [Mists spellbook integration test](../../../tests/mists_spellbook_panel.rs)
- [Mists global strings](../../../data/db2/mists-5.5.4.70268/GlobalStrings.csv)

## See Also

- [XML template system](../../xml-template-system.md)
