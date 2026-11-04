# Forever trait-group currency query

Forever 70707 `C_Traits.GetGroupCurrencyInfo(configID, groupIDs)` selects supplied
configuration nodes and group memberships and returns nested currency records.
WM102 recovers local selection and serialization, not full protected transport,
per-node cost merging, holdings/source policy or concrete accumulator traversal.

## What it must do

- [x] Parse both required inputs before missing-state lookup. Config ID and group-array elements accept native numeric strings, check signed-32 bounds and truncate fractions toward zero. Read the table's Lua array-length segment at integer keys, not its string/hash keys as a group set. Rejected inputs include the native usage body `Usage: local groupCurrencyInfos = C_Traits.GetGroupCurrencyInfo(configID, groupIDs)`; parser/security errors are not empty-result cases.
- [x] Unknown configuration, no known requested group, no positive-rank requested membership, or known groups from different trees produce one empty array through a named **INFERRED public-result seam**. Unknown groups do not invalidate matching known groups, and duplicate requests do not multiply membership.
- [x] Only positive stored ranks create eligible group accumulators. Return them in explicitly supplied bucket/chain order, not request order or numeric sorting. Nested currency rows follow a separately supplied tree-currency provider order.
- [x] Each outer row has traitNodeGroupID and currencyInfos. Each nested row has traitCurrencyID, quantity and spent plus optional maxQuantity and spentInTree. Preserve signed values, absent optionals, empty nested arrays and detached snapshots.
- [x] Native unresolved cost/holdings and traversal are one explicit **UNKNOWN supplied-input boundary**: callers supply already-resolved signed-32 quantities/costs and accumulator order. Missing populated sources raise named diagnostics rather than silently inventing zero costs, holdings or order. No native acquisition, per-rank formula or merge policy is claimed by these snapshots.
- [x] Publish on Forever only. The original Camelot ClassTalents header path must reach the modeled query without a missing-method failure.

## How it works

- [Forever talent state](../wiki/systems/forever-talents.md)

## Implementation inventory

- `src/c_api/trait_group_currency.rs` — required selectors, configuration/rank/tree selection, supplied provider snapshots and serialization.
- `src/c_api/forever_talents.rs` — group-currency input model in the existing Forever character state.
- `src/c_api/registration.rs` — ACTIVE-gated namespace publication.

## Tests asserting this spec

- `tests/forever_trait_group_currency.rs` — two supplied configurations, positive/zero ranks, memberships, duplicate and unknown groups, mixed trees, provider order, negative amounts, optional fields, empty arrays, detached results, input diagnostics and named UNKNOWN boundaries.
- `tests/test_keybindings_spellbook_talents.rs` — unchanged original tab-switch tooltip/no-errors regression.

## Known gaps (current cycle)

- [ ] Native group cost accumulation, quantity source eligibility, config-kind adjustments, concrete bucket/provider ordering and live character population remain unknown. Supplied amounts do not establish any of those policies.
- [ ] Other critical ClassTalents APIs are implemented only if this original path reaches an observed gap; WM102 inventory-only rows are not implementation authority.

## Out of scope

Deriving totals from Retail trait data, economy reconstruction, automatic acquisition, mutations/events and unrelated APIs from the 70-row inventory.

## Source evidence

Pinned Forever `Blizzard_APIDocumentationGenerated/SharedTraitsDocumentation.lua:281-295`,
`:972-980` and `:1188-1200` declare the query and nested records. This cache's
selected `Blizzard_PlayerSpells/Camelot/ClassTalents/Blizzard_ClassTalentsFrame.lua:100`
calls the query (WM102's separate cache reports line 112). WM102's native evidence
is Forever 70707, not Retail 70688 despite its handoff directory name. Empty
public output is inferred from the local collector and serializers across an
unresolved protected region. Snapshot input IDs/amounts are synthetic scenarios,
not native records or live observations.
