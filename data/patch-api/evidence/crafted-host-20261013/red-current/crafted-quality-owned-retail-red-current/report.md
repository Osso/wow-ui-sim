# Crafted-quality owned Retail RED verification

**Existing evidence verified; feature FAIL/RED, not implemented.** Source `6c38a82739dbf27fc58c3c801c32a0f0fc98d21c`. Epoch: `<PRIVATE_HOME>/.local/state/wow-ui-sim/verification/widget-discrete-progress-current/crafted-quality-owned-retail-red-20261014T014209924708Z`.

## Recorded execution and receipt

| Run | Wait completion | Launcher/main exit | Test result | Filtered | Duration |
|---|---|---|---|---|---|
| runtime-crafted-owned-red-2g | PASS: completed normally | 101 / 101 | 0 PASS, 5 FAIL | 11021 | 0.87s tests; 934ms service |
| runtime-crafted-itembutton-red-2g | PASS: completed normally | 101 / 101 | 0 PASS, 1 FAIL | 11025 | 0.84s tests; 901ms service |

Six unique failures, not twelve from repeated failure summaries. No extra child or rerun. Both admissions allowed Retail, MemoryMax 2GiB, MemoryHigh 1536MiB, CPUQuota 200%; peak 127.8MiB / 1.3GiB respectively, no swap. Recorded argv runs the sealed integration executable under network-unshared, read-only bwrap with 86s timeout, CASC disabled, sound disabled, serial tests and nocapture. Unit names containing `green` do not establish GREEN. Result scope strings are predictions, superseded by actual boundaries below.

Receipt stat reuse matches both results and current executable stat: dev 66306, inode 148663051, size 471470720, mtime_ns 1791942439981408462. Prior receipt authentication reused only; no rehash, binary read, library/archive/source-tree authentication, build, runtime, service change, delegation, Bash, cwd-tool use, or canonical edits.

Full stdout/stderr/argv/result/admission/launcher read for both runs. Consumer stderr: 14307 bytes; 114 LF-terminated lines / 115 LF-split segments including terminal empty segment; all read, not just displayed tail.

## Public positive contracts: first failures and coverage

All five fail `tests/crafted_quality_owned/assertions.lua:17`: `type(actual) == 'table'`, message **"supplied crafted item must return a table, not auto-nil"**. This message is assertion wording, NOT a print of actual return payload/type. No positive payload was printed; do not conclude it was nil.

| Test (crafted_quality_owned:: prefix) | Rust failure | Reached | Unreached |
|---|---|---|---|
| crafted_quality_supplied_record_projects_exact_thirteen_fields_and_one_result | tests/crafted_quality_owned.rs:50 | host item 900001 insertion, first query, failed table check | exact 13 fields/types/values, one-result arity |
| crafted_quality_existing_item_selector_forms_share_the_supplied_record | :71 | first numeric 900001 selector, failed table check | numeric result validation/arity; decimal string, item string, hyperlink |
| crafted_quality_detached_snapshots_survive_host_replacement_and_removal | :97 | two host records, initial queries, retained result table check | detached identities, Lua mutation isolation, replacement/removal, retained and other-item preservation |
| crafted_quality_simulator_unknown_is_one_nil_and_inputs_are_environment_owned | :170 | independent second environment with recipe-only input: 900001/900002/900003 all nil and exactly one result; then first environment positive query fails table check | later first-environment unknown 900003 checks; second positive record and cross-environment positive isolation |
| crafted_quality_real_vm_secret_selectors_guard_lookup_and_recover_wrappers | :255 | taint mode enabled, untainted setup, first genuine secretwrap(900001), issecretvalue check, positive query fails table check | further selectors/absent cases, arity, addon-tainted denial, type-order denial, wrapper/payload recovery, host-removal queries |

The miss/environment prelude ALREADY passes before first environment-positive failure. It is not blocked on catalog/equipment setup. Test source intent is broader than execution coverage.

## Whole original consumer: genuine RED, not closure/setup failure

Exactly **12** logged addon loads in this order: Blizzard_ScriptErrors, Blizzard_SharedXMLBase, Blizzard_Menu, Blizzard_Colors, Blizzard_LoadLocale, Blizzard_Fonts_Shared, Blizzard_HelpPlate, Blizzard_ProjectConstants, Blizzard_PrintHandler, Blizzard_Narration, Blizzard_SharedXML, Blizzard_ItemButton. Each load returns successfully; all 12 warning/error lists empty. Error lists, error-record lists and error-count maps empty at all 14 logged boundaries (12 loads + baseline + positive boundary). The positive Rust-returned Lua assertion error is separate from those empty simulator diagnostic collections.

