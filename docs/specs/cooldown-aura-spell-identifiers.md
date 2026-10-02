# Cooldown aura spell identifiers

Exact row387 changes `C_UnitAuras.GetCooldownAuraBySpellID` argument 1 from `number` to `SpellIdentifier`. The retained [source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines386–387 and [register](../../data/patch-api/sources/12.0.5-register.json) ID `global api-C_UnitAuras-GetCooldownAuraBySpellID-387` identify this delta. Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:323–338` declares required `spellID: SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one nullable `cooldownSpellID: number`. These declarations do not establish acquisition, association direction, identifier representations, or native secret behavior. The public input lives in `src/c_api/c_unit_aura_cooldown_spells.rs`; [Lua API architecture](../lua-api.md) describes the runtime boundary.

## What it must do

All chosen behavior below is **inferred simulator policy**, not native-client parity. The producer is implemented after corrected parent compiled RED; GREEN and independent acceptance remain parent-owned and pending. Unchecked requirements below are not passing claims.

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
- [Lua API cooldown association boundary](../wiki/systems/lua-api.md#retail-1205-cooldown-aura-spell-identifiers)
- [Frame representation and fixture boundary](../wiki/investigations/frame-surrogate-identity-slot.md)
- [C API audit context](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_unit_aura_cooldown_spells.rs`: unchanged public empty-default input plus sole getter, strict public validation, actual VM secret rejection and direct nullable numeric lookup.
- `src/lua_api/globals/register.rs`: `retail-12-0-5` registration after existing aura namespace/state initialization.
- `src/c_api/mod.rs`: public `retail-12-0-5`-gated module declaration.
- `src/lua_api/state/sim_state.rs`: public feature-gated environment input field.
- `src/lua_api/state.rs`: empty `Default::default()` initializer next to `spell_id_aliases`.
- `src/c_api/c_spell.rs`: existing shared identifier resolver; unchanged.

## Tests asserting this spec

`tests/cooldown_aura_spell_identifiers.rs`: 14 feature-gated grouped integration fixtures covering default absence, two numeric hits, explicit name/link aliases, unknowns, alias mutation, numeric override/string seed, association mutation/nonrecursive output, numeric endpoints, invalid representations, generic-buff independence, environment isolation/read-only inputs, tainted public calls, and actual-secret rejection/GC/recovery.

## First compiled RED and fixture correction — 2026-10-02

At input `14ee1504b`, compilation exit0/462.835s; 14 selected, **3 PASS / 11 FAIL**, exit101/7.791s. `/tmp/patch-12.0.5-batch47-red-{build-result,run}.json` and full outputs bind the preserved unowned duration diff, not a clean revision. Empty/miss/generic-buff controls pass the existing generic nil provider; numeric/name/link/metadata/public-taint cases show genuine missing-producer failures. One secret case failed earlier at an incidental `Val::Userdata` frame-shape assertion, not the query boundary. Current simulator frames are backed Lua tables; corrected setup checks `GetObjectType() == 'Frame'` and retained global rooting, then wraps the actual value without requiring its VM variant. No runtime/framework change or native frame-representation claim. This original secret setup failure is historical, not genuine producer RED.

## Corrected compiled RED and producer boundary — 2026-10-02

Inputs/model `14ee1504b`; fixture correction `22c84baf99a173fe81a4e6d431b6b295136f89eb`. Saved `/tmp/patch-12.0.5-batch47-red-fixed-build-result.json` records `cargo test --test integration --no-run --message-format=json`, exit0/296.8498461409472s. Saved `/tmp/patch-12.0.5-batch47-red-fixed-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c cooldown_aura_spell_identifiers:: --test-threads=1`, **14 selected / 3 PASS / 11 genuine FAIL**, exit101/4.286526938085444s. Full outputs are `/tmp/patch-12.0.5-batch47-red-fixed-build.{jsonl,log}` and `/tmp/patch-12.0.5-batch47-red-fixed-run.{stdout,stderr}`. Binary SHA256 `69ea9a86c11ef1051b4f080557f3d4cdd5e88a62b7db890f6f52a858e607445b`; saved dirty-source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. Evidence includes preserved unowned duration changes; not clean-revision proof.

| Capability | Corrected pre-producer evidence | Current proof limit |
|---|---|---|
| Empty map, misses, generic-buff independence | Three controls PASS through generic nil provider | Not producer proof |
| Numeric/name/link hits, alias/association changes, endpoints, environment/read-only and public-taint queries | Missing numeric responses FAIL | Implemented; parent GREEN pending |
| Invalid public representations | API argument rejection FAIL | Implemented strict boundary; parent GREEN pending |
| Actual secret inputs/wrapped wrong frame, GC/rooting/recovery | Frame `GetObjectType()` and root setup succeed; API argument rejection FAIL | Conservative rejection implemented; parent GREEN pending |

The getter uses the existing shared alias resolver only after validation and reads only the declared map value. No changes to model shape/default, fixtures, generic aura records, cooldown state/history or catalogs. Ordinary public calls have no secure-caller gate or taint mutation. Native secret permissions, output secrecy, acquisition and direction remain explicit gaps.

## Known gaps (current cycle)

- [ ] Parent compiled GREEN and independent acceptance of the implemented sole getter/registration remain pending. Corrected compiled RED is recorded above; producer work runs owned formatting only, no new execution proof.
- [ ] Native secret permissions, result secrecy, identifier acceptance, and acquisition/direction remain unverified; no row387 acceptance/accounting claim.

## Out of scope

- Native Forever probes: unavailable; informed guesses are authorized and explicitly labeled rather than a blocking prerequisite.
- Native metadata acquisition, production catalogs, generic aura-derived associations, cooldown/history derivation and listeners: no evidence or authorization; map models only host-declared pairs.
- General declassification, native secret `AllowedWhenTainted` implementation, other rows/profiles, audit totals and PLAN changes: parent-owned or separate scope.
