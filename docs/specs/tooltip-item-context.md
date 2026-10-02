# Item tooltip contexts

Retail12.0.5 exact source additions330/331 add nullable numeric `itemContext` (arg3) and `treasureContextLevel` (arg4) to `C_TooltipInfo.GetItemByID`. This slice implements an **inferred exact-key item-level producer** after compiled behavioral RED at `11d21cd28`. Producer GREEN and independent acceptance remain pending; no native variant acquisition or source credit is claimed. See [Lua API architecture](../lua-api.md) and [tooltip identifier boundary](tooltip-spell-mount-identifiers.md).

## What it must do

### Documented boundary

Cached retail `AddOns/Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:439–456` declares required numeric `itemID`, nullable numeric `quality`, nullable numeric `itemContext`, nullable numeric `treasureContextLevel`, function-wide `SecretArguments = "AllowedWhenUntainted"`, `MayReturnNothing = true`, and nonnullable `TooltipData` return. There is **no `NeverSecret`** declaration. Documentation establishes signature/annotation, not context lookup, numeric domain, native errors, acquisition or output secrecy.

- [ ] Consume both added inputs meaningfully: independently varying context or treasure level selects distinct supplied item-level variants for the **same known catalog item**. Annotation guards alone earn neither source330 nor source331 credit.
- [ ] Authenticate all four arguments using authentic VM `unwrap_secret` AllowedWhenUntainted behavior **before any type validation or model lookup**. Never clear caller taint, replace callbacks, or authorize through a fake query.
- [ ] Secure authentic secret numeric item/context/treasure inputs select meaningful variants; tainted authentic secrets in each position are denied before invalid public item/context types or catalog/context misses.
- [ ] Secure secret NIL optionals follow the chosen nil behavior; tainted secret NIL is denied. Secure secret BOOL/STRING/table/frame contexts reach type errors; tainted equivalents reach VM denial without private payload leakage and with exact API namespace in errors.
- [ ] Secure secret quality is authenticated then ignored, including nonnumeric payloads; tainted secret quality is denied. Preserve ordinary ignored quality behavior without introducing quality validation, public type policy or output override.
- [ ] Ordinary public tainted inputs retain exact context selection and caller taint. Preserve secure outer trust through denial/recovery and rooted secret wrapper identity, allocation and metadata across GC. BOOL identity is inspected only through host metadata, never tainted payload equality.

### Chosen bounded model — explicitly inferred, not native-verified

These policies are authorized informed guesses. They are not claims about native Requires permissions, native nil/domain rules or native return secrecy.

- [ ] Epoch125 exposes `ItemTooltipContext { item_id: u32, item_context: Option<u32>, treasure_context_level: Option<u32> }` and environment-local `SimState.item_tooltip_levels: HashMap<ItemTooltipContext, u16>` with an empty default. No production data is fabricated.
- [ ] Nil and zero remain distinct in both optional key positions. Context/treasure inputs accept only nil or finite integral u32 numbers; reject coercion, negative, fractional, nonfinite and overflowing values. Preserve existing arg1 u32 positive/zero behavior rather than creating a new itemID policy.
- [ ] Both context positions nil/omitted use the existing base catalog exactly unless an explicit `(itemID,None,None)` override exists. Any present context/treasure requires an exact supplied key; absent combinations return a fresh empty Item DTO. No cross-context, nearest-level, scaling or default-catalog fallback for explicit misses.
- [ ] Missing catalog IDs remain empty even when the override map contains their key. Never manufacture a catalog item, name, slot or stat budget from map entries.
- [ ] For known items, clone catalog `ItemInfo` ephemerally and change **only `item_level`**. Existing builder supplies item-level and estimated stat lines, retaining name, quality/color, slot, binding, budgets and every other DTO value. Full DTO comparisons use semantic RGBA, not color-method identity.
- [ ] Distinct contexts1/2 and treasure levels70/80 select explicit test levels601/602/603/604. These numbers are test data only; contexts are not arithmetic inputs, replacement item IDs or acquisition metadata.
- [ ] Query reads leave map inputs, source catalog, player-class input and caller objects unchanged. Fresh DTOs/lines/colors and empty misses are independent; result mutation cannot affect inputs or other results. Live host replace/clear affects later reads only; environments remain isolated.

