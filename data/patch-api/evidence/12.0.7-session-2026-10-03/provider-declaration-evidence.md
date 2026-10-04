## source-context-001
SOURCE Patch 12.0.7/API changes - Warcraft Wiki source snapshot from https://warcraft.wiki.gg/wiki/Patch_12.0.7/API_changes (searched 2026-07-06).

PROVIDER


DECLARATION


## source-context-003
SOURCE Resources: TOC 120007. Previous patch 12.0.5. Next patch 12.1.0.

PROVIDER


DECLARATION


## source-context-004
SOURCE Undocumented: Removed IMPORTANT from AuraFilters.

PROVIDER
src/lua_api/globals/auras.rs:156
154: }
155: 
156: fn ensure_aura_filters(state: &mut LuaState, aura_util: Val) {
157:     if matches!(table_get(state, aura_util, "AuraFilters"), Val::Table(_)) {
158:         return;
159:     }
160: 
161:     let filters = create_table_with_capacity(state, AURA_FILTER_COUNT);
162:     let helpful = create_string(state, "HELPFUL");
163:     let harmful = create_string(state, "HARMFUL");
164:     let raid = create_string(state, "RAID");
165:     let include_nameplate_only = create_string(state, "INCLUDE_NAME_PLATE_ONLY");
166:     table_set(state, filters, "Helpful", helpful);
167:     table_set(state, filters, "Harmful", harmful);
168:     table_set(state, filters, "Raid", raid);
169:     table_set(
170:         state,
171:         filters,
172:         "IncludeNameplateOnly",
173:         include_nameplate_only,
174:     );
175:     table_set(state, aura_util, "AuraFilters", filters);
176: }

DECLARATION


## source-context-006
SOURCE Blue posts / notes:

PROVIDER


DECLARATION


## prose-undated-007
SOURCE - Added GameTooltip_AddMoneyLine API using embedded atlases/MoneyFormatter; Blizzard removed usages of SetTooltipMoney.

PROVIDER
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GameTooltip/Mainline/GameTooltip.lua:320
318: end
319: 
320: function GameTooltip_AddMoneyLine(self, rawCopper, useRedLineColor)
321: 	local color = useRedLineColor and RED_FONT_COLOR or HIGHLIGHT_FONT_COLOR;
322: 	GameTooltip_AddColoredMoneyLine(self, rawCopper, color);
323: end
324: 
325: function GameTooltip_OnTooltipAddMoney(self, cost, maxcost)
326: 	if( not maxcost or maxcost < 1 ) then --We just have 1 price to display
327: 		GameTooltip_AddHighlightLine(self, string.format("%s: %s", SELL_PRICE, MoneyFormatterUtil.FormatMoney(cost, GameTooltipMoneyFormat)));
328: 	else
329: 		GameTooltip_AddColoredLine(self, ("%s:"):format(SELL_PRICE), HIGHLIGHT_FONT_COLOR);
330: 		local indent = string.rep(" ",4)
331: 		GameTooltip_AddHighlightLine(self, string.format("%s%s: %s", indent, MINIMUM, MoneyFormatterUtil.FormatMoney(cost, GameTooltipMoneyFormat)));
332: 		GameTooltip_AddHighlightLine(self, string.format("%s%s: %s", indent, MAXIMUM, MoneyFormatterUtil.FormatMoney(maxcost, GameTooltipMoneyFormat)));
333: 	end
334: end
335: 
336: GAME_TOOLTIP_BACKDROP_STYLE_DEFAULT_DARK = {
337: 	layoutType = "TooltipDefaultDarkLayout",
338: };
339: 
340: GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM = {

DECLARATION


## prose-undated-008
SOURCE - Unit identity: APIs restricting unit token types (UnitGUID, UnitAura, health/power APIs when called with PvP-restricted tokens) no longer raise Lua errors for unsupported tokens; return nil/default.

PROVIDER
src/lua_api/globals/unit_misc.rs:144
142: }
143: 
144: fn unit_guid(state: &mut LuaState) -> LuaResult<u32> {
145:     let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
146:     let guid = {
147:         let Ok(sim) = borrow_state(state) else {
148:             return Ok(0);
149:         };
150:         existing_guid_for_unit(&sim, &unit)
151:     };
152:     #[cfg(all(
153:         feature = "retail-12-0-5",
154:         any(feature = "profile-retail", feature = "client-ptr")
155:     ))]
156:     let secret = unit_identity_is_secret(state, &unit)?;
157:     #[cfg(not(all(
158:         feature = "retail-12-0-5",
159:         any(feature = "profile-retail", feature = "client-ptr")
160:     )))]
161:     let secret = false;
162:     let result = guid
163:         .map(|guid| identity_output(state, &guid, secret))
164:         .unwrap_or(Val::Nil);

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1213
1211: 		},
1212: 		{
1213: 			Name = "UnitGUID",
1214: 			Type = "Function",
1215: 			SecretWhenUnitIdentityRestricted = true,
1216: 			SecretArguments = "AllowedWhenUntainted",
1217: 
1218: 			Arguments =
1219: 			{
1220: 				{ Name = "unit", Type = "UnitTokenPvPRestrictedForAddOns", Nilable = false },
1221: 			},
1222: 
1223: 			Returns =
1224: 			{
1225: 				{ Name = "result", Type = "WOWGUID", Nilable = true },
1226: 			},
1227: 		},
1228: 		{
1229: 			Name = "UnitGetDetailedHealPrediction",
1230: 			Type = "Function",
1231: 			SecretArguments = "AllowedWhenUntainted",
1232: 
1233: 			Arguments =
1234: 			{
1235: 				{ Name = "unit", Type = "UnitTokenPvPRestrictedForAddOns", Nilable = false },
1236: 				{ Name = "healerUnit", Type = "UnitTokenPvPRestrictedForAddOns", Nilable = true, Documentation = { "If specified, a unit to evaluate as the 'healer' for incoming heal values. If nil, healer values will be zero." } },
1237: 				{ Name = "healPredictionCalculator", Type = "UnitHealPredictionCalculator", Nilable = false },
1238: 			},
1239: 		},

## prose-undated-009
SOURCE - ENCOUNTER_END includes additional payload: list of EncounterUnitStatus tables for all boss units engaged; fields creatureID, creatureName, remainingHealthPercent as non-secrets.

PROVIDER
src/lua_api/globals/admin_encounter.rs:20
18: // ── Encounter ─────────────────────────────────────────────────────────────────
19: 
20: pub(super) fn simulate_boss_kill(state: &mut LuaState) -> LuaResult<u32> {
21:     let encounter_id = i32::from_stack(state, 1)?;
22:     let name = String::from_stack(state, 2)?;
23:     let difficulty_id = i32::from_stack(state, 3)?;
24:     let group_size = i32::from_stack(state, 4)?;
25:     let name_val = crate::lua_api::methods::create_string(state, &name);
26:     let mut payload = vec![
27:         Val::Num(encounter_id as f64),
28:         name_val,
29:         Val::Num(difficulty_id as f64),
30:         Val::Num(group_size as f64),
31:         Val::Num(1.0),
32:     ];
33:     if cfg!(feature = "retail-12-0-7") {
34:         payload.push(unit_status::copy_status_input(state)?);
35:     }
36:     dispatch_encounter_end(state, &payload)?;
37:     let name_val = crate::lua_api::methods::create_string(state, &name);
38:     dispatch_event_now(
39:         state,
40:         "BOSS_KILL",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterInfoDocumentation.lua:40
38: 			Name = "EncounterEnd",
39: 			Type = "Event",
40: 			LiteralName = "ENCOUNTER_END",
41: 			SynchronousEvent = true,
42: 			Payload =
43: 			{
44: 				{ Name = "encounterID", Type = "number", Nilable = false },
45: 				{ Name = "encounterName", Type = "cstring", Nilable = false },
46: 				{ Name = "difficultyID", Type = "number", Nilable = false },
47: 				{ Name = "groupSize", Type = "number", Nilable = false },
48: 				{ Name = "success", Type = "number", Nilable = false },
49: 				{ Name = "encounterUnitStatus", Type = "table", InnerType = "EncounterUnitStatus", Nilable = false, Documentation = { "List of all boss units engaged during this encounter." } },
50: 			},
51: 		},
52: 		{
53: 			Name = "EncounterStart",
54: 			Type = "Event",
55: 			LiteralName = "ENCOUNTER_START",
56: 			SynchronousEvent = true,
57: 			Payload =
58: 			{
59: 				{ Name = "encounterID", Type = "number", Nilable = false },
60: 				{ Name = "encounterName", Type = "cstring", Nilable = false },
61: 				{ Name = "difficultyID", Type = "number", Nilable = false },
62: 				{ Name = "groupSize", Type = "number", Nilable = false },
63: 			},
64: 		},
65: 		{
66: 			Name = "InstanceLockStart",

## prose-undated-010
SOURCE - C_EncounterEvents allows configuration of different colors for text warnings/timeline events; custom color for 5 seconds remaining signaled via ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED; color APIs accept alpha.

PROVIDER
src/lua_api/globals/missing_surface/encounter_events.rs:112
110: }
111: 
112: fn get_event_color(state: &mut LuaState) -> LuaResult<u32> {
113:     let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
114:         return Ok(0);
115:     };
116:     match copy_event_color(state, event_id) {
117:         Some(color) => {
118:             state.push(color);
119:             Ok(1)
120:         }
121:         None => Ok(0),
122:     }
123: }
124: 
125: fn set_event_color(state: &mut LuaState) -> LuaResult<u32> {
126:     let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
127:         return Ok(0);
128:     };
129:     if !is_known_event_id(event_id) {
130:         return Ok(0);
131:     }
132: 

src/lua_api/globals/missing_surface/encounter_events.rs:125
123: }
124: 
125: fn set_event_color(state: &mut LuaState) -> LuaResult<u32> {
126:     let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
127:         return Ok(0);
128:     };
129:     if !is_known_event_id(event_id) {
130:         return Ok(0);
131:     }
132: 
133:     let color = match stack_val(state, 2) {
134:         Val::Nil => None,
135:         value @ Val::Table(_) => Some(copy_color_table(state, value)),
136:         _ => None,
137:     };
138:     let event_state = ensure_encounter_events_state(state);
139:     let colors = table_get(state, event_state, COLORS_KEY);
140:     match color {
141:         Some(color_table) => set_num_key(state, colors, event_id as f64, color_table),
142:         None => set_num_key(state, colors, event_id as f64, Val::Nil),
143:     }
144:     Ok(0)
145: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterEventsDocumentation.lua:11
9: 	{
10: 		{
11: 			Name = "GetEventColor",
12: 			Type = "Function",
13: 			SecretArguments = "AllowedWhenUntainted",
14: 			Documentation = { "Returns any custom color override applied for an encounter event." },
15: 
16: 			Arguments =
17: 			{
18: 				{ Name = "encounterEventID", Type = "number", Nilable = false },
19: 				{ Name = "trigger", Type = "EncounterEventColorTrigger", Nilable = false },
20: 			},
21: 
22: 			Returns =
23: 			{
24: 				{ Name = "color", Type = "colorRGBA", Mixin = "ColorMixin", Nilable = true },
25: 			},
26: 		},
27: 		{
28: 			Name = "GetEventInfo",
29: 			Type = "Function",
30: 			MayReturnNothing = true,
31: 			SecretArguments = "AllowedWhenUntainted",
32: 			Documentation = { "Returns information about an encounter event." },
33: 
34: 			Arguments =
35: 			{
36: 				{ Name = "encounterEventID", Type = "number", Nilable = false },
37: 			},

/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterEventsDocumentation.lua:106
104: 		},
105: 		{
106: 			Name = "SetEventColor",
107: 			Type = "Function",
108: 			SecretArguments = "NotAllowed",
109: 			Documentation = { "Sets a custom color override for an encounter event. This can be used to colorize text or timer bars individually." },
110: 
111: 			Arguments =
112: 			{
113: 				{ Name = "encounterEventID", Type = "number", Nilable = false },
114: 				{ Name = "trigger", Type = "EncounterEventColorTrigger", Nilable = false },
115: 				{ Name = "color", Type = "colorRGBA", Mixin = "ColorMixin", Nilable = true },
116: 			},
117: 		},
118: 		{
119: 			Name = "SetEventSound",
120: 			Type = "Function",
121: 			SecretArguments = "NotAllowed",
122: 			Documentation = { "Sets a custom sound file to be played when an encounter event trigger occurs." },
123: 
124: 			Arguments =
125: 			{
126: 				{ Name = "encounterEventID", Type = "number", Nilable = false },
127: 				{ Name = "trigger", Type = "EncounterEventSoundTrigger", Nilable = false },
128: 				{ Name = "sound", Type = "EncounterEventSoundInfo", Nilable = true },
129: 			},
130: 		},
131: 	},
132: 

## prose-undated-011
SOURCE - Added C_UIFileAsset namespace: GetFileID(asset), IsKnownFile(asset), IsLooseFile(asset).

