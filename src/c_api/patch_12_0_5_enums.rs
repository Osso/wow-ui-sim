//! Documented numeric publication; no downstream domain behavior implied.

use crate::lua_api::methods::{table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    let enums = super::ensure_global_table(state, "Enum");
    for &(name, member, value) in ADDITIONS {
        let values = table_get(state, enums, name);
        table_set(state, values, member, Val::Num(f64::from(value)));
        publish_metadata(state, enums, name, values);
    }
    #[cfg(feature = "client-retail")]
    publish_current_retail(state, enums);
}

pub(crate) fn refresh_metadata(state: &mut LuaState) {
    let enums = super::ensure_global_table(state, "Enum");
    for &(name, _, _) in ADDITIONS {
        let values = table_get(state, enums, name);
        publish_metadata(state, enums, name, values);
    }
    #[cfg(feature = "client-retail")]
    for &(name, _) in CURRENT_RETAIL_VALUES {
        let values = table_get(state, enums, name);
        publish_metadata(state, enums, name, values);
    }
}

#[cfg(feature = "client-retail")]
fn publish_current_retail(state: &mut LuaState, enums: Val) {
    for &(name, member) in CURRENT_RETAIL_REMOVALS {
        let values = table_get(state, enums, name);
        table_set(state, values, member, Val::Nil);
    }
    for &(name, members) in CURRENT_RETAIL_VALUES {
        let values = table_get(state, enums, name);
        for &(member, value) in members {
            table_set(state, values, member, Val::Num(f64::from(value)));
        }
        publish_metadata(state, enums, name, values);
    }
}

fn publish_metadata(state: &mut LuaState, enums: Val, name: &str, values: Val) {
    let Val::Table(values_ref) = values else {
        panic!("Enum.{name} must be initialized before 12.0.5 publication");
    };
    let members = state.gc.tables.get(values_ref).unwrap().hash_entries();
    let numbers = members.iter().filter_map(|(_, value)| match value {
        Val::Num(number) => Some(*number),
        _ => None,
    });
    let (count, min, max) = numbers.fold(
        (0, f64::INFINITY, f64::NEG_INFINITY),
        |(count, min, max), value| (count + 1, min.min(value), max.max(value)),
    );
    let metadata = table_get(state, enums, &format!("{name}Meta"));
    table_set(state, metadata, "NumValues", Val::Num(f64::from(count)));
    table_set(state, metadata, "MinValue", Val::Num(min));
    table_set(state, metadata, "MaxValue", Val::Num(max));
}

// Previously published under the cumulative retail-12-0-5 feature, including PTR
// and historical retail builds. Preserve that scope and its later members.
// CurrencyConstantsDocumentation.lua:95–114, TransmogSharedDocumentation.lua:46–56,
// PlayerHousingConstantsDocumentation.lua:52–68.
const ADDITIONS: &[(&str, &str, i32)] = &[
    ("CurrencyFlagsB", "CurrencyBNoBonusXP", 2048),
    ("TransmogIllusionFlags", "AllowedRangedShieldsHoldables", 4),
    ("HouseFinderSuggestionReason", "HomeOwner", 64),
];

// Complete cached enumerations omit these names. DebuffIconSize is IconSize's
// current name.
#[cfg(feature = "client-retail")]
const CURRENT_RETAIL_REMOVALS: &[(&str, &str)] = &[
    ("AbbreviationDataError", "InvalidAbbreviation"),
    ("HousingItemToastType", "House"),
    ("LootMethodStyles", "PersonalOnly"),
    ("EditModeUnitFrameSetting", "IconSize"),
];

