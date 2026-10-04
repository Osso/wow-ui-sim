//! Retail 12.1.0 page-listed struct field changes, checked against the cached
//! generated API documentation (`Type`/`Nilable`) through the public getters.
#![cfg(feature = "client-retail")]

use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

struct DocField {
    name: String,
    doc_type: String,
    nilable: bool,
}

struct ShapeCase {
    source_ids: &'static [&'static str],
    doc_file: &'static str,
    struct_name: &'static str,
    seed: fn(&WowLuaEnv),
    /// Lua function body returning the struct table under test.
    getter: &'static str,
    /// Page-listed fields that must exist in the cached doc and match its type.
    added: &'static [&'static str],
    /// Page-listed fields that must be absent from both the doc and the value.
    removed: &'static [&'static str],
    /// Lua function body over `info` returning "ok" or a failure label.
    values: &'static str,
}

fn no_seed(_: &WowLuaEnv) {}

fn doc_path(doc_file: &str) -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
    .join("Blizzard_APIDocumentationGenerated")
    .join(doc_file)
}

fn quoted_attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(&format!("{key} = \""))? + key.len() + 4;
    let len = line[start..].find('"')?;
    Some(&line[start..start + len])
}

/// Field rows of one `Type = "Structure"` table in a generated doc file.
fn doc_struct_fields(doc_file: &str, struct_name: &str) -> Vec<DocField> {
    let source = std::fs::read_to_string(doc_path(doc_file))
        .unwrap_or_else(|e| panic!("read cached doc {doc_file}: {e}"));
    let header = format!("Name = \"{struct_name}\",");
    let mut lines = source.lines().skip_while(|line| line.trim() != header);
    assert!(
        lines.next().is_some(),
        "{struct_name} missing from {doc_file}"
    );
    assert_eq!(
        lines.next().map(str::trim),
        Some("Type = \"Structure\","),
        "{struct_name} in {doc_file} is not a Structure"
    );
    lines
        .skip_while(|line| !line.trim().starts_with("{ Name = "))
        .take_while(|line| line.trim().starts_with("{ Name = "))
        .map(|line| DocField {
            name: quoted_attr(line, "Name").expect("field name").to_string(),
            doc_type: quoted_attr(line, "Type").expect("field type").to_string(),
            nilable: line.contains("Nilable = true"),
        })
        .collect()
}

fn lua_type_for(doc_type: &str, is_enum: bool) -> &'static str {
    match doc_type {
        "bool" => "boolean",
        "number" | "uiUnit" | "luaIndex" | "time_t" | "fileID" => "number",
        "string" | "cstring" | "WOWGUID" | "textureKit" | "kstringAuroraName" => "string",
        _ if is_enum => "number",
        // Structures, tables and frame handles (FrameRef reports "table").
        _ => "table",
    }
}