PROVIDER
src/c_api/c_ui_file_asset.rs:55
53: }
54: 
55: fn query_asset(state: &LuaState) -> Asset {
56:     if file_id_from_asset_arg(state).is_some() {
57:         return Asset::Shipped;
58:     }
59:     let Some(path) = val_to_string(state, stack_val(state, 1)) else {
60:         return Asset::Missing;
61:     };
62:     if query_selected_addon_file(state, &path) {
63:         Asset::Loose
64:     } else {
65:         Asset::Missing
66:     }
67: }
68: 
69: fn file_id_from_asset_arg(state: &LuaState) -> Option<u32> {
70:     if let Some(file_id) = numeric_file_id_arg(state) {
71:         return Some(file_id);
72:     }
73:     let path = val_to_string(state, stack_val(state, 1))?;
74:     crate::limited_listfile::lookup_path(&path)
75: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:27
25: 		},
26: 		{
27: 			Name = "IsKnownFile",
28: 			Type = "Function",
29: 			SecretArguments = "AllowedWhenUntainted",
30: 			Documentation = { "Determines whether a file asset is known to the client, either as a shipped asset or a locally existing loose file." },
31: 
32: 			Arguments =
33: 			{
34: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
35: 			},
36: 
37: 			Returns =
38: 			{
39: 				{ Name = "isValid", Type = "bool", Nilable = false, Documentation = { "True if the asset is shipped with the client or refers to a known loose file. Existence or openability of loose files is not verified." } },
40: 			},
41: 		},
42: 		{
43: 			Name = "IsLooseFile",
44: 			Type = "Function",
45: 			SecretArguments = "AllowedWhenUntainted",
46: 			Documentation = { "Determines whether a file asset refers to a known loose (local) file." },
47: 
48: 			Arguments =
49: 			{
50: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
51: 			},
52: 
53: 			Returns =

## prose-undated-012
SOURCE - Profiling APIs available to addons again: GetEventCPUUsage, GetFunctionCPUUsage, GetScriptCPUUsage.

PROVIDER
src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50
48: 
49: if GetEventCPUUsage == nil then
50:   function GetEventCPUUsage(_event)
51:     return 0
52:   end
53: end
54: 
55: if GetFunctionCPUUsage == nil then
56:   function GetFunctionCPUUsage(_fn, _includeSubroutines)
57:     return 0, 0
58:   end
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40
38: 		},
39: 		{
40: 			Name = "GetEventCPUUsage",
41: 			Type = "Function",
42: 
43: 			Returns =
44: 			{
45: 				{ Name = "call_time", Type = "number", Nilable = false },
46: 				{ Name = "call_count", Type = "number", Nilable = false },
47: 			},
48: 		},
49: 		{
50: 			Name = "GetFrameCPUUsage",
51: 			Type = "Function",
52: 			SecretArguments = "AllowedWhenUntainted",
53: 
54: 			Arguments =
55: 			{
56: 				{ Name = "frame", Type = "SimpleFrame", Nilable = false },
57: 				{ Name = "includeChildren", Type = "bool", Nilable = false, Default = false },
58: 			},
59: 
60: 			Returns =
61: 			{
62: 				{ Name = "call_time", Type = "number", Nilable = false },
63: 				{ Name = "call_count", Type = "number", Nilable = false },
64: 			},
65: 		},
66: 		{

## prose-undated-013
SOURCE - SimulateMouse APIs no longer carry taint when used, restricted to gamepad action and disallowed with forbidden/locked/script-inaccessible/protected mouse foci in combat.

PROVIDER


DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:266
264: 		},
265: 		{
266: 			Name = "SimulateMouseClick",
267: 			Type = "Function",
268: 			RequiresLimitedInput = true,
269: 			MouseFocusValidForLimitedInput = true,
270: 			SecretArguments = "AllowedWhenUntainted",
271: 			Documentation = { "Effectively the same as SimulateMouseDown plus SimulateMouseUp and consumes limited input for both." },
272: 
273: 			Arguments =
274: 			{
275: 				{ Name = "button", Type = "mouseButton", Nilable = false },
276: 			},
277: 		},
278: 		{
279: 			Name = "SimulateMouseDown",
280: 			Type = "Function",
281: 			RequiresLimitedInput = true,
282: 			MouseFocusValidForLimitedInput = true,
283: 			SecretArguments = "AllowedWhenUntainted",
284: 			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },
285: 
286: 			Arguments =
287: 			{
288: 				{ Name = "button", Type = "mouseButton", Nilable = false },
289: 			},
290: 		},
291: 		{
292: 			Name = "SimulateMouseUp",

/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:279
277: 		},
278: 		{
279: 			Name = "SimulateMouseDown",
280: 			Type = "Function",
281: 			RequiresLimitedInput = true,
282: 			MouseFocusValidForLimitedInput = true,
283: 			SecretArguments = "AllowedWhenUntainted",
284: 			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },
285: 
286: 			Arguments =
287: 			{
288: 				{ Name = "button", Type = "mouseButton", Nilable = false },
289: 			},
290: 		},
291: 		{
292: 			Name = "SimulateMouseUp",
293: 			Type = "Function",
294: 			RequiresLimitedInput = true,
295: 			MouseFocusValidForLimitedInput = true,
296: 			SecretArguments = "AllowedWhenUntainted",
297: 			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },
298: 
299: 			Arguments =
300: 			{
301: 				{ Name = "button", Type = "mouseButton", Nilable = false },
302: 			},
303: 		},
304: 		{
305: 			Name = "SimulateMouseWheel",

## prose-undated-014
SOURCE - debugstack and debuglocals return secret values if current function or caller accessed secret value.

PROVIDER
src/lua_api/workarounds/temporary/debug_environment_defaults.rs:121
119:   end
120: 
121:   function debugstack(level, count1, count2)
122:     if not debug or not debug.getinfo then
123:       return ""
124:     end
125:     local start = (tonumber(level) or 1) + 1
126:     local lines = {}
127:     local depth = start
128:     while true do
129:       local info = debug.getinfo(depth, "Sln")
130:       if not info then break end
131:       lines[#lines + 1] = debugstack_line(info)
132:       depth = depth + 1
133:     end
134: 
135:     if count1 or count2 then
136:       local top = tonumber(count1) or 12
137:       local bottom = tonumber(count2) or 10
138:       if #lines > top + bottom then
139:         local kept = {}
140:         for i = 1, top do kept[#kept + 1] = lines[i] end
141:         kept[#kept + 1] = "..."

src/lua_api/workarounds/temporary/debug_environment_defaults.rs:153
151: 
152: if debuglocals == nil then
153:   function debuglocals(level)
154:     if not debug or not debug.getinfo or not debug.getlocal then
155:       return ""
156:     end
157:     local start = (tonumber(level) or 1) + 1
158:     local info = debug.getinfo(start, "fS")
159:     if not info then return "" end
160:     local parts = {}
161:     local i = 1
162:     while true do
163:       local name, value = debug.getlocal(start, i)
164:       if not name then break end
165:       if not name:match("^%(") then
166:         parts[#parts + 1] = string.format("%s = %s", name, tostring(value))
167:       end
168:       i = i + 1
169:     end
170:     return table.concat(parts, "\n")
171:   end
172: end
173: 

DECLARATION


## prose-undated-015
SOURCE - Added secure action raidtarget option "set-unmarked" and /tm ~marker syntax.

PROVIDER
src/lua_api/globals/spell_macro_verbs.rs:189
187: /// Bounded numeric /tm dispatch; shares condition parsing and marker state.
188: #[cfg(feature = "retail-12-0-5")]
189: fn run_target_marker_command(state: &mut LuaState, argument: &str) -> LuaResult<()> {
190:     let selected = {
191:         let sim = borrow_state(state)?;
192:         super::security::resolve_cmd_option_with_unit(argument, &sim)
193:             .map(|(marker, unit)| (marker.to_owned(), unit.to_owned()))
194:     };
195:     let Some((marker, unit)) = selected else {
196:         return Ok(());
197:     };
198:     // INFERRED: invalid/out-of-range numeric input is an atomic no-op.
199:     // Native numeric coercion and !/~ marker prefixes are not modeled here.
200:     let Ok(marker) = marker.parse::<u8>() else {
201:         return Ok(());
202:     };
203:     if marker > 8 {
204:         return Ok(());
205:     }
206:     let function = LuaApiMut::get_global_val(state, "SetRaidTarget");
207:     let unit = create_string(state, &unit);
208:     call_function_state(state, function, &[unit, Val::Num(f64::from(marker))])?;
209:     Ok(())

DECLARATION


## prose-undated-016
SOURCE - C_MythicPlus.GetRunHistory, GetWeeklyBestForMap, GetSeasonBestForMap now return CalendarTime structs instead of MythicPlusDate structs.

PROVIDER
src/lua_api/globals/missing_surface/mythic_plus.rs:266
264: }
265: 
266: fn get_run_history(state: &mut LuaState) -> LuaResult<u32> {
267:     // Args: includePreviousWeeks, includeIncompleteRuns, currentSeasonOnly
268:     // We ignore the filter args and return all seeded runs.
269:     let runs = borrow_state(state)?.mythic_plus.run_history.clone();
270:     let array = create_table(state);
271:     for (i, run) in runs.into_iter().enumerate() {
272:         let entry = create_table(state);
273:         table_set(
274:             state,
275:             entry,
276:             "mapChallengeModeID",
277:             Val::Num(run.map_challenge_mode_id as f64),
278:         );
279:         table_set(state, entry, "level", Val::Num(run.level as f64));
280:         table_set(state, entry, "completed", Val::Bool(run.completed));
281:         table_set(state, entry, "season", Val::Num(run.season as f64));
282:         table_set(state, entry, "runScore", Val::Num(run.run_score));
283:         table_set(state, entry, "thisWeek", Val::Bool(run.this_week));
284:         table_set(
285:             state,
286:             entry,

src/lua_api/globals/missing_surface/mythic_plus.rs:325
323: }
324: 
325: fn get_weekly_best_for_map(state: &mut LuaState) -> LuaResult<u32> {
326:     let map_id = i32::from_stack(state, 1)?;
327:     let best: Option<MythicPlusWeeklyBest> = borrow_state(state)?
328:         .mythic_plus
329:         .weekly_best_per_map
330:         .get(&map_id)
331:         .cloned();
332:     let Some(b) = best else {
333:         // mayreturnnothing: no data for this map.
334:         return Ok(0);
335:     };
336:     state.push(Val::Num(b.duration_sec as f64));
337:     state.push(Val::Num(b.level as f64));
338:     state.push(Val::Nil); // completionDate nilable
339:     state.push(Val::Num(0.0)); // fraction
340:     let affix_ids = create_table(state);
341:     state.push(affix_ids);
342:     let members = create_table(state);
343:     state.push(members);
344:     Ok(6)
345: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MythicPlusInfoDocumentation.lua:137
135: 		},
136: 		{
137: 			Name = "GetRunHistory",
138: 			Type = "Function",
139: 			SecretArguments = "AllowedWhenUntainted",
140: 
141: 			Arguments =
142: 			{
143: 				{ Name = "includePreviousWeeks", Type = "bool", Nilable = false, Default = false },
144: 				{ Name = "includeIncompleteRuns", Type = "bool", Nilable = false, Default = false },
145: 				{ Name = "currentSeasonOnly", Type = "bool", Nilable = false, Default = false },
146: 			},
147: 
148: 			Returns =
149: 			{
150: 				{ Name = "runs", Type = "table", InnerType = "MythicPlusRunInfo", Nilable = false },
151: 			},
152: 		},
153: 		{
154: 			Name = "GetSeasonBestAffixScoreInfoForMap",
155: 			Type = "Function",
156: 			MayReturnNothing = true,
157: 			SecretArguments = "AllowedWhenUntainted",
158: 			Documentation = { "Gets the active players best runs by the seasonal tracked affixes as well as their overall score for the current season." },
159: 
160: 			Arguments =
161: 			{
162: 				{ Name = "mapChallengeModeID", Type = "number", Nilable = false },
163: 			},

/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TimeDocumentation.lua:6
4: 	{
5: 		{
6: 			Name = "CalendarTime",
7: 			Type = "Structure",
8: 			Fields =
9: 			{
10: 				{ Name = "monthDay", Type = "luaIndex", Nilable = false },
11: 				{ Name = "month", Type = "luaIndex", Nilable = false },
12: 				{ Name = "weekday", Type = "luaIndex", Nilable = false },
13: 				{ Name = "year", Type = "number", Nilable = false },
14: 				{ Name = "hour", Type = "number", Nilable = false },
15: 				{ Name = "minute", Type = "number", Nilable = false },
16: 			},
17: 		},
18: 	},
19: 	Predicates =
20: 	{
21: 	},
22: };
23: 
24: APIDocumentation:AddDocumentationTable(Time);

## prose-undated-017
SOURCE - GROUP_FORMED sent when player joins a follower dungeon or delve alone.

PROVIDER
src/event/valid_events_a.rs:718
716:     "GOSSIP_OPTIONS_REFRESHED",
717:     "GOSSIP_SHOW",
718:     "GROUP_FORMED",
719:     "GROUP_INVITE_CONFIRMATION",
720:     "GROUP_JOINED",
721:     "GROUP_LEFT",
722:     "GROUP_ROSTER_UPDATE",
723:     "GUILDBANKBAGSLOTS_CHANGED",
724:     "GUILDBANKFRAME_CLOSED",
725:     "GUILDBANKFRAME_OPENED",
726:     "GUILDBANKLOG_UPDATE",
727:     "GUILDBANK_ITEM_LOCK_CHANGED",
728:     "GUILDBANK_TEXT_CHANGED",
729:     "GUILDBANK_UPDATE_MONEY",
730:     "GUILDBANK_UPDATE_TABS",
731:     "GUILDBANK_UPDATE_TEXT",
732:     "GUILDBANK_UPDATE_WITHDRAWMONEY",
733:     "GUILDTABARD_UPDATE",
734: ];

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:641
639: 			Name = "GroupFormed",
640: 			Type = "Event",
641: 			LiteralName = "GROUP_FORMED",
642: 			SynchronousEvent = true,
643: 			Payload =
644: 			{
645: 				{ Name = "category", Type = "number", Nilable = false },
646: 				{ Name = "partyGUID", Type = "WOWGUID", Nilable = false },
647: 			},
648: 		},
649: 		{
650: 			Name = "GroupInviteConfirmation",
651: 			Type = "Event",
652: 			LiteralName = "GROUP_INVITE_CONFIRMATION",
653: 			SynchronousEvent = true,
654: 		},
655: 		{
656: 			Name = "GroupJoined",
657: 			Type = "Event",
658: 			LiteralName = "GROUP_JOINED",
659: 			SynchronousEvent = true,
660: 			Payload =
661: 			{
662: 				{ Name = "category", Type = "number", Nilable = false },
663: 				{ Name = "partyGUID", Type = "WOWGUID", Nilable = false },
664: 			},
665: 		},
666: 		{
667: 			Name = "GroupLeft",

## prose-undated-018
SOURCE - AuraData.isFromPlayerOrPlayerPet true if aura came from player-controlled vehicle.

PROVIDER
src/lua_api/globals/auras.rs:817
815:         state,
816:         t,
817:         "isFromPlayerOrPlayerPet",
818:         Val::Bool(aura.is_from_player_or_player_pet),
819:     );
820:     table_set(state, t, "nameplateShowAll", Val::Bool(false));
821:     table_set(state, t, "isHelpful", Val::Bool(aura.is_helpful));
822:     table_set(state, t, "isHarmful", Val::Bool(!aura.is_helpful));
823:     table_set(
824:         state,
825:         t,
826:         "isNameplateOnly",
827:         Val::Bool(aura.is_nameplate_only),
828:     );
829:     table_set(state, t, "isRaid", Val::Bool(aura.is_raid));
830: }
831: 
832: #[cfg(test)]
833: mod tests {
834:     use super::*;
835: 
836:     fn plain_helpful_aura() -> AuraInfo {
837:         AuraInfo {

DECLARATION


## prose-undated-019
SOURCE - Addons allowed to call C_UnitAuras.AddPrivateAuraAppliedSound during active M+ if player not in combat.

PROVIDER
src/c_api/private_aura_sounds/add.rs:167
165: }
166: 
167: fn check_context(state: &LuaState) -> LuaResult<()> {
168:     let sim = borrow_state(state)?;
169:     let restricted = sim.world.encounter_in_progress
170:         || sim.mythic_plus.is_active
171:         || sim.private_aura_sound_registrations.pvp_match_active;
172:     // INFERRED HasRestrictions policy: no insecure registration in these contexts.
173:     if restricted && !rilua::api::state_is_secure(state) {
174:         return Err(input_error(
175:             "insecure registration denied during encounter/M+/PvP match",
176:         ));
177:     }
178:     Ok(())
179: }
180: 
181: fn allocate_id(sounds: &mut PrivateAuraSoundRegistrations) -> Option<u32> {
182:     let mut candidate = sounds.next_id?;
183:     // Each collision corresponds to a stored live ID, not a scan of the u32 domain.
184:     while sounds.live_ids.contains(&candidate) {
185:         sounds.next_id = candidate.checked_add(1);
186:         candidate = sounds.next_id?;
187:     }

DECLARATION


## prose-undated-020
SOURCE - Fixed secret value errors in SetFrameStrata.

PROVIDER
src/lua_api/frame/methods/core_state/strata_level.rs:13
11: use rilua::{LuaResult, Val};
12: 
13: pub fn set_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
14:     let id = frame_id(state, 1)?;
15:     let strata = String::from_stack(state, 2)?;
16:     let Some(strata) = crate::widget::FrameStrata::from_str(&strata) else {
17:         return Ok(0);
18:     };
19:     if !can_change_protected_state_for(state, id) {
20:         emit_addon_action_blocked(state, id, "SetFrameStrata");
21:         return Ok(0);
22:     }
23:     let mut sim = borrow_state_mut(state)?;
24:     let mut strata_changed = false;
25:     if let Some(frame) = sim.widgets.get_mut_visual(id) {
26:         strata_changed |= frame.frame_strata != strata;
27:         frame.frame_strata = strata;
28:         frame.has_fixed_frame_strata = true;
29:     }
30:     let mut queue: Vec<u64> = sim
31:         .widgets
32:         .get(id)
33:         .map(|f| f.children.clone())

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1292
1290: 		},
1291: 		{
1292: 			Name = "SetFrameStrata",
1293: 			Type = "Function",
1294: 			IsProtectedFunction = true,
1295: 			SecretArguments = "NotAllowed",
1296: 
1297: 			Arguments =
1298: 			{
1299: 				{ Name = "strata", Type = "FrameStrata", Nilable = false },
1300: 			},
1301: 		},
1302: 		{
1303: 			Name = "SetHighlightLocked",
1304: 			Type = "Function",
1305: 			SecretArguments = "AllowedWhenUntainted",
1306: 
1307: 			Arguments =
1308: 			{
1309: 				{ Name = "locked", Type = "bool", Nilable = false },
1310: 			},
1311: 		},
1312: 		{
1313: 			Name = "SetHitRectInsets",
1314: 			Type = "Function",
1315: 			IsProtectedFunction = true,
1316: 			SecretArguments = "NotAllowed",
1317: 
1318: 			Arguments =

## prose-undated-021
SOURCE - BNInviteFriend migrated to C_BattleNet.InviteFriend.

PROVIDER
src/c_api/c_battle_net.rs:70
68:     #[cfg(feature = "retail-12-1-0")]
69:     register_patch_12_1_friend_query_methods(state, table_ref)?;
70:     table_set_rust_fn_static(state, table_ref, "InviteFriend", c_bnet_invite_friend)?;
71:     table_set_rust_fn_static(state, table_ref, "GetNumFriends", c_bnet_get_num_friends)?;
72:     table_set_rust_fn_static(
73:         state,
74:         table_ref,
75:         "GetFriendAccountInfo",
76:         c_bnet_get_friend_account_info,
77:     )?;
78:     table_set_rust_fn_static(
79:         state,
80:         table_ref,
81:         "GetAccountInfoByGUID",
82:         c_bnet_get_account_info_by_guid,
83:     )?;
84:     table_set_rust_fn_static(
85:         state,
86:         table_ref,
87:         "GetGameAccountInfoByGUID",
88:         c_bnet_get_game_account_info_by_guid,
89:     )?;
90:     table_set_rust_fn_static(

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:211
209: 		},
210: 		{
211: 			Name = "InviteFriend",
212: 			Type = "Function",
213: 			SecretArguments = "AllowedWhenUntainted",
214: 
215: 			Arguments =
216: 			{
217: 				{ Name = "gameAccountID", Type = "number", Nilable = false },
218: 			},
219: 		},
220: 		{
221: 			Name = "IsBattleNetFriendsListEnabled",
222: 			Type = "Function",
223: 
224: 			Returns =
225: 			{
226: 				{ Name = "isBattleNetFriendsListEnabled", Type = "bool", Nilable = false },
227: 			},
228: 		},
229: 		{
230: 			Name = "IsBattleNetFriendsListSupported",
231: 			Type = "Function",
232: 
233: 			Returns =
234: 			{
235: 				{ Name = "isBattleNetFriendsListSupported", Type = "bool", Nilable = false },
236: 			},
237: 		},

## prose-undated-022
SOURCE - Added DurationTextBinding script object type.

PROVIDER
src/c_api/duration_text_binding.rs:118
116:         return copy
117:     end
118:     local function create_duration_text_binding(duration, fontString)
119:         local configuration = {
120:             duration = duration ~= nil and duration or create_duration_value(0),
121:             fontString = fontString,
122:             enabled = true,
123:             updateInterval = 1,
124:             timeModifier = 0,
125:             expiredText = nil,
126:             zeroDurationText = nil,
127:             formatter = nil,
128:             textFormat = nil,
129:             textFormatComponents = nil,
130:             clock = create_duration_clock(0),
131:         }
132:         -- Native handles survive securecopy(options); configuration tables do not.
133:         local binding = newproxy(true)
134:         local metatable = getmetatable(binding)
135:         local schedule = {dirty = true, elapsed = 0}
136:         metatable.__index = configuration
137:         metatable.__newindex = function(_, key, value)
138:             configuration[key] = value

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21
19: 		},
20: 		{
21: 			Name = "CreateDurationTextBinding",
22: 			Type = "Function",
23: 			Documentation = { "Creates a duration text binding, which automatically updates a font string with formatted text derived from a duration object." },
24: 
25: 			Returns =
26: 			{
27: 				{ Name = "binding", Type = "DurationTextBinding", Nilable = false },
28: 			},
29: 		},
30: 		{
31: 			Name = "CreateManualClock",
32: 			Type = "Function",
33: 			Documentation = { "Creates a manually driven time source for use with duration objects." },
34: 
35: 			Returns =
36: 			{
37: 				{ Name = "clock", Type = "LuaDurationManualClock", Nilable = false },
38: 			},
39: 		},
40: 	},
41: 
42: 	Events =
43: 	{
44: 	},
45: 
46: 	Tables =
47: 	{

## source-context-024
SOURCE Consolidated changes 12.0.5 (67602) -> 12.0.7 (68182) Jun 12 2026.

PROVIDER


DECLARATION


## source-context-026
SOURCE Global API Added (35):

PROVIDER


DECLARATION


## global api-C_BattleNet-InviteFriend-027
SOURCE C_BattleNet.InviteFriend

PROVIDER
src/c_api/c_battle_net.rs:70
68:     #[cfg(feature = "retail-12-1-0")]
69:     register_patch_12_1_friend_query_methods(state, table_ref)?;
70:     table_set_rust_fn_static(state, table_ref, "InviteFriend", c_bnet_invite_friend)?;
71:     table_set_rust_fn_static(state, table_ref, "GetNumFriends", c_bnet_get_num_friends)?;
72:     table_set_rust_fn_static(
73:         state,
74:         table_ref,
75:         "GetFriendAccountInfo",
76:         c_bnet_get_friend_account_info,
77:     )?;
78:     table_set_rust_fn_static(
79:         state,
80:         table_ref,
81:         "GetAccountInfoByGUID",
82:         c_bnet_get_account_info_by_guid,
83:     )?;
84:     table_set_rust_fn_static(
85:         state,
86:         table_ref,
87:         "GetGameAccountInfoByGUID",
88:         c_bnet_get_game_account_info_by_guid,
89:     )?;
90:     table_set_rust_fn_static(

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:211
209: 		},
210: 		{
211: 			Name = "InviteFriend",
212: 			Type = "Function",
213: 			SecretArguments = "AllowedWhenUntainted",
214: 
215: 			Arguments =
216: 			{
217: 				{ Name = "gameAccountID", Type = "number", Nilable = false },
218: 			},
219: 		},
220: 		{
221: 			Name = "IsBattleNetFriendsListEnabled",
222: 			Type = "Function",
223: 
224: 			Returns =
225: 			{
226: 				{ Name = "isBattleNetFriendsListEnabled", Type = "bool", Nilable = false },
227: 			},
228: 		},
229: 		{
230: 			Name = "IsBattleNetFriendsListSupported",
231: 			Type = "Function",
232: 
233: 			Returns =
234: 			{
235: 				{ Name = "isBattleNetFriendsListSupported", Type = "bool", Nilable = false },
236: 			},
237: 		},

## global api-C_Container-CalculateTotalNumberOfFreeBagSlots-028
SOURCE C_Container.CalculateTotalNumberOfFreeBagSlots

PROVIDER
src/c_api/item_spell/c_container.rs:63
61:             ("GetContainerNumFreeSlots", c_container_get_num_free_slots),
62:             (
63:                 "CalculateTotalNumberOfFreeBagSlots",
64:                 c_container_calculate_total_number_of_free_bag_slots,
65:             ),
66:             ("GetContainerFreeSlots", c_container_get_free_slots),
67:             ("HasContainerItem", c_container_has_item),
68:             ("GetBagSlotFlag", c_container_get_bag_slot_flag),
69:             (
70:                 "GetBackpackAutosortDisabled",
71:                 c_container_get_backpack_autosort_disabled,
72:             ),
73:             (
74:                 "GetBackpackSellJunkDisabled",
75:                 c_container_get_backpack_sell_junk_disabled,
76:             ),
77:             ("GetContainerItemInfo", c_container_get_item_info),
78:             ("GetContainerItemCooldown", c_container_get_item_cooldown),
79:             ("GetItemCooldown", c_container_get_item_cooldown),
80:             ("GetContainerItemID", c_container_get_item_id),
81:             ("GetContainerItemLink", c_container_get_item_link),
82:             ("ContainerIDToInventoryID", c_container_id_to_inventory_id),
83:             ("GetBagName", c_container_get_bag_name),

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ContainerDocumentation.lua:11
9: 	{
10: 		{
11: 			Name = "CalculateTotalNumberOfFreeBagSlots",
12: 			Type = "Function",
13: 
14: 			Returns =
15: 			{
16: 				{ Name = "totalFreeSlots", Type = "number", Nilable = false },
17: 			},
18: 		},
19: 		{
20: 			Name = "ContainerIDToInventoryID",
21: 			Type = "Function",
22: 			MayReturnNothing = true,
23: 			SecretArguments = "AllowedWhenUntainted",
24: 
25: 			Arguments =
26: 			{
27: 				{ Name = "containerID", Type = "BagIndex", Nilable = false },
28: 			},
29: 
30: 			Returns =
31: 			{
32: 				{ Name = "inventoryID", Type = "luaIndex", Nilable = false },
33: 			},
34: 		},
35: 		{
36: 			Name = "ContainerRefundItemPurchase",
37: 			Type = "Function",

## global api-C_DelvesUI-GetDelveEntranceTitleString-029
SOURCE C_DelvesUI.GetDelveEntranceTitleString

PROVIDER
src/lua_api/globals/missing_surface/delves_ui.rs:81
79:     ),
80:     (
81:         "GetDelveEntranceTitleString",
82:         get_delve_entrance_title_string,
83:     ),
84:     ("GetDelveEntranceMapID", get_delve_entrance_map_id),
85:     ("GetDelveEntranceTiers", get_delve_entrance_tiers),
86:     ("GetDelvesMinRequiredLevel", get_delves_min_required_level),
87:     ("GetFactionForCompanion", get_faction_for_companion),
88:     ("GetFlavorNodeForCompanion", get_flavor_node_for_companion),
89:     (
90:         "GetFlavorNodeNameForCompanion",
91:         get_flavor_node_name_for_companion,
92:     ),
93:     ("GetPlayerCompanionPDEID", get_player_companion_pde_id),
94:     ("GetRoleNodeForCompanion", get_role_node_for_companion),
95:     ("GetRoleSubtreeForCompanion", get_role_subtree_for_companion),
96:     (
97:         "GetTieredEntranceOptionalAffixTraitTreeID",
98:         get_tiered_entrance_optional_affix_trait_tree_id,
99:     ),
100:     ("GetTieredEntrancePDEID", get_tiered_entrance_pde_id),
101:     ("GetTraitTreeForCompanion", get_trait_tree_for_companion),

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:147
145: 		},
146: 		{
147: 			Name = "GetDelveEntranceTitleString",
148: 			Type = "Function",
149: 
150: 			Returns =
151: 			{
152: 				{ Name = "title", Type = "cstring", Nilable = true },
153: 			},
154: 		},
155: 		{
156: 			Name = "GetDelvesAffixSpellsForSeason",
157: 			Type = "Function",
158: 
159: 			Returns =
160: 			{
161: 				{ Name = "affixSpellIDs", Type = "table", InnerType = "number", Nilable = false },
162: 			},
163: 		},
164: 		{
165: 			Name = "GetDelvesFactionForSeason",
166: 			Type = "Function",
167: 
168: 			Returns =
169: 			{
170: 				{ Name = "factionID", Type = "number", Nilable = false },
171: 			},
172: 		},
173: 		{

## global api-C_DelvesUI-GetWorldTierDifficultyForActivePlayer-030
SOURCE C_DelvesUI.GetWorldTierDifficultyForActivePlayer

PROVIDER
src/lua_api/globals/missing_surface/delves_ui.rs:104
102:     ("GetUnseenCuriosBySlotType", get_unseen_curios_by_slot_type),
103:     (
104:         "GetWorldTierDifficultyForActivePlayer",
105:         get_world_tier_difficulty_for_active_player,
106:     ),
107:     ("HasActiveDelve", has_active_delve),
108:     ("IsDelveEntranceTierEnabled", is_delve_entrance_tier_enabled),
109:     (
110:         "RequestPartyEligibilityForDelveTiers",
111:         request_party_eligibility_for_delve_tiers,
112:     ),
113:     ("SaveSeenCuriosBySlotType", save_seen_curios_by_slot_type),
114:     ("SelectDelveEntranceTier", select_delve_entrance_tier),
115: ];
116: 
117: fn get_active_delve_tier(state: &mut LuaState) -> LuaResult<u32> {
118:     let active_tier = load_active_tier(state);
119:     push_tier_info(state, active_tier);
120:     Ok(1)
121: }
122: 
123: fn get_companion_info_for_active_player(state: &mut LuaState) -> LuaResult<u32> {
124:     state.push(Val::Num(COMPANION_ID));

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:364
362: 		},
363: 		{
364: 			Name = "GetWorldTierDifficultyForActivePlayer",
365: 			Type = "Function",
366: 
367: 			Returns =
368: 			{
369: 				{ Name = "difficulty", Type = "WorldTierDifficulty", Nilable = false },
370: 			},
371: 		},
372: 		{
373: 			Name = "HasActiveDelve",
374: 			Type = "Function",
375: 
376: 			Returns =
377: 			{
378: 				{ Name = "result", Type = "bool", Nilable = false },
379: 			},
380: 		},
381: 		{
382: 			Name = "HasActiveLFGLair",
383: 			Type = "Function",
384: 
385: 			Returns =
386: 			{
387: 				{ Name = "result", Type = "bool", Nilable = false },
388: 			},
389: 		},
390: 		{

## global api-C_DurationUtil-CreateDurationTextBinding-031
SOURCE C_DurationUtil.CreateDurationTextBinding

PROVIDER
src/c_api/duration_text_binding.rs:237
235:         return binding
236:     end
237:     set_default(durationUtil, "CreateDurationTextBinding", create_duration_text_binding)
238: 
239:     -- This closure is retained only in the host registry, not a Lua global.
240:     -- Cadence uses engine elapsed time; duration objects retain their own clocks.
241:     local function update_binding(binding, schedule, elapsed)
242:         if binding:IsEnabled() and binding:CanUpdateFontString() then
243:             schedule.elapsed = schedule.elapsed + elapsed
244:             if schedule.dirty or schedule.elapsed >= binding:GetUpdateInterval() then
245:                 schedule.dirty = false
246:                 schedule.elapsed = 0
247:                 binding:UpdateFontString()
248:             end
249:         end
250:     end
251:     return function(elapsed)
252:         for binding, schedule in nextBinding, bindings do
253:             local ok, errorValue = protectedCall(update_binding, binding, schedule, elapsed)
254:             if not ok then reportUpdateError(errorValue) end
255:         end
256:     end
257: end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21
19: 		},
20: 		{
21: 			Name = "CreateDurationTextBinding",
22: 			Type = "Function",
23: 			Documentation = { "Creates a duration text binding, which automatically updates a font string with formatted text derived from a duration object." },
24: 
25: 			Returns =
26: 			{
27: 				{ Name = "binding", Type = "DurationTextBinding", Nilable = false },
28: 			},
29: 		},
30: 		{
31: 			Name = "CreateManualClock",
32: 			Type = "Function",
33: 			Documentation = { "Creates a manually driven time source for use with duration objects." },
34: 
35: 			Returns =
36: 			{
37: 				{ Name = "clock", Type = "LuaDurationManualClock", Nilable = false },
38: 			},
39: 		},
40: 	},
41: 
42: 	Events =
43: 	{
44: 	},
45: 
46: 	Tables =
47: 	{

## global api-C_DurationUtil-CreateManualClock-032
SOURCE C_DurationUtil.CreateManualClock

PROVIDER
src/lua_api/globals/lua_duration_object.rs:104
102: 
103:     // Install CreateManualClock only if missing.
104:     let existing = crate::lua_api::methods::table_get(state, ns, "CreateManualClock");
105:     if existing == Val::Nil {
106:         let create_clock_fn = make_closure(
107:             state,
108:             "C_DurationUtil.CreateManualClock",
109:             create_manual_clock,
110:         );
111:         table_set_static(state, ns, "CreateManualClock", create_clock_fn);
112:     }
113: 
114:     // Install GetCurrentTime only if missing.
115:     let existing = crate::lua_api::methods::table_get(state, ns, "GetCurrentTime");
116:     if existing == Val::Nil {
117:         let get_time_fn = make_closure(state, "C_DurationUtil.GetCurrentTime", get_current_time);
118:         table_set_static(state, ns, "GetCurrentTime", get_time_fn);
119:     }
120: 
121:     Ok(())
122: }
123: 
124: /// Create a new `LuaDurationObject` table value for callers that expose

src/lua_api/globals/lua_duration_object.rs:111
109:             create_manual_clock,
110:         );
111:         table_set_static(state, ns, "CreateManualClock", create_clock_fn);
112:     }
113: 
114:     // Install GetCurrentTime only if missing.
115:     let existing = crate::lua_api::methods::table_get(state, ns, "GetCurrentTime");
116:     if existing == Val::Nil {
117:         let get_time_fn = make_closure(state, "C_DurationUtil.GetCurrentTime", get_current_time);
118:         table_set_static(state, ns, "GetCurrentTime", get_time_fn);
119:     }
120: 
121:     Ok(())
122: }
123: 
124: /// Create a new `LuaDurationObject` table value for callers that expose
125: /// duration objects through other namespaces such as `C_ActionBar`.
126: pub(crate) fn new_duration_object_value(state: &mut LuaState) -> Val {
127:     ensure_metatable(state);
128:     new_duration_object(state)
129: }
130: 
131: /// Push a duration snapshot using the same validated setter exposed to Lua.

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:31
29: 		},
30: 		{
31: 			Name = "CreateManualClock",
32: 			Type = "Function",
33: 			Documentation = { "Creates a manually driven time source for use with duration objects." },
34: 
35: 			Returns =
36: 			{
37: 				{ Name = "clock", Type = "LuaDurationManualClock", Nilable = false },
38: 			},
39: 		},
40: 	},
41: 
42: 	Events =
43: 	{
44: 	},
45: 
46: 	Tables =
47: 	{
48: 	},
49: 	Predicates =
50: 	{
51: 	},
52: };
53: 
54: APIDocumentation:AddDocumentationTable(DurationUtil);

## global api-C_EncounterTimeline-GetEventColor-033
SOURCE C_EncounterTimeline.GetEventColor

PROVIDER
src/lua_api/globals/missing_surface/encounter_events.rs:24
22:     register_encounter_timeline_surface(state)?;
23:     ensure_encounter_events_state(state);
24:     table_set_rust_fn_static(state, ns, "GetEventColor", get_event_color)?;
25:     table_set_rust_fn_static(state, ns, "GetEventInfo", get_event_info)?;
26:     table_set_rust_fn_static(state, ns, "GetEventList", get_event_list)?;
27:     table_set_rust_fn_static(state, ns, "GetEventSound", get_event_sound)?;
28:     table_set_rust_fn_static(state, ns, "HasEventInfo", has_event_info)?;
29:     table_set_rust_fn_static(state, ns, "PlayEventSound", play_event_sound)?;
30:     table_set_rust_fn_static(state, ns, "SetEventColor", set_event_color)?;
31:     table_set_rust_fn_static(state, ns, "SetEventSound", set_event_sound)?;
32:     Ok(())
33: }
34: 
35: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
36: fn register_encounter_timeline_surface(state: &mut LuaState) -> LuaResult<()> {
37:     if cfg!(feature = "retail-12-1-5") {
38:         // The modeled PTR namespace owns its ColorMixin return contract.
39:         return Ok(());
40:     }
41:     let timeline = ensure_namespace(state, "C_EncounterTimeline")?;
42:     table_set_rust_fn_static(state, timeline, "GetEventColor", get_timeline_event_color)
43: }
44: 

