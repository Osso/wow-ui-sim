# Independent bounded 3.3.0 verification — PASS

Verified 2026-10-09 in `/home/osso/.worktrees/wow-ui-sim-p330-source`, HEAD `898d3fe1f89d12d947d5fd62773c3cc89355f558`, rebased onto `7a7a29519`. Scope: literal XML motion flag, existing model, frozen source/accounting, merged extractor compatibility. NOT whole integration/final acceptance or native 2009 parity.

Exact argv, cwd, revision, environment overrides, complete stdout/stderr, stream hashes, baseline code, per-input/output hashes and proof-scope hashes: `/tmp/p330-independent-receipt.json`. All commands ran without Bash or cwd-switch. No edits to repository, delegation, model CLI, rebase, push, merge or deployment. Working tree remains clean; HEAD unchanged.

## Fresh command proof

| Command | Result |
|---|---|
| `cargo test --test integration patch_3_3_0_motion_xml -- --nocapture` | exit 0; 1 passed, 0 failed, 10691 filtered; 0.14s test body |
| `cargo fmt --check` | exit 0; empty stdout/stderr |
| `python3 -B data/patch-api/evidence/3.3.0-session-2026-10-09/validate.py` | exit 0; fresh process, frozen receipts/accounting validated |
| `python3 -B -m unittest discover -s tools -p test_patch_3_3_0_source.py -v` | exit 0; 2/2 |

Python ran with `PYTHONDONTWRITEBYTECODE=1`. Necessary existing controls: `test_patch_cataclysm_register.py` 3/3; `test_extract_patch_non_inventory.py` 37/37; `test_gen_patch_wikitext_register.py` 35/35, each through the same unittest argv pattern above, exit 0. Complete test-name output retained in JSON.

Native output:

```text
running 1 test
test patch_3_3_0_motion_xml::patch_3_3_0_motion_xml_preserves_true_false_and_template_inheritance_before_onload ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10691 filtered out; finished in 0.14s
```

Six inherited iced-wgpu manifest key deprecation warnings; no non-vendor compilation warnings in this native invocation. No broad Rust suite, cargo check, Mists gate, startup/publication-all or parent portable gate run.

## Model coverage and boundary

| Requirement | Concrete proof |
|---|---|
| Literal true/false, omitted default, CheckButton | Fresh real TOC/XML addon load; Lua getter assertions, no loader warnings |
| Inheritance and instance false overriding true | Actual virtual base, derived false, inherited instance and explicit-false instance |
| Before XML OnLoad | Handler saves getter result; assertion observes true |
| Lua CreateFrame template path | Fresh CreateFrame(Button, base/derived) assertions observe true/false |
| Before Lua CreateFrame OnLoad | Source ordering: `template_chain.rs:280` applies properties before `:291` fires OnLoad; not a separate fresh OnLoad assertion |
| Lua mutation | Setter false followed by getter false on created inherited instance |
| Actual disabled pointer enter/leave | Original sealed 1/1 pointer test reused, not rerun, as requested |

Pointer receipt `closures/existing-motion-dispatch-green.log` explicitly records 1 passed/0 failed at `224109b55a0e29eec3adc63acbd1fc1118ce038e`. Fixture drives pointer outside→inside→outside, asserting disabled default enter/leave 0/0 and opt-in 1/1. All seven closure archive scope hashes match current files. Empty diff from original source330 `3528a63e4d616e1a1900246e415ad12c8ddc81a4` across the four runtime files, XML test, pointer fixture/helper and entire evidence directory. Additional empty source330 diff covers `src/iced_app`, widget backing code and motion API registration/implementation. This is compositional loader + unchanged-pointer proof, not a newly executed XML-to-pointer combined test.

## Artifact and readability checks

[EXIST] PASS — four runtime files and integration fixture exist; exact file hashes retained.

