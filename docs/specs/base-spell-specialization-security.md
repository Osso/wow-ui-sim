# Retail base-spell specialization security

Exact source row295 (`global api-C_Spell-GetBaseSpell-295`) adds **arg2 NeverSecret** to `C_Spell.GetBaseSpell`. Batch63 now publishes the existing meaningful `BaseSpellRelationships` provider for Retail 12.0.5+ and adds the specialization boundary after main's actual compiled RED. No stub or fabricated catalog. Producer GREEN and acceptance remain pending. See [historical Forever contract](spell-base.md) and [Lua API architecture](../lua-api.md).

## What it must do

### Retail publication and existing model behavior

- [ ] Publish the actual `C_Spell.GetBaseSpell` in the rooted Retail 12.0.5+ namespace using the existing relationship model; keep Forever publication and existing Forever-specific helpers/tests unchanged.
- [ ] Resolve numeric `19750`, known `Flash of Light`, case-insensitive names and numeric strings through the existing resolver. Do not change arg1 policy or introduce new alias semantics.
- [ ] With test-owned `(66,19750)->642`, `(70,19750)->853` and Paladin active index2, omitted/nil/zero public specialization selects current66; explicit70 selects853. Explicit66 selects642.
- [ ] Return supplied positive spell ID when no relationship exists, including unknown `4294967295`. This is the existing model's identity contract, **not fallback compatibility**. Positive finite integral u32 specialization IDs are accepted; malformed public types/ranges retain meaningful validation. These selection/validation policies are inferred, not native-verified.
- [ ] Observe live active-specialization and relationship-input updates; isolate environments. Queries must not mutate relationships, aliases, player selection, public tables or frames, or infer relationships by reversing aliases.
- [ ] Preserve addon stack taint on ordinary public calls and restore the outer secure caller after returning. No blanket security declassification.

### Exact row295 boundary

- [ ] Authenticate arg2 with actual VM secret metadata before arg1 validation/resolution, specialization validation/default selection or relationship lookup.
- [ ] Reject authentic secret NUM/NIL/BOOL/STRING/table/real-frame arg2 in secure and tainted callers. Secret NIL is not public nil; secret BOOL must not be compared in tainted Lua.
- [ ] Deny secret arg2 for valid numeric/known-name spell inputs and unknown-relationship inputs, and independently for invalid public arg1 `false`, `nil`, `'unknown'`. Errors must identify this API and second-argument context without disclosing private payloads; no exact native error wording is specified.
- [ ] Leave omitted/nil/zero public arg2 and existing positive-ID validation unchanged. Preserve wrapper secrecy, rooted host identity/allocation metadata and public recovery after rejection and forced GC.

## How it works

- [Lua API architecture](../lua-api.md)
- [Historical public base-spell contract and limitations](spell-base.md)

## Implementation inventory

**Shared inputs and batch63 producer:**

- `Cargo.toml` — small named `base-spell-relationships=[]` capability selected by `retail-12-0-5` and `client-wowforever`. Shares the current model between these existing consumers without repeated profile lists; not optional extensibility.
- `src/c_api/mod.rs` — model module compiled under that capability.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — same capability gates the existing per-environment field/default, empty initially.
- `src/c_api/spell_base.rs` — unchanged explicit model, identifier resolver and public validation. A `retail-12-0-5`-only early guard calls `rilua::table_security::is_secret_value(state, stack_val(state, 2))` before reading arg1 or consulting simulator state. It rejects even secure callers with `C_Spell.GetBaseSpell argument 2 is NeverSecret`; no payload interpolation, unwrap, callback clearing or fabricated metadata.
- `src/c_api/c_spell.rs` — existing `SPELL_QUERY_METHODS` entry now uses `base-spell-relationships`, publishing the real handler through the existing rooted namespace registration. No namespace replacement, new fallback or stub.

Forever retains existing public behavior and ordered validation/secret-argument errors: its default features do not select `retail-12-0-5`, so the early guard is absent. Other profiles and earlier Retail epochs remain default-unpublished by the inverse feature gate. This is static wiring evidence, not executed profile proof; manually enabling the shared capability is not default-profile certification.

