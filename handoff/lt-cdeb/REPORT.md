# lt-cdeb: inert defaults, social/native audit

## Status and evidence boundaries

Coordinator comment #1939 narrows implementation to IsPlayerInWorld only,
superseding the chat portion of #1934. The approved explicitly INFERRED mapping
reads existing in_world state: false during preload and true before
PLAYER_ENTERING_WORLD dispatch. IsChatDisabled is audit-only and unchanged:
the original consumer paths are blocked by missing preceding APIs, detailed
below. No chat-disabled byte implementation is approved in this narrowed scope.
Implementation, original-consumer verification, and all four ordered compile gates are complete.

Audited source: `src/lua_api/workarounds/temporary/inert_global_defaults.rs`.
Native evidence: `pr46-49-decomp-answers.md`, R88l.9–14, R88l.46, R88ap part A.
Native observations concern Retail 12.1.5.70688. Selected generated docs concern
Retail UI 70783 and Forever. Matching docs do not prove identical native
implementations across profiles. R88ap resolves several earlier R88l UNKNOWNs;
those later answers take precedence for the particular questions they resolve.

## Publication audit

The namespace expressions conditionally create tables; method bodies are guarded
fallback definitions, not reference-only hits. Global registration precedes
inert temporary bootstrap (`globals/register.rs:271`, `workarounds/mod.rs:411`).

| Surface | Retail | Forever | Disposition |
| --- | --- | --- | --- |
| C_SocialRestrictions.IsChatDisabled | false Lua fallback | false Lua fallback | Native byte read observed; public result and writers unresolved in R88l.9. No claim of VERIFIED Boolean mapping. |
| C_SocialRestrictions.IsFriendsDisabled | Existing Rust byte model | Not installed by this file's build gate | Retail fallback already compile-excluded. R88ap.4 proves one Boolean from settings byte, initially zero. |
| C_Commentator.IsSpectating | false Lua fallback | false Lua fallback | No commentator state model established here; separate C_SpectatingUI model is not equivalent evidence. |
| C_Commentator.SendAddonMessage | success-only Lua fallback | success-only Lua fallback | Native namespace binding unresolved; no send effect, target handling or tournament-policy model. |
| C_Commentator.SendAddonMessageLogged | Lua delegate to nonlogged fallback | Not installed by this file's build gate | R88l.46 does not establish this exact native registration or public arity. |
| C_FriendList.GetNumFriends | Existing Rust provider | Existing Rust provider | Lua guard inactive; uses social_friends length. |
| C_FriendList.GetNumOnlineFriends | Existing Rust provider | Existing Rust provider | Lua guard inactive; filters social_friends.is_online. |
| C_FriendList.GetNumIgnores | Existing Rust provider | Existing Rust provider | Lua guard inactive; Rust still returns constant zero. |
| C_FriendList.GetIgnoreName | Existing Rust provider | Existing Rust provider | Lua guard inactive; Rust parses integer index then returns nil. |
| C_GuildBank | Namespace ensure only | Namespace ensure only | No member implementation in this file. |
| BNGetNumFriends | Four zero Lua results | Four zero Lua results | BNet state exists, but native global tuple/filter contract not established by this audit. |
| BNGetNumFriendInvites | Zero Lua result | Zero Lua result | Pending invite state exists; native legacy-global contract remains unverified. |
| BNGetFriendInfo | Nil Lua result | Nil Lua result | No matching Rust global provider or generated declaration found. |
| CanShowSetRoleButton | Existing Rust rule-93 model | false Lua fallback | Retail fallback already compile-excluded; R88ap.3 proves integer-rule/low-byte predicate. |
| GetCurrentCinematicSummary | Existing Rust ID/summary model | Not installed by this file's build gate | Retail fallback already compile-excluded; R88ap.1 proves nil/no-text versus populated string. |
| C_SocialUI.IsSystemEnabled | Existing Rust CVar/policy model | Existing Rust CVar/policy model | Lua fallback already excluded on both profiles. |

Forever's interface version is 16001 (`client_profile.rs:145`). Consequently,
`>=120005` and `>=120100` runtime build tests do not install the listed members
on Forever, even though its selected generated docs declare them. This is a
publication observation, not authorization to add APIs or transplant Retail
behavior.

## Existing state and native evidence

