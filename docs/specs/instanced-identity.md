# Instanced-map identity secrecy

Bounded host-backed model of [retained 12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), lines 197–199. Line 197 says identities are secret on instanced maps except the player, their pet/vehicle and raid/party members; line 198 describes the previous attackable-only policy, not a current requirement; line 199 says mind control on allies no longer changes identity secrecy. This slice covers existing resolved player/party/target/focus GUIDs, not all native unit categories. [Input-token rejection](unit-name-secret-tokens.md) is a separate contract.

## What it must do

- [ ] Host-declared instance context makes existing attackable and friendly nongroup visitor name/GUID outputs secret. Attackability is not a secrecy input; this same test supplies line 198's historical contrast without emulating the obsolete policy.
- [ ] Active host-owned party roster membership follows resolved GUID aliases across party/target/focus getters; player-owned GUID aliases are explicitly declared by the host. The seeded local player remains public under the map rule.
- [ ] Host-recorded mind control neither removes an ally exemption nor creates a visitor exemption, even when the actual UnitCanAttack result changes.
- [ ] UnitName, UnitNameUnmodified, GetUnitName, UnitPVPName, UnitFullName and UnitGUID share the bounded identity policy while preserving current return arity and public payloads.
- [ ] INFERRED: leaving the instance changes new getter results to public without declassifying retained secret results or poisoning equal ordinary literals and realm strings.
- [ ] INFERRED: the sibling host-owned identity_secret_guids set remains independent and overrides map exemptions. Secure and tainted callers receive the same classification without altering caller taint. No input unwrapping or caller-security bypass is added.

INFERRED policies: false/empty default map/player-owned/control context; existing party_group_active and party_members supply group membership using the existing synthetic party GUID provider. Host owns map, player-owned identity and control lifecycle; no automatic world discovery. Explicit classification precedence is simulator policy, not native-verified semantics. Existing unresolved identities cannot be classified and retain existing absence/placeholder outputs; that is a known boundary, not an alternate classifier. Exempt membership is not inferred from a token spelling, friendliness or attackability.

## How it works

- [Lua API architecture](../lua-api.md).
- [Retail secret-value contract](retail-secret-values.md).
- [Sibling identity-token contract](unit-token-identity-secrecy.md).

## Implementation inventory

- `src/lua_api/globals/real/instanced_identity.rs`: explicit instance/player-owned/control state and shared GUID-based predicate.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: environment-local field and default construction.
- `src/lua_api/globals/unit_misc.rs`, `src/lua_api/globals/group_queries.rs`: actual getter outputs use trusted rilua host-secret string results, not permanently marked interned strings.
- `src/lua_api/globals/real/mod.rs`: module export, gated to retail-12-0-5 with profile-retail/client-ptr.

## Tests asserting this spec

- `tests/instanced_identity.rs`: six behavioral cases in the existing auto-discovered integration binary.
- `tests/security_api.rs`: three existing secrecy tests explicitly classify their fixture GUID instead of assuming every party token is secret.
- `tests/unit_name_secret_tokens.rs`: accepted input-authentication precedent; no changes proposed.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 6 FAIL. GREEN: 6/6 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-identity.md) SHA256 `70f728554b59a734ba418b08dbe1c668e3fd38c851f8ca831e4f915357516b36`. Partial: raid/pet/vehicle identities unmodeled and precedence inferred; default party names changed from secret to public on retail 12.0.5. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-04-10-197 partial-development-green, prose-2026-04-10-198 partial-development-green, prose-2026-04-10-199 partial-development-green under capability `instanced-identity`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Apply staged edits and run behavioral RED with only state/type/test changes, then GREEN with producers. No compilation or runtime evidence obtained by this authoring task.
- [ ] Native-client classification, exemption precedence and output-lifetime proof remain absent.
- [ ] Raid, pet and vehicle GUID producers are not currently modeled by existing_guid_for_unit/guid_for_unit; no implementation credit for their identity categories. Their existing name outputs are not proof of independent identities.
- [ ] Quest NPC, arena, boss, nameplate and other token categories without actual resolved GUIDs remain unmodeled here.

## Out of scope

Other identity producers such as GetRaidRosterInfo retain their current registry marking; pre-existing marks from them are not cleared. Automatic world/roster acquisition, pet/vehicle/raid GUID synthesis, control spell mechanics, restoring historical attackable-only behavior, reverse-token policy redesign, other identity consumers, vendor edits and broader native-parity claims. Missing native identities cannot honestly be supplied by wrapping fixture constants or parsing token names.

## Existing-caller behavior change

Within the scoped retail epoch, party names are no longer automatically secret solely because of token spelling. Ordinary party/raid names outside instances become public absent explicit classification (raid aliases lack resolved GUIDs and remain a stated gap). Active roster membership supplies the group exemption without a duplicate membership set. Host must declare player-owned GUID aliases explicitly. Classified/instanced nongroup target/focus names and GUIDs become VM-owned secret outputs. Equal public literals are no longer incidentally marked by an identity getter, and retained secret outputs remain secret after context changes. Other profiles preserve their previous name/GUID behavior. Existing security tests are adjusted by explicit fixture state; accepted UnitName token-input tests and sibling reverse-party suppression stay unchanged.