Known catalog fixture211995 is `Entombed Seraph's Sabatons`, base level571, quality4, Feet, bind-on-pickup, with existing Strength/Stamina/Mastery/Versatility budget estimates. Tests establish the base through public `wow_ui_sim::items::get_item`, then assert concrete names, line types, slot, binding, levels, colors and rounded stat values. The existing finite catalog and stat heuristic are bounded simulator behavior, **not native variant acquisition or complete item stat parity**.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame/model data flow](../frame-data-flow.md)

## Implementation inventory

- `src/c_api/tooltip_item_context.rs`: input key only, with Debug/Clone/Copy/PartialEq/Eq/Hash.
- `src/c_api/mod.rs`: epoch125 key export and first-class `c_tooltip_info_item_context` module declaration.
- `src/lua_api/state/sim_state.rs`: epoch125 public optional host override map only.
- `src/lua_api/state.rs`: empty map initialization only.
- `src/c_api/c_tooltip_info_item_context.rs`: authentic VM authentication of all four positions before parsing/model access; quality authenticated then ignored; exact finite-u32 parsing; copied exact host-map level; unknown catalog rejection and ephemeral catalog clone changing only `item_level`.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: single epoch125 publication into the existing globally rooted namespace after `ensure_namespace`; narrow model-result bridge to the existing builder. Legacy entry inversely gated.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs::c_tooltip_get_item_by_id`: inversely gated to earlier epochs; no active fallback. Shared builder import remains used by other probes.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: shared `tooltip_for_item_info` extraction reuses unchanged `populate_item_tooltip_lines`; no rendered DTO patch, replacement ID or fake line. Base queries and selected clones use the same construction path.

## Tests asserting this spec

`tests/tooltip_item_context.rs`: **24 substantive real-API tests**, discovered by existing grouped integration harness; no new Cargo target. Base controls, independent context/treasure outputs, nil/default override, nil-versus-zero, holes/misses, seeded unknown ID, strict domain, ignored quality, read-only inputs, freshness/mutation, live replacement/clear, isolation, public-tainted calls, secure authentic NUM/NIL/quality, secure invalid contexts, all-four-position denial ordering and rooted GC/recovery.

Positive context/security probes demand meaningful variants; a denial cannot substitute for producer proof. No query/callback is replaced. Expected stat output follows existing armor budget rounding with fixed fixture percentages, not a fake query. DTO comparisons normalize only level/stat texts to compare every unchanged field semantically.

## Recorded compiled RED — 2026-10-02

Proof ledger: input revision `11d21cd28b1d14339825bd14530c3051d7e08f2c`; saved `/tmp/patch-12.0.5-batch62-red-build-result.json` records integration compilation exit0 in287.12964176607784s including an unknown lock wait. Dirty-combined provenance, not a clean-revision build claim. Integration binary SHA256 `a74cab6137b7438e303d99a42da98c44d28f41ee6bc301f0864b5aa4228f60e3`.

`/tmp/patch-12.0.5-batch62-red-run.json` and companion `.stdout`/`.stderr` record `tooltip_item_context:: --nocapture --test-threads=1` under timeout90: **24 tests,2PASS/22FAIL, exit101,4.043648569029756s** wall time (harness reports3.62s). This is pre-producer evidence only; this producer change invalidates it as current execution proof.

Authoritative `PARENTFALSIFICATION` correction at the bottom of `/tmp/patch-12.0.5-item-context-red-diagnosis.md` rejects the preceding agent's seven-line/shared-setup-failure claim. Every failing test reaches probe line243, not setup line183. Shared `ITBase` setup passes. Full-startup base diagnostic shows eight lines: name,level571,Feet,binding,+721Strength,+1622Stamina,+411Mastery,+189Versatility. No fixture correction is justified or made.

| Boundary | Recorded RED evidence | Limit |
| --- | --- | --- |
| Existing base/u32/unknown ID | Two named controls PASS; shared base setup passes throughout | Not provider credit |
| Context/treasure selection | Supplied601–604 expectations receive unchanged571 | Exact inferred variants need GREEN |
| Explicit unmapped combinations | Default/base assertions pass before intentional `ITMiss` fails | Not a baseline fixture failure |
| Domain/wrapper/API boundary | Context rejection absent; secure secret ID gets userdata error; denial lacks API namespace | Not full downstream authentication proof |
| Freshness/recovery/GC | Tests stop at earlier variant/wrapper boundaries | Unreached assertions cannot be claimed |

## Saved parent GREEN — 2026-10-02

Producer `23efb40c84837dfa9564a7d21bc6297842ee4df3` compiled successfully with default integration in231.14114932902157s, including any unmeasured lock cost. `/tmp/patch-12.0.5-batch62-green-build.stdout.jsonl`, `.stderr` and `-result.json` retain full compiler output and executable hashes. Integration SHA256 `e5a498552708b25f6e8eef7776fd3a475798ebf95be71c88e93ca197f32d31db` binds four finite runs in `batch62-green-runs.json`.

**177 distinct PASS**:24 focused,123 tooltip controls,24 aura-instance controls and6 item-source controls; four exits0, no duplicate names. Execution47.36223625706043s is separate from compilation, below60s partition target and not a padded final whole-goal run. Startup separately returned `[]`, exit0; `green-startup-run.json` binds exact time/hash and full stdout/stderr. Proof remains dirty-combined, protected hash supplied rather than recomputed; no clean-revision/native/profile/full-page claim.

Independent security/wiring/readability/scoped formatting/default check and exact330/331 acceptance remain pending. Saved24 focused tests reach distinct context/treasure payloads and downstream authentication/GC/recovery assertions; initial RED still cannot claim those downstream outcomes. Do not rerun applicable build/runtime solely for docs/accounting.

## Readability follow-up — 2026-10-02

Independent497 identified two concrete new-test compound conditions: the five-part RGBA expectation and failed/type/nonempty rejection assertion. Parent source inspection accepts both; ordered separate assertions preserve identical predicates and short-circuit/type-before-length boundaries. Producer and host inputs remain unchanged. Refreshed24 focused runtime and independent equivalence/readability/format proof remain pending;153 controls/startup and production security/wiring/check remain source-valid, not fresh reruns. No native parity or accounting credit follows from readability edits.

## Known gaps (current cycle)

- [x] Main-owned compiled RED, producer, focused GREEN, adjacent controls and startup recorded; producer agent ran no gates.
- [ ] Independent scoped security/wiring/readability/Rust gates and exact source acceptance remain pending. Requirements stay unchecked until independent acceptance.
- [ ] Independent meaningful-provider proof and authenticated boundary proof must precede any credit for the two exact source additions. Native Requires/output-policy credit is excluded.
- [ ] Current user-supplied accounting remains199 pending/148 bounded/14 partial/1 metadata-only,362 IDs and68 capabilities. Only330/331 are potential candidates after independent meaningful-provider proof, never merely annotation guards. No accounting/catalog/coverage promotion accompanies producer creation; batch61 remains untouched.
- [ ] Native lookup, miss, numeric-domain, override, permission, error and secrecy semantics remain unknown. A trustworthy native production mapping is **not a prerequisite or blocker** for the authorized meaningful inferred model: production map stays empty; tests supply explicit fixture data. The contrary prerequisite in `/tmp/patch-12.0.5-item-context-provider-boundary.md` is rejected as a goal constraint, while its observed missing provider/catalog facts remain useful.

## Out of scope

- New/generated catalog data, context arithmetic, ItemID substitution, bonus/upgrade/scaling/acquisition models, prompt-state coupling, equipment variants or quality semantics.
- Whole-profile/UI/frame tooltip integration, all variant/stat parity, native probe prerequisites and full Requires/output-secrecy parity; broader goal remains open.
- Protected `src/c_api/aura_duration.rs` inspection/modification/staging, vendor edits, operations, deployment, push or broad gates.