[SUBSTANTIVE] PASS — `src/xml/types.rs:181` deserializes `@motionScriptsWhileDisabled` into `Option<bool>`; `src/lua_api/globals/template/direct.rs:515` implements inherited base→derived→instance selection and updates existing `Frame.motion_scripts_while_disabled`. Explicit false is retained via Option selection, not truthiness. No new model or placeholder.

[WIRED] PASS — XML setup call `src/loader/xml_frame/setup.rs:362`; Lua template/runtime call `src/lua_api/globals/create_frame/template_chain/runtime.rs:606`; root template finalization reaches runtime direct properties before OnLoad. Existing API setters/getters at `text_attribute_event/attributes.rs:546,556` read/write the same field; pointer gate in `src/iced_app/mouse_drag.rs:5` reads it.

[ANTI-PATTERN] PASS — no added TODO/FIXME/HACK/XXX, empty bodies, warning suppression or commented-out implementation in the runtime delta.

Rust readability: changed lines reviewed against the supplied skill; no new violations identified. New helper has one inheritance loop and explicit mutation, four arguments, no excessive nesting. Existing large surrounding functions were not treated as new findings.

## Source, seals and accounting

Frozen page 522376/revision 6055853/time `2024-06-04T04:47:16Z`; current literal wikitext matches frozen hash `8aba4ed7056b03aca55eb5136011a54cd6794668d3374909d2da6e1696bb9267`. Historical validator confirms response/source equality, archived register/extract reproduction and both independent manifests/archives. Combined original + closure manifests retain 24 sealed files and 166 archived files; no seal rewrite.

| Ledger | IDs | Bounded | Pending | Metadata |
|---|---:|---:|---:|---:|
| Frozen original | 47 | 9 | 25 | 13 |
| Current with separate closure | 47 | 10 | 24 | 13 |

Only changed source row: `prose-undated-024`. Eleven inventory occurrences retain nine publication/absence matches and two Texture dimension gaps. Full extract has 29 rows: 13 metadata plus 16 originally pending prose; one existing-model closure leaves 15 current prose gaps. Seven signature contracts remain pending. Negative original receipt retains exactly three gaps versus original two. No native historical callable/default/signature credit inferred from publication.

Actual 3.3.3, 3.3.5, 4.0.1 successor registers each have zero `(section,symbol)` and zero bare-symbol overlap with own inventory. Current list is sorted, unique, 71 Retail registers: those three followed by original 68. No Wrath Classic supersession.

## Merged tool reproduction

Current retained registers reproduce byte-for-byte 77/77 using recorded flags. Current retained extracts reproduce 74/77; exact inherited exclusions are `12.0.5`, `12.0.7`, `12.1.0`.

Initial provenance-only replay omitted historically recorded preserve-examples mode on five pages and inventory-only on 12.1.0. Those exploratory failures are retained, not erased. Replays with `--preserve-examples` on 9.2.5/10.0.2/10.1.0/10.1.7/10.2.5 and `--inventory-only` on 12.1.0 resolve only these mode-selection differences. No source/tool edits and no broad rerun. Each corrected command/stream retained separately.

The three excluded extracts fail at `extract_patch_non_inventory.py:431` (12.0.5/12.0.7, retained text differs) or `:262` (12.1.0, unhandled description2 template). Their retained text/raw inputs are unchanged from canonical `7a7a29519`; default outputs/errors are unchanged. They remain failures, not positive reproduction credit.

Default generator/extractor return data and exception type/message match BOTH canonical merged base and original archived pre-change base on all 84 current raw page inputs. Fresh frozen validator separately replays original 77-input control. Both `cataclysm_labeled_inventory` and `wrath_summary_markup` opt-ins survive conflict resolution: 4.0.1 and 3.3.0 recorded CLI reproduction and focused fixtures pass.

## Limits

Texture file dimensions, registerForClicks token grammar, historical GUID representation, server-query transport/throttling, vehicle macro state and seven signatures remain outside proven model coverage. Native 2009 parity, main integration gates, parent portable gate and full handoff acceptance are not claimed.
