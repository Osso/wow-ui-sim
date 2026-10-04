# Explicit special-bar membership — row245 extension

`C_ActionBar.IsOnBarOrSpecialBar(SpellIdentifier)` extends [existing public direct membership](action-bar-membership.md) with explicit per-environment host special-bar spell IDs. The [12.0.5 delta](../../data/patch-api/sources/12.0.5-api-changes.txt), row245, changes arg1 from number to `SpellIdentifier`. Cached retail `ActionBarFrameDocumentation.lua:826–837` declares one required identifier, `SecretArguments = 'AllowedWhenTainted'`, and one nonnil boolean. This is an INFERRED public host model, not native security or special-bar acquisition proof.

## What it must do

- [ ] INFERRED union policy: return exactly one public boolean, true if the resolved spell has a positive effective direct-spell slot OR belongs to `SimState.special_bar_spells`.
- [ ] INFERRED empty-state policy: initialize special membership to an empty set; do not derive membership from bar visibility, pages, vehicle state, macros or outfits.
- [ ] Preserve existing direct-slot precedence: macro/outfit overrides exclude the underlying direct assignment, but do not remove independent explicit special membership.
- [ ] Read insertion/removal and alias replacement/removal live. Unknown public aliases miss; use the existing alias-first public reader without introducing identifier grammar or coercions.
- [ ] Leave `HasSpellActionButtons`, `FindSpellActionButtons`, direct assignments, aliases and special records unchanged by queries; isolate environments.
- [ ] Retain the existing INFERRED public validator and conservative secret rejection for both untainted and tainted callers. This is an explicitly UNMODELED native `AllowedWhenTainted` boundary, not evidence that native secret calls should fail. Preserve wrapper secrecy and caller taint across failures and GC.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing slot-query contract](action-spell-slot-identifiers.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — host `HashSet<u32>` of resolved spell IDs, under `retail-12-0-5`.
- `src/lua_api/state.rs` — empty set initialization under the same feature.
- `src/c_api/c_action_bar_spell_slots.rs` — existing registered producer reads the union; sibling direct-only queries remain untouched.
- `src/c_api/c_spell.rs` — unchanged existing public identifier reader.

## Tests asserting this spec

`tests/pdeid_specialbar.rs`: `special_membership_is_live_union_not_direct_slot_assignment`, `special_membership_uses_live_alias_resolution_not_link_grammar`, `special_membership_survives_direct_macro_and_outfit_shadows`, `special_membership_queries_are_read_only_and_environment_local`, `special_membership_preserves_public_validation_and_unmodeled_secret_boundary`. Authored only; not executed. Existing eleven `tests/action_bar_membership.rs` cases retain their expectations with the empty special set.

## Development proof and independent bounded acceptance — 2026-10-03 (action-bar-special-membership)

Commit `4d142e325`. RED: 0 PASS / 5 FAIL. GREEN: 5/5 special-bar cases. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-housing-bars.md) SHA256 `6cc70517c54675b995a442bc47d2fa681c21e457ea32e5dd0d9d582baa6fc65d`. Identifier grammar and secret permission unmodeled; special-bar acquisition is host-declared. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): global api-C_ActionBar-IsOnBarOrSpecialBar-245 partial-development-green under capability `action-bar-special-membership`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Compile and run authored tests and existing direct-membership controls; no RED/GREEN execution is claimed.
- [ ] Native `AllowedWhenTainted` secret-input permission and output secrecy remain unmodeled. Do not substitute `unwrap_secret`: that implements the different `AllowedWhenUntainted` policy.

## Out of scope

Native bar acquisition/classification, visibility-derived membership, alias/name/link grammar, native secrecy/validation parity, vendor edits, older-profile parity and whole-row audit closure. No compatibility fallback.
