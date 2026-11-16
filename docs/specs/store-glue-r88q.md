# Store Glue VAS state: R88q and R88w

## Native evidence and boundaries

**VERIFIED returning paths, Retail 12.1.5.70688:** R88q.q1–q5 (lt-941b comments1223–1225) establish the following modeled contracts. These are not complete proofs of every cleanup-guard or protected-wrapper exit.

- q1 accepts argument1 as accessible string/number, ignores extras, and publishes `(productID, state, errors)` with signed32 numeric fields. Missing records publish `(0, 0, {})`. Error arrays preserve order and later zero elements only when their first element is nonzero. Missing/nil/wrong-type arguments use the native Usage reason. Failed GUID parsing resolves to the empty GUID, which may itself have a matching record.
- q2 disconnect and q3 product readiness are independent shared Boolean state, initially false. Initial false is not a reset on absent records, session loss or disconnect. Their setters accept non-nil Lua truthiness and return zero values; restriction checks precede argument parsing/storage. Direct setter/clear operations do not invent lifecycle or readiness events. Stores precede pending-error checks.
- q3 completion conditionally sets readiness true using two tested input values and two tested flags; failed tests leave existing readiness untouched. Completion data is copied before `STORE_VAS_PURCHASE_COMPLETE`, whose payload is verified to contain zero values, whether or not readiness was set.
- q4 no matching local character record means no preparation or sequence increment. Matching records prepare independently on repeated calls, including when disconnected; connection presence gates submission. Only the response producer emits `VAS_CHARACTER_QUEUE_STATUS_UPDATE`, matched by opaque correlation. Payload is `(matched GUID, signed32 number)`, with nil for an empty matched GUID; unmatched responses emit nothing. The numeric unit is UNKNOWN.
- q5 refresh reads no arguments and attempts a fixed submission only with a connection. Bulk receive replaces the complete purchase-state collection before its single state-change notification, including an empty update. Per-character removal clears the separate error list first; permitted state>1 updates use an explicit host predicate. Requests do not manufacture records or responses.

## Neutral host seams and later questions

The original dispositions1283/1285 established these seams; the R88w sections below resolve individual contracts in separately verified changes:

- R88w.w2 resolves the bounded ASCII GUID grammar and exact Opaque alias branch as documented below. Unresolved forms remain neutral, not classified malformed. Purchase states and queue-character records remain separate host inputs.
- Queue correlation is opaque host data. Simulator request counters do not establish native width, overflow or wire layout. No public packet name/opcode is invented.
- Restriction metadata is explicit; Lua taint alone is not treated as native restricted-execution metadata. Secret helpers apply only to matching known contracts.
- Completion values/flags and state>1 permission have neutral explicit inputs; their service meanings are UNKNOWN. Pending errors are host supplied and are not automatically cleared.
- R88w.w1/w4 resolve the earlier purchase-error payload and copy-gate questions: the implemented zero-payload notification and native copy gate are documented below.
- R88w.w3 resolves all three state-change notifications as zero payload; no host-controlled payload remains.
- Cleanup failures, complete protected-wrapper exits, exhaustive lifecycle clears, and server behavior remain unmodeled rather than receiving guessed fallbacks.

## Selected original Retail consumers

`Blizzard_GlueXML/Blizzard_GlueXML_Mainline.toc:6–7` selects Glue/mainline. Lines21,24,33 select respectively `Mainline/CharacterSelect/CharacterSelectListUtil.lua`, `.../CharacterSelectListElements.lua`, and `Mainline/CharacterSelect.lua`.

| Original source | Direct API use |
| --- | --- |
| `CharacterSelect/CharacterSelectListUtil.lua:573–580` | Reads q1 tuple in native product/state/errors order and obtains product information. |
| `CharacterSelect.lua:200` | OnShow requests purchase-state refresh. |
| `CharacterSelect.lua:208–211` | OnShow reads disconnect, explicitly sets it false, then requests disconnect. This is an original Lua writer, not an automatic native reset. |
| `CharacterSelect.lua:337–342` | OnUpdate reads readiness when no queued upgrade, then drives original character/store consumers. |
| `CharacterSelect/CharacterSelectListElements.lua:791–792` | WaitingOnQueue/no cached time branch requests queue status without an API-existence guard. |
| `CharacterSelect.lua:591–593,694–699` | Queue event forwards the matched GUID/value into original queue-time storage. |

Original StoreUI may consume readiness: `Blizzard_Shared_StoreUIInsecure.lua:55–58` explicitly clears it during character-list update. This must not be confused with a producer reset. The direct completion event handler in `Blizzard_Shared_StoreUISecure.lua:2523–2532,2620–2636` changes its own Lua-local `VASReady`, not the modeled Boolean via `ClearVASProductReady`.

