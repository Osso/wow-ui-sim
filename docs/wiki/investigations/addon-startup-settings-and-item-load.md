# Addon Startup Settings And Item Load

Addon startup errors in third-party addons can come from simulator compatibility surfaces that look unrelated: Settings canvas registration, secure attribute delegates, and synthetic item data loading.

## Content

### Settings canvases

**INFERRED (R15; native evidence pending):** On loaded Retail and Forever Settings panels, the original registration path leaves canvas visibility unchanged. Opening a category reparents and shows its canvas, and switching categories hides the prior canvas. The Forever original-consumer regression asserts this lifecycle and that an unrelated shown canvas stays shown.

The simulator keeps the visibility workaround only for profiles other than Retail and Forever. The bootstrap Settings fallback, including `InterfaceOptions_AddCategory`, remains separate for addons that register option panels before the real Settings implementation is available.

### Secure attribute delegates

Blizzard callback registries use forbidden-frame attribute delegates as a secure boundary. If addon-tainted code calls into `CallbackRegistryMixin:RegisterCallback()` or tooltip post-call registration, the delegate's `OnAttributeChanged` handler must run as secure code. `issecure()` also has to reflect rilua stack taint instead of returning a hardcoded `true`.

The symptom was a secure array failure during startup: `attempted to store a secret value in a SecureArray`. The root was the simulator running forbidden attribute delegate dispatch on the caller's tainted stack.

### Item subclass and synthetic item data

Search addons such as Syndicator compare `C_Item.GetItemSubClassInfo(...):lower()` with hardcoded enUS keywords. Returning `"Unknown"` for unknown subclass IDs or using generic names such as `"Axe"`, `"Mace"`, or `"Shield"` causes opaque addon errors like `unknown`, `cooking`, or `shield`. Unknown subclasses should return nil; known retail subclasses should use the real enUS keyword strings.

For item loading, many addons reference current live item IDs not present in the simulator's small seeded item DB. Treat positive item IDs as synthetically existing and cached enough for `Item:CreateFromItemID(...):ContinueOnItemLoad(...)`, but return placeholder item info for unknown IDs. This keeps Blizzard item-load callbacks from failing validation while preventing recursive addon refresh loops caused by `GetItemInfo(id)` returning nil forever.

### Verification signals

A good regression run checks both saved-vars modes:

- `target/release/wow-sim --no-saved-vars lua-errors` should return `[]`;
- `target/release/wow-sim lua-errors` should return `[]`;
- startup should remain well under the 90-second cap, including AllTheThings SavedVariables deserialization.

## Sources

- [runtime_surface_bootstrap.lua](../../src/lua_api/env_init/runtime_surface_bootstrap.lua) — Settings fallback, `issecure`, item load request fallback
- [attributes.rs](../../src/lua_api/frame/methods/text_attribute_event/attributes.rs) — secure forbidden attribute delegate dispatch
- [c_item.rs](../../src/c_api/item_spell/c_item.rs) — C_Item existence, item info, subclass APIs
- [helpers.rs](../../src/c_api/item_spell/helpers.rs) — item class and subclass names

## See Also

- [[taint-system]] — rilua stack taint and secure API behavior
- [[addon-loading]] — addon startup and SavedVariables loading flow
- [[api-coverage]] — C_* compatibility surface tracking
