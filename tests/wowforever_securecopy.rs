//! Compatibility copying used by Forever aura option exports.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn securecopy_preserves_graph_structure_and_userdata_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 10)
        curve:AddPoint(1, 20)
        local key = {name = 'option'}
        local shared = {enabled = true}
        local options = {nested = shared, alias = shared, curve = curve}
        options.self = options
        options[key] = key
        setmetatable(options, {marker = true})
        local copied = securecopy(options)
        assert(copied ~= options and copied.nested ~= shared)
        assert(copied.self == copied and copied.alias == copied.nested)
        assert(copied.curve == curve and copied.curve:Evaluate(0.5) == 15)
        assert(getmetatable(copied) == nil)
        copied.nested.enabled = false
        assert(shared.enabled)
        local copiedKey
        for k, v in pairs(copied) do
            if type(k) == 'table' then copiedKey = k; assert(k == v) end
        end
        assert(copiedKey ~= key and copiedKey.name == 'option')
        assert(securecopy(false) == false and securecopy(nil) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn aura_defaults_are_exported_as_independent_tables() {
    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for dependency in [
        "Blizzard_SharedXML/TimeUtil.lua",
        "Blizzard_SharedXMLBase/AnchorUtil.lua",
    ] {
        let source = std::fs::read_to_string(addons.join(dependency))
            .expect("synced Forever aura dependency");
        env.exec(&source).unwrap();
    }
    let path = addons.join("Blizzard_AuraContainer/Blizzard_AuraContainerShared.lua");
    let source = std::fs::read_to_string(&path).expect("synced Forever aura source");
    // Execute the actual source in an isolated environment, as the secure addon
    // does, while retaining the global environment used for its exports.
    env.exec(
        r#"
        auraSourceEnvironment = setmetatable({}, {__index = _G})
        function executeAuraSource(source)
            local chunk = assert(loadstring(source))
            setfenv(chunk, auraSourceEnvironment)
            chunk('Blizzard_AuraContainer', {})
        end
        "#,
    )
    .unwrap();
    env.exec(&format!("executeAuraSource({source:?})")).unwrap();
    env.exec(
        r#"
        local original = auraSourceEnvironment.CustomAuraContainerLayoutDefaults
        local exported = CustomAuraContainerLayoutDefaults
        assert(type(original) == 'table' and type(exported) == 'table')
        assert(original ~= exported)
        exported.securecopyTest = {enabled = true}
        assert(original.securecopyTest == nil)
        assert(AuraContainerAuraDataType ~= auraSourceEnvironment.AuraContainerAuraDataType)
        "#,
    )
    .unwrap();
}
