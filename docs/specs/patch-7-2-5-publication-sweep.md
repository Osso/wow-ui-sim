# Patch 7.2.5 publication sweep

Audit [pinned Warcraft Wiki source](../../data/patch-api/sources/7.2.5-api-changes.wikitext), revision 4311256. Publication is not native behavior parity; see [audit](../wiki/investigations/patch-7-2-5-api-audit.md).

## What it must do

- [ ] Account for every linked API identity and retained extract row.
- [ ] Probe real raw publication and lookup after unmodified cached retail startup; compare the exact gap set.
- [ ] Preserve complete historical inputs and reproduce saved registers/extracts with recorded flags.
- [ ] Query selected garrison tree ID and its optional friendship faction from simulator state.
- [ ] Enumerate matching tree IDs for both garrison type and class, without leaking a mutable catalog to Lua; unknown pairs return no values.
- [ ] Preserve existing profiles and all prior publication sweep results.

## How it works

- [Audit and evidence](../wiki/investigations/patch-7-2-5-api-audit.md).
- [C API boundary](../../AGENTS.md#c-api-boundary).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in top-level summary bullet parser, canonical identities and rename directions.
- `tests/patch_7_2_5_publication_sweep.rs`: publication and exact retained-gap sweep; 7.3.0 integration placeholder first.
- `src/c_api/c_garrison_trees.rs`: selected-tree context and class/type catalog queries.
- `data/patch-api/evidence/7.2.5-session-2026-10-08/validate.py`: portable, read-only historical proof.

## Tests asserting this spec

- `tools/test_gen_patch_wikitext_register.py`: canonical names, multi-reference rows, rename directions and boundaries.
- `tests/patch_7_2_5_publication_sweep.rs`: full cached publication.
- `tests/patch_7_2_5_garrison_trees.rs`: empty, populated, replacement, class/type filtering and output isolation.

## Known gaps (current cycle)

- [ ] `C_UI.Reload`: no VM teardown/recreation/SavedVariables reload lifecycle; an alias to the existing event-only ReloadUI would not model this.
- [ ] `ReloadUI` historical rename/removal: current retail consumers prohibit retirement.
- [ ] `C_Unit`: page names a namespace but links the unqualified owner/controller global; no explicit namespace member to implement.

## Out of scope

Native 7.2.5 client reconstruction; undocumented event names/payloads, unnamed commentator/transmog functions; full reload lifecycle and native protected/secret argument parity without captures; shipping a guessed Legion catalog. Existing chat-bubble metadata is not native frame/forbidden-filter parity.
