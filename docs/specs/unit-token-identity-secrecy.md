# UnitTokenFromGUID identity-secret party suppression

Bounded model of [retained 12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 48 and 84: UnitTokenFromGUID no longer returns listed token categories when identities are secret. This slice covers **party tokens only**, using existing party identities and explicit host classification. Both prose rows remain partial; unnamed APIs and other listed token categories receive no credit. [UnitName secret inputs](unit-name-secret-tokens.md) are a separate contract.

## What it must do

- [ ] Resolve two existing active party identities normally while classification is empty; marking the first identity secret suppresses only its party token, leaving the second public identity usable.
- [ ] **INFERRED:** suppression returns one public nil, preserving the existing missing-lookup return shape. Host removal of classification restores the same mapping.
- [ ] **INFERRED:** secure and tainted callers receive the same suppression without clearing or changing caller taint. Public GUID input does not imply public identity; caller taint is not identity secrecy.
- [ ] Never create a token for an unknown GUID or inactive roster. **INFERRED:** roster activation does not change host-owned classification.
- [ ] **INFERRED:** classification starts empty in each environment, remains environment-local, and does not alter the existing player/target/focus alias resolution priority. The player control remains resolvable even when its GUID is marked.

## How it works

- [Lua API architecture](../lua-api.md).
- [Retail secret-value contract](retail-secret-values.md).
- [Unit identity comparison permissions](unit-identity-equality.md).

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: proposed public host-owned `identity_secret_guids: HashSet<String>`, gated to Retail 12.0.5 / PTR.
- `src/lua_api/state.rs`: proposed empty-default initializer in the shared construction macro.
- `src/lua_api/globals/unit_misc.rs`: proposed party reverse-lookup guard; existing player/target/focus producers and current argument conversion stay unchanged.
- Exact edits are authored in the task's `handoff-identity-tokens.md`, not applied in this slice. No new registration, namespace, helper module or addon setter.

## Tests asserting this spec

- `tests/unit_token_identity_secrecy.rs`: five grouped API cases covering actual seeded party identities, live host classification/removal, one-nil arity, caller taint, inactive/unknown controls, player control and environment isolation. Gated by `all(retail-12-0-5, any(profile-retail, client-ptr))`.

## Known gaps (current cycle)

- [ ] Apply handoff edits, format, compile the integration binary and run targeted RED/GREEN behavioral evidence. No build or tests ran during authoring; compilation is not claimed.
- [ ] Native identity-secrecy activation, return/error/authentication parity and classification lifecycle remain unverified. Existing synthetic party GUIDs are simulator data, not native GUID evidence.
- [ ] Arena, nameplate, boss, raid and target-of-target reverse mappings have no current modeled provider in UnitTokenFromGUID. Implement real identity state before crediting these classes; do not fabricate lookup records merely to pass tests.
- [ ] Enumerate the prose's unnamed “various others” before crediting either whole row.
- [ ] Nameplate input rows 49/85 remain skipped. Cached generated docs identify UnitTokenNamePlate consumers but not rejection outcome or compound-token grammar. Current GetNamePlateForUnit is a zero-return shim, not evidence of input rejection or plate associations.

## Out of scope

Automatic combat/arena/instance secrecy inference, changing UnitGUID/name output secrecy, secret argument authentication, native token grammar, exact errors, global taint policy, nameplate rendering or associations, replacing unrelated providers, older-profile parity and whole-row audit acceptance. These require independent contracts/evidence, not placeholder results.
