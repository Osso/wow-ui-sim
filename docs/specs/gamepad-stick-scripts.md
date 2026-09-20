# Gamepad stick scripts

## Contract

- Frames accept canonical `OnGamePadStick` in XML and Lua script APIs.
- `OnGamepadStick`, used by Forever Lua, identifies the same binding for SetScript, GetScript, HookScript, removal and generic dispatch. This exact alias is source-based compatibility policy, not native-conformance evidence; other script names are not case-normalized.
- Generic script dispatch preserves self and supplied stick, x and y arguments, including hook order.
- Existing EnableGamePadStick state is unchanged. Explicit generic dispatch is not gated hardware input delivery.

## Evidence and limits

Forever UI.xsd/XML use `OnGamePadStick`; Blizzard_GamepadSharedUtility/InputBindingStack/InputAxisBinding.lua uses `OnGamepadStick`. No host gamepad integration or native alias/argument validation claim.

## Tests

`tests/gamepad_stick.rs`: Lua alias identity, hook order, removal, unrelated-case rejection, XML binding and concrete generic dispatch arguments.