Generated C_StoreGlue/C_StoreSecure namespace declarations were not found in the pinned Retail cache. Native publication evidence comes from R88q, not those absent generated files.

## Fallback inventory

On modeled profiles, five guessed functions formerly installed by `src/lua_api/workarounds/temporary/store_glue_state.rs` are replaced: `GetDisconnectOnLogout`, `GetVASProductReady`, `GetVASPurchaseStateInfo`, `RequestCharacterQueueTime`, `UpdateVASPurchaseStates`. The guessed `_state`/`__wow_store_glue_state` fixture is not the real model. `ClearVASProductReady` and the two secure setters are independently modeled from R88q, not claimed as five additional functions in that temporary file. Unchanged profiles retain their old compatibility behavior. Existing product/group/completion-detail compatibility surfaces outside this task remain explicitly bounded; this does not retire the entire Store subsystem.

The duplicate `SetDisconnectOnLogout` and `SetVASProductReady` definitions and `GetVASErrors` fallback in `runtime_surface_bootstrap.lua` stay active only outside modeled profiles. **UNKNOWN — Mists 5.5.4.70268:** this lane does not apply Retail native behavior to Mists; keep its existing bootstrap compatibility behavior unchanged. Other unmodeled profiles also retain their existing compatibility behavior. Unrelated product/completion-detail compatibility stays unchanged.

## Verification

Optional tests drive selected original Glue consumers and real XML list templates. The R88q/R88w selectors use at least two distinct backed records plus absence; the Store UI confirmation check exercises its original setter caller. Runtime receipts are evidence artifacts, not part of this behavior spec; native parity outside Retail remains separately labeled.

## Tests asserting this spec

- `tests/glue_character_select.rs` covers the original CharacterSelect, Store VAS error, and Store UI confirmation consumers.
- `tests/store_api.rs` covers Retail setter state and completion-event ownership.

## Known downstream boundary

The original Retail OnShow resets the q2 Boolean before line211, then calls missing `C_Login.DisconnectFromServer`. Coordinator1291 approves an exact test-only pcall boundary for that downstream error and no other error, preserving every q2 pre/post assertion and then executing original OnUpdate. No C_Login fallback or writer is introduced; native evidence is pending separately.

## Existing preservation boundary

Coordinator1293 retains the pre-existing R79 `character_creation_submit_real_controls_wait_for_host_result` RED outside StoreGlue scope. Report the existing suite as35/36 plus that known RED, never aggregate PASS. The assertion and SetCameraZoomLevel Usage diagnostic must stay unchanged; receipts compare their normalized failure-core SHA-256. R79/R79b customization collector joins remain UNKNOWN and tracked separately.

## INFERRED refresh relationship (separate commit)

**INFERRED relationship:** refresh is asynchronous, based on request submission and separate receive callbacks. No proven one-to-one request/response correlation, response arrival, or ordering between independent receive types is claimed.

PTR and Forever applicability is separately [INFERRED from their selected consumers](../store-glue-inferred-profiles.md), not native declaration parity.

## R88w.w1 shared error reader (lt-941b #1308, disposition #1310)

VERIFIED Retail 12.1.5.70688: `C_StoreSecure.GetVASErrors` normally returns one independent table of ascending distinct signed32 error numbers with consecutive one-based keys. Empty source produces an empty table. The reader sorts and compacts unique values into the existing source prefix before snapshotting and before its pending-error check. It retains source length/count/capacity, does not consume the source, and does not truncate duplicates from the stored vector. The returned table does not alias that storage. Per-character purchase-record errors remain separate.

The entry restriction's cause is UNKNOWN. Per #1310, `vas_errors_entry_rejected` is an explicit host input, default false; rejection returns zero Lua values before reading or modifying the shared list. It is not inferred from taint and emits no invented forbidden event. Later pending errors do not roll back completed source sorting/compaction. Native cleanup and allocation-release lifetime are not established as a Lua getter operation.

The selected original consumer is `VASErrorData_GetCombinedMessage`, `Shared/VASErrorLookup.lua:365-380`, approved by #1310. Native facts are Retail-only; publication on the already-modeled PTR/Forever profiles retains the separately labeled inferred applicability boundary. The three reader selectors passed after their baseline RED runs; existing R88q selectors also passed. Error-event publication, callback copy gates, state-change arity, and GUID parsing were delivered in separate subsequent commits, documented below.

