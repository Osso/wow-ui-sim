# Forever outbound addon messages

Forever exposes three state-backed namespace senders in `src/c_api/addon_messages.rs`. Successful calls append outbound intent to the existing environment-local `SimState.message_log`; they do not deliver messages. See [Lua API](../wiki/systems/lua-api.md#forever-outbound-addon-messages).

## What it must do

### Documented surface and security

Pinned `wowforever/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:516/535` declares required `cstring` prefix/message, optional chatType (default PARTY) and target, and `SecretArguments = NotAllowed`. Normal result is non-nil; logged result is nilable. `BattleNetDocumentation.lua:279` declares numeric gameAccountID, required `stringView` prefix/data, non-nil enum result and `AllowedWhenUntainted`. These are source contracts, not native-client probes.

- [ ] Publish `C_ChatInfo.SendAddonMessage`, `SendAddonMessageLogged`, and `C_BattleNet.SendGameData` only under `client-wowforever`; leave existing enum publication unchanged.
- [ ] Reject missing/wrong required types and wrong non-nil optional types without appending records; do not coerce numbers into strings or string IDs into numbers.
- [ ] Reject secrets in all four chat arguments even for secure callers. BattleNet accepts wrapped numeric/string arguments only through existing VM-authenticated untainted access; ordinary tainted calls remain allowed and caller taint must not be cleared.

### Inferred local acceptance policy

Every limit, validation/result mapping, channel rule and local acceptance rule below is simulator policy/inference, not native-verified behavior. Local AceComm evidence grounds the 16-byte prefix and 255-byte chat bounds. Actual BugSack ChatThrottleLib's BattleNet wrapper comment names 4078 as the native limit while choosing 255 for fairness; the simulator uses 4078, not that wrapper's fairness cap.

- [ ] Require 1–16 UTF-8 bytes in prefix (`InvalidPrefix=1` otherwise); allow empty payload, cap chat payload at 255 bytes and BattleNet at 4078 (`InvalidMessage=2` on overflow). No NUL-specific validation is claimed.
- [ ] Default omitted/nil chatType to PARTY. PARTY requires `party_group_active`; RAID additionally requires at least six roster members, matching the existing raid model (`NotInGroup=5`). GUILD requires `world.guild_name` (`NotInGuild=10`). WHISPER requires nonempty target (`TargetRequired=6`), without inventing recipient availability. CHANNEL and INSTANCE_CHAT have no local routing model (`GeneralError=9`); other channels return `InvalidChatType=4`. No channel normalization is inferred.
- [ ] BattleNet requires a finite positive integral ID representable by the existing i32 game-account field (`GeneralError=9` otherwise). Resolve only nested friend game-account IDs whose `is_online` is true; absent/offline targets, including friend-account IDs, return `TargetOffline=12`.
- [ ] On acceptance return `Success=0` and append one ordered record: chat kind `addon` or `addon_logged`, prefix/message/channel/target; BattleNet kind `bnet_game_data`, prefix/data, empty channel, decimal game-account target. Distinct logged kind is not native logging/throttling. Both chat methods return a result on every modeled content/routing outcome; logged nil outcomes remain unmodeled.
- [ ] Rejection leaves existing records and pending events unchanged. Acceptance emits no inbound events. State, recipient availability and logs remain environment-local.
- [ ] Preserve legacy global `SendAddonMessage`: permissive argument extraction, no return values, existing record and local four-argument `CHAT_MSG_ADDON`. Do not alias it unconditionally to namespace status-returning behavior.

## How it works

- [Lua API](../wiki/systems/lua-api.md#forever-outbound-addon-messages) — registration and backing state.

## Implementation inventory

- `src/c_api/addon_messages.rs` — strict parsing/security, acceptance and records.
- `src/c_api/mod.rs`, `src/c_api/registration.rs` — Forever-only module/chat publication.
- `src/c_api/c_battle_net.rs` — Forever-only BattleNet sender wiring.
- `src/lua_api/globals/message_verbs.rs` — unchanged legacy sender.

## Tests asserting this spec

- `tests/addon_messages.rs` — 15 cases: enums, accepted records, rejections, byte bounds, group/guild state, typed errors, secret/taint preservation, BattleNet online transitions, isolation and legacy contract.
- `tests/message_verbs.rs` — seven unchanged legacy controls.
- Tests-only `79bedc49e`: retained RED artifacts in `/home/osso/.local/state/wow-ui-sim-proof/addon-messages-2026-10-01/` show 22 cases, nine passes and thirteen assertion failures of unmodeled behavior, not missing methods. The minimal environment already exposed callable namespace senders: `require_senders` passed its function-type checks before the behavior assertions failed (`red.stderr:165–166` and other failure traces). This does not invalidate the earlier actual BugSack workflow's literal `MissingRequirements` metadata failure; that is separate workflow evidence. Implementation GREEN is parent-owned and pending; requirements remain unchecked, with no addon-inventory credit.

## Known gaps (current cycle)

- [ ] Parent-owned targeted GREEN and integration verification after Cargo release.
- [ ] Rust `MessageLogEntry` string fields cannot represent arbitrary non-UTF-8 `stringView` payload bytes. Current boundary rejects them explicitly rather than coercing or claiming binary support; fixtures cover UTF-8/ASCII only.

## Out of scope

Network delivery/queues, incoming events, native throttling/lockdown/channel rate limits, logged nil outcome modeling, unmodeled routing, native-client parity and vendor changes. These have no backing system/evidence in this bounded goal.
