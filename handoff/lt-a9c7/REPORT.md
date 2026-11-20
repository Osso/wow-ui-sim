# lt-a9c7: frame and chat defaults audit

## Scope and disposition

C_ChatInfo remains audit-only and unchanged, per coordinator comment #1913.
This report distinguishes native API compatibility defaults from duplicate
Blizzard Lua providers. Generated API documentation describes native functions;
it does not implement them. No original Lua definitions of the 14 audited
C_ChatInfo members were found in the selected Retail 70783 or Forever caches.

## C_ChatInfo classification

Source: `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs`.
`C_ChatInfo = C_ChatInfo or __wow_namespace()` conditionally creates the namespace.
The 13 `installChatInfoDefault` calls install real member definitions only when
`rawget` finds no member. They are not reference-only scan hits. Each is an
absent-native-model compatibility implementation, not a duplicate original Lua
provider. Their behavior remains **INFERRED**; generated signatures are verified
documentation, not proof of native state or invalid-input behavior. Correct return types alone do not establish native behavioral parity.

Documentation references below are to
`Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua`; cited lines and
signatures agree in both selected caches.

| Member | Documentation line | Current behavior and classification |
| --- | ---: | --- |
| `PerformEmote` | 451 | Returns false; no emote state or restriction model. Ignores documented optional targetName and suppressMoveError. Native compatibility default; retain unchanged. |
| `CancelEmote` | 36 | No-op; no active emote model. Documented no return. Native compatibility default; retain unchanged. |
| `IsValidChatLine` | 421 | Returns false for every input; no line validity model. Documentation accepts nullable numeric chatLine and returns boolean. Native compatibility default; retain unchanged. |
| `UncensorChatLine` | 590 | No-op; no censorship state transition. Documented numeric chatLine and no return. Native compatibility default; retain unchanged. |
| `DropCautionaryChatMessage` | 40 | No-op; no pending-message model. Documented argument is confirmNumber. Native compatibility default; retain unchanged. |
| `SendCautionaryChatMessage` | 554 | No-op; no pending-message/send transition. Documented argument is confirmNumber. Native compatibility default; retain unchanged. |
| `AreOutgoingAddonChatMessagesRestricted` | 11 | Returns false; no realm-policy model. This is not the Rust InChatMessagingLockdown query. Native compatibility default; retain unchanged. |
| `GetNumReservedChatWindows` | 275 | Returns zero; no reserved-window model. Native compatibility default; retain unchanged. |
| `GetChannelRulesetForChannelID` | 101 | Returns zero/None regardless of channel ID; no per-channel ruleset model. Native compatibility default; retain unchanged. |
| `GetChannelRuleset` | 86 | Returns zero/None; shim labels input channelName but documentation says channelIndex, luaIndex. Native compatibility default; retain unchanged pending native question below. |
| `GetChatLineText` | 178 | Shim takes two named parameters and returns nil; documentation says one numeric chatLine and nonnil string. Native compatibility default; retain unchanged pending native question below. |
| `IsTimerunningPlayer` | 406 | Returns false regardless of GUID; no player classification model. Native compatibility default; retain unchanged. |
| `GetChannelShortcutForChannelID` | 131 | Returns empty string; no channel shortcut lookup. Native compatibility default; retain unchanged. |
| `ReplaceIconAndGroupExpressions` | 485 | Retail already uses the Rust provider and excludes this Lua fallback. Forever receives identity fallback because it lacks retail-12-0-5; no expansion/flag validation there. Not a concurrently installed duplicate; retain unchanged. |

`Enum.ChatChannelRuleset.None = 0` is documented in
`Blizzard_APIDocumentationGenerated/ChatConstantsDocumentation.lua:28` in Retail.

### Provider timing and consumers

The defaults run in temporary bootstrap (`workarounds/mod.rs:342`), following
initial global registration and before addon loading (`env_init/mod.rs:73–78`).
Channel/message Rust providers supply other members, not these 13 defaults.
The expression implementation is registered by `c_api/c_chat_info.rs:9–17`,
gated by `retail-12-0-5` in `c_api/mod.rs:631–632`.
`c_api/chat_expressions.rs:40–49` provides icon/group transformations with
explicit host inputs, flag validation and secret-aware handling. The fallback
has the complementary feature guard, so removing it would remove Forever
coverage rather than expose an already registered implementation.

Representative original consumers in the Retail cache:

- `Blizzard_ChatFrameBase/Mainline/ChatFrameMenuButton.lua:122,138` and
  `Shared/ChatFrameEditBox.lua:267`: emote calls, including target argument.
- `Blizzard_ChatFrameBase/Mainline/ChatFrameUtilOverrides.lua:121,144,155`:
  shortcuts and rulesets.
- `Blizzard_ChatFrameBase/Shared/FloatingChatFrame.lua:2,27`: reserved windows.
- `Blizzard_ChatFrameBase/Shared/ChatFrameUtil.lua:867,1096`: line text and
  Timerunning player classification.
- `Blizzard_UIPanels_Game/Shared/ItemRefHandlersShared.lua:189,232,257`:
  uncensor and pending cautionary-message operations.
- `Blizzard_UnitPopup/Mainline/UnitPopupUtils.lua:52`: line validity.
- `Blizzard_Game/Mainline/EventImplementation.lua:911`: outgoing restriction.
- `Blizzard_WorldMap/Blizzard_WorldMap.lua:381` and
  `Blizzard_MailFrame/MailFrame.lua:71`: cancel emote.
- `Blizzard_ChatFrameBase/Mainline/ChatFrameOverrides.lua:569` and
  `Blizzard_ChatFrame/Shared/TextToSpeechFrame.lua:1003`: expression expansion.

### Native/decomp questions for coordinator handoff

