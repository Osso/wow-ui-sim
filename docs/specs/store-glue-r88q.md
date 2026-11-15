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

- Explicit empty-by-default resolved-GUID/alias input; no guessed GUID grammar and no claim that every unrecognized/noncanonical string is malformed. R88w.w2 is pending. Purchase states and queue-character records are separate inputs.
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

Both duplicate definitions of `SetDisconnectOnLogout` and `SetVASProductReady` in `runtime_surface_bootstrap.lua` are also skipped on modeled profiles. Unmodeled profiles preserve both earlier override stages; unrelated product/completion-detail compatibility stays unchanged.

## Verification

Planned optional tests drive selected original Glue consumer functions and real XML list templates, not replaced APIs. Each proposed RED selector must independently use at least two distinct backed records plus absence. Runtime validation must bind compiler-artifact hashes before/after each exact runtime following its own profile build. This draft claims no test or gate result; tracker receipts will record actual RED/GREEN results, startup errors and compile gates.

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

The selected original consumer is `VASErrorData_GetCombinedMessage`, `Shared/VASErrorLookup.lua:365-380`, approved by #1310. Native facts are Retail-only; publication on the already-modeled PTR/Forever profiles retains the separately labeled inferred applicability boundary. The three reader selectors passed after their baseline RED runs; existing R88q selectors also passed. Error-event publication, callback copy gates, state-change arity, and GUID parsing are separate subsequent changes.

The modeled profiles retire the unconditional empty-table `runtime_surface_bootstrap.lua` GetVASErrors override. Its scoped legacy branch remains for unmodeled profiles; the later nil-only fallback cannot overwrite this native reader.

## R88w.w4 native shared-list copy gate

VERIFIED Retail (#1308): every per-character path releases/resets the old shared list first (an empty new Vec in this bounded model). The explicit state0/1 removal route now copies precisely a nonempty vector whose first value is nonzero, preserving every later zero and signed32 code. Character matching is irrelevant to copying. Empty and first-zero lists leave it empty; the state>1 service route clears it without copying. The obsolete `copy_eligible` host argument is removed, not replaced with a guessed predicate.

The earlier absent-character assertion that expected no copy solely because a host boolean was false is corrected in this commit: input `[88]` now yields `[88]` even with no character match, exactly as w4 states. Record/event assertions are retained. Purchase-error publication follows separately; this commit does not claim that event delivery is implemented.

## R88w.w1/w4 purchase-error event

VERIFIED Retail (#1308): the state0/1 route clears/replaces the separate shared list using the w4 nonempty/first-nonzero gate, then emits `STORE_VAS_PURCHASE_ERROR` with exactly zero event-specific values when its callback is present. This occurs before looking up/removing a character record. A lookup miss therefore still copies and notifies; it does not emit a later removal state-change. Error and complete events both have zero payload, not codes or a GUID. The callback-present flag is explicit host input, default absent.

The approved #1310 RED fixture adapter could install only the existing callbacks; GREEN adds the real purchase-error callback flag. The producer is never mocked. Frozen tests exercise two different backed records and a lookup miss before checking state/order/zero arity.

## R88w.w3 state-change notifications

VERIFIED Retail (#1308): bulk replacement emits once after replacing the complete list, including empty replacement. State0/1 removal emits only after a matching record was removed. A state>1 update/addition emits only after the explicit native-predicate result permits it and the record is stored. All three `VAS_CHARACTER_STATE_CHANGED` sites now publish zero event-specific values. The obsolete host payload slices are removed from the host API.

Per #1310, RED exercised the real former producer with its existing nonempty `[77]` host slice. GREEN removes only that obsolete adapter argument; the frozen record inputs, original-consumer/listener observations and assertions remain byte-identical. There is no test-supplied replacement publisher.
