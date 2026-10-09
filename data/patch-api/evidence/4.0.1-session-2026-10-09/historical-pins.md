# Bounded historical 4.0.1 pins

`pin-builder.py` captures revision 567654d75 for accounting/source inputs; each targeted command retains its own recorded revision. Recorded revision aliases, original commit objects, recursive scope tree-entry inventories, and Git blob identities are retained compactly. Runtime code remains pinned by cryptographic tree identity, not copied as a second source checkout.

`historical-inputs.json.gz` contains the own source/register/extract/ledger, exact tools used to reproduce them, own tests/changed runtime files and the explicit retail successor-register set. `historical-scopes.json.gz` ties each selected input and command revision to its original Git commit/root/scope trees. Both archives and own evidence files are SHA-256 sealed in `historical-context.json`; every retained file is below 5 MB. No original commit lookup or mutable later-register enumeration is needed. Existing `patch_audit_pin_trees.py` object/tree helper is loaded from pinned bytes.

`validate.py` derives row/status/gap/successor counts from retained data, reproduces only the own register and extract using the recorded flags, validates receipts/logs and exact RED/GREEN/negative deltas. It does not run runtime tests, any prior validator, broad suite, current-head gate or native-client probe. Adding later state cannot silently enlarge its recorded proof scope. Current raw 4.0.1 wikitext must remain byte-identical; integration changes to successor lists or gap ledgers get separate new evidence, not retroactive credit.

Source/log/archive tamper fixtures mutate disposable copies only. Other Classic profiles and Mists cached UI execution remain unverified. Main owns final gates and replacing four queued successor placeholders.
