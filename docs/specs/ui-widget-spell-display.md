# Supplied SpellDisplay visualization

Contract for environment-owned supplied SpellDisplay input, fresh complete nested snapshots, and existing cached Retail/PTR consumers. Passive input scaffold and nine tests are integrated. Genuine pre-producer supplied-result RED was observed in Retail and PTR. The bounded producer is present. [Current bounded gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md#current-bounded-runtime-gate--2026-10-13) owns independent Retail PASS, independently accepted PTR execution and separate selected retention PASS. Both independent profile scopes cover all ten bounded requirements. Source belongs under `src/c_api/ui_widget_manager/`; [manager architecture](../wiki/systems/ui-widget-manager-state.md) owns mechanism.

## What it must do

### Supplied input and query

- [x] Store complete explicit `SpellDisplayRecord` (14 top-level fields) and `WidgetSpellInfoRecord` (16 nested fields), with no default DTO, invented catalog, or Lua `Val` stored input.
- [x] Return a fresh top-level AND nested record on each lookup; preserve every supplied scalar/string/bool without clamping. Unknown/deleted input returns exactly one nil. Caller mutations and replacement do not alter retained snapshots or host state.
- [x] Use existing `WidgetId` floating-point identity and strict numeric validator, including signed-zero equality, fractional/negative/large supplied keys and rejection of coercible strings/non-numbers. These are simulator lookup policy, not native ID admissibility.
- [x] Keep same numeric input IDs isolated across environments.
- [x] Honor cached `SecretArguments=AllowedWhenUntainted` via actual host secret numbers and tainted/untainted calls; successful snapshots are ordinary public values including nested fields. Numeric-input secrecy must not bypass tainted rejection or silently declassify IDs.

### Host feed and actual cached consumer

- [x] Reuse existing typed feed; both inspected cached schemas declare SpellDisplay=13, DoubleStateIconRow=14. Correct prior research's type14 assumption; no assertion about uninspected native versions.
- [x] Commit supplied map and membership before synchronous events, verified by complete nested snapshots inside real event callbacks; reject type mismatch/unknown set without mutations/notification. Membership removal retains independent visualization input.
- [x] Through real `UIWidgetContainerTemplate`, cached manager registration, events and `fire_on_update`, activate/update/Hidden-release/restore/remove actual SpellDisplay and its nested Spell. Assert nested text, tooltip, stacks, earned mark, tint, geometry and pool/layout state without direct Setup or mocks.
- [x] Separately prove attached-player context propagation (container/consumer field), not unit-specific input catalogs or world-loot identification. `attachedUnit` is consumer-added context, not a documented supplied query field.
- [x] Separately prove zero-spell polling: activate zero input by feed; replace input without event; real XML OnUpdate observes nonzero supplied spell. Do not substitute incremental event Setup for polling.

## How it works

- [Existing manager state](../wiki/systems/ui-widget-manager-state.md)
- [Existing event dispatch](../event-system.md)
- [Existing cached loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/ui_widget_manager/spell_display_record.rs`: passive complete DTOs, no defaults.
- `src/c_api/ui_widget_manager.rs`: DTO exports and producer module declaration.
- `src/c_api/ui_widget_manager/spell_display.rs`: Retail/PTR-gated supplied getter; cloned host record projected into fresh plain outer14/nested16 tables.
- `src/c_api/ui_widget_manager/inputs.rs`: empty-by-default environment-local map.
- `src/c_api/ui_widget_manager/feed.rs`: type13 variant/storage in existing feed before notification.
- `src/c_api/ui_widget_manager/queries.rs`: producer registration and reused numeric/secret input validator.

## Tests asserting this spec

- `tests/ui_widget_manager_spell_display.rs`: Retail/PTR integration module wiring.
- `tests/widget_spell_display/fixtures.rs`: explicit host fixture, first getter result before cache/secret construction, recursive exact14+16 snapshots.
- `tests/widget_spell_display/queries.rs`: 3 tests for freshness/update/delete, numeric identity and isolation.
- `tests/widget_spell_display/feed.rs`: 2 tests for event ordering/removal and rejected feed preservation.
- `tests/widget_spell_display/security.rs`: 1 real host secret-policy test, enabled in both Retail/PTR test modules.
- `tests/widget_spell_display/consumer.rs`: 3 separate tests for lifecycle, attached context and zero-spell polling.

[Current bounded gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-spell-display-2026-10-13/README.md#current-bounded-runtime-gate--2026-10-13) preserves pre-getter Retail/PTR 0 PASS / 9 FAIL, historical `38ec` partial 6 PASS / 3 FAIL and removal-fixture RED. Current independent Retail28/28 eight integration scopes plus normal CLEAN is separate from accepted independent PTR runtime review, default-check proof and selected retention integrity. Historical 741 counts compiler JSON records, not seconds; original report unchanged.

## Known gaps (current cycle)

- [x] Complete new selected-retention integrity, including full gate copies; independent Retail/PTR bounded runtime PASS is separate.
- [x] Combined independent Retail/PTR bounded acceptance: both reach all nine cases including cached lifecycle/context/polling. Native/catalog/world-loot/render/full-parent unproved.
- [ ] World-loot click, rarity-pip counts and ObjectShown/ObjectHidden/WorldLootObjectList interaction need a separately grounded backing fixture seam; none invented here.
- [ ] Nested color-object requirement is unresolved against these pinned schemas: they declare numeric `borderColor`/`tint` enums, no RGBA/tint objects or nullable nested fields. Consumer uses ColorManager atlas overrides and font-color objects downstream, not DTO color objects. No new color default model or opaque `Val` input added.

## Out of scope

- Native widget/spell catalogs, server feed, all-profile/full-parent acceptance and rendered/native parity: no supplied evidence.
- World-loot gameplay/context/range/cursor/click/pip policies: separate backing model/fixtures absent; ordinary-player context cannot prove them.
- `showGlowState` rendering behavior: field preservation only; inspected nested cached Setup does not consume it.
- TextWithState reimplementation, vendor/cache/CASC/WTF/settings changes and operations: prohibited in this input-only integration scope.
