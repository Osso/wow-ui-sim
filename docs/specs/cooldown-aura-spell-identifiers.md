# Cooldown aura spell identifiers

Exact row387 changes `C_UnitAuras.GetCooldownAuraBySpellID` argument 1 from `number` to `SpellIdentifier`. The retained [source](../../data/patch-api/sources/12.0.5-api-changes.txt) lines386–387 and [register](../../data/patch-api/sources/12.0.5-register.json) ID `global api-C_UnitAuras-GetCooldownAuraBySpellID-387` identify this delta. Cached Retail `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:323–338` declares required `spellID: SpellIdentifier`, `SecretArguments = "AllowedWhenTainted"`, and one nullable `cooldownSpellID: number`. These declarations do not establish acquisition, association direction, identifier representations, or native secret behavior. The public input lives in `src/c_api/c_unit_aura_cooldown_spells.rs`; [Lua API architecture](../lua-api.md) describes the runtime boundary.

## What it must do

All chosen behavior below is **inferred simulator policy**, not native-client parity. The producer is implemented after corrected parent compiled RED; saved parent GREEN is reconciled below; independent373 bounded acceptance follows. Checked requirements assert tested inferred simulator behavior only, not native parity.

### Explicit input and result

- [x] Start with an empty per-environment association map; no production catalog or default populated data.
- [x] Host-declared resolved query spell ID101 returns declared cooldown spell ID901; query202 returns702. A query ID is not asserted to identify an `AuraInfo` record.
- [x] Return exactly one public number for a hit or exactly one nil for a miss; never an aura DTO/table. Do not recursively resolve the returned ID.
- [x] Reflect host replacement/removal immediately; queries do not modify associations, aliases, generic aura records, or caller inputs. Two environments remain isolated.
- [x] A generic player buff with the same spell ID does not supply an association.

### Identifier boundary

- [x] Require an actual public UTF-8 STRING or finite integral u32 NUMBER, including0/u32MAX. Reject missing/nil, wrong types, invalid UTF-8, nonfinite/fractional/negative/out-of-range numbers before resolution or unknown lookup.
- [x] Reuse existing `c_spell::read_spell_identifier_at`: lowercase explicit aliases, alias-first numeric resolution, otherwise numeric identity. Strings require a seeded alias, including numeric strings, names and full colored links.
- [x] Alias replacement and numeric alias override change the resolved query key, not the returned value directly. A seeded link containing101 may explicitly resolve202 and return702.
- [x] Unknown/unseeded public inputs return one nil; no new parser, automatic hyperlink decoding, or implicit string catalog.

### Caller context and conservative security

- [x] Ordinary public numeric/name/link calls work in secure and tainted contexts without changing caller taint; returned values remain public.
- [x] Conservatively reject actual VM secret STRING/NUMBER and host-wrapped wrong objects even in secure context. Preserve rooted identity across GC, secrecy and caller taint; subsequent public calls recover in both contexts. This is explicitly **not** implementation of native secret `AllowedWhenTainted` permissions.

## How it works

