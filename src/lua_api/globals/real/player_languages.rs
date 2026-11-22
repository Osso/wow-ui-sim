//! INFERRED R88eg (native evidence pending): player chat languages follow race and faction.

use crate::lua_api::game_data::RACE_DATA;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[derive(Clone, Copy, PartialEq)]
struct Language {
    name: &'static str,
    id: i32,
}

const COMMON: Language = Language {
    name: "Common",
    id: 7,
};
const ORCISH: Language = Language {
    name: "Orcish",
    id: 1,
};
const RACIAL_LANGUAGES: &[Option<Language>] = &[
    None,
    None,
    Some(Language {
        name: "Dwarvish",
        id: 6,
    }),
    Some(Language {
        name: "Darnassian",
        id: 2,
    }),
    Some(Language {
        name: "Gutterspeak",
        id: 33,
    }),
    Some(Language {
        name: "Taurahe",
        id: 3,
    }),
    Some(Language {
        name: "Gnomish",
        id: 13,
    }),
    Some(Language {
        name: "Troll",
        id: 14,
    }),
    Some(Language {
        name: "Thalassian",
        id: 10,
    }),
    Some(Language {
        name: "Draenei",
        id: 35,
    }),
    Some(Language {
        name: "Worgen",
        id: 37,
    }),
    Some(Language {
        name: "Goblin",
        id: 40,
    }),
    Some(Language {
        name: "Pandaren",
        id: 42,
    }),
    Some(Language {
        name: "Draconic",
        id: 11,
    }),
    Some(Language {
        name: "Earthen",
        id: 84,
    }),
];

fn player_languages(state: &mut LuaState) -> LuaResult<Option<(Language, Option<Language>)>> {
    let sim = borrow_state(state)?;
    let Some((_, _, faction)) = RACE_DATA.get(sim.player.race_index) else {
        return Ok(None);
    };
    let racial = RACIAL_LANGUAGES
        .get(sim.player.race_index)
        .copied()
        .flatten();
    let default = match *faction {
        "Alliance" => COMMON,
        "Horde" => ORCISH,
        "Neutral" => match racial {
            Some(language) => language,
            None => return Ok(None),
        },
        _ => return Ok(None),
    };
    let alternative = racial.filter(|language| *language != default);
    Ok(Some((default, alternative)))
}

fn push_language(state: &mut LuaState, language: Language) {
    let name = create_string(state, language.name);
    state.push(name);
    state.push(Val::Num(language.id as f64));
}

fn get_default_language(state: &mut LuaState) -> LuaResult<u32> {
    if let Some((language, _)) = player_languages(state)? {
        push_language(state, language);
        return Ok(2);
    }
    Ok(0)
}

fn get_alternative_default_language(state: &mut LuaState) -> LuaResult<u32> {
    if let Some((_, Some(language))) = player_languages(state)? {
        push_language(state, language);
        return Ok(2);
    }
    Ok(0)
}

fn get_num_languages(state: &mut LuaState) -> LuaResult<u32> {
    let count = player_languages(state)?
        .map_or(0, |(_, alternative)| 1 + usize::from(alternative.is_some()));
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_language_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let language = match (player_languages(state)?, index) {
        (Some((language, _)), 1) => Some(language),
        (Some((_, Some(language))), 2) => Some(language),
        _ => None,
    };
    if let Some(language) = language {
        push_language(state, language);
        return Ok(2);
    }
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(2)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    LuaApiMut::register_function(lua, "GetDefaultLanguage", get_default_language)?;
    LuaApiMut::register_function(
        lua,
        "GetAlternativeDefaultLanguage",
        get_alternative_default_language,
    )?;
    LuaApiMut::register_function(lua, "GetNumLanguages", get_num_languages)?;
    LuaApiMut::register_function(lua, "GetLanguageByIndex", get_language_by_index)?;
    Ok(())
}