// Literal current-retail cache values, not inferred historical 12.0.5 positions.
// Only affected members and required shifted values are corrected; unrelated
// and later cumulative members remain. See the spec for the 33-row source map.
#[cfg(feature = "client-retail")]
const CURRENT_RETAIL_VALUES: &[(&str, &[(&str, i32)])] = &[
    // Retained 12.0.1 additions: CombatAudioAlertSharedDocumentation.lua:160–177.
    (
        "CombatAudioAlertSpecSetting",
        &[
            ("Resource1Percent", 0),
            ("Resource1Format", 1),
            ("Resource1Voice", 2),
            ("Resource1Volume", 3),
            ("Resource2Percent", 4),
            ("Resource2Format", 5),
            ("Resource2Voice", 6),
            ("Resource2Volume", 7),
            ("SayIfTargeted", 8),
        ],
    ),
    // LocalizationSharedDocumentation.lua:6–17.
    (
        "AbbreviationDataError",
        &[
            ("InvalidBreakpoint", 1),
            ("InvalidSignificandDivisor", 2),
            ("InvalidFractionDivisor", 4),
            ("NotMultipleOfTen", 8),
        ],
    ),
    // RestrictedActionsConstantsDocumentation.lua:19–32.
    ("AddOnRestrictionType", &[("Chat", 5)]),
    // EditModeManagerConstantsDocumentation.lua:202–245, 610–621, 636–670, 684–714.
    ("EditModeAccountSetting", &[("ShowTotemActionBar", 34)]),
    ("EditModeStatusTrackingBarSetting", &[("Size", 3)]),
    ("EditModeSystem", &[("TotemActionBar", 25)]),
    (
        "EditModeUnitFrameSetting",
        &[
            ("DebuffIconSize", 19),
            ("BigDefensiveIconSize", 21),
            ("BuffIconSize", 22),
        ],
    ),
    // WowCSConstantsDocumentation.lua:6–90.
    (
        "FragmentID",
        &[
            ("FPathingDynamicLinks", 40),
            ("TagHousingDecorProxyGameObject", 226),
        ],
    ),
    // TutorialDocumentation.lua:83–140.
    ("FrameTutorialAccount", &[("HousingEndeavorsTabSeen", 48)]),
    // PlayerHousingConstantsDocumentation.lua:37–49, 260–379.
    ("HouseExteriorWMODataFlags", &[("HiddenUnlessOwned", 8)]),
    (
        "HousingResult",
        &[("BoundToStartingArea", 19), ("InvalidLightOverlap", 60)],
    ),
    // HousingDecorSharedDocumentation.lua:6–20.
    (
        "HousingDecorPlacementRestriction",
        &[("InvalidLightOverlap", 64)],
    ),
    // HousingUIDocumentation.lua:945–957.
    ("HousingItemToastType", &[("HouseType", 4)]),
    // LootConstantsDocumentation.lua:22–31.
    ("LootMethodStyles", &[("Mainline", 0)]),
    // ImageSharingConstantsDocumentation.lua:50–87. Inserting Disabled shifts
    // every previous status, so retaining their old values would be incorrect.
    (
        "PhotoSharingUploadStatus",
        &[
            ("Disabled", 0),
            ("Failed", 1),
            ("Success", 2),
            ("Locked", 3),
            ("ListPagesApiCallFailed", 4),
            ("ListPagesBadRequest", 5),
            ("ListPagesUnauthorized", 6),
            ("ListPagesForbidden", 7),
            ("ListPagesNotFound", 8),
            ("ListPagesTooManyRequests", 9),
            ("ListPagesGenericFailure", 10),
            ("ListPagesEmptyBoardID", 11),
            ("ListPagesInvalidBookmark", 12),
            ("CreatePageApiCallFailed", 13),
            ("CreatePageBadRequest", 14),
            ("CreatePageUnauthorized", 15),
            ("CreatePageForbidden", 16),
            ("CreatePageNotFound", 17),
            ("CreatePageTooManyRequests", 18),
            ("CreatePageGenericFailure", 19),
            ("CreatePageNoBoardIDFound", 20),
            ("CreatePostApiCallFailed", 21),
            ("CreatePostBadRequest", 22),
            ("CreatePostUnauthorized", 23),
            ("CreatePostForbidden", 24),
            ("CreatePostNotFound", 25),
            ("CreatePostTooManyRequests", 26),
            ("CreatePostGenericFailure", 27),
            ("CreatePostNoIDFound", 28),
            ("CreatePostThrottled", 29),
        ],
    ),
    // WowSurveyConstantsDocumentation.lua:18–31.
    ("SurveyDeliveryMoment", &[("EncounterEnd", 5)]),
    // TransmogOutfitConstantsDocumentation.lua:291–330, 361–379.
    (
        "TransmogSituation",
        &[
            ("AllWeather", 22),
            ("WeatherClear", 23),
            ("WeatherRain", 24),
            ("WeatherSnow", 25),
            ("WeatherSand", 26),
            ("AllTime", 27),
            ("TimeMorning", 28),
            ("TimeDay", 29),
            ("TimeEvening", 30),
            ("TimeNight", 31),
        ],
    ),
    (
        "TransmogSituationTrigger",
        &[("Weather", 9), ("TimeOfDay", 10)],
    ),
];
