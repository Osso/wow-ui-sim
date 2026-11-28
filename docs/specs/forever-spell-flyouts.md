# Forever spell flyout catalog

The pinned Forever 1.60.1.70707 `Blizzard_APIDocumentationGenerated/FlyoutDocumentation.lua:10,27,43,59,92` declares the structured `C_Flyout` queries. They expose the shared simulator spell-flyout catalog with caller-owned result tables.

## Behavior

- `A_Admin.SetSpellFlyouts(rows)` replaces the catalog and its enumeration order. Supply a dense one-based array. Each row has `flyoutID`, `name`, `description`, `isKnown`, and `slots`, another dense one-based array.
- Each slot has unsigned `spellID`, `overrideSpellID`, `isKnown`, optional `name`, and optional `specID`. The IDs are scenario inputs; no native catalog IDs or membership are inferred.
- `GetNumFlyouts()` returns the number of supplied rows. `GetFlyoutID(index)` uses the supplied ordinal and returns no values for an absent ordinal.
- `GetFlyoutInfo(id)` returns a new table with `name`, `description`, `numSlots` and `isKnown`. `GetFlyoutSlotInfo(id,index)` returns a new table with the supplied slot fields, omitting absent optional fields. Unknown rows return no values.
- `FlyoutHasSpell(id,spellID)` checks supplied slot `spellID` membership. Unknown flyouts return no values. The catalog starts empty. Supply `{}` to clear it.
- Invalid field types, duplicate flyout IDs or sparse arrays raise a simulator input error without installing a partial catalog.

Native enumeration order and the treatment of override spell IDs in `FlyoutHasSpell` are not verified. The simulator reads the explicitly supplied order and spell IDs; it does not infer spell overrides from the catalog. This does not implement native texture lookup or flyout action selection.

## Verification

`tests/forever_flyout_catalog.rs` checks counts, distinct IDs, fresh structured projections, optional fields and positive/negative membership. No pinned Forever Lua caller references these five methods; the tests use the API directly.
