# Cooldown aura spell identifiers

Exact row387 changes `C_UnitAuras.GetCooldownAuraBySpellID` argument 1 from `number` to `SpellIdentifier`. The retained [source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines386–387 and [register](../../data/patch-api/sources/12.0.5-register.json) ID `global api-C_UnitAuras-GetCooldownAuraBySpellID-387` identify this delta. Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:323–338` declares required `spellID: SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one nullable `cooldownSpellID: number`. These declarations do not establish acquisition, association direction, identifier representations, or native secret behavior. The public input lives in `src/c_api/c_unit_aura_cooldown_spells.rs`; [Lua API architecture](../lua-api.md) describes the runtime boundary.

## What it must do

All chosen behavior below is **inferred simulator policy**, not native-client parity. Fixtures exist but have not been compiled or executed at this inputs-only stage.

### Explicit input and result

- [ ] Start with an empty per-environment association map; no production catalog or default populated data.
- [ ] Host-declared resolved query spell ID101 returns declared cooldown spell ID901; query202 returns702. A query ID is not asserted to identify an `AuraInfo` record.
- [ ] Return exactly one public number for a hit or exactly one nil for a miss; never an aura DTO/table. Do not recursively resolve the returned ID.
- [ ] Reflect host replacement/removal immediately; queries do not modify associations, aliases, generic aura records, or caller inputs. Two environments remain isolated.
- [ ] A generic player buff with the same spell ID does not supply an association.

### Identifier boundary

- [ ] Require an actual public UTF-8 STRING or finite integral u32 NUMBER, including0/u32MAX. Reject missing/nil, wrong types, invalid UTF-8, nonfinite/fractional/negative/out-of-range numbers before resolution or unknown lookup.
- [ ] Reuse existing `c_spell::read_spell_identifier_at`: lowercase explicit aliases, alias-first numeric resolution, otherwise numeric identity. Strings require a seeded alias, including numeric strings, names and full colored links.
- [ ] Alias replacement and numeric alias override change the resolved query key, not the returned value directly. A seeded link containing101 may explicitly resolve202 and return702.
- [ ] Unknown/unseeded public inputs return one nil; no new parser, automatic hyperlink decoding, or implicit string catalog.

### Caller context and conservative security

- [ ] Ordinary public numeric/name/link calls work in secure and tainted contexts without changing caller taint; returned values remain public.
- [ ] Conservatively reject actual VM secret STRING/NUMBER and host-wrapped wrong objects even in secure context. Preserve rooted identity across GC, secrecy and caller taint; subsequent public calls recover in both contexts. This is explicitly **not** implementation of native secret `AllowedWhenTainted` permissions.

## How it works

- [Lua API architecture](../lua-api.md)
- [C API audit context](../wiki/systems/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_unit_aura_cooldown_spells.rs`: public empty-default explicit association input; no getter or registration yet.
- `src/c_api/mod.rs`: public `retail-12-0-5`-gated module declaration.
- `src/lua_api/state/sim_state.rs`: public feature-gated environment input field.
- `src/lua_api/state.rs`: empty `Default::default()` initializer next to `spell_id_aliases`.
- `src/c_api/c_spell.rs`: existing shared identifier resolver; unchanged.

## Tests asserting this spec

`tests/cooldown_aura_spell_identifiers.rs`: 14 feature-gated grouped integration fixtures covering default absence, two numeric hits, explicit name/link aliases, unknowns, alias mutation, numeric override/string seed, association mutation/nonrecursive output, numeric endpoints, invalid representations, generic-buff independence, environment isolation/read-only inputs, tainted public calls, and actual-secret rejection/GC/recovery.

## First compiled RED and fixture correction — 2026-10-02

At input `14ee1504b`, compilation exit0/462.835s; 14 selected, **3 PASS / 11 FAIL**, exit101/7.791s. `/tmp/patch-12.0.5-batch47-red-{build-result,run}.json` and full outputs bind the preserved unowned duration diff, not a clean revision. Empty/miss/generic-buff controls pass the existing generic nil provider; numeric/name/link/metadata/public-taint cases show genuine missing-producer failures. One secret case failed earlier at an incidental `Val::Userdata` frame-shape assertion, not the query boundary. Current simulator frames are backed Lua tables; corrected setup checks `GetObjectType() == 'Frame'` and retained global rooting, then wraps the actual value without requiring its VM variant. No runtime/framework change or native frame-representation claim. Corrected compiled RED remains pending.

## Known gaps (current cycle)

- [ ] Compile and observe behavioral RED against the missing API; inputs avoid a missing-type compiler failure. Parent owns compiled RED, producer and subsequent gates.
- [ ] Add the sole feature-gated C API getter and registration after RED. This commit intentionally contains neither.
- [ ] Native secret permissions, result secrecy, identifier acceptance, and acquisition/direction remain unverified; no row387 acceptance/accounting claim.

## Out of scope

- Native Forever probes: unavailable; informed guesses are authorized and explicitly labeled rather than a blocking prerequisite.
- Native metadata acquisition, production catalogs, generic aura-derived associations, cooldown/history derivation and listeners: no evidence or authorization; map models only host-declared pairs.
- General declassification, native secret `AllowedWhenTainted` implementation, other rows/profiles, audit totals and wiki/index/log/PLAN changes: parent-owned or separate scope.
