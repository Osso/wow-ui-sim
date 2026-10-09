# Retail Patch 3.3.0 source accounting

Audit frozen page 522376/revision 6055853, not Wrath Classic 3.4.x. [Audit](../wiki/investigations/patch-3-3-0-api-audit.md).

## What it must do
- [ ] Preserve complete literal publication inventory, inline XML, attribution, signatures and prose.
- [ ] New parser/extractor behavior is opt-in; existing outputs remain unchanged.
- [ ] Separate callable publication from backing-model/signature/native parity.
- [ ] Preserve original ledger, gaps, parsers and logs with portable no-Git replay and serialized tamper controls.

## How it works
- [Audit](../wiki/investigations/patch-3-3-0-api-audit.md)

## Implementation inventory
- `tools/gen_patch_wikitext_register.py`: opt-in 2009 summary inventory.
- `tools/extract_patch_non_inventory.py`: opt-in attribution and inline XML retention.
- `data/patch-api/sources/3.3.0-*`: frozen register, extraction and accounting.

## Tests asserting this spec
- `tools/test_patch_3_3_0_source.py`: literal frozen inventory and markup/default-output controls.
- `tests/patch_3_3_0_motion_xml.rs`: actual addon XML loading, getter, inheritance and Lua mutation.

## Known gaps (current cycle)
- [ ] Targeted publication/accounting proof pending.
- [ ] XML motionScriptsWhileDisabled applies the existing button flag before OnLoad, preserves template inheritance and explicit false; behavioral RED captured, GREEN pending.

## Out of scope
Native historical-client parity, linked diff expansion, speculative models, shims, Classic supersession and final integration acceptance.
