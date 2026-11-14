# Tooltip comparison (lt-ff6d, R88n.6)

## Verified Retail boundary

R88n.6 (Retail 12.1.5.70688) establishes the ordinary prototype and result routes.

- Comparison items are tables. An accepted nonzero decoded item GUID takes precedence over a link, independently of inventory lookup; otherwise a hyperlink containing item: is used. Invalid required arguments raise errors. An optional finite signed-32 numeric or numeric-string item level truncates toward zero; an invalid optional value is ignored.
- CompareItem checks the descriptor, the backed GameTooltip, and optional backed Frame in that order; success returns no values. Only the native typed callback container is established, not its public event spelling. The simulator has no installed comparison bridge; it does not emit a guessed event, call a Lua manager directly, or assign an inferred tooltip hook.
- GetItemComparisonInfo publishes no values on producer failure and one record on success, with method, item, and a one-based additionalItems table even when empty. Existing Enum.TooltipComparisonMethod already supplies 0/1/2/3.
- GetItemComparisonDelta checks two required descriptors, an optional paired descriptor, and optional Lua truthiness; it does not infer an equipped operand. On success it publishes one one-based table of strings, including empty; unresolved inputs return no values.

The simulator reuses ordinary object backing and available access restrictions. Arbitrary native guard states, full contextual error decoration, native stack-capacity outcomes and unsupported restricted-object cases are not claimed. Do not loosen any access check to get comparison output.

## Explicit simulator inference

Use the current SimState equipment and bag storage, existing item descriptor/catalog and item-link builders, and the existing host-supplied item tooltip stat rows keyed by effective item level. No default equipment or stat data is inserted. A syntactically recognized simulator item GUID does not imply occupancy; resolve it from the live bag slot. Broader native GUID grammar is not modeled. Equipped copies have no public simulator GUID, so the comparison record uses the existing hyperlink, not an invented GUID.

Ordinary matched worn armor/accessories use simulator inventory slots and Single method. Use the second populated matching ring/trinket slot as primary with the first as additional, consistent with the verified two-match behavior without claiming the native slot scan's exact order. Unmodeled weapon-special/bag optimization branches are not guessed.

The inferred arithmetic uses supplied rows only. Delta is proposed minus equipped; paired rows go to the proposed side when addPairedStats is truthy, otherwise to the equipped side, as suggested by the original consumer's two-hands versus bag-pair branches. Missing host stats are not fabricated or silently zeroed; explicitly empty or equal supplied results can publish an empty table. Output rows use deterministic signed host-label text; native localized formatting, colors, ranking and two-entry cache are not modeled.

## Sources and profile applicability

Original Retail SharedXMLGame TooltipComparisonManager.lua:11–20,148–165,208–260,290–292 and Mainline GameTooltip.lua:483–490. The original Retail manager test uses real GameTooltip and ShoppingTooltip frames, two catalog helms, and host-supplied stat rows, and independently rejects constant Info and Delta. Native event registration and login/equipment population remain unknown. Matching generated TooltipComparisonDocumentation.lua declares the surface on PTR and Forever; publishing the Retail simulator adapter there is separate inference, not another native or original-consumer claim.
