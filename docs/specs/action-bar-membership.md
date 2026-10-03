# Action-bar membership — row245 bounded public producer

`C_ActionBar.IsOnBarOrSpecialBar(SpellIdentifier)` is exact Retail12.0.5 audit row245. This slice implements **INFERRED simulator public direct-spell membership only**, using existing assignments and explicit alias registry policy in `src/c_api/c_action_bar_spell_slots.rs`. Earlier epochs retain the constant-false provider. Independent670 accepted bounded public model proof; source245 stays **PENDING**. See [Lua API architecture](../lua-api.md) and [existing slot-query contract](action-spell-slot-identifiers.md).

Cached declaration: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`, lines826–837, declares `SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one non-nil bool. It does not define special-bar membership, native alias grammar, or result secrecy. Declaration metadata is not native behavioral proof.

## What it must do

The checked requirements below passed eleven public model tests; they remain **inferred simulator policy**, not native parity. Existing fixtures exercise public identifiers only; they prescribe neither native result secrecy nor special-bar semantics.

The producer reuses the existing public `SpellIdentifier` boundary: public UTF-8 strings or finite integral numbers in `0..=u32::MAX`, validated before existing alias-first resolution. Wrong types/domain values error with this API's context. Secret identifiers are conservatively rejected as **UNMODELED native AllowedWhenTainted access**, not native permission credit. No secret payload unwrapping, caller-taint change, output masking, additional normalization, or link/numeric-string parsing is introduced.

### Effective direct assignments

- [x] Return exactly one boolean: true when a resolved spell occupies any positive effective direct-spell slot; false when no such assignment exists.
- [x] Match concrete existing slots3/101 → spell7001 and slot5 → spell7002; spell7003 misses, and clearing the bar makes former matches miss.
- [x] Duplicate membership survives removing slot3 while slot101 remains; removing the last assignment immediately changes membership to false without affecting spell7002.
- [x] Slot0 alone does not create effective membership, matching the existing effective-slot predicate; this is not spell-identifier domain validation.
- [x] Macro/outfit priority excludes underlying direct-spell assignments. Outfit → macro → direct-spell transitions at slot101 restore membership only after both overrides are removed; an unrelated effective spell still matches.

### Public identifier fixtures — simulator registry policy

- [x] Resolve an explicitly registered lowercase name key from lowercase or uppercase public input; unresolved names miss.
- [x] Observe name alias replacement to an unassigned spell, replacement to another assigned spell, and removal immediately.
- [x] An explicit numeric key overrides numeric identity; public numeric-string input requires that registry entry. Replacement/removal is live, and removing the key restores numeric identity without enabling numeric-string coercion.
- [x] A full colored link resolves only through its explicitly registered normalized key. Its embedded spell7001 does not create membership: unresolved and registered-to7003 cases miss, registered-to7002 matches. This asserts registry policy, not native link parsing.

### State ownership

