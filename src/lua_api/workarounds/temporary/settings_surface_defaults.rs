//! Temporary Settings surface defaults for partial addon loads.
//!
//! Full UI loads get the real Settings system from Blizzard_Settings. These
//! defaults keep isolated addon loads and legacy InterfaceOptions callers
//! functional without leaving the surface in the generic runtime bootstrap.

const SETTINGS_SURFACE_DEFAULTS_LUA: &str = r#"
local function settings_surface_noop()
end

local function settings_surface_namespace(fields)
    local table = fields or {}
    table.__is_wow_namespace = true
    return table
end

local function settings_make_initializer_placeholder()
    local initializer = {
        data = {},
    }

    function initializer:SetSearchIgnoredInLayout(layout)
        self.searchIgnoredInLayout = layout
    end

    function initializer:SetParentInitializer(parentInitializer, modifyPredicate)
        self.parentInitializer = parentInitializer
        self.modifyPredicate = modifyPredicate
    end

    function initializer:SetKioskProtected()
        self.kioskProtected = true
    end

    function initializer:GetName()
        return self.name or ""
    end

    return initializer
end

local function ensure_ping_sounds_initializer(settings)
    if type(settings) == "table" and rawget(settings, "PingSoundsInitializer") == nil then
        rawset(settings, "PingSoundsInitializer", settings_make_initializer_placeholder())
    end
end

local function patch_settings_registrar_assignment(registrar)
    if type(registrar) ~= "table" or rawget(registrar, "__wow_ping_sounds_registrar_guard") then
        return registrar
    end

    rawset(registrar, "__wow_ping_sounds_registrar_guard", true)
    local registrar_mt = getmetatable(registrar) or {}
    local registrar_prev_newindex = registrar_mt.__newindex

    registrar_mt.__newindex = function(tbl, subkey, subvalue)
        if subkey == "AddRegistrant" and type(subvalue) == "function" then
            local original = subvalue
            subvalue = function(self, registrant)
                ensure_ping_sounds_initializer(rawget(_G, "Settings"))
                return original(self, registrant)
            end
        end
        if registrar_prev_newindex ~= nil then
            if type(registrar_prev_newindex) == "function" then
                registrar_prev_newindex(tbl, subkey, subvalue)
                return
            end
            registrar_prev_newindex[subkey] = subvalue
            return
        end
        rawset(tbl, subkey, subvalue)
    end
    setmetatable(registrar, registrar_mt)
    return registrar
end

function __wow_prepare_temporary_global_assignment(key, value)
    if key == "Settings" then
        ensure_ping_sounds_initializer(value)
    elseif key == "SettingsRegistrar" then
        value = patch_settings_registrar_assignment(value)
    end
    return value
end

Settings = Settings or settings_surface_namespace({
    SetOnValueChangedCallback = function()
        return {
            Disconnect = settings_surface_noop,
        }
    end,
    GetOrCreateSettingsGroup = function()
        return settings_surface_namespace({
            AddInitializer = settings_surface_noop,
            AddSetting = settings_surface_noop,
            AddCategory = settings_surface_noop,
            SetValue = settings_surface_noop,
            GetValue = function() return nil end,
        })
    end,
})
__wow_prepare_temporary_global_assignment("Settings", Settings)

local existingRegistrar = rawget(_G, "SettingsRegistrar")
if type(existingRegistrar) == "table" then
    __wow_prepare_temporary_global_assignment("SettingsRegistrar", existingRegistrar)
end

EditModeAccountSettingsMixin = EditModeAccountSettingsMixin or {}

local function ensure_settings_frame(parent, key, frameType)
    local child = rawget(parent, key)
    if child ~= nil then
        return child
    end
    if type(CreateFrame) == "function" then
        local ok, frame = pcall(CreateFrame, frameType or "Frame", nil, parent)
        if ok then
            child = frame
        end
    end
    if child == nil then
        child = {}
    end
    rawset(parent, key, child)
    return child
end

local function ensure_settings_preview(settingsPanel, key)
    local preview = ensure_settings_frame(settingsPanel, key, "Frame")
    if rawget(preview, "RegisterWithSettingInitializer") == nil then
        function preview:RegisterWithSettingInitializer(_initializer) end
    end
    if rawget(preview, "SetValueAccessor") == nil then
        function preview:SetValueAccessor(_getValue) end
    end
    if rawget(preview, "UpdatePreview") == nil then
        function preview:UpdatePreview(_value) end
    end
    return preview
end

