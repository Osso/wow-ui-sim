//! Retail 12.1.0 page-listed struct field changes, checked against the cached
//! generated API documentation (`Type`/`Nilable`) through the public getters.
#![cfg(feature = "client-retail")]

use std::path::PathBuf;

use wow_ui_sim::c_api::c_cooldown_viewer::CooldownViewerCooldown;
use wow_ui_sim::c_api::c_sound::PlaySoundRequest;
use wow_ui_sim::event::EventArg;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::host_chat_inputs::{
    DiscordChatInfo, HostChatArgument, HostChatKind, HostChatMessage,
};
use wow_ui_sim::lua_api::state::PlayerChoiceInfo;

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

fn seed_player_choice(env: &WowLuaEnv) {
    env.state().borrow_mut().player_choice.current = Some(PlayerChoiceInfo {
        choice_id: 77,
        hide_answer_art: true,
        ..Default::default()
    });
}

fn seed_pet_attachable_decor(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.housing.pet_attachable_decor_guids = vec!["Decor-Selection-2001".into()];
}

fn seed_bnet_title_friend(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let friend = &mut state.bnet_friends[0];
    friend.friend_level = 3;
    let account = &mut friend.game_accounts[0];
    account.class_id = 6;
    account.class_name = "Death Knight".into();
}

/// Publish one CHAT_MSG_SAY from the host queue and keep its payload in
/// `ChatParams`, named by the cached ChatMessageEventParams field order.
fn seed_discord_chat_line(env: &WowLuaEnv) {
    let names: Vec<String> = doc_struct_fields("ChatInfoDocumentation.lua", "ChatMessageEventParams")
        .into_iter()
        .map(|field| format!("{:?}", field.name))
        .collect();
    env.exec(&format!(
        r##"
        local names = {{ {} }}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("CHAT_MSG_SAY")
        frame:SetScript("OnEvent", function(_, _, ...)
            ChatParams = {{ count = select("#", ...) }}
            for index, name in ipairs(names) do
                ChatParams[name] = (select(index, ...))
            end
        end)
        "##,
        names.join(", ")
    ))
    .expect("install chat listener");
    env.state()
        .borrow_mut()
        .host_chat_inputs
        .pending
        .push_back(HostChatMessage {
            kind: HostChatKind::Say,
            arguments: vec![
                HostChatArgument {
                    value: EventArg::String("from the bridge".into()),
                    secret: false,
                },
                HostChatArgument {
                    value: EventArg::String("Osso".into()),
                    secret: false,
                },
            ],
            discord_info: DiscordChatInfo {
                user_id: 4242.0,
                global_name: "osso.discord".into(),
                display_name_type: 2,
                has_attachment: true,
                from_discord: true,
                ..Default::default()
            },
        });
    assert!(env.publish_next_host_chat().expect("publish host chat"));
}

fn seed_cooldown_viewer(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    for cooldown in [
        CooldownViewerCooldown {
            cooldown_id: 501,
            spell_id: Some(2061),
            spell_category_id: Some(1234),
            equip_slot: Some(13),
            linked_spell_ids: vec![2050, 2060],
            is_known: true,
            is_invisible: true,
            category: 1,
            ..Default::default()
        },
        CooldownViewerCooldown {
            cooldown_id: 502,
            spell_id: Some(34433),
            category: 1,
            ..Default::default()
        },
    ] {
        state
            .cooldown_viewer_cooldowns
            .insert(cooldown.cooldown_id, cooldown);
    }
}

