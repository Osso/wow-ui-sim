#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
//! `Blizzard_CombatLogProcessor` runs under secureenv (`## UseSecureEnvironment: 1`)
//! and reads `CombatLogUtil`, `COMBATLOG_FILTER_MINE` and
//! `COMBATLOG_DEFAULT_COLORS` from its declared dependency
//! `Blizzard_CombatLogBase`, which loads into `_G` after secureenv was
//! snapshotted. The loader replays that library into secureenv
//! (`is_secure_replay_library_addon` in `src/loader/addon.rs`); this pins
//! the outcome from a chunk that actually runs in the secure environment.
use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_closure;

#[test]
fn combat_log_processor_sees_combat_log_base_globals_from_secure_code() {
    common::with_timeout(240, || {
        with_blizzard_addon_closure(&["Blizzard_CombatLogProcessor"], &[], |env, loaded| {
            for required in ["Blizzard_CombatLogBase", "Blizzard_CombatLogProcessor"] {
                assert!(
                    loaded.iter().any(|name| name == required),
                    "{required} should be in the loaded closure: {loaded:?}"
                );
            }

            env.exec_rilua_secure(
                r#"
                SecureProbe_CombatLogUtilType = type(CombatLogUtil)
                SecureProbe_CombatLogFilterMineType = type(COMBATLOG_FILTER_MINE)
                SecureProbe_CombatLogDefaultColorsType = type(COMBATLOG_DEFAULT_COLORS)
                "#,
            )
            .expect("secure probe chunk should run");

            let (util_type, filter_type, colors_type, probe_in_global): (
                String,
                String,
                String,
                String,
            ) = env
                .eval(
                    r#"
                    return tostring(rawget(__secureenv, "SecureProbe_CombatLogUtilType")),
                           tostring(rawget(__secureenv, "SecureProbe_CombatLogFilterMineType")),
                           tostring(rawget(__secureenv, "SecureProbe_CombatLogDefaultColorsType")),
                           type(rawget(_G, "SecureProbe_CombatLogUtilType"))
                    "#,
                )
                .expect("secureenv probe readback should return");

            assert_eq!(
                probe_in_global, "nil",
                "probe writes must land in secureenv, not _G (the chunk did not run secure)"
            );
            assert_eq!(
                util_type, "table",
                "CombatLogUtil from Blizzard_CombatLogBase must be visible to secure code"
            );
            assert_eq!(
                filter_type, "number",
                "COMBATLOG_FILTER_MINE from Blizzard_CombatLogBase must be visible to secure code"
            );
            assert_eq!(
                colors_type, "table",
                "COMBATLOG_DEFAULT_COLORS from Blizzard_CombatLogBase must be visible to secure code"
            );
        });
    });
}
