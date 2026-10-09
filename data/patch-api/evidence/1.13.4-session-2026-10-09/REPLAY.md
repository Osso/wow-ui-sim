# Portable frozen Patch1.13.4 development replay

Outer commands run from `/home/osso/.worktrees/wow-ui-sim-p1134-page`. `test_portable.py` extracts `originals.tar.gz` into fresh temporary evidence-only directories, then executes copied historical Python scripts using the absolute interpreter, empty PATH, and the copied directory as worker cwd. No Git, target, current tools, source checkout lookup, cache copy or network. The archive does not contain `.git`, `target`, or root `tools`; `historical-tools` are retained exact snapshots. Current Era binary/test is NOT rerun in this portable SOURCE replay.

```
python3 -B data/patch-api/evidence/1.13.4-session-2026-10-09/test_portable.py --archive /home/osso/.worktrees/wow-ui-sim-p1134-page/data/patch-api/evidence/1.13.4-session-2026-10-09/originals.tar.gz --scratch /home/osso/.worktrees/wow-ui-sim-p1134-page/data/patch-api/evidence/1.13.4-session-2026-10-09/.scratch --receipts /home/osso/.worktrees/wow-ui-sim-p1134-page/data/patch-api/evidence/1.13.4-session-2026-10-09/fresh-portable-proof.json
```

The receipt destination must not exist. At any relocated evidence-only extraction, `python3 -B validator.py`, `python3 -B test_source_accounting.py`, and `python3 -B test_successors.py` use only copied inputs. Generator defaults reproduce saved register bytes; extractor defaults reproduce saved text. Serialized omission of a ledger contract and fabricated SOURCE GREEN log must each fail a seal check, then restoration must recover exact original bytes and pass all original seals/validator.

`seals.json` is created once and never updated. It seals original input/code/log/receipt bytes, including frozen queue and separate actual successor application, but not itself. The archive adds that exact map. Later portable GREEN/validator receipts and archive/map hashes are separate `receipt-seals.json`, never backfilled into original GREEN or queue. Documentation outside the sealed snapshots may gain later proof summaries.

Bounded targeted development proof only. Existing current Era model receipts establish host-slot lifecycle, not historical/native API semantics. Main owns integration/native/final gates and integrates1.13.4 before1.13.3.
