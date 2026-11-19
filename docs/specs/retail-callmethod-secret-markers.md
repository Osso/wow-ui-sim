# Retail CallMethod secret markers

Evidence: tracker lt-3b33, R88ae.4 (VERIFIED, Retail 12.1.5.70688):
`issecretvalue` reads the value-slot secret marker, not execution taint.
R88ae.1 separately verifies tainted table construction/writes do not make the
table secret; R88ae.3 verifies a `loadstring` closure is published nonsecret.
The canaccess*/scrub* behavior change is INFERRED from their shared marker
lookup; R88ae.4 directly verifies only `issecretvalue`.

RED proof is simulator-surface proof: `Frame:CallMethod` is simulator-only,
not Blizzard's `HANDLE:CallMethod`.

- INFERRED (simulator mapping of R88ae.4): Retail CallMethod preserves caller taint and dispatch/argument validation but
  never marks ordinary strings secret. It returns the method's values directly.
- INFERRED (simulator mapping of R88ae.4): Retail's `Frame:CallMethod` implementation does not call the private
  `__sim_mark_secret_value` bridge.
- VERIFIED (R88ae.4, marker predicate) / INFERRED (simulator mapping to VM
  wrappers): Retail shallow secrecy reads authentic VM secret wrappers only. Evidenced
  host-produced secret unit identities remain wrappers; their tests are unchanged.
- UNKNOWN (native): other profiles retain their existing CallMethod and secrecy
  behavior. This evidence does not establish their native behavior.

## Pre-change dependency audit

The three CallMethod marker calls reach `value_access::mark_secret_values` and
`secret_values::mark_secret_value`. The only other writer is
`unit_misc::identity_output`. In a Retail build with `retail-12-0-5`, it is
excluded in favor of host wrappers. Without `retail-12-0-5`, it still marks
through the registry, which this change no longer reads; party-name secrecy
in that historical configuration is UNKNOWN (tracked as lt-49cd).
`value_is_secret_shallow` feeds issecretvalue, canaccessvalue,
canaccessallvalues, scrub, scrubsecretvalues and recursive canaccesstable.

No tests access the registry directly. The legacy
`execute_callmethod_propagates_tainted_args_to_insecure_method` expectation is
retained unchanged on non-Retail. Retail instead runs
`security_api::retail_tainted_callmethod_keeps_plain_strings_public`, whose body
is identical for RED and GREEN. The table CallMethod test and party identity,
wrapped-secret and accessibility tests remain unchanged. Other CallMethod search
hits in AccountStore source tests refer to CallMethodOnNearestAncestor, not this
method or registry.