- `globals/missing_surface/friend_list.rs:49–59` registers all four audited friend
  methods unconditionally. Counts use `social_friends` (:83–96); ignore methods
  remain inert (:99–109). Removing inactive Lua guards would not add native
  behavior. Any retirement requires provider-preservation proof.
- `state/sim_state.rs:1072–1077` contains `bnet_friends` and
  `bnet_friend_invites`. Friend entries include favorite, friend status,
  appear-offline and game-account data (`state_types/social.rs:26–69`). Existing
  C_BattleNet queries consume those records. Available fields do not prove the
  legacy BN globals' count filtering, tuple order, invalid-index behavior or
  update events.
- `c_social_restrictions.rs:29–37` maps the Retail settings byte to
  `IsFriendsDisabled`; `SocialRestrictionsState` also has `is_squelched`, which
  is not evidence for chat-disabled state. R88l.9 verifies a separate shared
  byte read for IsChatDisabled but leaves public completion and writer/reset
  behavior UNKNOWN. R88ap.4 resolves IsFriendsDisabled, not IsChatDisabled.
- `real/set_role_button.rs:14–25` reads rule 93 using integer-type tags and its
  low byte. R88ap.3 proves one Boolean, false for absent/noninteger/zero-low-byte
  rules, true otherwise. Rule rows arrive from settings packets; consumer
  refresh event remains UNKNOWN. Group composition is not the proven predicate.
- `real/cinematic_summary.rs:16–47` uses current_cinematic_id and
  cinematic_summaries. R88ap.1 proves exactly one nil for zero ID, missing row,
  missing text or empty text, and exactly one string for nonempty text. Started
  and completed finish actions update the ID for types 0–3; exact Lua update
  events are UNKNOWN. This evidence supersedes R88l.12's unresolved return and
  the generated doc's nonnil-string claim.
- R88l.13 and R88ap.2 establish GetGuildFactionGroup separately: no object means
  zero results; otherwise one number from race/faction-table classification.
  It does not test guild membership. Existing Retail implementation is outside
  this proposed social-default replacement; do not infer that a missing guild
  should force a particular result.
- `c_social_ui.rs:33–42` uses socialUIEnabledClient and an explicitly inferred
  feature-policy constant; its native source is R88ba. This is already modeled
  on both audited profiles and needs no fallback replacement here.
- R88l.11 reports no exact C_Commentator.SendAddonMessage registration in its
  recovered inventory. The observed C_ChatInfo callback's positional parsing
  and send helper do not establish equivalence to Commentator. R88l.46 likewise
  finds only C_ChatInfo.SendAddonMessageLogged, not the requested Commentator
  member. Neither absence from that inventory nor a different namespace's
  implementation settles Commentator's public behavior.

## Generated declarations and original consumers

These declaration lines agree in both selected caches:

- SocialRestrictionsDocumentation.lua:60,69: chat/friends-disabled Booleans.
- FriendListDocumentation.lua:125,141,151,161: nullable ignore name, numeric
  friend/ignore/online counts, friend-list requirement.
- SocialUIDocumentation.lua:11: Boolean system-enabled query.
- CommentatorFrameDocumentation.lua:1036: Boolean spectating query;
  :1186 and :1205: send prefix/message/chatType/target, tournament/outgoing
  policy and secret handling. Logged result is nullable, nonlogged nonnullable.
- CinematicDocumentation.lua:34: nonnil string declaration, superseded for
  Retail absent-summary behavior by the more specific R88ap.1 result.
- UnitRoleDocumentation.lua:20: Boolean role-button query.
- No generated declarations found for BNGetNumFriends, BNGetNumFriendInvites,
  or BNGetFriendInfo. Consumer evidence is not full native-contract evidence.

Representative selected Retail consumers:

- FriendsFrame/Mainline/FriendsFrame.lua:801 consumes BN total, online,
  favorite and **favorite-online** counts; the existing inert unit fixture's
  fourth-variable label "mobile" is not the consumer's interpretation.
  Lines 840,914,1890 use invite counts; line 712 gates on friends-disabled.
- SharedXML/AccountUtil.lua:60 and QuickJoin/QuickJoinToast.lua:161 use the
  second BN count.
- Communities/CommunitiesFrame.lua:985 and
  SettingsDefinitions_Frame/Social.lua:54 use chat-disabled state.
- SocialUI/Mainline/SocialUI.lua:301 uses the system-enabled gate.
- CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua:301 uses
  commentator spectating for visibility; CooldownBroadcaster/MessageQueue.lua:21
  sends commentator messages.
