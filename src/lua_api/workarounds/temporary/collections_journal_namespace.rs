//! Temporary Collections journal namespace defaults.
//!
//! Collections startup paths expect a few mount/pet journal helpers before the
//! full journal state exists. Keep those placeholder defaults explicit.

use crate::lua_api::LoaderEnv;

const COLLECTIONS_FILTER_DEFAULTS_LUA: &str = r#"
local defaultFilterNamespaces = {
    "C_ToyBoxInfo",
    "C_HeirloomInfo",
    "C_TransmogCollection",
}

for _, namespaceName in ipairs(defaultFilterNamespaces) do
    local namespace = rawget(_G, namespaceName)
    if namespace == nil then
        namespace = {}
        rawset(_G, namespaceName, namespace)
    end
    if rawget(namespace, "IsUsingDefaultFilters") == nil then
        function namespace.IsUsingDefaultFilters()
            return true
        end
    end
end
"#;

const COLLECTIONS_JOURNAL_NAMESPACE_WORKAROUND_LUA: &str = r#"
if type(C_MountJournal) == "table" then
    if rawget(C_MountJournal, "GetDisplayedMountID") == nil then
        function C_MountJournal.GetDisplayedMountID(_index)
            return nil
        end
    end
    if rawget(C_MountJournal, "ApplyMountEquipment") == nil then
        function C_MountJournal.ApplyMountEquipment(_mountID)
        end
    end
    if rawget(C_MountJournal, "Pickup") == nil then
        function C_MountJournal.Pickup(_mountID)
        end
    end
end

if type(C_PetJournal) == "table" then
    if rawget(C_PetJournal, "IsUsingDefaultFilters") == nil then
        function C_PetJournal.IsUsingDefaultFilters()
            return true
        end
    end
    if rawget(C_PetJournal, "PetCanBeReleased") == nil then
        function C_PetJournal.PetCanBeReleased(_petID)
            return false
        end
    end
end

