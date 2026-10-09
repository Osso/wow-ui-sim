# Retail Patch 3.3.0 source accounting

Audit frozen page 522376/revision 6055853, not Wrath Classic 3.4.x. [Audit](../wiki/investigations/patch-3-3-0-api-audit.md).

## What it must do
- [x] Preserve literal inventory, full inline XML/attribution extract, signatures and prose as separate occurrence IDs.
- [x] New parser/extractor behavior is opt-in; base default outputs/errors remain unchanged.
- [x] Separate callable publication from backing-model/signature/native parity; apply only ordered same-line successors.
- [x] Preserve original ledger/gaps/parsers/logs independently of closures; replay without Git/target/current files and reject serialized tampering.
- [x] XML motionScriptsWhileDisabled applies the existing button flag before OnLoad, preserves template inheritance and explicit false; Lua mutation still uses the same state.

## How it works
- [Audit](../wiki/investigations/patch-3-3-0-api-audit.md)
- [Portable handoff](../../data/patch-api/evidence/3.3.0-session-2026-10-09/handoff.md)

## Implementation inventory
- `tools/gen_patch_wikitext_register.py`, `tools/extract_patch_non_inventory.py`: opt-in 2009 summary inventory/markup.
- `data/patch-api/sources/3.3.0-*`: frozen register, extract and current ledger; original evidence/closures separately sealed.
- `src/xml/types.rs`, `src/lua_api/globals/template/direct.rs`: deserialize/apply literal motion bool to existing state.
- `src/loader/xml_frame/setup.rs`, `src/lua_api/globals/create_frame/template_chain/runtime.rs`: apply attribute before scripts in both creation paths.

## Tests asserting this spec
- `tools/test_patch_3_3_0_source.py`: literal frozen inventory, markup and default-output controls.
- `tests/patch_3_3_0_publication_sweep.rs`: own current-retail publication accounting; known gaps do not grant model credit.
- `tools/test_patch_3_3_0_portable.py`, evidence `validate.py`: fresh process, dynamic accounting and serialized tamper/restore controls.
- `tests/patch_3_3_0_motion_xml.rs`: actual addon loading, getter, OnLoad, template inheritance and Lua mutation.
- `src/iced_app/mouse_hover_tests.rs`: existing real-pointer disabled enter/leave flag behavior.

## Known gaps (current cycle)
- [ ] Two Texture file-dimension publication methods lack actual-file metadata/semantics; no zero stub or layout-size alias.
- [ ] Fifteen current prose rows and seven signature contracts lack bounded behavioral/native proof; occurrence-specific reasons remain in ledger.
- [ ] Main integration supplies real queued 3.3.3/3.3.5/4.0.1 successors and owns acceptance.

## Out of scope
Native historical-client parity, linked diff/forum expansion, speculative models, shims/fallbacks, vendor/cache/Wowless edits, Classic supersession and final integration acceptance.
