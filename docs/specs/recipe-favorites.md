# Recipe favorites

## Scope

Retail 12.1.5 publishes `C_TradeSkillUI.IsRecipeFavorite` and
`SetRecipeFavorite`. The pinned `Blizzard_ProfessionsTemplates/Blizzard_ProfessionsRecipeSchematicForm.lua:460`
uses the getter, and `Blizzard_Professions.lua:843-851` uses the `favorite`
field of recipe information to build the favorites group. The selected Retail
`Blizzard_Professions.toc` loads the templates as a dependency. The generated
`TradeSkillUIDocumentation.lua:1327-1335` declares the favorite-event payload.
The API members themselves are not declared in the pinned generated docs.

PTR 70493 `Blizzard_ProfessionsTemplates/Blizzard_ProfessionsRecipeSchematicForm.lua:469`
consumes `IsRecipeFavorite` during the original crafting-form initialization.
VERIFIED: this PTR consumer needs the API. The registered contract is the same
as Retail WM83, **INFERRED for PTR** pending native R85 evidence; the Retail
native findings below do not establish PTR parity.

## Contract

- Numeric or numerically convertible recipe IDs are truncated to signed
  32-bit integers; invalid arguments and overflow produce the WM83 usage and
  overflow errors. The setter uses Lua truth conversion for its second argument.
- The getter returns one Boolean for a catalog recipe and zero values for an
  unknown recipe. Unlearned status alone does not make a cached recipe unknown.
- The setter returns no values. It changes only a catalog recipe in the active
  matching profession context. Unknown entries, missing context, and unchanged
  writes do not change state or request a notification.
- `IsRecipeFavorite` and `GetRecipeInfo().favorite` read one per-recipe model.
  Initially all favorites are false; this is an **EMULATED** default, not a
  claim about a live character's preferences. The simulator does not persist
  favorites across UI reload or process restart.
- Changed writes queue `TRADE_SKILL_FAVORITES_CHANGED` with `(favorite, recipeID)`.
  Delivery is deferred to a later update tick. Equal pending pairs coalesce;
  opposite values remain separate, in order. The favorites queue is drained
  before the separate recipe-list update marker. A favorite write does not
  directly request a list update; a native rebuild can request one later. The
  simulator does not yet model that rebuild-to-list-marker link, so automatic
  favorite-driven list refresh remains open.
- `HasFavoriteOrderRecipes` also reads this model; see
  [favorite order recipes](profession-tab-state-favorite-orders.md).
  The separate `firstCraft` field is described in
  [recipe first craft](recipe-first-craft.md).
- The update-tick boundary and immediate agreement of the two read APIs are
  simulator choices. Exact native delay and display-cache timing are unknown.
  Network submission, server acceptance and account-sharing behavior are not
  modeled.

## Evidence and tests

WM83 statically traced Retail 12.1.5 build 70688 registration, argument
conversion, local entry update, display projection, and the two separate event
queues. Initial preference and reload persistence were not established.
`tests/workarounds_professions.rs` exercises the selected Blizzard favorites
provider, per-recipe reads and writes, argument conversion, coalescing, event
ordering, unknown IDs and the emulated reload policy. The local catalog has no
unlearned recipe fixture, so that native boundary is not independently tested.