fn seed_owned_decor_pet(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let pet = &mut state.world.pets[0];
    pet.custom_name = Some("Sprocket".into());
    pet.is_favorite = true;
    pet.can_attach_to_decor = true;
    pet.creature_model_scale = Some(0.75);
}

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
    ShapeCase {
        source_ids: &[
            "structures-PlayerChoiceInfo-365",
            "structures-PlayerChoiceInfo-366",
        ],
        doc_file: "PlayerChoiceDocumentation.lua",
        struct_name: "PlayerChoiceInfo",
        seed: seed_player_choice,
        getter: "return C_PlayerChoice.GetCurrentPlayerChoiceInfo()",
        added: &["hideAnswerArt"],
        removed: &[],
        values: r#"return info.choiceID == 77 and info.hideAnswerArt == true and "ok" or "hideAnswerArt""#,
    },
    ShapeCase {
        source_ids: &[
            "structures-HousingDecorInstanceInfo-347",
            "structures-HousingDecorInstanceInfo-348",
        ],
        doc_file: "HousingDecorSharedDocumentation.lua",
        struct_name: "HousingDecorInstanceInfo",
        seed: seed_pet_attachable_decor,
        getter: "return C_HousingBasicMode.GetSelectedDecorInfo()",
        added: &["canAttachPet"],
        removed: &[],
        values: r#"
            if info.decorGUID ~= "Decor-Selection-2001" or info.canAttachPet ~= true then
                return "attachable"
            end
            local chair = C_HousingDecor.GetDecorInstanceInfoForGUID("Decor-Selection-1001")
            if chair.canAttachPet ~= false then return "not-attachable" end
            return "ok"
        "#,
    },
    ShapeCase {
        // The page lists isLFG; cached DelvesUIDocumentation.lua names the
        // field queueAsLFG and no cached Blizzard Lua reads either name.
        source_ids: &[
            "structures-TieredEntranceTierInfo-367",
            "structures-TieredEntranceTierInfo-368",
            "structures-TieredEntranceTierInfo-369",
        ],
        doc_file: "DelvesUIDocumentation.lua",
        struct_name: "TieredEntranceTierInfo",
        seed: no_seed,
        getter: "return C_DelvesUI.GetActiveDelveTier()",
        added: &["overrideTooltipSpellID", "queueAsLFG"],
        removed: &["isLFG"],
        values: r#"return info.queueAsLFG == false and "ok" or "queueAsLFG""#,
    },
    ShapeCase {
        source_ids: &[
            "structures-BNetAccountInfo-334",
            "structures-BNetAccountInfo-335",
            "structures-BNetAccountInfo-336",
        ],
        doc_file: "BattleNetDocumentation.lua",
        struct_name: "BNetAccountInfo",
        seed: seed_bnet_title_friend,
        getter: "return C_BattleNet.GetFriendAccountInfo(1)",
        added: &["friendLevel", "friendTags"],
        removed: &[],
        values: r#"
            if info.friendLevel ~= Enum.BattleNetFriendLevel.Title then return "friendLevel" end
            local other = C_BattleNet.GetFriendAccountInfo(2)
            if other.friendLevel ~= Enum.BattleNetFriendLevel.BattleTag then return "default-level" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-BNetGameAccountInfo-337",
            "structures-BNetGameAccountInfo-338",
        ],
        doc_file: "BattleNetDocumentation.lua",
        struct_name: "BNetGameAccountInfo",
        seed: seed_bnet_title_friend,
        getter: "return C_BattleNet.GetFriendAccountInfo(1).gameAccountInfo",
        added: &["classFilename"],
        removed: &[],
        values: r#"return info.classFilename == "DEATHKNIGHT" and "ok" or tostring(info.classFilename)"#,
    },
    ShapeCase {
        source_ids: &[
            "structures-ChatMessageEventParams-339",
            "structures-ChatMessageEventParams-340",
        ],
        doc_file: "ChatInfoDocumentation.lua",
        struct_name: "ChatMessageEventParams",
        seed: seed_discord_chat_line,
        getter: "return ChatParams",
        added: &["discordInfo"],
        removed: &[],
        // Blizzard's ChatFrameMixin:MessageEventHandler reads discordInfo as arg18.
        values: r#"
            if info.count ~= 18 or info.text ~= "from the bridge" or info.playerName ~= "Osso" then
                return "payload"
            end
            local discord = info.discordInfo
            if discord.userID ~= 4242 or discord.globalName ~= "osso.discord" then return "identity" end
            if discord.type ~= 2 or discord.fromDiscord ~= true or discord.hasAttachment ~= true then
                return "flags"
            end
            if discord.hasPoll ~= false or discord.lastOnlineName ~= "" then return "defaults" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-CooldownViewerCooldown-343",
            "structures-CooldownViewerCooldown-344",
            "structures-CooldownViewerCooldown-345",
            "structures-CooldownViewerCooldown-346",
        ],
        doc_file: "CooldownViewerDocumentation.lua",
        struct_name: "CooldownViewerCooldown",
        seed: seed_cooldown_viewer,
        getter: "return C_CooldownViewer.GetCooldownViewerCooldownInfo(501)",
        added: &["spellCategoryID", "equipSlot", "isInvisible"],
        removed: &[],
        values: r#"
            if info.cooldownID ~= 501 or info.spellID ~= 2061 then return "identity" end
            if info.spellCategoryID ~= 1234 or info.equipSlot ~= 13 or info.isInvisible ~= true then
                return "12.1.0 fields"
            end
            if info.linkedSpellIDs[2] ~= 2060 or info.category ~= 1 or info.isKnown ~= true then
                return "entry"
            end
            local plain = C_CooldownViewer.GetCooldownViewerCooldownInfo(502)
            if plain.spellCategoryID ~= nil or plain.equipSlot ~= nil or plain.isInvisible ~= false then
                return "unset fields"
            end
            local known = C_CooldownViewer.GetCooldownViewerCategorySet(1)
            if #known ~= 1 or known[1] ~= 501 then return "known set" end
            local all = C_CooldownViewer.GetCooldownViewerCategorySet(1, true)
            if #all ~= 2 or all[2] ~= 502 then return "unlearned set" end
            if C_CooldownViewer.GetCooldownViewerCooldownInfo(999) ~= nil then return "unknown" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &["structures-LfgEntryData-349", "structures-LfgEntryData-350"],
        doc_file: "LFGListInfoDocumentation.lua",
        struct_name: "LfgEntryData",
        seed: no_seed,
        getter: r#"
            if C_LFGList.GetActiveEntryInfo() ~= nil then return "listed before create" end
            assert(C_LFGList.CreateListing({
                activityIDs = { 493 }, isAutoAccept = true, requiredItemLevel = 600,
                generalPlaystyle = 2, isCrossFactionListing = true,
            }))
            return C_LFGList.GetActiveEntryInfo()
        "#,
        added: &["censored"],
        removed: &[],
        values: r#"
            if info.censored ~= false then return "censored" end
            if info.activityIDs[1] ~= 493 or info.autoAccept ~= true or info.requiredItemLevel ~= 600 then
                return "create data"
            end
            if info.generalPlaystyle ~= 2 or info.isCrossFactionListing ~= true or info.questID ~= nil then
                return "options"
            end
            if C_LFGList.HasActiveEntryInfo() ~= true then return "has" end
            if C_LFGList.CreateListing({ activityIDs = { 1 } }) ~= false then return "double create" end
            assert(C_LFGList.UpdateListing({ activityIDs = { 494 }, requiredItemLevel = 610 }))
            local updated = C_LFGList.GetActiveEntryInfo()
            if updated.activityIDs[1] ~= 494 or updated.autoAccept ~= false then return "update" end
            C_LFGList.RemoveListing()
            if C_LFGList.GetActiveEntryInfo() ~= nil or C_LFGList.HasActiveEntryInfo() then
                return "remove"
            end
            if C_LFGList.UpdateListing({ activityIDs = { 494 } }) ~= false then return "update unlisted" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-PetJournalPetInfo-353",
            "structures-PetJournalPetInfo-354",
            "structures-PetJournalPetInfo-355",
            "structures-PetJournalPetInfo-356",
            "structures-PetJournalPetInfo-357",
            "structures-PetJournalPetInfo-358",
            "structures-PetJournalPetInfo-359",
            "structures-PetJournalPetInfo-360",
            "structures-PetJournalPetInfo-361",
            "structures-PetJournalPetInfo-362",
        ],
        doc_file: "PetJournalInfoDocumentation.lua",
        struct_name: "PetJournalPetInfo",
        seed: seed_owned_decor_pet,
        // Owned pet: the Nilable owned-pet fields are populated.
        getter: r#"
            local petID = C_PetJournal.GetPetInfoByIndex(1)
            return C_PetJournal.GetPetInfoTableByPetID(petID)
        "#,
        added: &[
            "petLevel", "xp", "maxXP", "displayID", "isFavorite", "petType", "isWild",
            "tradable", "unique", "canAttachToDecor", "creatureModelScale",
        ],
        removed: &["isTradeable", "isUnique"],
        values: r#"
            if info.speciesID ~= 39 or info.customName ~= "Sprocket" or info.petLevel ~= 25 then
                return "owned identity"
            end
            if info.isFavorite ~= true or info.isWild ~= false or type(info.maxXP) ~= "number" then
                return "owned fields"
            end
            if info.canAttachToDecor ~= true or info.creatureModelScale ~= 0.75 then return "decor" end
            if C_PetJournal.GetPetInfoTableByPetID("BattlePet-0-FFFFFFFF") ~= nil then return "unknown" end
            return "ok"
        "#,
    },
    ShapeCase {
        source_ids: &[
            "structures-PetJournalPetInfo-353",
            "structures-PetJournalPetInfo-354",
            "structures-PetJournalPetInfo-355",
            "structures-PetJournalPetInfo-356",
            "structures-PetJournalPetInfo-357",
            "structures-PetJournalPetInfo-358",
            "structures-PetJournalPetInfo-360",
        ],
        doc_file: "PetJournalInfoDocumentation.lua",
        struct_name: "PetJournalPetInfo",
        seed: seed_owned_decor_pet,
        // Species query: the now-Nilable owned-pet fields are nil.
        getter: "return C_PetJournal.GetPetInfoTableBySpeciesID(39)",
        added: &[
            "petLevel", "xp", "maxXP", "displayID", "isFavorite", "isWild", "canAttachToDecor",
        ],
        removed: &[],
        values: r#"
            if info.customName ~= nil or info.petLevel ~= nil or info.xp ~= nil or info.maxXP ~= nil then
                return "owned fields leaked"
            end
            if info.isFavorite ~= nil or info.isWild ~= nil then return "owned flags leaked" end
            if info.name ~= "Mechanical Squirrel" or info.canAttachToDecor ~= true then return "species" end
            return "ok"
        "#,
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

/// PlaySoundParams is input-only: prove the documented fields reach the
/// recorded request (structures-PlaySoundParams-363/364).
#[test]
fn patch_12_1_0_play_sound_params_record_volume_override() {
    let doc = doc_struct_fields("SoundDocumentation.lua", "PlaySoundParams");
    let volume = doc
        .iter()
        .find(|field| field.name == "volumeOverride")
        .expect("cached PlaySoundParams lists volumeOverride");
    assert_eq!((volume.doc_type.as_str(), volume.nilable), ("number", true));

    let env = WowLuaEnv::new().expect("create sound environment");
    let returned: i32 = env
        .eval(
            r##"
            return select("#", C_Sound.PlaySoundWithOptions({
                soundKitID = 8959, uiSoundSubType = "Voice", volumeOverride = 0.35,
            }))
            "##,
        )
        .expect("play with volume override");
    assert_eq!(returned, 0, "headless playback returns nothing");
    let request = env.state().borrow().last_sound_request.clone();
    assert_eq!(
        request,
        Some(PlaySoundRequest {
            sound_kit_id: 8959,
            ui_sound_sub_type: Some("Voice".into()),
            force_no_duplicates: false,
            run_finish_callback: false,
            override_priority: None,
            volume_override: Some(0.35),
        })
    );

    env.exec("C_Sound.PlaySoundWithOptions({ soundKitID = 12867, forceNoDuplicates = true })")
        .expect("play without override");
    let request = env.state().borrow().last_sound_request.clone().unwrap();
    assert_eq!(
        (request.sound_kit_id, request.force_no_duplicates, request.volume_override),
        (12867, true, None)
    );
    assert!(
        env.exec(r#"C_Sound.PlaySoundWithOptions({ soundKitID = 1, volumeOverride = "loud" })"#)
            .is_err()
    );
    assert!(env.exec("C_Sound.PlaySoundWithOptions({ volumeOverride = 1 })").is_err());
}

/// Moderation state set on the active listing is what GetActiveEntryInfo reports,
/// and LFG_LIST_ACTIVE_ENTRY_UPDATE arrives on the next tick, not inside the call.
#[test]
fn patch_12_1_0_lfg_active_entry_censored_and_deferred_update() {
    let env = WowLuaEnv::new().expect("create lfg environment");
    env.exec(
        r#"
        EntryUpdates = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("LFG_LIST_ACTIVE_ENTRY_UPDATE")
        frame:SetScript("OnEvent", function(_, _, created)
            EntryUpdates[#EntryUpdates + 1] = tostring(created)
        end)
        assert(C_LFGList.CreateListing({ activityIDs = { 493 } }))
        assert(#EntryUpdates == 0)
        "#,
    )
    .expect("create listing");
    env.process_timers().expect("tick timers");
    env.state()
        .borrow_mut()
        .lfg_active_entry
        .as_mut()
        .expect("active listing")
        .censored = true;
    let observed: String = env
        .eval(
            r#"
            local censored = C_LFGList.GetActiveEntryInfo().censored
            C_LFGList.UpdateListing({ activityIDs = { 493 } })
            return tostring(censored) .. ":" .. tostring(C_LFGList.GetActiveEntryInfo().censored)
            "#,
        )
        .expect("read censored");
    env.process_timers().expect("tick timers");
    env.exec("C_LFGList.RemoveListing()").expect("remove");
    env.process_timers().expect("tick timers");
    let updates: String = env
        .eval("return table.concat(EntryUpdates, ',')")
        .expect("updates");
    assert_eq!(observed, "true:true");
    assert_eq!(updates, "true,false,nil");
}
