use sim_data::specializations::specs_for_class;
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CLASS_LABELS, RACE_DATA};

pub(super) fn resolve_class(value: &str) -> Result<i32, String> {
    let index = value
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0 && n <= CLASS_LABELS.len())
        .or_else(|| {
            CLASS_LABELS
                .iter()
                .position(|name| name.eq_ignore_ascii_case(value))
                .map(|n| n + 1)
        });
    index.map(|n| n as i32).ok_or_else(|| {
        format!(
            "unknown class '{value}'; valid classes: {}",
            CLASS_LABELS.join(", ")
        )
    })
}

pub(super) fn resolve_race(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|&index| index > 0 && index <= RACE_DATA.len())
        .map(|index| index - 1)
        .or_else(|| {
            RACE_DATA.iter().position(|(name, file_name, _)| {
                name.eq_ignore_ascii_case(value) || file_name.eq_ignore_ascii_case(value)
            })
        })
        .ok_or_else(|| {
            let valid = RACE_DATA
                .iter()
                .map(|(name, _, _)| *name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("unknown race '{value}'; valid races: {valid}")
        })
}

pub(super) fn resolve_spec(class: i32, value: &str) -> Result<i32, String> {
    let specs: Vec<_> = specs_for_class(class as u32).collect();
    let index = value
        .parse::<u32>()
        .ok()
        .and_then(|n| specs.iter().find(|s| s.order_index + 1 == n))
        .or_else(|| specs.iter().find(|s| s.name.eq_ignore_ascii_case(value)));
    index.map(|s| (s.order_index + 1) as i32).ok_or_else(|| {
        let valid = specs.iter().map(|s| s.name).collect::<Vec<_>>().join(", ");
        format!(
            "unknown specialization '{value}' for class {}; valid specializations: {valid}",
            CLASS_LABELS
                .get(class.saturating_sub(1) as usize)
                .copied()
                .unwrap_or("Unknown")
        )
    })
}

pub(super) struct Selection {
    pub(super) class: Option<i32>,
    pub(super) spec: Option<String>,
    pub(super) race: Option<usize>,
}

fn profile_supports_spec(profile: ClientProfile) -> bool {
    !matches!(
        profile,
        ClientProfile::Wrath | ClientProfile::Era | ClientProfile::Anniversary
    )
}

pub(super) fn parse_selection(
    class: Option<&str>,
    spec: Option<&str>,
    race: Option<&str>,
) -> Result<Option<Selection>, String> {
    if class.is_none() && spec.is_none() && race.is_none() {
        return Ok(None);
    }
    let class = class.map(resolve_class).transpose()?;
    let race = race.map(resolve_race).transpose()?;
    if spec.is_some() && !profile_supports_spec(ACTIVE) {
        return Err(format!(
            "--spec is not available for the {:?} profile",
            ACTIVE
        ));
    }
    if let (Some(class), Some(spec)) = (class, spec) {
        resolve_spec(class, spec)?;
    }
    Ok(Some(Selection {
        class,
        spec: spec.map(str::to_owned),
        race,
    }))
}

pub(super) fn schedule_selection(env: &WowLuaEnv, selection: Selection) {
    env.set_post_startup_exec_hook(move |env| apply_selection(env, selection));
}