do
    local settingsPanel = rawget(_G, "SettingsPanel")
    if settingsPanel == nil and type(CreateFrame) == "function" then
        settingsPanel = CreateFrame("Frame", "SettingsPanel", UIParent)
    end
    if rawget(Settings, "__wow_settings_surface_panel") == settingsPanel then
        return
    end
    if type(settingsPanel) == "table" then
        local container = ensure_settings_frame(settingsPanel, "Container", "Frame")
        local settingsList = ensure_settings_frame(container, "SettingsList", "Frame")
        local scrollBox = ensure_settings_frame(settingsList, "ScrollBox", "Frame")
        ensure_settings_frame(scrollBox, "ScrollTarget", "Frame")
        local header = ensure_settings_frame(settingsList, "Header", "Frame")
        if rawget(header, "Title") == nil then
            local title = nil
            if type(header.CreateFontString) == "function" then
                title = header:CreateFontString(nil, "OVERLAY")
                if type(title.SetText) == "function" then
                    title:SetText("")
                end
            else
                title = { text = "", GetText = function(self) return self.text end }
            end
            rawset(header, "Title", title)
        end
        ensure_settings_preview(settingsPanel, "AccessibilityFontPreview")
        ensure_settings_preview(settingsPanel, "QuestTextPreview")
    end

    local categories = rawget(Settings, "_categories")
    if type(categories) ~= "table" then
        categories = {}
        rawset(Settings, "_categories", categories)
    end

    local function ensure_category(id, name)
        local category = categories[id]
        if type(category) ~= "table" then
            category = {
                id = id,
                name = name,
                GetID = function(self) return self.id end,
                GetName = function(self) return self.name end,
            }
            categories[id] = category
        end
        return category
    end

    local interfaceCategory = ensure_category(1, "Interface")
    local audioCategory = ensure_category(2, "Audio")
    if rawget(Settings, "INTERFACE_CATEGORY_ID") == nil then
        rawset(Settings, "INTERFACE_CATEGORY_ID", interfaceCategory:GetID())
    end
    if rawget(Settings, "AUDIO_CATEGORY_ID") == nil then
        rawset(Settings, "AUDIO_CATEGORY_ID", audioCategory:GetID())
    end

    local function get_fallback_category(id)
        id = tonumber(id)
        if categories[id] == nil then
            if id == rawget(Settings, "INTERFACE_CATEGORY_ID") then
                return ensure_category(id, "Interface")
            end
            if id == rawget(Settings, "AUDIO_CATEGORY_ID") then
                return ensure_category(id, "Audio")
            end
        end
        return categories[id]
    end

    if rawget(Settings, "GetCategory") == nil then
        Settings.GetCategory = get_fallback_category
    end

    if type(settingsPanel) == "table" then
        settingsPanel._layouts = settingsPanel._layouts or {}

        local function ensure_layout(category)
            local categoryID = category:GetID()
            local layout = settingsPanel._layouts[categoryID]
            if type(layout) ~= "table" then
                layout = {
                    _initializers = {},
                    GetInitializers = function(self) return self._initializers end,
                }
                settingsPanel._layouts[categoryID] = layout
            end
            return layout
        end

        if rawget(settingsPanel, "GetLayout") == nil then
            function settingsPanel:GetLayout(category)
                if type(category) ~= "table" or type(category.GetID) ~= "function" then
                    return nil
                end
                return self._layouts and self._layouts[category:GetID()] or nil
            end
        end

        if rawget(settingsPanel, "GetCurrentCategory") == nil then
            function settingsPanel:GetCurrentCategory()
                return rawget(self, "_currentCategory")
            end
        end

        local nextDynamicCategoryID = 1000

        local function set_category_frame_shown(category, shown)
            local frame = category and category.frame or nil
            if type(frame) ~= "table" then
                return
            end
            if type(frame.SetShown) == "function" then
                pcall(frame.SetShown, frame, shown)
                return
            end
            if shown and type(frame.Show) == "function" then
                pcall(frame.Show, frame)
            elseif not shown and type(frame.Hide) == "function" then
                pcall(frame.Hide, frame)
            end
        end

        local function hide_inactive_category_frames(activeCategory)
            for _, registeredCategory in pairs(categories) do
                if registeredCategory ~= activeCategory then
                    set_category_frame_shown(registeredCategory, false)
                end
            end
        end

        if rawget(Settings, "RegisterCanvasLayoutCategory") == nil then
            function Settings.RegisterCanvasLayoutCategory(frame, name, parentCategory)
                nextDynamicCategoryID = nextDynamicCategoryID + 1
                local categoryName = name
                if categoryName == nil and type(frame) == "table" then
                    categoryName = rawget(frame, "name") or rawget(frame, "Name")
                    if categoryName == nil and type(frame.GetName) == "function" then
                        categoryName = frame:GetName()
                    end
                end
                local category = ensure_category(nextDynamicCategoryID, categoryName or "AddOn")
                category.frame = frame
                category.parentCategory = parentCategory
                settingsPanel._layouts[category:GetID()] = {
                    frame = frame,
                    GetFrame = function(self) return self.frame end,
                }
                set_category_frame_shown(category, false)
                return category, settingsPanel._layouts[category:GetID()]
            end
        end

        if rawget(Settings, "RegisterCanvasLayoutSubcategory") == nil then
            function Settings.RegisterCanvasLayoutSubcategory(parentCategory, frame, name)
                local category = Settings.RegisterCanvasLayoutCategory(frame, name, parentCategory)
                return category, settingsPanel._layouts[category:GetID()]
            end
        end

        if rawget(Settings, "RegisterAddOnCategory") == nil then
            function Settings.RegisterAddOnCategory(category) return category end
        end

        local audioLayout = ensure_layout(audioCategory)
        if #audioLayout:GetInitializers() == 0 then
            local setting = {
                GetVariable = function() return "Sound_OutputDriverIndex" end,
            }
            local initializer = {
                GetSetting = function() return setting end,
                GetOptions = function()
                    return function()
                        return {
                            { value = 0, label = "Silent Output Device" },
                        }
                    end
                end,
            }
            table.insert(audioLayout:GetInitializers(), initializer)
        end

        ensure_layout(interfaceCategory)

        if rawget(Settings, "OpenToCategory") == nil then
            function Settings.OpenToCategory(categoryID)
                local panel = rawget(_G, "SettingsPanel") or settingsPanel
                if type(panel.OpenToCategory) == "function" then
                    local openedSuccessfully, opened = pcall(panel.OpenToCategory, panel, categoryID)
                    if openedSuccessfully and opened then
                        return panel:GetCurrentCategory()
                    end
                end

                local category = get_fallback_category(categoryID)
                if category == nil then
                    return nil
                end
                rawset(panel, "_currentCategory", category)
                if type(panel.SetShown) == "function" then
                    pcall(panel.SetShown, panel, true)
                end
                if type(panel.Show) == "function" then
                    pcall(panel.Show, panel)
                end
                hide_inactive_category_frames(category)
                set_category_frame_shown(category, true)
                return category
            end
        end
    end

    rawset(Settings, "__wow_settings_surface_panel", settingsPanel)
