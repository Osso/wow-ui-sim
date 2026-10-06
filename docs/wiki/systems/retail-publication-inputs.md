# Retail publication input queries

New namespace functions read per-environment `SimState` fields or typed records declared in `src/c_api/`. Lua registration files only wire callbacks. Host inputs supply missing catalogs/policies; functions do not acquire native service data or synthesize DB2 records.

## State and producers

| Fields | Producer behavior |
|---|---|
| `action_spell_ranges`, existing `action_bars`/`pet_actions` | Resolve current spell binding, GUID-keyed range snapshots and pet autocast state; rebinding cannot reuse unrelated spell metadata. |
| `spell_classifications`, existing `aura_filter_facts.important_spell_ids` | Number/name spell classification; importance shares the existing aura filter input. |
| `cooldown_viewer_cooldowns[].valid_alert_types` | Fresh alert capability arrays, independent of category/flags. |
| `nameplate_configuration` | Size configuration, host camera-token flags and simplified-token membership; no 3D plate renderer. |
| `encounter_policy`, `encounter_warning_settings`, existing CVars | Separate release/resurrection/timeline policy; warning availability, enablement and severity sound IDs. |
| Existing BNet friend records/`message_log`, `bnet_custom_message` | Locally accepted outbound intent and broadcast state, not network delivery. |
| `pvp_catalog`, existing `battlefield_queue` | Catalog/eligibility snapshots and existing queue transitions; match completion and daily wins remain host-owned. |
| `recipe_quality_inputs` | Exact recipe/quality and recipe/slot/quality tuple lookups; fresh documented quality structures. |
| `prey_inputs`, existing rich quest log | Active prey selection requires quest membership; full progress-widget snapshots are detached from host records. |
| `weekly_reward_progress` | Completion-tier rows, not repeated vault unlock thresholds; descending difficulty and optional combined counts. |
| Existing item `bonding` metadata | Intrinsic account-binding classification for ItemBind 7–9; inclusion of until-equipped mode is inferred, not instance ownership/bound state. |
| Existing outfit catalog | Bounded title-only outfit TooltipData; rename/removal changes subsequent results. |
| `limited_input_allowed`, `combat_log_restricted`, `pet_bonus_slot_available`, exterior hover flag, catalog new-product IDs | Explicit query/configuration state, not native input dispatch, secrecy enforcement, entitlement acquisition, hit testing or commerce. |

## Limits

Empty/false/zero defaults and local policies are marked `INFERRED`. Public spell classifiers do not implement alias override selection or tainted-secret identifier parity. Outfit tooltip appearance/layout parity, reward acquisition, native service acknowledgements and strict historical epoch builds are unverified. No new temporary/permanent shim or Blizzard Lua patch was added.

Per-ID outcomes and verification belong to the [closure contract](../../specs/patch-12-0-0-publication-sweep.md#remaining-namespace-closure--2026-10-06), not this page.

## Sources

- [Closure contract and proof](../../specs/patch-12-0-0-publication-sweep.md#remaining-namespace-closure--2026-10-06).
- [State fields](../../../src/lua_api/state/sim_state.rs) and [C API producers](../../../src/c_api/mod.rs).
- Cached `Blizzard_APIDocumentationGenerated` declarations and Blizzard consumer paths captured in the per-ID evidence linked by the contract.

## See Also

- [[transmog-outfits]] — existing outfit catalog and lifecycle.
- [[secrets-publication-queries]] — publication versus native security parity.