Actual RED compilation reported five `dead_code` warnings: `public_argument`, `valid_id`, `read_spell_id`, `read_requested_spec`, `get_base_spell`. Root cause: compiled shared model without Retail registration. The real query entry now consumes the handler/helper chain; no suppression, visibility widening or aliases. Warning-free compilation remains unchecked.

## Tests asserting this spec

`tests/spell_base_retail_security.rs` is discovered by the existing grouped integration harness, gated by `retail-12-0-5` and a Retail/PTR profile. No new Cargo test target. Twenty substantive tests call the real namespace; no fake API callbacks or implementation-shape assertions.

| Contract slice | Concrete coverage | Proof level |
|---|---|---|
| Missing Retail publication/model | Empty-input identity, numeric seeded mapping, name/case/numeric-string, omitted/nil/0, explicit66/70, positive u32 unknown-spec identity | Six compiled RED failures at public-result boundary; downstream outcomes pending GREEN |
| Live inputs/isolation/read-only | Active-index update, relationship replacement, two environments, numeric alias independence, state/frame/table preservation | Five compiled RED failures at public-result boundary; downstream state assertions pending GREEN |
| Existing public validation | Invalid spell IDs/types/nil/name; invalid spec types/ranges including NaN/infinities/u32 overflow | Two compiled RED failures: invalid public inputs accepted |
| Authenticated arg2 denial | NUM, NIL, both BOOLs, STRING, table/real frame; valid/current/unknown spell cases; separate invalid-arg1 precedence | Six compiled RED failures at `secret spec denied`; context/precedence assertions pending GREEN |
| Lifetime/recovery | Lua-rooted wrappers, host allocation identity/secrecy, GC between rejection and public recovery | One compiled RED failure at `secret spec denied`; GC/recovery/metadata assertions pending GREEN |

All public/security probes run secure and addon-tainted contexts. Security fixtures retain wrapper roots in Lua and verify identities via host metadata, never secret BOOL equality in tainted Lua. Test relationships are deliberately configured data, not live mappings. Existing `tests/spell_base.rs` and its historical three Forever tests remain untouched.

### Proof ledger / handoff

- Input scaffold `2cd4a3fe3` starts from `f97d2c8edde8987edc9b06329878b0f1aca6d174`; Forever tests/helpers remain unchanged. Batch63 owns only `src/c_api/c_spell.rs`, `src/c_api/spell_base.rs` and these two specs; no test/input/state/catalog changes.
- Actual compiled RED at `a920153ef827bab420aba1c4facad994b35691ba`: `/tmp/patch-12.0.5-batch63-red-build-result.json` and `.stdout.jsonl` record default `cargo test --test integration --no-run --message-format=json`, exit0, **351.8558507799171s**, five warnings above. Provenance is dirty-combined; protected dirty hash was supplied, not recomputed.
- `/tmp/patch-12.0.5-batch63-red-run.json`, `.stdout` and `.stderr` record `timeout 90 target/debug/deps/integration-d36739a1204f0133 spell_base_retail_security:: --nocapture --test-threads=1`, exit101, **3.9593256909865886s** wall time (harness3.47s), **0 PASS / 20 FAIL**. Integration SHA256: `99af35e6ac927ebe49fad0e698c63972f6e6f70786dd66c0deaaaae7cf6aebee`.
- Public-result tests reach `public numeric base` after the single-result assertion; validation tests accept invalid inputs; security tests reach `secret spec denied`. Meaningful missing-publication RED, not twenty independent relationship/taint/GC outcomes. Downstream assertions remain pending until GREEN. Fixtures configure the actual model and real VM wrappers; no fixture correction is justified.
- Producer formatting is limited to `rustfmt --edition 2024 --config skip_children=true src/c_api/c_spell.rs src/c_api/spell_base.rs`. Main owns build/test/check/lint/readability/independent gates and ops; none is run by this producer task. No producer GREEN or acceptance credit.
- The shared Cargo feature changes whole-tree compilation scope: prior497 whole-tree check is invalidated outside its item-context owned scope; exact `23ef` item-context proof remains unaffected. This is the supplied scope classification, not a check performed here.
- Accounting unchanged: **199 pending / 148 bounded / 14 partial / 1 metadata = 362 IDs; 68 capabilities**. Row295 remains pending. Rows330/331 remain independently pending, not part of the prior63 capability set; no tooltip/context work here.

