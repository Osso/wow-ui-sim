//! Documented numeric publication; no downstream domain behavior implied.
//!
//! Exact cached 12.1.0 values replace generic max+1 appends, which misplace
//! flag bits and members inserted before existing ones.

use crate::lua_api::methods::{table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

/// Enum name, documented members, and documented (MinValue, MaxValue, NumValues).
type EnumDecl = (
    &'static str,
    &'static [(&'static str, i32)],
    (i32, i32, i32),
);

pub(crate) fn register(state: &mut LuaState) {
    let enums = super::ensure_global_table(state, "Enum");
    for decl in SHARED {
        publish(state, enums, decl);
    }
    // PTR publishes its own 12.1.5 FragmentID and TooltipDataLineType tables.
    #[cfg(not(feature = "retail-12-1-5"))]
    for decl in RETAIL_ONLY {
        publish(state, enums, decl);
    }
}

fn publish(state: &mut LuaState, enums: Val, &(name, members, (min, max, count)): &EnumDecl) {
    let values = table_get(state, enums, name);
    let metadata = table_get(state, enums, &format!("{name}Meta"));
    assert!(
        matches!((&values, &metadata), (Val::Table(_), Val::Table(_))),
        "Enum.{name} and its Meta must be initialized before 12.1.0 publication"
    );
    for &(member, value) in members {
        table_set(state, values, member, Val::Num(f64::from(value)));
    }
    // Metadata is native publication; Blizzard Lua may add members it never counts.
    table_set(state, metadata, "MinValue", Val::Num(f64::from(min)));
    table_set(state, metadata, "MaxValue", Val::Num(f64::from(max)));
    table_set(state, metadata, "NumValues", Val::Num(f64::from(count)));
}

// Retail and PTR caches declare these identically.
const SHARED: &[EnumDecl] = &[
    // DelvesConstantsDocumentation.lua:6–18.
    ("CompanionConfigSlotTypes", &[("Flavor", 3)], (0, 3, 4)),
    // CooldownViewerConstantsDocumentation.lua:71–88.
    (
        "CooldownViewerCategory",
        &[
            ("GroupBuff", 4),
            ("SpecAgnosticEssential", 5),
            ("SpecAgnosticTracked", 6),
            ("EquipSlotEssential", 7),
            ("EquipSlotTracked", 8),
        ],
        (0, 8, 9),
    ),
    // EditModeManagerConstantsDocumentation.lua:522–534.
    ("EditModeMinimapSetting", &[("IconScale", 3)], (0, 3, 4)),
    // TutorialDocumentation.lua:83–141.
    (
        "FrameTutorialAccount",
        &[("RunesOfPower", 49), ("HousingPetBeds", 50)],
        (1, 50, 50),
    ),
    // PlayerHousingConstantsDocumentation.lua:52–69 (flag bits).
    (
        "HouseFinderSuggestionReason",
        &[("Relinquished", 128)],
        (0, 128, 9),
    ),
    // NamePlateConstantsDocumentation.lua:121–136.
    ("NamePlateStyle", &[("Classic", 6)], (0, 6, 7)),
    // PingConstantsDocumentation.lua:18–35, 50–68.
    ("PingResult", &[("FailedSilent", 8)], (0, 8, 9)),
    (
        "PingSubjectType",
        &[
            ("ActionReady", 6),
            ("ActionOnCooldown", 7),
            ("ActionUnavailable", 8),
            ("ActionNotReady", 9),
        ],
        (0, 9, 10),
    ),
    // SecretAspectConstantsDocumentation.lua:6–44 (flag bits).
    (
        "SecretAspect",
        &[
            ("TooltipTexture", 1_048_576),
            ("ButtonState", 2_097_152),
            ("ScrollOffset", 4_194_304),
            ("RadialProgress", 8_388_608),
        ],
        (1, 8_388_608, 30),
    ),
];

#[cfg(not(feature = "retail-12-1-5"))]
const RETAIL_ONLY: &[EnumDecl] = &[
    // WowCSConstantsDocumentation.lua:6–91.
    (
        "FragmentID",
        &[
            ("FPathingDoor", 41),
            ("FWorldStateListenerData", 42),
            ("FMapObject", 43),
        ],
        (0, 255, 77),
    ),
    // TooltipInfoSharedDocumentation.lua:27–85.
    (
        "TooltipDataLineType",
        &[
            ("ItemSpellTriggerOnUse", 44),
            ("ItemSpellTriggerOnEquip", 45),
            ("ItemSpellTriggerOnProc", 46),
            ("UnitLevel", 47),
            ("UnitType", 48),
            ("UnitDead", 49),
        ],
        (0, 49, 50),
    ),
];
