//! Temporary LFG lock-list startup seeding.
//!
//! Retail initializes `LFGLockList` through `LFG_LOCK_INFO_RECEIVED`. Firing
//! that event during simulator startup also reaches unmodeled
//! RaidFinder/ScenarioFinder availability checks, so this seeds only the table
//! that `UpdateLFDDungeonList` reads before `LFGDungeonList_Setup` can lazily
//! initialize it.

use crate::lua_api::WowLuaEnv;

const LFG_LOCK_LIST_LUA: &str = r#"
if type(GetLFGLockList) == "function" and LFGLockList == nil then
    LFGLockList = GetLFGLockList()
end
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    // Retail's GroupFinder setup reads the supplied lock list before its first
    // queue refresh; Forever does not select GroupFinder.
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return;
    }
    let _ = env.exec(LFG_LOCK_LIST_LUA);
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use super::*;

    #[test]
    fn seeds_missing_lfg_lock_list_from_api() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            GetLFGLockList = function()
                return { [42] = { locked = true } }
            end
            "#,
        )
        .expect("lfg fixture should install");

        patch(&env);

        let locked: bool = env
            .eval("return LFGLockList[42].locked == true")
            .expect("seeded lfg lock list should be readable");

        assert!(locked);
    }

    #[test]
    fn preserves_existing_lfg_lock_list() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            LFGLockList = { existing = true }
            GetLFGLockList = function()
                return { existing = false }
            end
            "#,
        )
        .expect("lfg fixture should install");

        patch(&env);

        let existing: bool = env
            .eval("return LFGLockList.existing == true")
            .expect("preserved lfg lock list should be readable");

        assert!(existing);
    }
}
