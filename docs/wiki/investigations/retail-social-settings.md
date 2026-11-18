# Retail social settings queries

Retail 12.1.5 role selection and friends availability consume explicit settings state instead of constant-false fallbacks. Other profiles retain their previous registration.

## Contract and model boundaries

**VERIFIED (Retail 12.1.5.70688, `pr46-49-decomp-answers.md`, R88ap part A §3):** `CanShowSetRoleButton` returns one Boolean. An absent rule 93, a non-integer rule, or an integer with zero low byte returns false; a nonzero low byte returns true.

**VERIFIED (same handoff, §4):** `C_SocialRestrictions.IsFriendsDisabled` returns one Boolean from a settings byte initialized to zero; early calls return false without a nil or initialization error.

**INFERRED (simulator representation):** existing `game_rules.rules_by_id` stores integer values only. Retail `non_integer_rule_ids` supplies type tags overriding integer storage, allowing the verified non-integer case without interpreting a float as an integer. `social_restrictions.friends_disabled_byte` is an explicit host-supplied mapping of the native settings byte, defaulting to zero. These host mappings are isolated in a separate commit.

**UNKNOWN:** settings-packet ingestion, server conditions, and refresh events. None are invented. The host supplies snapshots; these queries do not derive account policy from friend lists or group membership.

## Original consumers

**VERIFIED (Retail 12.1.5.70783 source):** the selected `Blizzard_UnitPopupShared_Mainline.toc` loads `UnitPopupSharedButtonMixins.lua`; `UnitPopupSelectRoleButtonMixin:CanShow` checks the role query before scenario/group/LFG/AI conditions. The test holds those other conditions valid and asserts the original entry's visibility predicate for absent, zero-low-byte, nonzero-low-byte, and non-integer rules.

**VERIFIED (selected source):** `Blizzard_FriendsFrame.toc` permits mainline and loads `Mainline/FriendsFrame.lua` and XML. `FriendsListFrame_OnShow` switches the real ScrollBox, ScrollBar, disabled label and Add Friend button according to the friends-disabled query. The test supplies byte 0/1/0 and observes those original widgets.

Both selectors check original function source provenance after dependency-closure loading. Independent callable-wrong constant-false overlays exercise the same test bodies, without modifying vendor code or consumer methods. Compiler/runtime receipts remain outside the repository.

## Sources

- [Role query](../../../src/lua_api/globals/real/set_role_button.rs)
- [Friends restriction query](../../../src/c_api/c_social_restrictions.rs)
- [Consumer tests](../../../tests/social_settings.rs)
- `pr46-49-decomp-answers.md`, R88ap part A §3–4 (untracked native handoff, summarized in tracker lt-6234)

## See also

- [[social-ui-policy]]
- [[consumer-selection-and-red-proof]]
