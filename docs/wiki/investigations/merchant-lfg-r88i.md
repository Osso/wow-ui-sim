# Merchant requirement and LFG request boundaries

Retail R88i.5/R88i.6 replace unconditional merchant requirement success and LFD
request no-ops. Native evidence is Retail 12.1.5.70688; other profiles keep their
previous behavior. This is not native parity evidence for the selected UI build.

## Merchant requirements

The category-1 item-record gate distinguishes a missing record (zero results)
from a missing requirement owner or false requirement test (one false). Zero
requirement ID skips the test. The separate cost-record gate runs only for a
nonzero value and signed mode 0 through 8. These control-flow facts are VERIFIED;
unit tests supply predicate outcomes without asserting comparison direction.

`merchant_offer_categories` supplies explicit offer categories; absent entries
use category0 as a simulator policy, not a native default. Item records are
explicit in `merchant_item_requirements`. The MinFactionID/MinReputation names,
reading the existing `__wow_reputation_state.factions` reaction rank, and the
`>=` comparison are INFERRED. No duplicate faction model is created. Likewise,
`merchant_player_ratings` and the cost fields' RequiredArenaRating/ArenaBracket
interpretation and `>=` comparison are INFERRED. Missing scenario values do not
silently satisfy requirements. Live data and helper event effects remain UNKNOWN.

The original Retail MerchantFrame_Update consumer is exercised with explicit
scenario input and a forwarding observer that preserves producer arguments and
all return values. The selected original reaches affordability at line290, then
stops at unrelated missing GetMerchantItemLink at line329. Full money-font
presentation is unclaimed. These scenario calls are not proof of the inferred
data mapping. The existing signed-index parser and
`integer overflow attempting to store %f` diagnostic remain unchanged.

## LFD lock requests

VERIFIED: player and party requests ignore supplied arguments, apply two
throttles, and submit the same request with player flag1/0 when connected.
Player returns exactly one boolean, including true when admitted but disconnected.
Party boolean values are verified, but complete arity is UNKNOWN: the model's
single return is an explicitly provisional convention.

Two host throttle flags default to admit. Their durations/reset policy are a
neutral seam, not a clock approximation. Admission bookkeeping and the request
outbox model transport intent; they never synchronously fill lock information.
The existing LFGLockList seed is retained. No LFG_LOCK_INFO_RECEIVED or
LFG_UPDATE_RANDOM_INFO event order is invented. The original selected
Blizzard_GroupFinder Mainline QueueUpdater supplies both request flags.

## Sources

- Tracker lt-f07c #1016: coordinator's verbatim clean-room R88i.5/R88i.6 handoff.
- [Merchant gates](../../../src/lua_api/globals/real/merchant_requirements.rs).
- [Merchant API](../../../src/lua_api/globals/real/merchant_affordability.rs).
- [LFD request model](../../../src/lua_api/globals/real/lfd_lock_requests.rs).
- [Merchant consumer tests](../../../tests/reset_set_merchant_filter.rs).
- [LFD tests](../../../tests/lfd_lock_requests.rs).

## See Also

- [[consumer-selection-and-red-proof]]: runtime applicability versus API tests.
- [[lua-owned-fallback-retirement]]: separate original Lua fallback retirements.
