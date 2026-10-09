# Patch 1.13.5 portable historical SOURCE replay

`originals.tar.gz` copies every original file plus `seals.json`. Extract to a fresh directory without Git, target or current tools. Use absolute Python3.11+ with empty PATH and PYTHONDONTWRITEBYTECODE=1; every command cwd `/home/osso/.worktrees/wow-ui-sim-p1135-page` (cwd is command context, never an input lookup).

Run copied `audit.py` and `test_source_accounting.py`. Run copied `historical-tools/gen_patch_wikitext_register.py 1.13.5 <copied>/source.wikitext 3835002 <copied>/reproduced-register.json`; compare exact bytes with copied default-register.json. Run copied `audit.py default-extract`: expect the retained ValueError string, empty stdout and no extracted output. Shared-tool defaults flags[]; no fallback or template expansion.

`test_portable.py --archive <archive> --scratch <fresh-own-build-directory> --receipts <new-json>` executes that fresh-process replay with empty PATH. It also mutates copied ledger.json and green.log separately, requires serialized seal rejection, restores exact bytes in finally, checks all original seals and reruns copied validator after each restoration.

Original seals/proof log are immutable. Portable/current GREEN receipts and their receipt-seals.json are later evidence, not historical backfill. SOURCE only; no native/model/runtime/integration/final acceptance.
