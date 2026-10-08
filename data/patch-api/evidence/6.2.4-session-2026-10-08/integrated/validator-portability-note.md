# Historical proof portability repair

`../PLAN.md` and `PLAN.md` were ignored, uncommitted scratch files (`PLAN*.md`), not retained evidence. Their entries were removed from `historical-preservation.json` and `artifact-hashes.json` respectively. No historical logs, results, receipts, counts, negative controls or rebase mappings were regenerated.

The original `historical-preservation.json` seal in `artifact-hashes.json` remains unchanged: validation checks its Git blob at the recorded runtime revision, then requires the current JSON to equal that original record minus exactly the scratch entry. Apart from removing its scratch entry, only `artifact-hashes.json`'s validator self-hash changed.

Shared source/tool/runtime blobs and prior validators are now proved at the recorded master, runtime, validator-scope and replacement revisions, using the existing rebase mapping. Own retained session records remain checked against their seals. Future unrelated commits cannot invalidate the historical input scope.

Run `python3 -B tools/check_patch_validators.py` to test a clean detached checkout and a committed unrelated later audit. See [validator portability](../../../../../docs/wiki/investigations/patch-audit-validator-portability.md).