src/lua_api/globals/missing_surface/encounter_events.rs:42
40:     }
41:     let timeline = ensure_namespace(state, "C_EncounterTimeline")?;
42:     table_set_rust_fn_static(state, timeline, "GetEventColor", get_timeline_event_color)
43: }
44: 
45: #[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
46: fn register_encounter_timeline_surface(_state: &mut LuaState) -> LuaResult<()> {
47:     Ok(())
48: }
49: 
50: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
51: fn get_timeline_event_color(state: &mut LuaState) -> LuaResult<u32> {
52:     let color = parse_event_id(stack_val(state, 1), state)
53:         .and_then(|event_id| copy_event_color(state, event_id));
54:     push_timeline_color_components(state, color);
55:     Ok(4)
56: }
57: 
58: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
59: fn push_timeline_color_components(state: &mut LuaState, color: Option<Val>) {
60:     let color = color.unwrap_or(Val::Nil);
61:     for key in ["r", "g", "b", "a"] {
62:         let component = color_component(state, color, key);

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterTimelineDocumentation.lua:81
79: 		},
80: 		{
81: 			Name = "GetEventColor",
82: 			Type = "Function",
83: 			RequiresValidTimelineEvent = true,
84: 			SecretWhenEncounterEvent = true,
85: 			SecretArguments = "NotAllowed",
86: 			Documentation = { "Returns the current color used to render timeline event." },
87: 
88: 			Arguments =
89: 			{
90: 				{ Name = "eventID", Type = "EncounterTimelineEventID", Nilable = false },
91: 				{ Name = "overrideTrigger", Type = "EncounterEventColorTrigger", Nilable = true },
92: 			},
93: 
94: 			Returns =
95: 			{
96: 				{ Name = "color", Type = "colorRGBA", Mixin = "ColorMixin", Nilable = false },
97: 			},
98: 		},
99: 		{
100: 			Name = "GetEventCountBySource",
101: 			Type = "Function",
102: 			SecretArguments = "AllowedWhenUntainted",
103: 			Documentation = { "Returns the number of present events in the timeline by their source type." },
104: 
105: 			Arguments =
106: 			{
107: 				{ Name = "source", Type = "EncounterTimelineEventSource", Nilable = false },

## global api-C_HousingCatalog-GetCatalogCategoryAndSubcategoryNames-034
SOURCE C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames

PROVIDER
src/c_api/c_housing.rs:348
346: 
347: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
348: fn get_catalog_category_and_subcategory_names(state: &mut LuaState) -> LuaResult<u32> {
349:     state.push(Val::Nil);
350:     Ok(1)
351: }
352: 
353: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
354: fn room_connection_supports_door_type(state: &mut LuaState) -> LuaResult<u32> {
355:     state.push(Val::Bool(false));
356:     Ok(1)
357: }
358: 
359: #[cfg(feature = "retail-12-1-0")]
360: fn house_finder_ignore_neighborhood(_state: &mut LuaState) -> LuaResult<u32> {
361:     Ok(0)
362: }
363: 
364: #[cfg(feature = "retail-12-1-0")]
365: fn is_inside_owned_house_or_plot(state: &mut LuaState) -> LuaResult<u32> {
366:     let is_inside = {
367:         let sim = borrow_state(state)?;
368:         sim.housing.inside_owned_house || sim.housing.inside_owned_plot

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:92
90: 		},
91: 		{
92: 			Name = "GetCatalogCategoryAndSubcategoryNames",
93: 			Type = "Function",
94: 			MayReturnNothing = true,
95: 			SecretArguments = "AllowedWhenUntainted",
96: 			Documentation = { "If found, returns the names of the parent category and the specified subcategory" },
97: 
98: 			Arguments =
99: 			{
100: 				{ Name = "subcategoryID", Type = "number", Nilable = false },
101: 			},
102: 
103: 			Returns =
104: 			{
105: 				{ Name = "categoryName", Type = "string", Nilable = false },
106: 				{ Name = "subcategoryName", Type = "string", Nilable = false },
107: 			},
108: 		},
109: 		{
110: 			Name = "GetCatalogCategoryInfo",
111: 			Type = "Function",
112: 			SecretArguments = "AllowedWhenUntainted",
113: 
114: 			Arguments =
115: 			{
116: 				{ Name = "categoryID", Type = "number", Nilable = false },
117: 			},
118: 

## global api-C_HousingCustomizeMode-RoomConnectionSupportsDoorType-035
SOURCE C_HousingCustomizeMode.RoomConnectionSupportsDoorType

PROVIDER
src/c_api/c_housing.rs:354
352: 
353: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
354: fn room_connection_supports_door_type(state: &mut LuaState) -> LuaResult<u32> {
355:     state.push(Val::Bool(false));
356:     Ok(1)
357: }
358: 
359: #[cfg(feature = "retail-12-1-0")]
360: fn house_finder_ignore_neighborhood(_state: &mut LuaState) -> LuaResult<u32> {
361:     Ok(0)
362: }
363: 
364: #[cfg(feature = "retail-12-1-0")]
365: fn is_inside_owned_house_or_plot(state: &mut LuaState) -> LuaResult<u32> {
366:     let is_inside = {
367:         let sim = borrow_state(state)?;
368:         sim.housing.inside_owned_house || sim.housing.inside_owned_plot
369:     };
370:     state.push(Val::Bool(is_inside));
371:     Ok(1)
372: }
373: 
374: #[cfg(feature = "retail-12-1-0")]

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCustomizeModeUIDocumentation.lua:314
312: 		},
313: 		{
314: 			Name = "RoomConnectionSupportsDoorType",
315: 			Type = "Function",
316: 			SecretArguments = "AllowedWhenUntainted",
317: 			Documentation = { "Check whether a specific room component, within a specific room, supports a particular doorType" },
318: 
319: 			Arguments =
320: 			{
321: 				{ Name = "roomGUID", Type = "WOWGUID", Nilable = false },
322: 				{ Name = "componentID", Type = "number", Nilable = false },
323: 				{ Name = "newDoortype", Type = "HousingRoomComponentDoorType", Nilable = false },
324: 			},
325: 
326: 			Returns =
327: 			{
328: 				{ Name = "doorTypeSupported", Type = "bool", Nilable = false },
329: 			},
330: 		},
331: 		{
332: 			Name = "SetRoomComponentCeilingType",
333: 			Type = "Function",
334: 			SecretArguments = "AllowedWhenUntainted",
335: 			Documentation = { "Attempt to set a specific ceiling component, within a specific room, to a specific new ceiling type" },
336: 
337: 			Arguments =
338: 			{
339: 				{ Name = "roomGUID", Type = "WOWGUID", Nilable = false },
340: 				{ Name = "componentID", Type = "number", Nilable = false },

## global api-C_HousingLayout-CanSetViewedFloor-036
SOURCE C_HousingLayout.CanSetViewedFloor

PROVIDER
src/c_api/c_housing.rs:645
643: 
644: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
645: fn can_set_viewed_floor(state: &mut LuaState) -> LuaResult<u32> {
646:     state.push(Val::Bool(false));
647:     Ok(1)
648: }
649: 
650: #[cfg(feature = "retail-12-1-0")]
651: fn get_base_room_floor(state: &mut LuaState) -> LuaResult<u32> {
652:     let room_id = i32::from_stack(state, 1)?;
653:     let floor = borrow_state(state)?
654:         .housing
655:         .base_room_floors
656:         .get(&room_id)
657:         .copied();
658:     push_optional_i32(state, floor);
659:     Ok(1)
660: }
661: 
662: #[cfg(feature = "retail-12-1-0")]
663: fn get_room_player_is_in(state: &mut LuaState) -> LuaResult<u32> {
664:     let room_id = { borrow_state(state)?.housing.room_player_is_in };
665:     push_optional_i32(state, room_id);

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:26
24: 		},
25: 		{
26: 			Name = "CanSetViewedFloor",
27: 			Type = "Function",
28: 			SecretArguments = "AllowedWhenUntainted",
29: 
30: 			Arguments =
31: 			{
32: 				{ Name = "floor", Type = "number", Nilable = false },
33: 			},
34: 
35: 			Returns =
36: 			{
37: 				{ Name = "canSet", Type = "bool", Nilable = false },
38: 			},
39: 		},
40: 		{
41: 			Name = "CancelActiveLayoutEditing",
42: 			Type = "Function",
43: 		},
44: 		{
45: 			Name = "ConfirmStairChoice",
46: 			Type = "Function",
47: 			SecretArguments = "AllowedWhenUntainted",
48: 
49: 			Arguments =
50: 			{
51: 				{ Name = "choice", Type = "HousingLayoutStairDirection", Nilable = true, Documentation = { "If not set, the pending stair operation will be cancelled" } },
52: 			},

## global api-C_MerchantFrame-GetMerchantCurrencies-037
SOURCE C_MerchantFrame.GetMerchantCurrencies

PROVIDER
src/c_api/c_merchant_frame.rs:52
50: 
51: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
52: fn get_merchant_currencies(state: &mut LuaState) -> LuaResult<u32> {
53:     let currencies = create_table(state);
54:     state.push(currencies);
55:     Ok(1)
56: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MerchantFrameDocumentation.lua:43
41: 		},
42: 		{
43: 			Name = "GetMerchantCurrencies",
44: 			Type = "Function",
45: 			MayReturnNothing = true,
46: 
47: 			Returns =
48: 			{
49: 				{ Name = "currencies", Type = "table", InnerType = "number", Nilable = false },
50: 			},
51: 		},
52: 		{
53: 			Name = "GetNumJunkItems",
54: 			Type = "Function",
55: 
56: 			Returns =
57: 			{
58: 				{ Name = "numJunkItems", Type = "number", Nilable = false },
59: 			},
60: 		},
61: 		{
62: 			Name = "IsMerchantItemRefundable",
63: 			Type = "Function",
64: 			SecretArguments = "AllowedWhenUntainted",
65: 
66: 			Arguments =
67: 			{
68: 				{ Name = "index", Type = "luaIndex", Nilable = false },
69: 			},

## global api-C_PartyInfo-ConfirmReadyCheck-038
SOURCE C_PartyInfo.ConfirmReadyCheck

PROVIDER
src/c_api/c_party_info.rs:100
98:         state,
99:         table_ref,
100:         "ConfirmReadyCheck",
101:         c_party_info_confirm_ready_check,
102:     )?;
103:     Ok(())
104: }
105: 
106: fn register_loot_method_probes(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
107:     table_set_rust_fn_static(
108:         state,
109:         table_ref,
110:         "GetAvailableLootMethods",
111:         c_party_info_get_available_loot_methods,
112:     )?;
113:     table_set_rust_fn_static(
114:         state,
115:         table_ref,
116:         "IsLootMethodAvailable",
117:         c_party_info_is_loot_method_available,
118:     )?;
119:     #[cfg(not(feature = "retail-12-0-5"))]
120:     table_set_rust_fn_static(

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:103
101: 		},
102: 		{
103: 			Name = "ConfirmReadyCheck",
104: 			Type = "Function",
105: 			HasRestrictions = true,
106: 			SecretArguments = "AllowedWhenUntainted",
107: 
108: 			Arguments =
109: 			{
110: 				{ Name = "isReady", Type = "bool", Nilable = false },
111: 			},
112: 		},
113: 		{
114: 			Name = "ConfirmRequestInviteFromUnit",
115: 			Type = "Function",
116: 			RequiresValidInviteTarget = true,
117: 			SecretArguments = "AllowedWhenUntainted",
118: 			Documentation = { "Immediately request an invite into the target party, this is the confirmation function to call after RequestInviteFromUnit, or if you would like to skip the confirmation process." },
119: 
120: 			Arguments =
121: 			{
122: 				{ Name = "targetName", Type = "cstring", Nilable = false },
123: 				{ Name = "tank", Type = "bool", Nilable = true },
124: 				{ Name = "healer", Type = "bool", Nilable = true },
125: 				{ Name = "dps", Type = "bool", Nilable = true },
126: 			},
127: 		},
128: 		{
129: 			Name = "ConvertToParty",

## global api-C_PartyInfo-DemoteAssistant-039
SOURCE C_PartyInfo.DemoteAssistant

PROVIDER
src/c_api/c_party_info.rs:70
68:         state,
69:         table_ref,
70:         "DemoteAssistant",
71:         c_party_info_demote_assistant,
72:     )?;
73:     table_set_rust_fn_static(
74:         state,
75:         table_ref,
76:         "PromoteToAssistant",
77:         c_party_info_promote_to_assistant,
78:     )?;
79:     table_set_rust_fn_static(
80:         state,
81:         table_ref,
82:         "PromoteToLeader",
83:         c_party_info_promote_to_leader,
84:     )?;
85:     table_set_rust_fn_static(
86:         state,
87:         table_ref,
88:         "SetEveryoneIsAssistant",
89:         c_party_info_set_everyone_is_assistant,
90:     )?;

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:144
142: 		},
143: 		{
144: 			Name = "DemoteAssistant",
145: 			Type = "Function",
146: 			HasRestrictions = true,
147: 			SecretArguments = "AllowedWhenUntainted",
148: 
149: 			Arguments =
150: 			{
151: 				{ Name = "name", Type = "cstring", Nilable = false },
152: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
153: 			},
154: 		},
155: 		{
156: 			Name = "DoCountdown",
157: 			Type = "Function",
158: 			HasRestrictions = true,
159: 			SecretArguments = "AllowedWhenUntainted",
160: 
161: 			Arguments =
162: 			{
163: 				{ Name = "seconds", Type = "number", Nilable = false },
164: 			},
165: 
166: 			Returns =
167: 			{
168: 				{ Name = "success", Type = "bool", Nilable = false },
169: 			},
170: 		},

## global api-C_PartyInfo-DoReadyCheck-040
SOURCE C_PartyInfo.DoReadyCheck

PROVIDER
src/c_api/c_party_info.rs:94
92:         state,
93:         table_ref,
94:         "DoReadyCheck",
95:         c_party_info_do_ready_check,
96:     )?;
97:     table_set_rust_fn_static(
98:         state,
99:         table_ref,
100:         "ConfirmReadyCheck",
101:         c_party_info_confirm_ready_check,
102:     )?;
103:     Ok(())
104: }
105: 
106: fn register_loot_method_probes(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
107:     table_set_rust_fn_static(
108:         state,
109:         table_ref,
110:         "GetAvailableLootMethods",
111:         c_party_info_get_available_loot_methods,
112:     )?;
113:     table_set_rust_fn_static(
114:         state,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:172
170: 		},
171: 		{
172: 			Name = "DoReadyCheck",
173: 			Type = "Function",
174: 			HasRestrictions = true,
175: 		},
176: 		{
177: 			Name = "GetActiveCategories",
178: 			Type = "Function",
179: 			MayReturnNothing = true,
180: 
181: 			Returns =
182: 			{
183: 				{ Name = "categories", Type = "table", InnerType = "number", Nilable = false },
184: 			},
185: 		},
186: 		{
187: 			Name = "GetAvailableLootMethods",
188: 			Type = "Function",
189: 
190: 			Returns =
191: 			{
192: 				{ Name = "methods", Type = "table", InnerType = "LootMethod", Nilable = false },
193: 			},
194: 		},
195: 		{
196: 			Name = "GetInstanceAbandonShutdownTime",
197: 			Type = "Function",
198: 			Documentation = { "Returns the total duration of the shutdown time after a vote passes and how much time is left before it ends" },

## global api-C_PartyInfo-IsGUIDInGroup-041
SOURCE C_PartyInfo.IsGUIDInGroup

PROVIDER
src/c_api/c_party_info.rs:62
60:         state,
61:         table_ref,
62:         "IsGUIDInGroup",
63:         c_party_info_is_guid_in_group,
64:     )?;
65:     table_set_rust_fn_static(state, table_ref, "LeaveParty", c_party_info_leave_party)?;
66:     table_set_rust_fn_static(state, table_ref, "UninviteUnit", c_party_info_uninvite_unit)?;
67:     table_set_rust_fn_static(
68:         state,
69:         table_ref,
70:         "DemoteAssistant",
71:         c_party_info_demote_assistant,
72:     )?;
73:     table_set_rust_fn_static(
74:         state,
75:         table_ref,
76:         "PromoteToAssistant",
77:         c_party_info_promote_to_assistant,
78:     )?;
79:     table_set_rust_fn_static(
80:         state,
81:         table_ref,
82:         "PromoteToLeader",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:420
418: 		},
419: 		{
420: 			Name = "IsGUIDInGroup",
421: 			Type = "Function",
422: 			SecretArguments = "AllowedWhenUntainted",
423: 
424: 			Arguments =
425: 			{
426: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
427: 				{ Name = "category", Type = "luaIndex", Nilable = true, Documentation = { "If not provided, the active party is used" } },
428: 			},
429: 
430: 			Returns =
431: 			{
432: 				{ Name = "isInGroup", Type = "bool", Nilable = false },
433: 			},
434: 		},
435: 		{
436: 			Name = "IsLootMethodAvailable",
437: 			Type = "Function",
438: 			SecretArguments = "AllowedWhenUntainted",
439: 
440: 			Arguments =
441: 			{
442: 				{ Name = "method", Type = "LootMethod", Nilable = false },
443: 			},
444: 
445: 			Returns =
446: 			{

## global api-C_PartyInfo-PromoteToAssistant-042
SOURCE C_PartyInfo.PromoteToAssistant

PROVIDER
src/c_api/c_party_info.rs:76
74:         state,
75:         table_ref,
76:         "PromoteToAssistant",
77:         c_party_info_promote_to_assistant,
78:     )?;
79:     table_set_rust_fn_static(
80:         state,
81:         table_ref,
82:         "PromoteToLeader",
83:         c_party_info_promote_to_leader,
84:     )?;
85:     table_set_rust_fn_static(
86:         state,
87:         table_ref,
88:         "SetEveryoneIsAssistant",
89:         c_party_info_set_everyone_is_assistant,
90:     )?;
91:     table_set_rust_fn_static(
92:         state,
93:         table_ref,
94:         "DoReadyCheck",
95:         c_party_info_do_ready_check,
96:     )?;

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:495
493: 		},
494: 		{
495: 			Name = "PromoteToAssistant",
496: 			Type = "Function",
497: 			HasRestrictions = true,
498: 			SecretArguments = "AllowedWhenUntainted",
499: 
500: 			Arguments =
501: 			{
502: 				{ Name = "name", Type = "cstring", Nilable = false },
503: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
504: 			},
505: 		},
506: 		{
507: 			Name = "PromoteToLeader",
508: 			Type = "Function",
509: 			HasRestrictions = true,
510: 			SecretArguments = "AllowedWhenUntainted",
511: 
512: 			Arguments =
513: 			{
514: 				{ Name = "name", Type = "cstring", Nilable = false },
515: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
516: 			},
517: 		},
518: 		{
519: 			Name = "RequestInviteFromUnit",
520: 			Type = "Function",
521: 			RequiresValidInviteTarget = true,

## global api-C_PartyInfo-PromoteToLeader-043
SOURCE C_PartyInfo.PromoteToLeader

PROVIDER
src/c_api/c_party_info.rs:82
80:         state,
81:         table_ref,
82:         "PromoteToLeader",
83:         c_party_info_promote_to_leader,
84:     )?;
85:     table_set_rust_fn_static(
86:         state,
87:         table_ref,
88:         "SetEveryoneIsAssistant",
89:         c_party_info_set_everyone_is_assistant,
90:     )?;
91:     table_set_rust_fn_static(
92:         state,
93:         table_ref,
94:         "DoReadyCheck",
95:         c_party_info_do_ready_check,
96:     )?;
97:     table_set_rust_fn_static(
98:         state,
99:         table_ref,
100:         "ConfirmReadyCheck",
101:         c_party_info_confirm_ready_check,
102:     )?;

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:507
505: 		},
506: 		{
507: 			Name = "PromoteToLeader",
508: 			Type = "Function",
509: 			HasRestrictions = true,
510: 			SecretArguments = "AllowedWhenUntainted",
511: 
512: 			Arguments =
513: 			{
514: 				{ Name = "name", Type = "cstring", Nilable = false },
515: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
516: 			},
517: 		},
518: 		{
519: 			Name = "RequestInviteFromUnit",
520: 			Type = "Function",
521: 			RequiresValidInviteTarget = true,
522: 			SecretArguments = "AllowedWhenUntainted",
523: 			Documentation = { "Attempt to request an invite into the target party, requires confirmation in some cases (e.g. there is a party sync in progress)." },
524: 
525: 			Arguments =
526: 			{
527: 				{ Name = "targetName", Type = "cstring", Nilable = false },
528: 				{ Name = "tank", Type = "bool", Nilable = true },
529: 				{ Name = "healer", Type = "bool", Nilable = true },
530: 				{ Name = "dps", Type = "bool", Nilable = true },
531: 			},
532: 		},
533: 		{

## global api-C_PartyInfo-SetEveryoneIsAssistant-044
SOURCE C_PartyInfo.SetEveryoneIsAssistant

PROVIDER
src/c_api/c_party_info.rs:88
86:         state,
87:         table_ref,
88:         "SetEveryoneIsAssistant",
89:         c_party_info_set_everyone_is_assistant,
90:     )?;
91:     table_set_rust_fn_static(
92:         state,
93:         table_ref,
94:         "DoReadyCheck",
95:         c_party_info_do_ready_check,
96:     )?;
97:     table_set_rust_fn_static(
98:         state,
99:         table_ref,
100:         "ConfirmReadyCheck",
101:         c_party_info_confirm_ready_check,
102:     )?;
103:     Ok(())
104: }
105: 
106: fn register_loot_method_probes(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
107:     table_set_rust_fn_static(
108:         state,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:534
532: 		},
533: 		{
534: 			Name = "SetEveryoneIsAssistant",
535: 			Type = "Function",
536: 			HasRestrictions = true,
537: 			SecretArguments = "AllowedWhenUntainted",
538: 
539: 			Arguments =
540: 			{
541: 				{ Name = "isAssistant", Type = "bool", Nilable = false },
542: 			},
543: 
544: 			Returns =
545: 			{
546: 				{ Name = "updated", Type = "bool", Nilable = false },
547: 			},
548: 		},
549: 		{
550: 			Name = "SetInstanceAbandonVoteResponse",
551: 			Type = "Function",
552: 			HasRestrictions = true,
553: 			SecretArguments = "AllowedWhenUntainted",
554: 			Documentation = { "Vote on whether to abandon instance, true for yes, false for no" },
555: 
556: 			Arguments =
557: 			{
558: 				{ Name = "response", Type = "bool", Nilable = false },
559: 			},
560: 		},

## global api-C_PartyInfo-UninviteUnit-045
SOURCE C_PartyInfo.UninviteUnit

PROVIDER
src/c_api/c_party_info.rs:66
64:     )?;
65:     table_set_rust_fn_static(state, table_ref, "LeaveParty", c_party_info_leave_party)?;
66:     table_set_rust_fn_static(state, table_ref, "UninviteUnit", c_party_info_uninvite_unit)?;
67:     table_set_rust_fn_static(
68:         state,
69:         table_ref,
70:         "DemoteAssistant",
71:         c_party_info_demote_assistant,
72:     )?;
73:     table_set_rust_fn_static(
74:         state,
75:         table_ref,
76:         "PromoteToAssistant",
77:         c_party_info_promote_to_assistant,
78:     )?;
79:     table_set_rust_fn_static(
80:         state,
81:         table_ref,
82:         "PromoteToLeader",
83:         c_party_info_promote_to_leader,
84:     )?;
85:     table_set_rust_fn_static(
86:         state,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:596
594: 		},
595: 		{
596: 			Name = "UninviteUnit",
597: 			Type = "Function",
598: 			HasRestrictions = true,
599: 			SecretArguments = "AllowedWhenUntainted",
600: 
601: 			Arguments =
602: 			{
603: 				{ Name = "name", Type = "cstring", Nilable = false },
604: 				{ Name = "reason", Type = "cstring", Nilable = true },
605: 				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
606: 			},
607: 		},
608: 	},
609: 
610: 	Events =
611: 	{
612: 		{
613: 			Name = "BnetRequestInviteConfirmation",
614: 			Type = "Event",
615: 			LiteralName = "BNET_REQUEST_INVITE_CONFIRMATION",
616: 			SynchronousEvent = true,
617: 			Payload =
618: 			{
619: 				{ Name = "gameAccountID", Type = "number", Nilable = false },
620: 				{ Name = "questSessionActive", Type = "bool", Nilable = false },
621: 				{ Name = "tank", Type = "bool", Nilable = false },
622: 				{ Name = "healer", Type = "bool", Nilable = false },

## global api-C_PingSecure-ClearPendingPingOffScreenCallback-046
SOURCE C_PingSecure.ClearPendingPingOffScreenCallback

PROVIDER
src/c_api/c_ping_secure.rs:76
74:         state,
75:         table_ref,
76:         "ClearPendingPingOffScreenCallback",
77:         clear_pending_ping_off_screen_callback,
78:     )
79: }
80: 
81: const PING_SECURE_CALLBACK_SETTERS: &[(&str, fn(&mut LuaState) -> LuaResult<u32>)] = &[
82:     (
83:         "SetPendingPingOffScreenCallback",
84:         set_pending_ping_off_screen_callback,
85:     ),
86:     (
87:         "SetPingCooldownStartedCallback",
88:         set_ping_cooldown_started_callback,
89:     ),
90:     (
91:         "SetPingPinFrameAddedCallback",
92:         set_ping_pin_frame_added_callback,
93:     ),
94:     (
95:         "SetPingPinFrameRemovedCallback",
96:         set_ping_pin_frame_removed_callback,

DECLARATION


## global api-C_PingSecure-SetPendingPingOffScreenCallback-047
SOURCE C_PingSecure.SetPendingPingOffScreenCallback

PROVIDER
src/c_api/c_ping_secure.rs:83
81: const PING_SECURE_CALLBACK_SETTERS: &[(&str, fn(&mut LuaState) -> LuaResult<u32>)] = &[
82:     (
83:         "SetPendingPingOffScreenCallback",
84:         set_pending_ping_off_screen_callback,
85:     ),
86:     (
87:         "SetPingCooldownStartedCallback",
88:         set_ping_cooldown_started_callback,
89:     ),
90:     (
91:         "SetPingPinFrameAddedCallback",
92:         set_ping_pin_frame_added_callback,
93:     ),
94:     (
95:         "SetPingPinFrameRemovedCallback",
96:         set_ping_pin_frame_removed_callback,
97:     ),
98:     (
99:         "SetPingPinFrameScreenClampStateUpdatedCallback",
100:         set_ping_pin_frame_screen_clamp_state_updated_callback,
101:     ),
102:     (
103:         "SetPingRadialWheelCreatedCallback",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:159
157: 		},
158: 		{
159: 			Name = "SetPendingPingOffScreenCallback",
160: 			Type = "Function",
161: 			HasRestrictions = true,
162: 			SecretArguments = "AllowedWhenUntainted",
163: 
164: 			Arguments =
165: 			{
166: 				{ Name = "cb", Type = "PendingPingOffScreenCallback", Nilable = false },
167: 			},
168: 		},
169: 		{
170: 			Name = "SetPingCooldownStartedCallback",
171: 			Type = "Function",
172: 			HasRestrictions = true,
173: 			SecretArguments = "AllowedWhenUntainted",
174: 
175: 			Arguments =
176: 			{
177: 				{ Name = "cb", Type = "PingCooldownStartedCallback", Nilable = false },
178: 			},
179: 		},
180: 		{
181: 			Name = "SetPingPinFrameAddedCallback",
182: 			Type = "Function",
183: 			HasRestrictions = true,
184: 			SecretArguments = "AllowedWhenUntainted",
185: 

## global api-C_QuestHub-GetDragonridingRacesForAreaPOI-048
SOURCE C_QuestHub.GetDragonridingRacesForAreaPOI

PROVIDER
src/c_api/c_quest_hub.rs:27
25:         state,
26:         quest_hub,
27:         "GetDragonridingRacesForAreaPOI",
28:         get_dragonriding_races_for_area_poi,
29:     )
30: }
31: 
32: #[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
33: fn register_patch_12_0_7_quest_hub_surface(
34:     _state: &mut LuaState,
35:     _quest_hub: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
36: ) -> LuaResult<()> {
37:     Ok(())
38: }
39: 
40: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
41: fn get_dragonriding_races_for_area_poi(state: &mut LuaState) -> LuaResult<u32> {
42:     let races = create_table(state);
43:     state.push(races);
44:     Ok(1)
45: }

DECLARATION


## global api-C_UIFileAsset-GetFileID-049
SOURCE C_UIFileAsset.GetFileID

PROVIDER
src/c_api/c_ui_file_asset.rs:16
14: pub(crate) fn register_c_ui_file_asset(state: &mut LuaState) -> LuaResult<()> {
15:     let table_ref = ensure_namespace(state, "C_UIFileAsset")?;
16:     table_set_rust_fn_static(state, table_ref, "GetFileID", c_ui_file_asset_get_file_id)?;
17:     table_set_rust_fn_static(
18:         state,
19:         table_ref,
20:         "IsKnownFile",
21:         c_ui_file_asset_is_known_file,
22:     )?;
23:     table_set_rust_fn_static(
24:         state,
25:         table_ref,
26:         "IsLooseFile",
27:         c_ui_file_asset_is_loose_file,
28:     )
29: }
30: 
31: fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
32:     match file_id_from_asset_arg(state) {
33:         Some(file_id) => state.push(Val::Num(file_id as f64)),
34:         None => state.push(Val::Nil),
35:     }
36:     Ok(1)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:11
9: 	{
10: 		{
11: 			Name = "GetFileID",
12: 			Type = "Function",
13: 			SecretArguments = "AllowedWhenUntainted",
14: 			Documentation = { "Returns the numeric file ID associated with a file asset." },
15: 
16: 			Arguments =
17: 			{
18: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
19: 			},
20: 
21: 			Returns =
22: 			{
23: 				{ Name = "assetFileID", Type = "fileID", Nilable = true, Documentation = { "The file ID corresponding to the given asset. If the asset is already a file ID, it is returned unchanged; otherwise returns nil if the file path is not known to the client." } },
24: 			},
25: 		},
26: 		{
27: 			Name = "IsKnownFile",
28: 			Type = "Function",
29: 			SecretArguments = "AllowedWhenUntainted",
30: 			Documentation = { "Determines whether a file asset is known to the client, either as a shipped asset or a locally existing loose file." },
31: 
32: 			Arguments =
33: 			{
34: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
35: 			},
36: 
37: 			Returns =

## global api-C_UIFileAsset-IsKnownFile-050
SOURCE C_UIFileAsset.IsKnownFile

PROVIDER
src/c_api/c_ui_file_asset.rs:20
18:         state,
19:         table_ref,
20:         "IsKnownFile",
21:         c_ui_file_asset_is_known_file,
22:     )?;
23:     table_set_rust_fn_static(
24:         state,
25:         table_ref,
26:         "IsLooseFile",
27:         c_ui_file_asset_is_loose_file,
28:     )
29: }
30: 
31: fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
32:     match file_id_from_asset_arg(state) {
33:         Some(file_id) => state.push(Val::Num(file_id as f64)),
34:         None => state.push(Val::Nil),
35:     }
36:     Ok(1)
37: }
38: 
39: fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
40:     state.push(Val::Bool(!matches!(query_asset(state), Asset::Missing)));

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:27
25: 		},
26: 		{
27: 			Name = "IsKnownFile",
28: 			Type = "Function",
29: 			SecretArguments = "AllowedWhenUntainted",
30: 			Documentation = { "Determines whether a file asset is known to the client, either as a shipped asset or a locally existing loose file." },
31: 
32: 			Arguments =
33: 			{
34: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
35: 			},
36: 
37: 			Returns =
38: 			{
39: 				{ Name = "isValid", Type = "bool", Nilable = false, Documentation = { "True if the asset is shipped with the client or refers to a known loose file. Existence or openability of loose files is not verified." } },
40: 			},
41: 		},
42: 		{
43: 			Name = "IsLooseFile",
44: 			Type = "Function",
45: 			SecretArguments = "AllowedWhenUntainted",
46: 			Documentation = { "Determines whether a file asset refers to a known loose (local) file." },
47: 
48: 			Arguments =
49: 			{
50: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
51: 			},
52: 
53: 			Returns =

## global api-C_UIFileAsset-IsLooseFile-051
SOURCE C_UIFileAsset.IsLooseFile

PROVIDER
src/c_api/c_ui_file_asset.rs:26
24:         state,
25:         table_ref,
26:         "IsLooseFile",
27:         c_ui_file_asset_is_loose_file,
28:     )
29: }
30: 
31: fn c_ui_file_asset_get_file_id(state: &mut LuaState) -> LuaResult<u32> {
32:     match file_id_from_asset_arg(state) {
33:         Some(file_id) => state.push(Val::Num(file_id as f64)),
34:         None => state.push(Val::Nil),
35:     }
36:     Ok(1)
37: }
38: 
39: fn c_ui_file_asset_is_known_file(state: &mut LuaState) -> LuaResult<u32> {
40:     state.push(Val::Bool(!matches!(query_asset(state), Asset::Missing)));
41:     Ok(1)
42: }
43: 
44: fn c_ui_file_asset_is_loose_file(state: &mut LuaState) -> LuaResult<u32> {
45:     state.push(Val::Bool(matches!(query_asset(state), Asset::Loose)));
46:     Ok(1)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:43
41: 		},
42: 		{
43: 			Name = "IsLooseFile",
44: 			Type = "Function",
45: 			SecretArguments = "AllowedWhenUntainted",
46: 			Documentation = { "Determines whether a file asset refers to a known loose (local) file." },
47: 
48: 			Arguments =
49: 			{
50: 				{ Name = "asset", Type = "FileAsset", Nilable = false },
51: 			},
52: 
53: 			Returns =
54: 			{
55: 				{ Name = "isLooseFile", Type = "bool", Nilable = false, Documentation = { "True if the asset refers to a loose file known to the client." } },
56: 			},
57: 		},
58: 	},
59: 
60: 	Events =
61: 	{
62: 	},
63: 
64: 	Tables =
65: 	{
66: 	},
67: 	Predicates =
68: 	{
69: 	},

## global api-GetEventCPUUsage-052
SOURCE GetEventCPUUsage

PROVIDER
src/lua_api/workarounds/temporary/performance_metric_defaults.rs:49
47: end
48: 
49: if GetEventCPUUsage == nil then
50:   function GetEventCPUUsage(_event)
51:     return 0
52:   end
53: end
54: 
55: if GetFunctionCPUUsage == nil then
56:   function GetFunctionCPUUsage(_fn, _includeSubroutines)
57:     return 0, 0
58:   end
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1

src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50
48: 
49: if GetEventCPUUsage == nil then
50:   function GetEventCPUUsage(_event)
51:     return 0
52:   end
53: end
54: 
55: if GetFunctionCPUUsage == nil then
56:   function GetFunctionCPUUsage(_fn, _includeSubroutines)
57:     return 0, 0
58:   end
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40
38: 		},
39: 		{
40: 			Name = "GetEventCPUUsage",
41: 			Type = "Function",
42: 
43: 			Returns =
44: 			{
45: 				{ Name = "call_time", Type = "number", Nilable = false },
46: 				{ Name = "call_count", Type = "number", Nilable = false },
47: 			},
48: 		},
49: 		{
50: 			Name = "GetFrameCPUUsage",
51: 			Type = "Function",
52: 			SecretArguments = "AllowedWhenUntainted",
53: 
54: 			Arguments =
55: 			{
56: 				{ Name = "frame", Type = "SimpleFrame", Nilable = false },
57: 				{ Name = "includeChildren", Type = "bool", Nilable = false, Default = false },
58: 			},
59: 
60: 			Returns =
61: 			{
62: 				{ Name = "call_time", Type = "number", Nilable = false },
63: 				{ Name = "call_count", Type = "number", Nilable = false },
64: 			},
65: 		},
66: 		{

## global api-GetFunctionCPUUsage-053
SOURCE GetFunctionCPUUsage

PROVIDER
src/lua_api/workarounds/temporary/performance_metric_defaults.rs:55
53: end
54: 
55: if GetFunctionCPUUsage == nil then
56:   function GetFunctionCPUUsage(_fn, _includeSubroutines)
57:     return 0, 0
58:   end
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end
71: end
72: 
73: if GetMovieDownloadProgress == nil then
74:   function GetMovieDownloadProgress(_movieID)
75:     return false, 0, 0

src/lua_api/workarounds/temporary/performance_metric_defaults.rs:56
54: 
55: if GetFunctionCPUUsage == nil then
56:   function GetFunctionCPUUsage(_fn, _includeSubroutines)
57:     return 0, 0
58:   end
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end
71: end
72: 
73: if GetMovieDownloadProgress == nil then
74:   function GetMovieDownloadProgress(_movieID)
75:     return false, 0, 0
76:   end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:67
65: 		},
66: 		{
67: 			Name = "GetFunctionCPUUsage",
68: 			Type = "Function",
69: 
70: 			Returns =
71: 			{
72: 				{ Name = "call_time", Type = "number", Nilable = false },
73: 				{ Name = "call_count", Type = "number", Nilable = false },
74: 			},
75: 		},
76: 		{
77: 			Name = "GetScriptCPUUsage",
78: 			Type = "Function",
79: 
80: 			Returns =
81: 			{
82: 				{ Name = "result", Type = "number", Nilable = false },
83: 			},
84: 		},
85: 		{
86: 			Name = "ResetCPUUsage",
87: 			Type = "Function",
88: 		},
89: 		{
90: 			Name = "UpdateAddOnCPUUsage",
91: 			Type = "Function",
92: 		},
93: 		{

## global api-GetScriptCPUUsage-054
SOURCE GetScriptCPUUsage

PROVIDER
src/lua_api/workarounds/temporary/performance_metric_defaults.rs:61
59: end
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end
71: end
72: 
73: if GetMovieDownloadProgress == nil then
74:   function GetMovieDownloadProgress(_movieID)
75:     return false, 0, 0
76:   end
77: end
78: 
79: MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL = MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL or "Protocol: %s (home) %s (world)"
80: 
81: if GetProtocolTypes == nil then

src/lua_api/workarounds/temporary/performance_metric_defaults.rs:62
60: 
61: if GetScriptCPUUsage == nil then
62:   function GetScriptCPUUsage(_frame, _script, _includeChildren)
63:     return 0, 0
64:   end
65: end
66: 
67: if GetDownloadedPercentage == nil then
68:   function GetDownloadedPercentage()
69:     return 1
70:   end
71: end
72: 
73: if GetMovieDownloadProgress == nil then
74:   function GetMovieDownloadProgress(_movieID)
75:     return false, 0, 0
76:   end
77: end
78: 
79: MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL = MAINMENUBAR_COMMUNICATION_PROTOCOL_LABEL or "Protocol: %s (home) %s (world)"
80: 
81: if GetProtocolTypes == nil then
82:   function GetProtocolTypes()

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:77
75: 		},
76: 		{
77: 			Name = "GetScriptCPUUsage",
78: 			Type = "Function",
79: 
80: 			Returns =
81: 			{
82: 				{ Name = "result", Type = "number", Nilable = false },
83: 			},
84: 		},
85: 		{
86: 			Name = "ResetCPUUsage",
87: 			Type = "Function",
88: 		},
89: 		{
90: 			Name = "UpdateAddOnCPUUsage",
91: 			Type = "Function",
92: 		},
93: 		{
94: 			Name = "UpdateAddOnMemoryUsage",
95: 			Type = "Function",
96: 		},
97: 	},
98: 
99: 	Events =
100: 	{
101: 	},
102: 
103: 	Tables =