end

if rawget(_G, "InterfaceOptions_AddCategory") == nil then
    function InterfaceOptions_AddCategory(frame, addonName, position)
        if Settings and type(Settings.RegisterCanvasLayoutCategory) == "function" then
            local category = Settings.RegisterCanvasLayoutCategory(frame, addonName, position)
            if type(Settings.RegisterAddOnCategory) == "function" then
                Settings.RegisterAddOnCategory(category)
            end
            return category
        end
        return frame
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SETTINGS_SURFACE_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &crate::lua_api::WowLuaEnv) {
    let _ = env.exec(SETTINGS_SURFACE_DEFAULTS_LUA);
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    fn apply_again(env: &WowLuaEnv) {
        let mut lua = env.lua.borrow_mut();
        super::apply_bootstrap(&mut lua).expect("Settings surface defaults should apply");
    }

    #[test]
    fn installs_minimal_settings_categories_and_group_factory() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, String, String, String, String, String) = env
            .eval(
                r#"
                local group = Settings.GetOrCreateSettingsGroup()
                local interface = Settings.GetCategory(Settings.INTERFACE_CATEGORY_ID)
                local audio = Settings.GetCategory(Settings.AUDIO_CATEGORY_ID)
                return type(Settings),
                       type(group.AddInitializer),
                       type(group.GetValue),
                       interface:GetName(),
                       audio:GetName(),
                       type(InterfaceOptions_AddCategory)
                "#,
            )
            .expect("Settings default probe should run");

        assert_eq!(
            result,
            (
                "table".to_string(),
                "function".to_string(),
                "function".to_string(),
                "Interface".to_string(),
                "Audio".to_string(),
                "function".to_string()
            )
        );
    }

    #[test]
    fn installs_settings_value_changed_callback_noop() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, String) = env
            .eval(
                r#"
                local handle = Settings.SetOnValueChangedCallback(
                    "PROXY_STATUS_TEXT",
                    function() error("callback should not run during registration") end
                )
                return type(Settings.SetOnValueChangedCallback), type(handle.Disconnect)
                "#,
            )
            .unwrap();

        assert_eq!(result, ("function".to_string(), "function".to_string()));
    }

    #[test]
    fn installs_canvas_category_helpers_when_settings_panel_exists() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Settings = nil
            SettingsPanel = {
                shown = false,
                Show = function(self) self.shown = true end,
            }
            "#,
        )
        .expect("fixture should install SettingsPanel");

        apply_again(&env);

        let result: (String, String, bool, bool, String) = env
            .eval(
                r#"
                local frame = {
                    shown = true,
                    Hide = function(self) self.shown = false end,
                    Show = function(self) self.shown = true end,
                }
                local category, layout = Settings.RegisterCanvasLayoutCategory(frame, "Probe")
                local opened = Settings.OpenToCategory(category:GetID())
                return category:GetName(),
                       type(layout.GetFrame),
                       frame.shown,
                       SettingsPanel.shown,
                       SettingsPanel:GetCurrentCategory():GetName()
                "#,
            )
            .expect("Settings canvas category probe should run");

        assert_eq!(
            result,
            (
                "Probe".to_string(),
                "function".to_string(),
                true,
                true,
                "Probe".to_string()
            )
        );
    }

    #[test]
    fn reconciles_replacement_settings_panel_idempotently() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Settings = {
                INTERFACE_CATEGORY_ID = 1,
                AUDIO_CATEGORY_ID = 2,
                GetCategory = function()
                    return nil
                end,
                OpenToCategory = function() end,
            }
            SettingsPanel = CreateFrame("Frame", "ReplacementSettingsPanel", UIParent)
            SettingsPanel:Hide()
            "#,
        )
        .expect("fixture should replace the Settings panel and namespace");

        env.apply_post_load_workarounds();
        env.apply_post_load_workarounds();

        let result: (bool, i32) = env
            .eval(
                r#"
                Settings.INTERFACE_CATEGORY_ID = 7
                Settings.OpenToCategory(Settings.INTERFACE_CATEGORY_ID)
                local category = SettingsPanel:GetCurrentCategory()
                return SettingsPanel:IsShown(), category and category:GetID() or 0
                "#,
            )
            .expect("replacement Settings panel probe should run");

        assert_eq!(result, (true, 7));
    }

    #[test]
    fn preserves_existing_settings_category_id_and_open_function() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Settings.INTERFACE_CATEGORY_ID = 701
            Settings.OpenToCategory = function(category_id)
                return "retail:" .. tostring(category_id)
            end
            rawset(Settings, "__wow_settings_surface_panel", nil)
            "#,
        )
        .expect("fixture should install real Settings members");

        apply_again(&env);

        let result: (i32, String) = env
            .eval(
                r#"
                return Settings.INTERFACE_CATEGORY_ID,
                       Settings.OpenToCategory(Settings.INTERFACE_CATEGORY_ID)
                "#,
            )
            .expect("Settings preservation probe should run");

        assert_eq!(result, (701, "retail:701".to_string()));
    }

    #[test]
    fn preserves_existing_settings_get_category() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            Settings.GetCategory = function(name)
                return "existing:" .. tostring(name)
            end
            "#,
        )
        .expect("fixture should install existing Settings.GetCategory");

        apply_again(&env);

        let value: String = env
            .eval(r#"return Settings.GetCategory("KrowiTest")"#)
            .expect("Settings.GetCategory preservation probe should run");

        assert_eq!(value, "existing:KrowiTest");
    }

    #[test]
    fn installs_edit_mode_account_settings_mixin_table() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let mixin_type: String = env
            .eval(r#"return type(EditModeAccountSettingsMixin)"#)
            .expect("EditMode account settings mixin probe should run");

        assert_eq!(mixin_type, "table");
    }

    #[test]
    fn global_settings_assignment_installs_ping_sounds_initializer_placeholder() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, String, bool, bool, bool) = env
            .eval(
                r#"
                rawset(_G, "Settings", nil)
                Settings = {}
                local initializer = Settings.PingSoundsInitializer
                initializer:SetSearchIgnoredInLayout("probe-layout")
                initializer:SetParentInitializer("parent-initializer", "predicate")
                initializer:SetKioskProtected()
                return type(initializer),
                       initializer:GetName(),
                       initializer.searchIgnoredInLayout == "probe-layout",
                       initializer.parentInitializer == "parent-initializer",
                       initializer.kioskProtected == true
                "#,
            )
            .expect("Settings assignment hook probe should run");

        assert_eq!(
            result,
            ("table".to_string(), String::new(), true, true, true)
        );
    }

    #[test]
    fn settings_registrar_assignment_restores_missing_ping_sounds_initializer() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, String, String) = env
            .eval(
                r#"
                rawset(_G, "Settings", {})
                rawset(_G, "SettingsRegistrar", nil)
                SettingsRegistrar = {}
                SettingsRegistrar.AddRegistrant = function(_self, registrant)
                    return type(Settings.PingSoundsInitializer),
                           Settings.PingSoundsInitializer:GetName(),
                           registrant
                end
                rawset(Settings, "PingSoundsInitializer", nil)
                return SettingsRegistrar:AddRegistrant("probe-registrant")
                "#,
            )
            .expect("SettingsRegistrar assignment hook probe should run");

        assert_eq!(
            result,
            (
                "table".to_string(),
                String::new(),
                "probe-registrant".to_string()
            )
        );
    }
}
