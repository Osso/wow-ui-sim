# Retail inventory candidates and purchase records

R88k.1/.2 (Retail 12.1.5.70688, tracker lt-52e7 comments1088/1089)
establish synchronous inventory candidate publication and the conditional
normal-return refund query. Both APIs are now modeled for Retail only.

## VERIFIED conditional behavior

`GetInventoryItemsForSlot` accepts numeric conversion (subtract one, truncate to
signed64, test low unsigned32 against 0–29), or a case-insensitive supplied slot
name. Native name identifier numbering is not established; see the separate
INFERRED adapter below. An invalid numeric slot returns zero values before checking the table.
An accepted slot requires a table and uses the exact general Usage reason.
The optional mode compares ASCII case-insensitively through its first NUL.
Absent player, or missing selected-item/record preconditions for transmog mode,
returns zero values. Completion returns the same table, even for an empty scan.

Accepted candidates replace packed-location-plus-one keys without clearing
unrelated entries. A selected link builder with no text removes its key.
Equipment, carried, bag0–3 and conditional bank5–10 keys use the supplied native
formulas. Carried enable/count and interaction-GUID-kind/transmog bank gating are explicit. The supplied
interaction Boolean means nonzero GUID kind, not nonzero arbitrary GUID bytes.
Actual writes check tainted-access and frozen-table restrictions; empty scans do
not invoke those write guards. No event or asynchronous completion is invented.

`C_Container.GetContainerItemPurchaseInfo` parses all three required arguments,
including the container on the equipped route. Signed container conversion and
unsigned slot conversion have different subtraction/truncation order; truthiness
selects the equipped route. Required/range errors use the Usage reason and the
VM's caller-aware argument-error formatter, including method adjustment.

Native inventory/container bounds are checked before selecting a supplied item
identity. Missing player/item/record, excluded identity, or nonpositive remaining
refund time returns zero values, never one nil. A valid record returns one table
with exactly money, itemCount, refundSeconds, currencyCount, hasEnchants. Quantities
count positive entries among five slots rather than summing. Refund seconds use
expiration minus refund clock plus7200. Modification checks continue after a
failed flags test and do not suppress the refund table.

## INFERRED slot-name adapter

The host `slot_names` map uses one-based identifiers and maps a nonzero ID to
internal ID minus one. This is an INFERRED simulator adapter, not a VERIFIED
native numbering rule. R88k.1 proves case-insensitive lookup and a nonzero usable
identifier but does not establish that stored identifier's numbering. The map
is empty by default. `inventory_purchase_inferred::` tests this adapter separately;
VERIFIED numeric slot tests do not use a named-slot fixture.

## UNKNOWN host seams and limitations

All new input maps are empty by default. Candidate eligibility, slot-name
collection population, transmog selected-record availability, link builder text,
native item identities and purchase-record/refund-clock writers remain explicit
host inputs. The simulator does not synthesize link fields, refundable records,
item flags, server messages or native filters. Native-indexed purchase maps are
not automatically inferred from the simulator's different legacy bag indexing.
A host must supply the corresponding native identity/count state.

This is the proved normal-return route, not parity for arbitrary internal guards.
Secret access delegates to the existing VM untainted-access policy; alternate
native access-mode configuration and the readers' native stack-reservation
failure are not implemented or claimed. Full native debug source-line prefix
parity is not claimed separately from the VM formatter. Numeric inventory
conversion tests cover the established ordinary finite examples, not undefined
out-of-signed64 conversion behavior.

## Tests and consumers

`inventory_purchase_getters::` covers diagnostics/arity, numeric strings and
fractional conversion, supplied-table identity, key replacement/removal/stale
entries, carried/bank gating, transmog/frozen writes, native selection ranges,
positive-entry counts, independent modification checks, and expiration/exclusion.

`inventory_purchase_consumers::` loads the normal selected Retail Game startup
and verifies the Mainline TOC and original source paths. The original
`PaperDollFrameItemFlyout_GetItems` forwards two explicit host link-text inputs
and then no-text into the same supplied table, asserting distinct changes and
preserved sentinel. These strings are publication fixtures, not complete native
item links or evidence of unknown link construction.

The original `ContainerFrame_GetExtendedPriceString` uses a real pooled item
button, current bag item, and two supplied money-only records. A terminal popup
observer retains the original function and checks exact changing money text;
expiration must produce false without another popup. Neither selector replaces
its target API or original consumer, and each rejects a constant/no-op producer.
Positive item/currency counts are API-level tests because the consumer then calls
separate purchase-item/currency APIs outside this task. No Blizzard Lua is edited.

These are proposed selectors, not a receipt claiming successful execution.
Final gates belong to the immutable candidate SHA.

## See Also

- [[merchant-item-link]] — explicit payloads and bounded serializer evidence.
- [[consumer-selection-and-red-proof]] — selected original consumer requirements.
