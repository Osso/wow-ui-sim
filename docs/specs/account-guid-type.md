# Battle.net account GUID type

`C_AccountInfo.IsGUIDBattleNetAccountType` classifies a GUID for the Retail, PTR, and Forever `PlayerLocation` consumer. The cached Forever `AccountInfoDocumentation.lua:1–40` declares `Environment = "All"`, the WOWGUID argument, and boolean result. Native evidence WM80 describes the permissive 70688 BNetAccount parser and argument policy.

## What it must do

- [ ] Require one string or number, ignore extra arguments, and reject missing, nil, and other types with the WM80 usage reason.
- [ ] Return true for the BNetAccount GUID kind and false for other kinds, independently of friends or presence. The BNet prefix is ASCII case-insensitive; its decimal part consumes at most 20 digits with wrapping arithmetic and must fit 58 bits. Partial payloads are accepted.
- [ ] Reject secret input from tainted callers when secret access is restricted; untainted callers may use accessible secret input.
- [ ] Let the original PlayerLocation `IsValid` consumer evaluate the account predicate for both BNet and non-BNet GUIDs.
- [ ] `GetIDFromBattleNetAccountGUID` returns the low 32 bits of the parsed hexadecimal field, or one nil for a non-BNet GUID (ordinary returns INFERRED by WM80b).
- [ ] `IsGUIDRelatedToLocalAccount` compares the full local BNet GUID when a local identity is modeled; with no local BNet identity it returns false. For other GUIDs, comparison to the current player is INFERRED by WM80b.

## How it works

- [C API and Lua boundary](../lua-api.md)

## Implementation inventory

- `src/c_api/c_account_info.rs` — namespace registration and BNet kind classification.
- `src/c_api/registration.rs` — namespace registration during environment construction.

## Tests asserting this spec

- `tests/guid_is_player.rs` — permissive inputs and the unmodified PlayerLocation consumer.

## Known gaps (current cycle)

- [ ] Local Battle.net identity is not modeled; `BNGetInfo` is an inert workaround, not a state source. BNet relatedness is false while that state is absent.
- [ ] Non-BNet relatedness compares the simulator's exact current-player GUID string; full native GUID normalization is not modeled.
- [ ] Native opaque GUID aliases can map to BNetAccount GUIDs; the simulator does not model the opaque map.
- [ ] Native output secrecy and locale-dependent prefix behavior are not fully established.

## Out of scope

Friend lists, account existence, online state, and live Battle.net service behavior do not determine this type predicate.
