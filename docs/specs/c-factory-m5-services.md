# M5 housing, pets, bank and transmog startup inputs

M5 restores individually required startup inputs after [factory removal](c-namespace-nil-members.md). [Implementation notes](../wiki/systems/c-factory-startup-followup.md#m5-service-and-catalog-inputs) separate catalog-backed queries from named temporary service gaps.

## What it must do

- [x] Retail `C_Housing.GetPlayerOwnedHouses()` queues one `PLAYER_HOUSE_LIST_UPDATED(houseInfos)` reply per call for a later timer tick; the host supplies the documented HouseInfo rows. Timing is **INFERRED (R88fj; native evidence pending)**. PTR keeps its existing temporary empty-list request bridge. Forever does not load the consumer because its TOC game-type filter excludes `standard`.
- [x] Retail/PTR/Mists/Forever original pet action buttons remain disabled when swap/skip legality is unavailable. Named temporary predicates return false, not eligibility inferred from health alone.
- [x] Those profiles' pet-icon query joins the supplied battle species to pet-journal icon metadata; clearing the supplied pet record produces no icon.
- [x] Forever purchased bank-tab and player-bag capability queries return the documented table/boolean shapes through named temporary workarounds. Their fixed empty/false implementation choices are not native bank policy.
- [x] Forever `C_TransmogSets.HasAnyValidSets()` reads supplied catalog `valid_for_character` flags independently of UI filters. Other profile API publication is unchanged.

## How it works

- [Service/capability defaults and catalog projections](../wiki/systems/c-factory-startup-followup.md#m5-service-and-catalog-inputs)

## Implementation inventory

- `src/c_api/c_pet_battles.rs` — species/catalog icon projection.
- `src/c_api/c_transmog_sets{.rs,/catalog.rs}` — Forever valid-set query and existing catalog.
- `src/c_api/{mod.rs,registration.rs}` and `src/lua_api/{state.rs,state/sim_state.rs}` — share catalog storage without publishing unrelated profile members.
- `src/c_api/c_housing/owned_houses.rs` — Retail supplied owned-house records and queued responses.
- `src/lua_api/workarounds/temporary/{housing_owned_house_requests,pet_battle_action_legality_defaults,forever_bank_capability_defaults}.rs` — PTR's temporary owned-house response plus missing battle-legality and bank metadata/capability systems.

## Tests asserting this spec

- `tests/cfactory_m5_services.rs` — complete original pet UI chunks, generated icon/housing/bank/set contracts and asynchronous ownership response.

## Known gaps (current cycle)

[Bounded proof and residuals](../wiki/systems/c-factory-startup-followup.md#bounded-proof--2026-10-21) include a supplied validity flip independent of UI filters. New CanPurchaseBankTab and pet GetName cascades remain held outside the frozen F0 list.

## Out of scope

Wrath behavior; authoritative ownership, swap locks/turn legality, purchased bank-tab metadata and bank-type capability policies. No invented sample pets/houses/tabs, vendor changes or arbitrary member fabrication.
