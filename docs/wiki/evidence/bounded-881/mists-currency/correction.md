# Mists014 withholding correction — 2026-10-10

Supersedes the missing-panic-evidence claim in [retained report](report.md) and [historical main review](main-review.json); those snapshots and original hash manifests remain unchanged.

Original014 stderr contains panic details with parenthesized thread IDs. Main's parser wrongly expected thread name immediately followed by `panicked`, missed those diagnostics, and incorrectly withheld available assertion evidence. Original audit860 field RED is accepted: assertion120 observed namespace/legacy **nil/false versus true/true**; assertion149 observed **nil/nil versus 90/90**. Positive wrapper assertion192 and negative assertion46 report generic nil-call Lua63 after size-equality and failure-status assertions respectively. Exact TokenFrame_Update failing callee remains unproven.

Corrected source: `/home/osso/.local/state/wow-ui-sim/verification/mists-currency-observer-current/20261010T043019Z/audit/main-review.json`, correction timestamp `2026-10-10T05:09:47.111205+00:00`. [Updated review snapshot](main-review-corrected.json) preserves both original mistaken withholding and explicit correction; [new retention record](correction-retention.json) hashes this snapshot separately, without altering original retention/seals.

Supplemental518 diagnostic audit: `/home/osso/.local/state/wow-ui-sim/verification/mists-currency-diagnostic-current/20261010T045459Z/audit/report.md`. Separate revision `5184a8cf8feea9a820a15264604a6af303bb5a9d`, **1 PASS / 4 FAIL**, exit101: confirms namespace watched nil/nil, legacy false/false, cap nil/nil. This adds row diagnostics; it is not replacement evidence for missing original output, because original output was present. Parsed symbol keys `[]` do not establish an empty observer interval or exact callee.

Original014 aggregate **26 PASS / 4 FAIL** and Honor/Dialog singleton proof unchanged. Current production field fix4231 and fixture4b remain **GREEN unverified**; source518 autohide remains **pending**. No current CurrencyGREEN or parent acceptance. Sanitized counts/data only; no raw streams/vendor payloads copied.
