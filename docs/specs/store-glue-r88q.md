# Store Glue q1 purchase-state reader

**VERIFIED returning path, Retail12.1.5.70688, R88q.q1:** `GetVASPurchaseStateInfo` consumes argument1 as accessible string/number, ignores extras, and publishes signed32 productID, signed32 service state, and an errors table in that order. A missing record supplies `(0, 0, {})`. Only a nonempty error list whose first value is nonzero is copied; later zero values remain in order. Wrong-type/missing/nil input uses the native Usage reason. Protected-wrapper/cleanup exits remain UNKNOWN, not a blanket three-result claim.

This stage replaces only the guessed q1 reader with explicit empty purchase-record backing. Per coordinator1283, resolved GUIDs and aliases are host supplied; general grammar remains UNKNOWN pending R88w.w2. Unrecognized input uses a neutral empty resolver result, not a claim that every noncanonical string is malformed. Other q2–q5 routes and compatibility state are not retired by this stage.

The pinned Retail original consumer is `Blizzard_GlueXML/Mainline/CharacterSelect/CharacterSelectListUtil.lua:573–580`. `Blizzard_GlueXML_Mainline.toc:6–7` selects Glue/mainline and line21 selects that file. Generated StoreGlue/StoreSecure declarations are absent; native evidence is R88q.

The optional q1 regression uses two distinct backed GUID records plus absence and calls the original selected consumer. Tracker receipts record actual compiler-bound RED/GREEN results. Product189 is supplied through the existing product-catalog input so both VAS records reach the original product lookup.
