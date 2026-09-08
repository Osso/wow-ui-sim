//! Native aura option structures, from AuraContainerUtilDocumentation.lua.
//! Copies recognized fields and applies only the documented defaults.

use crate::c_api::{ensure_namespace, global_val};
use crate::lua_api::methods::{
    call_function_state, create_string, create_table, table_get, table_set, table_set_num,
    table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Copy)]
enum Kind {
    Number,
    Bool,
    String,
    Texture,
    Enum(u8),
    DrawLayer,
    Color(bool),
    Object(&'static str, &'static [&'static str]),
    Structure(&'static [Field]),
    Map(&'static Kind),
    Sequence(&'static Kind),
}

#[derive(Clone, Copy)]
enum Presence {
    Required,
    Optional,
    Number(f64),
    Bool(bool),
}

type Field = (&'static str, Kind, Presence);
use Kind::{
    Bool, DrawLayer, Enum, Map, Number, Object, Sequence, String as Text, Structure, Texture,
};
use Presence::{Optional, Required};

const FORMATTER: Kind = Object("NumericFormatter", &["FormatNumber"]);
const COLOR_CURVE: Kind = Object("LuaColorCurveObject", &["Evaluate", "Copy"]);
const DURATION_BINDING: Kind = Object("DurationTextBinding", &["GetDuration", "SetDuration"]);
const ANCHOR_OFFSETS: &[Field] = &[
    ("left", Number, Presence::Number(0.0)),
    ("right", Number, Presence::Number(0.0)),
    ("top", Number, Presence::Number(0.0)),
    ("bottom", Number, Presence::Number(0.0)),
];
const TEX_COORDS: &[Field] = &[
    ("left", Number, Presence::Number(0.0)),
    ("right", Number, Presence::Number(1.0)),
    ("top", Number, Presence::Number(0.0)),
    ("bottom", Number, Presence::Number(1.0)),
];
const SLICE_MARGINS: &[Field] = &[
    ("left", Number, Required),
    ("top", Number, Required),
    ("right", Number, Required),
    ("bottom", Number, Required),
];
const BACKDROP_INFO: &[Field] = &[
    ("bgFile", Texture, Optional),
    ("edgeFile", Texture, Optional),
    ("edgeSize", Number, Optional),
    ("insets", Structure(ANCHOR_OFFSETS), Optional),
    ("tile", Bool, Optional),
    ("tileEdge", Bool, Optional),
    ("tileSize", Number, Optional),
];
const BACKDROP: &[Field] = &[
    ("backdropInfo", Structure(BACKDROP_INFO), Required),
    ("borderColor", Kind::Color(true), Optional),
    ("centerColor", Kind::Color(true), Optional),
    ("anchorOffsets", Structure(ANCHOR_OFFSETS), Optional),
];
const NINE_SLICE: &[Field] = &[
    ("layoutName", Text, Required),
    ("borderColor", Kind::Color(true), Optional),
    ("centerColor", Kind::Color(true), Optional),
    ("anchorOffsets", Structure(ANCHOR_OFFSETS), Optional),
];
const TEXTURE_SLICE: &[Field] = &[
    ("asset", Texture, Required),
    ("sliceMargins", Structure(SLICE_MARGINS), Optional),
    ("sliceMode", Enum(1), Optional),
    ("color", Kind::Color(true), Optional),
    ("anchorOffsets", Structure(ANCHOR_OFFSETS), Optional),
    ("drawLayer", DrawLayer, Optional),
    ("drawLayerSublevel", Number, Presence::Number(0.0)),
];
const APPLICATION_BAR: &[Field] = &[
    ("maxApplications", Number, Required),
    ("interpolation", Enum(1), Optional),
];
const APPLICATION_COUNT: &[Field] = &[("formatter", FORMATTER, Optional)];
const DISPEL_TEXT: &[Field] = &[
    ("showWhenHarmful", Bool, Presence::Bool(true)),
    ("showWhenHelpful", Bool, Presence::Bool(false)),
    ("showWithoutDispelType", Bool, Presence::Bool(false)),
    ("customDispelTextMap", Map(&Text), Optional),
];
const TEXTURE_ASSET: &[Field] = &[
    ("asset", Texture, Required),
    ("useAtlasSize", Bool, Presence::Bool(false)),
    ("texCoords", Structure(TEX_COORDS), Optional),
];
const DISPEL_TEXTURE: &[Field] = &[
    ("showAlways", Bool, Presence::Bool(false)),
    ("showWhenHarmful", Bool, Presence::Bool(true)),
    ("showWhenHelpful", Bool, Presence::Bool(false)),
    ("showWithoutDispelType", Bool, Presence::Bool(false)),
    ("stealableFilter", Enum(1), Optional),
    // AuraContainerSharedDocumentation: BorderWithIcon = 1.
    ("style", Enum(4), Presence::Number(1.0)),
    (
        "customDispelAssetMap",
        Map(&Structure(TEXTURE_ASSET)),
        Optional,
    ),
    ("customDispelColorMap", Map(&Kind::Color(false)), Optional),
    ("customDispelColorCurve", COLOR_CURVE, Optional),
];
const DURATION_BAR: &[Field] = &[
    ("interpolation", Enum(1), Optional),
    ("direction", Enum(1), Optional),
];
const FORMAT_COMPONENT: &[Field] = &[
    ("property", Enum(6), Required),
    ("formatter", FORMATTER, Required),
];
const TEXT_FORMAT: &[Field] = &[
    ("formatString", Text, Required),
    (
        "components",
        Sequence(&Structure(FORMAT_COMPONENT)),
        Required,
    ),
];
const TEXT_COLOR: &[Field] = &[
    ("curve", COLOR_CURVE, Required),
    ("property", Enum(6), Required),
];
const DURATION_TEXT: &[Field] = &[
    ("binding", DURATION_BINDING, Optional),
    ("textFormatter", FORMATTER, Optional),
    ("textFormat", Structure(TEXT_FORMAT), Optional),
    ("textColor", Structure(TEXT_COLOR), Optional),
];

fn register_duration_property_enum(state: &mut LuaState) -> LuaResult<()> {
    let enums = ensure_namespace(state, "Enum")?;
    let values = create_table(state);
    for (index, name) in [
        "RemainingDuration",
        "RemainingPercent",
        "ElapsedDuration",
        "ElapsedPercent",
        "TotalDuration",
        "StartTime",
        "EndTime",
    ]
    .into_iter()
    .enumerate()
    {
        table_set_static(state, values, name, Val::Num(index as f64));
    }
    table_set_static(
        state,
        Val::Table(enums),
        "DurationTextBindingProperty",
        values,
    );
    let metadata = create_table(state);
    for (name, value) in [("MinValue", 0.0), ("MaxValue", 6.0), ("NumValues", 7.0)] {
        table_set_static(state, metadata, name, Val::Num(value));
    }
    let enum_meta = ensure_namespace(state, "EnumMeta")?;
    table_set_static(
        state,
        Val::Table(enum_meta),
        "DurationTextBindingProperty",
        metadata,
    );
    Ok(())
}

macro_rules! processors {
    ($($function:ident => ($name:literal, $fields:ident, $nilable:literal)),+ $(,)?) => {
        pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
            register_duration_property_enum(state)?;
            let namespace = ensure_namespace(state, "C_AuraContainerUtil")?;
            $(table_set_rust_fn_static(state, namespace, $name, $function)?;)+
            Ok(())
        }
        $(fn $function(state: &mut LuaState) -> LuaResult<u32> {
            let options = stack_val(state, 1);
            if options == Val::Nil && !$nilable {
                return Err(runtime_error(concat!($name, ": options must be a table")));
            }
            let result = copy_structure(state, options, $fields, concat!($name, ".options"))?;
            state.push(result);
            Ok(1)
        })+
    };
}

processors! {
    tooltip_backdrop => ("ProcessAuraTooltipBackdropOptions", BACKDROP, false),
    tooltip_nine_slice => ("ProcessAuraTooltipNineSliceOptions", NINE_SLICE, false),
    tooltip_texture_slice => ("ProcessAuraTooltipTextureSliceOptions", TEXTURE_SLICE, false),
    application_bar => ("ProcessCustomAuraButtonApplicationBarOptions", APPLICATION_BAR, false),
    application_count => ("ProcessCustomAuraButtonApplicationCountOptions", APPLICATION_COUNT, true),
    dispel_text => ("ProcessCustomAuraButtonDispelTypeTextOptions", DISPEL_TEXT, true),
    dispel_texture => ("ProcessCustomAuraButtonDispelTypeTextureOptions", DISPEL_TEXTURE, true),
    duration_bar => ("ProcessCustomAuraButtonDurationBarOptions", DURATION_BAR, true),
    duration_text => ("ProcessCustomAuraButtonDurationTextOptions", DURATION_TEXT, true),
}

fn invalid(path: &str, expected: &str) -> rilua::LuaError {
    runtime_error(format!("{path}: expected {expected}"))
}

fn copy_structure(
    state: &mut LuaState,
    input: Val,
    fields: &[Field],
    path: &str,
) -> LuaResult<Val> {
    if !matches!(input, Val::Nil | Val::Table(_)) {
        return Err(invalid(path, "table"));
    }
    build_rooted_table(state, |state, output| {
        for &(name, kind, presence) in fields {
            let value = table_get(state, input, name);
            let field_path = format!("{path}.{name}");
            let value = normalize_field(state, value, kind, presence, &field_path)?;
            if value != Val::Nil {
                table_set(state, output, name, value);
            }
        }
        Ok(())
    })
}

fn normalize_field(
    state: &mut LuaState,
    value: Val,
    kind: Kind,
    presence: Presence,
    path: &str,
) -> LuaResult<Val> {
    if value != Val::Nil {
        return normalize_value(state, value, kind, path);
    }
    match presence {
        Required => Err(invalid(path, "required value")),
        Optional => Ok(Val::Nil),
        Presence::Number(value) => Ok(Val::Num(value)),
        Presence::Bool(value) => Ok(Val::Bool(value)),
    }
}

fn normalize_value(state: &mut LuaState, value: Val, kind: Kind, path: &str) -> LuaResult<Val> {
    match kind {
        Structure(fields) => copy_structure(state, value, fields, path),
        Map(inner) => copy_map(state, value, *inner, path),
        Sequence(inner) => copy_sequence(state, value, *inner, path),
        Kind::Color(alpha) => copy_color(state, value, alpha, path),
        Object(name, methods) => validate_object_reference(state, value, name, methods, path),
        scalar => validate_scalar(state, value, scalar, path),
    }
}

fn validate_scalar(state: &LuaState, value: Val, kind: Kind, path: &str) -> LuaResult<Val> {
    let valid = match (kind, value) {
        (Number, Val::Num(_)) | (Bool, Val::Bool(_)) | (Text, Val::Str(_)) => true,
        (Texture, Val::Num(_) | Val::Str(_)) => true,
        (Enum(max), Val::Num(value)) => {
            value >= 0.0 && value <= f64::from(max) && value.fract() == 0.0
        }
        (DrawLayer, Val::Str(key)) => state.gc.string_arena.get(key).is_some_and(|text| {
            matches!(
                text.data(),
                b"BACKGROUND" | b"BORDER" | b"ARTWORK" | b"OVERLAY" | b"HIGHLIGHT"
            )
        }),
        _ => false,
    };
    if valid {
        Ok(value)
    } else {
        Err(invalid(path, "documented field type or enum value"))
    }
}

fn validate_object_reference(
    state: &mut LuaState,
    value: Val,
    name: &str,
    methods: &[&str],
    path: &str,
) -> LuaResult<Val> {
    if !matches!(value, Val::Table(_) | Val::Userdata(_)) {
        return Err(invalid(path, name));
    }
    for method in methods {
        let key = create_string(state, method);
        if !matches!(state.gettable(value, key)?, Val::Function(_)) {
            return Err(invalid(path, name));
        }
    }
    // Formatter/binding/curve objects have their own state and copy contracts.
    Ok(value)
}

fn table_entries(state: &LuaState, value: Val, path: &str) -> LuaResult<Vec<(Val, Val)>> {
    let Val::Table(reference) = value else {
        return Err(invalid(path, "table"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| invalid(path, "table"))?;
    let mut entries: Vec<_> = table
        .array_slice()
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, value)| *value != Val::Nil)
        .map(|(index, value)| (Val::Num((index + 1) as f64), value))
        .collect();
    entries.extend(
        table
            .hash_entries()
            .into_iter()
            .filter(|(_, value)| *value != Val::Nil),
    );
    Ok(entries)
}

fn copy_map(state: &mut LuaState, input: Val, inner: Kind, path: &str) -> LuaResult<Val> {
    let entries = table_entries(state, input, path)?;
    build_rooted_table(state, |state, output| {
        for (key, value) in entries {
            let Val::Str(_) = key else {
                return Err(invalid(path, "string-keyed map"));
            };
            let normalized = normalize_value(state, value, inner, path)?;
            let Val::Table(output) = output else {
                unreachable!()
            };
            state.gc.tables.get_mut(output).unwrap().raw_set(
                key,
                normalized,
                &state.gc.string_arena,
            )?;
            state.gc.barrier_back(output);
        }
        Ok(())
    })
}

fn copy_sequence(state: &mut LuaState, input: Val, inner: Kind, path: &str) -> LuaResult<Val> {
    let mut entries = table_entries(state, input, path)?;
    if entries.iter().any(|(key, _)| !matches!(key, Val::Num(number) if number.is_finite() && *number >= 1.0 && number.fract() == 0.0)) {
        return Err(invalid(path, "dense array"));
    }
    entries.sort_by(|(a, _), (b, _)| match (a, b) {
        (Val::Num(a), Val::Num(b)) => a.total_cmp(b),
        _ => unreachable!("array keys were validated"),
    });
    build_rooted_table(state, |state, output| {
        let Val::Table(output_ref) = output else {
            unreachable!()
        };
        for (index, (key, value)) in entries.into_iter().enumerate() {
            if key != Val::Num((index + 1) as f64) {
                return Err(invalid(path, "dense array"));
            }
            let entry_path = format!("{path}[{}]", index + 1);
            let normalized = normalize_value(state, value, inner, &entry_path)?;
            table_set_num(state, output_ref, (index + 1) as f64, normalized);
        }
        Ok(())
    })
}

fn copy_color(state: &mut LuaState, input: Val, alpha: bool, path: &str) -> LuaResult<Val> {
    if !matches!(input, Val::Table(_)) {
        return Err(invalid(path, "color table"));
    }
    let channels = if alpha {
        &["r", "g", "b", "a"][..]
    } else {
        &["r", "g", "b"][..]
    };
    let mut values = Vec::with_capacity(channels.len());
    for channel in channels {
        let value = table_get(state, input, channel);
        values.push(validate_scalar(
            state,
            value,
            Number,
            &format!("{path}.{channel}"),
        )?);
    }
    let create_color = global_val(state, "CreateColor");
    call_function_state(state, create_color, &values)
}

fn build_rooted_table(
    state: &mut LuaState,
    fill: impl FnOnce(&mut LuaState, Val) -> LuaResult<()>,
) -> LuaResult<Val> {
    let output = create_table(state);
    let saved_top = state.top;
    state.push(output);
    let result = fill(state, output);
    state.top = saved_top;
    result.map(|()| output)
}
