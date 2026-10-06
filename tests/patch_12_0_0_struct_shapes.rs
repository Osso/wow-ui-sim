//! Full cached DTO contracts for the 12.0.0 extract, not native-domain parity.
use wow_ui_sim::lua_api::WowLuaEnv;

/// Same cached-field/type pattern as patch_12_1_0_struct_shapes, applied to
/// every parent field rather than only a patch's changed fields.
pub(crate) fn assert_shape(env: &WowLuaEnv, file: &str, name: &str, getter: &str) {
    let path = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ))
    .join("Blizzard_APIDocumentationGenerated")
    .join(file);
    let source = std::fs::read_to_string(path).expect("read cached API declaration");
    let header = format!("Name = \"{name}\",");
    let mut lines = source.lines().skip_while(|line| line.trim() != header);
    assert!(lines.next().is_some(), "missing structure {name}");
    assert_eq!(lines.next().map(str::trim), Some("Type = \"Structure\","));
    let fields: Vec<_> = lines
        .skip_while(|line| !line.trim().starts_with("{ Name = "))
        .take_while(|line| line.trim().starts_with("{ Name = "))
        .collect();
    assert!(!fields.is_empty(), "empty structure {name}");
    let script = format!(
        r#"
        local info = (function() {getter} end)()
        assert(type(info) == 'table', '{name}: populated DTO required')
        for _, field in ipairs({{ {fields} }}) do
            local kind = field.Type
            local expected = 'table'
            if kind == 'bool' then expected = 'boolean'
            elseif kind == 'number' or kind == 'uiUnit' or kind == 'luaIndex'
                or kind == 'time_t' or kind == 'fileID' or type(Enum[kind]) == 'table' then expected = 'number'
            elseif kind == 'string' or kind == 'cstring' or kind == 'WOWGUID'
                or kind == 'textureKit' or kind == 'textureAtlas' or string.sub(kind, 1, 7) == 'kstring' then expected = 'string' end
            local value = info[field.Name]
            assert(type(value) == expected or (field.Nilable and value == nil),
                '{name}.' .. field.Name .. ': expected ' .. expected .. ', got ' .. type(value))
            if field.InnerType == 'number' and value then
                for _, element in ipairs(value) do assert(type(element) == 'number', field.Name) end
            end
        end
        "#,
        fields = fields.join("\n"),
    );
    env.exec(&script).expect("cached parent contract");
}

#[test]
fn advanced_filter_parent_and_playstyle_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    assert_shape(
        &env,
        "LFGListInfoDocumentation.lua",
        "AdvancedFilterOptions",
        "return C_LFGList.GetAdvancedFilter()",
    );
    env.exec(
        r#"
        local before = C_LFGList.GetAdvancedFilter()
        for index = 1, 4 do
            local selected = C_LFGList.GetAdvancedFilter()
            for flag = 1, 4 do selected['generalPlaystyle' .. flag] = flag == index end
            selected.activities = {1195, 1240}
            C_LFGList.SaveAdvancedFilter(selected)
            selected.activities[1] = -1
            selected['generalPlaystyle' .. index] = false
            local actual = C_LFGList.GetAdvancedFilter()
            for flag = 1, 4 do assert(actual['generalPlaystyle' .. flag] == (flag == index)) end
            assert(actual.activities[1] == 1195 and actual.activities[2] == 1240)
        end
        for flag = 1, 4 do assert(before['generalPlaystyle' .. flag] == false) end
        assert(C_LFGList.GetSearchResultInfo(1).name == '+15 Mists chill run')
    "#,
    )
    .unwrap();
    assert_shape(
        &env,
        "LFGListInfoDocumentation.lua",
        "AdvancedFilterOptions",
        "return C_LFGList.GetAdvancedFilter()",
    );
}

#[test]
fn appearance_parent_and_old_casing_absence() {
    use wow_ui_sim::lua_api::state::AppearanceSourceInfo;
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_appearance_sources.insert(
        901,
        AppearanceSourceInfo {
            category: 4,
            item_appearance_id: 902,
            can_have_illusion: true,
            icon: 903,
            is_collected: false,
            item_link: "item:fixture".into(),
            transmoglink: "transmog:fixture".into(),
            source_type: Some(3),
            item_subclass: 7,
            ignore_model_attachment_checks_for_illusion: true,
        },
    );
    assert_shape(
        &env,
        "TransmogItemsDocumentation.lua",
        "TransmogAppearanceSourceInfoData",
        "return C_TransmogCollection.GetAppearanceSourceInfo(901)",
    );
    env.exec(
        r#"
        local info = C_TransmogCollection.GetAppearanceSourceInfo(901)
        assert(info.itemSubclass == 7)
        assert(rawget(info, 'itemSubClass') == nil and info.itemSubClass == nil)
    "#,
    )
    .unwrap();
}