fn apply_selection(env: &WowLuaEnv, selection: Selection) {
    let current_class = selection
        .class
        .unwrap_or_else(|| env.state().borrow().player.class_index);
    let spec_index = selection
        .spec
        .as_deref()
        .map(|name| resolve_spec(current_class, name))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        });
    // SetSpec emits its own notification when the index changes.
    let spec_changes =
        spec_index.is_some_and(|index| index != env.state().borrow().player.active_spec_index);
    let mut code = String::new();
    if let Some(class) = selection.class {
        code.push_str(&format!("A_Admin.SetPlayerClass({class})\n"));
    }
    if let Some(race) = selection.race {
        code.push_str(&format!("A_Admin.SetPlayerRace({race})\n"));
    }
    if let Some(spec) = spec_index {
        code.push_str(&format!("A_Admin.SetSpec({spec})\n"));
    }
    if (selection.class.is_some() || selection.spec.is_some()) && !spec_changes {
        code.push_str("A_Admin.FireEvent('PLAYER_SPECIALIZATION_CHANGED', 'player')\n");
    }
    if selection.race.is_some() {
        code.push_str("A_Admin.FireEvent('UNIT_PORTRAIT_UPDATE', 'player')\n");
    }
    if let Err(error) = env.exec(&code) {
        eprintln!("failed to apply player selection: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_selection, profile_supports_spec, resolve_class, resolve_race, resolve_spec,
        schedule_selection,
    };
    use wow_ui_sim::client_profile::ClientProfile;
    use wow_ui_sim::lua_api::WowLuaEnv;

    #[test]
    fn class_names_and_indices() {
        assert_eq!(resolve_class("dRuId").unwrap(), 11);
        assert_eq!(resolve_class("Death Knight").unwrap(), 6);
        assert_eq!(resolve_class("8").unwrap(), 8);
        assert!(resolve_class("0").unwrap_err().contains("Warrior"));
        assert!(resolve_class("unknown").unwrap_err().contains("Druid"));
    }

    #[test]
    fn race_names_file_tokens_and_one_based_indices() {
        assert_eq!(resolve_race("oRc").unwrap(), 1);
        assert_eq!(resolve_race("Night Elf").unwrap(), 3);
        assert_eq!(resolve_race("nightelf").unwrap(), 3);
        assert_eq!(resolve_race("Scourge").unwrap(), 4);
        assert_eq!(resolve_race("1").unwrap(), 0);
        assert_eq!(resolve_race("4").unwrap(), 3);
        for invalid in ["", "0", "999", "NotARace"] {
            assert!(
                resolve_race(invalid)
                    .unwrap_err()
                    .contains("valid races: Human, Orc")
            );
        }
    }

    #[test]
    fn race_selection_is_validated_without_class_or_spec() {
        assert!(parse_selection(None, None, None).unwrap().is_none());
        assert!(parse_selection(None, None, Some("Orc")).unwrap().is_some());
        assert!(
            parse_selection(None, None, Some("NotARace"))
                .err()
                .unwrap()
                .contains("valid races: Human, Orc")
        );
    }

    fn selection_event_probe() -> WowLuaEnv {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            "selection_events = {}; local frame = CreateFrame('Frame'); \
             frame:RegisterEvent('PLAYER_SPECIALIZATION_CHANGED'); \
             frame:RegisterEvent('UNIT_PORTRAIT_UPDATE'); \
             frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
             frame:SetScript('OnEvent', function(_, event, unit) \
                 table.insert(selection_events, event .. ':' .. tostring(unit) .. ':' .. \
                     select(2, UnitClass('player')) .. ':' .. select(2, UnitRace('player'))) \
             end)",
        )
        .unwrap();
        env
    }

    #[test]
    fn race_only_selection_is_deferred_and_preserves_class_and_reload_state() {
        let mut env = selection_event_probe();
        let selection = parse_selection(None, None, Some("Orc")).unwrap().unwrap();
        schedule_selection(&env, selection);
        assert_eq!(
            env.eval::<String>("return UnitRace('player')").unwrap(),
            "Human"
        );
        assert_eq!(env.eval::<i32>("return #selection_events").unwrap(), 0);

        env.exec_maybe_secure("", false).unwrap();
        assert_eq!(
            env.eval::<(String, String)>(
                "local _, class = UnitClass('player'); \
                 local _, race = UnitRace('player'); return class, race",
            )
            .unwrap(),
            ("PALADIN".into(), "Orc".into())
        );
        assert_eq!(
            env.eval::<String>("return table.concat(selection_events, ',')")
                .unwrap(),
            "UNIT_PORTRAIT_UPDATE:player:PALADIN:Orc"
        );

        env.exec("ReloadUI()").unwrap();
        assert!(env.service_reload(&mut None).unwrap());
        assert_eq!(
            env.eval::<String>("return UnitRace('player')").unwrap(),
            "Orc"
        );
    }

    #[test]
    fn combined_class_and_race_selection_dispatches_each_relevant_event_once() {
        let env = selection_event_probe();
        let selection = parse_selection(Some("Mage"), None, Some("NightElf"))
            .unwrap()
            .unwrap();
        schedule_selection(&env, selection);
        env.exec_maybe_secure("", false).unwrap();
        assert_eq!(
            env.eval::<(String, String)>(
                "local _, class = UnitClass('player'); \
                 local _, race = UnitRace('player'); return class, race",
            )
            .unwrap(),
            ("MAGE".into(), "NightElf".into())
        );
        assert_eq!(
            env.eval::<String>("return table.concat(selection_events, ',')")
                .unwrap(),
            "PLAYER_SPECIALIZATION_CHANGED:player:MAGE:NightElf,UNIT_PORTRAIT_UPDATE:player:MAGE:NightElf"
        );
    }

    #[test]
    fn specialization_support_matches_client_profiles() {
        for profile in [
            ClientProfile::Retail,
            ClientProfile::Ptr,
            ClientProfile::Mists,
            ClientProfile::WowForever,
        ] {
            assert!(profile_supports_spec(profile), "{profile:?}");
        }
        for profile in [
            ClientProfile::Wrath,
            ClientProfile::Era,
            ClientProfile::Anniversary,
        ] {
            assert!(!profile_supports_spec(profile), "{profile:?}");
        }
    }

    #[test]
    fn specialization_names_and_indices_are_class_scoped() {
        assert_eq!(resolve_spec(11, "restoration").unwrap(), 4);
        assert_eq!(resolve_spec(8, "FIRE").unwrap(), 2);
        assert_eq!(resolve_spec(8, "3").unwrap(), 3);
        assert!(
            resolve_spec(8, "Restoration")
                .unwrap_err()
                .contains("Arcane")
        );
        assert!(resolve_spec(8, "4").is_err());
    }
}
