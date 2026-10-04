# Retail 12.1.0 supplemental publication sweep

A data-driven breadth probe covers all 778 entries in the [wikitext register](../../data/patch-api/sources/12.1.0-wikitext-register.json), guided by the [supplemental triage](../../data/patch-api/evidence/12.1.0-session-2026-10-04/triage-wikitext.md). This proves publication (or absence) only, not behavior. Added/changed rows earn at most `partial-development-green`; removed symbols may earn `bounded-coverage` only when their whole contract is absence.

## What it must do

- [ ] Probe every unique source ID across global-api, framexml, scriptobjects, widgets, events and cvars, for added, removed and changed directions, in one fully loaded cached Game environment.
- [ ] Classify raw global publication; raw namespace/mixin member publication; normal method resolution on objects made by real widget/scriptobject factories. Added/changed members must resolve to functions without accepting fabricated namespace fallbacks. Removed namespace/mixin members must be nil under both raw and ordinary lookup.
- [ ] Probe concrete events through a real frame's `RegisterEvent` and registration state, preserving return/error details. Restricted-event false returns do not mean absence if the frame registered the event.
- [ ] Query CVar current and default values: both nonnil for added/changed and both nil for removed. Report page-default mismatches separately without changing publication success.
- [ ] Emit per-ID `{expected, observed, ok}` JSON to `P1210_SWEEP_OUT`, when set, before comparing the actual non-ok ID set with the committed known-gap list. Both newly failing and newly successful known-gap rows require review; the test never updates its own expectations.
- [ ] Preserve unsupported factories, malformed symbols, wildcard inventory occurrences and probe errors as explicit non-ok observations rather than silently granting coverage. Reject missing/duplicate inventory rows and invalid section/direction data.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md) — cached Blizzard Game loading.
- [Lua API](../lua-api.md) — publication and factory surface.
- [Event system](../event-system.md) — event registerability.

## Implementation inventory

- `tests/patch_12_1_0_publication_sweep.rs` — register-driven classifier and one environment per sweep.
- `tests/data/patch_12_1_0_sweep_known_gaps.json` — reviewed exact set of non-ok IDs; initially empty.
- `tests/common/prefork_full_ui_preload.rs` — existing full cached Game preload reused without changes.

## Tests asserting this spec

- `tests/patch_12_1_0_publication_sweep.rs::patch_12_1_0_publication_sweep` — publication/absence breadth and exact known-gap comparison.

## Known gaps (current cycle)

- [ ] Staged only; not compiled or executed. Integrator must capture and review first-run observations before committing a known-gap baseline or granting ledger credit.
- [ ] `CHAT_MSG_*` describes a family, not one registerable event; its row remains non-ok without an authoritative concrete expansion.
- [ ] RadialProgress is not a recognized simulator animation type at authoring time; its real factory probe rejects a returned generic Animation. No generic-method fallback earns credit.
- [ ] The current register does not carry CVar page defaults (all CVar annotations are empty); default comparison can only apply when `page_default` (or `default`) metadata is supplied.

## Out of scope

- Function signatures, outputs, security/secret policies, changed behavioral contracts, event payloads, dispatch, CVar mutability, and native-client parity.
- Loading optional LoD panels or Glue screens beyond the existing cached Game startup helper. Their unpublished rows remain gaps; this does not prove they are missing in native Retail.
- Authenticating the cached UI as a particular native build, fixing runtime gaps, or automatically upgrading ledger status from a baseline pass. Known gaps remain gaps even when expected.