fn observe(env: &WowLuaEnv, case: &ShapeCase, fields: &[&DocField]) -> Result<Vec<String>, String> {
    let names: Vec<String> = fields
        .iter()
        .map(|f| format!("{{ {:?}, {:?} }}", f.name, f.doc_type))
        .chain(
            case.removed
                .iter()
                .map(|name| format!("{{ {name:?}, \"\" }}")),
        )
        .collect();
    let code = format!(
        r#"
        local info = (function() {getter} end)()
        if type(info) ~= "table" then return "getter returned " .. type(info), "" end
        local out = {{}}
        for _, field in ipairs({{ {names} }}) do
            local isEnum = field[2] ~= "" and type(Enum[field[2]]) == "table"
            out[#out + 1] = field[1] .. "=" .. type(info[field[1]]) .. "=" .. tostring(isEnum)
        end
        return table.concat(out, ";"), (function(info) {values} end)(info)
        "#,
        getter = case.getter,
        names = names.join(", "),
        values = case.values,
    );
    let (observed, values): (String, String) = env.eval(&code).map_err(|e| e.to_string())?;
    let mut failures = Vec::new();
    if values != "ok" {
        failures.push(format!("values: {values}"));
    }
    for (index, entry) in observed.split(';').enumerate() {
        let mut parts = entry.split('=');
        let (name, lua_type, is_enum) = (parts.next(), parts.next(), parts.next());
        let (Some(name), Some(lua_type)) = (name, lua_type) else {
            return Err(observed);
        };
        match fields.get(index) {
            Some(field) => {
                let expected = lua_type_for(&field.doc_type, is_enum == Some("true"));
                if lua_type != expected && !(field.nilable && lua_type == "nil") {
                    failures.push(format!(
                        "{name}: {lua_type}, doc {} Nilable={} expects {expected}",
                        field.doc_type, field.nilable
                    ));
                }
            }
            None if lua_type != "nil" => {
                failures.push(format!("removed {name} present as {lua_type}"))
            }
            None => {}
        }
    }
    Ok(failures)
}

fn check_case(case: &ShapeCase) -> Vec<String> {
    let doc = doc_struct_fields(case.doc_file, case.struct_name);
    let mut failures = Vec::new();
    let mut fields = Vec::new();
    for name in case.added {
        match doc.iter().find(|f| f.name == *name) {
            Some(field) => fields.push(field),
            None => failures.push(format!("doc lacks added field {name}")),
        }
    }
    for name in case.removed {
        if doc.iter().any(|f| f.name == *name) {
            failures.push(format!("doc still lists removed field {name}"));
        }
    }
    let env = WowLuaEnv::new().expect("create struct shape environment");
    (case.seed)(&env);
    match observe(&env, case, &fields) {
        Ok(found) => failures.extend(found),
        Err(error) => failures.push(error),
    }
    failures
}

const CASES: &[ShapeCase] = &[
    ShapeCase {
        source_ids: &[
            "structures-AddPrivateAuraAnchorArgs-329",
            "structures-AddPrivateAuraAnchorArgs-330",
            "structures-AddPrivateAuraAnchorArgs-331",
            "structures-AddPrivateAuraAnchorArgs-332",
            "structures-AddPrivateAuraAnchorArgs-333",
        ],
        doc_file: "UnitConstantsDocumentation.lua",
        struct_name: "AddPrivateAuraAnchorArgs",
        seed: no_seed,
        // Args are input: omitted Default=false flags and the removed
        // showCountdownFrame must round-trip through the published record.
        getter: r#"
            local parent = CreateFrame("Frame")
            C_UnitAuras.AddPrivateAuraAnchor({
                unitToken = "player", auraIndex = 1, parent = parent, showCountdownFrame = true,
            })
            local anchors = C_UnitAurasPrivate.GetPrivateAuraAnchors("player")
            return anchors[#anchors]
        "#,
        added: &["showDispelIcon", "showCooldownEdge", "showCooldownFrame"],
        removed: &["showCountdownFrame"],
        values: r#"
            if info.showDispelIcon or info.showCooldownEdge or info.showCooldownFrame then
                return "defaults"
            end
            C_UnitAuras.AddPrivateAuraAnchor({
                unitToken = "target", auraIndex = 2, parent = CreateFrame("Frame"),
                showDispelIcon = true, showCooldownEdge = true, showCooldownFrame = true,
            })
            local set = C_UnitAurasPrivate.GetPrivateAuraAnchors("target")[1]
            if not (set.showDispelIcon and set.showCooldownEdge and set.showCooldownFrame) then
                return "explicit"
            end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-UnitPrivateAuraAnchorInfo-370",
            "structures-UnitPrivateAuraAnchorInfo-371",
            "structures-UnitPrivateAuraAnchorInfo-372",
            "structures-UnitPrivateAuraAnchorInfo-373",
            "structures-UnitPrivateAuraAnchorInfo-374",
        ],
        doc_file: "UnitConstantsDocumentation.lua",
        struct_name: "UnitPrivateAuraAnchorInfo",
        seed: no_seed,
        getter: r#"
            C_UnitAuras.AddPrivateAuraAnchor({
                unitToken = "player", auraIndex = 3, parent = CreateFrame("Frame"),
                showCooldownFrame = true, showCooldownEdge = false, showDispelIcon = true,
            })
            return C_UnitAurasPrivate.GetPrivateAuraAnchors("player")[1]
        "#,
        added: &["showDispelIcon", "showCooldownEdge", "showCooldownFrame"],
        removed: &["showCountdownFrame"],
        values: r#"
            if info.showCooldownFrame ~= true or info.showCooldownEdge ~= false then return "cooldown" end
            if info.showDispelIcon ~= true then return "dispel" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-LfgSearchResultData-351",
            "structures-LfgSearchResultData-352",
        ],
        doc_file: "LFGListInfoDocumentation.lua",
        struct_name: "LfgSearchResultData",
        seed: no_seed,
        getter: "return C_LFGList.GetSearchResultInfo(7)",
        added: &["censored"],
        removed: &[],
        values: r#"return info.censored == false and "ok" or "censored""#,
    },
    ShapeCase {
        source_ids: &[
            "structures-ClubMemberInfo-341",
            "structures-ClubMemberInfo-342",
        ],
        doc_file: "ClubDocumentation.lua",
        struct_name: "ClubMemberInfo",
        seed: no_seed,
        // GetInfoFromLastCommunityChatLine has no producer; GetMemberInfo
        // returns the same ClubMemberInfo. Discord membership is unmodeled,
        // so discordInfo is the Nilable=true nil.
        getter: r#"return C_Club.GetMemberInfo("guild-0", 1)"#,
        added: &["discordInfo"],
        removed: &[],
        values: r#"return info.discordInfo == nil and "ok" or "discord""#,
    },
];

#[test]
fn patch_12_1_0_struct_shapes_match_cached_docs() {
    let failures: Vec<String> = CASES
        .iter()
        .flat_map(|case| {
            check_case(case)
                .into_iter()
                .map(move |f| format!("{} {:?}: {f}", case.struct_name, case.source_ids))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "struct shape failures:\n  {}",
        failures.join("\n  ")
    );
}
