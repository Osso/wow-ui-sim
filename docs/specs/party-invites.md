# Party invites

`C_PartyInfo.InviteUnit(targetName)` attempts a party invitation. Retail and Forever declare the function in their cached `PartyInfoDocumentation.lua` files.

## What it must do

- [x] Publish `C_PartyInfo.InviteUnit` on Retail and Forever with one required `cstring` argument and no return values.
- [x] **INFERRED (simulator mapping):** do not change local group state, fire events, or show popups; group membership changes only after server acceptance.
- [ ] Native confirmation, event, and taint behavior remain UNKNOWN pending R88bl.

## How it works

- [Party API](../admin-api/party.md).
- [Party conversion](party-conversion.md).

## Implementation inventory

- `src/c_api/c_party_info.rs` — profile-gated registration and server-mediated no-local-mutation mapping.

## Tests asserting this spec

- `tests/party_invite.rs` — required argument, zero return values, unchanged group state, no invite-related events or popups.
- AllTheThings wrapper probe — confirms the original wrapper finds `C_PartyInfo.InviteUnit` on Retail.

## Known gaps (current cycle)

- [ ] Replace the inferred no-local-mutation mapping when R88bl provides native evidence.

## Out of scope

Party request transport, confirmation UI, event emission, and taint behavior; native behavior remains under investigation.