## global api-GetSecurePendingButtonCallback-055
SOURCE GetSecurePendingButtonCallback

PROVIDER
src/c_api/c_ping_secure.rs:30
28:         state,
29:         globals,
30:         "GetSecurePendingButtonCallback",
31:         get_button_callback,
32:     )?;
33:     table_set_rust_fn_static(
34:         state,
35:         globals,
36:         "GetSecurePendingPingOffScreenCallback",
37:         get_pending_ping_off_screen_callback,
38:     )?;
39:     table_set_rust_fn_static(
40:         state,
41:         globals,
42:         "GetSecurePendingToggleRunCallback",
43:         get_toggle_run_callback,
44:     )?;
45:     table_set_rust_fn_static(
46:         state,
47:         globals,
48:         "SetSecurePendingButtonCallback",
49:         set_button_callback,
50:     )?;

DECLARATION


## global api-GetSecurePendingPingOffScreenCallback-056
SOURCE GetSecurePendingPingOffScreenCallback

PROVIDER
src/c_api/c_ping_secure.rs:36
34:         state,
35:         globals,
36:         "GetSecurePendingPingOffScreenCallback",
37:         get_pending_ping_off_screen_callback,
38:     )?;
39:     table_set_rust_fn_static(
40:         state,
41:         globals,
42:         "GetSecurePendingToggleRunCallback",
43:         get_toggle_run_callback,
44:     )?;
45:     table_set_rust_fn_static(
46:         state,
47:         globals,
48:         "SetSecurePendingButtonCallback",
49:         set_button_callback,
50:     )?;
51:     table_set_rust_fn_static(
52:         state,
53:         globals,
54:         "SetSecurePendingPingOffScreenCallback",
55:         set_pending_ping_off_screen_callback,
56:     )?;

DECLARATION


## global api-GetSecurePendingToggleRunCallback-057
SOURCE GetSecurePendingToggleRunCallback

PROVIDER
src/c_api/c_ping_secure.rs:42
40:         state,
41:         globals,
42:         "GetSecurePendingToggleRunCallback",
43:         get_toggle_run_callback,
44:     )?;
45:     table_set_rust_fn_static(
46:         state,
47:         globals,
48:         "SetSecurePendingButtonCallback",
49:         set_button_callback,
50:     )?;
51:     table_set_rust_fn_static(
52:         state,
53:         globals,
54:         "SetSecurePendingPingOffScreenCallback",
55:         set_pending_ping_off_screen_callback,
56:     )?;
57:     table_set_rust_fn_static(
58:         state,
59:         globals,
60:         "SetSecurePendingToggleRunCallback",
61:         set_toggle_run_callback,
62:     )?;

DECLARATION


## global api-GameTooltip_AddMoneyLine-058
SOURCE GameTooltip_AddMoneyLine

PROVIDER
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GameTooltip/Mainline/GameTooltip.lua:320
318: end
319: 
320: function GameTooltip_AddMoneyLine(self, rawCopper, useRedLineColor)
321: 	local color = useRedLineColor and RED_FONT_COLOR or HIGHLIGHT_FONT_COLOR;
322: 	GameTooltip_AddColoredMoneyLine(self, rawCopper, color);
323: end
324: 
325: function GameTooltip_OnTooltipAddMoney(self, cost, maxcost)
326: 	if( not maxcost or maxcost < 1 ) then --We just have 1 price to display
327: 		GameTooltip_AddHighlightLine(self, string.format("%s: %s", SELL_PRICE, MoneyFormatterUtil.FormatMoney(cost, GameTooltipMoneyFormat)));
328: 	else
329: 		GameTooltip_AddColoredLine(self, ("%s:"):format(SELL_PRICE), HIGHLIGHT_FONT_COLOR);
330: 		local indent = string.rep(" ",4)
331: 		GameTooltip_AddHighlightLine(self, string.format("%s%s: %s", indent, MINIMUM, MoneyFormatterUtil.FormatMoney(cost, GameTooltipMoneyFormat)));
332: 		GameTooltip_AddHighlightLine(self, string.format("%s%s: %s", indent, MAXIMUM, MoneyFormatterUtil.FormatMoney(maxcost, GameTooltipMoneyFormat)));
333: 	end
334: end
335: 
336: GAME_TOOLTIP_BACKDROP_STYLE_DEFAULT_DARK = {
337: 	layoutType = "TooltipDefaultDarkLayout",
338: };
339: 
340: GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM = {

DECLARATION


## global api-SetSecurePendingButtonCallback-059
SOURCE SetSecurePendingButtonCallback

PROVIDER
src/c_api/c_ping_secure.rs:48
46:         state,
47:         globals,
48:         "SetSecurePendingButtonCallback",
49:         set_button_callback,
50:     )?;
51:     table_set_rust_fn_static(
52:         state,
53:         globals,
54:         "SetSecurePendingPingOffScreenCallback",
55:         set_pending_ping_off_screen_callback,
56:     )?;
57:     table_set_rust_fn_static(
58:         state,
59:         globals,
60:         "SetSecurePendingToggleRunCallback",
61:         set_toggle_run_callback,
62:     )?;
63:     Ok(())
64: }
65: 
66: fn register_callback_setters(
67:     state: &mut LuaState,
68:     table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,

DECLARATION


## global api-SetSecurePendingPingOffScreenCallback-060
SOURCE SetSecurePendingPingOffScreenCallback

PROVIDER
src/c_api/c_ping_secure.rs:54
52:         state,
53:         globals,
54:         "SetSecurePendingPingOffScreenCallback",
55:         set_pending_ping_off_screen_callback,
56:     )?;
57:     table_set_rust_fn_static(
58:         state,
59:         globals,
60:         "SetSecurePendingToggleRunCallback",
61:         set_toggle_run_callback,
62:     )?;
63:     Ok(())
64: }
65: 
66: fn register_callback_setters(
67:     state: &mut LuaState,
68:     table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
69: ) -> LuaResult<()> {
70:     for (name, function) in PING_SECURE_CALLBACK_SETTERS {
71:         table_set_rust_fn_static(state, table_ref, name, *function)?;
72:     }
73:     table_set_rust_fn_static(
74:         state,

DECLARATION


## global api-SetSecurePendingToggleRunCallback-061
SOURCE SetSecurePendingToggleRunCallback

PROVIDER
src/c_api/c_ping_secure.rs:60
58:         state,
59:         globals,
60:         "SetSecurePendingToggleRunCallback",
61:         set_toggle_run_callback,
62:     )?;
63:     Ok(())
64: }
65: 
66: fn register_callback_setters(
67:     state: &mut LuaState,
68:     table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
69: ) -> LuaResult<()> {
70:     for (name, function) in PING_SECURE_CALLBACK_SETTERS {
71:         table_set_rust_fn_static(state, table_ref, name, *function)?;
72:     }
73:     table_set_rust_fn_static(
74:         state,
75:         table_ref,
76:         "ClearPendingPingOffScreenCallback",
77:         clear_pending_ping_off_screen_callback,
78:     )
79: }
80: 

DECLARATION


## source-context-063
SOURCE Global API Removed (17):

PROVIDER


DECLARATION


## global api-BNInviteFriend-064
SOURCE BNInviteFriend

PROVIDER


DECLARATION


## global api-C_ClickBindings-GetStringFromModifiers-065
SOURCE C_ClickBindings.GetStringFromModifiers

PROVIDER
src/lua_api/workarounds/temporary/click_bindings_defaults.rs:27
25: end
26: 
27: if rawget(C_ClickBindings, "GetStringFromModifiers") == nil then
28:     function C_ClickBindings.GetStringFromModifiers(modifiers)
29:         return GetStringFromModifiers(modifiers)
30:     end
31: end
32: 
33: if rawget(C_ClickBindings, "GetTutorialShown") == nil then
34:     function C_ClickBindings.GetTutorialShown()
35:         return true
36:     end
37: end
38: 
39: if rawget(_G, "MakeModifiers") == nil then
40:     function MakeModifiers()
41:         local modifiers = 0
42:         if type(IsShiftKeyDown) == "function" and IsShiftKeyDown() then
43:             modifiers = modifiers + 1
44:         end
45:         if type(IsControlKeyDown) == "function" and IsControlKeyDown() then
46:             modifiers = modifiers + 2
47:         end

DECLARATION


## global api-C_ClickBindings-MakeModifiers-066
SOURCE C_ClickBindings.MakeModifiers

PROVIDER
src/lua_api/workarounds/temporary/click_bindings_defaults.rs:55
53: end
54: 
55: if rawget(C_ClickBindings, "MakeModifiers") == nil then
56:     function C_ClickBindings.MakeModifiers()
57:         return MakeModifiers()
58:     end
59: end
60: 
61: if rawget(C_ClickBindings, "SetTutorialShown") == nil then
62:     function C_ClickBindings.SetTutorialShown(_shown)
63:     end
64: end
65: "#;
66: 
67: pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
68:     lua.exec(CLICK_BINDINGS_DEFAULTS_LUA)?;
69:     Ok(())
70: }
71: 
72: #[cfg(test)]
73: mod tests {
74:     use crate::lua_api::WowLuaEnv;
75: 

DECLARATION


## global api-C_Spell-GetMawPowerBorderAtlasBySpellID-067
SOURCE C_Spell.GetMawPowerBorderAtlasBySpellID

PROVIDER
src/c_api/c_spell_maw_powers.rs:25
23:         state,
24:         namespace,
25:         "GetMawPowerBorderAtlasBySpellID",
26:         get_maw_power_border_atlas_by_spell_id,
27:     )?;
28:     table_set_rust_fn_static(
29:         state,
30:         namespace,
31:         "GetMawPowerLinkBySpellID",
32:         get_maw_power_link_by_spell_id,
33:     )
34: }
35: 
36: fn get_maw_power_border_atlas_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
37:     let atlas = match super::c_spell::read_public_spell_identifier_at(
38:         state,
39:         1,
40:         "C_Spell.GetMawPowerBorderAtlasBySpellID",
41:     )? {
42:         Some(spell_id) => borrow_state(state)?
43:             .maw_powers
44:             .border_atlases
45:             .get(&spell_id)

DECLARATION


## global api-ConfirmReadyCheck-068
SOURCE ConfirmReadyCheck

PROVIDER


DECLARATION


## global api-DemoteAssistant-069
SOURCE DemoteAssistant

PROVIDER


DECLARATION


## global api-DoReadyCheck-070
SOURCE DoReadyCheck

PROVIDER


DECLARATION


## global api-GetMerchantCurrencies-071
SOURCE GetMerchantCurrencies

PROVIDER


DECLARATION


## global api-IsGUIDInGroup-072
SOURCE IsGUIDInGroup

PROVIDER


DECLARATION


## global api-PromoteToAssistant-073
SOURCE PromoteToAssistant

PROVIDER


DECLARATION


## global api-PromoteToLeader-074
SOURCE PromoteToLeader

PROVIDER


DECLARATION


## global api-SetEveryoneIsAssistant-075
SOURCE SetEveryoneIsAssistant

PROVIDER


DECLARATION


## global api-UninviteUnit-076
SOURCE UninviteUnit

PROVIDER
src/lua_api/globals/group_verbs.rs:234
232:     LuaApiMut::register_function(lua, "LeaveParty", leave_party)?;
233:     LuaApiMut::register_function(lua, "RemoveFromParty", remove_from_party)?;
234:     LuaApiMut::register_function(lua, "UninviteUnit", uninvite_unit)?;
235:     LuaApiMut::register_function(lua, "KickUnit", kick_unit)?;
236:     LuaApiMut::register_function(lua, "ReadyCheck", ready_check)?;
237:     LuaApiMut::register_function(lua, "GetReadyCheckStatus", get_ready_check_status)?;
238:     LuaApiMut::register_function(lua, "GetReadyCheckTimeLeft", get_ready_check_time_left)?;
239:     Ok(())
240: }

DECLARATION


## global api-GetAutoCompletePresenceID-077
SOURCE GetAutoCompletePresenceID

PROVIDER


DECLARATION


## global api-GetAutoCompleteResults-078
SOURCE GetAutoCompleteResults

PROVIDER
src/c_api/c_auto_complete.rs:41
39:         state,
40:         table_ref,
41:         "GetAutoCompleteResults",
42:         c_auto_complete_get_results,
43:     )
44: }
45: 
46: fn c_auto_complete_get_results(state: &mut LuaState) -> LuaResult<u32> {
47:     let query = String::from_stack(state, 1)?;
48:     let max_results = Option::<i32>::from_stack(state, 2)?.unwrap_or(0);
49:     let cursor_position = Option::<i32>::from_stack(state, 3)?.unwrap_or(query.len() as i32);
50:     let allow_full_match = Option::<bool>::from_stack(state, 4)?.unwrap_or(true);
51:     let include_flags = flags_from_stack(state, 5, FLAG_ALL)?;
52:     let exclude_flags = flags_from_stack(state, 6, 0)?;
53: 
54:     let search_text = search_text_at_cursor(&query, cursor_position);
55:     let candidates = collect_candidates(state)?;
56:     let results = filter_candidates(
57:         candidates,
58:         &search_text,
59:         max_results,
60:         allow_full_match,
61:         include_flags,

DECLARATION


## global api-GetAutoCompleteRealms-079
SOURCE GetAutoCompleteRealms

PROVIDER
src/lua_api/workarounds/temporary/auto_complete_defaults.rs:10
8: const AUTO_COMPLETE_DEFAULTS_LUA: &str = r#"
9: C_AutoComplete = C_AutoComplete or __wow_namespace()
10: if rawget(C_AutoComplete, "GetAutoCompleteRealms") == nil then
11:     function C_AutoComplete.GetAutoCompleteRealms()
12:         return {}
13:     end
14: end
15: if GetAutoCompleteRealms == nil then
16:     function GetAutoCompleteRealms()
17:         return C_AutoComplete.GetAutoCompleteRealms()
18:     end
19: end
20: if GetAutoCompleteResults == nil then
21:     function GetAutoCompleteResults(name, numResults, cursorPosition, allowFullMatch, includeFlags, excludeFlags)
22:         return C_AutoComplete.GetAutoCompleteResults(name, numResults, cursorPosition, not not allowFullMatch, includeFlags, excludeFlags)
23:     end
24: end
25: "#;
26: 
27: pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
28:     lua.exec(AUTO_COMPLETE_DEFAULTS_LUA)?;
29:     Ok(())
30: }

DECLARATION


## global api-IsRecognizedName-080
SOURCE IsRecognizedName

PROVIDER


DECLARATION


## source-context-082
SOURCE ScriptObjects Added (36):

PROVIDER


DECLARATION


## scriptobjects-DurationClock-GetTime-083
SOURCE DurationClock:GetTime

PROVIDER
src/lua_api/globals/lua_duration_object.rs:176
174:         state,
175:         clock,
176:         "GetTime",
177:         "ManualClock.GetTime",
178:         clock_get_time,
179:     );
180:     install_clock_method(
181:         state,
182:         clock,
183:         "SetTime",
184:         "ManualClock.SetTime",
185:         clock_set_time,
186:     );
187:     install_clock_method(
188:         state,
189:         clock,
190:         "AdvanceTime",
191:         "ManualClock.AdvanceTime",
192:         clock_advance_time,
193:     );
194:     install_clock_method(
195:         state,
196:         clock,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationClockAPIDocumentation.lua:11
9: 	{
10: 		{
11: 			Name = "GetTime",
12: 			Type = "Function",
13: 			Documentation = { "Returns the current time value represented by this clock." },
14: 
15: 			Arguments =
16: 			{
17: 			},
18: 
19: 			Returns =
20: 			{
21: 				{ Name = "time", Type = "FrameTime", Nilable = false },
22: 			},
23: 		},
24: 	},
25: 
26: 	Events =
27: 	{
28: 	},
29: 
30: 	Tables =
31: 	{
32: 	},
33: 	Predicates =
34: 	{
35: 	},
36: };
37: 

## scriptobjects-DurationManualClock-AdvanceTime-084
SOURCE DurationManualClock:AdvanceTime

PROVIDER
src/lua_api/globals/lua_duration_object.rs:190
188:         state,
189:         clock,
190:         "AdvanceTime",
191:         "ManualClock.AdvanceTime",
192:         clock_advance_time,
193:     );
194:     install_clock_method(
195:         state,
196:         clock,
197:         "RewindTime",
198:         "ManualClock.RewindTime",
199:         clock_rewind_time,
200:     );
201:     install_clock_method(
202:         state,
203:         clock,
204:         "ResetTime",
205:         "ManualClock.ResetTime",
206:         clock_reset_time,
207:     );
208:     state.push(clock);
209:     Ok(1)
210: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:11
9: 	{
10: 		{
11: 			Name = "AdvanceTime",
12: 			Type = "Function",
13: 			SecretArguments = "AllowedWhenUntainted",
14: 			Documentation = { "Advances the clock by a specified number of seconds." },
15: 
16: 			Arguments =
17: 			{
18: 				{ Name = "delta", Type = "DurationSeconds", Nilable = false },
19: 			},
20: 		},
21: 		{
22: 			Name = "ResetTime",
23: 			Type = "Function",
24: 			Documentation = { "Resets the clock to a zero time value." },
25: 
26: 			Arguments =
27: 			{
28: 			},
29: 		},
30: 		{
31: 			Name = "RewindTime",
32: 			Type = "Function",
33: 			SecretArguments = "AllowedWhenUntainted",
34: 			Documentation = { "Rewinds the clock by a specified number of seconds." },
35: 
36: 			Arguments =
37: 			{

## scriptobjects-DurationManualClock-ResetTime-085
SOURCE DurationManualClock:ResetTime

PROVIDER
src/lua_api/globals/lua_duration_object.rs:204
202:         state,
203:         clock,
204:         "ResetTime",
205:         "ManualClock.ResetTime",
206:         clock_reset_time,
207:     );
208:     state.push(clock);
209:     Ok(1)
210: }
211: 
212: /// `C_DurationUtil.GetCurrentTime()` — simulator elapsed time, matching GetTime.
213: fn get_current_time(state: &mut LuaState) -> LuaResult<u32> {
214:     let time = core::current_time(state)?;
215:     state.push(Val::Num(time));
216:     Ok(1)
217: }
218: 
219: fn clock_get_time(state: &mut LuaState) -> LuaResult<u32> {
220:     let clock = crate::lua_bridge::stack_val(state, 1);
221:     let time = clock_time(state, clock);
222:     state.push(time);
223:     Ok(1)
224: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:22
20: 		},
21: 		{
22: 			Name = "ResetTime",
23: 			Type = "Function",
24: 			Documentation = { "Resets the clock to a zero time value." },
25: 
26: 			Arguments =
27: 			{
28: 			},
29: 		},
30: 		{
31: 			Name = "RewindTime",
32: 			Type = "Function",
33: 			SecretArguments = "AllowedWhenUntainted",
34: 			Documentation = { "Rewinds the clock by a specified number of seconds." },
35: 
36: 			Arguments =
37: 			{
38: 				{ Name = "delta", Type = "DurationSeconds", Nilable = false },
39: 			},
40: 		},
41: 		{
42: 			Name = "SetTime",
43: 			Type = "Function",
44: 			SecretArguments = "AllowedWhenUntainted",
45: 			Documentation = { "Sets the current clock timestamp to a given value." },
46: 
47: 			Arguments =
48: 			{

## scriptobjects-DurationManualClock-RewindTime-086
SOURCE DurationManualClock:RewindTime

PROVIDER
src/lua_api/globals/lua_duration_object.rs:197
195:         state,
196:         clock,
197:         "RewindTime",
198:         "ManualClock.RewindTime",
199:         clock_rewind_time,
200:     );
201:     install_clock_method(
202:         state,
203:         clock,
204:         "ResetTime",
205:         "ManualClock.ResetTime",
206:         clock_reset_time,
207:     );
208:     state.push(clock);
209:     Ok(1)
210: }
211: 
212: /// `C_DurationUtil.GetCurrentTime()` — simulator elapsed time, matching GetTime.
213: fn get_current_time(state: &mut LuaState) -> LuaResult<u32> {
214:     let time = core::current_time(state)?;
215:     state.push(Val::Num(time));
216:     Ok(1)
217: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:31
29: 		},
30: 		{
31: 			Name = "RewindTime",
32: 			Type = "Function",
33: 			SecretArguments = "AllowedWhenUntainted",
34: 			Documentation = { "Rewinds the clock by a specified number of seconds." },
35: 
36: 			Arguments =
37: 			{
38: 				{ Name = "delta", Type = "DurationSeconds", Nilable = false },
39: 			},
40: 		},
41: 		{
42: 			Name = "SetTime",
43: 			Type = "Function",
44: 			SecretArguments = "AllowedWhenUntainted",
45: 			Documentation = { "Sets the current clock timestamp to a given value." },
46: 
47: 			Arguments =
48: 			{
49: 				{ Name = "time", Type = "FrameTime", Nilable = false },
50: 			},
51: 		},
52: 	},
53: 
54: 	Events =
55: 	{
56: 	},
57: 

