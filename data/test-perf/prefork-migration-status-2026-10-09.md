# Prefork migration placement — 2026-10-09 epoch

## Historical input boundary

[prefork-migration-plan.json](prefork-migration-plan.json) is the historical 2026-10-08 classification and batch record, not current placement. Its `status`, module `remaining_count`/`remaining_seconds`, `remaining_priority`, and batch deferrals describe that epoch. In particular, its four `integration` MIGRATE/ADAPT rows do **not** describe placement at the later exact-fixture epoch.

The harness contract separates historical sealed counts from later epochs. No explicit mutable-status contract or file-specific cryptographic seal for this plan was established in the inspected spec, system page, or migration documentation references. Preserve the entire original plan rather than infer permission to rewrite its historical fields: classification, date, timings, source traces, reasons and batch results remain unchanged. This separate status document changes no immutable evidence input and makes no claim that the plan itself has a verified seal.

## Bounded scope reconciliation

The historical plan contains 186 MIGRATE/ADAPT names (93 + 93), partitioned without overlap by recorded status:

| Placement tranche | Names | Interpretation |
| --- | ---: | --- |
| Already prefork | 22 | Historical `already_prefork` rows |
| Historical migration batches | 160 | 97 + 38 + 25 `prefork` rows |
| Later exact fixtures | 4 | Historical `integration` rows; superseded placement below |
| Unaccounted names | 0 | 186 − (22 + 160 + 4) |

This is placement accounting, not passing-test or migration-gate acceptance. KEEP's 1,496 names are outside this migration scope; historical deferred reasons remain valid explanations of incompatibility with the completed-Game parent, not evidence of missing current migrations.

## Exact-fixture placement superseding the four historical rows

| Original stable name | Migration revision | Current bounded placement |
| --- | --- | --- |
| `chat_frame::test_chat_editbox_click_type_and_submit` | `29e48f225` | Retail exact chat fixture, prefork |
| `chat_frame::test_chat_editbox_text_color_after_activation` | `29e48f225` | Retail exact chat fixture, prefork |
| `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix` | `911c980fc` | Retail exact cast-bar fixture, prefork |
| `blizzard_player_spells_loads::mainline_spellbook_keybind_opens_and_closes_without_runtime_errors` | `911c980fc` | Retail exact spellbook fixture, prefork |

Macro-scope repair `3de874658` retains original constructors and bodies. [Integrated proof SSOT](../../docs/wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#exact-startup-fixtures-retained-redgreen-epoch) owns retained execution, conformance, listing, preservation and full-suite group-coverage facts; those proof counts are not duplicated here. Its separate base and exact-fixture summaries do not imply omitted execution.

## Acceptance limits

No missing migration names identified within the 186-name scope. This does **not** close non-Retail execution, exhaustive per-group selection/failure gates, or broader acceptance. The retained full suite is FAIL with unchanged failure identities, not an all-gates-pass result. [Harness spec](../../docs/specs/prefork-test-harness.md#exact-startup-fixtures-bounded-retail-proof-at-3de874658-2026-10-09) retains the open requirements. Main owns acceptance; this reconciliation runs no tests, builds or checks and grants no speedup, latest-HEAD execution or parent-completion credit.
