# Cooldown aura spell identifiers

Exact row387 changes `C_UnitAuras.GetCooldownAuraBySpellID` argument 1 from `number` to `SpellIdentifier`. The retained [source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines386–387 and [register](../../data/patch-api/sources/12.0.5-register.json) ID `global api-C_UnitAuras-GetCooldownAuraBySpellID-387` identify this delta. Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:323–338` declares required `spellID: SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one nullable `cooldownSpellID: number`. These declarations do not establish acquisition, association direction, identifier representations, or native secret behavior. The public input lives in `src/c_api/c_unit_aura_cooldown_spells.rs`; [Lua API architecture](../lua-api.md) describes the runtime boundary.

## What it must do

All chosen behavior below is **inferred simulator policy**, not native-client parity. The producer is implemented after corrected parent compiled RED; saved parent GREEN is reconciled below, while independent acceptance remains pending. Unchecked requirements below are not passing claims.

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

## Reconciled batch47 parent GREEN — 2026-10-02

Saved parent evidence binds producer `c6fe6093b4a77335d5f2bbcdfe9e7a62b30aeff5`, inputs/model `14ee1504b`, and fixture correction `22c84baf99a173fe81a4e6d431b6b295136f89eb`, **plus preserved unowned dirty source**, not a clean revision. Dirty diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`; its contents were not inspected or modified here. Original RED3PASS/11FAIL (including invalid FrameShape setup) and corrected RED3PASS/11 genuine FAIL above remain historical evidence.

`/tmp/patch-12.0.5-batch47-green-build-result.json` records `cargo test --test integration --no-run --message-format=json`: exit0, **259.7876708320109s**. Full build output: `/tmp/patch-12.0.5-batch47-green-build.{jsonl,log}`. Integration executable `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`, SHA256 `3a433f3fdac41b264281080d3d919cb99679e30b97c714c1a28c8b2126aa4875`.

`/tmp/patch-12.0.5-batch47-green-runs.json` records each command as `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c <filter> --test-threads=1`. Full corresponding outputs: `/tmp/patch-12.0.5-batch47-green-run-{0..13}.{stdout,stderr}`, in table order; all14 exits0. Full saved test names establish **14 new +196 controls =210 unique PASS**, not duplicated filter counts.

| Exact filter | PASS | Saved seconds |
|---|---:|---:|
| `cooldown_aura_spell_identifiers::` | 14 | 2.2547551459865645 |
| `aura_dispel_color_arguments::` | 16 | 6.496058761025779 |
| `unit_aura_slot_enumeration_arguments::` | 12 | 2.263698844006285 |
| `c_unit_auras_admin::` | 14 | 2.710818149964325 |
| `userdata_proxy::color_curve_` | 18 | 3.356312908930704 |
| `unit_aura_slot_secret_arguments::` | 12 | 2.5015637930482626 |
| `aura_application_display_count::` | 14 | 3.873023972962983 |
| `next125aura::` | 12 | 2.8686415660195053 |
| `unit_aura_filter_query::` | 14 | 3.1403976880246773 |
| `aura_table_shape::` | 7 | 1.5626325160264969 |
| `aura_api::` | 29 | 5.521967396955006 |
| `admin_buff_api::` | 18 | 3.6171123179374263 |
| `aura_refresh_duration::` | 18 | 3.778685806086287 |
| `aura_spell_identifier::` | 12 | 2.8660488630412146 |

Actual warm test runtime sums to **46.811717730015516s**, below the60s partition target. Retained without padding or rerun as **bounded development evidence**, not final whole-page, whole-goal or native acceptance.

`/tmp/patch-12.0.5-batch47-green-startup-run.json` records `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors`: exit0, **6.032388805993833s**, stdout `[]`. Full output `/tmp/patch-12.0.5-batch47-green-startup.{stdout,stderr}`; simulator SHA256 `7c51db5659e2d230f9d6d6deaee60a14d4c30e3e64d833d22c23d63c38229d8f`.

| Capability | Saved parent GREEN | Proof limit |
|---|---|---|
| Empty default, misses, generic-buff independence | PASS in new14 | Inferred empty association model; no acquisition/catalog proof |
| Numeric/name/link hits, explicit aliases/overrides, association replacement/removal, nonrecursive output | PASS in new14 | Host-declared direction and alias policy, not native identifier acceptance |
| Numeric endpoints and invalid public representations | PASS in new14 | Inferred strict public UTF-8 STRING / integral u32 boundary |
| Environment isolation, read-only inputs, secure/tainted public calls | PASS in new14 | Simulator public result and caller-taint behavior only |
| Actual VM secret inputs/wrapped wrong frame, rooted GC identity and recovery | PASS in new14 after FrameShape correction | Conservative rejection, not native AllowedWhenTainted/access/result-secrecy permission |

Independent verifier remains pending. No acceptance, source accounting or all-pass checkbox claim: **row387 remains pending**, requirements stay unchecked. Native `AllowedWhenTainted`, access, result secrecy, acquisition, association direction and identifier parity remain unknown. No checks/tests/builds were run for this documentation reconciliation; saved parent artifacts are the sole execution proof.

## Known gaps (current cycle)

- [ ] Independent acceptance of the implemented sole getter/registration remains pending. Saved parent compiled GREEN is recorded above; it is dirty-source-bound bounded development proof, not final acceptance.
- [ ] Native secret permissions, result secrecy, identifier acceptance, and acquisition/direction remain unverified; no row387 acceptance/accounting claim.

## Out of scope

- Native Forever probes: unavailable; informed guesses are authorized and explicitly labeled rather than a blocking prerequisite.
- Native metadata acquisition, production catalogs, generic aura-derived associations, cooldown/history derivation and listeners: no evidence or authorization; map models only host-declared pairs.
- General declassification, native secret `AllowedWhenTainted` implementation, other rows/profiles, audit totals and PLAN changes: parent-owned or separate scope.
