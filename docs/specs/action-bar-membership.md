# Action-bar membership — row245 inputs-only

`C_ActionBar.IsOnBarOrSpecialBar(SpellIdentifier)` is exact Retail12.0.5 audit row245. This slice proposes **INFERRED simulator direct-spell membership only**, using existing assignments and explicit alias registry policy. The registered provider in `src/lua_api/globals/action_bar_api.rs` currently returns constant false. See [Lua API architecture](../lua-api.md) and [existing slot-query contract](action-spell-slot-identifiers.md).

Cached declaration: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua`, lines826–837, declares `SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one non-nil bool. It does not define special-bar membership, native alias grammar, or result secrecy. Declaration metadata is not native behavioral proof.

## What it must do

All requirements below are **unverified inferred policy**, not native parity. Only public identifier fixtures are authored; no authentication policy is prescribed.

### Effective direct assignments

- [ ] Return exactly one boolean: true when a resolved spell occupies any positive effective direct-spell slot; false when no such assignment exists.
- [ ] Match concrete existing slots3/101 → spell7001 and slot5 → spell7002; spell7003 misses, and clearing the bar makes former matches miss.
- [ ] Duplicate membership survives removing slot3 while slot101 remains; removing the last assignment immediately changes membership to false without affecting spell7002.
- [ ] Slot0 alone does not create effective membership, matching the existing effective-slot predicate; this is not spell-identifier domain validation.
- [ ] Macro/outfit priority excludes underlying direct-spell assignments. Outfit → macro → direct-spell transitions at slot101 restore membership only after both overrides are removed; an unrelated effective spell still matches.

### Public identifier fixtures — simulator registry policy

- [ ] Resolve an explicitly registered lowercase name key from lowercase or uppercase public input; unresolved names miss.
- [ ] Observe name alias replacement to an unassigned spell, replacement to another assigned spell, and removal immediately.
- [ ] An explicit numeric key overrides numeric identity; public numeric-string input requires that registry entry. Replacement/removal is live, and removing the key restores numeric identity without enabling numeric-string coercion.
- [ ] A full colored link resolves only through its explicitly registered normalized key. Its embedded spell7001 does not create membership: unresolved and registered-to7003 cases miss, registered-to7002 matches. This asserts registry policy, not native link parsing.

### State ownership

- [ ] Queries do not mutate existing bars, macros, outfits, UI-button tuples, or aliases; snapshot populated override fixtures before repeated public matches/misses.
- [ ] Removing assignments and replacing aliases in one environment does not change membership or input snapshots in another environment.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing effective slot-query contract](action-spell-slot-identifiers.md)
- [Client profiles](client-profiles.md)

## Implementation inventory

- `src/lua_api/globals/action_bar_api.rs`: existing constant-false membership provider; unchanged by this slice.
- `src/lua_api/globals/action_bar_api/registration.rs`: existing registered namespace entry; unchanged.
- `src/c_api/c_action_bar_spell_slots.rs`: existing positive direct-slot predicate excluding macro/outfit overrides; semantic reference, not a new producer in this slice.
- `src/c_api/c_spell.rs`: existing shared public identifier/alias policy used as the public fixture reference; authentication scope for row245 remains Main-owned.
- `src/lua_api/state/sim_state.rs`: existing typed assignments and alias registry; tests mutate these actual fields, not new production records.
- `src/lua_api/globals/inventory_verbs.rs`: existing `GetActionInfo` effective-kind priority used for fixture preconditions.

## Tests asserting this spec

`tests/action_bar_membership.rs`: eleven proposed tests under `retail-12-0-5`, auto-discovered by the existing grouped integration runner; no Cargo target added. Assertion helper calls the actual registered API, checks one boolean and expected membership, and never replaces the API. No secrecy assertion is made. Existing C_Spell/C_ActionBar slot-query files and contracts remain unchanged.

| Proposed case | Observable assertion | Proof level |
|---|---|---|
| `effective_direct_spell_assignment_matches` | Assigned7001/7002 true | Authored; not compiled/run |
| `unassigned_spell_and_empty_bar_miss` | Unassigned7003 and cleared assignments false | Authored; not compiled/run |
| `duplicate_membership_survives_one_removal_then_disappears_after_last` | True → true → false;7002 remains true | Authored; not compiled/run |
| `slot_zero_alone_does_not_create_effective_membership` | Slot0-only false | Authored; not compiled/run |
| `registered_lowercase_name_resolves_case_normalized_public_input` | Registered lower/uppercase true; unknown false | Authored; not compiled/run |
| `name_registry_replace_and_remove_are_live` | Assigned → unassigned → assigned → unresolved | Authored; not compiled/run |
| `registered_numeric_key_overrides_identity_and_string_requires_registry` | Numeric alias precedence and live removal; unregistered string misses | Authored; not compiled/run |
| `full_link_is_only_an_explicit_registry_alias_not_embedded_id_grammar` | Embedded assigned ID insufficient; explicit registry controls result | Authored; not compiled/run |
| `macro_and_outfit_overrides_exclude_direct_spell_until_removed` | Both shadows exclude; outfit removal exposes macro, macro removal exposes spell | Authored; not compiled/run |
| `membership_queries_leave_existing_inputs_unchanged` | Populated bars/overrides/aliases snapshot unchanged | Authored; not compiled/run |
| `membership_and_alias_mutations_are_environment_local` | First changes; second membership and snapshot unchanged | Authored; not compiled/run |

## Known gaps (current cycle)

- [ ] Main must commit inputs and asynchronously observe compiled RED before producer work. No local test/build/check execution, compiled RED, GREEN, or acceptance is claimed.
- [ ] Producer remains constant false; true assertions are proposed failure boundaries, not observed RED evidence.
- [ ] Main chooses producer authentication scope later; public-only fixtures establish no native secret-input permission, rejection, taint propagation, or result secrecy.
- [ ] Special-bar membership has no defined contract/model in this slice. Direct assignment coverage cannot close row245 or establish special-bar behavior.

## Out of scope

- Production edits, new state records, Cargo targets, commits, delegation, desktop operations, and local execution gates: inputs-only authorization.
- Special bars, active page/visibility, vehicle/possess/pet/stance/bonus/override/temporary-bar membership: no bounded model or native definition supplied.
- Native alias grammar, base/override normalization, assignment acquisition, full catalog and profile parity: public simulator fixtures provide no such evidence.
- Native secret-input permissions and result secrecy: `AllowedWhenTainted` metadata alone is insufficient; no native security credit.
- Wrong-type/domain validation and producer authentication decisions: deliberately deferred to Main, without altering existing shared identifier or slot-query contracts.