- FrameXML/MovieFrame.lua:101 consumes cinematic summary;
  UnitPopupShared/UnitPopupSharedButtonMixins.lua:3471 uses role-button visibility.

## Bounded native follow-up questions

1. **World/chat state completion:** Can the unresolved normal return paths in
   R88l.3 and R88l.9 be connected to establish public count/type/nilability and
   the exact mapping of the read flags/byte? Which initialization/reset writers
   and existing UI refresh events are established? Until then, host-supplied
   Boolean/byte behavior must remain explicitly INFERRED, separate from
   friends-disabled and squelched state, with no invented emitted events.
2. **BNet legacy-global projection:** For BNGetNumFriends and
   BNGetNumFriendInvites, establish result tuple/count and how pending,
   offline/appear-offline, favorite and game-account records contribute.
   Establish whether BNGetFriendInfo is published in the selected builds and
   its absent/invalid-index result shape. Existing record vectors can support a
   model once the projection is known; UI usage alone must not define it.
3. **Commentator binding and transport:** Resolve the exact public binding for
   IsSpectating and the two SendAddonMessage variants, including whether the
   send names alias another implementation. Establish tournament/outgoing
   restriction, target/logged handling, result count/nilability and send-state
   effects before replacing unconditional success or forwarding to C_ChatInfo.
   R88l.11/.46 do not establish that equivalence.
4. **Forever scope where behavior is already known on Retail:** Confirm whether
   the selected Forever native surface actually publishes friends-disabled,
   cinematic-summary and logged-commentator methods despite the simulator's
   16001 interface gates. Obtain profile-specific evidence or an explicit
   shared-model decision before extending Retail behavior to that profile.

No further native questions are needed here for the already resolved Retail
role/friends-disabled/cinematic return predicates. Their remaining event and
writer gaps are explicitly outside a fabricated event implementation.

## Implementation

Coordinator #1939 limits implementation to IsPlayerInWorld. The new
`globals/real/world_presence.rs` provider reads existing `SimState.in_world` on
Retail/Forever. Their constant-true Lua workaround is compile-excluded. The
startup owner sets the state true immediately before PLAYER_ENTERING_WORLD;
initial state is false, and the UI-reconstruction owner explicitly resets it
to false on Retail/Forever. Other profiles retain
the original fallback and their existing startup timing.

This entire producer change is INFERRED, not a claimed native correction.
No Blizzard source, API mock, chat-restriction producer, event payload, or
unrelated fallback body is changed. The only old test expectation change is the
exact Retail/Forever bare-env true→false check approved in #1934. Other profiles
retain the true check.

## Verification

Test-first commit: `1b1647975f6703db50055aee0d5dfdb93c45a935`.
The exact selector
`world_social_consumers::original_spellbook_defers_world_events_until_world_entry`
rejects the old constant-true provider at the original OnLoad subscription
assertion on both Retail and Forever. Final GREEN additionally repeats the
original-consumer sequence after ReloadUI reconstruction. Bound receipts record
executable/dylib hashes before and after each run; no API producer is mocked.

The approved `installs_region_social_and_group_defaults` unit test passes on
both selected profiles with only the approved bare-env world expectation changed.
Retail `--no-addons --no-saved-vars lua-errors` is `[]`, exit 0. Forever's
stock JSON list is byte-for-byte identical before/after (existing
PrivateAurasTooltip missing-call diagnostics, exit 1 in both); no errors were
filtered or hidden. The original-consumer test likewise retains these unrelated
Forever diagnostics. It does not claim a clean Forever startup.

Final compile receipt stems: `cdeb-{forever,mists,wrath,retail}-final`, in that
order, all exit 0 with zero compiler warnings. Each command is `cargo test --locked --no-default-features --features
sound,gui,casc,client-PROFILE,optional-tests --lib --test integration --no-run
--message-format=json` (Forever uses `client-wowforever`). No Mists/Wrath runtime
or full-suite execution. Incremental is disabled, jobs match nproc (9), test/dev
debug info is disabled. Toolchain Rust/Cargo 1.99.1. All raw proof remains in the
tracker receipt attachment, not committed into the repository.


## Remaining global-function audit

This table completes coverage of functions in inert_global_defaults.rs. Unless
otherwise stated, the guarded Lua default remains active on both Retail and
Forever at the audited base; no corresponding earlier Rust global registration
was found. Available neighboring state is not proof of native equivalence.