1. **GetChatLineText:** In both selected generated API documents, line 178
   declares `GetChatLineText(chatLine: number) -> text: string`, with text
   nonnil, `SecretArguments = AllowedWhenUntainted`, and
   `SecretInChatMessagingLockdown = true`. The current shim at
   `c_chat_info_defaults.rs:52–54` declares `(_chatFrame, _chatLine)` and always
   returns nil; its unit fixture calls `(1, 1)`. What does the native function
   return or raise for an unknown, expired, or absent chat-line ID, and how are
   extra arguments handled? Please establish valid-ID and invalid-ID behavior,
   including returned string secrecy in chat messaging lockdown, before any
   replacement model or signature correction is implemented. Current fallback
   behavior is not evidence for those native cases.
2. **GetChannelRuleset:** In both documents, line 86 declares
   `GetChannelRuleset(channelIndex: luaIndex) -> ruleset: ChatChannelRuleset`,
   nonnil. The current shim at `c_chat_info_defaults.rs:48–50` names its argument
   `_channelName`, ignores it, and returns 0; its unit fixture passes
   `"General"`. Does native strictly require a numeric local channel index,
   or accept/coerce channel names, numeric strings, nil or invalid indices?
   What ruleset or error results for an unjoined/unknown index, and how is this
   distinguished from `GetChannelRulesetForChannelID` (documented at line 101)?
   Please establish these cases before changing validation or lookup behavior.

These are questions, not inferred native contracts or authorization to change
behavior. Coordinator/main owns the decomp follow-up. Both discrepancies remain
unchanged in this task.

## Frame-provider retirement

Coordinator #1913 approved skipping `misc_global_frame_defaults` bootstrap on
Retail and Forever only. Other profiles retain the workaround as **INFERRED**
compatibility until their provider ordering is separately proven.

| Name/member | Classification and selected provider |
| --- | --- |
| `GameMenuFrame` | Guarded global frame definition. Selected `Blizzard_GameMenu/Shared/GameMenuFrame.xml` owns the real frame. |
| `GameMenuFrame.buttonPool` and two initial buttons | Conditional field/object definitions. Original `MainMenuFrameMixin:OnLoad` creates its pool; `GameMenuFrameMixin:OnShow` populates real buttons. |
| `ObjectiveTrackerFrame` | Reference here; its separate built-in frame definition is unchanged. Selected `Blizzard_ObjectiveTracker.xml` applies original templates/mixins. |
| `ObjectiveTrackerFrame.OnAdded` | Conditional member definition. Original `ObjectiveTrackerContainerMixin:OnAdded` initializes the header and background. |
| `ObjectiveTrackerFrame.Header` | Conditional child-field definition. Original tracker XML supplies the real container header. |
| `ObjectiveTrackerFrame.Header.MinimizeButton` | Conditional child-field definition. Original container header template supplies and wires the collapse button. |
| `ObjectiveTrackerContainerMixin`, `UIParent`, `CreateFrame`, `CreateFramePool`, `GAMEMENU_OPTIONS`, `LOGOUT` | References, not global definitions in this workaround. |

GameMenu shared Lua/XML is selected by the Mainline TOC in Retail and the
bare game-filtered TOC in Forever; the Plunderstorm alternative is not used.
The real menu template initializes the pool before menu input is dispatched.
ObjectiveTracker loads its container Lua/XML before its main frame XML;
`ObjectiveTrackerManager:Init` is deferred until both `PLAYER_ENTERING_WORLD`
and `VARIABLES_LOADED` (`Blizzard_ObjectiveTrackerManager.lua`, final line).
Thus its `AddContainer` consumer runs after the original `OnAdded` is installed.
No unguarded before-provider consumer was identified in these two profiles.

The two old bare-environment shim tests are profile-gated together with their
test module; their bodies/assertions are unchanged. No absence or source-location
assertions were added. C_ChatInfo code and tests are unchanged.

## Verification receipts

PRESERVATION, not native RED. Base `587ec18ed`; test commit `c84070bf5`.

Selected owner-provided Retail 12.1.5.70783 and Forever caches; no Blizzard CDN.
Rust 1.99.1, 9 build jobs, `CARGO_INCREMENTAL=0`, dev/test debug info disabled.

Build gates (all exit 0, zero warnings), in order Forever, Mists, Wrath, Retail:
```sh
cargo test --locked --no-default-features --features sound,gui,casc,client-PROFILE,optional-tests --lib --test integration --no-run --message-format=json
cargo test --locked --features optional-tests --lib --test integration --no-run --message-format=json
```
`PROFILE` was `wowforever`, `mists`, then `wrath`; final command was Retail.

Exact selectors in `misc_frame_provider_preservation`, separately invoked with
`--exact --test-threads=1 --nocapture`, bounded to 90 seconds each:

| Profile | Test | Before | After |
| --- | --- | --- | --- |
| retail | `original_game_menu_populates_buttons_and_handles_escape` | PASS 13.19s | PASS 13.29s |
| retail | `original_tracker_manager_initializes_header_and_collapse_button` | PASS 9.29s | PASS 10.27s |
| forever | `original_game_menu_populates_buttons_and_handles_escape` | PASS 15.75s | PASS 15.35s |
| forever | `original_tracker_manager_initializes_header_and_collapse_button` | PASS 11.25s | PASS 10.66s |

Final stock Retail `wow-sim --no-addons --no-saved-vars lua-errors`: exit 0, `[]`.
Writable `XDG_DATA_HOME` avoids the VM read-only home directory.
Runtime receipts record executable/dylib SHA256 before and after each run; all
bindings match. Compiler JSON artifacts, transcripts and source hashes are
attached to the tracker delivery, not committed. No full suites were run.
C_ChatInfo code/tests and legacy shim assertion bodies are byte-identical to base.
