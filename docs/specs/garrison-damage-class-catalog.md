# Garrison damage-class catalog

`C_Garrison.GetAutoCombatDamageClassValues()` supplies simulator-defined enUS school labels to the cached `Blizzard_GarrisonUI/Mainline/Blizzard_AdventuresCombatLog.lua` consumer. `AdventuresCombatLogMixin:OnLoad` maps each row's `damageClassValue` to `locString`; `AddCombatEvent` looks up that map using `spellInfo.schoolMask`. Coherent addon-compatible behavior follows the [acceptance SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#acceptance-authority--2026-10-11); native parity is not an acceptance blocker.

## What it must do

- [x] Return exactly one table containing an `ipairs`-iterable catalog of rows with numeric `damageClassValue` school-mask keys and nonempty readable string `locString` labels; keys must be unambiguous.
- [x] Include pure school masks `1=Physical`, `2=Holy`, `4=Fire`, `8=Nature`, `16=Frost`, `32=Shadow`, `64=Arcane` under simulator enUS policy. These are masks, not damage-class ordinals; seven entries are required coverage, not an exact row-count ceiling.
- [x] Return a fresh outer table and independent fresh row tables on every call. Mutating or removing returned rows or adding fields must not affect another existing result or later queries.
- [x] Exercise the authentic cached addon `OnLoad` → `AddCombatEvent` path with the real catalog query intact. Observable combat-log entries must distinguish mask `4` as Fire from mask `2` as Holy; UI wiring and supplied spell metadata may be test collaborators.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Client profiles and profile-scoped runtime sources](../wiki/systems/client-profiles.md)

## Implementation inventory

- Profile cache `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GarrisonUI/Mainline/Blizzard_AdventuresCombatLog.lua`: authentic row-to-mask map and combat-event label consumer; not simulator-owned code.
- Profile cache `Blizzard_APIDocumentationGenerated/GarrisonInfoDocumentation.lua`: declares the catalog query and row fields, not native catalog contents or ordering.
- `src/c_api/c_garrison_damage_classes.rs`: immutable simulator enUS labels with fresh outer/row tables, registered by the canonical C API utility bootstrap. No native catalog claim.
- `src/lua_api/globals/font_strings_collection/colors.rs`: four engine color globals required by the authentic combat-log UI, supplied by the normal named-color bootstrap. Shared simulator palette copies retained Mists5.5.4.70268 GlobalColor rows86–89: GREY0xffb9b9b9, BLUE0xff4eadd7, ORANGE0xffc6644b, YELLOW0xffffd200, interpreted literally as AARRGGBB. This is not current Retail-native RGB/RGBA parity or an addon Lua override.
- `tests/garrison_damage_classes.rs`: simulator enUS catalog, result-isolation, concrete color-bootstrap RGB and authentic addon-path assertions.

## Tests asserting this spec

`tests/garrison_damage_classes.rs` exposes the catalog, isolation and RGB-bootstrap probes under every profile configuration. Only the authentic cached-addon consumer remains Retail-gated. This supports pure simulator-model verification without acquiring blocked vendor inputs; it is not evidence of native API publication across profiles. [Pure-profile matrix](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#shared-querybootstrap-pure-profiles--partial-2026-10-11) admits the three pure cases under all seven configurations, including independently authenticated Forever. This does not credit native API publication or loaded-addon behavior outside the separate Retail consumer proof. Retail proof is retained below; cfg-only gating preserves its four test bodies.

Tests:

- `garrison_damage_classes_returns_one_table_with_en_us_pure_school_masks`
- `garrison_damage_classes_returns_fresh_tables_and_rows_without_mutation_leaks`
- `garrison_adventures_combat_log_consumes_fire_school_mask_not_damage_ordinal`
- `adventures_combat_log_colors_bootstrap_with_mists_row_rgb_palette`

Initial test/spec slice was unexecuted; subsequent proof is revision-scoped below. Catalog requirements are checked against b58020420's bounded passes, not current consumer completion. Initial joint default attempt `20261011T165520Z` was COMPILE_FAILED with zero executed cases: a duplicate `common` module exported four macros twice, not query RED. Commit `7635fef91` reuses the root integration harness via `use crate::common`; this import repair establishes no catalog/runtime PASS. Retry epoch173738 at `a0833b2dd` compiles successfully and executes three unique cases:0PASS/3FAIL, exit101, eleven endpoint hashes unchanged, before producer changes. Nested timeout summaries describe the same consumer case. Authentic combat-log OnLoad reaches `ipairs` with nil at cached line90; the pure schema case fails its table assertion. Other printed addon-load errors are not erased by this RED. Producer implemented afterward. Independent saved-artifact gate2295 at b58020420/epoch174814 authenticates two catalog/isolation PASS, original Garrison load PASS, both publication checks PASS and normal Retail startup[]/exit0; overall121PASS/1consumerFAIL among122 unique tests. Authentic AddCombatEvent now reaches line129 and fails on niltextColor because the four engine color globals are absent from the simulator bootstrap; closure discovery already follows the declared Blizzard_Colors dependency, whose scripts do not define these engine globals. Final Fire/Holy formatted-message assertions remain unreached. This is the behavioral RED preceding the four literal named-color rows. [Retained color follow-up](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/adventures-color-green-2026-10-11/README.md),33cd07687/epoch180245, independently authenticates four catalog/color/consumer cases plus original Garrison load PASS:five unique executions. Both strict formatted Fire/Holy assertions now execute successfully, normal startup[]/exit0 and fmt/check0 remain bounded Retail proof. Later cfg-only ba413e8e1 retains those Retail bodies; no new binary proof or all-profile/native claim follows from that equivalence. No fake fixture color or weakened consumer assertion.

## Known gaps (current cycle)

- [x] Establish passing catalog and mutation-isolation proof at b58020420; see [retained packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/achievement-garrison-policy-2026-10-11/README.md).
- [x] Complete authentic AddCombatEvent and concrete color-bootstrap follow-up at33cd07687; see retained five-case proof.
- [x] Verify three pure catalog/isolation/color cases under all seven configurations; no vendor input acquired.
- [ ] Relevant final broad integration acceptance remains FAIL: [authenticated outcome](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/query-final-suite-result-2026-10-11/README.md) has three performance/asset failures; original Garrison load and four catalog/color/consumer cases PASS inside totals. Initial zero-case capacity block and actual retry submission remain preserved; native/all-profile loaded-addon coverage stays separate.
- [x] Pass the unchanged `tests/blizzard_garrison_ui_loads.rs::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` assertion; preserve its historical failures.

## Out of scope

- Native catalog contents, order and locale parity remain unverified; simulator enUS labels do not establish those properties.
- Mixed-school masks may require additional rows later; this spec neither requires them now nor forbids expansion.
- Implementing or changing `C_Garrison.GetCombatLogSpellInfo` is unrelated scope; supplied spell metadata isolates the catalog contract in consumer tests.