## Saved parent Retail GREEN — 2026-10-02

Producer `815456e844fbb8e4cbfa5eb39e1d199733616101` compiled default integration successfully in157.61813914496452s with **zero compiler-message diagnostics**; the five scaffold unused-helper warnings disappear through actual registration, without suppressions. Full compiler JSON/stderr and executable hashes: `/tmp/patch-12.0.5-batch63-green-build.stdout.jsonl`, `.stderr`, `-result.json`. Integration SHA256 `15c62c858697dd81cab48ec5b770c88c4441bf7aa87d4fcd24b64724990f7c05` binds saved runs.

`batch63-green-runs.json` records **81 distinct PASS**:20 focused,8 glyph-security,22 tooltip spell/mount,17 action-slot identifier and14 existing spell/flyout controls. Six commands exit0; `c_spell::` matched zero and gives no proof. Actual14 controls ran under `c_spell_flyout_probes::`. Runtime17.516695430967957s is separate from compilation, below60s bounded-development target, not padded whole-goal acceptance. Startup separately returned `[]`, exit0; `green-startup-run.json` records exact cost/hash/full outputs.

Evidence remains dirty-combined. Independent502 source/security/wiring/readability/scoped formatting/default check and separate Forever compile/three existing tests are pending; no current Forever runtime or exact295 credit claimed. Native arg1 permissions/relationship acquisition/other profiles/UI remain unproved. Separate batch62 acceptance now197 pending/150 bounded/14 partial/1 metadata,362 IDs/69 capabilities; row295 stays pending. Do not rerun applicable Retail build/tests/startup solely for docs/accounting.

## Known gaps (current cycle)

- [x] Main-owned actual compiled RED classified and warning root cause recorded; bounded producer implemented afterward.
- [x] Parent observed focused Retail GREEN, controls/startup and warning-free default compilation as recorded above.
- [ ] Main-owned startup, check, Forever/profile regression proof, readability and independent verification.
- [ ] Requirement acceptance and exact source295 accounting; requirements above remain unchecked pending proof.
- [ ] Native Retail publication/profile permissions, explicit specialization interpretation, result secrecy and live relationship acquisition remain unknown. No retained-surface/native availability certification.
- [ ] Arg1 `AllowedWhenTainted` semantics remain unmodeled: existing conservative secret-arg1 rejection is acknowledged, unchanged and earns no annotation credit.
- [ ] No real-world relationship dataset is populated; no fabricated production data, alias traversal or substitute catalog.

### Deferred native probes — not a completion gate

No client observed for these cases. Under an **actually configured override** (if available), record `GetBaseSpell(19750)` with spec omitted, public nil/0, current66 and explicit70; record actual relationship values rather than assuming fixture642/853. Probe a verified unknown relationship for identity. Record class/spec/build/context and observed results; do not invent live mappings.

Separately probe secure/tainted authentic secret arg2 including NIL/NUM and other supported payload kinds, with valid/current/unknown spell and invalid public arg1 `false`/`nil`/`'unknown'` to establish precedence. Secret arg1 `AllowedWhenTainted` requires a distinct probe; its native behavior cannot be credited from arg2 denial. Native error text, permissions and secrecy are unknown until observed.

## Out of scope

- No arg1-policy or existing provider-helper refactor; no test changes. Actual compiled RED precedes this producer.
- Maw Power APIs, row311 rework, tooltip rows330/331 or any other patch row; no accounting promotion.
- New relationship/catalog data, blanket declassification, VM extensions, native-client execution or earlier-profile execution claims.
