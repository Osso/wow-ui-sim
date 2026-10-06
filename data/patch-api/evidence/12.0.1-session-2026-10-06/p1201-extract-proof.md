# 12.0.1 extract proof — B01–B04

Code revision `0abd710a8`; master baseline `f1b6977a9`. Page ledgers unchanged. Bounded current Retail fixture proof only; no native or historical execution claim.

Outcomes: 57 proven-by-test, 18 still-pending, 11 superseded.

| Source ID | Outcome | Test / patch / pending reason |
|---|---|---|
| `prose-2026-03-21-167` | proven-by-test | patch_12_0_1_cached_secure_delegate_removed |
| `prose-2026-03-21-173` | proven-by-test | patch_12_0_1_cached_secure_delegate_removed |
| `prose-2026-03-21-174` | still-pending | Secret-bearing duration objects reject tainted callers; retained ignored RED patch_12_0_1_tainted_secret_duration_object. Opaque rendering consumption is not implemented; caller taint is never cleared. |
| `prose-2026-03-21-176` | proven-by-test | patch_12_0_1_cached_secure_delegate_removed |
| `prose-2026-03-21-177` | still-pending | Numeric secret setter rejection proven, but full addon replication lacks secret-bearing duration consumption and derived LoC replacement. |
| `prose-2026-03-21-179` | proven-by-test | patch_12_0_1_cooldown_cast_expiry_transitions, patch_12_0_1_cooldown_charge_formula_and_zero_span |
| `prose-2026-03-21-180` | proven-by-test | patch_12_0_1_cooldown_cast_expiry_transitions, patch_12_0_1_cooldown_charge_formula_and_zero_span, unrestricted_action_and_existing_spell_dtos_match_concrete_record |
| `prose-2026-03-21-181` | still-pending | Positive/nonpositive intervals and real cast expiry proven; producer always enables regular cooldowns, so disabled/held branch cannot be exercised. |
| `prose-2026-03-21-182` | proven-by-test | patch_12_0_1_cooldown_charge_formula_and_zero_span |
| `prose-2026-03-21-183` | still-pending | LoC isActive is a supplied host flag, not computed from the positive start/duration formula. |
| `prose-2026-03-21-185` | still-pending | Regular and charge inactive zero spans proven; inactive spell LoC duration still returns no object rather than a zero-span object. |
| `prose-2026-03-21-186` | still-pending | Cooldown-aura association query does not participate in spell/action interval selection. |
| `prose-2026-03-21-187` | still-pending | No PvP-trinket/Adaptation cooldown-aura producer or authoritative hotfix mapping; no mapping fabricated. |
| `prose-2026-03-21-188` | still-pending | Association lookup still must be consumed by callers; upstream aura-derived interval selection missing. |
| `prose-2026-03-21-192` | proven-by-test | unrestricted_action_and_existing_spell_dtos_match_concrete_record, assigned_first_slot_returns_all_five_concrete_values |
| `prose-2026-03-21-193` | still-pending | No behavioral old-name deprecation/unpacked-return proof; publication sweep is insufficient. |
| `prose-2026-03-21-194` | still-pending | Secret numeric fields/public flags are fixture-tested, but shouldReplaceNormalCooldown is a host flag rather than derived expiration comparison. |
| `prose-2026-03-21-195` | still-pending | Existing seeded publisher returns numeric 4 for player/index 1, not a duration object. Populated real producer belongs to concurrent publication-sweep scope. |
| `prose-2026-03-21-196` | still-pending | No populated totem-duration state producer/proof in this checkout; missing-publication producer changes belong to concurrent sweep task. |
| `enumerations-Enum-CombatAudioAlertSpecSetting-215` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CombatAudioAlertSpecSetting-216` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CombatAudioAlertSpecSetting-217` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CombatAudioAlertSpecSetting-218` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CombatAudioAlertSpecSetting-219` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CooldownViewerAlertEventType-220` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CooldownViewerAlertEventType-221` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-CooldownViewerAlertEventType-222` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-DamageMeterType-223` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-DamageMeterType-224` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-DamageMeterType-225` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-NeighborhoodInviteResult-226` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-NeighborhoodInviteResult-227` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-PartyRequestJoinRelation-228` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-PartyRequestJoinRelation-229` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-SecretAspect-230` | superseded | 12.0.5 and 12.1.0 |
| `enumerations-Enum-SecretAspect-231` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-SecretAspect-232` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-233` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-234` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-235` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-236` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-237` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-238` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-239` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-240` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-241` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-242` | superseded | 12.1.0 cached surface; earlier renumbering patch not established |
| `enumerations-Enum-TooltipDataLineType-243` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-244` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-245` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-246` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-247` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-248` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-249` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-250` | proven-by-test | patch_12_0_1_enum_publication |
| `enumerations-Enum-TooltipDataLineType-251` | proven-by-test | patch_12_0_1_enum_publication |
| `structures-CatalogShopBundleChildInfo-256` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots |
| `structures-CatalogShopBundleChildInfo-257` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots |
| `structures-CatalogShopProductDisplayInfo-258` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots, fully_populated_display_has_exact_declared_shape |
| `structures-CatalogShopProductDisplayInfo-259` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots |
| `structures-CatalogShopProductInfo-260` | proven-by-test | fully_populated_product_has_exact_declared_shape |
| `structures-CatalogShopProductInfo-261` | proven-by-test | fully_populated_product_has_exact_declared_shape |
| `structures-CatalogShopProductInfo-262` | proven-by-test | fully_populated_product_has_exact_declared_shape |
| `structures-CatalogShopSectionInfo-263` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots |
| `structures-CatalogShopSectionInfo-264` | proven-by-test | catalog_shop_patch_12_0_1_bundle_section_snapshots |
| `structures-DamageMeterAvailableCombatSession-265` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterAvailableCombatSession-266` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSession-267` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterCombatSession-268` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSession-269` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSessionSource-270` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterCombatSessionSource-271` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSource-272` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterCombatSource-273` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSource-274` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSource-275` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSource-276` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSource-277` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpell-278` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterCombatSpell-279` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpell-280` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpell-281` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpellUnitDetails-282` | still-pending | Complete populated out-of-combat shape/value/snapshot proof passes, but parent ConditionalSecret/combat field policy remains unmodeled; existing producer explicitly blocks combat publication. |
| `structures-DamageMeterCombatSpellUnitDetails-283` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpellUnitDetails-284` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |
| `structures-DamageMeterCombatSpellUnitDetails-285` | proven-by-test | damage_meter_patch_12_0_1_exact_values_and_snapshot_transitions |

## Verification

Exact commands, revisions and outputs: [proof ledger](p1201-extract-proof.json). Existing enum filters each pass before/after. DamageMeter 14/1 before, 15/1 after; same `entry_frame_count=0` UI failure. Cooldown 205/1/1 ignored before, 208/1/1 ignored final; same WoWForever preservation failure. Eight intermediate failures from invalid negative/zero active fixture starts resolved with positive host-clock fixtures, not weakened active assertions. Catalog 21/0 before, 23/0 after. All five isolated publication sweeps pass against unchanged known-gap fixtures. New enum, exact damage DTO, catalog and three cooldown contract tests pass; cached delegate test passes alone. Ignored secret-bearing duration RED remains pending; explicitly running it with `--ignored` fails with the existing master guard (`secret-origin widget readout requires an untainted caller`). This is not a passing test or separate original-master runtime proof. Audio lib filter selects zero tests, not behavioral proof. Formatting and local cargo check pass. Six vendor manifest warnings persist.

Startup retry printed `[]` and CLEAN/0 errors; process still required 90-second timeout (exit124). Initial required invocation also timed out. No master process-termination comparison; do not call this a clean exit. No source changes follow final code revision.

## Inferences and blockers

- Public out-of-combat damage snapshots; combat queries explicitly blocked.
- Explicit host charge inputs; no automatic spending/replenishment.
- Fresh catalog maps/snapshots; unknown bundle returns empty; missing nonnullable section raises contextual input error; exact public i32 selectors.
- Later first-supported retail-12-0-5 gate, with current Retail enum corrections; no 12.0.1 runtime epoch.

Parent damage DTO closure is withheld for missing combat ConditionalSecret policy. LoC activity/replacement flags are supplied input rather than derived expiration comparisons. Cooldown-aura associations do not feed interval selection. Duration-object secret consumption rejects tainted callers; no taint-clearing workaround added. Totem/active-LoC publication producers remain concurrent-sweep-owned.