## scriptobjects-DurationManualClock-SetTime-087
SOURCE DurationManualClock:SetTime

PROVIDER
src/lua_api/globals/lua_duration_object.rs:183
181:         state,
182:         clock,
183:         "SetTime",
184:         "ManualClock.SetTime",
185:         clock_set_time,
186:     );
187:     install_clock_method(
188:         state,
189:         clock,
190:         "AdvanceTime",
191:         "ManualClock.AdvanceTime",
192:         clock_advance_time,
193:     );
194:     install_clock_method(
195:         state,
196:         clock,
197:         "RewindTime",
198:         "ManualClock.RewindTime",
199:         clock_rewind_time,
200:     );
201:     install_clock_method(
202:         state,
203:         clock,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:42
40: 		},
41: 		{
42: 			Name = "SetTime",
43: 			Type = "Function",
44: 			SecretArguments = "AllowedWhenUntainted",
45: 			Documentation = { "Sets the current clock timestamp to a given value." },
46: 
47: 			Arguments =
48: 			{
49: 				{ Name = "time", Type = "FrameTime", Nilable = false },
50: 			},
51: 		},
52: 	},
53: 
54: 	Events =
55: 	{
56: 	},
57: 
58: 	Tables =
59: 	{
60: 	},
61: 	Predicates =
62: 	{
63: 	},
64: };
65: 
66: APIDocumentation:AddDocumentationTable(LuaDurationManualClockAPI);

## scriptobjects-DurationObject-GetClock-088
SOURCE DurationObject:GetClock

PROVIDER
src/lua_api/globals/lua_duration_object.rs:533
531:         methods,
532:         "GetClock",
533:         "LuaDurationObject.GetClock",
534:         m_get_clock,
535:     );
536:     install_method(
537:         state,
538:         methods,
539:         "HasSecretValues",
540:         "LuaDurationObject.HasSecretValues",
541:         m_has_secret_values,
542:     );
543: }
544: 
545: fn install_method(
546:     state: &mut LuaState,
547:     methods: Val,
548:     key: &'static str,
549:     closure_name: &'static str,
550:     func: rilua::RustFn,
551: ) {
552:     let closure = make_closure(state, closure_name, func);
553:     table_set_static(state, methods, key, closure);

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:181
179: 		},
180: 		{
181: 			Name = "GetClock",
182: 			Type = "Function",
183: 			Documentation = { "Returns the clock source used by this object." },
184: 
185: 			Arguments =
186: 			{
187: 			},
188: 
189: 			Returns =
190: 			{
191: 				{ Name = "clock", Type = "LuaDurationClock", Nilable = true, Documentation = { "If nil, the duration object is using an internal default clock source equivalent to GetTime()." } },
192: 			},
193: 		},
194: 		{
195: 			Name = "GetClockTime",
196: 			Type = "Function",
197: 			Documentation = { "Returns the current time of the clock source used by this object." },
198: 
199: 			Arguments =
200: 			{
201: 			},
202: 
203: 			Returns =
204: 			{
205: 				{ Name = "clockTime", Type = "FrameTime", Nilable = false },
206: 			},
207: 		},

## scriptobjects-DurationObject-HasExpired-089
SOURCE DurationObject:HasExpired

PROVIDER
src/lua_api/globals/lua_duration_object/core.rs:392
390:         ("IsZero", |s| query(s, Query::Zero)),
391:         ("HasStarted", |s| query(s, Query::Started)),
392:         ("HasExpired", |s| query(s, Query::Expired)),
393:         ("IsActive", |s| query(s, Query::Active)),
394:     ];
395:     for &(name, function) in methods_to_install {
396:         install_method(state, methods, name, name, function);
397:     }
398: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:335
333: 		},
334: 		{
335: 			Name = "HasExpired",
336: 			Type = "Function",
337: 			SecretArguments = "AllowedWhenUntainted",
338: 			Documentation = { "Returns true once the duration has reached its end time." },
339: 
340: 			Arguments =
341: 			{
342: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false, Default = "RealTime" },
343: 			},
344: 
345: 			Returns =
346: 			{
347: 				{ Name = "hasExpired", Type = "bool", Nilable = false },
348: 			},
349: 		},
350: 		{
351: 			Name = "HasSecretValues",
352: 			Type = "Function",
353: 			ReturnsNeverSecret = true,
354: 			Documentation = { "Returns true if the duration has been configured with any secret values." },
355: 
356: 			Arguments =
357: 			{
358: 			},
359: 
360: 			Returns =
361: 			{

## scriptobjects-DurationObject-HasStarted-090
SOURCE DurationObject:HasStarted

PROVIDER
src/lua_api/globals/lua_duration_object/core.rs:391
389:         ("GetClockTime", |s| query(s, Query::Clock)),
390:         ("IsZero", |s| query(s, Query::Zero)),
391:         ("HasStarted", |s| query(s, Query::Started)),
392:         ("HasExpired", |s| query(s, Query::Expired)),
393:         ("IsActive", |s| query(s, Query::Active)),
394:     ];
395:     for &(name, function) in methods_to_install {
396:         install_method(state, methods, name, name, function);
397:     }
398: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:366
364: 		},
365: 		{
366: 			Name = "HasStarted",
367: 			Type = "Function",
368: 			SecretArguments = "AllowedWhenUntainted",
369: 			Documentation = { "Returns true once the duration has reached its start time." },
370: 
371: 			Arguments =
372: 			{
373: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false, Default = "RealTime" },
374: 			},
375: 
376: 			Returns =
377: 			{
378: 				{ Name = "hasStarted", Type = "bool", Nilable = false },
379: 			},
380: 		},
381: 		{
382: 			Name = "IsActive",
383: 			Type = "Function",
384: 			SecretArguments = "AllowedWhenUntainted",
385: 			Documentation = { "Returns true while the duration is at or after its start time and before its end time." },
386: 
387: 			Arguments =
388: 			{
389: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false, Default = "RealTime" },
390: 			},
391: 
392: 			Returns =

## scriptobjects-DurationObject-IsActive-091
SOURCE DurationObject:IsActive

PROVIDER
src/lua_api/globals/lua_duration_object/core.rs:393
391:         ("HasStarted", |s| query(s, Query::Started)),
392:         ("HasExpired", |s| query(s, Query::Expired)),
393:         ("IsActive", |s| query(s, Query::Active)),
394:     ];
395:     for &(name, function) in methods_to_install {
396:         install_method(state, methods, name, name, function);
397:     }
398: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:382
380: 		},
381: 		{
382: 			Name = "IsActive",
383: 			Type = "Function",
384: 			SecretArguments = "AllowedWhenUntainted",
385: 			Documentation = { "Returns true while the duration is at or after its start time and before its end time." },
386: 
387: 			Arguments =
388: 			{
389: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false, Default = "RealTime" },
390: 			},
391: 
392: 			Returns =
393: 			{
394: 				{ Name = "isActive", Type = "bool", Nilable = false },
395: 			},
396: 		},
397: 		{
398: 			Name = "IsZero",
399: 			Type = "Function",
400: 			Documentation = { "Returns true if the duration object is measuring a zero duration time span." },
401: 
402: 			Arguments =
403: 			{
404: 			},
405: 
406: 			Returns =
407: 			{
408: 				{ Name = "isZero", Type = "bool", Nilable = false },

## scriptobjects-DurationObject-SetClock-092
SOURCE DurationObject:SetClock

PROVIDER
src/lua_api/globals/lua_duration_object.rs:522
520:         methods,
521:         "SetClock",
522:         "LuaDurationObject.SetClock",
523:         m_set_clock,
524:     );
525: }
526: 
527: /// Install duration query methods with local best-effort state.
528: fn install_query_methods(state: &mut LuaState, methods: Val) {
529:     install_method(
530:         state,
531:         methods,
532:         "GetClock",
533:         "LuaDurationObject.GetClock",
534:         m_get_clock,
535:     );
536:     install_method(
537:         state,
538:         methods,
539:         "HasSecretValues",
540:         "LuaDurationObject.HasSecretValues",
541:         m_has_secret_values,
542:     );

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:421
419: 		},
420: 		{
421: 			Name = "SetClock",
422: 			Type = "Function",
423: 			SecretArguments = "AllowedWhenUntainted",
424: 			Documentation = { "Configures the clock source used by this object." },
425: 
426: 			Arguments =
427: 			{
428: 				{ Name = "clock", Type = "LuaDurationClock", Nilable = true, Documentation = { "If nil, the duration object will use an internal default clock source equivalent to GetTime()." } },
429: 			},
430: 		},
431: 		{
432: 			Name = "SetTimeFromEnd",
433: 			Type = "Function",
434: 			SecretArguments = "AllowedWhenUntainted",
435: 			Documentation = { "Configures the duration object to represent an end time and a duration." },
436: 
437: 			Arguments =
438: 			{
439: 				{ Name = "endTime", Type = "FrameTime", Nilable = false },
440: 				{ Name = "duration", Type = "DurationSeconds", Nilable = false },
441: 				{ Name = "modRate", Type = "number", Nilable = false, Default = 1, Documentation = { "Optional divisor for converting this time span to a base time." } },
442: 			},
443: 		},
444: 		{
445: 			Name = "SetTimeFromStart",
446: 			Type = "Function",
447: 			SecretArguments = "AllowedWhenUntainted",

## scriptobjects-DurationTextBinding-CanFormatText-093
SOURCE DurationTextBinding:CanFormatText

PROVIDER
src/c_api/duration_text_binding.rs:155
153:             return copy
154:         end
155:         function binding:CanFormatText() return true end
156:         function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
157:         function binding:Disable() self:SetEnabled(false) end
158:         function binding:Enable() self:SetEnabled(true) end
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:22
20: 		},
21: 		{
22: 			Name = "CanFormatText",
23: 			Type = "Function",
24: 			Documentation = { "Returns true if this binding has enough configuration to produce formatted text." },
25: 
26: 			Arguments =
27: 			{
28: 			},
29: 
30: 			Returns =
31: 			{
32: 				{ Name = "canFormatText", Type = "bool", Nilable = false },
33: 			},
34: 		},
35: 		{
36: 			Name = "CanUpdateFontString",
37: 			Type = "Function",
38: 			Documentation = { "Returns true if this binding has enough configuration to update its font string with formatted text." },
39: 
40: 			Arguments =
41: 			{
42: 			},
43: 
44: 			Returns =
45: 			{
46: 				{ Name = "canUpdateText", Type = "bool", Nilable = false },
47: 			},
48: 		},

## scriptobjects-DurationTextBinding-CanUpdateFontString-094
SOURCE DurationTextBinding:CanUpdateFontString

PROVIDER
src/c_api/duration_text_binding.rs:156
154:         end
155:         function binding:CanFormatText() return true end
156:         function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
157:         function binding:Disable() self:SetEnabled(false) end
158:         function binding:Enable() self:SetEnabled(true) end
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:36
34: 		},
35: 		{
36: 			Name = "CanUpdateFontString",
37: 			Type = "Function",
38: 			Documentation = { "Returns true if this binding has enough configuration to update its font string with formatted text." },
39: 
40: 			Arguments =
41: 			{
42: 			},
43: 
44: 			Returns =
45: 			{
46: 				{ Name = "canUpdateText", Type = "bool", Nilable = false },
47: 			},
48: 		},
49: 		{
50: 			Name = "ClearTextColorCurve",
51: 			Type = "Function",
52: 			Documentation = { "Clears the text color curve used by this binding." },
53: 
54: 			Arguments =
55: 			{
56: 			},
57: 		},
58: 		{
59: 			Name = "Copy",
60: 			Type = "Function",
61: 			ReturnsNeverSecret = true,
62: 			Documentation = { "Returns a copy of this duration text binding." },

## scriptobjects-DurationTextBinding-Disable-095
SOURCE DurationTextBinding:Disable

PROVIDER
src/c_api/duration_text_binding.rs:157
155:         function binding:CanFormatText() return true end
156:         function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
157:         function binding:Disable() self:SetEnabled(false) end
158:         function binding:Enable() self:SetEnabled(true) end
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:74
72: 		},
73: 		{
74: 			Name = "Disable",
75: 			Type = "Function",
76: 			Documentation = { "Disables automatic updates for this duration text binding." },
77: 
78: 			Arguments =
79: 			{
80: 			},
81: 		},
82: 		{
83: 			Name = "Enable",
84: 			Type = "Function",
85: 			Documentation = { "Enables automatic updates for this duration text binding." },
86: 
87: 			Arguments =
88: 			{
89: 			},
90: 		},
91: 		{
92: 			Name = "GetDuration",
93: 			Type = "Function",
94: 			Documentation = { "Returns the duration object used by this duration text binding." },
95: 
96: 			Arguments =
97: 			{
98: 			},
99: 
100: 			Returns =

## scriptobjects-DurationTextBinding-Enable-096
SOURCE DurationTextBinding:Enable

PROVIDER
src/c_api/duration_text_binding.rs:158
156:         function binding:CanUpdateFontString() return self.fontString ~= nil and type(self.fontString.SetText) == "function" end
157:         function binding:Disable() self:SetEnabled(false) end
158:         function binding:Enable() self:SetEnabled(true) end
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)
178:                 if ok then text = value end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:83
81: 		},
82: 		{
83: 			Name = "Enable",
84: 			Type = "Function",
85: 			Documentation = { "Enables automatic updates for this duration text binding." },
86: 
87: 			Arguments =
88: 			{
89: 			},
90: 		},
91: 		{
92: 			Name = "GetDuration",
93: 			Type = "Function",
94: 			Documentation = { "Returns the duration object used by this duration text binding." },
95: 
96: 			Arguments =
97: 			{
98: 			},
99: 
100: 			Returns =
101: 			{
102: 				{ Name = "duration", Type = "LuaDurationObject", Nilable = true },
103: 			},
104: 		},
105: 		{
106: 			Name = "GetExpiredText",
107: 			Type = "Function",
108: 			Documentation = { "Returns the text shown when the duration has fully expired." },
109: 

## scriptobjects-DurationTextBinding-GetDuration-097
SOURCE DurationTextBinding:GetDuration

PROVIDER
src/c_api/duration_text_binding.rs:160
158:         function binding:Enable() self:SetEnabled(true) end
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)
178:                 if ok then text = value end
179:             end
180:             return text

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:92
90: 		},
91: 		{
92: 			Name = "GetDuration",
93: 			Type = "Function",
94: 			Documentation = { "Returns the duration object used by this duration text binding." },
95: 
96: 			Arguments =
97: 			{
98: 			},
99: 
100: 			Returns =
101: 			{
102: 				{ Name = "duration", Type = "LuaDurationObject", Nilable = true },
103: 			},
104: 		},
105: 		{
106: 			Name = "GetExpiredText",
107: 			Type = "Function",
108: 			Documentation = { "Returns the text shown when the duration has fully expired." },
109: 
110: 			Arguments =
111: 			{
112: 			},
113: 
114: 			Returns =
115: 			{
116: 				{ Name = "text", Type = "string", Nilable = true },
117: 			},
118: 		},

## scriptobjects-DurationTextBinding-GetExpiredText-098
SOURCE DurationTextBinding:GetExpiredText

PROVIDER
src/c_api/duration_text_binding.rs:161
159:         function binding:GetClock() return self.clock end
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)
178:                 if ok then text = value end
179:             end
180:             return text
181:         end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:106
104: 		},
105: 		{
106: 			Name = "GetExpiredText",
107: 			Type = "Function",
108: 			Documentation = { "Returns the text shown when the duration has fully expired." },
109: 
110: 			Arguments =
111: 			{
112: 			},
113: 
114: 			Returns =
115: 			{
116: 				{ Name = "text", Type = "string", Nilable = true },
117: 			},
118: 		},
119: 		{
120: 			Name = "GetFontString",
121: 			Type = "Function",
122: 			Documentation = { "Returns the font string updated by this duration text binding." },
123: 
124: 			Arguments =
125: 			{
126: 			},
127: 
128: 			Returns =
129: 			{
130: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = true },
131: 			},
132: 		},

## scriptobjects-DurationTextBinding-GetFontString-099
SOURCE DurationTextBinding:GetFontString

PROVIDER
src/c_api/duration_text_binding.rs:162
160:         function binding:GetDuration() return self.duration end
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)
178:                 if ok then text = value end
179:             end
180:             return text
181:         end
182:         function binding:GetTimeModifier() return self.timeModifier end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:120
118: 		},
119: 		{
120: 			Name = "GetFontString",
121: 			Type = "Function",
122: 			Documentation = { "Returns the font string updated by this duration text binding." },
123: 
124: 			Arguments =
125: 			{
126: 			},
127: 
128: 			Returns =
129: 			{
130: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = true },
131: 			},
132: 		},
133: 		{
134: 			Name = "GetFormattedText",
135: 			Type = "Function",
136: 			Documentation = { "Returns the text that would currently be assigned to the configured font string." },
137: 
138: 			Arguments =
139: 			{
140: 			},
141: 
142: 			Returns =
143: 			{
144: 				{ Name = "text", Type = "string", Nilable = false, ConditionalSecret = true },
145: 			},
146: 		},

## scriptobjects-DurationTextBinding-GetFormattedText-100
SOURCE DurationTextBinding:GetFormattedText

