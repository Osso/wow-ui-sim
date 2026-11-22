# Social chat-disabled scenario state

The simulator supplies the existing `C_SocialRestrictions.IsChatDisabled()` query from environment-local `social_restrictions.chat_disabled`. Its Retail and WowForever registrations are already modeled. The host setter and Communities UI consumer test are INFERRED under R88ek (native evidence pending); the initial `false` is not a native account-policy claim.

## What it must do

- [x] Register `C_SocialRestrictions.IsChatDisabled()` on Retail and WowForever and return the supplied environment-local boolean.
- [x] Let `A_Admin.SetChatDisabled(bool)` supply the local scenario without changing the public C API surface.
- [x] The original Social settings proxy reflects both supplied values.

## How it works

- [Social squelch state](social-squelch.md) documents the adjacent, distinct `IsSquelched` input.
- [Chat messaging lockdown](chat-messaging-lockdown.md) documents the separate `C_ChatInfo.InChatMessagingLockdown` predicate.

## Implementation inventory

- `src/c_api/c_social_restrictions.rs`: existing getter and host input setter.
- `src/lua_api/state_types/social.rs`: environment-local chat-disabled fixture field.
- `src/lua_api/globals/admin.rs`: host setter registration.
- `tests/social_restriction_consumers.rs`: real Communities frame consumer.

## Tests asserting this spec

`tests/social_restriction_consumers.rs`: original Settings and Communities consumers; `original_social_settings_reads_chat_disabled_admin_input` supplies false and true through `A_Admin.SetChatDisabled`.

## Known gaps (current cycle)

- [ ] Native account-policy producer and default remain unknown.
- [ ] The original Communities chat-edit-box path is not proven in the closure test: `CommunitiesFrame.lua:1523` fails during frame setup because `inviteButton` is nil.

## Out of scope

`C_SocialRestrictions.IsChatDisabled` is not re-registered or changed here. Native account policy, the native setter, chat-messaging lockdown, and enforcement outside the selected Communities consumer are not modeled.
