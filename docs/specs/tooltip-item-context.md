# Item tooltip contexts

Retail12.0.5 exact source additions330/331 add nullable numeric `itemContext` (arg3) and `treasureContextLevel` (arg4) to `C_TooltipInfo.GetItemByID`. This slice supplies **inputs and unexecuted tests only**. No getter, handler, provider or registration implementation is present. See [Lua API architecture](../lua-api.md) and [tooltip identifier boundary](tooltip-spell-mount-identifiers.md).

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
- `src/c_api/mod.rs`: epoch125 module and root type export only; no registration change.
- `src/lua_api/state/sim_state.rs`: epoch125 public optional host override map only.
- `src/lua_api/state.rs`: empty map initialization only.
- Existing `src/lua_api/globals/missing_surface/tooltip_info/probes.rs::c_tooltip_get_item_by_id`: unchanged, reads arg1 u32 and ignores args2–4.
- Existing `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: unchanged meaningful finite-catalog builder; no context provider yet.

## Tests asserting this spec

`tests/tooltip_item_context.rs`: **24 substantive real-API tests**, discovered by existing grouped integration harness; no new Cargo target. Base controls, independent context/treasure outputs, nil/default override, nil-versus-zero, holes/misses, seeded unknown ID, strict domain, ignored quality, read-only inputs, freshness/mutation, live replacement/clear, isolation, public-tainted calls, secure authentic NUM/NIL/quality, secure invalid contexts, all-four-position denial ordering and rooted GC/recovery.

Positive context/security probes demand meaningful variants; a denial cannot substitute for producer proof. No query/callback is replaced. Expected stat output follows existing armor budget rounding with fixed fixture percentages, not a fake query. DTO comparisons normalize only level/stat texts to compare every unchanged field semantically.

## Known gaps (current cycle)

- [ ] Main owns compilation and actual runtime RED classification. No builds, tests, checks, lint, readability, broad gates or delegation were performed for this scaffold. Compiler errors are **not behavioral RED**; passing base controls do not prove new provider behavior.
- [ ] No getter/provider/handler/registration work may begin before main establishes actual compiled behavioral RED. Inputs alone do not resolve source330/331; all requirements remain unchecked pending proof.
- [ ] Independent meaningful-provider proof and authenticated boundary proof must precede any credit for the two exact source additions. Native Requires/output-policy credit is excluded.
- [ ] User-supplied accounting remains205 pending/142 bounded/14 partial/1 metadata-only,362 IDs and67 capabilities. Only330/331 are candidates in this slice; six aura rows remain independently pending. No accounting/catalog/coverage promotion accompanies scaffold creation.
- [ ] Native lookup, miss, numeric-domain, override, permission, error and secrecy semantics remain unknown. A trustworthy native production mapping is **not a prerequisite or blocker** for the authorized meaningful inferred model: production map stays empty; tests supply explicit fixture data. The contrary prerequisite in `/tmp/patch-12.0.5-item-context-provider-boundary.md` is rejected as a goal constraint, while its observed missing provider/catalog facts remain useful.

## Out of scope

- Production getter/provider/handler/registration implementation in this slice; main owns work after compiled RED.
- New/generated catalog data, context arithmetic, ItemID substitution, bonus/upgrade/scaling/acquisition models, prompt-state coupling, equipment variants or quality semantics.
- Whole-profile/UI/frame tooltip integration, all variant/stat parity, native probe prerequisites and full Requires/output-secrecy parity; broader goal remains open.
- Protected `src/c_api/aura_duration.rs` inspection/modification/staging, vendor edits, operations, deployment, push or broad gates.
