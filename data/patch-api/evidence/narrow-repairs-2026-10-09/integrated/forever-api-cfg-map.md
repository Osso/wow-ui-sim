# Read-only compile E0433 root-cause/minimal-fix map

## Scope and inspected evidence

Canonical checkout `/home/osso/Projects/wow/wow-ui-sim` at user-reported `894680abe`; source failure independently reported at `/home/osso/.worktrees/wow-ui-sim-p1601-source` revision `e039d6395`. No production files edited; no commands, tests, network, delegation, or cwd switch performed. Read `docs/wiki/index.md` and the relevant Cargo feature contract, C API module declarations, registration code, and implementation files.

The independent report records the exact failed invocation:

```text
cargo test --offline --locked --no-default-features --features client-wowforever --test patch_1_60_1_source_model -- --nocapture
```

It failed compiling the library before running any tests, with E0433 at `src/c_api/registration.rs:57` and `:59`.

## Root cause

`Cargo.toml` defines `client-wowforever` as its own profile feature bundle. It includes shared capabilities such as `timed-signal-maps`, `aura-xml-widgets`, and `on-update-modes`, but does **not** include `retail-12-0-0`. In `src/c_api/mod.rs`, `addon_messages` and `c_combat_log` are each declared only under `#[cfg(feature = "retail-12-0-0")]`. Meanwhile, `register_interaction_tables` has Forever-only registrations calling both modules:

- `#[cfg(feature = "client-wowforever")] super::addon_messages::register_chat(state)?;`
- `#[cfg(feature = "client-wowforever")] super::c_combat_log::register_publication(state)?;`

Thus Forever compiles the call sites but cfg-removes their module names. This is the direct cause of both E0433s. Do not infer broad retail API sharing from a source/bluepost statement: the feature graph and these explicit Forever call sites are sufficient evidence.

## Existing registration intent (literal code evidence)

- `addon_messages.rs::register_chat` is itself gated by `client-wowforever`; it registers `C_ChatInfo.SendAddonMessage` and `SendAddonMessageLogged`. Its implementation uses Forever-gated chat argument validation and calls shared `SimState` state for chat routing and message-log recording. The module also contains BNet helpers; inspection shows `register_bnet_chat` exists, but this Forever call site specifically registers chat, not BNet. No need to change BNet exposure.
- `c_combat_log.rs::register_publication` is gated by `client-wowforever`; it obtains `C_CombatLog` and adds `GetCurrentEventInfo` to `__wow_removed_keys`, marking that member absent from the public surface. `register_restriction`, which exposes `IsCombatLogRestricted`, is a separate registration call under `retail-12-0-0` only.
- The Forever registration block calls `register_publication` but not `register_restriction`. Therefore making the module available for Forever need not publish the Retail restriction API; function-level behavior and call-site gates already distinguish the sets.
- The current module-level cfg for `c_combat_log` is the remaining compile barrier despite the function-level Forever gate. For `addon_messages`, the existing implementation already has Forever-specific cfg attributes while the module declaration has not been extended to expose it.

## Smallest consistent fix map (recommendation, not applied)

Change only the module declaration gates in `src/c_api/mod.rs` so the two modules are compiled when their respective actual consumers are enabled, preserving existing registrations:

- `addon_messages`: compile for the existing Retail module consumer(s) **or** `client-wowforever`, expressed as `#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]`.
- `c_combat_log`: same union gate, `#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]`.

Keep all registration call-site cfgs unchanged. In particular, do not gate the whole `register_interaction_tables` block or Retail-only `register_restriction` under Forever, and do not add `retail-12-0-0` as a dependency of `client-wowforever`: that would enable a much larger Retail registration/API surface and contradict the current explicitly distinct feature bundle. No backing state or enum gate needs adjustment for the two compile errors: `addon_messages` uses existing `SimState` fields and the Forever cfg-local validation/constants; the compile failure cites unresolved module names, not missing fields or enum variants. Preserve all other profile gates.

## Verification required after implementation

1. Re-run the exact failed offline/locked `cargo test ... --test patch_1_60_1_source_model -- --nocapture` target to prove it now compiles and executes; inspect actual test assertions/results. The prior report's expected UnitName values are unproven because that invocation did not reach the harness.
2. Add/run a focused Forever behavioral check, or use existing target coverage if it already asserts these exact observable surfaces: `C_ChatInfo.SendAddonMessage` and `SendAddonMessageLogged` are registered and exercise the intended local validation/message-log behavior; `C_CombatLog.GetCurrentEventInfo` is absent as specified by removed-member publication, while `IsCombatLogRestricted` is not newly exposed by this fix. Check concrete returned values/state rather than internal implementation shape.
3. Compile/check a Retail profile that enables `retail-12-0-0` to confirm its preexisting API/registration set remains intact. Keep profile verification targeted; do not run broad suites for this cfg-only correction.

No production test result or implementation verification is claimed by this read-only report. The independent attempt had zero tests executed and zero assertions observed. It also reports the unrelated native-source build/interface mismatch; this map does not reinterpret that as API parity evidence or expand scope to address it.
