# Host-supplied Calendar day-event records

`C_Calendar.GetDayEvent` and `GetNumDayEvents` expose host-owned day records with empty defaults. Passive DTO source: `src/c_api/c_calendar/day_event_record.rs`; storage: `SimState.calendar_day_events`. This bounded model contract is distinct from [numeric event-index lookup](calendar-event-index-info.md). See [Calendar source boundary](../wiki/investigations/patch-8-1-0-api-audit.md#current-retail-calendar-query-contract--2026-10-10) for provenance limits.

Primary identifier reference: upstream wowless `data/products/wow/structures.yaml`, `C_Calendar.CalendarDayEvent`, inspected 2026-10-13: `eventID` and `clubID` are strings. Cached Retail/PTR `Blizzard_APIDocumentationGenerated/CalendarDocumentation.lua` declares the same 21-field inventory via named ID typedefs; `TimeDocumentation.lua` declares six CalendarTime fields. These sources are declarations, not native behavior evidence. Local profile cache files remain external runtime inputs, not tracked implementation.

## What it must do

### Supplied records and lifetime

- [x] Keep day storage empty in normal defaults; only host-supplied records make day queries positive. Do not add a Lua editor, calendar producer, or default day rows.
- [x] Address host rows by exact `(offset_months: i32, month_day: u32)` key and dense Vec order. `GetNumDayEvents` returns the matching length, zero for absent/empty keys. `GetDayEvent` uses 1-based positions, one DTO for a supplied row, one nil on miss. Ordering, representability and one-nil arity are explicit simulator model policies, not native bounds/arity proof; `MayReturnNothing` does not establish arity.
- [x] Return independent outer and start/end tables. Caller mutation must not change host data or other results; replacement and erase must affect later lookups while retained snapshots stay unchanged. Environments must not share day rows.

### Projection inventory

- [x] Project exactly 21 outer fields with `iconTexture` present, exactly 20 when absent; both required CalendarTime tables contain exactly six fields. Preserve concrete values and declared primitive types, including false booleans and fractional declared numbers.
- [x] Preserve `eventID='9007199254740997'` and `clubID='9007199254740995'` byte-for-byte as strings. Never convert these to Calendar event-index fixture IDs.

Outer field inventory:

| Lua fields | Host fields / representation |
| --- | --- |
| eventID, clubID | event_id, club_id: String |
| title, calendarType, sequenceType, modStatus, invitedBy, difficultyName | title, calendar_type, sequence_type, mod_status, invited_by, difficulty_name: String |
| isCustomTitle, dontDisplayBanner, dontDisplayEnd, isLocked | is_custom_title, dont_display_banner, dont_display_end, is_locked: bool |
| eventType, inviteStatus, inviteType | event_type, invite_status, invite_type: i32 |
| iconTexture | icon_texture: Option<u32> |
| difficulty, numSequenceDays | difficulty, num_sequence_days: f64 |
| sequenceIndex | sequence_index: u32 |
| startTime, endTime | start_time, end_time: separate CalendarTimeRecord |

Time field inventory, independently required for each time:

| Lua fields | Host fields / representation |
| --- | --- |
| monthDay, month, weekday | month_day, month, weekday: u32 |
| year, hour, minute | year, hour, minute: f64 |

### Inputs and secrets

- [x] Require numeric month offset/day for both functions, plus numeric position for GetDayEvent; reject missing/nil/wrong types without string coercion. Validate every required slot even for absent keys.
- [x] Nonintegral, nonfinite or nonrepresentable selectors must not alias a represented key/position. Counts zero and rows one nil are model miss policy; this does not assert native date/index bounds.
- [x] Apply AllowedWhenUntainted to authentic VM secret inputs for both functions: permit untainted calls, deny tainted secret calls, authenticate every slot before type validation or lookup. Preserve original wrappers and caller taint. Secret tests require the existing retail-12-1-0 feature and actual VM secret machinery.

### Real cached consumers

- [x] Unchanged cached complete `Mainline/ItemRef.lua` GetCalendarEventLink must consume day.title, reflect host title replacement, and return nil after erase; no mock, extracted replacement, or ID conversion.
- [x] Unchanged cached CalendarUtil ongoing broadcast must consume a separate caller-owned numeric index fixture 910001 at `(0,15,1)`, optional monthDay=15, and supplied day title. Caller event's numeric ID is not the day DTO's string ID. Use real GameTime/LinkUtil/colors/constants/localization dependencies, not a whole Club producer.
- [x] Keep existing numeric event-index map/default fixture policy unchanged; day tests may explicitly insert their own numeric fixture only inside their environment.

## How it works

- [Calendar source boundary](../wiki/investigations/patch-8-1-0-api-audit.md#current-retail-calendar-query-contract--2026-10-10)
- [Event-index contract](calendar-event-index-info.md)
- [C API boundary](../../AGENTS.md#c-api-boundary)

## Implementation inventory

- `src/c_api/c_calendar/day_event_record.rs`: passive owned DTOs.
- `src/c_api/c_calendar/day_event_queries.rs`: strict numeric/secret arguments, exact host lookups and cloned results.
- `src/c_api/c_calendar/day_event_projection.rs`: fresh rooted outer and independent time-table snapshots.
- `SimState.calendar_day_events`: `HashMap<(i32,u32),Vec<CalendarDayEventRecord>>`, empty default.
- `src/c_api/c_calendar.rs`: owning Calendar namespace; existing event-index lookup unchanged; day queries registered. Only the replaced constant-zero day-count workaround is removed.
- Active profile cache `Blizzard_UIPanels_Game/Mainline/ItemRef.lua` and `Blizzard_FrameXMLUtil/CalendarUtil.lua`: unchanged consumer dependencies.

## Tests asserting this spec

Pre-producer RED executed separately on Retail (`122e272bb`) and PTR (`48902323e`, documentation-only changes over the same source): each 0 PASS / 9 FAIL, actual runtime exit101. Eight cases stop at the missing supplied DTO and one at the positive day count. Downstream DTO/security/snapshot/cached-consumer assertions remain unreached. Each separate no-run compile completed with 741 records, terminal exit0 and six manifest deprecations; compilation is not runtime acceptance. Independent saved-evidence Retail and PTR runtime gates each PASS 20/20 selected cases (9 day producers + 11 numeric-index preservation); each separate normal CLI exits 0 with `CALENDAR_DAY_EMPTY_DEFAULT_INDEX_PRESERVED`, `[]`, CLEAN and zero Lua errors. Normal CLI is not a 21st test:

- `tests/calendar_day_events.rs`: Retail/PTR-only root module, automatically discovered by test harness.
- `tests/calendar_day_events/queries.rs`: five cases for projection/lifetime, exact count/order/defaults, isolation, required types and nonrepresented numeric policy.
- `tests/calendar_day_events/security.rs`: two actual-VM secret cases under retail-12-1-0.
- `tests/calendar_day_events/consumer.rs`: two real-cache consumer cases.
- `tests/calendar_day_events/fixtures.rs`: explicit public passive DTO fixtures and exact Lua value assertions.

Each seeded case first requires a supplied DTO before security/consumer setup. Count case first requires a positive count, then a supplied DTO. Function guard distinguishes missing registration from MissingSuppliedRecord. No case earns behavior credit with only function existence or absence/default assertions.

## Known gaps (current cycle)

- [x] Integrate passive DTO/map and nine tests before the day-query producer; formatting/integration are not executed RED evidence.
- [x] Compile and execute nine targeted cases separately on Retail and PTR before producers; both reached genuine seeded-result/count RED, not setup/admission/compiler failure.
- [x] Implement day-query producers after genuine behavioral RED; implementation is not GREEN evidence.
- [x] Execute producer GREEN on both profiles, including real cached consumers and authentic secrets; preserve existing numeric-index regressions.
- [x] Consumer closure dependencies must load unchanged and real required localization/colors must exist; fail visibly rather than skip or patch them.
- [ ] Complete independent GREEN publication retention/privacy acceptance; public runtime-gate copies are published; their copy gate remains pending.

## Current bounded acceptance — 2026-10-13

Independent Retail and PTR saved-evidence gates cover the eleven requirements above: exact DTO inventories and types, owned defaults/lookup/order, snapshots/replacement/erase/isolation, strict selectors, authentic VM secrets, unchanged complete cached ItemRef and CalendarUtil consumers, and numeric-index regressions. Producer source association is `396f0c9a6`; saved metadata/stat comparisons are not renewed byte authentication or native provenance.

Default lib/bins check PASS: 733 JSON records, build-finished success=true, terminal exited/0, zero source compiler-message records. Six manifest deprecations remain, not warning-free. Independent source/spec/readability inspection identified no concrete in-scope findings; existing fmt --check exit0 is reused for unchanged tracked Rust. These gates are distinct from runtime acceptance.

Public raw [Retail GREEN packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-day-host-records-2026-10-13/green-retail/README.md) and [PTR GREEN packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-day-host-records-2026-10-13/green-ptr/README.md) are present, including Retail's force-added formatter log. Independent GREEN retention/privacy gate PASS (108/108 checks, 76 complete texts). Complete additive [runtime/check gates](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-day-host-records-2026-10-13/gates-current/README.md) have independent copy-retention PASS (43/43 checks, 19 complete texts). Final four-copy frontier has corrected bounded PASS (4/4), reusing 49 valid historical checks and rejecting the unsupported nine-link premise; all eleven owner links resolve. Original FAIL artifacts remain unchanged. The public frontier is 141 complete selected texts. Packet pending-at-selection runtime wording is historical, not the current independent runtime result.

Both runtimes disabled CASC/sound and third-party addons/SavedVariables; existing WTF EditMode cache was read-only and still read. External profile caches/libraries were not authenticated. Native bounds/date/order/arity, lockdown output secrecy, full Calendar/Club lifecycle, render/native CASC, all-profile coverage and full-parent acceptance remain unproved. Historical RED and admission-blocked zero-launch attempts are separate from GREEN.

## Out of scope

Native validation/bounds/return arity, CalendarID conversions, native date/order parity, whole Calendar/Club event producer, normal day seeding, selected/open-event/edit/invitation lifecycle, index-policy changes, native-client observation and broad startup acceptance.

`SecretInChatMessagingLockdown` output secrecy has no established backing model here: UNPROVED, not tested or claimed. Only unlocked host-data output is covered by this bounded contract.