Retail, PTR and Forever use the modeled C API reader, which snapshots the shared `StoreGlueState.purchase_errors` source. Its scoped bootstrap fallbacks remain for unmodeled profiles. **UNKNOWN — Mists:** do not infer native list behavior from the Retail evidence; preserve the current Mists bootstrap path.

## R88w.w4 native shared-list copy gate

VERIFIED Retail (#1308): every per-character path releases/resets the old shared list first (an empty new Vec in this bounded model). The explicit state0/1 removal route now copies precisely a nonempty vector whose first value is nonzero, preserving every later zero and signed32 code. Character matching is irrelevant to copying. Empty and first-zero lists leave it empty; the state>1 service route clears it without copying. The obsolete `copy_eligible` host argument is removed, not replaced with a guessed predicate.

The earlier absent-character assertion that expected no copy solely because a host boolean was false is corrected in this commit: input `[88]` now yields `[88]` even with no character match, exactly as w4 states. Record/event assertions are retained. Purchase-error publication was added in the separate commit documented below.

## R88w.w1/w4 purchase-error event

VERIFIED Retail (#1308): the state0/1 route clears/replaces the separate shared list using the w4 nonempty/first-nonzero gate, then emits `STORE_VAS_PURCHASE_ERROR` with exactly zero event-specific values when its callback is present. This occurs before looking up/removing a character record. A lookup miss therefore still copies and notifies; it does not emit a later removal state-change. Error and complete events both have zero payload, not codes or a GUID. The callback-present flag is explicit host input, default absent.

The approved #1310 RED fixture adapter could install only the existing callbacks; GREEN adds the real purchase-error callback flag. The producer is never mocked. Frozen tests exercise two different backed records and a lookup miss before checking state/order/zero arity.

## R88w.w3 state-change notifications

VERIFIED Retail (#1308): bulk replacement emits once after replacing the complete list, including empty replacement. State0/1 removal emits only after a matching record was removed. A state>1 update/addition emits only after the explicit native-predicate result permits it and the record is stored. All three `VAS_CHARACTER_STATE_CHANGED` sites now publish zero event-specific values. The obsolete host payload slices are removed from the host API.

Per #1310, RED exercised the real former producer with its existing nonempty `[77]` host slice. GREEN removes only that obsolete adapter argument; the frozen record inputs, original-consumer/listener observations and assertions remain byte-identical. There is no test-supplied replacement publisher.

## R88w.w2 bounded GUID argument reader

VERIFIED Retail (#1308), with representation/UNKNOWN scope approved #1310: arguments use exact-case `Opaque-` plus a nonempty ASCII decimal suffix for the explicit u32-wrapping alias map. Missing or empty aliases resume ordinary parsing. Arbitrary text aliases have been removed. Ordinary default-ASCII prefix dispatch, bounded wrapping decimal/hex scanners, documented numeric limits, and the quoted Player/Party examples are implemented. Host records and queue requests compare parsed identities; queue response publication retains the host-supplied display string.

The internal identity is symbolic `(kind, form, parsed fields)`, not a claimed native 128-bit packing. Player selector forms remain distinct representations, with no asserted native cross-form equality. Only the verified LMMParty/Party equivalence is introduced. Host inputs must have unique supported parsed identities; duplicate equivalent display-key precedence is not claimed. Unsupported/UNKNOWN input cannot match another UNKNOWN input or an empty record.

UNKNOWN: nondefault locale/non-ASCII classification, embedded-NUL bounds, short matching `WOWGUID_` markers, unresolved sentinel priority/comparison length, separator-skip exceptions, exhausted-input behavior, WorldLayer progression, and unspecified final-stage rules remain neutral and untested. The explicit native empty-string case remains modeled; unresolved exhaustion inside typed field parsing is not asserted. Canonical hyphens and verified numeric limits do not establish a strict native grammar that rejects all other spellings. Only explicitly verified empty forms and definite default-marker mismatches yield the modeled empty identity.

Prior R88q fixtures used arbitrary resolved text aliases while grammar was UNKNOWN. Their logical A/B/absent records now use valid Player strings; numeric argument 17/29 still verifies type acceptance and ignored extras but no longer invents raw-number alias lookup. Backed Opaque lookup uses the exact native prefix and typed keys. These corrections cite R88w.w2, not a weakening to accommodate producer failures.

Five new optional original-consumer selectors independently exercise two distinct backed records plus absence before assertion: ordinary normalization, quoted Player examples, Party/LMMParty, Opaque wrapping/fallback, and bounded limits. Parser unit tests are supplemental coverage, not independent RED proofs. PTR/Forever applicability remains INFERRED under the previously isolated ACTIVE gate.
