# Item usability

## What it must do

- Retail `C_Item.IsUsableItem(itemInfo)` returns `usable, noMana` in that order. A missing player, zero ID or unowned item returns `(false, false)`.
- For an owned item with a selected use spell, a supplied eligibility rejection returns `(false, false)`. An accepted eligibility outcome evaluates SpellPower costs against the supplied player's power; sufficient power returns `(true, false)`, and insufficient power returns `(false, true)`.
- Eligibility is an explicit item-and-spell input. If it is not supplied for an owned item with a use spell, the simulator reports an error rather than inventing an eligibility result.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1590` declares the ItemInfo input and two boolean returns. WM37-C native evidence for retail 12.1.5/70688 establishes the ordered ownership, eligibility and resource decisions. The result publications are **EMULATED**, not observed on a live client. SpellPower comes from the 12.1.5.70391 CSV; percentage costs use the simulator's existing spell-cost approximation. The eligibility predicate is supplied state, not a modeled native level/class/combat rule.

## Known gaps

- The native eligibility procedure and map/item suppression checks are not modeled; the host supplies their combined result. No selected use spell currently yields `(false, false)` as a simulator approximation.
- Native modified costs, level scaling, optional costs, alternate-power negative-cost limits, ready-rune detail and full aura-dependent selection are not established. Alternate power uses a supplied bar ID; missing power pools supply zero. Full protected-wrapper publication and loading remain unverified.
