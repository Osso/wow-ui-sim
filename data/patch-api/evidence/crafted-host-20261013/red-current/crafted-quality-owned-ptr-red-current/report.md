# Existing PTR crafted-quality RED verification

**RED observed in both runs.** Verification PASS; feature tests FAIL. No rerun or new authentication.

## Evidence identity

Epoch: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-ptr-red-20261014T014829545770Z`.
Source: `c652aaa8824f87089f0747a3ad0e19a4dbe3350e`; production delta: `6c38a8273`.
Epoch revision metadata identifies c652 as publication-only after 6c. Caller supplies that the exact tests, assertion helper, consumer and passive map are unchanged 6c→c652, and current c0 is publication-only. No new source-tree authentication or current-tree validation performed; this report proves the recorded epoch only.

Existing integration receipt stat matches dev 66306, inode 148789013, size 471553176, mtime_ns 1791942755964525331. Binary contents, libraries and archive were not read or rehashed.

## Exact results

| Run | Exit | Passed | Failed | Ignored | Measured | Filtered | stdout bytes | stderr bytes | result bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| runtime-crafted-owned-red-2g | 101 | 0 | 5 | 0 | 0 | 10198 | 1221 | 5912 | 682 |
| runtime-crafted-itembutton-red-2g | 101 | 0 | 1 | 0 | 0 | 10202 | 376 | 14231 | 722 |

All six stdout/stderr/result files consumed completely. Full text retained in proof.json.

## Positive-query failure boundary

All five tests fail at `tests/crafted_quality_owned/assertions.lua:17`:
`assert(type(actual) == 'table', 'supplied crafted item must return a table, not auto-nil')`.
Actual positive payload was not printed: non-table is established, nil specifically is not.
Rust panic boundaries in `tests/crafted_quality_owned.rs`: detached snapshots 97:6; selector forms 71:6; genuine VM secrets 255:6; independent environment 170:10; thirteen-field projection 50:6.

Fresh environments and helper installation completed. Each test supplies the per-item host map; source declares `HashMap<u32, CraftingQualityInfo>` at `src/lua_api/state/sim_state.rs:156-157` and initializes it empty at `src/lua_api/state.rs:310`, independently of recipe inputs.
Execution reached the positive assertion after the unknown/recipe-environment one-nil prelude passed. In the secret test, untainted setup and the first genuine VM wrapper assertion passed before the first positive result failed.

**UNREACHED:** thirteen fields, positive arity, later aliases, snapshot freshness/mutation, replacement/removal, positive environment isolation, taint denial/type-ordering/recovery. Test names are not evidence these later assertions ran.

## Whole-original consumer failure boundary

12 original TOCs loaded: Blizzard_ScriptErrors, Blizzard_SharedXMLBase, Blizzard_Menu, Blizzard_Colors, Blizzard_LoadLocale, Blizzard_Fonts_Shared, Blizzard_HelpPlate, Blizzard_ProjectConstants, Blizzard_PrintHandler, Blizzard_Narration, Blizzard_SharedXML, Blizzard_ItemButton.
Full closure diagnostics: **0 warnings, 0 Lua errors**, but **19 nil observations and 1 missing requirement**. Nil observations: Fonts_Shared 4, PrintHandler 1, Narration 7, SharedXML 7. Missing requirement: `C_Reveal`, SharedXML `DebugBarManager.lua:106`. Diagnostics were not wholly empty.

Runtime provenance printed `Lua @Interface/AddOns/Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua 323 350`. Actual public reagent baseline printed `nil nil`; empty-input crafted baseline printed `nil nil`. Original handler, whole-addon load, untainted setup, real Button and real host visibility Frame assertions completed. The Frame is a supplied host input, not the original professions panel.

Positive host map printed `900001 professions-icon-quality-tier3-small`; original handler effects printed `nil false false` (overlay, crafted flag, profession flag).
Failure: `positive supplied item must be crafted, not auto-nil`, surfaced at `tests/crafted_quality_item_button.rs:172:35`, the Rust boundary helper. This is genuine consumer RED after valid closure/fixture setup, not setup RED.

**UNREACHED:** positive profession flag, Texture/atlas, anchors, draw layer, visibility toggles, quality/full clears and host removal. Callback unregister is not asserted and is not a requirement of this proof.

## Scope and handoff

PTR only. No Retail inference or native, panel, rendering, or getter-correctness claim. No rebuild, runtime rerun, auth, vendor/service action, delegation, Bash, cwd tool or canonical edit.
Main may implement after the two actual observed RED boundaries. Subsequent independent gate remains asynchronous; this is not GREEN or completion evidence.
