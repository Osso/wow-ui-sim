# Forever cfg failure and function gates

## Exact failed repair boundary

[Original independent report](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-cfg-independent-report.md) verifies canonical `0b5c69d6f`, copied as `811a0491b` in the source worktree. Enabling the two module declarations removes E0433 but reveals **RED E0609**: `addon_messages.rs` references Retail-only `bnet_custom_message`; `c_combat_log.rs` references Retail-only `combat_log_restricted`. Exact Forever source-model invocation exits 101; zero tests executed. Module availability does not establish function-body/backing-field availability.

| Gate | Retained proof at failed repair | Remaining boundary |
|---|---|---|
| Forever source model / UnitName | Build RED E0609; zero assertions | Configured-name behavior unproven |
| Forever addon senders | Source assertions inspected only | Success/rejection, ordered logs and no inbound echo unproven |
| Removed combat-log getter | Source assertions inspected only | Public/raw/legacy absence unproven at runtime |
| Retail-only restriction function | Registration remains Retail-only, static evidence | No runtime Forever non-exposure proof |
| Retail regression | Focused `--lib` check exit 0; aggregate test E0432 disabled iced | Zero Retail runtime assertions; formatting exit 0 is separate |

The report preserves exact argv, cwd, revisions, times, compiler warnings and failure sites. [Retention manifest](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-cfg-independent-retention.json) binds byte-identical copies of report, full streams, receipts, equivalence and proof-scope hashes. Post-command hashes remain post-command evidence, not invented pre-command snapshots.

## Follow-up: bounded compile/state PASS

Canonical `1044215d0`, copied as `2218c0f41` in the source worktree, gates Retail-only BNet bodies/constants/import and `register_restriction` to their existing Retail callers and state. No state or registrations enabled. [Independent follow-up](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-bodies/forever-bodies-independent-report.md) records exact Forever source-model command exit 0, **1/1**: before test-dispatched login, `UnitName('player')` reads Ada then Grace while a separate environment retains Linus. Both E0609 errors gone. Fresh formatting and changed-attribute readability pass; prior Retail library check reused because every added predicate evaluates true there and enabled bodies are unchanged.

[Full retained receipts](../../../data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/forever-bodies/main-retention.json) keep this GREEN epoch separate from E0433/E0609 RED. Seven library, one binary and six vendor warnings remain unsuppressed; newly observed Forever warnings do not establish regressions. Chat/removal API execution, real pre-login lifecycle, native tuple/security/identity and loaded UI remain UNPROVEN. Aggregate-only API tests cannot supply headless proof: disabled iced blocks their harness, and the sender helper also requires Retail BNet. No broad acceptance.

## Standalone execution target

`patch-tests/forever_runtime_contracts.rs` is registered as the Forever-only `forever_runtime_contracts` target. It constructs direct `WowLuaEnv` environments without GUI/BNet dependencies and checks ordered addon/logged/whisper records, independent-environment isolation, invalid-prefix no-append behavior, and the three explicitly removed public/raw/legacy combat-log getter lookups. These assert existing simulator contracts, not native/security/transport behavior. No runtime implementation or Retail restriction registration changed. Target execution and independent verification receipts are pending; the earlier proof epochs above remain unchanged.

## See also

- [Narrow repairs](narrow-validator-discovery-and-forever-cfg-repairs.md).
- [Integrated source/factory proof](integrated-source-and-factory-proof-2026-10-09.md).
