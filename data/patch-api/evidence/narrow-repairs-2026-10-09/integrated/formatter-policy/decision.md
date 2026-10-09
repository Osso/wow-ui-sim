# Formatter policy conflict

## Scope and evidence

Read-only review of `/home/osso/Projects/wow/wow-ui-sim` because the requested canonical path `/home/osso/Projects/wow/wow-ui-sim4142f3d41` does not exist in this environment. No tests, builds, network, delegation, or repo edits were performed. The supplied `/tmp/duration-fixture-compiled-independent/report.md` records a compiled Retail artifact and failure, but its visible report excerpt does not include the exact assertion panic; the user-provided summary says the custom-table `SetFormatter` rejects `NumericFormatter expected`.

## Facts

- `src/c_api/duration_text_binding.rs` (`register`, around lines 338–347): binding registration returns early for every profile except Forever, Retail, and PTR. Retail/PTR use `modern_methods` only when `ACTIVE_INTERFACE_VERSION >= 120100`; Forever always enables it.
- Same file, `SetFormatter` implementation: the underlying method stores the passed formatter without type validation.
- Same file, modern-method setter wrapper: for `SetFormatter`, it requires `type(value) == "userdata"` and callable `value.FormatNumber`; otherwise it errors `NumericFormatter expected`. This wrapper is installed only when `isPatch1207` is true. Thus a compiled Retail 12.1+ profile reaches the typed setter; the legacy table/function callback branch is not reachable through `SetFormatter` there.
- Same file, legacy `GetFormattedText` branch (when `isPatch1207` is false): recognizes userdata `FormatNumber`, a function formatter, or a table with `Format`; function/table callbacks receive the original duration input. This is a distinct legacy implementation path, but its profile reachability is not established merely by the current Retail artifact.
- `docs/specs/duration-text-binding.md`, “Bounded secret-duration handoff”: explicitly says function/table formatters retain their existing duration-object/input argument. The section is explicitly labeled informed guesses/simulator policy, not native probe evidence. “Tests asserting this spec” associates existing callback tests with that policy.
- `docs/specs/numeric-rule-formatter.md`, “What it must do”: requires native `FormatNumber` output through bindings “while retaining the existing custom table `.Format` contract.” The test description says the custom callback reads `GetRemainingDuration()` and clarifies that it tests modeled callback behavior, not native parity.
- `tests/numeric_rule_formatter.rs::numeric_rule_formatter_updates_duration_binding_text` sets a custom table after exercising NumericFormatter and expects `custom:8.2`. Under the present 12.1+ typed setter, it fails before that output assertion. The documented fixture behavior and the active modern setter contract conflict in this profile.
- The supplied compiled artifact record says features include `client-retail`, `retail-12-1-0`, and `numeric-rule-formatters`; it establishes this is a Retail build, not all-profile reachability.

## Governing contract and prior finding

The runtime’s explicit modern contract is a typed `NumericFormatter` setter. The custom function/table callback contract exists in the legacy formatter branch and is locally documented as preserved behavior, but that branch is bypassed by the modern typed path. Therefore the prior recommendation to retain `custom:8.2` by adapting the callback was inapplicable to this reproduced Retail 12.1+ failure: it reasoned from the legacy callback branch without accounting for the modern setter override. **Reject the prior finding for this failure boundary.**

Current typed validation is intentionally sourced by the `isPatch1207` setter wrapper and is consistent with the modern API path as implemented. However, the reviewed local specs do not explicitly state that this typed behavior supersedes the custom-table requirement, nor provide native-client evidence adjudicating the contradiction. Do not infer native compatibility beyond the source and compiled-profile facts.

## Explicit requirements vs assumptions

- Explicit local requirement: preserve custom table `.Format` contract (`docs/specs/numeric-rule-formatter.md`).
- Explicit implemented behavior: modern setter rejects non-NumericFormatter inputs (`src/c_api/duration_text_binding.rs`).
- Explicit local scope marker: callback handoff is an informed simulator guess, not native probe proof (`duration-text-binding.md`).
- Assumption not established: that the old custom-table contract should remain valid on Retail 12.1+ despite typed `SetFormatter`.
- Assumption not established: that changing runtime to allow tables would preserve the native compatibility target.

## Smallest correct next action

Do not change runtime or weaken the typed assertion. Resolve the local contract ownership before implementation: either document that custom table formatting is legacy-only and update/remove its conflicting modern-profile fixture expectation, or provide authoritative/native evidence that modern `SetFormatter` accepts custom tables and then revise the runtime/spec accordingly. Since native compatibility is the governing target and the supplied evidence does not establish table acceptance in Retail 12.1+, the smallest safe action is to clarify/document scope and make the fixture profile-specific to the legacy path; a user decision is needed only if they intend to retain custom-table support on modern Retail despite the typed contract.
