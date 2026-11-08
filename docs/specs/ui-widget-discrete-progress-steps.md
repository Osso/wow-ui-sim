# Supplied DiscreteProgressSteps visualization

Host-supplied DiscreteProgressSteps records belong to `src/c_api/ui_widget_manager/`; [widget manager architecture](../wiki/systems/ui-widget-manager-state.md) describes existing supplied-record behavior. Complete snapshot producer and typed host-feed variant are implemented after genuine five-case RED at `42f206f66`: 0 PASS / 5 missing-supplied-record FAIL, before cache/atlas prerequisites. Bounded actual Retail/PTR query/feed/security/cached-consumer, preservation, fresh startup and default-check proof pass in separate scopes; [current packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-discrete-progress-2026-10-13/README.md#retail-and-ptr-current-proof) owns receipts. Saved independent GREEN text-retention audit PASS at `a6723200be40145ec352666b538998bf4035a608`: 106 payloads, 302,209 public bytes, 63 JSON equivalents and 16 doc links/anchors; GREEN manifest and six modified docs included, original RED and other packets not rechecked. Cached Retail/PTR schema identifies visualization19 and all19 nonnil fields; native runtime/catalog/render parity remains unknown.

## What it must do

- [x] `C_UIWidgetManager.GetDiscreteProgressStepsVisualizationInfo(number)` returns nil without supplied data, otherwise a fresh, complete 19-field cached-schema snapshot preserving supplied numbers/enums/strings/bool without inferred defaults or consumer clamping; caller mutation cannot alter retained snapshots or input state.
- [x] Supplied replacement/removal and independent environments are observable; numeric IDs and non-number rejection match existing bars. Actual host-secret numbers are allowed when untainted, denied before supplied/absent lookup when tainted, with public recovery and public snapshot fields.
- [x] Existing typed host update feed accepts visualization19, validates entry/visualization type and supplied-set existence before mutation, stores the supplied record before synchronous notification, and preserves independent record lookup after membership removal.
- [x] Unchanged active Retail/PTR cached manager/template consumes the supplied record and observes actual progress/numSteps geometry, incremental updates, hidden/restore, record/membership removal, pool cleanup and unregister/no reacquisition. Atlas application is established through real catalog-backed texture state, never injected data.

Complete snapshot fields: `shownState`, `tooltip`, `progressMin`, `progressMax`, `progressVal`, `numSteps`, `tooltipLoc`, `widgetSizeSetting`, `textureKit`, `frameTextureKit`, `hasTimer`, `orderIndex`, `widgetTag`, `inAnimType`, `outAnimType`, `widgetScale`, `layoutDirection`, `modelSceneLayer`, `scriptedAnimationEffectID`. Numeric key/snapshot policies follow the [manager contract](ui-widget-manager-state.md); no native acceptance range is inferred.

## How it works

- [Widget manager state](../wiki/systems/ui-widget-manager-state.md)

## Implementation inventory

- `src/c_api/ui_widget_manager/discrete_record.rs`: complete explicit input DTO.
- `src/c_api/ui_widget_manager/inputs.rs`: environment-owned keyed map, empty by default.
- `src/c_api/ui_widget_manager/discrete.rs`: authenticated numeric lookup and fresh complete19-field snapshot projection.
- `src/c_api/ui_widget_manager/queries.rs`: registers the query in the existing namespace.
- `src/c_api/ui_widget_manager/feed.rs`: typed visualization19 validation/storage before existing synchronous notification.
- `src/c_api/ui_widget_manager.rs`: module wiring and public DTO export.

## Tests asserting this spec

`tests/ui_widget_manager_discrete.rs` is automatically included by existing integration macro; explicit child paths cover `tests/widget_discrete/{queries,consumer,security,feed,fixtures}.rs`. Seven intended cases: query3, actual cached consumer1, actual-secret1 gated by `retail-12-1-0`, typed-feed2. At `42f206f66`, original five cases physically fail on missing supplied record; one timeout-child summary is duplicate, not a sixth case. Later fields/numeric/security/cache assertions were unreached. Original RED does not establish an independent feed RED. Query/security tests directly seed inputs; current consumer activation/update/hidden/restore/membership removal uses typed feeds, with explicit direct record deletion plus actual event for the separate record-removal boundary. Feed cases assert state visible inside real event callbacks and rejection without mutation/notification. Actual Retail50 and PTR7fb (docs-only over50) each pass seven unique Discrete cases; child consumer summaries are duplicate observations. Each profile separately passes five preservation cases (bar1/HUD2/core lifecycle2) and one fresh normal startup: empty-state getter type function, exactly one nil, trailing[]; full stderr CLEAN (219 content lines; 220 LF-split entries including the final empty entry). Default7fb offline/locked-j2 check and Cargo/direct format pass; six inherited manifest deprecations remain, zero source compiler messages. These are twelve executed cases per profile, not twelve new Discrete cases or twenty-four distinct features. Actual atlas application/paused-cooldown state is proven, not pixels/native/tutorial acceptance.

## Known gaps (current cycle)

- [x] Bounded current [GREEN text-retention audit](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-discrete-progress-2026-10-13/README.md#retail-and-ptr-current-proof) PASS at `a6723200be40145ec352666b538998bf4035a608`; original22-text RED retention independently passed earlier, not rechecked here. Preserve runner trailing spaces as exact evidence, not code-format failures.
- [ ] Native/catalog/render/tutorial/other-profile and full widget-system/parent acceptance remain open; this bounded record slice does not establish them.

## Out of scope

Native/server catalog discovery and rendered/native parity are not established by host-supplied fixture records. No vendor behavior changes or fabricated texture catalogs.
