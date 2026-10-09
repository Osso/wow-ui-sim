# Historical validator boundary

`python3 data/patch-api/evidence/4.3.4-session-2026-10-09/validate.py`
validates the original sealed artifact set and sparse archived UTF-8 inputs only.
`test_validate.py` exercises detached evidence and serialized source, ledger,
archive, receipt, log, negative-result, supersession and acceptance-seal tampering;
mutations happen in temporary copies and all original copy bytes are restored.

Default register and extract replay uses the archived parsers in-process, without
network, Cargo, present-day shared files, Git checkout or original commit objects.
Archive SHA-256 and Git blob identities are computed from bytes. Successor,
observation, ledger, header and gap counts derive from historical data. Current
ledger/fixture changes cannot retroactively alter this proof.

`validator-acceptance-pins.json` seals validator, tests, this note and original
`session-seals.json`; it deliberately does not seal itself. Existing 23 seals and
74 archived inputs remain unchanged. Pins express artifact integrity, not an
independent trust anchor or a recorded successful acceptance invocation.

Historical results remain 11 observations, 12 ledger IDs, 64 retail successors,
three supersessions and seven gaps; same-cardinality fabricated control has eight.
No fresh runtime/model/native parity, later GetSessionTime closure, fresh-clone,
global-clean, integration, broad checks or final coordinator acceptance credit.
The sealed README's statement that no validator existed describes its earlier
capture boundary; this additive note records the new component without rewriting it.
