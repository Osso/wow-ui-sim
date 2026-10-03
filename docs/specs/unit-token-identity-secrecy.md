# UnitTokenFromGUID identity-secret party suppression

Bounded model of [retained 12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 48 and 84: UnitTokenFromGUID no longer returns listed token categories when identities are secret. This slice covers **party tokens only**, using existing party identities and explicit host classification. Both prose rows remain partial; unnamed APIs and other listed token categories receive no credit. [UnitName secret inputs](unit-name-secret-tokens.md) are a separate contract.

## What it must do

- [x] Resolve two existing active party identities normally while classification is empty; marking the first identity secret suppresses only its party token, leaving the second public identity usable.
- [x] **INFERRED:** suppression returns one public nil, preserving the existing missing-lookup return shape. Host removal of classification restores the same mapping.
- [x] **INFERRED:** secure and tainted callers receive the same suppression without clearing or changing caller taint. Public GUID input does not imply public identity; caller taint is not identity secrecy.
- [x] Never create a token for an unknown GUID or inactive roster. **INFERRED:** roster activation does not change host-owned classification.
- [ ] **INFERRED:** classification starts empty in each environment, remains environment-local, and does not alter the existing player/target/focus alias resolution priority. The player control remains resolvable even when its GUID is marked.

## How it works

- [Lua API architecture](../lua-api.md).
- [Retail secret-value contract](retail-secret-values.md).
- [Unit identity comparison permissions](unit-identity-equality.md).

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: host-owned `identity_secret_guids: HashSet<String>`, gated to Retail 12.0.5 / PTR.
- `src/lua_api/state.rs`: empty-default initializer in the shared construction macro.
- `src/lua_api/globals/unit_misc.rs`: party reverse-lookup guard; existing player/target/focus producers and argument conversion unchanged.
- No new registration, namespace, helper module or addon setter.

## Tests asserting this spec

- `tests/unit_token_identity_secrecy.rs`: five grouped API cases covering actual seeded party identities, live host classification/removal, one-nil arity, caller taint, inactive/unknown controls, player control and environment isolation. Gated by `all(retail-12-0-5, any(profile-retail, client-ptr))`.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `92e4ea045`. RED with producers withheld: 1 PASS / 4 FAIL. GREEN: 5/5 inside a 62/62 narrow run; `cargo fmt --check` exit0; startup `lua-errors` `[]`. A broad run gave 1189 PASS / 3 FAIL: a parked bundle slice (since removed) and two tooltip tests that fail identically with this commit's tooltip change reverted.

Main accepts an independent GPT-6.1-sol source review (it did not rerun tests): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b97-verify-tooltip-identity.md) SHA256 `c3581b79231718590e04709f79e717ba21d56434b8e1040ebcf9170098af6dab`. Partial by construction: party tokens only; target/focus overlap is unchanged by inspection and untested. Checked requirements are bounded simulator proof on the tested fixtures, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows prose-2026-03-12-048, prose-2026-03-25-084 under new capability `unit-token-identity-secrecy`; **110 capabilities/362 IDs; 71 pending /243 bounded /15 partial /33 metadata**.

## Known gaps (current cycle)

- [x] Applied, compiled and executed by the main session; see the proof section.
- [ ] Native identity-secrecy activation, return/error/authentication parity and classification lifecycle remain unverified. Existing synthetic party GUIDs are simulator data, not native GUID evidence.
- [ ] Arena, nameplate, boss, raid and target-of-target reverse mappings have no current modeled provider in UnitTokenFromGUID. Implement real identity state before crediting these classes; do not fabricate lookup records merely to pass tests.
- [ ] Enumerate the prose's unnamed “various others” before crediting either whole row.
- [ ] Nameplate input rows 49/85 remain skipped. Cached generated docs identify UnitTokenNamePlate consumers but not rejection outcome or compound-token grammar. Current GetNamePlateForUnit is a zero-return shim, not evidence of input rejection or plate associations.

## Out of scope

Automatic combat/arena/instance secrecy inference, changing UnitGUID/name output secrecy, secret argument authentication, native token grammar, exact errors, global taint policy, nameplate rendering or associations, replacing unrelated providers, older-profile parity and whole-row audit acceptance. These require independent contracts/evidence, not placeholder results.
