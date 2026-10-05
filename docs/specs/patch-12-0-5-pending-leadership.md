# 12.0.5 pending leadership combat restriction

Bounded follow-up to source row `prose-2026-03-25-074` in [retained page text](../../data/patch-api/sources/12.0.5-api-changes.txt). Four modeled leadership operations receive the addon-in-combat restriction. The aggregate row stays pending: original global names and conversion operations are not restored. [Audit](../wiki/investigations/patch-12-0-5-api-audit.md) owns row accounting.

## What it must do

- [x] Deny addon-tainted calls to `C_PartyInfo.PromoteToLeader`, `PromoteToAssistant`, `DemoteAssistant`, and `SetEveryoneIsAssistant` during live player combat, before role/leader mutation or events.
- [x] Permit secure calls during combat and addon-tainted calls outside combat, subject to existing host permission and argument rules.
- [x] Restore the caller's original taint after denial; permit later out-of-combat recovery in the same environment.
- [x] Preserve current March 31 chat-lockdown behavior for countdown, ready checks, ping restrictions and loot methods; leadership denial does not replace that separate policy.

**INFERRED:** the current namespace successors inherit the historical global leadership restriction. Simulator denial raises a nonempty runtime error; native error wording/convention is unspecified. Combat denial precedes existing argument authentication. No caller taint is cleared to execute the operation.

## How it works

- [Lua API](../lua-api.md)
- [Existing party operation contract](party-12-0-7-audit.md)
- [Chat-lockdown family contract](party-countdown.md#exact-row169-decision--bounded-predicate-acceptance)

## Implementation inventory

- `src/c_api/c_party_info.rs` — shared VM-taint/live-combat guard on four current leadership entry points.
- `tests/patch_12_0_5_pending_party.rs` — real roster mutations, denial atomicity, caller restoration, events and recovery.

## Tests asserting this spec

- `leadership_addon_combat_denial_preserves_roles_leader_events_and_caller`
- `leadership_secure_combat_and_addon_out_of_combat_calls_mutate_live_state`
- Existing `party_countdown`, `party_ping_restrictions`, `party_loot_method`, and `ready_check` suites retain the separate lockdown contract.

## Known gaps (current cycle)

- [ ] `ConvertToParty`, `ConvertToRaid`, `ConfirmConvertToRaid` have no real provider or category conversion model in this checkout.
- [ ] Historical global names in row 074 differ from current namespace publication. This slice does not establish exhaustive historical/native API parity.

## Out of scope

Native error wording, server leadership permissions and reintroducing removed globals. No new permission flag, shim, or vendor patch.
