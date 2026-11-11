# Glue login submission

## Scope and status

Task [lt-a667](https://tasks.sakuin.org/tasks/lt-a667) covers the real Blizzard
login UI on Retail 12.1.5.70575 and WoW Forever 1.60.1.70707. Implementation is
pending the native contract requested in task comment 22 and routed as R40
by the coordinator in comment 27. This document records
the acceptance target and observed consumers, not an implemented login model.

## Simulator policy

The simulator accepts arbitrary credentials without contacting an authentication
service. This is explicitly simulator policy, not a claim about native credential
validation. Tests use invented account/password strings. Account, realm and
character data must be supplied by the host, with empty or evidenced defaults;
no account or server records may be fabricated to force the transition.

## Original consumers

Paths below are relative to each profile's pinned `AddOns` cache.

- Both profiles select `[Family]/AccountLogin.lua` and `.xml` in
  `Blizzard_GlueXML/Blizzard_GlueXML.toc`; their family is Mainline. Forever also
  selects `Camelot/AccountLoginOverrides.lua`, which adjusts the account box
  anchor without replacing submission.
- `Blizzard_GlueXML/Mainline/AccountLogin.lua:204` (Retail) and `:213` (Forever)
  call `C_Login.Login(username, PasswordEditBox)` from `AccountLogin_Login`.
  The second argument is the actual EditBox, not its text string.
- Retail `Mainline/AccountLogin.xml:104,137-141,150-155` wires the edit boxes'
  Enter handlers and the login button to the real login functions.
- `Blizzard_GlueParent/Mainline/GlueParent.lua:157` (Retail) and `:172` (Forever)
  handle `LOGIN_STATE_CHANGED` by ensuring a valid screen and updating dialogs.
- The same file reads `C_Login.GetState` at Retail `:203,216,243` and Forever
  `:221,242,277`. Retail consumes multiple returns; Forever consumes a table.
  The current simulator exposes those shapes, but this does not establish the
  native successful-login transition or event chronology.
- `Blizzard_GlueXML/Mainline/CharacterSelect.lua:401,594` (Retail) and `:407,609`
  (Forever) handle `CHARACTER_LIST_UPDATE` and `LOGIN_STATE_CHANGED` respectively.

## Acceptance tests

`tests/glue_login.rs` adds `login_submit_enter_reaches_character_select` and
`login_submit_button_reaches_character_select`, gated to Retail and Forever.
Each boots the cached Blizzard login UI, hit-tests the visible centers of both
edit boxes, clicks them, types through key dispatch, and verifies the resulting
text. Submission uses the password box's Enter handler or the hit-tested login
button. Ordinary update/timer ticks follow; tests never switch the screen,
replay startup after submission, or directly invoke a simulator login shortcut.
The required result is visible `CharacterSelect` and hidden `AccountLogin`.

## Open native questions

Task comment 22 requests `C_Login.Login` argument validation and returns,
successful state transitions, event ordering/payloads and dispatch timing for
both pins, and how empty host-supplied data affects that path. Until answered,
no new API implementation or event sequence is asserted here.

## Preparation boundary

Coordinator comment 27 permits non-native credential policy and host-supplied
input preparation, but explicitly keeps native implementation blocked until R40
is answered. Credential acceptance does not authorize publishing `C_Login.Login`
with invented returns, argument validation, connection state, or events.

The existing character-list bootstrap is temporary compatibility behavior:
`src/lua_api/workarounds/temporary/glue_character_select_defaults.rs` publishes
an empty group list and directly invokes selected character-list event handlers.
It is not evidence for native event order or a suitable new authentication
producer. Likewise, the empty Forever subregion/district catalogs in
`forever_glue_display_defaults.rs` do not specify populated record schemas.
New unused input types are deferred until their actual consumer contract is
known; no speculative account/realm/character records are introduced.

The five pre-existing Forever character-select/create failures are owned by the
coordinator's `forever-int-clean` branch (lt-95ac comment 28). Rebase onto master
when those fixes land; do not duplicate them in this task.

Both new submission regressions have now reproduced the gap on both profiles:
Retail reports missing `C_Login.Login` at `AccountLogin.lua:204`, Forever at
`:213`. These are RED observations, not passing acceptance results.
