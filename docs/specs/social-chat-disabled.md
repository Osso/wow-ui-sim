# Social chat-disabled scenario state

The simulator supplies the existing `C_SocialRestrictions.IsChatDisabled()` query from environment-local `social_restrictions.chat_disabled`. Its Retail and WowForever registrations are already modeled. The host setter and its original Social settings consumer test are INFERRED under R88ek (native evidence pending); the initial `false` is not a native account-policy claim.

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
- `tests/social_restriction_consumers.rs`: original Social settings consumer.

## Tests asserting this spec

`tests/social_restriction_consumers.rs`: the original Social settings consumer; `original_social_settings_reads_chat_disabled_admin_input` supplies false and true through `A_Admin.SetChatDisabled`.

## Known gaps (current cycle)

- [ ] Native account-policy producer and default remain unknown.
- [ ] The Communities chat-edit-box consumer remains unproven. A closure probe failed at cached Forever `CommunitiesFrame.lua:1523` because `inviteButton` was nil.

## Out of scope

`C_SocialRestrictions.IsChatDisabled` is not re-registered or changed here. Native account policy, the native setter, chat-messaging lockdown, and chat-edit-box enforcement are not modeled.
