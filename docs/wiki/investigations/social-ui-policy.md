# Retail social-UI policy

The Retail 12.1.5 social-system availability producer consumes an explicit optional host policy input, instead of the inert constant-false fallback. Other profiles retain their existing fallback.

## Native contract and boundary

**VERIFIED (Retail 12.1.5.70688, `pr46-49-decomp-answers.md`, R88ap part B §6):** `C_SocialUI.IsSystemEnabled` returns exactly one Boolean and reads no system ID, so it performs no ID selection and no unknown-ID rejection. Without a subordinate value it returns false. **INFERRED:** that extra arguments are otherwise ignored; complete native argument validation is not established.

**UNKNOWN:** native state writers and event emission. `SimState.social_system_policy_enabled` is an explicit host input, not a derived friend-list/account state. **INFERRED (simulator mapping):** a single optional Boolean collapses the native no-owner default-plus-policy-bytes path and the absent-subordinate path; `None` represents the absent subordinate value and projects to false. No writer, refresh event, or server policy is invented.

## Original consumer

**VERIFIED (selected Retail 12.1.5.70783 source):** `Blizzard_FriendsFrame.toc` allows both screens and mainline, selecting `Mainline/FriendsFrame.lua` and XML. The local `GetValidSubFrames` first checks Plunderstorm, then this producer. True selects only `WhoFrame`; false selects `FRIENDSFRAME_SUBFRAMES`. The public `FriendsFrame_ShowSubFrame` consumes that selection.

The integration fixture loads original FriendsFrame and the QuickJoin, RaidFrame, and RecruitAFriend owners for the full selection. It checks original function provenance and excludes Plunderstorm. The consumer test exercises true/false/absent/true transitions, with a separate arity/type/ignored-argument test. A test-only explicit producer overlay supports a constant-false counterfactual without changing the original consumer or assertion body.

## Sources

- [Producer](../../../src/c_api/c_social_ui.rs)
- [Regression tests](../../../tests/social_ui_policy.rs)
- `pr46-49-decomp-answers.md`, R88ap part B §6 (untracked native handoff, summarized in tracker lt-5d7e)

## See also

- [[consumer-selection-and-red-proof]]
