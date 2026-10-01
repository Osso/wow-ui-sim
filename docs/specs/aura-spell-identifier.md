# Aura spell-identifier lookup

Bounded Retail 12.0.5 contract for `C_UnitAuras.GetPlayerAuraBySpellID` and `C_UnitAuras.GetUnitAuraBySpellID`, using existing numeric `AuraInfo` records and explicitly seeded `SimState.spell_id_aliases`. Existing aura publication lives in `src/lua_api/globals/auras.rs`; the C API surface belongs in `src/c_api/`. See [Lua API architecture](../lua-api.md). Epoch `retail-12-0-5` now supplies C API-owned producers; targeted GREEN and acceptance remain parent-owned. Checkboxes below remain pending verification.

## What it must do

### Identity and compatibility

- [ ] Preserve numeric C player lookup identity and existing AuraData fields, including spell/instance IDs, display data, duration, source, applications, polarity and points.
- [ ] Preserve C player lookup's helpful-only, unblocked collector behavior. Existing blocked-list entries must not hide matching player buffs. This is simulator compatibility, not native blocked-aura semantics.
- [ ] Leave legacy `GetPlayerAuraBySpellID` unchanged: numeric exact lookup, helpful-only/unblocked, missing/nil default to zero, numeric fractional cast behavior. New C identifier validation and alias resolution must not leak into this global.
- [ ] **INFERRED:** C lookups reuse existing shared C_Spell identifier semantics: lowercase public strings, finite numeric key conversion, alias-first map lookup and numeric identity absent an alias. An explicitly seeded label and its case variant resolve the numeric aura identity; a numeric alias may override an ID. No new parser or implicit label/name producer.
- [ ] Unknown public strings and unmatched resolved IDs return nil, never a fabricated aura. Empty unseeded strings are unknown public strings, not malformed types.

### Supported lookup scope

- [ ] New unit lookup reads existing player and modeled party helpful/harmful records. **INFERRED order:** helpful first, then harmful, preserving record order within each polarity; first matching instance wins. Removing a helpful duplicate exposes the next helpful instance, then the first harmful instance.
- [ ] Target lookup uses only the actual built-in collector fixture: spell `113746`, instance `1`, Weakened Armor, harmful, source `target`. Do not introduce a target aura store or claim modeled target ownership/visibility.
- [ ] Unknown or missing units and missing party entries return nil for valid identifiers. Unit identity stays isolated: player records do not satisfy party queries or vice versa. This does not implement native visibility restrictions.
- [ ] Returned DTOs are independent snapshots surviving full GC; Lua mutations do not alter host auras or later results. Subsequent host removal is visible to new queries; aura and alias state remain environment-local.

### Validation and taint

- [ ] **INFERRED strict representation policy:** missing/nil identifiers, booleans, tables, functions and nonfinite numbers raise meaningful errors before returning a DTO or absent-unit nil. This adds no numeric range/integrality policy beyond existing shared resolver semantics.
- [ ] **INFERRED conservative secret policy:** reject actual secret identifiers in secure and addon-tainted callers, including absent-unit queries. Full GC preserves secret identity. Rejection must not unwrap/change the secret, clear/replace caller taint, or prevent later public queries. Ordinary successful calls also preserve caller taint. This is not native `AllowedWhenTainted` enforcement parity.

### Exact evidence and limits

Observed cached files on **2026-10-01**, under `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`:

- `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:374–389` declares player lookup's non-nilable `SpellIdentifier`, nullable AuraData, `SecretWhenUnitAuraRestricted`, `RequiresNonSecretAura`, and `SecretArguments = "AllowedWhenTainted"`.
- Same file `:412–430` declares unit lookup's non-nilable, `NeverSecret` restricted unit token and non-nilable `SpellIdentifier`. It describes the “first instance” matching a spell and nil for no match; its not-visible example is party members on other maps. No polarity traversal order is specified.
- `Blizzard_APIDocumentationGenerated/SpellDocumentation.lua:357–373` documents spell ID, name, name(subtext), or link for **C_Spell.GetSpellIDForSpellIdentifier**, names checking overrides and IDs returning themselves. This is not evidence that aura APIs natively share this resolver or accept each representation.
- `Blizzard_CooldownViewer/CooldownViewerItemData.lua:1–29` scans `player` then `target`, queries unit lookup with values iterated from `cooldownInfo.linkedSpellIDs`, and requires `sourceUnit == "player"` before accepting a link. Numeric representation is inferred from that field/caller, not a demonstrated string caller or fixture source-player match.
- Retained [`12.0.5-register.json`](../../data/patch-api/sources/12.0.5-register.json) rows `global api-C_UnitAuras-GetPlayerAuraBySpellID-392` and `global api-C_UnitAuras-GetUnitAuraBySpellID-396` record arg1/arg2 type changes from number to SpellIdentifier. Row394 concerns refresh duration and remains excluded.

Current local evidence: `src/c_api/c_spell.rs::alias_key_from_input/read_spell_identifier` normalizes keys and reads the initially empty alias map; these tests explicitly populate that map. `src/lua_api/globals/auras.rs::collect_unit_auras` reads player buffs by polarity, party buffs/debuffs, and `target_fixture_auras`; it has no generic target store. Existing player lookup uses the helpful collector, not the blocked-filtering visible collector. `build_aura_table` provides existing DTO fields and forces player-query sourceUnit to `player`.

