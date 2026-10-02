# Retail base-spell specialization security

Exact source row295 (`global api-C_Spell-GetBaseSpell-295`) adds **arg2 NeverSecret** to `C_Spell.GetBaseSpell`. Retail currently lacks the simulator publication; the approved missing-Retail-API scope requires the existing meaningful `BaseSpellRelationships` provider, not a stub or fabricated catalog. This commit promotes only model inputs and adds pending tests. Publication and the security producer follow main's actual compiled RED. See [historical Forever contract](spell-base.md) and [Lua API architecture](../lua-api.md).

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

**Input scaffold now:**

- `Cargo.toml` — small named `base-spell-relationships=[]` capability selected by `retail-12-0-5` and `client-wowforever`. Shares the current model between these existing consumers without repeated profile lists; not optional extensibility.
- `src/c_api/mod.rs` — model module compiled under that capability.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — same capability gates the existing per-environment field/default, empty initially.
- `src/c_api/spell_base.rs` — existing model/handler and argument logic **unchanged**.
- `src/c_api/c_spell.rs` — existing Forever-only registration **unchanged**; Retail runtime publication is still missing.

**Planned only after actual RED:** publish the existing handler for the applicable Retail epoch and add the exact arg2 authentication/denial boundary before unchanged arg1 handling. No producer implementation belongs to this scaffold commit.

Other profiles and earlier Retail epochs remain default-unpublished; feature wiring is not earlier-profile or native execution proof. The shared feature can expose unused-handler/helper warnings in Retail until registration consumes them. Record actual warnings during main's compilation; do not suppress them or publish early to hide them.

## Tests asserting this spec

`tests/spell_base_retail_security.rs` is discovered by the existing grouped integration harness, gated by `retail-12-0-5` and a Retail/PTR profile. No new Cargo test target. Twenty substantive tests call the real namespace; no fake API callbacks or implementation-shape assertions.

| Contract slice | Concrete coverage | Proof level |
|---|---|---|
| Missing Retail publication/model | Empty-input identity, numeric seeded mapping, name/case/numeric-string, omitted/nil/0, explicit66/70, positive u32 unknown-spec identity | Six tests authored; uncompiled/unrun |
| Live inputs/isolation/read-only | Active-index update, relationship replacement, two environments, numeric alias independence, state/frame/table preservation | Five tests authored; uncompiled/unrun |
| Existing public validation | Invalid spell IDs/types/nil/name; invalid spec types/ranges including NaN/infinities/u32 overflow | Two tests authored; uncompiled/unrun |
| Authenticated arg2 denial | NUM, NIL, both BOOLs, STRING, table/real frame; valid/current/unknown spell cases; separate invalid-arg1 precedence | Six tests authored; uncompiled/unrun |
| Lifetime/recovery | Lua-rooted wrappers, host allocation identity/secrecy, GC between rejection and public recovery | One test authored; uncompiled/unrun |

All public/security probes run secure and addon-tainted contexts. Security fixtures retain wrapper roots in Lua and verify identities via host metadata, never secret BOOL equality in tainted Lua. Test relationships are deliberately configured data, not live mappings. Existing `tests/spell_base.rs` and its historical three Forever tests remain untouched.

### Proof ledger / handoff

- Scaffold starts from `f97d2c8edde8987edc9b06329878b0f1aca6d174`; only input gates, this test file and two specs are owned.
- No build/test/check/lint/readability/delegation/ops/push is authorized here. Owned Rust formatting command: `rustfmt --edition 2024 --config skip_children=true tests/spell_base_retail_security.rs src/c_api/mod.rs src/lua_api/state.rs src/lua_api/state/sim_state.rs` exited0; the subsequently edited test file is formatted again before commit. This proves formatting only; child traversal is disabled. No compiled RED, GREEN, acceptance or coverage credit is claimed.
- Main must classify actual compiled failures before producer work. Early default-result/nil failures indicate **missing Retail publication**, not a missing relationship model. Later denial/precedence/lifetime assertions may remain unreached; do not credit them from the first failure.
- The shared Cargo feature changes whole-tree compilation scope: prior497 whole-tree check is invalidated outside its item-context owned scope; exact `23ef` item-context proof remains unaffected. This is the supplied scope classification, not a check performed here.
- Accounting unchanged: **199 pending / 148 bounded / 14 partial / 1 metadata = 362 IDs; 68 capabilities**. Row295 remains pending. Rows330/331 remain independently pending, not part of the prior63 capability set; no tooltip/context work here.

## Known gaps (current cycle)

- [ ] Main-owned actual compiled RED and warning ledger; subsequent producer and bounded behavioral acceptance.
- [ ] Native Retail publication/profile permissions, explicit specialization interpretation, result secrecy and live relationship acquisition remain unknown. No retained-surface/native availability certification.
- [ ] Arg1 `AllowedWhenTainted` semantics remain unmodeled: existing conservative secret-arg1 rejection is acknowledged, unchanged and earns no annotation credit.
- [ ] No real-world relationship dataset is populated; no fabricated production data, alias traversal or substitute catalog.

### Deferred native probes — not a completion gate

No client observed for these cases. Under an **actually configured override** (if available), record `GetBaseSpell(19750)` with spec omitted, public nil/0, current66 and explicit70; record actual relationship values rather than assuming fixture642/853. Probe a verified unknown relationship for identity. Record class/spec/build/context and observed results; do not invent live mappings.

Separately probe secure/tainted authentic secret arg2 including NIL/NUM and other supported payload kinds, with valid/current/unknown spell and invalid public arg1 `false`/`nil`/`'unknown'` to establish precedence. Secret arg1 `AllowedWhenTainted` requires a distinct probe; its native behavior cannot be credited from arg2 denial. Native error text, permissions and secrecy are unknown until observed.

## Out of scope

- No runtime publication/security producer before actual RED; no handler or arg1-policy changes in this commit.
- Maw Power APIs, row311 rework, tooltip rows330/331 or any other patch row; no accounting promotion.
- New relationship/catalog data, blanket declassification, VM extensions, native-client execution or earlier-profile execution claims.