| Function | Existing behavior / ownership | Disposition |
| --- | --- | --- |
| IsPlayerInWorld | Lua true at audited base | #1934 approves INFERRED existing in_world projection, with explicit startup timing and consumer proof. |
| GetCurrentRegionName | Lua US | R88l.4 proves numeric lookup and image seed 1, not full public completion/runtime default. Retain pending model scope. |
| GetDefaultLanguage | Lua Common, 1 | R88l.5 only partial player/table paths; do not treat Common/1 as native universal default. |
| GetAlternativeDefaultLanguage | Lua nil | No complete native alternative-language contract established here. |
| GetNumLanguages | Lua 1 | R88l.6 proves selected pushes/counting paths, not all public returns or filter semantics. |
| GetLanguageByIndex | Lua index 1 delegates to default language; otherwise nil | R88l.7 proves numeric-convertibility branch, not bounds/result contract. Current equality check is not native validation proof. |
| GetMaxBattlefieldID | Lua 0 | Unmodeled queue bound; no replacement authorized. |
| IsActiveBattlefieldArena | Lua false | Unmodeled active battlefield classification. |
| IsPVPTimerRunning | Lua false | Unmodeled timer state. |
| HasArtifactEquipped | Lua false | Neighboring inventory state does not establish this native predicate. |
| UnitExists | Existing Rust provider; Lua guard inactive | globals/group_queries.rs:69 registers the unit query before fallback. R88l.8 partial evidence must not replace existing behavior. |
| IsOnTournamentRealm | Lua false | No tournament policy supplied here; commentator success fallback is not proof of policy consistency. |
| GetNumDisplayChannels | Lua 0 | Display-channel projection not modeled by this fallback; channel lookup/join models are distinct. |
| GetChannelDisplayInfo | Lua nil | No display-channel tuple established here. |
| AreTalentsLocked | Lua false | R88l.50 proves exactly one Boolean on normal return, not a universal false predicate. |
| GetGuildFactionGroup | Retail Rust provider, inactive Lua guard; Forever Lua 1 | player_identity.rs:179–180 registers Retail model. R88ap.2 classifies race/faction rows, not guild membership. Preserve profile distinction. |
| WorldLootObjectExists | Lua false | c_api/world_loot_object.rs exposes namespaced IsWorldLootObject and related methods, not this exact global; do not silently alias. |
| GetActiveLootRollIDs | Existing Rust provider; Lua guard inactive | state_backed_queries.rs:30,184–198 returns sorted world.loot_rolls keys. |
| GetNumArenaOpponents | Lua 0 | No native arena-roster projection established by this audit. |
| GetNumArenaOpponentSpecs | Lua 0 | No native spec-count projection established by this audit. |
| ClearCursorHoveredItem | Lua one nil | No hover-state transition implemented; return arity is not native-proved here. |
| SetCursorHoveredItem | Lua one nil | Ignores itemLocation; no hover-state transition implemented. |
| SetCursorHoveredItemTradeItem | Lua one nil | Ignores enabled flag; no hover-state transition implemented. |
| GetNumGuildPerks | Lua 0 | Unmodeled perk query. |
| RequestGuildRewards | Lua one nil | No request/event transition; do not invent a completion event. |
| GetGuildRenameRequired | Lua false | No rename-policy input modeled. |
| GetAvailableBandwidth | Retail Rust model and Lua block excluded; Forever Lua max of first two GetNetStats results | Existing performance_bar.rs registration owns Retail. R88l.16/R88ap.7 concern native bandwidth; do not call the Forever heuristic verified. |
| GetSpellConfirmationPromptsInfo | Retail Rust model and Lua block excluded; Forever empty-table fallback | globals/register.rs:201–208 gates modeled registration; real/spell_confirmation_prompts.rs:182 publishes it. Preserve feature scope. |

The remaining nonfunction entries are namespace/constant writes, not callable
providers: C_GuildBank ensure, ACCOUNT_BINDINGS/CHARACTER_BINDINGS aliases,
EVENT_TOAST_MANAGER_OFFSET_Y_OVERRIDE unconditional false and
CLOCK_TICKER_Y_OVERRIDE truthy-preserving fallback. They are not silently
retired or reclassified as native function definitions by this task.

### Approved world-entry original-consumer proof boundary