if type(MountJournalToggleDynamicFlightFlyoutButtonMixin) == "table"
    and type(MountJournalToggleDynamicFlightFlyoutButtonMixin.UpdateUnspentGlyphsAnimation) == "function"
    and not MountJournalToggleDynamicFlightFlyoutButtonMixin.__wow_popup_guard then
    local original = MountJournalToggleDynamicFlightFlyoutButtonMixin.UpdateUnspentGlyphsAnimation
    MountJournalToggleDynamicFlightFlyoutButtonMixin.UpdateUnspentGlyphsAnimation = function(self, ...)
        if not self.popup then
            return
        end
        return original(self, ...)
    end
    MountJournalToggleDynamicFlightFlyoutButtonMixin.__wow_popup_guard = true
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(COLLECTIONS_FILTER_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &LoaderEnv<'_>) {
    let _ = env.exec(COLLECTIONS_JOURNAL_NAMESPACE_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn seeds_missing_collections_filter_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if C_ToyBoxInfo.IsUsingDefaultFilters() ~= true then return "toybox" end
                if C_HeirloomInfo.IsUsingDefaultFilters() ~= true then return "heirloom" end
                if C_TransmogCollection.IsUsingDefaultFilters() ~= true then return "transmog" end
                return "ok"
                "#,
            )
            .expect("collections filter defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_collections_filter_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_ToyBoxInfo.IsUsingDefaultFilters = function()
                return false
            end
            C_HeirloomInfo.IsUsingDefaultFilters = function()
                return false
            end
            C_TransmogCollection.IsUsingDefaultFilters = function()
                return false
            end
            "#,
        )
        .expect("fixture should install collection filter functions");

        apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                if C_ToyBoxInfo.IsUsingDefaultFilters() ~= false then return "toybox" end
                if C_HeirloomInfo.IsUsingDefaultFilters() ~= false then return "heirloom" end
                if C_TransmogCollection.IsUsingDefaultFilters() ~= false then return "transmog" end
                return "ok"
                "#,
            )
            .expect("existing collection filter functions should remain callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn seeds_missing_mount_and_pet_journal_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("C_MountJournal = {}; C_PetJournal = {}")
            .expect("journal namespaces should install");
        let loader_env = LoaderEnv::new(&env);

        patch(&loader_env);

        let (
            mount_defaults,
            displayed_id_is_nil,
            applied_mount_equipment_is_nil,
            picked_up_mount_is_nil,
            pet_defaults,
            pet_can_be_released,
        ): (bool, bool, bool, bool, bool, bool) = env
            .eval(
                r#"
                return C_MountJournal.IsUsingDefaultFilters == nil,
                    C_MountJournal.GetDisplayedMountID(1) == nil,
                    C_MountJournal.ApplyMountEquipment(1) == nil,
                    C_MountJournal.Pickup(1) == nil,
                    C_PetJournal.IsUsingDefaultFilters(),
                    C_PetJournal.PetCanBeReleased("pet")
                "#,
            )
            .expect("journal namespace defaults should be callable");

        assert!(mount_defaults);
        assert!(displayed_id_is_nil);
        assert!(applied_mount_equipment_is_nil);
        assert!(picked_up_mount_is_nil);
        assert!(pet_defaults);
        assert!(!pet_can_be_released);
    }

    #[test]
    fn preserves_existing_journal_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_MountJournal = {
                IsUsingDefaultFilters = function()
                    return false
                end,
                GetDisplayedMountID = function()
                    return 123
                end,
                ApplyMountEquipment = function()
                    return "applied"
                end,
                Pickup = function()
                    return "picked"
                end,
            }
            C_PetJournal = {
                IsUsingDefaultFilters = function()
                    return false
                end,
                PetCanBeReleased = function()
                    return true
                end,
            }
            "#,
        )
        .expect("journal namespaces should install");
        let loader_env = LoaderEnv::new(&env);

        patch(&loader_env);

        let (
            mount_defaults,
            displayed_id,
            applied_mount_equipment,
            picked_up_mount,
            pet_defaults,
            pet_can_be_released,
        ): (bool, i64, String, String, bool, bool) = env
            .eval(
                r#"
                return C_MountJournal.IsUsingDefaultFilters(),
                    C_MountJournal.GetDisplayedMountID(1),
                    C_MountJournal.ApplyMountEquipment(1),
                    C_MountJournal.Pickup(1),
                    C_PetJournal.IsUsingDefaultFilters(),
                    C_PetJournal.PetCanBeReleased("pet")
                "#,
            )
            .expect("existing journal namespace defaults should be callable");

        assert!(!mount_defaults);
        assert_eq!(displayed_id, 123);
        assert_eq!(applied_mount_equipment, "applied");
        assert_eq!(picked_up_mount, "picked");
        assert!(!pet_defaults);
        assert!(pet_can_be_released);
    }

    #[test]
    fn guards_dynamic_flight_animation_when_popup_is_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            animation_calls = 0
            MountJournalToggleDynamicFlightFlyoutButtonMixin = {
                UpdateUnspentGlyphsAnimation = function()
                    animation_calls = animation_calls + 1
                    return "animated"
                end,
            }
            "#,
        )
        .expect("dynamic flight mixin should install");
        let loader_env = LoaderEnv::new(&env);

        patch(&loader_env);

        let (missing_popup_result, with_popup_result, calls, guarded): (
            Option<String>,
            String,
            i64,
            bool,
        ) = env
            .eval(
                r#"
                local missingPopupResult =
                    MountJournalToggleDynamicFlightFlyoutButtonMixin.UpdateUnspentGlyphsAnimation({})
                local withPopupResult =
                    MountJournalToggleDynamicFlightFlyoutButtonMixin.UpdateUnspentGlyphsAnimation({ popup = {} })
                return missingPopupResult,
                    withPopupResult,
                    animation_calls,
                    MountJournalToggleDynamicFlightFlyoutButtonMixin.__wow_popup_guard == true
                "#,
            )
            .expect("dynamic flight popup guard should run");

        assert_eq!(missing_popup_result, None);
        assert_eq!(with_popup_result, "animated");
        assert_eq!(calls, 1);
        assert!(guarded);
    }
}
