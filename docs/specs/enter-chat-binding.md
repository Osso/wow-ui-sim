# Enter-to-chat binding

Current Retail/PTR simulator default Enter dispatch must open the unchanged Blizzard chat editor from an unfocused startup state. Historical Blizzard manual §5.2 describes Enter opening the chat prompt; this grounds the usability requirement, not an exhaustive current native keymap. Cached Retail/PTR `Blizzard_DeprecatedChatInfo/Deprecated_ChatFrame.lua` aliases `ChatFrame_OpenChat` to the whole `ChatFrameUtil.OpenChat` consumer. Default binding is simulator policy; original handlers and configured bindings remain authoritative within this model.

## What it must do

- [ ] An unmodified Enter key through ordinary loaded-game dispatch opens/focuses the real chat editor. Subsequent printable input reaches that editor; subsequent Enter uses its original submit handler, clearing text and releasing focus without reopening chat.
- [ ] Explicit Enter binding overrides the default; explicit Enter unbinding suppresses it. Do not alter configured/account bindings to install this default.
- [ ] Preserve focused EditBox special-key dispatch and existing non-Enter bindings. No copied/replaced Blizzard handlers or VNC-specific keyboard path.

## How it works

- [Event system](../event-system.md)
- [Prefork harness](../wiki/systems/prefork-test-harness.md)

## Implementation inventory

- `src/lua_api/globals/keybindings.rs`: existing binding actions/default-key dispatch; Enter uses `OPENCHAT` and the unchanged `ChatFrame_OpenChat` consumer.
- `src/lua_api/key_dispatch.rs`: existing focused EditBox and general key dispatch; unchanged.
- `src/iced_app/view.rs`: existing native/toolkit keyboard subscription; unchanged.

## Tests asserting this spec

- [chat_enter_binding.rs](../../tests/chat_enter_binding.rs): three loaded Game-parent/fork cases. At `7e8c5ffb6`, default activation failed before downstream typing/submission, while configured/cleared preservation controls passed; prefork exit1. Retail3/3 completed at `7d9d9f621`, including downstream typing/submission and override/unbind controls. [Packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-2026-10-12/README.md). No PTR execution credit at that epoch.
- PTR ordinary libtest wrappers in [chat_enter_binding.rs](../../tests/chat_enter_binding.rs): `chat_enter_binding::ptr_default_enter_opens_real_chat_for_typing_and_submission`, `chat_enter_binding::ptr_explicit_enter_binding_overrides_default_chat_action`, and `chat_enter_binding::ptr_explicitly_cleared_enter_binding_stays_unbound` — PTR3/3 PASS at `f3145424c`, [packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-ptr-2026-10-12/README.md). Each uses the existing process timeout, a fresh environment/display and unchanged `edit_mode_cache_startup::load_whole_game_ui`, invokes its original case's `run` body, then asserts collected loader warnings are empty. Bounded PTR runtime PASS only; prefork profile eligibility is unchanged.
- [chat_frame.rs](../../tests/chat_frame.rs): existing click/type/submit consumer proof; not a default Enter activation test.

## Known gaps (current cycle)

- [x] Bounded Retail3 GREEN after genuine default-activation RED; typing/submission unreached in RED, completed in GREEN. Formatter/default check PASS at the recorded source scope.
- [x] PTR3 original Enter bodies PASS through fresh whole Game startup at `f3145424c`; normal zero Lua errors and PTR fmt/check PASS. No PTR GUI/native or prefork-cache/process credit. Current GUI initially had native PointerRoot focus and zero Enter deliveries; on the same artifact, native focus assignment produced five deliveries and user confirmation that chat and `/command` both worked. Fresh automatic-focus startup/owned-native-input gate independently PASS ([packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/native-window-focus-2026-10-12/README.md)); earlier zero-delivery XTest probe remains invalid application RED. The user authorized stopping after each test instead of automatic90s expiry; CPU/memory limits preserved. Public packet retention/link/privacy gate PASS through `b1c7c4348`; [gate](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/edit-mode-enter-2026-10-12/public-retention-gate.json).

## Out of scope

Native account-keymap parity, network chat delivery, all-key/gameplay support, vendor changes, original WTF mutation, and treating GetBindingAction's current omission of simulator convenience defaults as proof that dispatch has no default. Earlier-profile/native acceptance remains separate.