Baseline completes before failure: original addon loaded; global handler is Lua; printed provenance `@Interface/AddOns/Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua 323 350`; untainted setup; actual reagent prints `nil nil`; actual empty crafted prints `nil nil`. Real simulator Button created/shown; original no-quality call yields false/false flags and no overlay. Minimal ProfessionsFrame absence asserted; host visibility input is real simulator Frame, hidden, with always-show false. No replacement getter or sibling override. This host Frame is NOT full original Professions panel, native acceptance, or render proof.

Host inserts item 900001, quality 3, inventory atlas `professions-icon-quality-tier3-small`. Original handler is invoked; prints positive effects **`nil false false`** (overlay, crafted flag, profession flag). First positive flag assertion fails at `tests/crafted_quality_item_button.rs:79`, propagated at :172: **"positive supplied item must be crafted, not auto-nil"**. Therefore valid whole-original-consumer RED, not TOC closure or setup-fixture failure. This effects print does not print positive getter payload; bounded cached source shows handler queries actual reagent first, then crafted, and uses non-nil quality to set flags/setup overlay. No catalog/equipment prerequisites are warranted by this already-working baseline.

Later profession flag assertion, real overlay Texture, atlas, TOPLEFT (-3,2), OVERLAY sublevel 7, host visibility toggles, always-show behavior, clears, callback-unregistration effects and host-removal behavior remain unproved/unreached. No callback-unregistration assertion is directly reached or demonstrated.

## Non-empty diagnostics: classify all

These are nil-symbol observations/missing requirements, not warnings, load exceptions, or Lua error records. They did NOT prevent whole-closure loading/baseline/positive handler execution; do not call diagnostics wholly empty.

- Blizzard_Fonts_Shared: 4 Public nil globals, inline: LOCALE_koKR:1, LOCALE_ruRU:13, LOCALE_zhCN:5, LOCALE_zhTW:9.
- Blizzard_PrintHandler: 1 Public nil global, PrintToDebugWindow, Blizzard_PrintHandler.lua:15.
- Blizzard_Narration: 7 Public nil globals, Blizzard_NarrationUtil.lua:37: NARRATION_CONTEXT_CHAR_CREATE, NARRATION_CONTEXT_CHAR_SELECT, NARRATION_CONTEXT_LOGIN, NARRATION_CONTEXT_REALM_LIST; :46: NARRATION_OBJECT_BUTTON, NARRATION_OBJECT_CHECK_BUTTON, NARRATION_OBJECT_SLIDER.
- Blizzard_SharedXML: 7 nil globals: Public DebugMenu, DebugBarManager.lua:106; Secure SharedColorConstants.lua:34 KYRIAN_BLUE_COLOR, NECROLORD_GREEN_COLOR, NIGHT_FAE_BLUE_COLOR, VENTHYR_RED_COLOR; :28 PVP_SCOREBOARD_ALLIANCE_CELL_COLOR and PVP_SCOREBOARD_HORDE_CELL_COLOR. Also 1 missing Public CNamespace C_Reveal, DebugBarManager.lua:106.

Totals: **19 nil observations (13 Public, 6 Secure), 1 missing requirement, 0 warnings, 0 collected Lua errors/records/count entries**. Other 8 addons have empty nil lists; other 11 addons have empty missing-requirement lists. No claim these nils are universally benign or fixed; only non-blocking at this observed boundary. Full raw diagnostics preserved in proof.json.

## Source and profile limits

Bounded commit inspection: six files changed (spec, two state files, two Rust tests, Lua assertion helper). Production changes only add cfg(retail-12-0-0) `crafted_item_quality: HashMap<u32, CraftingQualityInfo>` and empty default. Independent of recipe inputs. No getter/registration implementation added by this revision; passive host input is not a public return implementation. Source comments saying unexecuted/fixture-unproved are stale relative to these recorded Retail executions, not grounds to disregard runtime evidence.

Retail evidence only. Tests authored for Retail/PTR; PTR positive executions still pending. No native secret-policy proof, original Professions panel proof, visual/render proof, or new implementation claim. Verification complete as artifact diagnosis; feature remains RED.