PROVIDER
src/c_api/duration_text_binding.rs:163
161:         function binding:GetExpiredText() return self.expiredText end
162:         function binding:GetFontString() return self.fontString end
163:         function binding:GetFormattedText()
164:             local duration = self.duration
165:             if hasSecretInput(duration) then return format_secret_duration(self, duration) end
166:             local text = duration_value_to_text(duration)
167:             if type(self.formatter) == "userdata" and type(self.formatter.FormatNumber) == "function" then
168:                 text = self.formatter:FormatNumber(tonumber(text))
169:             elseif type(self.formatter) == "function" then
170:                 local ok, value = pcall(self.formatter, self.duration)
171:                 if ok and value ~= nil then text = tostring(value) end
172:             elseif type(self.formatter) == "table" and type(self.formatter.Format) == "function" then
173:                 local ok, value = pcall(self.formatter.Format, self.formatter, self.duration)
174:                 if ok and value ~= nil then text = tostring(value) end
175:             end
176:             if type(self.textFormat) == "string" and self.textFormat ~= "" then
177:                 local ok, value = pcall(string.format, self.textFormat, text)
178:                 if ok then text = value end
179:             end
180:             return text
181:         end
182:         function binding:GetTimeModifier() return self.timeModifier end
183:         function binding:GetUpdateInterval() return self.updateInterval end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:134
132: 		},
133: 		{
134: 			Name = "GetFormattedText",
135: 			Type = "Function",
136: 			Documentation = { "Returns the text that would currently be assigned to the configured font string." },
137: 
138: 			Arguments =
139: 			{
140: 			},
141: 
142: 			Returns =
143: 			{
144: 				{ Name = "text", Type = "string", Nilable = false, ConditionalSecret = true },
145: 			},
146: 		},
147: 		{
148: 			Name = "GetFormattedTextColor",
149: 			Type = "Function",
150: 			MayReturnNothing = true,
151: 			Documentation = { "Returns the text color that would currently be assigned to the configured font string." },
152: 
153: 			Arguments =
154: 			{
155: 			},
156: 
157: 			Returns =
158: 			{
159: 				{ Name = "color", Type = "colorRGBA", Mixin = "ColorMixin", Nilable = false, ConditionalSecret = true },
160: 			},

## scriptobjects-DurationTextBinding-GetTimeModifier-101
SOURCE DurationTextBinding:GetTimeModifier

PROVIDER
src/c_api/duration_text_binding.rs:182
180:             return text
181:         end
182:         function binding:GetTimeModifier() return self.timeModifier end
183:         function binding:GetUpdateInterval() return self.updateInterval end
184:         function binding:GetZeroDurationText() return self.zeroDurationText end
185:         function binding:HasExpired() return type(self.duration) == "number" and self.duration <= 0 end
186:         function binding:HasSecretValues() return hasSecretInput(self.duration) end
187:         function binding:HasStarted() return true end
188:         function binding:IsActive() return self.enabled end
189:         function binding:IsEnabled() return self.enabled end
190:         function binding:SetClock(clock) self.clock = clock end
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:179
177: 		},
178: 		{
179: 			Name = "GetTimeModifier",
180: 			Type = "Function",
181: 			Documentation = { "Returns the time modifier used when sampling duration values for this binding." },
182: 
183: 			Arguments =
184: 			{
185: 			},
186: 
187: 			Returns =
188: 			{
189: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false },
190: 			},
191: 		},
192: 		{
193: 			Name = "GetUpdateInterval",
194: 			Type = "Function",
195: 			Documentation = { "Returns the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },
196: 
197: 			Arguments =
198: 			{
199: 			},
200: 
201: 			Returns =
202: 			{
203: 				{ Name = "updateInterval", Type = "number", Nilable = false },
204: 			},
205: 		},

## scriptobjects-DurationTextBinding-GetUpdateInterval-102
SOURCE DurationTextBinding:GetUpdateInterval

PROVIDER
src/c_api/duration_text_binding.rs:183
181:         end
182:         function binding:GetTimeModifier() return self.timeModifier end
183:         function binding:GetUpdateInterval() return self.updateInterval end
184:         function binding:GetZeroDurationText() return self.zeroDurationText end
185:         function binding:HasExpired() return type(self.duration) == "number" and self.duration <= 0 end
186:         function binding:HasSecretValues() return hasSecretInput(self.duration) end
187:         function binding:HasStarted() return true end
188:         function binding:IsActive() return self.enabled end
189:         function binding:IsEnabled() return self.enabled end
190:         function binding:SetClock(clock) self.clock = clock end
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:193
191: 		},
192: 		{
193: 			Name = "GetUpdateInterval",
194: 			Type = "Function",
195: 			Documentation = { "Returns the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },
196: 
197: 			Arguments =
198: 			{
199: 			},
200: 
201: 			Returns =
202: 			{
203: 				{ Name = "updateInterval", Type = "number", Nilable = false },
204: 			},
205: 		},
206: 		{
207: 			Name = "GetZeroDurationText",
208: 			Type = "Function",
209: 			Documentation = { "Returns the text shown when the duration is not configured, or represents a zero-duration time span." },
210: 
211: 			Arguments =
212: 			{
213: 			},
214: 
215: 			Returns =
216: 			{
217: 				{ Name = "text", Type = "string", Nilable = true },
218: 			},
219: 		},

## scriptobjects-DurationTextBinding-GetZeroDurationText-103
SOURCE DurationTextBinding:GetZeroDurationText

PROVIDER
src/c_api/duration_text_binding.rs:184
182:         function binding:GetTimeModifier() return self.timeModifier end
183:         function binding:GetUpdateInterval() return self.updateInterval end
184:         function binding:GetZeroDurationText() return self.zeroDurationText end
185:         function binding:HasExpired() return type(self.duration) == "number" and self.duration <= 0 end
186:         function binding:HasSecretValues() return hasSecretInput(self.duration) end
187:         function binding:HasStarted() return true end
188:         function binding:IsActive() return self.enabled end
189:         function binding:IsEnabled() return self.enabled end
190:         function binding:SetClock(clock) self.clock = clock end
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:207
205: 		},
206: 		{
207: 			Name = "GetZeroDurationText",
208: 			Type = "Function",
209: 			Documentation = { "Returns the text shown when the duration is not configured, or represents a zero-duration time span." },
210: 
211: 			Arguments =
212: 			{
213: 			},
214: 
215: 			Returns =
216: 			{
217: 				{ Name = "text", Type = "string", Nilable = true },
218: 			},
219: 		},
220: 		{
221: 			Name = "HasSecretValues",
222: 			Type = "Function",
223: 			ReturnsNeverSecret = true,
224: 			Documentation = { "Returns true if the duration text binding has been configured with any secret values." },
225: 
226: 			Arguments =
227: 			{
228: 			},
229: 
230: 			Returns =
231: 			{
232: 				{ Name = "hasSecretValues", Type = "bool", Nilable = false },
233: 			},

## scriptobjects-DurationTextBinding-IsEnabled-104
SOURCE DurationTextBinding:IsEnabled

PROVIDER
src/c_api/duration_text_binding.rs:189
187:         function binding:HasStarted() return true end
188:         function binding:IsActive() return self.enabled end
189:         function binding:IsEnabled() return self.enabled end
190:         function binding:SetClock(clock) self.clock = clock end
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1
205:             self.timeModifier = 0
206:             self.expiredText = nil
207:             self.zeroDurationText = nil
208:             self.formatter = nil
209:             self.textFormat = nil

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:236
234: 		},
235: 		{
236: 			Name = "IsEnabled",
237: 			Type = "Function",
238: 			Documentation = { "Returns true if this duration text binding updates its font string automatically." },
239: 
240: 			Arguments =
241: 			{
242: 			},
243: 
244: 			Returns =
245: 			{
246: 				{ Name = "enabled", Type = "bool", Nilable = false },
247: 			},
248: 		},
249: 		{
250: 			Name = "SetDuration",
251: 			Type = "Function",
252: 			SecretArguments = "AllowedWhenUntainted",
253: 			Documentation = { "Configures the duration object used by this duration text binding." },
254: 
255: 			Arguments =
256: 			{
257: 				{ Name = "duration", Type = "LuaDurationObject", Nilable = false },
258: 			},
259: 		},
260: 		{
261: 			Name = "SetEnabled",
262: 			Type = "Function",

## scriptobjects-DurationTextBinding-SetDuration-105
SOURCE DurationTextBinding:SetDuration

PROVIDER
src/c_api/duration_text_binding.rs:191
189:         function binding:IsEnabled() return self.enabled end
190:         function binding:SetClock(clock) self.clock = clock end
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1
205:             self.timeModifier = 0
206:             self.expiredText = nil
207:             self.zeroDurationText = nil
208:             self.formatter = nil
209:             self.textFormat = nil
210:             self.textFormatComponents = nil
211:             self.clock = create_duration_clock(0)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:250
248: 		},
249: 		{
250: 			Name = "SetDuration",
251: 			Type = "Function",
252: 			SecretArguments = "AllowedWhenUntainted",
253: 			Documentation = { "Configures the duration object used by this duration text binding." },
254: 
255: 			Arguments =
256: 			{
257: 				{ Name = "duration", Type = "LuaDurationObject", Nilable = false },
258: 			},
259: 		},
260: 		{
261: 			Name = "SetEnabled",
262: 			Type = "Function",
263: 			SecretArguments = "AllowedWhenUntainted",
264: 			Documentation = { "Configures whether this duration text binding updates its font string automatically." },
265: 
266: 			Arguments =
267: 			{
268: 				{ Name = "enabled", Type = "bool", Nilable = false },
269: 			},
270: 		},
271: 		{
272: 			Name = "SetExpiredText",
273: 			Type = "Function",
274: 			SecretArguments = "AllowedWhenUntainted",
275: 			Documentation = { "Configures the text shown when the duration has fully expired." },
276: 

## scriptobjects-DurationTextBinding-SetExpiredText-106
SOURCE DurationTextBinding:SetExpiredText

PROVIDER
src/c_api/duration_text_binding.rs:193
191:         function binding:SetDuration(value) self.duration = value ~= nil and value or create_duration_value(0) end
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1
205:             self.timeModifier = 0
206:             self.expiredText = nil
207:             self.zeroDurationText = nil
208:             self.formatter = nil
209:             self.textFormat = nil
210:             self.textFormatComponents = nil
211:             self.clock = create_duration_clock(0)
212:         end
213:         function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:272
270: 		},
271: 		{
272: 			Name = "SetExpiredText",
273: 			Type = "Function",
274: 			SecretArguments = "AllowedWhenUntainted",
275: 			Documentation = { "Configures the text shown when the duration has fully expired." },
276: 
277: 			Arguments =
278: 			{
279: 				{ Name = "text", Type = "string", Nilable = true },
280: 			},
281: 		},
282: 		{
283: 			Name = "SetFontString",
284: 			Type = "Function",
285: 			SecretArguments = "AllowedWhenUntainted",
286: 			Documentation = { "Configures the font string updated by this duration text binding." },
287: 
288: 			Arguments =
289: 			{
290: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = false },
291: 			},
292: 		},
293: 		{
294: 			Name = "SetFormatter",
295: 			Type = "Function",
296: 			SecretArguments = "AllowedWhenUntainted",
297: 			Documentation = { "Configures the text format used by this duration text binding to display the remaining duration using the supplied formatter." },
298: 

## scriptobjects-DurationTextBinding-SetFontString-107
SOURCE DurationTextBinding:SetFontString

PROVIDER
src/c_api/duration_text_binding.rs:194
192:         function binding:SetEnabled(value) self.enabled = not not value end
193:         function binding:SetExpiredText(text) self.expiredText = text end
194:         function binding:SetFontString(value) self.fontString = value end
195:         function binding:SetFormatter(formatter) self.formatter = formatter end
196:         function binding:SetTextFormat(format, components)
197:             self.textFormat = format
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1
205:             self.timeModifier = 0
206:             self.expiredText = nil
207:             self.zeroDurationText = nil
208:             self.formatter = nil
209:             self.textFormat = nil
210:             self.textFormatComponents = nil
211:             self.clock = create_duration_clock(0)
212:         end
213:         function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end
214:         function binding:SetZeroDurationText(text) self.zeroDurationText = text end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:283
281: 		},
282: 		{
283: 			Name = "SetFontString",
284: 			Type = "Function",
285: 			SecretArguments = "AllowedWhenUntainted",
286: 			Documentation = { "Configures the font string updated by this duration text binding." },
287: 
288: 			Arguments =
289: 			{
290: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = false },
291: 			},
292: 		},
293: 		{
294: 			Name = "SetFormatter",
295: 			Type = "Function",
296: 			SecretArguments = "AllowedWhenUntainted",
297: 			Documentation = { "Configures the text format used by this duration text binding to display the remaining duration using the supplied formatter." },
298: 
299: 			Arguments =
300: 			{
301: 				{ Name = "formatter", Type = "NumericFormatter", Nilable = false },
302: 			},
303: 		},
304: 		{
305: 			Name = "SetTextColorCurve",
306: 			Type = "Function",
307: 			SecretArguments = "AllowedWhenUntainted",
308: 			Documentation = { "Configures this duration text binding to adjust fontstring text color by evaluating a duration property through a curve." },
309: 

## scriptobjects-DurationTextBinding-SetTimeModifier-108
SOURCE DurationTextBinding:SetTimeModifier

PROVIDER
src/c_api/duration_text_binding.rs:200
198:             self.textFormatComponents = components
199:         end
200:         function binding:SetTimeModifier(value) self.timeModifier = value or 0 end
201:         function binding:SetToDefaults()
202:             self.duration = create_duration_value(0)
203:             self.enabled = true
204:             self.updateInterval = 1
205:             self.timeModifier = 0
206:             self.expiredText = nil
207:             self.zeroDurationText = nil
208:             self.formatter = nil
209:             self.textFormat = nil
210:             self.textFormatComponents = nil
211:             self.clock = create_duration_clock(0)
212:         end
213:         function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end
214:         function binding:SetZeroDurationText(text) self.zeroDurationText = text end
215:         function binding:UpdateFontString()
216:             if self:CanUpdateFontString() then
217:                 local secret = hasSecretInput(self.duration)
218:                 local text = self:GetFormattedText()
219:                 if secret then text = wrapSecretOutput(text) end
220:                 self.fontString:SetText(text)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:329
327: 		},
328: 		{
329: 			Name = "SetTimeModifier",
330: 			Type = "Function",
331: 			SecretArguments = "AllowedWhenUntainted",
332: 			Documentation = { "Configures the time modifier used when sampling duration values for this binding." },
333: 
334: 			Arguments =
335: 			{
336: 				{ Name = "modifier", Type = "DurationTimeModifier", Nilable = false },
337: 			},
338: 		},
339: 		{
340: 			Name = "SetToDefaults",
341: 			Type = "Function",
342: 			Documentation = { "Resets this duration text binding to its default state, clearing the configured font string, duration, format, formatter, and fallback text." },
343: 
344: 			Arguments =
345: 			{
346: 			},
347: 		},
348: 		{
349: 			Name = "SetUpdateInterval",
350: 			Type = "Function",
351: 			SecretArguments = "AllowedWhenUntainted",
352: 			Documentation = { "Configures the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },
353: 
354: 			Arguments =
355: 			{

## scriptobjects-DurationTextBinding-SetUpdateInterval-109
SOURCE DurationTextBinding:SetUpdateInterval

PROVIDER
src/c_api/duration_text_binding.rs:213
211:             self.clock = create_duration_clock(0)
212:         end
213:         function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end
214:         function binding:SetZeroDurationText(text) self.zeroDurationText = text end
215:         function binding:UpdateFontString()
216:             if self:CanUpdateFontString() then
217:                 local secret = hasSecretInput(self.duration)
218:                 local text = self:GetFormattedText()
219:                 if secret then text = wrapSecretOutput(text) end
220:                 self.fontString:SetText(text)
221:             end
222:         end
223:         if isPatch121 then
224:             function binding:ClearTextColorCurve()
225:                 self.textColorCurve = nil
226:                 self.textColorProperty = nil
227:             end
228:             function binding:GetFormattedTextColor() return 1, 1, 1, 1 end
229:             function binding:GetTextColorCurve() return self.textColorCurve, self.textColorProperty end
230:             function binding:SetTextColorCurve(curve, property)
231:                 self.textColorCurve = curve
232:                 self.textColorProperty = property
233:             end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:349
347: 		},
348: 		{
349: 			Name = "SetUpdateInterval",
350: 			Type = "Function",
351: 			SecretArguments = "AllowedWhenUntainted",
352: 			Documentation = { "Configures the minimum number of seconds between automatic text updates. A value of zero updates every game tick." },
353: 
354: 			Arguments =
355: 			{
356: 				{ Name = "updateInterval", Type = "number", Nilable = false },
357: 			},
358: 		},
359: 		{
360: 			Name = "SetZeroDurationText",
361: 			Type = "Function",
362: 			SecretArguments = "AllowedWhenUntainted",
363: 			Documentation = { "Configures the text shown when the duration is not configured, or represents a zero-duration time span." },
364: 
365: 			Arguments =
366: 			{
367: 				{ Name = "text", Type = "string", Nilable = true },
368: 			},
369: 		},
370: 		{
371: 			Name = "UpdateFontString",
372: 			Type = "Function",
373: 			Documentation = { "Immediately updates the configured font string from the current duration state." },
374: 
375: 			Arguments =

## scriptobjects-DurationTextBinding-SetZeroDurationText-110
SOURCE DurationTextBinding:SetZeroDurationText

PROVIDER
src/c_api/duration_text_binding.rs:214
212:         end
213:         function binding:SetUpdateInterval(value) self.updateInterval = value or 1 end
214:         function binding:SetZeroDurationText(text) self.zeroDurationText = text end
215:         function binding:UpdateFontString()
216:             if self:CanUpdateFontString() then
217:                 local secret = hasSecretInput(self.duration)
218:                 local text = self:GetFormattedText()
219:                 if secret then text = wrapSecretOutput(text) end
220:                 self.fontString:SetText(text)
221:             end
222:         end
223:         if isPatch121 then
224:             function binding:ClearTextColorCurve()
225:                 self.textColorCurve = nil
226:                 self.textColorProperty = nil
227:             end
228:             function binding:GetFormattedTextColor() return 1, 1, 1, 1 end
229:             function binding:GetTextColorCurve() return self.textColorCurve, self.textColorProperty end
230:             function binding:SetTextColorCurve(curve, property)
231:                 self.textColorCurve = curve
232:                 self.textColorProperty = property
233:             end
234:         end

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:360
358: 		},
359: 		{
360: 			Name = "SetZeroDurationText",
361: 			Type = "Function",
362: 			SecretArguments = "AllowedWhenUntainted",
363: 			Documentation = { "Configures the text shown when the duration is not configured, or represents a zero-duration time span." },
364: 
365: 			Arguments =
366: 			{
367: 				{ Name = "text", Type = "string", Nilable = true },
368: 			},
369: 		},
370: 		{
371: 			Name = "UpdateFontString",
372: 			Type = "Function",
373: 			Documentation = { "Immediately updates the configured font string from the current duration state." },
374: 
375: 			Arguments =
376: 			{
377: 			},
378: 		},
379: 	},
380: 
381: 	Events =
382: 	{
383: 	},
384: 
385: 	Tables =
386: 	{

## scriptobjects-DurationTextFormattingOptions-GetAddRemainingText-111
SOURCE DurationTextFormattingOptions:GetAddRemainingText

PROVIDER


DECLARATION


## scriptobjects-DurationTextFormattingOptions-GetDurationType-112
SOURCE DurationTextFormattingOptions:GetDurationType

PROVIDER


DECLARATION


## scriptobjects-DurationTextFormattingOptions-SetAddRemainingText-113
SOURCE DurationTextFormattingOptions:SetAddRemainingText

PROVIDER


DECLARATION


## scriptobjects-DurationTextFormattingOptions-SetDurationType-114
SOURCE DurationTextFormattingOptions:SetDurationType

PROVIDER


DECLARATION


## scriptobjects-DurationTextRawValue-GetMilliseconds-115
SOURCE DurationTextRawValue:GetMilliseconds

PROVIDER


DECLARATION


## scriptobjects-DurationTextRawValue-GetSeconds-116
SOURCE DurationTextRawValue:GetSeconds

PROVIDER


DECLARATION


## scriptobjects-DurationTextRawValue-SetMilliseconds-117
SOURCE DurationTextRawValue:SetMilliseconds

PROVIDER


DECLARATION


## scriptobjects-DurationTextRawValue-SetSeconds-118
SOURCE DurationTextRawValue:SetSeconds

PROVIDER


DECLARATION


## source-context-120
SOURCE Widgets Removed (6):

PROVIDER


DECLARATION


## widgets-Minimap-SetBlipTexture-121
SOURCE Minimap:SetBlipTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:62
60:     method!("StartPlayerPing", start_player_ping),
61:     method!("StopPlayerPing", stop_player_ping),
62:     method!("SetBlipTexture", set_blip_texture),
63:     method!("SetMaskTexture", set_minimap_mask_texture),
64:     method!("SetIconTexture", set_minimap_icon_texture),
65:     method!("SetPOIArrowTexture", set_poi_arrow_texture),
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),

DECLARATION


## widgets-Minimap-SetCorpsePOIArrowTexture-122
SOURCE Minimap:SetCorpsePOIArrowTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:66
64:     method!("SetIconTexture", set_minimap_icon_texture),
65:     method!("SetPOIArrowTexture", set_poi_arrow_texture),
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
83:     method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
84:     method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),
85:     method!("SetArchBlobOutsideTexture", set_arch_blob_outside_texture),
86:     method!("SetArchBlobOutsideAlpha", set_arch_blob_outside_alpha),

DECLARATION


## widgets-Minimap-SetIconTexture-123
SOURCE Minimap:SetIconTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:64
62:     method!("SetBlipTexture", set_blip_texture),
63:     method!("SetMaskTexture", set_minimap_mask_texture),
64:     method!("SetIconTexture", set_minimap_icon_texture),
65:     method!("SetPOIArrowTexture", set_poi_arrow_texture),
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
83:     method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
84:     method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),

DECLARATION


## widgets-Minimap-SetPOIArrowTexture-124
SOURCE Minimap:SetPOIArrowTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:65
63:     method!("SetMaskTexture", set_minimap_mask_texture),
64:     method!("SetIconTexture", set_minimap_icon_texture),
65:     method!("SetPOIArrowTexture", set_poi_arrow_texture),
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
83:     method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
84:     method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),
85:     method!("SetArchBlobOutsideTexture", set_arch_blob_outside_texture),

DECLARATION


## widgets-Minimap-SetPlayerTexture-125
SOURCE Minimap:SetPlayerTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:68
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
83:     method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
84:     method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),
85:     method!("SetArchBlobOutsideTexture", set_arch_blob_outside_texture),
86:     method!("SetArchBlobOutsideAlpha", set_arch_blob_outside_alpha),
87:     method!("SetArchBlobRingTexture", set_arch_blob_ring_texture),
88:     method!("SetArchBlobRingAlpha", set_arch_blob_ring_alpha),

DECLARATION


## widgets-Minimap-SetStaticPOIArrowTexture-126
SOURCE Minimap:SetStaticPOIArrowTexture

PROVIDER
src/lua_api/frame/methods/map_frames.rs:67
65:     method!("SetPOIArrowTexture", set_poi_arrow_texture),
66:     method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
67:     method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
68:     method!("SetPlayerTexture", set_minimap_player_texture),
69:     method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
70:     method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
71:     method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
72:     method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
73:     method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
74:     method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
75:     method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
76:     method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
77:     method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
78:     method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
79:     method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
80:     method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
81:     method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
82:     method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
83:     method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
84:     method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),
85:     method!("SetArchBlobOutsideTexture", set_arch_blob_outside_texture),
86:     method!("SetArchBlobOutsideAlpha", set_arch_blob_outside_alpha),
87:     method!("SetArchBlobRingTexture", set_arch_blob_ring_texture),

DECLARATION


## source-context-128
SOURCE Widget Changes:

PROVIDER


DECLARATION


## widgets-Button-GetButtonState-129
SOURCE Button:GetButtonState + SecretReturnsForAspect

