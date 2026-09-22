# Guild invite preference

Forever exposes `SetAutoDeclineGuildInvites` in `src/lua_api/globals/real/guild_invites.rs`, sharing the existing getter's state. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [x] Setter stores true/false in existing `SimState.auto_decline_guild_invites` and returns no values; the existing getter immediately observes the change.
- [x] Omitted/nil setter arguments default to false. Other nonboolean public values fail without changing state.
- [x] Existing VM `AllowedWhenUntainted` checks run before argument default/type conversion. Untainted secret booleans are accepted; tainted secret arguments fail, while public arguments remain usable by tainted callers.
- [x] State remains independent of neighborhood invites, location visibility and other environments; setter/getter remain usable after bootstrap restoration.
- [x] AccountUI-shaped save/change/load and `hooksecurefunc` observe updated guild preference state.

Cached Forever `PlayerScriptDocumentation.lua` documents `SetAutoDeclineGuildInvites(allow)` with `Type=bool`, `Nilable=false`, `Default=false`, and `SecretArguments=AllowedWhenUntainted`. Nil follows the existing simulator optional-argument convention. The existing stored initial false is preserved, not newly asserted as native behavior. No undocumented event is introduced. Getter registration and non-Forever behavior are unchanged.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/guild_invites.rs`: setter, defaulted boolean conversion and VM security validation.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only setter registration.
- Existing `src/lua_api/globals/guild_probes.rs`: unchanged getter for the shared field.
- Existing `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: reused field and unchanged initial state.

## Tests asserting this spec

- `tests/guild_invite_preference.rs`: grouped integration tests for toggles, defaults, shared-state/bootstrap/isolation, security and hooked save/load.

## Known gaps (current cycle)

Independent verification reuses the 5/5 grouped proof, passes formatting, default offline checking, readability and security, and authenticates frozen `6632d6373`: `/tmp/forever-addon-audit/verify-guild-preference-ledger.json`. The exact unchanged AccountUI archive completes its self-cast save/change/load round trip with `DONE`, empty Lua errors and unchanged host CVars: `/tmp/forever-addon-runtime/account-guild-workflow-hgl3rw6d/ledger.json`.
- [x] Parent-owned independent verification and bounded AccountUI replay.
- [ ] All-settings fidelity, bag preferences, cross-character/restart persistence, native behavior and rendered UI remain unproven.

## Out of scope

Invitation delivery, guild membership systems, all-settings or bag-preference fidelity, cross-process persistence, native coercion/initial-state proof, other-profile setter publication, rendered UI and undocumented events.
