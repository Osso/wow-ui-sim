# Hyperlink tooltip requests

## Contract

R82, Retail 12.1.5.70688, is the native source. Applying this contract to PTR or Forever is INFERRED, not separately verified native behavior.

- Search the NUL-terminated input for the 22 R82 markers in native priority order, not text-position order. Accept raw tags and colored links without a leading `|H` requirement. Parse decimal and `0x`/`0X` numbers.
- Item links carry a parsed descriptor, enchant, three gems, fourth-gem scalar, link level, request context, upgrade mask, creation context, ordered bonus lists/modifiers, socket-bonus groups, crafter GUID and final scalar. Discard suffix and unique ID; never derive an owned GUID from them. Reject bonus count 16 or higher. Preserve the two wrapper integers separately; their native downstream meanings are UNKNOWN.
- Spell links select mode 0, link semantics, no ordinary override resolution and optional difficulty (`-1` when absent), not rank. Keep wrapper integers separate from the spell ID.
- Currency links send quantity unchanged, default zero, to the currency producer. Their title is type 0; they do not divide quantity by 100.
- `urlIndex`, `storecategory`, `perksactivity`, and `initiativetask` call no producer. Trade returns false; server response state is unmodelled. Neither battle-pet spelling is a branch.

## Bounded output

Item, spell and currency routes reuse their existing producers. Request fields with UNKNOWN downstream effects are parsed, not converted into invented rows. Spell difficulty is retained in the typed request; the current spell producer has no difficulty input.

Quest uses an explicitly supplied quest-log record as its record-provider seam and emits only identity type 23 and one type-0 title. No objective, level, progress or completion rows are inferred. Unit GUID lookup, achievement and instance-lock rows remain unmodelled. No simulated player/party GUID lookup is used for unit links.

Talent and PvP talent have different row sources and suppress annotations. Retail PvP talent uses the existing supplied/catalogued primary SpellID, not OverridesSpellID. Talent output remains unmodelled because no selected Retail producer of talent links was established. Non-Retail PvP talent output remains unmodelled without a row source. Other opaque record providers do not fall back to unrelated producers.

## Proof

`tests/tooltip_hyperlink_r82.rs` exercises original `GameTooltip:SetHyperlink` and `C_TooltipInfo.GetHyperlink`, plus typed parser input/output. Assertions cite R82 Q1/Q2/Q3; no native Lua miss arity is asserted. Existing catalog values used as fixtures do not prove equivalence to native build 70688.
