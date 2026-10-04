# Classic secret policy

The 12.0.5 [source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines 213–215 state that on Classic builds the secret value system is disabled, Midnight chat/guild restrictions are inactive, pre-Midnight taint and restricted actions stay active, and duration objects and curves are available.

## What it must do

- [x] On Wrath, Mists, Era and Anniversary, identity and roster name markers are not produced and tainted `loadstring` closures are not classified secret. Retail, PTR and Forever keep their behavior.
- [x] Taint stamping and protected-action denial are unchanged on Classic.
- [x] Restricted stat and cooldown inputs yield ordinary numbers on Classic (already true before this change; control).
- [x] Duration objects and numeric/color curve constructors work on Classic (already true; control).
- [ ] Every secret-producing API and injected VM secrets on Classic.
- [ ] Chat, guild and combat-log clauses of line 214.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/client_profile.rs` — runtime predicate `uses_secret_values()`.
- `src/lua_api/globals/security/secret_values.rs` — marker production and closure classification consult it.

## Tests asserting this spec

- `tests/classic_secret_policy.rs` — seven cases, compiled only on the four Classic profiles.
- `tests/security_api.rs` — seven expectations follow the profile predicate.

## Development proof and independent bounded acceptance — 2026-10-03 (classic-secret-policy)

Commit `ce418cbe4`. RED: 4 PASS / 3 FAIL under client-mists. GREEN: 7/7 new + 46 security_api + 15 admin_identity_api under client-mists; retail 122/122 after cherry-pick. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/classic-secrets-review.md) SHA256 `edae62bd107133e94808f39f7015ff1fe3cd032a3edb4778387da80e09eb1333`. One Classic profile executed; line 214's chat/guild/combat-log clauses untested. Row 215 (documentation cleanup) stays pending. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-04-17-213 partial-development-green, prose-2026-04-17-214 partial-development-green under capability `classic-secret-policy`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Only the Mists profile was executed. The seven `security_api` expectations derive from the same predicate as the implementation.

## Out of scope

- Native VM secret propagation machinery, which stays compiled; host-injected wrappers still propagate.