Selected Retail source
`Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrame.lua:36` defines the
original OnLoad. Lines 65–68 explicitly register SpellBookInWorldEvents only
when IsPlayerInWorld is true, with a comment identifying load-on-demand after
world entry as the case. Lines 120–124 register the same events on
PLAYER_ENTERING_WORLD and unregister them on PLAYER_LEAVING_WORLD. The
lifecycle event is in the frame's event list at line 14. This provides an
observable boundary: pre-world load must not take the already-in-world branch;
world entry then enables the subscriptions; a later LoD load enables them
immediately. Tests should inspect original consumer effects rather than only
assert the API result or inspect the provider's implementation.

The coordinator-approved mapping uses the existing simulator in_world field.
The state must be true before dispatch so a consumer evaluating during
PLAYER_ENTERING_WORLD sees the entered state. That timing is an explicitly
INFERRED simulator contract approved in #1934, not a public native return proved
by R88l.3. The exact original-consumer test verifies both subscription phases
and observes true inside the original frame's world-entry dispatch, repeating
the sequence after ReloadUI reconstruction. Forever's existing shutdown path
also clears the state before PLAYER_LEAVING_WORLD; no new shutdown event is added.

### Grouped region/language follow-up

Resolve the remaining normal return paths from R88l.4–7 as one bounded query:
for region, establish final result count/nilability and runtime selector
initialization/update ownership; for default/alternative languages and indexed
language enumeration, establish no-player, empty-list and populated-list return
shapes, entry filtering/order, index bounds/coercion, and the authoritative
language-change refresh. R88l.4's verified lookup entries/image seed and
R88l.5–7's partial pushes/parser branches are useful implementation inputs, not
proof of complete public contracts. Do not independently invent region or
language events, default languages, or alternate-language behavior merely to
replace these constants.


## Scope correction #1939: chat proof blockers

The chat-byte proposal is not part of this implementation. Existing fallback
behavior remains unchanged. Original consumers cannot yet provide the required
proof without expanding unrelated API scope:

| Profile | Original path | Earlier blocking calls | Intended chat read |
| --- | --- | --- | --- |
| Retail | Blizzard_SettingsDefinitions_Frame/Social.lua:GetDisableChatValue, line 47 | Line 50 calls C_SocialRestrictions.IsMuted and IsAgeVerificationRestricted | Line 54 calls IsChatDisabled |
| Forever | Blizzard_SettingsDefinitions_Frame/Social.lua:GetDisableChatValue, line 47 | Line 50 calls C_SocialRestrictions.IsMuted and IsAgeVerificationRestricted | Line 54 calls IsChatDisabled |
| Retail | Blizzard_Communities/CommunitiesFrame.lua:ValidateDisplayMode, line 978 | Selected-club branch calls absent C_Club.IsAccountMuted at line 984 | Line 985 assigns accountChatDisabled |
| Forever | Blizzard_Communities/CommunitiesFrame.lua:ValidateDisplayMode, line 1022 | Selected-club branch calls absent C_Club.IsAccountMuted at line 1028 | Line 1029 assigns accountChatDisabled |

The exact original age-related label is IsAgeVerificationRestricted, not
IsAgeRestricted. No mock replacement of those preceding calls or additional
API model is introduced to manufacture a passing consumer proof. The native
chat question above remains an audit follow-up, not active implementation.

### Forever original SpellBook lifecycle

Forever uses an original Camelot wrapper, not a replacement that bypasses the
shared world-state gate. In
Blizzard_PlayerSpells/Camelot/SpellBook/Blizzard_SpellBookFrame.lua:43–45,
BaseOnLoad captures SpellBookFrameMixin.OnLoad and the wrapper calls
BaseOnLoad(self). The captured shared implementation is defined in
Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrame.lua:38 and reads
IsPlayerInWorld at line 60. Its PLAYER_ENTERING_WORLD/PLAYER_LEAVING_WORLD
branches at lines 137–140 register/unregister the in-world events. Thus the
original shared pre-world/LoD gate executes through the Camelot path too.

Before loading that consumer, the test must explicitly call
common::supply_empty_forever_spellbook_outfits (tests/common/mod.rs:364).
This supplies an empty-outfits scenario and disabled transmog availability,
not a production fallback or replacement consumer. An unspecified outfit input
is an error; the explicit scenario lets the original Camelot OnLoad run without
silently inventing outfit state.
