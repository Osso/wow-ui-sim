# Action-slot text

`C_ActionBar.UsesActionText(slot)` and `C_ActionBar.GetActionText(slot)` read an occupied macro action's current name. Real queries live in `src/c_api/action_macros.rs`; see [Lua API architecture](../lua-api.md). The retained 12.0.5 API source, `/tmp/patch-12.0.5-api-source-plain.txt:170`, says: “Added a new C_ActionBar.UsesActionText API and switched to using it inside ActionBarActionButtonMixin:Update.” That establishes occurrence and intended consumer, not qualifying action types or name rules.

## What it must do

All following label rules are **best-supported simulator inferences, not native-verified semantics**.

- [ ] An action associated with an existing macro with a nonempty name returns one plain, non-secret boolean `true` and one text result equal to the exact stored name, preserving whitespace. Macro body and `#showtooltip` do not select label usage.
- [ ] Rename/body edits are visible without reassignment. An empty name marks an unused macro entry: `false` and nil; restoring a name restores the associated label.
- [ ] Cursor pickup/place and `C_ActionBar.PutActionInSlot` follow current slot assignments. Moving, clearing, deleting or replacing with a spell clears obsolete labels; reuse of a deleted macro ID does not restore old assignments.
- [ ] Empty and ordinary spell slots return `false` and nil. An item cursor rejected by the existing placement model leaves an empty destination returning `false` and nil; this is not native item-action-slot coverage.
- [ ] Queries retain existing cross-profile registration, without a new retail patch gate.

## How it works

- [Lua API and state](../lua-api.md)
- [Existing macro association lifecycle](macro-action-showtooltip.md)

## Implementation inventory

- `src/c_api/action_macros.rs`: shared current macro-name lookup, boolean/text queries and existing registration.
- `src/lua_api/globals/action_bar_api.rs` and `action_bar_api/registration.rs`: obsolete placeholders removed; existing registration hook retained.
- Existing `src/lua_api/globals/spell_macro_verbs.rs` and `inventory_verbs.rs`: macro storage edits and public cursor placement, unchanged.

## Tests asserting this spec

`tests/action_text.rs`, automatically grouped in the existing `integration` target: public create/pickup/place, rename/body edits, empty name, move/transfer, spell replacement, deletion/reuse, clear, scalar result count/type/secrecy and empty/spell/unplaceable-item controls. Tests are unguarded; only current default retail compilation is exercised in this bounded implementation.

## Known gaps (current cycle)

- [ ] Native probe: on an identified 12.0.5 client build, create account and character macros named `LabelProbe`, a single space, and a leading/trailing-space name, with `/say`, empty, and `#showtooltip` bodies. Place each alongside a known spell, usable bag item and empty slot. Save `(GetActionInfo, UsesActionText, type, issecretvalue, GetActionText)` per slot before/after rename, pickup/place movement, replacement and deletion. Record build and exact names/body strings; compare boolean qualification and text bytes. Reject unsupported empty-name edits explicitly rather than inferring native acceptance.
- [ ] Real item action slots remain unmodeled, so their inferred false qualification is not exercised by a populated-item fixture.

## Out of scope

Adding item-slot storage, other action-kind redesign, macro condition evaluation, secret text/security enforcement, invalid-slot/coercion rules, notifications/rendering, native probes and cross-profile execution proof. No Blizzard/vendor edits.