PROVIDER
src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:187
185: }
186: 
187: pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
188:     let id = frame_id_from_stack(state, 1)?;
189:     let pushed = {
190:         let sim = borrow_state(state)?;
191:         sim.widgets
192:             .get(id)
193:             .map(|frame| frame.button_state == 1)
194:             .unwrap_or(false)
195:     };
196:     let name = if pushed { "PUSHED" } else { "NORMAL" };
197:     let name_val = create_string(state, name);
198:     state.push(name_val);
199:     Ok(1)
200: }
201: 
202: pub(super) fn is_down(state: &mut LuaState) -> LuaResult<u32> {
203:     let id = frame_id_from_stack(state, 1)?;
204:     let is_down = {
205:         let sim = borrow_state(state)?;
206:         sim.widgets
207:             .get(id)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:72
70: 		},
71: 		{
72: 			Name = "GetButtonState",
73: 			Type = "Function",
74: 			SecretReturnsForAspect = { Enum.SecretAspect.ButtonState },
75: 
76: 			Arguments =
77: 			{
78: 			},
79: 
80: 			Returns =
81: 			{
82: 				{ Name = "buttonState", Type = "SimpleButtonStateToken", Nilable = false },
83: 			},
84: 		},
85: 		{
86: 			Name = "GetDisabledFontObject",
87: 			Type = "Function",
88: 
89: 			Arguments =
90: 			{
91: 			},
92: 
93: 			Returns =
94: 			{
95: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
96: 			},
97: 		},
98: 		{

## widgets-Button-IsEnabled-130
SOURCE Button:IsEnabled + SecretReturnsForAspect

PROVIDER
src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:72
70: // ── Button methods ────────────────────────────────────────────────────────────
71: 
72: pub(super) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
73:     let id = frame_id_from_stack(state, 1)?;
74:     let enabled = {
75:         let sim = borrow_state(state)?;
76:         sim.widgets.get(id).map(button_enabled).unwrap_or(true)
77:     };
78:     state.push(Val::Bool(enabled));
79:     Ok(1)
80: }
81: 
82: /// Fire all registered `OnEnable` or `OnDisable` bindings in order.
83: fn fire_enable_disable_script(state: &mut LuaState, id: u64, enabled: bool) -> LuaResult<()> {
84:     let handler_name = if enabled { "OnEnable" } else { "OnDisable" };
85:     let handlers = get_scripts_for_dispatch(state, id, handler_name);
86:     if handlers.is_empty() {
87:         return Ok(());
88:     }
89:     let frame = frame_ref(state, id)?;
90:     for handler in handlers {
91:         if let Err(error) = call_function_state(state, handler, &[frame]) {
92:             call_error_handler_state(state, &error.to_string());

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:257
255: 		},
256: 		{
257: 			Name = "IsEnabled",
258: 			Type = "Function",
259: 			SecretReturnsForAspect = { Enum.SecretAspect.ButtonState },
260: 
261: 			Arguments =
262: 			{
263: 			},
264: 
265: 			Returns =
266: 			{
267: 				{ Name = "isEnabled", Type = "bool", Nilable = false },
268: 			},
269: 		},
270: 		{
271: 			Name = "RegisterForClicks",
272: 			Type = "Function",
273: 			IsProtectedFunction = true,
274: 			SecretArguments = "NotAllowed",
275: 
276: 			Arguments =
277: 			{
278: 				{ Name = "buttons", Type = "ClickButton", Nilable = false, StrideIndex = 1 },
279: 			},
280: 		},
281: 		{
282: 			Name = "RegisterForMouse",
283: 			Type = "Function",

## widgets-Button-SetButtonState-131
SOURCE Button:SetButtonState + SecretArgumentsAddAspect

PROVIDER
src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:173
171: }
172: 
173: pub(super) fn set_button_state(state: &mut LuaState) -> LuaResult<u32> {
174:     let id = frame_id_from_stack(state, 1)?;
175:     let state_name = String::from_stack(state, 2)?;
176:     let pushed = state_name.eq_ignore_ascii_case("PUSHED");
177:     {
178:         let mut sim = borrow_state_mut(state)?;
179:         if let Some(frame) = sim.widgets.get_mut_visual(id) {
180:             frame.button_state = if pushed { 1 } else { 0 };
181:         }
182:         sync_button_slot_visibility(&mut sim, id);
183:     }
184:     Ok(0)
185: }
186: 
187: pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
188:     let id = frame_id_from_stack(state, 1)?;
189:     let pushed = {
190:         let sim = borrow_state(state)?;
191:         sim.widgets
192:             .get(id)
193:             .map(|frame| frame.button_state == 1)

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:293
291: 		},
292: 		{
293: 			Name = "SetButtonState",
294: 			Type = "Function",
295: 			SecretArgumentsAddAspect = { Enum.SecretAspect.ButtonState },
296: 			SecretArguments = "AllowedWhenUntainted",
297: 
298: 			Arguments =
299: 			{
300: 				{ Name = "buttonState", Type = "SimpleButtonStateToken", Nilable = false },
301: 				{ Name = "lock", Type = "bool", Nilable = false, Default = false },
302: 			},
303: 		},
304: 		{
305: 			Name = "SetDisabledAtlas",
306: 			Type = "Function",
307: 			SecretArguments = "AllowedWhenUntainted",
308: 
309: 			Arguments =
310: 			{
311: 				{ Name = "atlas", Type = "textureAtlas", Nilable = false },
312: 			},
313: 		},
314: 		{
315: 			Name = "SetDisabledFontObject",
316: 			Type = "Function",
317: 			SecretArguments = "AllowedWhenUntainted",
318: 
319: 			Arguments =

## widgets-Button-SetEnabled-132
SOURCE Button:SetEnabled + SecretArgumentsAddAspect, SecretArguments NotAllowed -> AllowedWhenUntainted

PROVIDER
src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:98
96: }
97: 
98: pub(super) fn set_enabled(state: &mut LuaState) -> LuaResult<u32> {
99:     let id = frame_id_from_stack(state, 1)?;
100:     let enabled = bool::from_stack(state, 2).ok().unwrap_or(true);
101:     let changed = {
102:         let sim = borrow_state(state)?;
103:         sim.widgets
104:             .get(id)
105:             .map(|f| button_enabled(f) != enabled)
106:             .unwrap_or(false)
107:     };
108:     set_button_enabled_value(state, id, enabled)?;
109:     if changed {
110:         fire_enable_disable_script(state, id, enabled)?;
111:     }
112:     Ok(0)
113: }
114: 
115: pub(super) fn enable(state: &mut LuaState) -> LuaResult<u32> {
116:     let id = frame_id_from_stack(state, 1)?;
117:     let was_disabled = {
118:         let sim = borrow_state(state)?;

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:336
334: 		},
335: 		{
336: 			Name = "SetEnabled",
337: 			Type = "Function",
338: 			IsProtectedFunction = true,
339: 			SecretArgumentsAddAspect = { Enum.SecretAspect.ButtonState },
340: 			SecretArguments = "AllowedWhenUntainted",
341: 
342: 			Arguments =
343: 			{
344: 				{ Name = "enabled", Type = "bool", Nilable = false, Default = false },
345: 			},
346: 		},
347: 		{
348: 			Name = "SetFontString",
349: 			Type = "Function",
350: 			CheckAllowChangeParent = true,
351: 			SecretArguments = "AllowedWhenUntainted",
352: 
353: 			Arguments =
354: 			{
355: 				{ Name = "fontString", Type = "SimpleFontString", Nilable = false },
356: 			},
357: 		},
358: 		{
359: 			Name = "SetFormattedText",
360: 			Type = "Function",
361: 			SecretArgumentsAddAspect = { Enum.SecretAspect.Text },
362: 			SecretArguments = "AllowedWhenUntainted",

## widgets-EditBox-SetFont-133
SOURCE EditBox:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

PROVIDER
src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330
328: }
329: 
330: pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
331:     let id = frame_id_from_stack(state, 1)?;
332:     let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
333:     if is_simple_html_frame(state, id) {
334:         let font = val_to_string(state, stack_val(state, 3));
335:         let size = match stack_val(state, 4) {
336:             Val::Num(n) => Some(n as f32),
337:             _ => None,
338:         };
339:         let flags = val_to_string(state, stack_val(state, 5));
340:         set_simple_html_font(state, id, text_type, font, size, flags);
341:         return Ok(0);
342:     }
343:     let font = val_to_string(state, stack_val(state, 2));
344:     if font.is_none() {
345:         state.push(Val::Bool(false));
346:         return Ok(1);
347:     }
348:     let size = match stack_val(state, 3) {
349:         Val::Num(n) => Some(n as f32),
350:         _ => None,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleEditBoxAPIDocumentation.lua:677
675: 		},
676: 		{
677: 			Name = "SetFont",
678: 			Type = "Function",
679: 			RequiresValidFontAsset = true,
680: 			RequiresValidFontHeight = true,
681: 			SecretArguments = "AllowedWhenUntainted",
682: 
683: 			Arguments =
684: 			{
685: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
686: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
687: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
688: 			},
689: 
690: 			Returns =
691: 			{
692: 				{ Name = "success", Type = "bool", Nilable = false },
693: 			},
694: 		},
695: 		{
696: 			Name = "SetFontObject",
697: 			Type = "Function",
698: 			SecretArguments = "AllowedWhenUntainted",
699: 
700: 			Arguments =
701: 			{
702: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
703: 			},

## widgets-Font-SetFont-134
SOURCE Font:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

PROVIDER
src/lua_api/globals/font_strings_collection/fonts.rs:102
100: }
101: 
102: fn font_set_font(state: &mut LuaState) -> LuaResult<u32> {
103:     let font = stack_val(state, 1);
104:     let path = Option::<String>::from_stack(state, 2)?;
105:     let height = Option::<f64>::from_stack(state, 3)?;
106:     let flags = Option::<String>::from_stack(state, 4)?;
107:     let Some(path) = path else { return Ok(0) };
108:     let path_val = create_string(state, &path);
109:     table_set_static(state, font, "__fontPath", path_val);
110:     if let Some(h) = height {
111:         table_set_static(state, font, "__fontHeight", Val::Num(h));
112:     }
113:     let flags_val = create_string(state, flags.as_deref().unwrap_or(""));
114:     table_set_static(state, font, "__fontFlags", flags_val);
115:     Ok(0)
116: }
117: 
118: fn font_get_font(state: &mut LuaState) -> LuaResult<u32> {
119:     let font = stack_val(state, 1);
120:     let path = table_get_static(state, font, "__fontPath");
121:     let path_val = match path {
122:         Val::Str(_) => path,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontAPIDocumentation.lua:199
197: 		},
198: 		{
199: 			Name = "SetFont",
200: 			Type = "Function",
201: 			RequiresValidFontAsset = true,
202: 			RequiresValidFontHeight = true,
203: 			SecretArguments = "AllowedWhenUntainted",
204: 
205: 			Arguments =
206: 			{
207: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
208: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
209: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
210: 			},
211: 		},
212: 		{
213: 			Name = "SetFontHeight",
214: 			Type = "Function",
215: 			SecretArguments = "AllowedWhenUntainted",
216: 			Documentation = { "Preserves all flags, does correct height conversion due to fixedHeight." },
217: 
218: 			Arguments =
219: 			{
220: 				{ Name = "height", Type = "number", Nilable = false },
221: 			},
222: 		},
223: 		{
224: 			Name = "SetFontObject",
225: 			Type = "Function",

## widgets-FontString-SetFont-135
SOURCE FontString:SetFont + RequiresValidFontHeight + RequiresValidFontAsset, arg2.Type number -> uiFontHeight

PROVIDER
src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330
328: }
329: 
330: pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
331:     let id = frame_id_from_stack(state, 1)?;
332:     let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
333:     if is_simple_html_frame(state, id) {
334:         let font = val_to_string(state, stack_val(state, 3));
335:         let size = match stack_val(state, 4) {
336:             Val::Num(n) => Some(n as f32),
337:             _ => None,
338:         };
339:         let flags = val_to_string(state, stack_val(state, 5));
340:         set_simple_html_font(state, id, text_type, font, size, flags);
341:         return Ok(0);
342:     }
343:     let font = val_to_string(state, stack_val(state, 2));
344:     if font.is_none() {
345:         state.push(Val::Bool(false));
346:         return Ok(1);
347:     }
348:     let size = match stack_val(state, 3) {
349:         Val::Num(n) => Some(n as f32),
350:         _ => None,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua:500
498: 		},
499: 		{
500: 			Name = "SetFont",
501: 			Type = "Function",
502: 			RequiresValidFontAsset = true,
503: 			RequiresValidFontHeight = true,
504: 			SecretArguments = "AllowedWhenUntainted",
505: 
506: 			Arguments =
507: 			{
508: 				{ Name = "fontFile", Type = "FontAsset", Nilable = false },
509: 				{ Name = "fontHeight", Type = "uiFontHeight", Nilable = false },
510: 				{ Name = "flags", Type = "TBFFlags", Nilable = true },
511: 			},
512: 
513: 			Returns =
514: 			{
515: 				{ Name = "success", Type = "bool", Nilable = false },
516: 			},
517: 		},
518: 		{
519: 			Name = "SetFontHeight",
520: 			Type = "Function",
521: 			SecretArguments = "AllowedWhenUntainted",
522: 
523: 			Arguments =
524: 			{
525: 				{ Name = "height", Type = "uiUnit", Nilable = false },
526: 			},

## widgets-MessageFrame-SetFont-136
SOURCE MessageFrame:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

PROVIDER
src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330
328: }
329: 
330: pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
331:     let id = frame_id_from_stack(state, 1)?;
332:     let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
333:     if is_simple_html_frame(state, id) {
334:         let font = val_to_string(state, stack_val(state, 3));
335:         let size = match stack_val(state, 4) {
336:             Val::Num(n) => Some(n as f32),
337:             _ => None,
338:         };
339:         let flags = val_to_string(state, stack_val(state, 5));
340:         set_simple_html_font(state, id, text_type, font, size, flags);
341:         return Ok(0);
342:     }
343:     let font = val_to_string(state, stack_val(state, 2));
344:     if font.is_none() {
345:         state.push(Val::Bool(false));
346:         return Ok(1);
347:     }
348:     let size = match stack_val(state, 3) {
349:         Val::Num(n) => Some(n as f32),
350:         _ => None,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleMessageFrameAPIDocumentation.lua:295
293: 		},
294: 		{
295: 			Name = "SetFont",
296: 			Type = "Function",
297: 			RequiresValidFontAsset = true,
298: 			RequiresValidFontHeight = true,
299: 			SecretArguments = "AllowedWhenUntainted",
300: 
301: 			Arguments =
302: 			{
303: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
304: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
305: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
306: 			},
307: 		},
308: 		{
309: 			Name = "SetFontObject",
310: 			Type = "Function",
311: 			SecretArguments = "AllowedWhenUntainted",
312: 
313: 			Arguments =
314: 			{
315: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
316: 			},
317: 		},
318: 		{
319: 			Name = "SetIndentedWordWrap",
320: 			Type = "Function",
321: 			SecretArguments = "AllowedWhenUntainted",

## widgets-ModelSceneActorBase-GetModelUnitGUID-137
SOURCE ModelSceneActorBase:GetModelUnitGUID - ret1.ConditionalSecret

PROVIDER


DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/FrameAPIModelSceneFrameActorBaseDocumentation.lua:138
136: 		},
137: 		{
138: 			Name = "GetModelUnitGUID",
139: 			Type = "Function",
140: 
141: 			Arguments =
142: 			{
143: 			},
144: 
145: 			Returns =
146: 			{
147: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
148: 			},
149: 		},
150: 		{
151: 			Name = "GetParticleOverrideScale",
152: 			Type = "Function",
153: 
154: 			Arguments =
155: 			{
156: 			},
157: 
158: 			Returns =
159: 			{
160: 				{ Name = "scale", Type = "number", Nilable = true },
161: 			},
162: 		},
163: 		{
164: 			Name = "GetPitch",

## widgets-ScrollFrame-GetHorizontalScroll-138
SOURCE ScrollFrame:GetHorizontalScroll + SecretReturnsForAspect

PROVIDER
src/lua_api/frame/methods/widgets/slider.rs:469
467: }
468: 
469: pub(super) fn get_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
470:     let id = frame_id_from_stack(state, 1)?;
471:     let offset = borrow_state(state)?
472:         .widgets
473:         .get(id)
474:         .map(|frame| frame.scroll_horizontal)
475:         .unwrap_or(0.0);
476:     state.push(Val::Num(offset));
477:     Ok(1)
478: }
479: 
480: pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
481:     let id = frame_id_from_stack(state, 1)?;
482:     let offset = val_to_f64(stack_val(state, 2));
483:     let mut sim = borrow_state_mut(state)?;
484:     if sim
485:         .widgets
486:         .get(id)
487:         .is_some_and(|frame| frame.scroll_horizontal == offset)
488:     {
489:         return Ok(0);

src/lua_api/frame/methods/widgets/slider.rs:500
498: }
499: 
500: pub(super) fn get_horizontal_scroll_range(state: &mut LuaState) -> LuaResult<u32> {
501:     let id = frame_id_from_stack(state, 1)?;
502:     let range = borrow_state(state)?
503:         .widgets
504:         .get(id)
505:         .map(|frame| scroll_range(frame, 'h'))
506:         .unwrap_or(0.0);
507:     state.push(Val::Num(range));
508:     Ok(1)
509: }
510: 
511: pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
512:     let id = frame_id_from_stack(state, 1)?;
513:     let offset = borrow_state(state)?
514:         .widgets
515:         .get(id)
516:         .map(|frame| frame.scroll_vertical)
517:         .unwrap_or(0.0);
518:     state.push(Val::Num(offset));
519:     Ok(1)
520: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:10
8: 	{
9: 		{
10: 			Name = "GetHorizontalScroll",
11: 			Type = "Function",
12: 			SecretReturnsForAspect = { Enum.SecretAspect.ScrollOffset },
13: 
14: 			Arguments =
15: 			{
16: 			},
17: 
18: 			Returns =
19: 			{
20: 				{ Name = "offset", Type = "uiUnit", Nilable = false },
21: 			},
22: 		},
23: 		{
24: 			Name = "GetHorizontalScrollRange",
25: 			Type = "Function",
26: 			SecretReturnsForAspect = { Enum.SecretAspect.ScrollRange },
27: 
28: 			Arguments =
29: 			{
30: 			},
31: 
32: 			Returns =
33: 			{
34: 				{ Name = "range", Type = "uiUnit", Nilable = false },
35: 			},
36: 		},

## widgets-ScrollFrame-GetVerticalScroll-139
SOURCE ScrollFrame:GetVerticalScroll + SecretReturnsForAspect

PROVIDER
src/lua_api/frame/methods/widgets/slider.rs:511
509: }
510: 
511: pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
512:     let id = frame_id_from_stack(state, 1)?;
513:     let offset = borrow_state(state)?
514:         .widgets
515:         .get(id)
516:         .map(|frame| frame.scroll_vertical)
517:         .unwrap_or(0.0);
518:     state.push(Val::Num(offset));
519:     Ok(1)
520: }
521: 
522: pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
523:     let id = frame_id_from_stack(state, 1)?;
524:     let offset = val_to_f64(stack_val(state, 2));
525:     let mut sim = borrow_state_mut(state)?;
526:     if sim
527:         .widgets
528:         .get(id)
529:         .is_some_and(|frame| frame.scroll_vertical == offset)
530:     {
531:         return Ok(0);

src/lua_api/frame/methods/widgets/slider.rs:542
540: }
541: 
542: pub(super) fn get_vertical_scroll_range(state: &mut LuaState) -> LuaResult<u32> {
543:     let id = frame_id_from_stack(state, 1)?;
544:     let range = borrow_state(state)?
545:         .widgets
546:         .get(id)
547:         .map(|frame| scroll_range(frame, 'v'))
548:         .unwrap_or(0.0);
549:     state.push(Val::Num(range));
550:     Ok(1)
551: }
552: 
553: pub(super) fn get_scroll_child(state: &mut LuaState) -> LuaResult<u32> {
554:     let id = frame_id_from_stack(state, 1)?;
555:     let child_id = {
556:         let sim = borrow_state(state)?;
557:         sim.widgets.get(id).and_then(|frame| frame.scroll_child_id)
558:     };
559:     match child_id {
560:         Some(child_id) => {
561:             let child = frame_ref(state, child_id)?;
562:             state.push(child);

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:51
49: 		},
50: 		{
51: 			Name = "GetVerticalScroll",
52: 			Type = "Function",
53: 			SecretReturnsForAspect = { Enum.SecretAspect.ScrollOffset },
54: 
55: 			Arguments =
56: 			{
57: 			},
58: 
59: 			Returns =
60: 			{
61: 				{ Name = "offset", Type = "uiUnit", Nilable = false },
62: 			},
63: 		},
64: 		{
65: 			Name = "GetVerticalScrollRange",
66: 			Type = "Function",
67: 			SecretReturnsForAspect = { Enum.SecretAspect.ScrollRange },
68: 
69: 			Arguments =
70: 			{
71: 			},
72: 
73: 			Returns =
74: 			{
75: 				{ Name = "range", Type = "uiUnit", Nilable = false },
76: 			},
77: 		},

## widgets-ScrollFrame-SetHorizontalScroll-140
SOURCE ScrollFrame:SetHorizontalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect

PROVIDER
src/lua_api/frame/methods/widgets/slider.rs:480
478: }
479: 
480: pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
481:     let id = frame_id_from_stack(state, 1)?;
482:     let offset = val_to_f64(stack_val(state, 2));
483:     let mut sim = borrow_state_mut(state)?;
484:     if sim
485:         .widgets
486:         .get(id)
487:         .is_some_and(|frame| frame.scroll_horizontal == offset)
488:     {
489:         return Ok(0);
490:     }
491:     if let Some(frame) = sim.widgets.get_mut_visual(id) {
492:         frame.scroll_horizontal = offset;
493:     }
494:     crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
495:     drop(sim);
496:     fire_scroll_frame_event(state, id, "OnHorizontalScroll", &[Val::Num(offset)])?;
497:     Ok(0)
498: }
499: 
500: pub(super) fn get_horizontal_scroll_range(state: &mut LuaState) -> LuaResult<u32> {

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:79
77: 		},
78: 		{
79: 			Name = "SetHorizontalScroll",
80: 			Type = "Function",
81: 			IsProtectedFunction = true,
82: 			SecretArgumentsAddAspect = { Enum.SecretAspect.ScrollOffset },
83: 			SecretArguments = "AllowedWhenUntainted",
84: 
85: 			Arguments =
86: 			{
87: 				{ Name = "offset", Type = "uiUnit", Nilable = false },
88: 			},
89: 		},
90: 		{
91: 			Name = "SetScrollChild",
92: 			Type = "Function",
93: 			IsProtectedFunction = true,
94: 			CheckAllowChangeParent = true,
95: 			SecretArguments = "AllowedWhenUntainted",
96: 
97: 			Arguments =
98: 			{
99: 				{ Name = "scrollChild", Type = "SimpleFrame", Nilable = false },
100: 			},
101: 		},
102: 		{
103: 			Name = "SetVerticalScroll",
104: 			Type = "Function",
105: 			IsProtectedFunction = true,

## widgets-ScrollFrame-SetVerticalScroll-141
SOURCE ScrollFrame:SetVerticalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect

PROVIDER
src/lua_api/frame/methods/widgets/slider.rs:522
520: }
521: 
522: pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
523:     let id = frame_id_from_stack(state, 1)?;
524:     let offset = val_to_f64(stack_val(state, 2));
525:     let mut sim = borrow_state_mut(state)?;
526:     if sim
527:         .widgets
528:         .get(id)
529:         .is_some_and(|frame| frame.scroll_vertical == offset)
530:     {
531:         return Ok(0);
532:     }
533:     if let Some(frame) = sim.widgets.get_mut_visual(id) {
534:         frame.scroll_vertical = offset;
535:     }
536:     crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
537:     drop(sim);
538:     fire_scroll_frame_event(state, id, "OnVerticalScroll", &[Val::Num(offset)])?;
539:     Ok(0)
540: }
541: 
542: pub(super) fn get_vertical_scroll_range(state: &mut LuaState) -> LuaResult<u32> {

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:103
101: 		},
102: 		{
103: 			Name = "SetVerticalScroll",
104: 			Type = "Function",
105: 			IsProtectedFunction = true,
106: 			SecretArgumentsAddAspect = { Enum.SecretAspect.ScrollOffset },
107: 			SecretArguments = "AllowedWhenUntainted",
108: 
109: 			Arguments =
110: 			{
111: 				{ Name = "offset", Type = "uiUnit", Nilable = false },
112: 			},
113: 		},
114: 		{
115: 			Name = "UpdateScrollChildRect",
116: 			Type = "Function",
117: 
118: 			Arguments =
119: 			{
120: 			},
121: 		},
122: 	},
123: 
124: 	Events =
125: 	{
126: 	},
127: 
128: 	Tables =
129: 	{

## widgets-SimpleHTML-SetFont-142
SOURCE SimpleHTML:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

PROVIDER
src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330
328: }
329: 
330: pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
331:     let id = frame_id_from_stack(state, 1)?;
332:     let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
333:     if is_simple_html_frame(state, id) {
334:         let font = val_to_string(state, stack_val(state, 3));
335:         let size = match stack_val(state, 4) {
336:             Val::Num(n) => Some(n as f32),
337:             _ => None,
338:         };
339:         let flags = val_to_string(state, stack_val(state, 5));
340:         set_simple_html_font(state, id, text_type, font, size, flags);
341:         return Ok(0);
342:     }
343:     let font = val_to_string(state, stack_val(state, 2));
344:     if font.is_none() {
345:         state.push(Val::Bool(false));
346:         return Ok(1);
347:     }
348:     let size = match stack_val(state, 3) {
349:         Val::Num(n) => Some(n as f32),
350:         _ => None,

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleHTMLAPIDocumentation.lua:203
201: 		},
202: 		{
203: 			Name = "SetFont",
204: 			Type = "Function",
205: 			RequiresValidFontAsset = true,
206: 			RequiresValidFontHeight = true,
207: 			SecretArguments = "AllowedWhenUntainted",
208: 
209: 			Arguments =
210: 			{
211: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
212: 				{ Name = "fontFile", Type = "cstring", Nilable = false },
213: 				{ Name = "height", Type = "uiFontHeight", Nilable = false },
214: 				{ Name = "flags", Type = "TBFFlags", Nilable = false },
215: 			},
216: 		},
217: 		{
218: 			Name = "SetFontObject",
219: 			Type = "Function",
220: 			SecretArguments = "AllowedWhenUntainted",
221: 
222: 			Arguments =
223: 			{
224: 				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
225: 				{ Name = "font", Type = "SimpleFont", Nilable = false },
226: 			},
227: 		},
228: 		{
229: 			Name = "SetHyperlinkFormat",

## source-context-144
SOURCE Events Added (2):

PROVIDER


DECLARATION


## events-ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED-145
SOURCE ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED

PROVIDER
src/c_api/c_encounter_timeline/notifications.rs:32
30:             .is_some_and(|event| event.state == EventState::Active && layout::visible(event));
31:         if current {
32:             event(state, "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED", id)?;
33:             let active = borrow_state(state)?
34:                 .encounter_timeline
35:                 .events
36:                 .get(&id)
37:                 .is_some_and(|event| event.state == EventState::Active && layout::visible(event));
38:             if active {
39:                 event(state, "ENCOUNTER_TIMELINE_EVENT_HIGHLIGHT", id)?;
40:             }
41:         }
42:     }
43:     Ok(())
44: }
45: 
46: pub(super) fn event(state: &mut LuaState, name: &str, id: u32) -> LuaResult<()> {
47:     dispatch_event_now(state, name, &[Val::Num(f64::from(id))])
48: }

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterTimelineDocumentation.lua:481
479: 			Name = "EncounterTimelineEventColorChanged",
480: 			Type = "Event",
481: 			LiteralName = "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED",
482: 			UniqueEvent = true,
483: 			Documentation = { "Fired when an event has met a condition that should trigger a color change." },
484: 			Payload =
485: 			{
486: 				{ Name = "eventID", Type = "EncounterTimelineEventID", Nilable = false },
487: 			},
488: 		},
489: 		{
490: 			Name = "EncounterTimelineEventHighlight",
491: 			Type = "Event",
492: 			LiteralName = "ENCOUNTER_TIMELINE_EVENT_HIGHLIGHT",
493: 			UniqueEvent = true,
494: 			Documentation = { "Fired when an event has met a condition that should trigger its highlight glow animation." },
495: 			Payload =
496: 			{
497: 				{ Name = "eventID", Type = "EncounterTimelineEventID", Nilable = false },
498: 			},
499: 		},
500: 		{
501: 			Name = "EncounterTimelineEventRemoved",
502: 			Type = "Event",
503: 			LiteralName = "ENCOUNTER_TIMELINE_EVENT_REMOVED",
504: 			UniqueEvent = true,
505: 			Documentation = { "Fired when an event has been removed from the timeline. This is guaranteed to fire after an event has transitioned to a 'final' state such as Canceled or Finished, and will be delayed at least one game tick to allow for API queries to still access event data in OnUpdate scripts. This is fired post-removal of the event, and so queries using the supplied event ID will return nil." },
506: 			Payload =
507: 			{

## events-URL_TEXTURE_REQUEST_RESULT-146
SOURCE URL_TEXTURE_REQUEST_RESULT

PROVIDER
src/event/valid_events_c.rs:475
473:     "USE_GLYPH",
474:     "USE_NO_REFUND_CONFIRM",
475:     "URL_TEXTURE_REQUEST_RESULT",
476:     "VARIABLES_LOADED",
477:     "VAS_CHARACTER_QUEUE_STATUS_UPDATE",
478:     "VAS_CHARACTER_STATE_CHANGED",
479:     "VAS_QUEUE_STATUS_UPDATE",
480:     "VAS_TRANSFER_VALIDATION_UPDATE",
481:     "VEHICLE_ANGLE_SHOW",
482:     "VEHICLE_ANGLE_UPDATE",
483:     "VEHICLE_PASSENGERS_CHANGED",
484:     "VEHICLE_POWER_SHOW",
485:     "VEHICLE_UPDATE",
486:     "VIEWED_TRANSMOG_OUTFIT_CHANGED",
487:     "VIEWED_TRANSMOG_OUTFIT_SECONDARY_SLOTS_CHANGED",
488:     "VIEWED_TRANSMOG_OUTFIT_SITUATIONS_CHANGED",
489:     "VIEWED_TRANSMOG_OUTFIT_SLOT_REFRESH",
490:     "VIEWED_TRANSMOG_OUTFIT_SLOT_SAVE_SUCCESS",
491:     "VIEWED_TRANSMOG_OUTFIT_SLOT_WEAPON_OPTION_CHANGED",
492:     "VIEW_HOUSES_LIST_RECIEVED",
493:     "VIGNETTES_UPDATED",
494:     "VIGNETTE_MINIMAP_UPDATED",
495:     "VOICE_CHAT_ACTIVE_INPUT_DEVICE_UPDATED",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:164
162: 			Name = "UrlTextureRequestResult",
163: 			Type = "Event",
164: 			LiteralName = "URL_TEXTURE_REQUEST_RESULT",
165: 			SynchronousEvent = true,
166: 			Payload =
167: 			{
168: 				{ Name = "texture", Type = "SimpleTexture", Nilable = false },
169: 				{ Name = "result", Type = "UrlTextureResult", Nilable = false },
170: 			},
171: 		},
172: 	},
173: 
174: 	Tables =
175: 	{
176: 		{
177: 			Name = "TitleIconVersion",
178: 			Type = "Enumeration",
179: 			NumValues = 3,
180: 			MinValue = 0,
181: 			MaxValue = 2,
182: 			Fields =
183: 			{
184: 				{ Name = "Small", Type = "TitleIconVersion", EnumValue = 0 },
185: 				{ Name = "Medium", Type = "TitleIconVersion", EnumValue = 1 },
186: 				{ Name = "Large", Type = "TitleIconVersion", EnumValue = 2 },
187: 			},
188: 		},
189: 		{
190: 			Name = "UrlTextureResult",

## source-context-148
SOURCE Event Changes:

PROVIDER


DECLARATION


## events-CHAT_MSG_COMBAT_FACTION_CHANGE-149
SOURCE CHAT_MSG_COMBAT_FACTION_CHANGE - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:332
330:     "CHAT_MSG_CHANNEL_NOTICE",
331:     "CHAT_MSG_CHANNEL_NOTICE_USER",
332:     "CHAT_MSG_COMBAT_FACTION_CHANGE",
333:     "CHAT_MSG_COMBAT_HONOR_GAIN",
334:     "CHAT_MSG_COMBAT_MISC_INFO",
335:     "CHAT_MSG_COMBAT_XP_GAIN",
336:     "CHAT_MSG_COMMUNITIES_CHANNEL",
337:     "CHAT_MSG_CURRENCY",
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1350
1348: 			Name = "ChatMsgCombatFactionChange",
1349: 			Type = "Event",
1350: 			LiteralName = "CHAT_MSG_COMBAT_FACTION_CHANGE",
1351: 			SynchronousEvent = true,
1352: 			Payload =
1353: 			{
1354: 				{ Name = "text", Type = "cstring", Nilable = false },
1355: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1356: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1357: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1358: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1359: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1360: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1361: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1362: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1363: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1364: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1365: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1366: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1367: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1368: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1369: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1370: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1371: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1372: 			},
1373: 		},
1374: 		{
1375: 			Name = "ChatMsgCombatHonorGain",
1376: 			Type = "Event",

## events-CHAT_MSG_COMBAT_HONOR_GAIN-150
SOURCE CHAT_MSG_COMBAT_HONOR_GAIN - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:333
331:     "CHAT_MSG_CHANNEL_NOTICE_USER",
332:     "CHAT_MSG_COMBAT_FACTION_CHANGE",
333:     "CHAT_MSG_COMBAT_HONOR_GAIN",
334:     "CHAT_MSG_COMBAT_MISC_INFO",
335:     "CHAT_MSG_COMBAT_XP_GAIN",
336:     "CHAT_MSG_COMMUNITIES_CHANNEL",
337:     "CHAT_MSG_CURRENCY",
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1377
1375: 			Name = "ChatMsgCombatHonorGain",
1376: 			Type = "Event",
1377: 			LiteralName = "CHAT_MSG_COMBAT_HONOR_GAIN",
1378: 			SynchronousEvent = true,
1379: 			Payload =
1380: 			{
1381: 				{ Name = "text", Type = "cstring", Nilable = false },
1382: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1383: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1384: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1385: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1386: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1387: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1388: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1389: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1390: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1391: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1392: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1393: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1394: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1395: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1396: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1397: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1398: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1399: 			},
1400: 		},
1401: 		{
1402: 			Name = "ChatMsgCombatMiscInfo",
1403: 			Type = "Event",

## events-CHAT_MSG_COMBAT_MISC_INFO-151
SOURCE CHAT_MSG_COMBAT_MISC_INFO - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:334
332:     "CHAT_MSG_COMBAT_FACTION_CHANGE",
333:     "CHAT_MSG_COMBAT_HONOR_GAIN",
334:     "CHAT_MSG_COMBAT_MISC_INFO",
335:     "CHAT_MSG_COMBAT_XP_GAIN",
336:     "CHAT_MSG_COMMUNITIES_CHANNEL",
337:     "CHAT_MSG_CURRENCY",
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1404
1402: 			Name = "ChatMsgCombatMiscInfo",
1403: 			Type = "Event",
1404: 			LiteralName = "CHAT_MSG_COMBAT_MISC_INFO",
1405: 			SynchronousEvent = true,
1406: 			Payload =
1407: 			{
1408: 				{ Name = "text", Type = "cstring", Nilable = false },
1409: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1410: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1411: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1412: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1413: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1414: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1415: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1416: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1417: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1418: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1419: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1420: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1421: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1422: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1423: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1424: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1425: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1426: 			},
1427: 		},
1428: 		{
1429: 			Name = "ChatMsgCombatXpGain",
1430: 			Type = "Event",

## events-CHAT_MSG_COMBAT_XP_GAIN-152
SOURCE CHAT_MSG_COMBAT_XP_GAIN - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:335
333:     "CHAT_MSG_COMBAT_HONOR_GAIN",
334:     "CHAT_MSG_COMBAT_MISC_INFO",
335:     "CHAT_MSG_COMBAT_XP_GAIN",
336:     "CHAT_MSG_COMMUNITIES_CHANNEL",
337:     "CHAT_MSG_CURRENCY",
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",
355:     "CHAT_MSG_OPENING",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1431
1429: 			Name = "ChatMsgCombatXpGain",
1430: 			Type = "Event",
1431: 			LiteralName = "CHAT_MSG_COMBAT_XP_GAIN",
1432: 			SynchronousEvent = true,
1433: 			Payload =
1434: 			{
1435: 				{ Name = "text", Type = "cstring", Nilable = false },
1436: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1437: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1438: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1439: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1440: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1441: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1442: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1443: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1444: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1445: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1446: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1447: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1448: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1449: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1450: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1451: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1452: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1453: 			},
1454: 		},
1455: 		{
1456: 			Name = "ChatMsgCommunitiesChannel",
1457: 			Type = "Event",

## events-CHAT_MSG_CURRENCY-153
SOURCE CHAT_MSG_CURRENCY - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:337
335:     "CHAT_MSG_COMBAT_XP_GAIN",
336:     "CHAT_MSG_COMMUNITIES_CHANNEL",
337:     "CHAT_MSG_CURRENCY",
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",
355:     "CHAT_MSG_OPENING",
356:     "CHAT_MSG_PARTY",
357:     "CHAT_MSG_PARTY_LEADER",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1486
1484: 			Name = "ChatMsgCurrency",
1485: 			Type = "Event",
1486: 			LiteralName = "CHAT_MSG_CURRENCY",
1487: 			SynchronousEvent = true,
1488: 			Payload =
1489: 			{
1490: 				{ Name = "text", Type = "cstring", Nilable = false },
1491: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1492: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1493: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1494: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1495: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1496: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1497: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1498: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1499: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1500: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1501: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1502: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1503: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1504: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1505: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1506: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1507: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1508: 			},
1509: 		},
1510: 		{
1511: 			Name = "ChatMsgDnd",
1512: 			Type = "Event",

## events-CHAT_MSG_FILTERED-154
SOURCE CHAT_MSG_FILTERED - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:340
338:     "CHAT_MSG_DND",
339:     "CHAT_MSG_EMOTE",
340:     "CHAT_MSG_FILTERED",
341:     "CHAT_MSG_GUILD",
342:     "CHAT_MSG_GUILD_ACHIEVEMENT",
343:     "CHAT_MSG_GUILD_ITEM_LOOTED",
344:     "CHAT_MSG_IGNORED",
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",
355:     "CHAT_MSG_OPENING",
356:     "CHAT_MSG_PARTY",
357:     "CHAT_MSG_PARTY_LEADER",
358:     "CHAT_MSG_PET_BATTLE_COMBAT_LOG",
359:     "CHAT_MSG_PET_BATTLE_INFO",
360:     "CHAT_MSG_PET_INFO",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1569
1567: 			Name = "ChatMsgFiltered",
1568: 			Type = "Event",
1569: 			LiteralName = "CHAT_MSG_FILTERED",
1570: 			SynchronousEvent = true,
1571: 			Payload =
1572: 			{
1573: 				{ Name = "text", Type = "cstring", Nilable = false },
1574: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1575: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1576: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1577: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1578: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1579: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1580: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1581: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1582: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1583: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1584: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1585: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1586: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1587: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1588: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1589: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1590: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1591: 			},
1592: 		},
1593: 		{
1594: 			Name = "ChatMsgGuild",
1595: 			Type = "Event",

## events-CHAT_MSG_LOOT-155
SOURCE CHAT_MSG_LOOT - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:347
345:     "CHAT_MSG_INSTANCE_CHAT",
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",
355:     "CHAT_MSG_OPENING",
356:     "CHAT_MSG_PARTY",
357:     "CHAT_MSG_PARTY_LEADER",
358:     "CHAT_MSG_PET_BATTLE_COMBAT_LOG",
359:     "CHAT_MSG_PET_BATTLE_INFO",
360:     "CHAT_MSG_PET_INFO",
361:     "CHAT_MSG_PING",
362:     "CHAT_MSG_RAID",
363:     "CHAT_MSG_RAID_BOSS_EMOTE",
364:     "CHAT_MSG_RAID_BOSS_WHISPER",
365:     "CHAT_MSG_RAID_LEADER",
366:     "CHAT_MSG_RAID_WARNING",
367:     "CHAT_MSG_RESTRICTED",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1789
1787: 			Name = "ChatMsgLoot",
1788: 			Type = "Event",
1789: 			LiteralName = "CHAT_MSG_LOOT",
1790: 			SynchronousEvent = true,
1791: 			Payload =
1792: 			{
1793: 				{ Name = "text", Type = "cstring", Nilable = false },
1794: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1795: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1796: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1797: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1798: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1799: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1800: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1801: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1802: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1803: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1804: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1805: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1806: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1807: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1808: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1809: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1810: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1811: 			},
1812: 		},
1813: 		{
1814: 			Name = "ChatMsgMoney",
1815: 			Type = "Event",

## events-CHAT_MSG_MONEY-156
SOURCE CHAT_MSG_MONEY - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:348
346:     "CHAT_MSG_INSTANCE_CHAT_LEADER",
347:     "CHAT_MSG_LOOT",
348:     "CHAT_MSG_MONEY",
349:     "CHAT_MSG_MONSTER_EMOTE",
350:     "CHAT_MSG_MONSTER_PARTY",
351:     "CHAT_MSG_MONSTER_SAY",
352:     "CHAT_MSG_MONSTER_WHISPER",
353:     "CHAT_MSG_MONSTER_YELL",
354:     "CHAT_MSG_OFFICER",
355:     "CHAT_MSG_OPENING",
356:     "CHAT_MSG_PARTY",
357:     "CHAT_MSG_PARTY_LEADER",
358:     "CHAT_MSG_PET_BATTLE_COMBAT_LOG",
359:     "CHAT_MSG_PET_BATTLE_INFO",
360:     "CHAT_MSG_PET_INFO",
361:     "CHAT_MSG_PING",
362:     "CHAT_MSG_RAID",
363:     "CHAT_MSG_RAID_BOSS_EMOTE",
364:     "CHAT_MSG_RAID_BOSS_WHISPER",
365:     "CHAT_MSG_RAID_LEADER",
366:     "CHAT_MSG_RAID_WARNING",
367:     "CHAT_MSG_RESTRICTED",
368:     "CHAT_MSG_SAY",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1816
1814: 			Name = "ChatMsgMoney",
1815: 			Type = "Event",
1816: 			LiteralName = "CHAT_MSG_MONEY",
1817: 			SynchronousEvent = true,
1818: 			Payload =
1819: 			{
1820: 				{ Name = "text", Type = "cstring", Nilable = false },
1821: 				{ Name = "playerName", Type = "cstring", Nilable = false },
1822: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
1823: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
1824: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
1825: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
1826: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
1827: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
1828: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
1829: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
1830: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
1831: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
1832: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
1833: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
1834: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
1835: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
1836: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
1837: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
1838: 			},
1839: 		},
1840: 		{
1841: 			Name = "ChatMsgMonsterEmote",
1842: 			Type = "Event",

## events-CHAT_MSG_RESTRICTED-157
SOURCE CHAT_MSG_RESTRICTED - SecretInChatMessagingLockdown

PROVIDER
src/event/valid_events_a.rs:367
365:     "CHAT_MSG_RAID_LEADER",
366:     "CHAT_MSG_RAID_WARNING",
367:     "CHAT_MSG_RESTRICTED",
368:     "CHAT_MSG_SAY",
369:     "CHAT_MSG_SKILL",
370:     "CHAT_MSG_SYSTEM",
371:     "CHAT_MSG_TARGETICONS",
372:     "CHAT_MSG_TEXT_EMOTE",
373:     "CHAT_MSG_TRADESKILLS",
374:     "CHAT_MSG_VOICE_TEXT",
375:     "CHAT_MSG_WHISPER",
376:     "CHAT_MSG_WHISPER_INFORM",
377:     "CHAT_MSG_YELL",
378:     "CHAT_REGIONAL_SEND_FAILED",
379:     "CHAT_REGIONAL_STATUS_CHANGED",
380:     "CHAT_SERVER_DISCONNECTED",
381:     "CHAT_SERVER_RECONNECTED",
382:     "CHECK_CHARACTER_NAME_AVAILABILITY_RESULT",
383:     "CHEST_REWARDS_UPDATED_FROM_SERVER",
384:     "CINEMATIC_START",
385:     "CINEMATIC_STOP",
386:     "CLASS_TRIAL_TIMER_START",
387:     "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_NAME",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2344
2342: 			Name = "ChatMsgRestricted",
2343: 			Type = "Event",
2344: 			LiteralName = "CHAT_MSG_RESTRICTED",
2345: 			SynchronousEvent = true,
2346: 			Payload =
2347: 			{
2348: 				{ Name = "text", Type = "cstring", Nilable = false },
2349: 				{ Name = "playerName", Type = "cstring", Nilable = false },
2350: 				{ Name = "languageName", Type = "cstring", Nilable = false, NeverSecret = true },
2351: 				{ Name = "channelName", Type = "cstring", Nilable = false, NeverSecret = true },
2352: 				{ Name = "playerName2", Type = "cstring", Nilable = false },
2353: 				{ Name = "specialFlags", Type = "cstring", Nilable = false, NeverSecret = true },
2354: 				{ Name = "zoneChannelID", Type = "number", Nilable = false, NeverSecret = true },
2355: 				{ Name = "channelIndex", Type = "number", Nilable = false, NeverSecret = true },
2356: 				{ Name = "channelBaseName", Type = "cstring", Nilable = false, NeverSecret = true },
2357: 				{ Name = "languageID", Type = "number", Nilable = false, NeverSecret = true },
2358: 				{ Name = "lineID", Type = "number", Nilable = false, NeverSecret = true },
2359: 				{ Name = "guid", Type = "WOWGUID", Nilable = false },
2360: 				{ Name = "bnSenderID", Type = "number", Nilable = false },
2361: 				{ Name = "isMobile", Type = "bool", Nilable = false, NeverSecret = true },
2362: 				{ Name = "isSubtitle", Type = "bool", Nilable = false, NeverSecret = true },
2363: 				{ Name = "hideSenderInLetterbox", Type = "bool", Nilable = false, NeverSecret = true },
2364: 				{ Name = "suppressRaidIcons", Type = "bool", Nilable = false, NeverSecret = true },
2365: 				{ Name = "discordInfo", Type = "DiscordChatInfo", Nilable = false },
2366: 			},
2367: 		},
2368: 		{
2369: 			Name = "ChatMsgSay",
2370: 			Type = "Event",

## events-CLUB_MEMBER_ADDED-158
SOURCE CLUB_MEMBER_ADDED [2].Type number -> ClubMemberOpaqueId

PROVIDER
src/event/valid_events_a.rs:421
419:     "CLUB_INVITATION_REMOVED_FOR_SELF",
420:     "CLUB_MEMBERS_UPDATED",
421:     "CLUB_MEMBER_ADDED",
422:     "CLUB_MEMBER_PRESENCE_UPDATED",
423:     "CLUB_MEMBER_REMOVED",
424:     "CLUB_MEMBER_ROLE_UPDATED",
425:     "CLUB_MEMBER_UPDATED",
426:     "CLUB_MESSAGE_ADDED",
427:     "CLUB_MESSAGE_HISTORY_RECEIVED",
428:     "CLUB_MESSAGE_UPDATED",
429:     "CLUB_REMOVED",
430:     "CLUB_REMOVED_MESSAGE",
431:     "CLUB_SELF_MEMBER_ROLE_UPDATED",
432:     "CLUB_STREAMS_LOADED",
433:     "CLUB_STREAM_ADDED",
434:     "CLUB_STREAM_REMOVED",
435:     "CLUB_STREAM_SUBSCRIBED",
436:     "CLUB_STREAM_UNSUBSCRIBED",
437:     "CLUB_STREAM_UPDATED",
438:     "CLUB_TICKETS_RECEIVED",
439:     "CLUB_TICKET_CREATED",
440:     "CLUB_TICKET_RECEIVED",
441:     "CLUB_UPDATED",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1291
1289: 			Name = "ClubMemberAdded",
1290: 			Type = "Event",
1291: 			LiteralName = "CLUB_MEMBER_ADDED",
1292: 			SynchronousEvent = true,
1293: 			Payload =
1294: 			{
1295: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1296: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1297: 			},
1298: 		},
1299: 		{
1300: 			Name = "ClubMemberPresenceUpdated",
1301: 			Type = "Event",
1302: 			LiteralName = "CLUB_MEMBER_PRESENCE_UPDATED",
1303: 			SynchronousEvent = true,
1304: 			Payload =
1305: 			{
1306: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1307: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1308: 				{ Name = "presence", Type = "ClubMemberPresence", Nilable = false },
1309: 			},
1310: 		},
1311: 		{
1312: 			Name = "ClubMemberRemoved",
1313: 			Type = "Event",
1314: 			LiteralName = "CLUB_MEMBER_REMOVED",
1315: 			SynchronousEvent = true,
1316: 			Payload =
1317: 			{

## events-CLUB_MEMBER_PRESENCE_UPDATED-159
SOURCE CLUB_MEMBER_PRESENCE_UPDATED [2].Type number -> ClubMemberOpaqueId

PROVIDER
src/event/valid_events_a.rs:422
420:     "CLUB_MEMBERS_UPDATED",
421:     "CLUB_MEMBER_ADDED",
422:     "CLUB_MEMBER_PRESENCE_UPDATED",
423:     "CLUB_MEMBER_REMOVED",
424:     "CLUB_MEMBER_ROLE_UPDATED",
425:     "CLUB_MEMBER_UPDATED",
426:     "CLUB_MESSAGE_ADDED",
427:     "CLUB_MESSAGE_HISTORY_RECEIVED",
428:     "CLUB_MESSAGE_UPDATED",
429:     "CLUB_REMOVED",
430:     "CLUB_REMOVED_MESSAGE",
431:     "CLUB_SELF_MEMBER_ROLE_UPDATED",
432:     "CLUB_STREAMS_LOADED",
433:     "CLUB_STREAM_ADDED",
434:     "CLUB_STREAM_REMOVED",
435:     "CLUB_STREAM_SUBSCRIBED",
436:     "CLUB_STREAM_UNSUBSCRIBED",
437:     "CLUB_STREAM_UPDATED",
438:     "CLUB_TICKETS_RECEIVED",
439:     "CLUB_TICKET_CREATED",
440:     "CLUB_TICKET_RECEIVED",
441:     "CLUB_UPDATED",
442:     "COLOR_OVERRIDES_RESET",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1302
1300: 			Name = "ClubMemberPresenceUpdated",
1301: 			Type = "Event",
1302: 			LiteralName = "CLUB_MEMBER_PRESENCE_UPDATED",
1303: 			SynchronousEvent = true,
1304: 			Payload =
1305: 			{
1306: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1307: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1308: 				{ Name = "presence", Type = "ClubMemberPresence", Nilable = false },
1309: 			},
1310: 		},
1311: 		{
1312: 			Name = "ClubMemberRemoved",
1313: 			Type = "Event",
1314: 			LiteralName = "CLUB_MEMBER_REMOVED",
1315: 			SynchronousEvent = true,
1316: 			Payload =
1317: 			{
1318: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1319: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1320: 			},
1321: 		},
1322: 		{
1323: 			Name = "ClubMemberRoleUpdated",
1324: 			Type = "Event",
1325: 			LiteralName = "CLUB_MEMBER_ROLE_UPDATED",
1326: 			SynchronousEvent = true,
1327: 			Payload =
1328: 			{

## events-CLUB_MEMBER_REMOVED-160
SOURCE CLUB_MEMBER_REMOVED [2].Type number -> ClubMemberOpaqueId

PROVIDER
src/event/valid_events_a.rs:423
421:     "CLUB_MEMBER_ADDED",
422:     "CLUB_MEMBER_PRESENCE_UPDATED",
423:     "CLUB_MEMBER_REMOVED",
424:     "CLUB_MEMBER_ROLE_UPDATED",
425:     "CLUB_MEMBER_UPDATED",
426:     "CLUB_MESSAGE_ADDED",
427:     "CLUB_MESSAGE_HISTORY_RECEIVED",
428:     "CLUB_MESSAGE_UPDATED",
429:     "CLUB_REMOVED",
430:     "CLUB_REMOVED_MESSAGE",
431:     "CLUB_SELF_MEMBER_ROLE_UPDATED",
432:     "CLUB_STREAMS_LOADED",
433:     "CLUB_STREAM_ADDED",
434:     "CLUB_STREAM_REMOVED",
435:     "CLUB_STREAM_SUBSCRIBED",
436:     "CLUB_STREAM_UNSUBSCRIBED",
437:     "CLUB_STREAM_UPDATED",
438:     "CLUB_TICKETS_RECEIVED",
439:     "CLUB_TICKET_CREATED",
440:     "CLUB_TICKET_RECEIVED",
441:     "CLUB_UPDATED",
442:     "COLOR_OVERRIDES_RESET",
443:     "COLOR_OVERRIDE_UPDATED",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1314
1312: 			Name = "ClubMemberRemoved",
1313: 			Type = "Event",
1314: 			LiteralName = "CLUB_MEMBER_REMOVED",
1315: 			SynchronousEvent = true,
1316: 			Payload =
1317: 			{
1318: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1319: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1320: 			},
1321: 		},
1322: 		{
1323: 			Name = "ClubMemberRoleUpdated",
1324: 			Type = "Event",
1325: 			LiteralName = "CLUB_MEMBER_ROLE_UPDATED",
1326: 			SynchronousEvent = true,
1327: 			Payload =
1328: 			{
1329: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1330: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1331: 				{ Name = "roleId", Type = "number", Nilable = false },
1332: 			},
1333: 		},
1334: 		{
1335: 			Name = "ClubMemberUpdated",
1336: 			Type = "Event",
1337: 			LiteralName = "CLUB_MEMBER_UPDATED",
1338: 			SynchronousEvent = true,
1339: 			Payload =
1340: 			{

## events-CLUB_MEMBER_ROLE_UPDATED-161
SOURCE CLUB_MEMBER_ROLE_UPDATED [2].Type number -> ClubMemberOpaqueId

PROVIDER
src/event/valid_events_a.rs:424
422:     "CLUB_MEMBER_PRESENCE_UPDATED",
423:     "CLUB_MEMBER_REMOVED",
424:     "CLUB_MEMBER_ROLE_UPDATED",
425:     "CLUB_MEMBER_UPDATED",
426:     "CLUB_MESSAGE_ADDED",
427:     "CLUB_MESSAGE_HISTORY_RECEIVED",
428:     "CLUB_MESSAGE_UPDATED",
429:     "CLUB_REMOVED",
430:     "CLUB_REMOVED_MESSAGE",
431:     "CLUB_SELF_MEMBER_ROLE_UPDATED",
432:     "CLUB_STREAMS_LOADED",
433:     "CLUB_STREAM_ADDED",
434:     "CLUB_STREAM_REMOVED",
435:     "CLUB_STREAM_SUBSCRIBED",
436:     "CLUB_STREAM_UNSUBSCRIBED",
437:     "CLUB_STREAM_UPDATED",
438:     "CLUB_TICKETS_RECEIVED",
439:     "CLUB_TICKET_CREATED",
440:     "CLUB_TICKET_RECEIVED",
441:     "CLUB_UPDATED",
442:     "COLOR_OVERRIDES_RESET",
443:     "COLOR_OVERRIDE_UPDATED",
444:     "COMBAT_LOG_ENTRIES_CLEARED",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1325
1323: 			Name = "ClubMemberRoleUpdated",
1324: 			Type = "Event",
1325: 			LiteralName = "CLUB_MEMBER_ROLE_UPDATED",
1326: 			SynchronousEvent = true,
1327: 			Payload =
1328: 			{
1329: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1330: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1331: 				{ Name = "roleId", Type = "number", Nilable = false },
1332: 			},
1333: 		},
1334: 		{
1335: 			Name = "ClubMemberUpdated",
1336: 			Type = "Event",
1337: 			LiteralName = "CLUB_MEMBER_UPDATED",
1338: 			SynchronousEvent = true,
1339: 			Payload =
1340: 			{
1341: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1342: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1343: 			},
1344: 		},
1345: 		{
1346: 			Name = "ClubMembersUpdated",
1347: 			Type = "Event",
1348: 			LiteralName = "CLUB_MEMBERS_UPDATED",
1349: 			UniqueEvent = true,
1350: 			Payload =
1351: 			{

## events-CLUB_MEMBER_UPDATED-162
SOURCE CLUB_MEMBER_UPDATED [2].Type number -> ClubMemberOpaqueId

PROVIDER
src/event/valid_events_a.rs:425
423:     "CLUB_MEMBER_REMOVED",
424:     "CLUB_MEMBER_ROLE_UPDATED",
425:     "CLUB_MEMBER_UPDATED",
426:     "CLUB_MESSAGE_ADDED",
427:     "CLUB_MESSAGE_HISTORY_RECEIVED",
428:     "CLUB_MESSAGE_UPDATED",
429:     "CLUB_REMOVED",
430:     "CLUB_REMOVED_MESSAGE",
431:     "CLUB_SELF_MEMBER_ROLE_UPDATED",
432:     "CLUB_STREAMS_LOADED",
433:     "CLUB_STREAM_ADDED",
434:     "CLUB_STREAM_REMOVED",
435:     "CLUB_STREAM_SUBSCRIBED",
436:     "CLUB_STREAM_UNSUBSCRIBED",
437:     "CLUB_STREAM_UPDATED",
438:     "CLUB_TICKETS_RECEIVED",
439:     "CLUB_TICKET_CREATED",
440:     "CLUB_TICKET_RECEIVED",
441:     "CLUB_UPDATED",
442:     "COLOR_OVERRIDES_RESET",
443:     "COLOR_OVERRIDE_UPDATED",
444:     "COMBAT_LOG_ENTRIES_CLEARED",
445:     "COMBAT_LOG_EVENT",

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1337
1335: 			Name = "ClubMemberUpdated",
1336: 			Type = "Event",
1337: 			LiteralName = "CLUB_MEMBER_UPDATED",
1338: 			SynchronousEvent = true,
1339: 			Payload =
1340: 			{
1341: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1342: 				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
1343: 			},
1344: 		},
1345: 		{
1346: 			Name = "ClubMembersUpdated",
1347: 			Type = "Event",
1348: 			LiteralName = "CLUB_MEMBERS_UPDATED",
1349: 			UniqueEvent = true,
1350: 			Payload =
1351: 			{
1352: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1353: 			},
1354: 		},
1355: 		{
1356: 			Name = "ClubMessageAdded",
1357: 			Type = "Event",
1358: 			LiteralName = "CLUB_MESSAGE_ADDED",
1359: 			SynchronousEvent = true,
1360: 			Payload =
1361: 			{
1362: 				{ Name = "clubId", Type = "ClubId", Nilable = false },
1363: 				{ Name = "streamId", Type = "ClubStreamId", Nilable = false },

## events-ENCOUNTER_END-163
SOURCE ENCOUNTER_END + encounterUnitStatus

PROVIDER
src/lua_api/globals/admin_encounter.rs:53
51:         state.push(*value);
52:     }
53:     let result = dispatch_event_now(state, "ENCOUNTER_END", payload);
54:     state.top = saved_top;
55:     result
56: }
57: 
58: pub(super) fn start_loot_roll(state: &mut LuaState) -> LuaResult<u32> {
59:     let roll_id = i32::from_stack(state, 1)?;
60:     let roll_time = f64::from_stack(state, 2)?;
61:     let info = build_loot_roll_info(state, roll_id, roll_time);
62:     let mut st = borrow_state_mut(state)?;
63:     st.world.loot_rolls.insert(roll_id, info);
64:     drop(st);
65:     dispatch_event_now(
66:         state,
67:         "START_LOOT_ROLL",
68:         &[Val::Num(roll_id as f64), Val::Num(roll_time)],
69:     )?;
70:     Ok(0)
71: }
72: 
73: fn build_loot_roll_info(

DECLARATION
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterInfoDocumentation.lua:40
38: 			Name = "EncounterEnd",
39: 			Type = "Event",
40: 			LiteralName = "ENCOUNTER_END",
41: 			SynchronousEvent = true,
42: 			Payload =
43: 			{
44: 				{ Name = "encounterID", Type = "number", Nilable = false },
45: 				{ Name = "encounterName", Type = "cstring", Nilable = false },
46: 				{ Name = "difficultyID", Type = "number", Nilable = false },
47: 				{ Name = "groupSize", Type = "number", Nilable = false },
48: 				{ Name = "success", Type = "number", Nilable = false },
49: 				{ Name = "encounterUnitStatus", Type = "table", InnerType = "EncounterUnitStatus", Nilable = false, Documentation = { "List of all boss units engaged during this encounter." } },
50: 			},
51: 		},
52: 		{
53: 			Name = "EncounterStart",
54: 			Type = "Event",
55: 			LiteralName = "ENCOUNTER_START",
56: 			SynchronousEvent = true,
57: 			Payload =
58: 			{
59: 				{ Name = "encounterID", Type = "number", Nilable = false },
60: 				{ Name = "encounterName", Type = "cstring", Nilable = false },
61: 				{ Name = "difficultyID", Type = "number", Nilable = false },
62: 				{ Name = "groupSize", Type = "number", Nilable = false },
63: 			},
64: 		},
65: 		{
66: 			Name = "InstanceLockStart",

## source-context-165
SOURCE CVars Added examples from crawled page:

PROVIDER


DECLARATION


## cvars-assistedCombatReduceHighlights-166
SOURCE assistedCombatReduceHighlights

PROVIDER
src/cvars.rs:357
355: #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
356: const PATCH_12_0_7_CVARS: &[(&str, &str)] = &[
357:     ("assistedCombatReduceHighlights", "1"),
358:     ("developerLog", "0"),
359:     ("developerLogFilterDebug", "0"),
360:     ("developerLogFilterError", "1"),
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]

src/cvars.rs:595
593:             );
594:         }
595:         assert!(storage.get_bool("assistedCombatReduceHighlights"));
596:         assert!(!storage.get_bool("developerLogFilterDebug"));
597:         assert_eq!(
598:             storage.get("gxWindowedResolution"),
599:             Some("auto".to_string())
600:         );
601:         for name in PATCH_12_0_7_REMOVED_CVARS {
602:             assert_eq!(storage.get(name), None, "{name}");
603:         }
604:     }
605: }

DECLARATION


## cvars-developerLogFilterDebug-167
SOURCE developerLogFilterDebug

PROVIDER
src/cvars.rs:359
357:     ("assistedCombatReduceHighlights", "1"),
358:     ("developerLog", "0"),
359:     ("developerLogFilterDebug", "0"),
360:     ("developerLogFilterError", "1"),
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]
378: const PATCH_12_1_REMOVED_CVARS: &[&str] =
379:     &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];

