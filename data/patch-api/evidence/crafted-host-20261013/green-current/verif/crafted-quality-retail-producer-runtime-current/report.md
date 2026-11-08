# Retail crafted-quality producer runtime — PASS (bounded existing evidence)

Source: `cd9f42756472b8f8d706fd3e592f1977b3d89868`.
Epoch: `<epochs>/crafted-quality-retail-green-retry2-20261014T020756467959Z`.

Reinspected all six files (stdout, stderr, argv.json, result.json, admission.json, launcher.txt) per scope, including all 14,223 consumer stderr bytes. No reruns, build, authentication, vendor reads, service changes, delegation, Bash, cwd changes, or repository edits. New actual waiting: 0 seconds each. Existing launchers themselves used systemd-run --wait; their recorded runtimes were not zero.

## Exact runtime coverage

| Existing scope | Passed | Failed / ignored / measured | Filtered (not additional coverage) | Runtime / CPU / memory peak / swap |
|---|---:|---|---:|---|
| initial | 2 | 0 / 0 / 0 | 11024 | 733ms / 584ms / 129.3M / 0B |
| owned | 5 | 0 / 0 / 0 | 11021 | 1.527s / 1.466s / 85.7M / 0B |
| ItemButton | 1 | 0 / 0 / 0 | 11025 | 1.901s / 1.267s / 1.3G / 0B |

**8/8 unique test names**, no duplicate counting of aliases, assertion iterations, environments, diagnostics, or prior RED. All three launcher exits 0, success, status=0/SUCCESS. Zero observed skips, timeout exits, panic/failure or resource-kill indicators in these GREEN runs. All admissions allowed; worker MemoryMax=2147483648, MemoryHigh=1610612736, CPUQuota=200%; timeout 86s, TERM then kill-after=2; read-only mounts and unshare-net; CASC=0, NO_SOUND=1. Admission snapshots contain nonzero historic pressure/swap counters, not proof of current test resource failure; no per-run memory.events delta was supplied. Full snapshots and launcher text retained in proof.json.

- `crafted_quality_api::crafted_quality_required_item_info_errors_have_context`
- `crafted_quality_api::crafted_quality_secret_nil_authenticates_before_required_validation`
- `crafted_quality_owned::crafted_quality_detached_snapshots_survive_host_replacement_and_removal`
- `crafted_quality_owned::crafted_quality_existing_item_selector_forms_share_the_supplied_record`
- `crafted_quality_owned::crafted_quality_real_vm_secret_selectors_guard_lookup_and_recover_wrappers`
- `crafted_quality_owned::crafted_quality_simulator_unknown_is_one_nil_and_inputs_are_environment_owned`
- `crafted_quality_owned::crafted_quality_supplied_record_projects_exact_thirteen_fields_and_one_result`
- `crafted_quality_item_button::crafted_quality_original_item_button_observes_supplied_quality_and_visibility`

## Assertion reach and source wiring

| Contract | Evidence / proof level |
|---|---|
| Required input | crafted_quality_api.rs:8–27, existing initial GREEN: omitted and explicit nil reject; method and itemInfo context; ordinary errors not secret denial. |
| Secret nil | crafted_quality_api.rs:31 onward: genuine secretwrap(nil), issecretvalue, untainted contextual validation; tainted denial before validation, denied secretunwrap, original wrapper and payload preserved, caller/outer taint recovery and repeated query. Runtime GREEN, not merely authored tests. |
| Exact projection | owned test and assertions.lua: exactly 13 keys, exact values/types, one-result arity, no extras. quality=3; icon=crafted-icon; iconSmall=crafted-small; iconInventory=crafted-inventory; iconMixed=crafted-mixed; iconAppear=crafted-appear; iconDissolve=crafted-dissolve; barFill=crafted-fill; barBackground=crafted-background; barBackgroundCap=crafted-cap; barHighlight=crafted-highlight; iconChat=crafted-chat; iconQuestObjective=crafted-quest. |
| Ownership/lifecycle | Fresh tables (not rawequal), mutate every returned key without changing retained/new snapshots; host replacement quality=4 and iconInventory=crafted-replaced-inventory, old snapshot retained; removal one nil, other item unchanged; separate environments quality=3 versus 5; recipe tuple (164,900001) cannot seed crafted item key 900001. |
| Four selector forms/security | Number, decimal string, item string, hyperlink; genuine VM secrets for all four present and four absent forms; denial under taint before lookup, retained wrappers and exact payloads, untainted recovery; secret boolean denial before type validation. Removal makes retained selectors unknown with one nil. No general name lookup or broader numeric grammar proven. |
| Original consumer | Whole original cached TOCs through load_addon, no overrides; 12-addon closure. Printed original Lua provenance and actual reagent/crafted nil baselines. Positive map print 900001 / professions-icon-quality-tier3-small; printed real result table and true/true flags. |
| Consumer observable effects | Real Texture, atlas professions-icon-quality-tier3-small, TOPLEFT relative to button TOPLEFT offset (-3,2), OVERLAY sublevel 7. Host-created real Frame hidden/shown/hidden drives visibility. Explicit boolean alwaysShowProfessionsQuality true then false. Quality-only clear hides/removes texture but preserves flags; full clear removes texture and resets flags; host map removal stays hidden and false/false. Not original professions panel or rendered-pixel acceptance. |

