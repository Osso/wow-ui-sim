# Forbidden-aspect script execution and auto-focus

Patch 12.1.0 `UntrustedScriptExecution` / `UntrustedLayoutScriptExecution` suppress addon-installed script handlers; EditBox auto-focus respects the `Shown` secret aspect. Stored masks and inheritance follow [forbidden-aspect inheritance](forbidden-aspect-inheritance.md).

## What it must do

- [x] On a frame owning `UntrustedScriptExecution` (own or inherited), engine dispatch skips handlers whose closure carries addon taint; untainted (Blizzard/simulator) handlers still run. Bindings stay installed.
- [x] On a frame owning `UntrustedLayoutScriptExecution`, the same rule applies to `OnSizeChanged` only. Anchored frames already own the aspect (SetPoint rejects implicit gain).
- [x] `HookScript` chains gate each component when it runs, so an addon hook on a secure handler is skipped without dropping the secure part.
- [x] `SetHyperlinkPropagateToParent` rejects tainted callers on `UntrustedScriptExecution` receivers (`ChecksForbiddenAspects`).
- [x] An EditBox with `SetAutoFocus(true)` takes focus after its `OnShow` handlers when it becomes visible, unless the `Shown` secret aspect applies to it or an ancestor.

INFERRED: "lives in the Forbidden Partition and execution is untainted" is modeled as the handler closure being untainted; the dispatching stack is ignored so Blizzard intrinsic `OnLoad` handlers still run for addon-created frames. Auto-focus checks the ancestor chain like `IsVisible` readability.

## Implementation inventory

- `src/lua_api/frame/methods/forbidden_aspects.rs` — suppression mask, handler taint, hook gate.
- `src/lua_api/script_helpers.rs` — dispatch lookups (`get_script_handlers_for_dispatch`, `ScriptHandlerLookup`, `get_dispatch_script`).
- `src/lua_api/frame/methods/text_attribute_event/events/script_binding_args.rs` — gated hook chains.
- `src/lua_api/frame/methods/core_state/visibility.rs`, `widgets/editbox.rs` — auto-focus on show.

## Tests asserting this spec

`tests/forbidden_aspect_enforcement.rs`: `untrusted_script_execution_runs_only_secure_handlers_on_restricted_hierarchy`, `untrusted_layout_script_execution_suppresses_only_addon_size_handlers`, `aura_button_aspects_block_addon_handlers_and_input_on_button_and_children`, `autofocus_editbox_skips_focus_while_shown_secret_aspect_applies`.

## Known gaps

- The simulator never dispatches `OnSizeChanged` from layout changes; the layout rule is proven only through `fire_script_handler`.
- XML `autoFocus` is not parsed and the EditBox default stays `false` (native default is `true`).
- Handlers chained by XML `inherit="prepend|append"` are not gated per component.