src/cvars.rs:596
594:         }
595:         assert!(storage.get_bool("assistedCombatReduceHighlights"));
596:         assert!(!storage.get_bool("developerLogFilterDebug"));
597:         assert_eq!(
598:             storage.get("gxWindowedResolution"),
599:             Some("auto".to_string())
600:         );
601:         for name in PATCH_12_0_7_REMOVED_CVARS {
602:             assert_eq!(storage.get(name), None, "{name}");
603:         }
604:     }
605: }

DECLARATION


## cvars-developerLogFilterError-168
SOURCE developerLogFilterError

PROVIDER
src/cvars.rs:360
358:     ("developerLog", "0"),
359:     ("developerLogFilterDebug", "0"),
360:     ("developerLogFilterError", "1"),
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]
378: const PATCH_12_1_REMOVED_CVARS: &[&str] =
379:     &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];
380: 

DECLARATION


## cvars-developerLogFilterFatal-169
SOURCE developerLogFilterFatal

PROVIDER
src/cvars.rs:361
359:     ("developerLogFilterDebug", "0"),
360:     ("developerLogFilterError", "1"),
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]
378: const PATCH_12_1_REMOVED_CVARS: &[&str] =
379:     &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];
380: 
381: #[cfg(feature = "retail-12-1-0")]

DECLARATION


## cvars-developerLogFilterNormal-170
SOURCE developerLogFilterNormal

PROVIDER
src/cvars.rs:362
360:     ("developerLogFilterError", "1"),
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]
378: const PATCH_12_1_REMOVED_CVARS: &[&str] =
379:     &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];
380: 
381: #[cfg(feature = "retail-12-1-0")]
382: const PATCH_12_1_CVARS: &[(&str, &str)] = &[

DECLARATION


## cvars-developerLogFilterSpam-171
SOURCE developerLogFilterSpam

PROVIDER
src/cvars.rs:363
361:     ("developerLogFilterFatal", "1"),
362:     ("developerLogFilterNormal", "1"),
363:     ("developerLogFilterSpam", "0"),
364:     ("developerLogFilterWarning", "1"),
365:     ("developerLogWriteToFile", "1"),
366:     ("housingDecorLightRadiusIndicatorsEnabled", "1"),
367:     ("housingOtherDecorLightRadiusIndicatorType", "1"),
368:     ("housingSelectedDecorLightRadiusIndicatorType", "1"),
369:     ("KioskCanSessionExpire", "1"),
370:     ("KioskCharacterTemplateSet", "0"),
371:     ("KioskLobbyKickSeconds", "30"),
372:     ("ThreadPoolPerThreadAllocator", "1"),
373:     ("useBLEEP", "0"),
374:     ("gxWindowedResolution", "auto"),
375: ];
376: 
377: #[cfg(feature = "retail-12-1-0")]
378: const PATCH_12_1_REMOVED_CVARS: &[&str] =
379:     &["lastLockedDelvesCompanionAbilities", "SlugSupersampling"];
380: 
381: #[cfg(feature = "retail-12-1-0")]
382: const PATCH_12_1_CVARS: &[(&str, &str)] = &[
383:     ("accessibilityScreenNarrationEnabled", "0"),

DECLARATION


## source-context-172
SOURCE (20 added, 5 removed; crawler excerpt did not include full list.)

PROVIDER


DECLARATION


## deprecated-api-174
SOURCE Deprecated API added:

PROVIDER


DECLARATION


## deprecated-api-175
SOURCE 12.0.7 Deprecated_12_0_7.lua: C_ClickBindings.MakeModifiers -> MakeModifiers; C_ClickBindings.GetStringFromModifiers -> GetStringFromModifiers; C_Spell.GetMawPowerBorderAtlasBySpellID -> C_Spell.GetMawPowerRarityInfoBySpellID; GetMerchantCurrencies -> C_MerchantFrame.GetMerchantCurrencies.

PROVIDER
/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:8
6: end
7: 
8: function MenuUtil.ShowTooltip(owner, func, ...)
9: 	MenuUtil.ShowTooltipEx(owner, GetAppropriateTooltip(), func, ...);
10: end
11: 
12: function MenuUtil.HideTooltip(owner)
13: 	MenuUtil.HideTooltipEx(owner, GetAppropriateTooltip());
14: end
15: 
16: function C_ClickBindings.MakeModifiers(...)
17: 	return MakeModifiers();
18: end
19: 
20: function C_ClickBindings.GetStringFromModifiers(modifiers)
21: 	return GetStringFromModifiers(modifiers);
22: end
23: 
24: function C_Spell.GetMawPowerBorderAtlasBySpellID(spellID)
25: 	local _rarityID, rarityAtlas = C_Spell.GetMawPowerRarityInfoBySpellID(spellID);
26: 	return rarityAtlas;
27: end
28: 

/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:12
10: end
11: 
12: function MenuUtil.HideTooltip(owner)
13: 	MenuUtil.HideTooltipEx(owner, GetAppropriateTooltip());
14: end
15: 
16: function C_ClickBindings.MakeModifiers(...)
17: 	return MakeModifiers();
18: end
19: 
20: function C_ClickBindings.GetStringFromModifiers(modifiers)
21: 	return GetStringFromModifiers(modifiers);
22: end
23: 
24: function C_Spell.GetMawPowerBorderAtlasBySpellID(spellID)
25: 	local _rarityID, rarityAtlas = C_Spell.GetMawPowerRarityInfoBySpellID(spellID);
26: 	return rarityAtlas;
27: end
28: 
29: function GetMerchantCurrencies()
30: 	return unpack(C_MerchantFrame.GetMerchantCurrencies());
31: end

DECLARATION


## deprecated-api-176
SOURCE Deprecated_BattleNet.lua: BNInviteFriend -> C_BattleNet.InviteFriend.

PROVIDER


DECLARATION


## deprecated-api-177
SOURCE Deprecated_PartyInfo.lua: ConfirmReadyCheck/DemoteAssistant/DoReadyCheck/PromoteToAssistant/PromoteToLeader/SetEveryoneIsAssistant/UninviteUnit/IsGUIDInGroup -> C_PartyInfo namespace.

PROVIDER


DECLARATION


## deprecated-api-178
SOURCE 12.0.5 Deprecated_AutoComplete.lua wrappers: GetAutoCompletePresenceID/GetAutoCompleteResults/GetAutoCompleteRealms/IsRecognizedName -> C_AutoComplete namespace.

PROVIDER


DECLARATION