- [x] Queries do not mutate existing bars, macros, outfits, UI-button tuples, or aliases; snapshot populated override fixtures before repeated public matches/misses.
- [x] Removing assignments and replacing aliases in one environment does not change membership or input snapshots in another environment.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing effective slot-query contract](action-spell-slot-identifiers.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/globals/action_bar_api.rs`: constant-false membership provider gated to epochs before `retail-12-0-5`, matching adjacent `FindSpellActionButtons`.
- `src/lua_api/globals/action_bar_api/registration.rs`: old membership entry gated identically; modern namespace receives only the owned modeled registration.
- `src/c_api/c_action_bar_spell_slots.rs`: modern membership provider; named shared public read/query helper with per-API error context for membership and `HasSpellActionButtons`, reusing the positive effective-slot predicate excluding macro/outfit overrides. Existing `HasSpellActionButtons` results/errors and `FindSpellActionButtons` remain unchanged.
- `src/c_api/c_spell.rs`: untouched existing shared public validator and explicit alias registry resolution; no C_Spell companion changes.
- `src/lua_api/state/sim_state.rs`: existing typed assignments and alias registry; tests mutate these actual fields, not new production records.
- `src/lua_api/globals/inventory_verbs.rs`: existing `GetActionInfo` effective-kind priority used for fixture preconditions.

## Tests asserting this spec

`tests/action_bar_membership.rs`: eleven existing public tests under `retail-12-0-5`, auto-discovered by the existing grouped integration runner; no Cargo target added. Assertion helper calls the actual registered API, checks one boolean and expected membership, and never replaces the API. No secrecy assertion is made. Fixtures and C_Spell companion remain untouched.

Historical input RED at `7af22f643ef1ba1584aadf59c1faf0bc14a8efbd`, saved under `/tmp/patch-12.0.5-action-membership-red-ops/`: grouped integration compile exit0 in81.829544s with zero diagnostics (`build.json`); actual API run exit101 in1.578094s (`red.json`), eleven tests **2PASS/9FAIL**, all failures at `effective direct-spell membership`, no fixture errors (`red.stdout`). This is actual behavioral RED against the old provider, not producer proof. Subsequent producer proof is recorded below; historical failures remain unchanged.

| Case | Observable assertion | Historical input proof; all eleven producer cases PASS |
|---|---|---|
| `effective_direct_spell_assignment_matches` | Assigned7001/7002 true | RED: membership assertion |
| `unassigned_spell_and_empty_bar_miss` | Unassigned7003 and cleared assignments false | PASS against old constant-false provider |
| `duplicate_membership_survives_one_removal_then_disappears_after_last` | True → true → false;7002 remains true | RED: membership assertion |
| `slot_zero_alone_does_not_create_effective_membership` | Slot0-only false | PASS against old constant-false provider |
| `registered_lowercase_name_resolves_case_normalized_public_input` | Registered lower/uppercase true; unknown false | RED: membership assertion |
| `name_registry_replace_and_remove_are_live` | Assigned → unassigned → assigned → unresolved | RED: membership assertion |
| `registered_numeric_key_overrides_identity_and_string_requires_registry` | Numeric alias precedence and live removal; unregistered string misses | RED: membership assertion |
| `full_link_is_only_an_explicit_registry_alias_not_embedded_id_grammar` | Embedded assigned ID insufficient; explicit registry controls result | RED: membership assertion |
| `macro_and_outfit_overrides_exclude_direct_spell_until_removed` | Both shadows exclude; outfit removal exposes macro, macro removal exposes spell | RED: membership assertion |
| `membership_queries_leave_existing_inputs_unchanged` | Populated bars/overrides/aliases snapshot unchanged | RED: membership assertion |
| `membership_and_alias_mutations_are_environment_local` | First changes; second membership and snapshot unchanged | RED: membership assertion |

## Independent bounded acceptance — 2026-10-03

Producer `2f6e575f9` compiled with zero diagnostics in106.715008s. Async pinned execution passed **11 membership cases plus17 existing slot-query controls**, followed by pinned simulator startup exit0/`[]` in4.541442s. Independent670 accepted raw evidence, hashes, wiring, companion preservation and changed-code readability. Fresh default check and scoped formatting passed. Global `cargo fmt --check` failed only on untouched concurrent `aura_duration.rs:44`; no global formatting clearance is claimed.

New membership secret/wrong-type behavior has source reasoning only; neighboring controls are not new-API probes. Older-profile parity, special bars and native permissions remain unverified. Exact245 remains pending; capability inventory credit covers only this public direct-spell model. Evidence: `/tmp/patch-12.0.5-action-membership-independent-proof.{md,json}`.

## Known gaps (current cycle)
- [ ] Public validation and conservative secret rejection reuse existing helper policy; native `AllowedWhenTainted` semantics remain UNMODELED. Public-only fixtures establish no native permission, rejection, taint propagation, or result secrecy.
- [ ] Special-bar membership has no defined contract/model in this slice. Direct assignment coverage cannot close row245 or establish special-bar behavior.

## Out of scope

- New production state/fixtures/datasets, Cargo targets and vendor overrides: unnecessary for this existing-model slice.
- Special bars, active page/visibility, vehicle/possess/pet/stance/bonus/override/temporary-bar membership: no bounded model or native definition supplied.
- Native alias grammar, base/override normalization, assignment acquisition, full catalog and profile parity: public simulator fixtures provide no such evidence.
- Native secret-input permissions and result secrecy: `AllowedWhenTainted` metadata alone is insufficient; no native security credit.
- Native wrong-type/domain validation parity: reused public validator is inferred simulator policy, not native evidence; shared identifier and slot-query contracts remain unchanged.