Source: c_trade_skill_quality.rs registers the public method for Retail/PTR, unwraps genuine secrets before kind validation, uses existing parse_item_id_from_val, clones independent environment-owned map records, pushes fresh exact projection or one nil. c_api/registration.rs:77 calls registration; state/sim_state.rs:156 declares the map; state.rs:310 initializes it empty. Parser c_api/item_spell/c_item.rs:35 supports tested forms. No external rilua/vendor source read. Historical “unexecuted” test header comments do not override recorded runtime outcomes.

[EXIST] PASS — six existing artifacts per scope and essential authored tests/source present and read.
[SUBSTANTIVE] PASS — real supplied map, native query, Lua field/security/lifecycle assertions; not auto-nil acceptance.
[WIRED] PASS — public registration plus existing integration binary's exact named cases; original consumer calls public query.
[ANTI-PATTERN] PASS — TODO/FIXME/HACK/XXX: zero in six fully read essential source/test files; bounded scan, not whole-project audit.

## Diagnostics (not conflated)

All 12 full warning lists empty. All 15 Lua-error lists and records empty; all 15 count maps empty. **0 warnings, 0 recorded Lua errors, 0 Lua error records, 0 error-count entries.** Distinct diagnostics: **19 nil observations, 1 missing requirement**. Nil counts: Fonts_Shared 4, PrintHandler 1, Narration 7, SharedXML 7; other eight addons 0. Missing requirement: SharedXML public DebugBarManager.lua:106 CNamespace C_Reveal. Every full attributed nil/missing diagnostic is preserved verbatim in proof.json. These are not warnings/errors, but must not be erased by a “zero diagnostics” claim.

No callback, event-delivery, or unregistration assertions exist in this acceptance set; **no callback acceptance claimed**.

## Receipt and historical boundaries

Reused existing PASS integration receipt; current stat matches both recorded stat snapshots and all three runtime receipt_stat_reused entries: dev 66306, inode 148916762, size 471394760, mtime_ns 1791943878167372238. Zero new hashes or binary content reads. Receipt associates sealed bytes with recorded revision/profile; no fresh cryptographic authentication of source tree, archive, libraries, vendor cache, or runtime transitive inputs.

Historical initial RED: two separate one-test failures at source 8b46433a4568e26458a15907ea79636e38ff65f9, exits 101: omitted/nil rejection first assertion; untainted secret-nil required validation first assertion. Later assertions were unreached then, not previously proven. Historical positive RED source 6c38a82739dbf27fc58c3c801c32a0f0fc98d21c: owned 0/5 (positive returned nil); consumer 0/1 at original positive-map boundary after actual public baselines, not setup failure. Old consumer result scope warns about possible setup failure; observed stderr classifies actual failure as “positive supplied item must be crafted, not auto-nil.” Original positive print was nil/false/false. Historical RED remains preserved and is not current GREEN failure count.

Two Retail GREEN build admission denials (01:59:21 and 02:05:28 UTC October 14, 2026) remain preserved, allowed=false, zero launches. They are not executions, passing tests, or hidden retries in these eight cases. Original evidence files untouched.

## Separate pending / unproved

PTR GREEN remains unexecuted/not accepted here. Four preservation scopes remain pending separate acceptance; no results imported into this report. Default check, fmt check, and startup acceptance remain pending. Native catalog/classification, item-name lookup, rendering, full original professions panel/parent, callbacks/event delivery/unregistration remain unproved. This PASS applies only to the eight existing Retail producer runtime cases above.