**INFERRED** seeded-label reuse, numeric alias precedence on aura queries, helpful-then-harmful ordering, missing-unit policy and conservative validation/security are bounded simulator requirements. Fixture labels are not native spell names, subtext, links, localization or overrides. No native behavior was probed. Generated access/secret annotations do not establish native enforcement or justify declassification.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing unit-aura enumeration contract](unit-aura-instance-enumeration.md)
- [Public classification contract](aura-classification-flags.md)

## Implementation inventory

- `src/lua_api/game_data.rs`, `src/lua_api/state/sim_state.rs` — existing AuraInfo records, party stores and explicit spell alias map; no changes in this slice.
- `src/lua_api/globals/auras.rs` — unchanged legacy numeric provider; narrow `push_aura_by_spell_id` reuses unblocked collector and DTO builder. Earlier epochs retain the numeric C player provider.
- `src/c_api/c_spell.rs` — narrow `read_spell_identifier_at` shares the existing alias-first resolver; C_Spell arg1 behavior is unchanged.
- `src/c_api/c_unit_aura_spell_queries.rs` — `retail-12-0-5` C providers validate public identifiers before resolution or absent-unit results. Unit nil returns nil; non-string/secret unit tokens error. Neither caller taint nor secret identity is modified.
- `src/lua_api/globals/register.rs` — installs authoritative C spell queries after legacy aura registration, which no longer installs the C player entry at this epoch. Separate from the `aura-instance-enumeration`-gated `c_unit_auras` module; enumeration registrations do not overwrite these queries.
- `tests/aura_spell_identifier.rs` — twelve retail-12-0-5-gated cases automatically included in the existing grouped integration target; no additional Cargo target.

## Tests asserting this spec

Grouped filter: `aura_spell_identifier::`, with feature `retail-12-0-5` (parent's epoch retail12-0-5 configuration).

| Test | Contract |
|---|---|
| `numeric_player_lookup_preserves_current_dto_identity_and_fields` | Numeric control and DTO identity/fields |
| `player_lookup_remains_helpful_only_and_ignores_existing_block_list` | Existing helpful-only/unblocked compatibility |
| `legacy_player_global_retains_optional_numeric_input_behavior` | Unchanged legacy missing/nil/numeric behavior |
| `explicitly_seeded_aliases_reuse_shared_spell_resolution_case_normalization` | INFERRED seeded label reuse, player and party |
| `numeric_alias_precedence_matches_shared_resolver_without_changing_legacy` | Existing alias-first resolution vs unchanged legacy |
| `unit_lookup_reads_modeled_party_helpful_and_harmful_aura_rows` | Party/player polarity and unit DTO fields |
| `inferred_unit_order_prefers_first_helpful_then_first_harmful_duplicate` | INFERRED stable first-match order and live removal |
| `unit_target_query_uses_actual_builtin_harmful_fixture_not_invented_state` | Actual target collector fixture only |
| `missing_units_and_unmatched_public_identifiers_do_not_fabricate_auras` | Missing unit, unmatched ID/string and unit isolation |
| `missing_and_invalid_identifiers_error_even_when_unit_has_no_aura_store` | Validation before absent-unit result |
| `dto_snapshots_and_alias_state_are_environment_isolated_across_gc` | DTO snapshots, GC, live state and environment isolation |
| `actual_secret_identifier_rejection_survives_gc_and_preserves_caller_taint` | Real host secret, secure/tainted rejection and public recovery |

Rust eval outputs use supported `i32`, not `u8`/`usize`. Secret fixture uses `wrap_host_secret_number`, not a mocked Lua secret predicate.

## Known gaps (current cycle)

- Saved parent RED at `28393b01ba6bafc3d233cbc700c7b02303de1064`: `/tmp/patch-12.0.5-batch37-red-build-result.json` records build exit0 / 130.04s; corresponding `-run.json` and `-run.log` record exit101, three existing numeric/legacy controls PASS and nine FAIL. This evidence covers the pre-producer revision only.
- [ ] Parent owns targeted GREEN, C_Spell/legacy controls, startup registration proof, checks/readability and acceptance. Producer work runs formatting only: no build/test/check/delegation. Numeric/seeded-label/numeric-alias, ordering, GC snapshots and actual-secret/taint tests exist but have no post-producer passing proof.
- No source accounting or audit-row credit changes. Native visibility, name/link parsing, secret annotations and all-profile acceptance remain unproved.

## Out of scope

- `GetRefreshExtendedDuration` / retained row394: no duration formula, provider, fixture or test in this slice.
- Native visibility, protected/blocked semantics, real native secret/access enforcement and full audit-row completion: annotations and current fixtures cannot prove them. Only existing player blocked-list compatibility is pinned.
- Native name/subtext/link/localization/override acceptance or alias production: map entries are explicitly seeded fixture tokens only.
- New target storage, arbitrary units, event/combat producers, additional profiles or cached UI closure proof: reuse existing collector only.
