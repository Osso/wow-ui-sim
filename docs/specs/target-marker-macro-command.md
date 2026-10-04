# Target-marker macro command (bounded Retail 12.0.5 slice)

`C_Macro.RunMacroText('/tm ...')` selects numeric marker commands against live simulator state. Source: [12.0.5 prose](../../data/patch-api/sources/12.0.5-api-changes.txt), line 98 (`prose-2026-03-25-098`). Runtime flow: [Lua API](../lua-api.md). Bounded simulator execution does not establish full native macro compatibility.

## What it must do

- [ ] A selected `/tm [combat] 1` assigns marker 1 and delivers the existing `RAID_TARGET_UPDATE` notification without a Lua error; changing combat state changes selection.
- [ ] An unselected condition preserves unit icons and emits no marker notification. Existing comma conjunction, supported modifier predicates and first-matching semicolon clauses select commands from current state.
- [ ] Unconditional `/tm 2` assigns marker 2; `/tm 0` clears it. Selected `@focus`, `target=focus` and `unit=focus` selectors drive both predicate evaluation and the destination unit.
- [ ] **INFERRED simulator policy:** invalid, malformed, unselected or out-of-range decimal integer commands are atomic no-ops. A later valid command still succeeds. Destination absence follows the existing marker API's no-op policy; no synthetic unit is created.
- [ ] Preserve existing marker assignment/movement and synchronous notification semantics. Their equivalence to native repeated-assignment/toggle behavior remains unproven.
- [ ] `SecureCmdOptionParse` returns the selected text and explicit unit selector to the unchanged cached `/tm` handler. **INFERRED from cached consumers:** successful clauses without selectors return nil as the second value, not an invented target; `@unit`, `target=unit`, and the existing `unit=unit` alias return the selected unit. A rejected clause cannot leak its selector into a later match. Preserve existing first-value predicates, Rust resolver default destination, and no-selection return shape. Native parser implementation/return-arity documentation is not present in the inspected cache.

## How it works

- [Lua API](../lua-api.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/spell_macro_verbs.rs`: Retail-gated `/tm` command dispatch.
- `src/lua_api/globals/security/cmd_option.rs`: shared condition selection and unit-selector extraction; no second parser.
- `src/lua_api/globals/security/mod.rs`: crate-visible parser entry point.
- `src/lua_api/globals/targeting_verbs.rs`: existing GUID-keyed marker mutation/query and notifications; unchanged.

## Tests asserting this spec

`tests/mouse_tm_commands.rs` — one auto-included `integration` module, feature-gated for `retail-12-0-5`. Assertions cover marker queries, actual stored unit icons, queued events and synchronous Lua event callbacks.

`tests/cmd_option_selected_unit.rs` — direct text/unit results with and without selectors, rejected-clause isolation, and the entire unchanged cached `SlashCommands.lua` followed by its registered `TARGET_MARKER` callback. This exercises the vendor handler and existing marker/event APIs, not full chat input or addon loading. Round-100 follow-up: 0/2 RED with tests only, 2/2 GREEN after the parser producer; prior B98 proof below does not cover it.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 5 FAIL. GREEN: 5/5 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. Macro-runner path only; vendor slash handler compatibility not earned. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-098 bounded-coverage under capability `target-marker-macro-command`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [x] Round-100 direct parser and unchanged cached vendor handler tests passed 2/2 after observed 0/2 RED. Full chat/addon-load parity remains unproved.
- [ ] Native permission/taint/secret-argument matrix and exact invalid-input behavior lack probe evidence.

## Out of scope

This bounded numeric slice does not cover native `!`/`~` prefixes, toggle parity, slash aliases, Lua `tonumber`'s broader coercion, adjacent-bracket alternatives, unsupported macro predicates, or a fully loaded chat addon/input path. Existing parser predicates are reused, not certified wholesale (notably placeholder pet/vehicle existence and coarse group matching are not native proof). The follow-up supplies `SecureCmdOptionParse`'s explicit second return to the real cached handler; the shared Rust resolver retains its default destination for simulator macro dispatch. Explicit-selector nil policy and existing last-selector-wins/`unit=` behavior are bounded simulator policies, not a native grammar certification.

Row `prose-2026-03-25-099` is skipped, not counted covered: cached declarations contain `IsMouseOver` but no exact `IsUnderMouse` declaration. No invented alias or geometry policy is supplied.
