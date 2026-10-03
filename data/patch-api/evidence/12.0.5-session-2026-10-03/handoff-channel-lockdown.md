# Channel lockdown authoring handoff

Date: 2026-10-03. Row: `prose-2026-04-10-196`.

## Active goal

Author only a scratch-staged, exact-inventory channel lockdown/macro enforcement slice using existing explicit state and real mutations. Completion requires identifiable affected APIs, behavioral fixtures, exact unique-anchor edits, and a state/producer RED split. Excludes repository writes, cargo/test execution, git mutation, agents, model CLIs, vendor changes, wrapped constants, fallbacks and unsupported scope claims. Verification here is read-only source inspection; runtime proof belongs to the integrating caller.

## Disposition: blocked, not implemented

The source sentence does not enumerate APIs: “Various chat channel APIs are no longer usable while chat lockdown is in effect, or from within macros.” The required cached declarations do not identify which existing legacy channel mutators this applies to. Do not equate all channel mutations with “various,” or grant this row bounded coverage from a guessed inventory.

No enforcement code or tests staged. Existing files require no edits. 

## Read-only evidence

- Read `scout-prose-0331.md`, section `prose-2026-04-10-196`; exact retained source line196; accepted `chat-lockdown-predicate` record in `12.0.5-page-coverage.json`; `docs/specs/chat-messaging-lockdown.md`; all `tests/chat_messaging_lockdown.rs`; `channel_verbs.rs`; macro producers and their existing tests; cached generated declarations.
- Accepted prerequisite explicitly excludes “native producer/security/enforcement/macros/all-profile/full-page” coverage. Existing input is per-environment `SimState.chat_messaging_lockdown`, independent of combat. `src/c_api/c_chat_info.rs:20–30` now supplies a reusable state-backed rejection helper; that is not proof of channel enforcement.
- Scanned every `*.lua` in the specified generated-documentation directory for exact `Name` declarations of all nine existing legacy channel mutators below. None was found. This proves absence in that directory, not absence in native WoW or permission to treat them as unrestricted.

## Exact inventory and coverage

**Covered APIs for row196: none.** No row acceptance or implementation credit is justified.

Legacy mutation candidates actually modeled in `channel_verbs.rs` (affected status UNKNOWN):

| API | Existing real state transition |
|---|---|
| `JoinChannelByName` | Append channel if absent |
| `JoinTemporaryChannel` | Calls join implementation |
| `ChannelLeave` | Remove named channel |
| `ChannelBan` | Add ban; remove membership and moderation |
| `ChannelInvite` | Add unbanned member |
| `ChannelKick` | Remove membership and moderation |
| `ChannelModerator` | Grant moderation to member |
| `ChannelUnmoderator` | Remove moderation |
| `SwapChatChannelLinks` | Swap valid channel positions |

Generated `ChatInfoDocumentation.lua` explicitly declares these channel-related `C_ChatInfo` functions (name-line numbers); none carries `HasRestrictions`, `RestrictedForMacro*`, or `SecretInChatMessagingLockdown` in its function block:

`GetChannelInfoFromIdentifier:50`, `GetChannelRosterInfo:66`, `GetChannelRuleset:86`, `GetChannelRulesetForChannelID:101`, `GetChannelShortcut:116`, `GetChannelShortcutForChannelID:131`, `GetGeneralChannelID:239`, `GetGeneralChannelLocalID:248`, `GetMentorChannelID:257`, `GetNumActiveChannels:266`, `IsChannelRegional:318`, `IsChannelRegionalForChannelID:333`, `IsPartyChannelType:382`, `ResetDefaultZoneChannels:512`, `SwapChatChannelsByChannelIndex:579`.

Missing restriction metadata does not establish allowed behavior. `SwapChatChannelsByChannelIndex` is not the simulator global `SwapChatChannelLinks`; do not silently equate them.

`C_ChatInfo.SendChatMessage:564` has `HasRestrictions = true` and `RestrictedForMacroChatMessages = true`. It is a messaging function, not evidence that any particular channel membership/moderation function belongs to row196. `SecretArguments = "AllowedWhenUntainted"` is an argument-security contract, not a lockdown/macro execution ban. Voice-chat getters with `SecretInChatMessagingLockdown` and Discord functions with `HasRestrictions` likewise cannot supply the unspecified membership/moderation inventory.

## Macro provenance blocker

`spell_macro_verbs.rs:131–146`: `RunMacro` resolves a slot and stores `running_macro = Some(slot)` without executing its body. `StopMacro` clears it. `tests/spell_macro_verbs.rs` asserts precisely that persistent slot behavior.

`C_Macro.RunMacroText` (`spell_macro_verbs.rs:149–199`) executes supported `/target`, `/focus`, `/cast` aliases and `/equipset`; it does not enter/restore a dynamic macro context and does not execute `/run` or `/script`. `running_macro` therefore does not honestly mean “this channel call is currently executing within a macro.” Testing `RunMacro(); JoinChannelByName()` would assert persistence, not production macro ordering. A Lua macro fixture cannot currently reach these channel producers through supported macro commands.

**INFERRED policies not adopted:** all nine mutators are affected; `running_macro.is_some()` indicates dynamic macro origin; rejection throws an error rather than returning/no-op; secure callers bypass restrictions; chat lockdown equals combat. None is established by this sentence/declarations. Error wording and native permission details remain unknown.

## State versus producer split / RED

Existing explicit lockdown state needs no new field or setter. Its query and rejection helper can be reused once API membership is established. Dynamic macro provenance would require a scoped producer with entry/restoration, including error exits, not a reinterpretation of persistent macro slot state. Such a change alone would not identify the affected APIs.

No state edits, producer edits, unique-anchor replacements, staged integration binary, or expected RED failures are supplied: deliberately stopped before inventing the contract. No RED/GREEN or compilation claim. The caller should not run a nonexistent filter or accept guessed tests. Once the API set and macro execution boundary are established, stage one epoch-gated integration file, apply state/tests first with enforcement producers withheld, demonstrate actual mutation-on-denial RED, then apply producers and verify allowed/blocked/recovery behavior.

## Existing expectation changes and proof ledger

**Every existing test whose expectation changes: none.** No implementation proposed/applied. Read controls: all `tests/channel_verbs.rs`, both `tests/c_chat_info_probes.rs`, five prerequisite predicate tests, and RunMacro/StopMacro slot tests. Preserve these existing contracts; changing macro execution would require separately identified expectation changes.

Proof: read-only file/declaration inspection only; no cargo, test invocation, git mutation, repository write, agent, or model CLI. The sole authored deliverable is this scratch handoff. No files exist for this task under `staging/channel-lockdown/`; no producer patch should be applied.

## Required missing evidence

An authoritative affected-API list (or native per-API probes) and a genuine executable macro route to the listed channel calls. Native rejection shape may be labeled INFERRED if simulator policy is explicitly accepted; affected inventory and dynamic caller provenance must not be fabricated. Row196 remains unmodeled.

