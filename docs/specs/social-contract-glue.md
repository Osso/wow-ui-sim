# Social contract (glue)

`C_SocialContractGlue` lets CharacterSelect ask whether to show the social contract and record acceptance.

## Requirements

- [x] Published on every profile except Wrath (consumers: `Blizzard_GlueXML/SocialContract.lua` and CharacterSelect).
- [x] `GetShouldShowSocialContract()` returns nothing and fires `SOCIAL_CONTRACT_STATUS_UPDATE(shouldShow)`, where `shouldShow` is `not SimState.social_contract_accepted`.
- [x] `TryUpdateSocialContract()` sets `social_contract_accepted = true`.
- [x] Default `social_contract_accepted = true` (established account).

## Best-guess basis

best-guess, native UNKNOWN. The CharacterSelect handler reads the answer only from `SOCIAL_CONTRACT_STATUS_UPDATE`, so the query is a request and the event is the response. Native delivery is asynchronous (server round-trip); the simulator fires it synchronously. The contract is shown once per account, so an account that accepted it is not asked again.

## Known gaps

- Server rejection of `TryUpdateSocialContract` is not modeled.
- Acceptance does not persist across simulator sessions.

## Tests

- `blizzard_glue_xml_loads::character_select_social_contract_follows_account_acceptance`.