- [Lua API architecture](../lua-api.md)
- [Lua API cooldown association boundary](../wiki/systems/lua-api.md#retail-1205-cooldown-aura-spell-identifiers)
- [Frame representation and fixture boundary](../wiki/investigations/frame-surrogate-identity-slot.md)
- [C API audit context](../wiki/investigations/patch-12-0-5-api-audit.md)

## Implementation inventory

- `src/c_api/c_unit_aura_cooldown_spells.rs`: unchanged public empty-default input plus sole getter, shared public identifier validation and direct nullable numeric lookup; number/nil results and API-specific errors preserved.
- `src/lua_api/globals/register.rs`: `retail-12-0-5` registration after existing aura namespace/state initialization.
- `src/c_api/mod.rs`: public `retail-12-0-5`-gated module declaration.
- `src/lua_api/state/sim_state.rs`: public feature-gated environment input field.
- `src/lua_api/state.rs`: empty `Default::default()` initializer next to `spell_id_aliases`.
- `src/c_api/c_spell.rs`: unchanged alias-first `read_spell_identifier_at` resolver; `read_public_spell_identifier_at` now owns the existing strict public validation/conservative VM-secret rejection shared by this getter and both classification getters. Extraction adds no native permission or acceptance proof.

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

Historical GREEN-only checkpoint, superseded by independent bounded acceptance below: row387 was pending and requirements unchecked. Native `AllowedWhenTainted`, access, result secrecy, acquisition, association direction and identifier parity remain unknown. No checks/tests/builds were run for this documentation reconciliation; saved parent artifacts are the sole execution proof.

## Known gaps (current cycle)

- [x] Independent373 accepted the sole getter/registration within the bounded inferred simulator contract; dirty-source-bound development proof, not final whole-page/goal acceptance.
- [ ] Native AllowedWhenTainted permissions, secret access, result secrecy, identifier parity, catalog and acquisition/direction remain unknown; bounded row387 credit does not close them.

## Out of scope

- Native Forever probes: unavailable; informed guesses are authorized and explicitly labeled rather than a blocking prerequisite.
- Native metadata acquisition, production catalogs, generic aura-derived associations, cooldown/history derivation and listeners: no evidence or authorization; map models only host-declared pairs.
- General declassification, native secret `AllowedWhenTainted` implementation, other rows/profiles, audit totals and PLAN changes: parent-owned or separate scope.

## Independent bounded acceptance — 2026-10-02

Parent fully read and accepted independent373 report `/tmp/patch-12.0.5-cooldown-aura-identifier-independent-proof.md`. Only source ID `global api-C_UnitAuras-GetCooldownAuraBySpellID-387` promotes to bounded coverage, with one `cooldown-aura-spell-identifiers` capability. **239 pending/109 bounded/14 partial →238 pending/110 bounded/14 partial =362;53→54 capabilities.** Ordered362 IDs,361 unrelated rows,53 prior capabilities, source hashes/register/plaintext are retained. Parent-owned before snapshot: `/tmp/patch-12.0.5-batch47-accounting-before.json`; parent owns postcommit full accounting validation and PLAN. No new tests/builds/checks, delegation or source edits for this acceptance.

Accepted saved proof: **210 unique PASS =14 new +196 controls**, all14 nonzero selections exit0; startup0 `[]`/6.032388805993833s. Compile259.7876708320109s; warm runtime46.811717730015516s below60 target retained without padding as bounded development evidence, not final whole-page, whole-goal or native acceptance. Producer `c6fe6093b4a77335d5f2bbcdfe9e7a62b30aeff5`, inputs `14ee1504b`, fixtures `22c84baf99a173fe81a4e6d431b6b295136f89eb`, GREEN docs `3dba3d8cb`; preserved unowned dirty source means combined proof, not clean-revision proof. Concurrent batch48 mod/state edits are independent and outside this acceptance.

Independent pinned rilua `6044544b960cd68b4b0c58bb3373412757c2caee` security review, existence/substance/wiring and bounded readability PASS combine source review with saved compiled VM-secret/taint execution. Scopedfmt0/0.042104011052288115s and dirtycheck0/22.13509062002413s accepted. Globalfmt1/18.085063977050595s remains an unowned repository formatting failure, not repaired or global success. Four nonblocking advisories deferred: numeric guard naming/extraction, getter naming, rooting helper length, historical fixture comments; no bounded behavioral counterexample. No Rust edits, including comments.

The capability matrix above is accepted only for inferred empty declared-map behavior, strict aliases/public representation, public NUMBER/nil output, mutation/isolation, tainted public calls and conservative secret rejection. Native AllowedWhenTainted, access, result secrecy, catalog/acquisition/direction remain explicitly unchecked. Prior pending checkpoints are historical.

### Retained independent command and provenance record

Copied from accepted independent373; commands were not rerun. Full report also owns pinned-VM/security/wiring/readability analysis and original global-format output. Readability outputs: `/tmp/patch-12.0.5-batch47-independent-readability`; exact invocation `rust-code-analysis-cli -m -p <owned-file> -O json -o /tmp/patch-12.0.5-batch47-independent-readability` for the six owned paths below, all exit0.

## Exact command ledger

All commands cwd repository unless noted. Full revision c6fe6093b4a77335d5f2bbcdfe9e7a62b30aeff5 plus declared preserved unowned dirty source; all execution is combined dirty proof.

| Proof | Exact argv | Exit | Seconds | Full artifacts |
|---|---|---:|---:|---|
| Saved compiled GREEN | `cargo test --test integration --no-run --message-format=json` | 0 | 259.7876708320109 | /tmp/patch-12.0.5-batch47-green-build-result.json; /tmp/patch-12.0.5-batch47-green-build.jsonl; /tmp/patch-12.0.5-batch47-green-build.log |
| Saved run0 ok. 14 passed; 0 failed; 0 ignored; 0 measured; 9514 filtered out; finished in 2.22s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c cooldown_aura_spell_identifiers:: --test-threads=1` | 0 | 2.2547551459865645 | /tmp/patch-12.0.5-batch47-green-run-0.stdout; /tmp/patch-12.0.5-batch47-green-run-0.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run1 ok. 16 passed; 0 failed; 0 ignored; 0 measured; 9512 filtered out; finished in 6.47s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_dispel_color_arguments:: --test-threads=1` | 0 | 6.496058761025779 | /tmp/patch-12.0.5-batch47-green-run-1.stdout; /tmp/patch-12.0.5-batch47-green-run-1.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run2 ok. 12 passed; 0 failed; 0 ignored; 0 measured; 9516 filtered out; finished in 2.24s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c unit_aura_slot_enumeration_arguments:: --test-threads=1` | 0 | 2.263698844006285 | /tmp/patch-12.0.5-batch47-green-run-2.stdout; /tmp/patch-12.0.5-batch47-green-run-2.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run3 ok. 14 passed; 0 failed; 0 ignored; 0 measured; 9514 filtered out; finished in 2.69s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c c_unit_auras_admin:: --test-threads=1` | 0 | 2.710818149964325 | /tmp/patch-12.0.5-batch47-green-run-3.stdout; /tmp/patch-12.0.5-batch47-green-run-3.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run4 ok. 18 passed; 0 failed; 0 ignored; 0 measured; 9510 filtered out; finished in 3.33s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c userdata_proxy::color_curve_ --test-threads=1` | 0 | 3.356312908930704 | /tmp/patch-12.0.5-batch47-green-run-4.stdout; /tmp/patch-12.0.5-batch47-green-run-4.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run5 ok. 12 passed; 0 failed; 0 ignored; 0 measured; 9516 filtered out; finished in 2.48s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c unit_aura_slot_secret_arguments:: --test-threads=1` | 0 | 2.5015637930482626 | /tmp/patch-12.0.5-batch47-green-run-5.stdout; /tmp/patch-12.0.5-batch47-green-run-5.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run6 ok. 14 passed; 0 failed; 0 ignored; 0 measured; 9514 filtered out; finished in 3.85s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_application_display_count:: --test-threads=1` | 0 | 3.873023972962983 | /tmp/patch-12.0.5-batch47-green-run-6.stdout; /tmp/patch-12.0.5-batch47-green-run-6.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run7 ok. 12 passed; 0 failed; 0 ignored; 0 measured; 9516 filtered out; finished in 2.82s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c next125aura:: --test-threads=1` | 0 | 2.8686415660195053 | /tmp/patch-12.0.5-batch47-green-run-7.stdout; /tmp/patch-12.0.5-batch47-green-run-7.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run8 ok. 14 passed; 0 failed; 0 ignored; 0 measured; 9514 filtered out; finished in 3.12s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c unit_aura_filter_query:: --test-threads=1` | 0 | 3.1403976880246773 | /tmp/patch-12.0.5-batch47-green-run-8.stdout; /tmp/patch-12.0.5-batch47-green-run-8.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run9 ok. 7 passed; 0 failed; 0 ignored; 0 measured; 9521 filtered out; finished in 1.54s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_table_shape:: --test-threads=1` | 0 | 1.5626325160264969 | /tmp/patch-12.0.5-batch47-green-run-9.stdout; /tmp/patch-12.0.5-batch47-green-run-9.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run10 ok. 29 passed; 0 failed; 0 ignored; 0 measured; 9499 filtered out; finished in 5.50s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_api:: --test-threads=1` | 0 | 5.521967396955006 | /tmp/patch-12.0.5-batch47-green-run-10.stdout; /tmp/patch-12.0.5-batch47-green-run-10.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run11 ok. 18 passed; 0 failed; 0 ignored; 0 measured; 9510 filtered out; finished in 3.59s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c admin_buff_api:: --test-threads=1` | 0 | 3.6171123179374263 | /tmp/patch-12.0.5-batch47-green-run-11.stdout; /tmp/patch-12.0.5-batch47-green-run-11.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run12 ok. 18 passed; 0 failed; 0 ignored; 0 measured; 9510 filtered out; finished in 3.76s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_refresh_duration:: --test-threads=1` | 0 | 3.778685806086287 | /tmp/patch-12.0.5-batch47-green-run-12.stdout; /tmp/patch-12.0.5-batch47-green-run-12.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved run13 ok. 12 passed; 0 failed; 0 ignored; 0 measured; 9516 filtered out; finished in 2.84s | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c aura_spell_identifier:: --test-threads=1` | 0 | 2.8660488630412146 | /tmp/patch-12.0.5-batch47-green-run-13.stdout; /tmp/patch-12.0.5-batch47-green-run-13.stderr; /tmp/patch-12.0.5-batch47-green-runs.json |
| Saved startup [] | `timeout 90 /syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim --no-addons --no-saved-vars lua-errors` | 0 | 6.032388805993833 | /tmp/patch-12.0.5-batch47-green-startup-run.json; /tmp/patch-12.0.5-batch47-green-startup.stdout; /tmp/patch-12.0.5-batch47-green-startup.stderr |
| Independent gate | `rustfmt --check --config skip_children=true --edition 2024 src/c_api/c_unit_aura_cooldown_spells.rs src/c_api/mod.rs src/lua_api/state.rs src/lua_api/state/sim_state.rs src/lua_api/globals/register.rs tests/cooldown_aura_spell_identifiers.rs` | 0 | 0.042104011052288115 | /tmp/patch-12.0.5-batch47-independent-scoped-fmt.stdout; /tmp/patch-12.0.5-batch47-independent-scoped-fmt.stderr |
| Independent gate | `cargo fmt --check` | 1 | 18.085063977050595 | /tmp/patch-12.0.5-batch47-independent-global-fmt.stdout; /tmp/patch-12.0.5-batch47-independent-global-fmt.stderr |
| Independent gate | `cargo check` | 0 | 22.13509062002413 | /tmp/patch-12.0.5-batch47-independent-check.stdout; /tmp/patch-12.0.5-batch47-independent-check.stderr |

Compiler JSONL 738 records read/decoded; 0 compiler diagnostics. Derived exhaustive run-name ledger: `/tmp/patch-12.0.5-batch47-independent-artifacts.json`. Independent gate ledger: `/tmp/patch-12.0.5-batch47-independent-gates.json`.

### Binary identities

- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/panel-visual-metrics`: `c54af25fdab3933decd9b4825bb6fe8b4a47d3db152b2ae122aca2d0f65ef19d` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/bench_talents`: `85836de318dfbe4f1b078dc6697692a4a2279c12d2869783339affbe98ac6c54` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-sim`: `7c51db5659e2d230f9d6d6deaee60a14d4c30e3e64d833d22c23d63c38229d8f` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/bench_steady_state`: `afaa6b0ca6303d4912b91ead8aaa38f9a45d7085e23530db56aad47508746f46` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/bench_spellbook`: `3e85539d6eea8099d97553b683b8d67e072e49c6dca6bcf3ae14093ef97a5d9b` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/wow-cli`: `956b0e7d6574ce361c38775cb74645927e276461671a9cc7ab9f0c354564f2fe` (current binary independently hashed, matched).
- `/syncthing/Sync/Projects/wow/wow-ui-sim/target/debug/deps/integration-a11e89d240f9bd0c`: `3a433f3fdac41b264281080d3d919cb99679e30b97c714c1a28c8b2126aa4875` (current binary independently hashed, matched).

### Owned source identities

- `src/c_api/c_unit_aura_cooldown_spells.rs`: `70d859613d18eab2042bde18c60224d122cfe316282829381ccd80b5a2e193b5`.
- `src/c_api/mod.rs`: `442d81e4a1e6abe05ba4af6bf9be37c9fd1b423b80fa2846aafc0912ffa3726d`.
- `src/lua_api/state.rs`: `140a1cc2737e3858d17486514031366000a9f0a6c3b1c88db435b8d8022e4af6`.
- `src/lua_api/state/sim_state.rs`: `b685ca05ade8fbaf272e70f4a34da10e5a73977d7bcabf6ced54319b2ee68ebb`.
- `src/lua_api/globals/register.rs`: `d6adaf0277a058c14b477ed877a18694b4c2abb1188a82ee7c5ac3b8d036ec61`.
- `tests/cooldown_aura_spell_identifiers.rs`: `1d4c375269f4870c0ce412c0c0a497121817f11f19b7802be00ac50d2d711f90`.

### Full saved artifact SHA256

- `/tmp/patch-12.0.5-batch47-green-build-result.json` (1724 bytes): `9cf759cfba5658968e0d9af5b114ad36e47a22b52f72fc3a4bc5eeaf34dab008`.
- `/tmp/patch-12.0.5-batch47-green-build.jsonl` (579161 bytes): `f46e0c340a3333fafe56b5b97e2f9d721a4c5f36f896c58265535efa6dd81eb0`.
- `/tmp/patch-12.0.5-batch47-green-build.log` (145 bytes): `393c50f3df581d098d2995ab27178b2f83411a5d21f5911b809f9d9d2c46d637`.
- `/tmp/patch-12.0.5-batch47-green-run-0.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-0.stdout` (1620 bytes): `35b4bbbbc004c5b39ff843e1c9a4596052d423666e3d3ed95c36e6c0dc5994e8`.
- `/tmp/patch-12.0.5-batch47-green-run-1.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-1.stdout` (1900 bytes): `93d4708b17fc74870dcc9b07bed95c1b2b2cfbae745aa57d3751cf740b3ca7d4`.
- `/tmp/patch-12.0.5-batch47-green-run-10.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-10.stdout` (1972 bytes): `d734b33774d4de8d874c744358c16649cc1cb8f506454f84fc9003286fa21139`.
- `/tmp/patch-12.0.5-batch47-green-run-11.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-11.stdout` (1307 bytes): `487b61f79f437003625886ddc9010dd6223fc1530c71ee3a135da2dc14972cca`.
- `/tmp/patch-12.0.5-batch47-green-run-12.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-12.stdout` (1946 bytes): `f83c1ec8e6edfa73306ea53b12622cbad950ac449076efda3a96a26a49100a28`.
- `/tmp/patch-12.0.5-batch47-green-run-13.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-13.stdout` (1364 bytes): `cba48250c3ddfca3d4f44528b74715e7f85fcf58ec9352c61661fbbd3443e4dc`.
- `/tmp/patch-12.0.5-batch47-green-run-2.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-2.stdout` (1647 bytes): `0c7a826d75780ad99df13f14e8164ae712b96d51b2a8a31e1f3d5d4ede2b6088`.
- `/tmp/patch-12.0.5-batch47-green-run-3.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-3.stdout` (1324 bytes): `4b414f323147a7e61f400f043f669000437218677918a025e2f053acb139a09b`.
- `/tmp/patch-12.0.5-batch47-green-run-4.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-4.stdout` (1560 bytes): `a6f3452918ea3a37622cfbb83d57a0150bd188e8085e8c61d3d4313f7cf263a3`.
- `/tmp/patch-12.0.5-batch47-green-run-5.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-5.stdout` (1455 bytes): `0cb311d24a2e74a662e8efc84d63814cba59e44060b0cb56f0f8fd0766dabe02`.
- `/tmp/patch-12.0.5-batch47-green-run-6.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-6.stdout` (1693 bytes): `028d8bcfcab9553bf280314ec91fb793651c7456ff345d5e2a925a9a03431948`.
- `/tmp/patch-12.0.5-batch47-green-run-7.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-7.stdout` (1263 bytes): `d5cdd5460b4c3ee12fca6e726b843931eb78784d265a14e98708dc59e81c94ff`.
- `/tmp/patch-12.0.5-batch47-green-run-8.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-8.stdout` (1554 bytes): `2014adc930fec35a26c268cca6622a115ad88bee31f1824df8a3ad9644812e3a`.
- `/tmp/patch-12.0.5-batch47-green-run-9.stderr` (0 bytes): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `/tmp/patch-12.0.5-batch47-green-run-9.stdout` (667 bytes): `888ed63e386685c31cd10d846a3cd68720c90d37323442c3f2d9054b687daba8`.
- `/tmp/patch-12.0.5-batch47-green-runs.json` (12406 bytes): `ab63def22754f90179a9869bce55e6995904558f1cc68cbbfb781f3ae62515d7`.
- `/tmp/patch-12.0.5-batch47-green-source.diff` (1245 bytes): `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`.
- `/tmp/patch-12.0.5-batch47-green-startup-run.json` (651 bytes): `82a28d05fd9177066f02d87651d0457dfff5872a5dddc5ff08575c1b36b44f32`.
- `/tmp/patch-12.0.5-batch47-green-startup.stderr` (14785 bytes): `d0b5d914bce9925319109fa020b109d385c5e36ef1df7024c32dde445f10edf6`.
- `/tmp/patch-12.0.5-batch47-green-startup.stdout` (3 bytes): `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570`.
