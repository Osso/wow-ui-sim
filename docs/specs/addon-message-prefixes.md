# Addon-message prefix registration

Addon startup can subscribe to addon-message prefixes through three state-backed
`C_ChatInfo` functions. The registry belongs to each Lua environment and is
independent of outbound message logging or delivery.

## Surface and behavior

Retail, PTR, Mists, Era, Anniversary and Forever publish:

| Function | Result |
|---|---|
| `C_ChatInfo.RegisterAddonMessagePrefix(prefix)` | One `Enum.RegisterAddonMessagePrefixResult` number |
| `C_ChatInfo.IsAddonMessagePrefixRegistered(prefix)` | One boolean |
| `C_ChatInfo.GetRegisteredAddonMessagePrefixes()` | One fresh Lua array of registered byte strings |

Wrath keeps its existing legacy surface; this change does not install these
namespace members there.

The registry starts empty. Prefix identity is case-sensitive and preserves
non-NUL bytes, including non-UTF-8 strings. Missing or non-string prefix
arguments fail rather than being coerced. Each snapshot is independent of the
registry and previous snapshots; modifying it cannot change subscriptions.

Registration validates in this order:

1. Empty prefixes or prefixes longer than 16 bytes return `InvalidPrefix` (`2`).
2. An existing prefix returns `DuplicatePrefix` (`1`), including at capacity.
3. A new valid prefix at the 512-prefix capacity returns `MaxPrefixes` (`3`).
4. Otherwise the prefix is stored and the call returns `Success` (`0`).

Rejected registrations leave the registry unchanged. Membership queries share
the same normalized identity and return `false` for absent prefixes. Snapshots
retain registration order in this implementation; native ordering is unverified.

## Evidence and compatibility limits

Pinned Blizzard ChatInfo and ChatConstants documentation supplies the function
signatures and result enum. The published Blizzard 4.1 announcement supplies the
historical 16-byte prefix and 512-prefix limits; current native boundary behavior
has not been measured.

The prefix parameter is documented as `cstring`. The implementation truncates
at the first embedded NUL before validation, duplicate detection and lookup.
That normalization is an explicit inference, not a measured native result.

The supplied startup log traced missing registration to Auctionator and
DialogKey's LibVersionCheck. EllesmereUI's LibKeystone and LibSpecialization
also use the API, but their installed TOC filters select Standard and exclude
Forever. Cache presence alone does not establish an executed consumer path.

This change adds subscriptions and queries only. Outbound senders, message
delivery, incoming events, network filtering and native throttling are outside
this registry's scope. No generated catalog or external runtime data is required.

## Implementation and verification

- [Registry](../../src/c_api/addon_message_prefixes.rs): byte-preserving state and Lua callbacks.
- [Namespace registration](../../src/c_api/mod.rs) and
  [environment state](../../src/lua_api/state/sim_state.rs): profile publication and isolation.
- [Regression tests](../../tests/addon_message_prefixes.rs): state changes, exact result codes,
  byte limits and round trips, fresh snapshots, capacity preservation, environment
  isolation, inferred NUL normalization and Wrath namespace absence.

The tests exercise modeled API behavior. They do not establish current native
boundary parity or installed-addon startup compatibility.

## Sources

- [Pinned ChatInfo documentation](https://raw.githubusercontent.com/Gethe/wow-ui-source/15881d522c44cd106d3d8660106ec9f9d2f44167/Interface/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua): API signatures.
- [Pinned ChatConstants documentation](https://raw.githubusercontent.com/Gethe/wow-ui-source/15881d522c44cd106d3d8660106ec9f9d2f44167/Interface/AddOns/Blizzard_APIDocumentationGenerated/ChatConstantsDocumentation.lua): registration result enum.
- [Blizzard's 4.1 announcement](https://www.bluetracker.gg/wow/topic/us-en/2228413591-patch-41-addon-messages-will-be-filtered/): historical limits.
- Supplied November 2, 2026 startup log and installed addon TOC inspection:
  affected consumers and Standard-only library selection.

See the [outbound addon-message contract](addon-messages.md) for the separate
sender behavior.
