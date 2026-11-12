# Covenant renown rewards

`C_CovenantSanctumUI.GetRenownRewardsForLevel(covenantID, renownLevel)` returns covenant reward rows. The temporary fallback supplies no Retail catalog. See [Covenant renown reward model](../wiki/systems/covenant-renown-rewards.md) for implementation details.

## What it must do

- [x] When Retail has no supplied reward catalog, return an empty table. Do not invent a reward for a covenant and level pair.
- [ ] Preserve the existing non-Retail simulator reward scenario. This is simulator compatibility, not a native-parity claim.
- [x] The selected Retail `Blizzard_CovenantToasts` tooltip consumer must take its no-row path without querying reward-item details.

## How it works

- [C API boundary](../wiki/index.md)
- [Covenant renown reward model](../wiki/systems/covenant-renown-rewards.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/campaign_covenant_defaults.rs` — profile-specific fallback while no Covenant reward catalog is modeled.
- `tests/covenant_sanctum_ui.rs` — non-Retail scenario coverage.
- `tests/blizzard_covenant_toasts_loads.rs` — Retail original-consumer coverage.

## Tests asserting this spec

- `src/lua_api/workarounds/temporary/campaign_covenant_defaults.rs::tests::retail_missing_renown_catalog_returns_no_reward_rows`
- `src/lua_api/workarounds/temporary/campaign_covenant_defaults.rs::tests::non_retail_keeps_previous_renown_reward_scenario`
- `tests/covenant_sanctum_ui.rs::non_retail_covenant_sanctum_get_renown_rewards_returns_seeded_scenario`
- `tests/blizzard_covenant_toasts_loads.rs::retail_renown_tooltip_does_not_invent_the_path_of_ascension_reward`

## Known gaps (current cycle)

- [ ] No Covenant RenownRewards catalog or player-condition filter is modeled.
- [ ] The empty-list fallback is not a claim that every loaded Retail client returns no rewards.
- [ ] The direct empty-sequence unit test and the non-Retail scenario test remain unrun in this cycle.

## Out of scope

- Populated reward records, eligibility-condition evaluation, transmog-set filtering, reward events, and parity beyond the inspected Retail 12.1.5 build.
- `C_MajorFactions.GetRenownRewardsForLevel`, which is a separate API and data model.
