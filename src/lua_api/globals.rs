//! Global WoW API functions.

use super::SimState;
use crate::widget::{AttributeValue, Frame, WidgetType};
use mlua::{Lua, MetaMethod, ObjectLike, Result, UserData, UserDataMethods, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all global WoW API functions.
pub fn register_globals(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // CreateFrame(frameType, name, parent, template, id)
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();

        let frame_type: String = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Frame".to_string());

        let name: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        let parent_id: Option<u64> = args_iter.next().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });

        // Get parent ID (default to UIParent)
        let parent_id = parent_id.or_else(|| {
            let state = state_clone.borrow();
            state.widgets.get_id_by_name("UIParent")
        });

        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame = Frame::new(widget_type, name.clone(), parent_id);
        let frame_id = frame.id;

        let mut state = state_clone.borrow_mut();
        state.widgets.register(frame);

        if let Some(pid) = parent_id {
            state.widgets.add_child(pid, frame_id);
        }

        // Create userdata handle
        let handle = FrameHandle {
            id: frame_id,
            state: Rc::clone(&state_clone),
        };

        let ud = lua.create_userdata(handle)?;

        // Store reference in globals if named
        if let Some(ref n) = name {
            lua.globals().set(n.as_str(), ud.clone())?;
        }

        // Store reference for event dispatch
        let frame_key = format!("__frame_{}", frame_id);
        lua.globals().set(frame_key.as_str(), ud.clone())?;

        Ok(ud)
    })?;
    globals.set("CreateFrame", create_frame)?;

    // CreateTexturePool(parent, layer, subLayer, textureTemplate, resetterFunc)
    // Creates a pool for managing reusable textures
    let create_texture_pool_state = Rc::clone(&state);
    let create_texture_pool = lua.create_function(move |lua, args: mlua::MultiValue| {
        let _state = create_texture_pool_state.borrow();
        let _args: Vec<Value> = args.into_iter().collect();
        // Create a simple pool table with Acquire/Release methods
        let pool = lua.create_table()?;
        let pool_storage = lua.create_table()?;
        pool.set("__storage", pool_storage)?;
        pool.set("__active", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|lua, this: mlua::Table| {
                // Return a new texture-like table
                let texture = lua.create_table()?;
                texture.set("SetTexture", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetTexCoord", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetVertexColor", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetBlendMode", lua.create_function(|_, _: String| Ok(()))?)?;
                texture.set("SetDrawLayer", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetAllPoints", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("SetPoint", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("ClearAllPoints", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("SetAlpha", lua.create_function(|_, _: f64| Ok(()))?)?;
                texture.set("SetSize", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                texture.set("Show", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("Hide", lua.create_function(|_, ()| Ok(()))?)?;
                texture.set("SetParent", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                // Track in active list
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, texture.clone())?;
                Ok(texture)
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _texture): (mlua::Table, mlua::Table)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateTexturePool", create_texture_pool)?;

    // CreateFramePool(frameType, parent, template, resetterFunc, forbidden)
    // Creates a pool for managing reusable frames
    let create_frame_pool_state = Rc::clone(&state);
    let create_frame_pool = lua.create_function(move |lua, args: mlua::MultiValue| {
        let _state = create_frame_pool_state.borrow();
        let _args: Vec<Value> = args.into_iter().collect();
        let pool = lua.create_table()?;
        pool.set("__active", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|lua, this: mlua::Table| {
                // Create a simple frame table
                let frame = lua.create_table()?;
                frame.set("Show", lua.create_function(|_, ()| Ok(()))?)?;
                frame.set("Hide", lua.create_function(|_, ()| Ok(()))?)?;
                frame.set("SetPoint", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                frame.set("ClearAllPoints", lua.create_function(|_, ()| Ok(()))?)?;
                frame.set("SetParent", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, frame.clone())?;
                Ok(frame)
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _frame): (mlua::Table, mlua::Table)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateFramePool", create_frame_pool)?;

    // CreateObjectPool(creatorFunc, resetterFunc) - generic object pool
    let create_object_pool = lua.create_function(|lua, (creator_func, _resetter_func): (mlua::Function, Option<mlua::Function>)| {
        let pool = lua.create_table()?;
        pool.set("__creator", creator_func.clone())?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("__inactive", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|_lua, this: mlua::Table| {
                let creator: mlua::Function = this.get("__creator")?;
                let obj = creator.call::<Value>(())?;
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, obj.clone())?;
                Ok((obj, true)) // Return object and isNew flag
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _obj): (mlua::Table, Value)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        pool.set(
            "GetNumActive",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                Ok(active.raw_len())
            })?,
        )?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, _this: mlua::Table| {
                // Return an iterator function (simple stub)
                lua.create_function(|_, ()| Ok(Value::Nil))
            })?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateObjectPool", create_object_pool)?;

    // UIParent reference
    let ui_parent_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("UIParent").unwrap()
    };
    let ui_parent = lua.create_userdata(FrameHandle {
        id: ui_parent_id,
        state: Rc::clone(&state),
    })?;
    globals.set("UIParent", ui_parent)?;

    // Minimap reference (built-in UI element)
    let minimap_id = {
        let state = state.borrow();
        state.widgets.get_id_by_name("Minimap").unwrap()
    };
    let minimap = lua.create_userdata(FrameHandle {
        id: minimap_id,
        state: Rc::clone(&state),
    })?;
    globals.set("Minimap", minimap)?;

    // AddonCompartmentFrame (retail UI element for addon buttons)
    let addon_compartment = lua.create_table()?;
    addon_compartment.set(
        "RegisterAddon",
        lua.create_function(|_, _info: mlua::Table| Ok(()))?,
    )?;
    addon_compartment.set(
        "UnregisterAddon",
        lua.create_function(|_, _addon: String| Ok(()))?,
    )?;
    globals.set("AddonCompartmentFrame", addon_compartment)?;

    // StaticPopupDialogs - table for popup dialog definitions
    let static_popup_dialogs = lua.create_table()?;
    globals.set("StaticPopupDialogs", static_popup_dialogs)?;

    // StaticPopup_Show(name, text1, text2, ...) - show a static popup
    globals.set(
        "StaticPopup_Show",
        lua.create_function(|_, _args: mlua::Variadic<Value>| Ok(Value::Nil))?,
    )?;
    globals.set(
        "StaticPopup_Hide",
        lua.create_function(|_, _name: String| Ok(()))?,
    )?;

    // print() - already exists in Lua but we can customize if needed

    // strsplit(delimiter, str, limit) - WoW string utility
    let strsplit = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let delimiter = args
            .first()
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| " ".to_string());

        let input = args
            .get(1)
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let limit = args
            .get(2)
            .and_then(|v| {
                if let Value::Integer(n) = v {
                    Some(*n as usize)
                } else if let Value::Number(n) = v {
                    Some(*n as usize)
                } else {
                    None
                }
            });

        let parts: Vec<&str> = if let Some(limit) = limit {
            input.splitn(limit, &delimiter).collect()
        } else {
            input.split(&delimiter).collect()
        };

        let mut result = mlua::MultiValue::new();
        for part in parts {
            result.push_back(Value::String(lua.create_string(part)?));
        }
        Ok(result)
    })?;
    globals.set("strsplit", strsplit)?;

    // wipe(table) - Clear a table in place
    let wipe = lua.create_function(|_, table: mlua::Table| {
        // Get all keys first to avoid modification during iteration
        let keys: Vec<Value> = table
            .pairs::<Value, Value>()
            .filter_map(|r| r.ok().map(|(k, _)| k))
            .collect();

        for key in keys {
            table.set(key, Value::Nil)?;
        }
        Ok(table)
    })?;
    globals.set("wipe", wipe)?;

    // tinsert - alias for table.insert
    let tinsert = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_insert: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("insert")?;
        table_insert.call::<()>(args)?;
        Ok(())
    })?;
    globals.set("tinsert", tinsert)?;

    // tremove - alias for table.remove
    let tremove = lua.create_function(|lua, args: mlua::MultiValue| {
        let table_remove: mlua::Function = lua.globals().get::<mlua::Table>("table")?.get("remove")?;
        table_remove.call::<Value>(args)
    })?;
    globals.set("tremove", tremove)?;

    // tInvert - invert table (swap keys and values)
    let tinvert = lua.create_function(|lua, tbl: mlua::Table| {
        let result = lua.create_table()?;
        for pair in tbl.pairs::<Value, Value>() {
            let (k, v) = pair?;
            result.set(v, k)?;
        }
        Ok(result)
    })?;
    globals.set("tInvert", tinvert)?;

    // hooksecurefunc(name, hook) or hooksecurefunc(table, name, hook)
    let hooksecurefunc = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();

        let (table, name, hook) = if args.len() == 2 {
            // hooksecurefunc("FuncName", hookFunc)
            let name = if let Value::String(s) = &args[0] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[1].clone();
            (lua.globals(), name, hook)
        } else if args.len() >= 3 {
            // hooksecurefunc(someTable, "FuncName", hookFunc)
            let table = if let Value::Table(t) = &args[0] {
                t.clone()
            } else {
                lua.globals()
            };
            let name = if let Value::String(s) = &args[1] {
                s.to_string_lossy().to_string()
            } else {
                String::new()
            };
            let hook = args[2].clone();
            (table, name, hook)
        } else {
            return Ok(());
        };

        // Get the original function
        let original: Value = table.get::<Value>(name.as_str())?;

        if let (Value::Function(orig_fn), Value::Function(hook_fn)) = (original, hook) {
            // Create a wrapper that calls original then hook
            let wrapper = lua.create_function(move |_, args: mlua::MultiValue| {
                // Call original
                let result = orig_fn.call::<mlua::MultiValue>(args.clone())?;
                // Call hook (ignoring its result)
                let _ = hook_fn.call::<mlua::MultiValue>(args);
                Ok(result)
            })?;

            table.set(name.as_str(), wrapper)?;
        }

        Ok(())
    })?;
    globals.set("hooksecurefunc", hooksecurefunc)?;

    // GetBuildInfo() - Return mock game version
    let get_build_info = lua.create_function(|lua, ()| {
        // Return: version, build, date, tocversion, localizedVersion, buildType
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.0")?),  // version
            Value::String(lua.create_string("99999")?),   // build
            Value::String(lua.create_string("Jan 1 2025")?), // date
            Value::Integer(110000),                        // tocversion
            Value::String(lua.create_string("11.0.0")?),  // localizedVersion
            Value::String(lua.create_string("Release")?), // buildType
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // GetRealmName() - Return mock realm name
    let get_realm_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("SimulatedRealm")?))
    })?;
    globals.set("GetRealmName", get_realm_name)?;

    // GetNormalizedRealmName() - Return mock normalized realm name
    let get_normalized_realm_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("SimulatedRealm")?))
    })?;
    globals.set("GetNormalizedRealmName", get_normalized_realm_name)?;

    // GetLocale() - Return mock locale
    let get_locale = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("enUS")?))
    })?;
    globals.set("GetLocale", get_locale)?;

    // SlashCmdList table
    let slash_cmd_list = lua.create_table()?;
    globals.set("SlashCmdList", slash_cmd_list)?;

    // Enum table (WoW uses this for various enumerations)
    let enum_table = lua.create_table()?;

    // Enum.LFGRole
    let lfg_role = lua.create_table()?;
    lfg_role.set("Tank", 0)?;
    lfg_role.set("Healer", 1)?;
    lfg_role.set("Damage", 2)?;
    enum_table.set("LFGRole", lfg_role)?;

    // Enum.UnitSex
    let unit_sex = lua.create_table()?;
    unit_sex.set("Male", 2)?;
    unit_sex.set("Female", 3)?;
    enum_table.set("UnitSex", unit_sex)?;

    // Enum.GameMode
    let game_mode = lua.create_table()?;
    game_mode.set("Standard", 0)?;
    game_mode.set("Plunderstorm", 1)?;
    game_mode.set("WoWHack", 2)?;
    enum_table.set("GameMode", game_mode)?;

    // Enum.Profession
    let profession = lua.create_table()?;
    profession.set("Mining", 1)?;
    profession.set("Skinning", 2)?;
    profession.set("Herbalism", 3)?;
    profession.set("Blacksmithing", 4)?;
    profession.set("Leatherworking", 5)?;
    profession.set("Alchemy", 6)?;
    profession.set("Tailoring", 7)?;
    profession.set("Engineering", 8)?;
    profession.set("Enchanting", 9)?;
    profession.set("Fishing", 10)?;
    profession.set("Cooking", 11)?;
    profession.set("Jewelcrafting", 12)?;
    profession.set("Inscription", 13)?;
    profession.set("Archaeology", 14)?;
    enum_table.set("Profession", profession)?;

    // Enum.VasTransactionPurchaseResult - all values used by VASErrorLookup.lua
    let vas_result = lua.create_table()?;
    for (i, name) in [
        "Ok", "NotAvailable", "InProgress", "OnlyOneVasAtATime",
        "InvalidDestinationAccount", "InvalidSourceAccount", "InvalidCharacter",
        "NotEnoughMoney", "NotEligible", "TransferServiceDisabled",
        "DifferentRegion", "RealmNotEligible", "CharacterNotOnAccount",
        "TooManyCharacters", "InternalError", "PendingOtherProduct",
        "PendingItemDelivery", "PurchaseInProgress", "GenericError",
        "DisallowedSourceAccount", "DisallowedDestinationAccount", "LowerBoxLevel",
        "MaxCharactersOnServer", "CantAffordService", "ServiceAvailable",
        "CharacterHasGuildBank", "NameNotAvailable", "CharacterBelongsToGuild",
        "LockedForVas", "MoveInProgress", "AgeRestriction", "UnderMinAge",
        "BoostedTooRecently", "NewPlayerRestrictions", "CannotRestore",
        "GuildHasGuildBank", "CharacterArenaTeam", "CharacterTransferInProgress",
        "CharacterTransferPending", "RaceClassComboNotEligible", "InvalidStartingLevel",
        // Proxy errors
        "ProxyBadRequestContained", "ProxyCharacterTransferredNoBoostInProgress",
        // Database errors
        "DbRealmNotEligible", "DbCannotMoveGuildmaster", "DbMaxCharactersOnServer",
        "DbNoMixedAlliance", "DbDuplicateCharacterName", "DbHasMail", "DbMoveInProgress",
        "DbUnderMinLevelReq", "DbIneligibleTargetRealm", "DbTransferDateTooSoon",
        "DbCharLocked", "DbAllianceNotEligible", "DbTooMuchMoneyForLevel",
        "DbHasAuctions", "DbLastSaveTooRecent", "DbNameNotAvailable",
        "DbLastRenameTooRecent", "DbAlreadyRenameFlagged", "DbCustomizeAlreadyRequested",
        "DbLastCustomizeTooSoon", "DbFactionChangeTooSoon", "DbRaceClassComboIneligible",
        "DbPendingItemAudit", "DbGuildRankInsufficient", "DbCharacterWithoutGuild",
        "DbGmSenorityInsufficient", "DbAuthenticatorInsufficient", "DbIneligibleMapID",
        "DbBpayDeliveryPending", "DbHasBpayToken", "DbHasHeirloomItem",
        "DbResultAccountRestricted", "DbLastSaveTooDistant", "DbCagedPetInInventory",
        "DbOnBoostCooldown", "DbPvEPvPTransferNotAllowed", "DbNewLeaderInvalid",
        "DbNeedsLevelSquish", "DbHasNewPlayerExperienceRestriction", "DbHasCraftingOrders",
        "DbInvalidName", "DbNeedsEraChoice", "DbCannotMoveArenaCaptn",
    ].iter().enumerate() {
        vas_result.set(*name, i as i32)?;
    }
    enum_table.set("VasTransactionPurchaseResult", vas_result)?;

    // Enum.StoreError - store error codes
    let store_error = lua.create_table()?;
    for (i, name) in [
        "InvalidPaymentMethod", "PaymentFailed", "WrongCurrency", "BattlepayDisabled",
        "InsufficientBalance", "Other", "AlreadyOwned", "ParentalControlsNoPurchase",
        "PurchaseDenied", "ConsumableTokenOwned", "TooManyTokens", "ItemUnavailable",
        "ClientRestricted",
    ].iter().enumerate() {
        store_error.set(*name, i as i32)?;
    }
    enum_table.set("StoreError", store_error)?;

    // Enum.GameRule - game rule identifiers
    let game_rule = lua.create_table()?;
    for (i, name) in [
        "PlayerCastBarDisabled", "TargetCastBarDisabled", "NameplateCastBarDisabled",
        "UserAddonsDisabled", "EncounterJournalDisabled", "EjSuggestedContentDisabled",
        "EjDungeonsDisabled", "EjRaidsDisabled", "EjItemSetsDisabled",
        "ExperienceBarDisabled", "ActionButtonTypeOverlayStrategy",
    ].iter().enumerate() {
        game_rule.set(*name, i as i32)?;
    }
    enum_table.set("GameRule", game_rule)?;

    // Enum.ScriptedAnimationBehavior (many values needed)
    let animation_behavior = lua.create_table()?;
    for (i, name) in [
        "None", "FollowsCaster", "FollowsTarget", "SourceRecoil",
        "SourceCollideWithTarget", "TargetShake", "TargetKnockBack",
        "UIParentShake", "TargetCenter", "TargetCenterToSource",
    ].iter().enumerate() {
        animation_behavior.set(*name, i as i32)?;
    }
    enum_table.set("ScriptedAnimationBehavior", animation_behavior)?;

    // Enum.ScriptedAnimationTrajectory
    let animation_trajectory = lua.create_table()?;
    for (i, name) in [
        "AtSource", "Straight", "CurveLeft", "CurveRight", "CurveRandom",
        "AtTarget", "HalfwayBetween", "SourceToTarget", "TargetToSource",
    ].iter().enumerate() {
        animation_trajectory.set(*name, i as i32)?;
    }
    enum_table.set("ScriptedAnimationTrajectory", animation_trajectory)?;

    globals.set("Enum", enum_table)?;

    // C_UIColor namespace (color utilities)
    let c_ui_color = lua.create_table()?;
    let get_colors = lua.create_function(|lua, ()| {
        // Return an empty table of colors
        lua.create_table()
    })?;
    c_ui_color.set("GetColors", get_colors)?;
    globals.set("C_UIColor", c_ui_color)?;

    // C_ClassColor namespace
    let c_class_color = lua.create_table()?;
    let get_class_color = lua.create_function(|lua, _class_name: String| {
        // Return a color object with methods (same as CreateColor)
        let r = 1.0f32;
        let g = 1.0f32;
        let b = 1.0f32;
        let a = 1.0f32;

        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("a", a)?;

        let get_rgb = lua.create_function(move |_, ()| Ok((r, g, b)))?;
        color.set("GetRGB", get_rgb)?;

        let get_rgba = lua.create_function(move |_, ()| Ok((r, g, b, a)))?;
        color.set("GetRGBA", get_rgba)?;

        let generate_hex = lua.create_function(move |lua, ()| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            Ok(Value::String(lua.create_string(&hex)?))
        })?;
        color.set("GenerateHexColor", generate_hex)?;

        let wrap_text = lua.create_function(move |lua, text: String| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            let wrapped = format!("|cff{}{}|r", hex, text);
            Ok(Value::String(lua.create_string(&wrapped)?))
        })?;
        color.set("WrapTextInColorCode", wrap_text)?;

        Ok(color)
    })?;
    c_class_color.set("GetClassColor", get_class_color)?;
    globals.set("C_ClassColor", c_class_color)?;

    // C_GameRules namespace
    let c_game_rules = lua.create_table()?;
    let is_active = lua.create_function(|_, _rule: Value| {
        Ok(false) // No special game rules in simulation
    })?;
    c_game_rules.set("IsGameRuleActive", is_active)?;

    let get_active_game_mode = lua.create_function(|_, ()| {
        Ok(0) // Enum.GameMode.Standard
    })?;
    c_game_rules.set("GetActiveGameMode", get_active_game_mode)?;

    let get_game_rule_as_float = lua.create_function(|_, _rule: Value| {
        Ok(0.0f32) // Default value for numeric game rules
    })?;
    c_game_rules.set("GetGameRuleAsFloat", get_game_rule_as_float)?;

    let is_standard = lua.create_function(|_, ()| {
        Ok(true) // Always standard mode in simulation
    })?;
    c_game_rules.set("IsStandard", is_standard)?;

    globals.set("C_GameRules", c_game_rules)?;

    // C_ScriptedAnimations namespace
    let c_scripted_anims = lua.create_table()?;
    let get_all_effects = lua.create_function(|lua, ()| {
        // Return empty array - no scripted animation effects in simulation
        lua.create_table()
    })?;
    c_scripted_anims.set("GetAllScriptedAnimationEffects", get_all_effects)?;
    globals.set("C_ScriptedAnimations", c_scripted_anims)?;

    // C_Glue namespace (glue screen utilities)
    let c_glue = lua.create_table()?;
    let is_on_glue_screen = lua.create_function(|_, ()| {
        Ok(false) // Not on character select/login screen
    })?;
    c_glue.set("IsOnGlueScreen", is_on_glue_screen)?;
    globals.set("C_Glue", c_glue)?;

    // Unit info functions (stubs for simulation)
    let unit_race = lua.create_function(|lua, _unit: String| {
        // Return: raceName, raceFile
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Human")?),
            Value::String(lua.create_string("Human")?),
        ]))
    })?;
    globals.set("UnitRace", unit_race)?;

    let unit_sex = lua.create_function(|_, _unit: String| {
        // Return: 2 for male, 3 for female (matches Enum.UnitSex)
        Ok(2)
    })?;
    globals.set("UnitSex", unit_sex)?;

    let unit_class = lua.create_function(|lua, _unit: String| {
        // Return: className, classFile, classID
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Warrior")?),
            Value::String(lua.create_string("WARRIOR")?),
            Value::Integer(1),
        ]))
    })?;
    globals.set("UnitClass", unit_class)?;

    // UnitName(unit) - Return name and realm
    let unit_name = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm (realm is nil for same-realm units)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::Nil,
        ]))
    })?;
    globals.set("UnitName", unit_name)?;

    // UnitNameUnmodified(unit) - Return raw name (used for BattleTag lookups)
    let unit_name_unmodified = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm (realm is nil for same-realm units)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::Nil,
        ]))
    })?;
    globals.set("UnitNameUnmodified", unit_name_unmodified)?;

    // UnitFullName(unit) - Return name with realm
    let unit_full_name = lua.create_function(|lua, unit: String| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        // Return: name, realm
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string(name)?),
            Value::String(lua.create_string("SimRealm")?),
        ]))
    })?;
    globals.set("UnitFullName", unit_full_name)?;

    // GetUnitName(unit, showServerName) - alias for UnitName with server name option
    let get_unit_name = lua.create_function(|lua, (unit, _show_server): (String, Option<bool>)| {
        let name = match unit.as_str() {
            "player" => "SimPlayer",
            _ => "SimUnit",
        };
        Ok(Value::String(lua.create_string(name)?))
    })?;
    globals.set("GetUnitName", get_unit_name)?;

    // UnitGUID(unit) - Return unit GUID
    let unit_guid = lua.create_function(|lua, unit: String| {
        let guid = match unit.as_str() {
            "player" => "Player-0000-00000001",
            _ => "Creature-0000-00000000",
        };
        Ok(Value::String(lua.create_string(guid)?))
    })?;
    globals.set("UnitGUID", unit_guid)?;

    // UnitLevel(unit) - Return unit level
    let unit_level = lua.create_function(|_, _unit: String| Ok(70))?;
    globals.set("UnitLevel", unit_level)?;

    // UnitExists(unit) - Check if unit exists
    let unit_exists = lua.create_function(|_, unit: String| {
        Ok(matches!(unit.as_str(), "player" | "target" | "pet"))
    })?;
    globals.set("UnitExists", unit_exists)?;

    // UnitFactionGroup(unit) - Return faction
    let unit_faction_group = lua.create_function(|lua, _unit: String| {
        // Return: englishFaction, localizedFaction
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("Alliance")?),
            Value::String(lua.create_string("Alliance")?),
        ]))
    })?;
    globals.set("UnitFactionGroup", unit_faction_group)?;

    // IsLoggedIn() - Check if player is logged in (always true in sim)
    globals.set("IsLoggedIn", lua.create_function(|_, ()| Ok(false))?)?;

    // GetCurrentRegion() - Return region ID
    let get_current_region = lua.create_function(|_, ()| {
        // 1=US, 2=Korea, 3=Europe, 4=Taiwan, 5=China
        Ok(1)
    })?;
    globals.set("GetCurrentRegion", get_current_region)?;

    // GetCurrentRegionName() - Return region name
    let get_current_region_name = lua.create_function(|lua, ()| {
        Ok(Value::String(lua.create_string("US")?))
    })?;
    globals.set("GetCurrentRegionName", get_current_region_name)?;

    // GetBuildInfo() - Return game version info
    let get_build_info = lua.create_function(|lua, ()| {
        // Returns: version, build, date, tocversion
        // 11.0.7 is The War Within (TWW)
        Ok(mlua::MultiValue::from_vec(vec![
            Value::String(lua.create_string("11.0.7")?), // version
            Value::String(lua.create_string("58238")?),  // build
            Value::String(lua.create_string("Jan 7 2025")?), // date
            Value::Integer(110007), // tocversion
        ]))
    })?;
    globals.set("GetBuildInfo", get_build_info)?;

    // GetDifficultyInfo(difficultyID) - Return difficulty info
    globals.set(
        "GetDifficultyInfo",
        lua.create_function(|lua, difficulty_id: i32| {
            // Return: name, groupType, isHeroic, isChallengeMode, displayHeroic, displayMythic, toggleDifficultyID
            let (name, group_type, is_heroic, is_mythic) = match difficulty_id {
                1 => ("Normal", "party", false, false),
                2 => ("Heroic", "party", true, false),
                3 => ("Normal", "raid", false, false),
                4 => ("Normal", "raid", false, false),
                5 => ("Heroic", "raid", true, false),
                6 => ("Heroic", "raid", true, false),
                7 => ("Legacy LFR", "raid", false, false),
                8 => ("Mythic Keystone", "party", false, true),
                9 => ("Legacy 40-player", "raid", false, false),
                14 => ("Normal", "raid", false, false),
                15 => ("Heroic", "raid", true, false),
                16 => ("Mythic", "raid", false, true),
                17 => ("LFR", "raid", false, false),
                23 => ("Mythic", "party", false, true),
                _ => ("Unknown", "none", false, false),
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string(name)?),
                Value::String(lua.create_string(group_type)?),
                Value::Boolean(is_heroic),
                Value::Boolean(false), // isChallengeMode
                Value::Boolean(is_heroic),
                Value::Boolean(is_mythic),
                Value::Nil, // toggleDifficultyID
            ]))
        })?,
    )?;

    // GetNumShapeshiftForms() - Return number of shapeshift forms
    globals.set(
        "GetNumShapeshiftForms",
        lua.create_function(|_, ()| Ok(0))?,
    )?;

    // GetShapeshiftFormInfo(index) - Return shapeshift form info
    globals.set(
        "GetShapeshiftFormInfo",
        lua.create_function(|_, _index: i32| {
            // Return: texture, name, isActive, isCastable
            Ok(Value::Nil)
        })?,
    )?;

    // GetPhysicalScreenSize() - Return physical screen dimensions
    let get_physical_screen_size = lua.create_function(|_, ()| {
        // Return simulated 1920x1080 screen
        Ok((1920, 1080))
    })?;
    globals.set("GetPhysicalScreenSize", get_physical_screen_size)?;

    // GetScreenWidth() - Return screen width in UI units
    globals.set(
        "GetScreenWidth",
        lua.create_function(|_, ()| Ok(1920.0))?,
    )?;

    // GetScreenHeight() - Return screen height in UI units
    globals.set(
        "GetScreenHeight",
        lua.create_function(|_, ()| Ok(1080.0))?,
    )?;

    // UnitAttackSpeed(unit) - Return attack speed info
    globals.set(
        "UnitAttackSpeed",
        lua.create_function(|_, _unit: String| {
            // Return: mainHandSpeed, offHandSpeed
            Ok((2.0, Value::Nil))
        })?,
    )?;

    // GetTexCoordsByGrid(row, col, rows, cols) - Calculate texture coordinates for grid
    globals.set(
        "GetTexCoordsByGrid",
        lua.create_function(|_, (col, row, grid_cols, grid_rows): (i32, i32, Option<i32>, Option<i32>)| {
            let cols = grid_cols.unwrap_or(1);
            let rows = grid_rows.unwrap_or(1);
            if cols == 0 || rows == 0 {
                return Ok((0.0, 1.0, 0.0, 1.0));
            }
            let cell_width = 1.0 / cols as f64;
            let cell_height = 1.0 / rows as f64;
            let left = (col - 1) as f64 * cell_width;
            let right = col as f64 * cell_width;
            let top = (row - 1) as f64 * cell_height;
            let bottom = row as f64 * cell_height;
            Ok((left, right, top, bottom))
        })?,
    )?;

    // IsAddonMessagePrefixRegistered(prefix) - Check if addon message prefix is registered
    globals.set(
        "IsAddonMessagePrefixRegistered",
        lua.create_function(|_, _prefix: String| Ok(false))?,
    )?;

    // RegisterAddonMessagePrefix(prefix) - Register addon message prefix
    globals.set(
        "RegisterAddonMessagePrefix",
        lua.create_function(|_, _prefix: String| Ok(true))?,
    )?;

    // CreateTextureMarkup(file, fileWidth, fileHeight, width, height, left, right, top, bottom) - create texture markup string
    globals.set(
        "CreateTextureMarkup",
        lua.create_function(
            |lua, (file, _file_width, _file_height, width, height, _left, _right, _top, _bottom): (
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
                Option<f64>,
            )| {
                let w = width.unwrap_or(0);
                let h = height.unwrap_or(0);
                let markup = format!("|T{}:{}:{}|t", file, h, w);
                Ok(Value::String(lua.create_string(&markup)?))
            },
        )?,
    )?;

    // CreateAtlasMarkup(atlas, width, height, offsetX, offsetY) - create atlas texture markup string
    globals.set(
        "CreateAtlasMarkup",
        lua.create_function(
            |lua, (atlas, width, height, _offset_x, _offset_y): (
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
            )| {
                let w = width.unwrap_or(0);
                let h = height.unwrap_or(0);
                let markup = format!("|A:{}:{}:{}|a", atlas, h, w);
                Ok(Value::String(lua.create_string(&markup)?))
            },
        )?,
    )?;

    // GetInventoryItemTexture(unit, slot) - get texture ID for equipped item
    globals.set(
        "GetInventoryItemTexture",
        lua.create_function(|_, (_unit, _slot): (String, i32)| {
            // Return nil - no items equipped in simulation
            Ok(Value::Nil)
        })?,
    )?;

    // GetInventoryItemID(unit, slot) - get item ID for equipped item
    globals.set(
        "GetInventoryItemID",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;

    // GetInventoryItemLink(unit, slot) - get item link for equipped item
    globals.set(
        "GetInventoryItemLink",
        lua.create_function(|_, (_unit, _slot): (String, i32)| Ok(Value::Nil))?,
    )?;

    // UnitPlayerControlled(unit) - Check if unit is player controlled
    let unit_player_controlled = lua.create_function(|_, unit: String| {
        // Player, party, raid members are player controlled
        Ok(unit.starts_with("player")
            || unit.starts_with("party")
            || unit.starts_with("raid")
            || unit == "pet")
    })?;
    globals.set("UnitPlayerControlled", unit_player_controlled)?;

    // UnitIsTapDenied(unit) - Check if unit is tapped by another player
    let unit_is_tap_denied = lua.create_function(|_, _unit: String| {
        // In simulation, nothing is tapped
        Ok(false)
    })?;
    globals.set("UnitIsTapDenied", unit_is_tap_denied)?;

    // PixelUtil namespace - pixel snapping utilities
    let pixel_util = lua.create_table()?;
    pixel_util.set(
        "SetWidth",
        lua.create_function(|_, (frame, width): (mlua::AnyUserData, f64)| {
            // Just forward to frame:SetWidth
            frame.call_method::<()>("SetWidth", width)?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetHeight",
        lua.create_function(|_, (frame, height): (mlua::AnyUserData, f64)| {
            frame.call_method::<()>("SetHeight", height)?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetSize",
        lua.create_function(|_, (frame, width, height): (mlua::AnyUserData, f64, f64)| {
            frame.call_method::<()>("SetSize", (width, height))?;
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "SetPoint",
        lua.create_function(|_, args: mlua::MultiValue| {
            let mut args_iter = args.into_iter();
            if let Some(Value::UserData(frame)) = args_iter.next() {
                // Forward remaining args to frame:SetPoint
                let remaining: Vec<Value> = args_iter.collect();
                frame.call_method::<()>("SetPoint", mlua::MultiValue::from_vec(remaining))?;
            }
            Ok(())
        })?,
    )?;
    pixel_util.set(
        "GetPixelToUIUnitFactor",
        lua.create_function(|_, ()| Ok(1.0))?,
    )?;
    globals.set("PixelUtil", pixel_util)?;

    // Constants table (WoW uses this for various constants)
    let constants_table = lua.create_table()?;
    // LFG role constants
    let lfg_role_constants = lua.create_table()?;
    lfg_role_constants.set("LFG_ROLE_TANK", 0)?;
    lfg_role_constants.set("LFG_ROLE_HEALER", 1)?;
    lfg_role_constants.set("LFG_ROLE_DAMAGE", 2)?;
    lfg_role_constants.set("LFG_ROLE_NO_ROLE", 3)?;
    constants_table.set("LFG_ROLEConstants", lfg_role_constants)?;

    // AccountStoreConsts
    let account_store_consts = lua.create_table()?;
    account_store_consts.set("PlunderstormStoreFrontID", 1)?;
    account_store_consts.set("WowhackStoreFrontID", 2)?;
    constants_table.set("AccountStoreConsts", account_store_consts)?;

    globals.set("Constants", constants_table)?;

    // GetCurrentEnvironment() - returns the current global environment table
    let get_current_environment = lua.create_function(|lua, ()| {
        // Return _G (the global environment table)
        Ok(lua.globals())
    })?;
    globals.set("GetCurrentEnvironment", get_current_environment)?;

    // WOW_PROJECT constants
    globals.set("WOW_PROJECT_MAINLINE", 1)?;
    globals.set("WOW_PROJECT_CLASSIC", 2)?;
    globals.set("WOW_PROJECT_BURNING_CRUSADE_CLASSIC", 5)?;
    globals.set("WOW_PROJECT_WRATH_CLASSIC", 11)?;
    globals.set("WOW_PROJECT_CATACLYSM_CLASSIC", 14)?;
    globals.set("WOW_PROJECT_ID", 1)?; // Mainline

    // nop() - no-operation function
    let nop = lua.create_function(|_, _: mlua::MultiValue| {
        Ok(())
    })?;
    globals.set("nop", nop)?;

    // securecallfunction(func, ...) - calls a function in protected mode
    let securecallfunction = lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
        // In WoW this provides taint protection, but for simulation we just call it
        func.call::<mlua::MultiValue>(args)
    })?;
    globals.set("securecallfunction", securecallfunction)?;

    // securecall(func, ...) - alias for securecallfunction
    let securecall = lua.create_function(|_, (func, args): (mlua::Function, mlua::MultiValue)| {
        func.call::<mlua::MultiValue>(args)
    })?;
    globals.set("securecall", securecall)?;

    // geterrorhandler() - returns error handler function
    let geterrorhandler = lua.create_function(|lua, ()| {
        // Return a simple error handler that just prints
        let handler = lua.create_function(|_, msg: String| {
            println!("Lua error: {}", msg);
            Ok(())
        })?;
        Ok(handler)
    })?;
    globals.set("geterrorhandler", geterrorhandler)?;

    // Lua stdlib global aliases (WoW compatibility)
    lua.load(r##"
        -- String library aliases
        strlen = string.len
        strsub = string.sub
        strfind = string.find
        strmatch = string.match
        strbyte = string.byte
        strchar = string.char
        strrep = string.rep
        strrev = string.reverse
        strlower = string.lower
        strupper = string.upper
        strtrim = function(s) return (s:gsub("^%s*(.-)%s*$", "%1")) end
        strsplittable = function(del, str) local t = {} for v in string.gmatch(str, "([^"..del.."]+)") do t[#t+1] = v end return t end
        format = string.format
        gsub = string.gsub
        gmatch = string.gmatch

        -- Math library aliases
        abs = math.abs
        ceil = math.ceil
        floor = math.floor
        max = math.max
        min = math.min
        mod = math.fmod
        sqrt = math.sqrt
        sin = function(x) return math.sin(math.rad(x)) end
        cos = function(x) return math.cos(math.rad(x)) end
        tan = function(x) return math.tan(math.rad(x)) end
        asin = function(x) return math.deg(math.asin(x)) end
        acos = function(x) return math.deg(math.acos(x)) end
        atan = function(x) return math.deg(math.atan(x)) end
        atan2 = function(x, y) return math.deg(math.atan2(x, y)) end
        deg = math.deg
        rad = math.rad
        exp = math.exp
        log = math.log
        log10 = math.log10
        frexp = math.frexp
        ldexp = math.ldexp
        random = math.random
        PI = math.pi

        -- WoW math utility functions
        function Round(value)
            if value < 0 then
                return math.ceil(value - 0.5)
            else
                return math.floor(value + 0.5)
            end
        end

        function Lerp(startValue, endValue, amount)
            return startValue + (endValue - startValue) * amount
        end

        function Clamp(value, min, max)
            if value < min then return min end
            if value > max then return max end
            return value
        end

        function Saturate(value)
            return Clamp(value, 0.0, 1.0)
        end

        function ClampedPercentageBetween(value, min, max)
            if max <= min then return 0.0 end
            return Saturate((value - min) / (max - min))
        end

        -- Table library aliases
        foreach = table.foreach
        foreachi = table.foreachi
        getn = table.getn or function(t) return #t end
        sort = table.sort
        table.wipe = wipe

        -- Bit operations (pure Lua 5.1 implementation)
        bit = {}

        local function tobits(n)
            local t = {}
            while n > 0 do
                t[#t + 1] = n % 2
                n = math.floor(n / 2)
            end
            return t
        end

        local function frombits(t)
            local n = 0
            for i = 1, #t do
                n = n + t[i] * (2 ^ (i - 1))
            end
            return n
        end

        function bit.band(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) == 1 and (tb[i] or 0) == 1) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bor(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) == 1 or (tb[i] or 0) == 1) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bxor(a, b)
            local ta, tb = tobits(a), tobits(b)
            local result = {}
            local len = math.max(#ta, #tb)
            for i = 1, len do
                result[i] = ((ta[i] or 0) ~= (tb[i] or 0)) and 1 or 0
            end
            return frombits(result)
        end

        function bit.bnot(a)
            -- 32-bit NOT
            return 4294967295 - a
        end

        function bit.lshift(a, n)
            return math.floor(a * (2 ^ n)) % 4294967296
        end

        function bit.rshift(a, n)
            return math.floor(a / (2 ^ n))
        end

        function bit.arshift(a, n)
            local r = bit.rshift(a, n)
            if a >= 2147483648 then
                r = r + (2 ^ 32 - 2 ^ (32 - n))
            end
            return r
        end

        function bit.mod(a, b)
            return a % b
        end

        -- Lua 5.2 compatibility: bit32 is an alias for bit with different names
        bit32 = {
            band = bit.band,
            bor = bit.bor,
            bxor = bit.bxor,
            bnot = bit.bnot,
            lshift = bit.lshift,
            rshift = bit.rshift,
            arshift = bit.arshift,
            -- bit32-specific functions
            extract = function(n, field, width)
                width = width or 1
                return bit.band(bit.rshift(n, field), (2 ^ width) - 1)
            end,
            replace = function(n, v, field, width)
                width = width or 1
                local mask = (2 ^ width) - 1
                return bit.bor(bit.band(n, bit.bnot(bit.lshift(mask, field))), bit.lshift(bit.band(v, mask), field))
            end,
            btest = function(...)
                return bit.band(...) ~= 0
            end,
        }

        -- Mixin system (WoW C++ intrinsics)
        function Mixin(object, ...)
            for i = 1, select("#", ...) do
                local mixin = select(i, ...)
                if mixin then
                    for k, v in pairs(mixin) do
                        object[k] = v
                    end
                end
            end
            return object
        end

        function CreateFromMixins(...)
            return Mixin({}, ...)
        end

        function CreateAndInitFromMixin(mixin, ...)
            local object = CreateFromMixins(mixin)
            if object.Init then
                object:Init(...)
            end
            return object
        end

        -- Security functions (always "secure" in simulation)
        function issecure()
            return true
        end

        function issecurevariable(table, variable)
            return true, "secure"
        end

        function forceinsecure()
            -- no-op in simulation
        end

        -- Debug functions
        function debugstack(start, count1, count2)
            start = start or 1
            count1 = count1 or 12
            count2 = count2 or 12

            local result = {}
            local level = start + 1  -- +1 to skip debugstack itself

            for i = 1, count1 do
                local info = debug.getinfo(level, "Sln")
                if not info then break end

                local source = info.source or "?"
                -- Convert @path to just path
                if source:sub(1, 1) == "@" then
                    source = source:sub(2)
                end

                local line = info.currentline or 0
                local name = info.name or ""

                if name ~= "" then
                    table.insert(result, source .. ":" .. line .. ": in function `" .. name .. "'")
                else
                    table.insert(result, source .. ":" .. line .. ": in main chunk")
                end

                level = level + 1
            end

            return table.concat(result, "\n")
        end

        function debuglocals(level)
            return ""
        end

        -- Time functions
        function GetTime()
            return os.clock()
        end

        function GetServerTime()
            return os.time()
        end

        -- SecondsFormatter class - formats seconds into time strings
        SecondsFormatter = {}
        SecondsFormatter.__index = SecondsFormatter

        -- Constants
        SecondsFormatter.Abbreviation = {
            None = 0,
            Truncate = 1,
            OneLetter = 2,
        }

        -- Interval descriptions (used by WoW for time formatting)
        SecondsFormatter.IntervalDescription = {
            { seconds = 86400, formatString = { "%d Day", "%d Days", "d" } },
            { seconds = 3600, formatString = { "%d Hour", "%d Hours", "h" } },
            { seconds = 60, formatString = { "%d Min", "%d Mins", "m" } },
            { seconds = 1, formatString = { "%d Sec", "%d Secs", "s" } },
        }

        function SecondsFormatter:Init(interval, abbreviation, roundUpLastUnit, convertToLower)
            self.interval = interval or 0
            self.abbreviation = abbreviation or SecondsFormatter.Abbreviation.None
            self.roundUpLastUnit = roundUpLastUnit or false
            self.convertToLower = convertToLower or false
        end

        function SecondsFormatter:SetDesiredUnitCount(count)
            self.desiredUnitCount = count
        end

        function SecondsFormatter:SetStripIntervalWhitespace(strip)
            self.stripIntervalWhitespace = strip
        end

        function SecondsFormatter:Format(seconds)
            if not seconds or seconds < 0 then
                return ""
            end
            local days = math.floor(seconds / 86400)
            local hours = math.floor((seconds % 86400) / 3600)
            local minutes = math.floor((seconds % 3600) / 60)
            local secs = math.floor(seconds % 60)

            if days > 0 then
                return string.format("%d d %02d h", days, hours)
            elseif hours > 0 then
                return string.format("%d h %02d m", hours, minutes)
            elseif minutes > 0 then
                return string.format("%d m %02d s", minutes, secs)
            else
                return string.format("%d s", secs)
            end
        end

        function CreateSecondsFormatter(interval, abbreviation, roundUpLastUnit, convertToLower)
            local formatter = setmetatable({}, SecondsFormatter)
            formatter:Init(interval, abbreviation, roundUpLastUnit, convertToLower)
            return formatter
        end

        -- SecondsFormatterMixin alias (some code uses this)
        SecondsFormatterMixin = SecondsFormatter

        function time()
            return os.time()
        end

        function date(fmt, t)
            return os.date(fmt, t)
        end

        function difftime(t2, t1)
            return os.difftime(t2, t1)
        end

        -- CopyTable - deep copy a table
        function CopyTable(settings, shallow)
            if type(settings) ~= "table" then
                return settings
            end
            local copy = {}
            for k, v in pairs(settings) do
                if type(v) == "table" and not shallow then
                    copy[k] = CopyTable(v, shallow)
                else
                    copy[k] = v
                end
            end
            return copy
        end

        -- MergeTable - merge source into destination
        function MergeTable(destination, source)
            for k, v in pairs(source) do
                destination[k] = v
            end
            return destination
        end

        -- ChatFrame message filter (store filters but don't actually filter in simulation)
        __chatFilters = {}
        function ChatFrame_AddMessageEventFilter(event, filter)
            __chatFilters[event] = __chatFilters[event] or {}
            table.insert(__chatFilters[event], filter)
        end

        function ChatFrame_RemoveMessageEventFilter(event, filter)
            if __chatFilters[event] then
                for i, f in ipairs(__chatFilters[event]) do
                    if f == filter then
                        table.remove(__chatFilters[event], i)
                        break
                    end
                end
            end
        end
    "##).exec()?;

    // C_Timer namespace
    let c_timer = lua.create_table()?;
    // C_Timer.After(seconds, callback) - simplified version
    let c_timer_after = lua.create_function(|_, (_seconds, callback): (f64, mlua::Function)| {
        // In a real implementation, this would schedule for later
        let _ = callback; // Would need an event loop to actually call this
        Ok(())
    })?;
    c_timer.set("After", c_timer_after)?;

    // C_Timer.NewTicker(seconds, callback, iterations) - creates a repeating timer
    let c_timer_new_ticker = lua.create_function(|lua, (_seconds, callback, _iterations): (f64, mlua::Function, Option<i32>)| {
        // Return a ticker object with Cancel method
        let _ = callback; // Would need an event loop to actually call this
        let ticker = lua.create_table()?;
        let cancel = lua.create_function(|_, ()| Ok(()))?;
        ticker.set("Cancel", cancel)?;
        Ok(ticker)
    })?;
    c_timer.set("NewTicker", c_timer_new_ticker)?;

    // C_Timer.NewTimer(seconds, callback) - creates a one-shot timer
    let c_timer_new_timer = lua.create_function(|lua, (_seconds, callback): (f64, mlua::Function)| {
        let _ = callback;
        let timer = lua.create_table()?;
        let cancel = lua.create_function(|_, ()| Ok(()))?;
        timer.set("Cancel", cancel)?;
        Ok(timer)
    })?;
    c_timer.set("NewTimer", c_timer_new_timer)?;

    globals.set("C_Timer", c_timer)?;

    // C_ChatInfo namespace
    let c_chat_info = lua.create_table()?;
    c_chat_info.set(
        "RegisterAddonMessagePrefix",
        lua.create_function(|_, _prefix: String| Ok(true))?,
    )?;
    c_chat_info.set(
        "IsAddonMessagePrefixRegistered",
        lua.create_function(|_, _prefix: String| Ok(false))?,
    )?;
    c_chat_info.set(
        "SendAddonMessage",
        lua.create_function(
            |_, (_prefix, _message, _channel, _target): (String, String, Option<String>, Option<String>)| {
                Ok(())
            },
        )?,
    )?;
    c_chat_info.set(
        "GetRegisteredAddonMessagePrefixes",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    globals.set("C_ChatInfo", c_chat_info)?;

    // Legacy global version
    let register_addon_message_prefix = lua.create_function(|_, _prefix: String| Ok(true))?;
    globals.set("RegisterAddonMessagePrefix", register_addon_message_prefix)?;

    // C_EventUtils namespace
    let c_event_utils = lua.create_table()?;
    c_event_utils.set(
        "IsEventValid",
        lua.create_function(|_, _event: String| {
            // In simulation, all events are valid
            Ok(true)
        })?,
    )?;
    globals.set("C_EventUtils", c_event_utils)?;

    // C_CVar namespace - console variables
    let c_cvar = lua.create_table()?;
    c_cvar.set(
        "GetCVar",
        lua.create_function(|lua, cvar: String| {
            // Return default values for common cvars
            let value = match cvar.as_str() {
                "nameplateShowEnemies" => "1",
                "nameplateShowFriends" => "0",
                "nameplateShowAll" => "1",
                _ => "",
            };
            Ok(Value::String(lua.create_string(value)?))
        })?,
    )?;
    c_cvar.set(
        "SetCVar",
        lua.create_function(|_, (_cvar, _value): (String, String)| Ok(()))?,
    )?;
    c_cvar.set(
        "GetCVarBool",
        lua.create_function(|_, cvar: String| {
            // Return default values for common cvars
            Ok(matches!(
                cvar.as_str(),
                "nameplateShowEnemies" | "nameplateShowAll"
            ))
        })?,
    )?;
    c_cvar.set(
        "RegisterCVar",
        lua.create_function(|_, (_cvar, _default): (String, Option<String>)| Ok(()))?,
    )?;
    globals.set("C_CVar", c_cvar)?;

    // C_SpellBook namespace - spell book functions
    let c_spell_book = lua.create_table()?;
    c_spell_book.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetNumSpellBookSkillLines",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_spell_book.set(
        "GetSpellBookSkillLineInfo",
        lua.create_function(|_, _tab: i32| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    c_spell_book.set(
        "HasPetSpells",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_spell_book.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    globals.set("C_SpellBook", c_spell_book)?;

    // C_Spell namespace - spell information
    let c_spell = lua.create_table()?;
    c_spell.set(
        "GetSpellInfo",
        lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?,
    )?;
    c_spell.set(
        "IsSpellPassive",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;
    c_spell.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;
    c_spell.set(
        "GetSchoolString",
        lua.create_function(|lua, school_mask: i32| {
            // WoW spell school bitmask to name
            let name = match school_mask {
                1 => "Physical",
                2 => "Holy",
                4 => "Fire",
                8 => "Nature",
                16 => "Frost",
                32 => "Shadow",
                64 => "Arcane",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    globals.set("C_Spell", c_spell)?;

    // C_Traits namespace - talent/loadout system (Dragonflight+)
    let c_traits = lua.create_table()?;
    c_traits.set(
        "GenerateImportString",
        lua.create_function(|_, _config_id: i32| {
            // Return a dummy talent string
            Ok("dummy_talent_string".to_string())
        })?,
    )?;
    c_traits.set(
        "GetConfigIDBySystemID",
        lua.create_function(|_, _system_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigIDByTreeID",
        lua.create_function(|_, _tree_id: i32| Ok(0))?,
    )?;
    c_traits.set(
        "GetConfigInfo",
        lua.create_function(|_, _config_id: i32| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetNodeInfo",
        lua.create_function(|_, (_config_id, _node_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetEntryInfo",
        lua.create_function(|_, (_config_id, _entry_id): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_traits.set(
        "GetDefinitionInfo",
        lua.create_function(|_, _def_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_Traits", c_traits)?;

    // C_ClassTalents namespace - class talent functions
    let c_class_talents = lua.create_table()?;
    c_class_talents.set(
        "GetActiveConfigID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_class_talents.set(
        "GetConfigIDsBySpecID",
        lua.create_function(|lua, _spec_id: i32| {
            // Return empty table
            lua.create_table()
        })?,
    )?;
    c_class_talents.set(
        "GetStarterBuildActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_class_talents.set(
        "GetHasStarterBuild",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    c_class_talents.set(
        "GetTraitTreeForSpec",
        lua.create_function(|_, _spec_id: i32| Ok(0))?,
    )?;
    c_class_talents.set(
        "UpdateLastSelectedSavedConfigID",
        lua.create_function(|_, (_spec_id, _config_id): (i32, i32)| Ok(()))?,
    )?;
    globals.set("C_ClassTalents", c_class_talents)?;

    // Legacy global spell functions
    globals.set(
        "GetSpellInfo",
        lua.create_function(|_, _spell_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetSpellBookItemName",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetNumSpellTabs",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    globals.set(
        "GetSpellTabInfo",
        lua.create_function(|_, _tab: i32| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetSpellBookItemInfo",
        lua.create_function(|_, (_index, _book): (i32, Option<String>)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "IsPassiveSpell",
        lua.create_function(|_, _spell_id: i32| Ok(false))?,
    )?;
    globals.set(
        "HasPetSpells",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set(
        "GetOverrideSpell",
        lua.create_function(|_, spell_id: i32| Ok(spell_id))?,
    )?;

    // C_Item namespace - item information
    let c_item = lua.create_table()?;
    c_item.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| {
            // Return nil - no item info in simulation
            Ok(Value::Nil)
        })?,
    )?;
    c_item.set(
        "GetItemInfoInstant",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;
    c_item.set(
        "GetItemIconByID",
        lua.create_function(|_, _item_id: i32| Ok(Value::Nil))?,
    )?;
    c_item.set(
        "GetItemSubClassInfo",
        lua.create_function(|lua, (class_id, subclass_id): (i32, i32)| {
            // Return item subclass name based on class/subclass IDs
            let name = match (class_id, subclass_id) {
                // Weapons (class 2)
                (2, 0) => "One-Handed Axes",
                (2, 1) => "Two-Handed Axes",
                (2, 2) => "Bows",
                (2, 3) => "Guns",
                (2, 4) => "One-Handed Maces",
                (2, 5) => "Two-Handed Maces",
                (2, 6) => "Polearms",
                (2, 7) => "One-Handed Swords",
                (2, 8) => "Two-Handed Swords",
                (2, 9) => "Warglaives",
                (2, 10) => "Staves",
                (2, 13) => "Fist Weapons",
                (2, 14) => "Miscellaneous",
                (2, 15) => "Daggers",
                (2, 16) => "Thrown",
                (2, 18) => "Crossbows",
                (2, 19) => "Wands",
                (2, 20) => "Fishing Poles",
                // Armor (class 4)
                (4, 0) => "Miscellaneous",
                (4, 1) => "Cloth",
                (4, 2) => "Leather",
                (4, 3) => "Mail",
                (4, 4) => "Plate",
                (4, 6) => "Shield",
                _ => "Unknown",
            };
            Ok(Value::String(lua.create_string(name)?))
        })?,
    )?;
    globals.set("C_Item", c_item)?;

    // Legacy global GetItemInfo
    globals.set(
        "GetItemInfo",
        lua.create_function(|_, _item_id: Value| Ok(Value::Nil))?,
    )?;

    // C_Container namespace - bag/container functions
    let c_container = lua.create_table()?;
    c_container.set(
        "GetContainerNumSlots",
        lua.create_function(|_, bag: i32| {
            // Return bag slot counts (0 = backpack has 16 slots, bags 1-4 vary)
            Ok(if bag == 0 { 16 } else { 0 })
        })?,
    )?;
    c_container.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    c_container.set(
        "GetContainerItemInfo",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set("C_Container", c_container)?;

    // Legacy global container functions
    let get_container_num_slots = lua.create_function(|_, bag: i32| {
        Ok(if bag == 0 { 16 } else { 0 })
    })?;
    globals.set("GetContainerNumSlots", get_container_num_slots)?;
    globals.set(
        "GetContainerItemID",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;
    globals.set(
        "GetContainerItemLink",
        lua.create_function(|_, (_bag, _slot): (i32, i32)| Ok(Value::Nil))?,
    )?;

    // Inventory slot functions
    globals.set(
        "GetInventorySlotInfo",
        lua.create_function(|_, slot_name: String| {
            // Return slot ID for known slot names
            let slot_id = match slot_name.as_str() {
                "HeadSlot" => 1,
                "NeckSlot" => 2,
                "ShoulderSlot" => 3,
                "BackSlot" => 15,
                "ChestSlot" => 5,
                "ShirtSlot" => 4,
                "TabardSlot" => 19,
                "WristSlot" => 9,
                "HandsSlot" => 10,
                "WaistSlot" => 6,
                "LegsSlot" => 7,
                "FeetSlot" => 8,
                "Finger0Slot" => 11,
                "Finger1Slot" => 12,
                "Trinket0Slot" => 13,
                "Trinket1Slot" => 14,
                "MainHandSlot" => 16,
                "SecondaryHandSlot" => 17,
                "RangedSlot" => 18,
                "AmmoSlot" => 0,
                _ => 0,
            };
            Ok(slot_id)
        })?,
    )?;

    // Pet action functions
    globals.set(
        "GetPetActionInfo",
        lua.create_function(|_, _slot: i32| {
            // Return: name, texture, isToken, isActive, autoCastAllowed, autoCastEnabled, spellID, checksRange, inRange
            // Return nil for no action in slot
            Ok(Value::Nil)
        })?,
    )?;
    globals.set(
        "GetPetActionCooldown",
        lua.create_function(|_, _slot: i32| {
            // Return: start, duration, enable
            Ok((0.0, 0.0, 0))
        })?,
    )?;

    // Legacy global CVar functions
    let get_cvar = lua.create_function(|lua, cvar: String| {
        let value = match cvar.as_str() {
            "nameplateShowEnemies" => "1",
            "nameplateShowFriends" => "0",
            _ => "",
        };
        Ok(Value::String(lua.create_string(value)?))
    })?;
    globals.set("GetCVar", get_cvar)?;

    let set_cvar = lua.create_function(|_, (_cvar, _value): (String, String)| Ok(()))?;
    globals.set("SetCVar", set_cvar)?;

    // C_AddOns namespace - addon management
    let c_addons = lua.create_table()?;
    c_addons.set(
        "GetAddOnMetadata",
        lua.create_function(|lua, (addon, field): (String, String)| {
            // Return stub metadata - WeakAuras checks Version and X-Flavor
            let value = match field.as_str() {
                "Version" => "@project-version@",
                "X-Flavor" => "Mainline",
                "Title" => addon.as_str(),
                "Notes" => "",
                "Author" => "",
                _ => "",
            };
            if value.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(value)?))
            }
        })?,
    )?;
    c_addons.set(
        "EnableAddOn",
        lua.create_function(|_, _addon: String| Ok(()))?,
    )?;
    c_addons.set(
        "DisableAddOn",
        lua.create_function(|_, _addon: String| Ok(()))?,
    )?;
    c_addons.set(
        "GetNumAddOns",
        lua.create_function(|_, ()| Ok(1))?,
    )?;
    c_addons.set(
        "GetAddOnInfo",
        lua.create_function(|lua, index: i32| {
            // Return: name, title, notes, loadable, reason, security, newVersion
            if index == 1 {
                Ok((
                    Value::String(lua.create_string("TestAddon")?),
                    Value::String(lua.create_string("Test Addon")?),
                    Value::String(lua.create_string("")?),
                    Value::Boolean(true),
                    Value::Nil,
                    Value::String(lua.create_string("SECURE")?),
                    Value::Boolean(false),
                ))
            } else {
                Ok((
                    Value::Nil,
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                    Value::Nil,
                    Value::Nil,
                    Value::Boolean(false),
                ))
            }
        })?,
    )?;
    c_addons.set(
        "IsAddOnLoaded",
        lua.create_function(|_, addon: String| {
            // Return true only for addons we actually load in the simulator
            // Return false for optional addons like CustomNames that aren't loaded
            Ok(matches!(
                addon.as_str(),
                "WeakAuras" | "Ace3" | "LibStub" | "CallbackHandler-1.0"
            ))
        })?,
    )?;
    c_addons.set(
        "IsAddOnLoadable",
        lua.create_function(|_, _addon: String| Ok(true))?,
    )?;
    c_addons.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;
    c_addons.set(
        "DoesAddOnExist",
        lua.create_function(|_, _addon: String| Ok(true))?,
    )?;
    globals.set("C_AddOns", c_addons)?;

    // C_Reputation namespace - faction reputation system
    let c_reputation = lua.create_table()?;
    c_reputation.set(
        "GetFactionDataByID",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "IsFactionParagon",
        lua.create_function(|_, _faction_id: i32| Ok(false))?,
    )?;
    c_reputation.set(
        "GetFactionParagonInfo",
        lua.create_function(|_, _faction_id: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetNumFactions",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_reputation.set(
        "GetFactionInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "GetWatchedFactionData",
        lua.create_function(|_, ()| Ok(Value::Nil))?,
    )?;
    c_reputation.set(
        "SetWatchedFactionByID",
        lua.create_function(|_, _faction_id: i32| Ok(()))?,
    )?;
    globals.set("C_Reputation", c_reputation)?;

    // C_Texture namespace - texture handling
    let c_texture = lua.create_table()?;
    c_texture.set(
        "GetAtlasInfo",
        lua.create_function(|lua, _atlas_name: String| {
            // Return atlas info table (or nil if not found)
            // Fields: width, height, leftTexCoord, rightTexCoord, topTexCoord, bottomTexCoord, file
            let info = lua.create_table()?;
            info.set("width", 64)?;
            info.set("height", 64)?;
            info.set("leftTexCoord", 0.0)?;
            info.set("rightTexCoord", 1.0)?;
            info.set("topTexCoord", 0.0)?;
            info.set("bottomTexCoord", 1.0)?;
            info.set("file", "")?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_texture.set(
        "GetFilenameFromFileDataID",
        lua.create_function(|_, _file_data_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_Texture", c_texture)?;

    // C_CreatureInfo namespace - NPC/creature information
    let c_creature_info = lua.create_table()?;
    c_creature_info.set(
        "GetClassInfo",
        lua.create_function(|lua, class_id: i32| {
            // Return class info table
            let info = lua.create_table()?;
            let class_name = match class_id {
                1 => "WARRIOR",
                2 => "PALADIN",
                3 => "HUNTER",
                4 => "ROGUE",
                5 => "PRIEST",
                6 => "DEATHKNIGHT",
                7 => "SHAMAN",
                8 => "MAGE",
                9 => "WARLOCK",
                10 => "MONK",
                11 => "DRUID",
                12 => "DEMONHUNTER",
                13 => "EVOKER",
                _ => "UNKNOWN",
            };
            info.set("className", class_name)?;
            info.set("classFile", class_name)?;
            info.set("classID", class_id)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetRaceInfo",
        lua.create_function(|lua, race_id: i32| {
            let info = lua.create_table()?;
            // WoW race data: (name, clientFileString)
            let (race_name, client_file) = match race_id {
                1 => ("Human", "Human"),
                2 => ("Orc", "Orc"),
                3 => ("Dwarf", "Dwarf"),
                4 => ("Night Elf", "NightElf"),
                5 => ("Undead", "Scourge"),
                6 => ("Tauren", "Tauren"),
                7 => ("Gnome", "Gnome"),
                8 => ("Troll", "Troll"),
                9 => ("Goblin", "Goblin"),
                10 => ("Blood Elf", "BloodElf"),
                11 => ("Draenei", "Draenei"),
                22 => ("Worgen", "Worgen"),
                24 => ("Pandaren", "Pandaren"),
                25 => ("Pandaren", "Pandaren"),
                26 => ("Pandaren", "Pandaren"),
                27 => ("Nightborne", "Nightborne"),
                28 => ("Highmountain Tauren", "HighmountainTauren"),
                29 => ("Void Elf", "VoidElf"),
                30 => ("Lightforged Draenei", "LightforgedDraenei"),
                31 => ("Zandalari Troll", "ZandalariTroll"),
                32 => ("Kul Tiran", "KulTiran"),
                34 => ("Dark Iron Dwarf", "DarkIronDwarf"),
                35 => ("Vulpera", "Vulpera"),
                36 => ("Mag'har Orc", "MagharOrc"),
                37 => ("Mechagnome", "Mechagnome"),
                52 | 70 => ("Dracthyr", "Dracthyr"),
                84 | 85 => ("Earthen", "Earthen"),
                _ => ("Unknown", "Unknown"),
            };
            info.set("raceName", race_name)?;
            info.set("raceID", race_id)?;
            info.set("clientFileString", client_file)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeIDs",
        lua.create_function(|lua, ()| {
            // WoW creature types: Beast, Dragonkin, Demon, Elemental, Giant, Undead, Humanoid, Critter, Mechanical, etc.
            let ids = lua.create_table()?;
            for (i, id) in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().enumerate() {
                ids.set(i + 1, *id)?;
            }
            Ok(ids)
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureTypeInfo",
        lua.create_function(|lua, creature_type_id: i32| {
            let info = lua.create_table()?;
            let name = match creature_type_id {
                1 => "Beast",
                2 => "Dragonkin",
                3 => "Demon",
                4 => "Elemental",
                5 => "Giant",
                6 => "Undead",
                7 => "Humanoid",
                8 => "Critter",
                9 => "Mechanical",
                10 => "Not specified",
                _ => "Unknown",
            };
            info.set("name", name)?;
            info.set("creatureTypeID", creature_type_id)?;
            Ok(Value::Table(info))
        })?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyIDs",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_creature_info.set(
        "GetCreatureFamilyInfo",
        lua.create_function(|_, _family_id: i32| Ok(Value::Nil))?,
    )?;
    globals.set("C_CreatureInfo", c_creature_info)?;

    // C_Covenants namespace - Shadowlands covenant system
    let c_covenants = lua.create_table()?;
    c_covenants.set(
        "GetCovenantData",
        lua.create_function(|lua, covenant_id: i32| {
            let data = lua.create_table()?;
            let name = match covenant_id {
                1 => "Kyrian",
                2 => "Venthyr",
                3 => "Night Fae",
                4 => "Necrolord",
                _ => "None",
            };
            data.set("ID", covenant_id)?;
            data.set("name", name)?;
            data.set("textureKit", "")?;
            Ok(Value::Table(data))
        })?,
    )?;
    c_covenants.set(
        "GetActiveCovenantID",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_covenants.set(
        "GetCovenantIDs",
        lua.create_function(|lua, ()| {
            let ids = lua.create_table()?;
            ids.set(1, 1)?;
            ids.set(2, 2)?;
            ids.set(3, 3)?;
            ids.set(4, 4)?;
            Ok(ids)
        })?,
    )?;
    globals.set("C_Covenants", c_covenants)?;

    // C_CurrencyInfo namespace - currency information
    let c_currency_info = lua.create_table()?;
    c_currency_info.set(
        "GetCurrencyInfo",
        lua.create_function(|_, _currency_id: i32| Ok(Value::Nil))?,
    )?;
    c_currency_info.set(
        "GetCurrencyInfoFromLink",
        lua.create_function(|_, _link: String| Ok(Value::Nil))?,
    )?;
    c_currency_info.set(
        "GetCurrencyListSize",
        lua.create_function(|_, ()| Ok(0))?,
    )?;
    c_currency_info.set(
        "GetCurrencyListInfo",
        lua.create_function(|_, _index: i32| Ok(Value::Nil))?,
    )?;
    c_currency_info.set(
        "GetWarResourcesCurrencyID",
        lua.create_function(|_, ()| Ok(1560))?, // War Resources currency ID
    )?;
    globals.set("C_CurrencyInfo", c_currency_info)?;

    // Legacy addon functions
    globals.set(
        "GetAddOnMetadata",
        lua.create_function(|lua, (addon, field): (String, String)| {
            let value = match field.as_str() {
                "Version" => "@project-version@",
                "X-Flavor" => "Mainline",
                "Title" => addon.as_str(),
                _ => "",
            };
            if value.is_empty() {
                Ok(Value::Nil)
            } else {
                Ok(Value::String(lua.create_string(value)?))
            }
        })?,
    )?;
    globals.set(
        "GetNumAddOns",
        lua.create_function(|_, ()| Ok(1))?,
    )?;
    globals.set(
        "IsAddOnLoaded",
        lua.create_function(|_, addon: String| {
            // Return true only for addons we actually load
            Ok(matches!(
                addon.as_str(),
                "WeakAuras" | "Ace3" | "LibStub" | "CallbackHandler-1.0"
            ))
        })?,
    )?;
    globals.set(
        "LoadAddOn",
        lua.create_function(|_, _addon: String| Ok((true, Value::Nil)))?,
    )?;

    // CreateColor(r, g, b, a) - creates a color object
    let create_color = lua.create_function(|lua, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("a", a.unwrap_or(1.0))?;

        // Add color methods
        let get_rgb = lua.create_function(move |_, ()| {
            Ok((r, g, b))
        })?;
        color.set("GetRGB", get_rgb)?;

        let get_rgba = lua.create_function(move |_, ()| {
            Ok((r, g, b, a.unwrap_or(1.0)))
        })?;
        color.set("GetRGBA", get_rgba)?;

        let generate_hex = lua.create_function(move |lua, ()| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            Ok(Value::String(lua.create_string(&hex)?))
        })?;
        color.set("GenerateHexColor", generate_hex)?;

        Ok(color)
    })?;
    globals.set("CreateColor", create_color)?;

    // WrapTextInColorCode(text, colorStr) - wrap text in color escape codes
    globals.set(
        "WrapTextInColorCode",
        lua.create_function(|lua, (text, color_str): (String, String)| {
            let wrapped = format!("|c{}{}|r", color_str, text);
            Ok(Value::String(lua.create_string(&wrapped)?))
        })?,
    )?;

    // Faction color globals (now that CreateColor exists)
    lua.load(r#"
        PLAYER_FACTION_COLOR_HORDE = CreateColor(1.0, 0.1, 0.1)
        PLAYER_FACTION_COLOR_ALLIANCE = CreateColor(0.1, 0.1, 1.0)
        FACTION_HORDE = "Horde"
        FACTION_ALLIANCE = "Alliance"
    "#).exec()?;

    // Error message strings used by addons
    globals.set("ERR_CHAT_PLAYER_NOT_FOUND_S", "%s is not online")?;
    globals.set("ERR_NOT_IN_COMBAT", "You can't do that while in combat")?;
    globals.set("ERR_GENERIC_NO_TARGET", "You have no target")?;

    // Game constants
    globals.set("NUM_PET_ACTION_SLOTS", 10)?;
    globals.set("NUM_ACTIONBAR_BUTTONS", 12)?;
    globals.set("NUM_BAG_SLOTS", 5)?;
    globals.set("MAX_SKILLLINE_TABS", 8)?;
    globals.set("MAX_PLAYER_LEVEL", 80)?;
    globals.set("MAX_NUM_TALENTS", 20)?;
    globals.set("BOOKTYPE_SPELL", "spell")?;
    globals.set("BOOKTYPE_PET", "pet")?;

    // Raid target marker names
    globals.set("RAID_TARGET_1", "Star")?;
    globals.set("RAID_TARGET_2", "Circle")?;
    globals.set("RAID_TARGET_3", "Diamond")?;
    globals.set("RAID_TARGET_4", "Triangle")?;
    globals.set("RAID_TARGET_5", "Moon")?;
    globals.set("RAID_TARGET_6", "Square")?;
    globals.set("RAID_TARGET_7", "Cross")?;
    globals.set("RAID_TARGET_8", "Skull")?;

    // UI strings
    globals.set("SPECIALIZATION", "Specialization")?;
    globals.set("TALENT", "Talent")?;
    globals.set("NONE", "None")?;
    globals.set("UNKNOWN", "Unknown")?;
    globals.set("YES", "Yes")?;
    globals.set("NO", "No")?;
    globals.set("OKAY", "Okay")?;
    globals.set("CANCEL", "Cancel")?;
    globals.set("ACCEPT", "Accept")?;
    globals.set("DECLINE", "Decline")?;
    globals.set("ENABLE", "Enable")?;
    globals.set("DISABLE", "Disable")?;
    globals.set("BINDING_HEADER_RAID_TARGET", "Raid Target")?;
    globals.set("BINDING_HEADER_ACTIONBAR", "Action Bar")?;
    globals.set("BINDING_HEADER_MULTIACTIONBAR", "Multi-Action Bar")?;
    globals.set("BINDING_HEADER_MOVEMENT", "Movement")?;
    globals.set("BINDING_HEADER_CHAT", "Chat")?;
    globals.set("BINDING_HEADER_TARGETING", "Targeting")?;
    globals.set("BINDING_HEADER_INTERFACE", "Interface")?;
    globals.set("BINDING_HEADER_MISC", "Miscellaneous")?;

    // Role strings
    globals.set("TANK", "Tank")?;
    globals.set("HEALER", "Healer")?;
    globals.set("DAMAGER", "Damage")?;
    globals.set("COMBATLOG_FILTER_MELEE", "Melee")?;
    globals.set("COMBATLOG_FILTER_RANGED", "Ranged")?;
    globals.set("RANGED_ABILITY", "Ranged")?;
    globals.set("MELEE", "Melee")?;

    // Role icon markup strings
    globals.set("INLINE_TANK_ICON", "|A:groupfinder-icon-role-large-tank:16:16:0:0|a")?;
    globals.set("INLINE_HEALER_ICON", "|A:groupfinder-icon-role-large-healer:16:16:0:0|a")?;
    globals.set("INLINE_DAMAGER_ICON", "|A:groupfinder-icon-role-large-dps:16:16:0:0|a")?;

    // RAID_CLASS_COLORS - color table for each class
    lua.load(
        r##"
        RAID_CLASS_COLORS = {
            ["WARRIOR"] = { r = 0.78, g = 0.61, b = 0.43, colorStr = "ffc79c6e" },
            ["PALADIN"] = { r = 0.96, g = 0.55, b = 0.73, colorStr = "fff58cba" },
            ["HUNTER"] = { r = 0.67, g = 0.83, b = 0.45, colorStr = "ffabd473" },
            ["ROGUE"] = { r = 1.00, g = 0.96, b = 0.41, colorStr = "fffff569" },
            ["PRIEST"] = { r = 1.00, g = 1.00, b = 1.00, colorStr = "ffffffff" },
            ["DEATHKNIGHT"] = { r = 0.77, g = 0.12, b = 0.23, colorStr = "ffc41f3b" },
            ["SHAMAN"] = { r = 0.00, g = 0.44, b = 0.87, colorStr = "ff0070de" },
            ["MAGE"] = { r = 0.41, g = 0.80, b = 0.94, colorStr = "ff69ccf0" },
            ["WARLOCK"] = { r = 0.58, g = 0.51, b = 0.79, colorStr = "ff9482c9" },
            ["MONK"] = { r = 0.00, g = 1.00, b = 0.59, colorStr = "ff00ff96" },
            ["DRUID"] = { r = 1.00, g = 0.49, b = 0.04, colorStr = "ffff7d0a" },
            ["DEMONHUNTER"] = { r = 0.64, g = 0.19, b = 0.79, colorStr = "ffa330c9" },
            ["EVOKER"] = { r = 0.20, g = 0.58, b = 0.50, colorStr = "ff33937f" },
        }
        -- Add colorStr getter method to each color
        for class, color in pairs(RAID_CLASS_COLORS) do
            setmetatable(color, {
                __index = {
                    GenerateHexColor = function(self) return self.colorStr end,
                    GenerateHexColorMarkup = function(self) return "|c" .. self.colorStr end,
                    WrapTextInColorCode = function(self, text) return "|c" .. self.colorStr .. text .. "|r" end,
                }
            })
        end
    "##,
    )
    .exec()?;

    // C_SpecializationInfo namespace
    let c_spec_info = lua.create_table()?;
    c_spec_info.set(
        "GetSpellsDisplay",
        lua.create_function(|lua, _spec_id: i32| lua.create_table())?,
    )?;
    c_spec_info.set(
        "GetInspectSelectedSpecialization",
        lua.create_function(|_, _unit: Option<String>| Ok(0))?,
    )?;
    c_spec_info.set(
        "CanPlayerUseTalentSpecUI",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    c_spec_info.set(
        "IsInitialized",
        lua.create_function(|_, ()| Ok(true))?,
    )?;
    globals.set("C_SpecializationInfo", c_spec_info)?;

    // C_ChallengeMode namespace - Mythic+ dungeons
    let c_challenge_mode = lua.create_table()?;
    c_challenge_mode.set(
        "GetMapUIInfo",
        lua.create_function(|lua, _map_id: i32| {
            // Return: name, id, timeLimit, texture, backgroundTexture
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("Unknown Dungeon")?),
                Value::Integer(0),
                Value::Integer(0),
                Value::Nil,
                Value::Nil,
            ]))
        })?,
    )?;
    c_challenge_mode.set(
        "GetMapTable",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    c_challenge_mode.set(
        "GetActiveKeystoneInfo",
        lua.create_function(|_, ()| {
            // Return: activeKeystoneLevel, activeAffixIDs, wasActiveKeystoneCharged
            Ok((0, Value::Nil, false))
        })?,
    )?;
    c_challenge_mode.set(
        "GetAffixInfo",
        lua.create_function(|lua, _affix_id: i32| {
            // Return: name, description, filedataid
            Ok((
                Value::String(lua.create_string("Unknown Affix")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0),
            ))
        })?,
    )?;
    c_challenge_mode.set(
        "IsChallengeModeActive",
        lua.create_function(|_, ()| Ok(false))?,
    )?;
    globals.set("C_ChallengeMode", c_challenge_mode)?;

    // GetSpecializationInfoByID(specID) - Get spec info
    globals.set(
        "GetSpecializationInfoByID",
        lua.create_function(|lua, spec_id: i32| {
            // Return: specID, specName, description, icon, role, isRecommended, isAllowed
            // Stub - return some default values
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(spec_id as i64),
                Value::String(lua.create_string("Unknown Spec")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0), // icon
                Value::String(lua.create_string("DAMAGER")?),
                Value::Boolean(false),
                Value::Boolean(true),
            ]))
        })?,
    )?;

    // GetSpecialization() - Get current player spec index
    globals.set(
        "GetSpecialization",
        lua.create_function(|_, ()| Ok(1))?,
    )?;

    // GetNumSpecializations() - Get number of specs for player class
    globals.set(
        "GetNumSpecializations",
        lua.create_function(|_, ()| Ok(3))?,
    )?;

    // GetSpecializationInfo(specIndex) - Get spec info by index
    globals.set(
        "GetSpecializationInfo",
        lua.create_function(|lua, _spec_index: i32| {
            Ok(mlua::MultiValue::from_vec(vec![
                Value::Integer(0), // specID
                Value::String(lua.create_string("Unknown")?),
                Value::String(lua.create_string("")?),
                Value::Integer(0), // icon
                Value::String(lua.create_string("DAMAGER")?),
                Value::Boolean(false),
                Value::Boolean(true),
            ]))
        })?,
    )?;

    Ok(())
}

/// Userdata handle to a frame (passed to Lua).
#[derive(Clone)]
pub struct FrameHandle {
    pub id: u64,
    pub state: Rc<RefCell<SimState>>,
}

impl UserData for FrameHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Support custom field access via __index/__newindex
        // This allows addons to do: frame.customField = value

        methods.add_meta_function(MetaMethod::Index, |lua: &Lua, (ud, key): (mlua::AnyUserData, String)| {
            // Try to get from the custom fields table
            let frame_id: u64 = ud.borrow::<FrameHandle>()?.id;
            let fields_table: Option<mlua::Table> = lua.globals().get("__frame_fields").ok();

            if let Some(table) = fields_table {
                let frame_fields: Option<mlua::Table> = table.get::<mlua::Table>(frame_id).ok();
                if let Some(fields) = frame_fields {
                    let value: Value = fields.get::<Value>(key.as_str()).unwrap_or(Value::Nil);
                    if value != Value::Nil {
                        return Ok(value);
                    }
                }
            }

            // Not found in custom fields, return nil (methods are handled separately by mlua)
            Ok(Value::Nil)
        });

        methods.add_meta_function(MetaMethod::NewIndex, |lua: &Lua, (ud, key, value): (mlua::AnyUserData, String, Value)| {
            let frame_id: u64 = ud.borrow::<FrameHandle>()?.id;

            // Get or create the fields table
            let fields_table: mlua::Table = lua.globals().get::<mlua::Table>("__frame_fields").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("__frame_fields", t.clone()).unwrap();
                t
            });

            // Get or create the frame's field table
            let frame_fields: mlua::Table = fields_table.get::<mlua::Table>(frame_id).unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                fields_table.set(frame_id, t.clone()).unwrap();
                t
            });

            frame_fields.set(key, value)?;
            Ok(())
        });
        // GetName()
        methods.add_method("GetName", |_, this, ()| {
            let state = this.state.borrow();
            let name = state
                .widgets
                .get(this.id)
                .and_then(|f| f.name.clone())
                .unwrap_or_default();
            Ok(name)
        });

        // GetWidth()
        methods.add_method("GetWidth", |_, this, ()| {
            let state = this.state.borrow();
            let width = state.widgets.get(this.id).map(|f| f.width).unwrap_or(0.0);
            Ok(width)
        });

        // GetHeight()
        methods.add_method("GetHeight", |_, this, ()| {
            let state = this.state.borrow();
            let height = state.widgets.get(this.id).map(|f| f.height).unwrap_or(0.0);
            Ok(height)
        });

        // SetSize(width, height)
        methods.add_method("SetSize", |_, this, (width, height): (f32, f32)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_size(width, height);
            }
            Ok(())
        });

        // SetWidth(width)
        methods.add_method("SetWidth", |_, this, width: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.width = width;
            }
            Ok(())
        });

        // SetHeight(height)
        methods.add_method("SetHeight", |_, this, height: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.height = height;
            }
            Ok(())
        });

        // SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
        methods.add_method("SetPoint", |_, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let point_str = args
                .first()
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "CENTER".to_string());

            let point =
                crate::widget::AnchorPoint::from_str(&point_str).unwrap_or_default();

            // Helper to extract numeric value from Value (handles both Number and Integer)
            fn get_number(v: &Value) -> Option<f32> {
                match v {
                    Value::Number(n) => Some(*n as f32),
                    Value::Integer(n) => Some(*n as f32),
                    _ => None,
                }
            }

            // Parse the variable arguments
            let (relative_to, relative_point, x_ofs, y_ofs) = match args.len() {
                1 => (None, point, 0.0, 0.0),
                2 | 3 => {
                    // SetPoint("CENTER", x, y) or SetPoint("CENTER", relativeTo)
                    let x = args.get(1).and_then(get_number);
                    let y = args.get(2).and_then(get_number);
                    if let (Some(x), Some(y)) = (x, y) {
                        (None, point, x, y)
                    } else {
                        (None, point, 0.0, 0.0)
                    }
                }
                _ => {
                    // Full form: SetPoint(point, relativeTo, relativePoint, x, y)
                    let rel_point_str = args.get(2).and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    });
                    let rel_point = rel_point_str
                        .and_then(|s| crate::widget::AnchorPoint::from_str(&s))
                        .unwrap_or(point);
                    let x = args.get(3).and_then(get_number).unwrap_or(0.0);
                    let y = args.get(4).and_then(get_number).unwrap_or(0.0);
                    (None, rel_point, x, y)
                }
            };

            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
            }
            Ok(())
        });

        // ClearAllPoints()
        methods.add_method("ClearAllPoints", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.clear_all_points();
            }
            Ok(())
        });

        // Show()
        methods.add_method("Show", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = true;
            }
            Ok(())
        });

        // Hide()
        methods.add_method("Hide", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = false;
            }
            Ok(())
        });

        // IsVisible() / IsShown()
        methods.add_method("IsVisible", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        methods.add_method("IsShown", |_, this, ()| {
            let state = this.state.borrow();
            let visible = state.widgets.get(this.id).map(|f| f.visible).unwrap_or(false);
            Ok(visible)
        });

        // RegisterEvent(event)
        methods.add_method("RegisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.register_event(&event);
            }
            Ok(())
        });

        // RegisterUnitEvent(event, unit1, unit2, ...) - register for unit-specific events
        methods.add_method(
            "RegisterUnitEvent",
            |_, this, (event, _units): (String, mlua::Variadic<String>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.register_event(&event);
                }
                Ok(())
            },
        );

        // UnregisterEvent(event)
        methods.add_method("UnregisterEvent", |_, this, event: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.unregister_event(&event);
            }
            Ok(())
        });

        // UnregisterAllEvents()
        methods.add_method("UnregisterAllEvents", |_, this, ()| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.registered_events.clear();
            }
            Ok(())
        });

        // SetScript(handler, func)
        methods.add_method("SetScript", |lua, this, (handler, func): (String, Value)| {
            let handler_type = crate::event::ScriptHandler::from_str(&handler);

            if let (Some(h), Value::Function(f)) = (handler_type, func) {
                // Store function in a global Lua table for later retrieval
                let scripts_table: mlua::Table =
                    lua.globals().get("__scripts").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__scripts", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_{}", this.id, handler);
                scripts_table.set(frame_key.as_str(), f)?;

                // Mark that this widget has this handler
                let mut state = this.state.borrow_mut();
                state.scripts.set(this.id, h, 1); // Just mark it exists
            }
            Ok(())
        });

        // GetScript(handler)
        methods.add_method("GetScript", |lua, this, handler: String| {
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_{}", this.id, handler);
                let func: Value = table.get(frame_key.as_str()).unwrap_or(Value::Nil);
                Ok(func)
            } else {
                Ok(Value::Nil)
            }
        });

        // HookScript(handler, func) - Hook into existing script handler
        methods.add_method("HookScript", |lua, this, (handler, func): (String, Value)| {
            if let Value::Function(f) = func {
                // Store hook in a global table
                let hooks_table: mlua::Table =
                    lua.globals().get("__script_hooks").unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        lua.globals().set("__script_hooks", t.clone()).unwrap();
                        t
                    });

                let frame_key = format!("{}_{}", this.id, handler);
                // Get existing hooks array or create new
                let hooks_array: mlua::Table = hooks_table
                    .get::<mlua::Table>(frame_key.as_str())
                    .unwrap_or_else(|_| {
                        let t = lua.create_table().unwrap();
                        hooks_table.set(frame_key.as_str(), t.clone()).unwrap();
                        t
                    });
                // Append the new hook
                let len = hooks_array.len().unwrap_or(0);
                hooks_array.set(len + 1, f)?;
            }
            Ok(())
        });

        // GetParent()
        methods.add_method("GetParent", |lua, this, ()| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(parent_id) = frame.parent_id {
                    let handle = FrameHandle {
                        id: parent_id,
                        state: Rc::clone(&this.state),
                    };
                    return Ok(Value::UserData(lua.create_userdata(handle)?));
                }
            }
            Ok(Value::Nil)
        });

        // GetObjectType()
        methods.add_method("GetObjectType", |_, this, ()| {
            let state = this.state.borrow();
            let obj_type = state
                .widgets
                .get(this.id)
                .map(|f| f.widget_type.as_str())
                .unwrap_or("Frame");
            Ok(obj_type.to_string())
        });

        // SetAlpha(alpha)
        methods.add_method("SetAlpha", |_, this, alpha: f32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.alpha = alpha.clamp(0.0, 1.0);
            }
            Ok(())
        });

        // GetAlpha()
        methods.add_method("GetAlpha", |_, this, ()| {
            let state = this.state.borrow();
            let alpha = state.widgets.get(this.id).map(|f| f.alpha).unwrap_or(1.0);
            Ok(alpha)
        });

        // SetFrameStrata(strata)
        methods.add_method("SetFrameStrata", |_, this, strata: String| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                if let Some(s) = crate::widget::FrameStrata::from_str(&strata) {
                    frame.frame_strata = s;
                }
            }
            Ok(())
        });

        // GetFrameStrata()
        methods.add_method("GetFrameStrata", |_, this, ()| {
            let state = this.state.borrow();
            let strata = state
                .widgets
                .get(this.id)
                .map(|f| f.frame_strata.as_str())
                .unwrap_or("MEDIUM");
            Ok(strata.to_string())
        });

        // SetFrameLevel(level)
        methods.add_method("SetFrameLevel", |_, this, level: i32| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.frame_level = level;
            }
            Ok(())
        });

        // GetFrameLevel()
        methods.add_method("GetFrameLevel", |_, this, ()| {
            let state = this.state.borrow();
            let level = state.widgets.get(this.id).map(|f| f.frame_level).unwrap_or(0);
            Ok(level)
        });

        // SetFixedFrameStrata(fixed) - Controls if strata is fixed
        methods.add_method("SetFixedFrameStrata", |_, _this, _fixed: bool| {
            // Accept but don't track (affects strata inheritance behavior)
            Ok(())
        });

        // SetFixedFrameLevel(fixed) - Controls if level is fixed
        methods.add_method("SetFixedFrameLevel", |_, _this, _fixed: bool| {
            // Accept but don't track (affects level inheritance behavior)
            Ok(())
        });

        // SetToplevel(toplevel) - Mark frame as toplevel (raises on click)
        methods.add_method("SetToplevel", |_, _this, _toplevel: bool| {
            Ok(())
        });

        // IsToplevel()
        methods.add_method("IsToplevel", |_, _this, ()| {
            Ok(false)
        });

        // Raise() - Raise frame above siblings
        methods.add_method("Raise", |_, _this, ()| {
            Ok(())
        });

        // Lower() - Lower frame below siblings
        methods.add_method("Lower", |_, _this, ()| {
            Ok(())
        });

        // SetBackdrop(backdropInfo) - WoW backdrop system for frame backgrounds
        methods.add_method("SetBackdrop", |_, this, backdrop: Option<mlua::Table>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                if let Some(info) = backdrop {
                    frame.backdrop.enabled = true;
                    // Parse edge size if provided
                    if let Ok(edge_size) = info.get::<f32>("edgeSize") {
                        frame.backdrop.edge_size = edge_size;
                    }
                    // Parse insets if provided
                    if let Ok(insets) = info.get::<mlua::Table>("insets") {
                        if let Ok(left) = insets.get::<f32>("left") {
                            frame.backdrop.insets = left;
                        }
                    }
                } else {
                    frame.backdrop.enabled = false;
                }
            }
            Ok(())
        });

        // SetBackdropColor(r, g, b, a) - Set backdrop background color
        methods.add_method(
            "SetBackdropColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.backdrop.enabled = true;
                    frame.backdrop.bg_color =
                        crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // SetBackdropBorderColor(r, g, b, a) - Set backdrop border color
        methods.add_method(
            "SetBackdropBorderColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.backdrop.enabled = true;
                    frame.backdrop.border_color =
                        crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // SetID(id) - Set frame ID (used for tab ordering, etc.)
        methods.add_method("SetID", |_, _this, _id: i32| {
            // Accept ID but don't use it for now
            Ok(())
        });

        // GetID() - Get frame ID
        methods.add_method("GetID", |_, _this, ()| {
            Ok(0) // Default ID
        });

        // EnableMouse(enable)
        methods.add_method("EnableMouse", |_, this, enable: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.mouse_enabled = enable;
            }
            Ok(())
        });

        // IsMouseEnabled()
        methods.add_method("IsMouseEnabled", |_, this, ()| {
            let state = this.state.borrow();
            let enabled = state.widgets.get(this.id).map(|f| f.mouse_enabled).unwrap_or(false);
            Ok(enabled)
        });

        // SetAllPoints(relativeTo)
        methods.add_method("SetAllPoints", |_, this, _relative_to: Option<mlua::AnyUserData>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.clear_all_points();
                // SetAllPoints makes the frame fill its relative frame
                frame.set_point(
                    crate::widget::AnchorPoint::TopLeft,
                    None,
                    crate::widget::AnchorPoint::TopLeft,
                    0.0,
                    0.0,
                );
                frame.set_point(
                    crate::widget::AnchorPoint::BottomRight,
                    None,
                    crate::widget::AnchorPoint::BottomRight,
                    0.0,
                    0.0,
                );
            }
            Ok(())
        });

        // GetPoint(index) -> point, relativeTo, relativePoint, xOfs, yOfs
        methods.add_method("GetPoint", |lua, this, index: Option<i32>| {
            let idx = index.unwrap_or(1) - 1; // Lua is 1-indexed
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(anchor) = frame.anchors.get(idx as usize) {
                    return Ok(mlua::MultiValue::from_vec(vec![
                        Value::String(lua.create_string(anchor.point.as_str())?),
                        Value::Nil, // relativeTo (would need to return frame reference)
                        Value::String(lua.create_string(anchor.relative_point.as_str())?),
                        Value::Number(anchor.x_offset as f64),
                        Value::Number(anchor.y_offset as f64),
                    ]));
                }
            }
            Ok(mlua::MultiValue::new())
        });

        // GetNumPoints()
        methods.add_method("GetNumPoints", |_, this, ()| {
            let state = this.state.borrow();
            let count = state.widgets.get(this.id).map(|f| f.anchors.len()).unwrap_or(0);
            Ok(count as i32)
        });

        // CreateTexture(name, layer, inherits, subLevel)
        methods.add_method("CreateTexture", |lua, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let name: Option<String> = args.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });

            let texture = Frame::new(WidgetType::Texture, name.clone(), Some(this.id));
            let texture_id = texture.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(texture);
                state.widgets.add_child(this.id, texture_id);
            }

            let handle = FrameHandle {
                id: texture_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;

            if let Some(ref n) = name {
                lua.globals().set(n.as_str(), ud.clone())?;
            }

            let frame_key = format!("__frame_{}", texture_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(ud)
        });

        // CreateFontString(name, layer, inherits)
        methods.add_method("CreateFontString", |lua, this, args: mlua::MultiValue| {
            let args: Vec<Value> = args.into_iter().collect();

            let name: Option<String> = args.first().and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.to_string_lossy().to_string())
                } else {
                    None
                }
            });

            let fontstring = Frame::new(WidgetType::FontString, name.clone(), Some(this.id));
            let fontstring_id = fontstring.id;

            {
                let mut state = this.state.borrow_mut();
                state.widgets.register(fontstring);
                state.widgets.add_child(this.id, fontstring_id);
            }

            let handle = FrameHandle {
                id: fontstring_id,
                state: Rc::clone(&this.state),
            };

            let ud = lua.create_userdata(handle)?;

            if let Some(ref n) = name {
                lua.globals().set(n.as_str(), ud.clone())?;
            }

            let frame_key = format!("__frame_{}", fontstring_id);
            lua.globals().set(frame_key.as_str(), ud.clone())?;

            Ok(ud)
        });

        // SetTexture(path) - for Texture widgets
        methods.add_method("SetTexture", |_, this, path: Option<String>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.texture = path;
            }
            Ok(())
        });

        // GetTexture() - for Texture widgets
        methods.add_method("GetTexture", |_, this, ()| {
            let state = this.state.borrow();
            let texture = state
                .widgets
                .get(this.id)
                .and_then(|f| f.texture.clone());
            Ok(texture)
        });

        // SetText(text) - for FontString widgets
        methods.add_method("SetText", |_, this, text: Option<String>| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.text = text;
            }
            Ok(())
        });

        // GetText() - for FontString widgets
        methods.add_method("GetText", |_, this, ()| {
            let state = this.state.borrow();
            let text = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.clone())
                .unwrap_or_default();
            Ok(text)
        });

        // SetFont(font, size, flags) - for FontString widgets
        methods.add_method("SetFont", |_, this, (font, size, _flags): (String, Option<f32>, Option<String>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.font = Some(font);
                if let Some(s) = size {
                    frame.font_size = s;
                }
            }
            Ok(true) // Returns success
        });

        // SetVertexColor(r, g, b, a) - for Texture widgets
        methods.add_method(
            "SetVertexColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.vertex_color =
                        Some(crate::widget::Color::new(r, g, b, a.unwrap_or(1.0)));
                }
                Ok(())
            },
        );

        // SetTextColor(r, g, b, a) - for FontString widgets
        methods.add_method(
            "SetTextColor",
            |_, this, (r, g, b, a): (f32, f32, f32, Option<f32>)| {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    frame.text_color = crate::widget::Color::new(r, g, b, a.unwrap_or(1.0));
                }
                Ok(())
            },
        );

        // SetTexCoord(left, right, top, bottom) - for Texture widgets
        methods.add_method("SetTexCoord", |_, _this, _args: mlua::MultiValue| {
            // Store texture coordinates if needed
            Ok(())
        });

        // SetColorTexture(r, g, b, a) - for Texture widgets
        methods.add_method("SetColorTexture", |_, _this, (_r, _g, _b, _a): (f32, f32, f32, Option<f32>)| {
            // Set a solid color instead of texture
            Ok(())
        });

        // SetFontObject(fontObject) - for FontString widgets
        methods.add_method("SetFontObject", |_, _this, _font_object: Value| {
            // Would copy font settings from another FontString
            Ok(())
        });

        // SetJustifyH(justify) - for FontString widgets
        methods.add_method("SetJustifyH", |_, _this, _justify: String| {
            Ok(())
        });

        // SetJustifyV(justify) - for FontString widgets
        methods.add_method("SetJustifyV", |_, _this, _justify: String| {
            Ok(())
        });

        // GetStringWidth() - for FontString widgets
        methods.add_method("GetStringWidth", |_, this, ()| {
            let state = this.state.borrow();
            // Approximate: 7 pixels per character
            let width = state
                .widgets
                .get(this.id)
                .and_then(|f| f.text.as_ref())
                .map(|t| t.len() as f32 * 7.0)
                .unwrap_or(0.0);
            Ok(width)
        });

        // GetStringHeight() - for FontString widgets
        methods.add_method("GetStringHeight", |_, this, ()| {
            let state = this.state.borrow();
            let height = state.widgets.get(this.id).map(|f| f.font_size).unwrap_or(12.0);
            Ok(height)
        });

        // SetForbidden() - marks frame as forbidden (security feature, no-op in simulation)
        methods.add_method("SetForbidden", |_, _this, _forbidden: Option<bool>| {
            Ok(())
        });

        // IsForbidden() - check if frame is forbidden
        methods.add_method("IsForbidden", |_, _this, ()| {
            Ok(false)
        });

        // CanChangeProtectedState() - check if we can change protected state
        methods.add_method("CanChangeProtectedState", |_, _this, ()| {
            Ok(true) // Always true in simulation
        });

        // SetPassThroughButtons(...) - set which mouse buttons pass through
        methods.add_method("SetPassThroughButtons", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetFlattensRenderLayers(flatten) - for render optimization
        methods.add_method("SetFlattensRenderLayers", |_, _this, _flatten: Option<bool>| {
            Ok(())
        });

        // SetClipsChildren(clips) - whether to clip child frames
        methods.add_method("SetClipsChildren", |_, _this, _clips: Option<bool>| {
            Ok(())
        });

        // SetShown(shown) - show/hide based on boolean
        methods.add_method("SetShown", |_, this, shown: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.visible = shown;
            }
            Ok(())
        });

        // GetEffectiveScale() - get combined scale of frame and parents
        methods.add_method("GetEffectiveScale", |_, _this, ()| {
            Ok(1.0f32) // No scaling in simulation
        });

        // GetScale() - get frame's scale
        methods.add_method("GetScale", |_, _this, ()| {
            Ok(1.0f32)
        });

        // SetScale(scale) - set frame's scale
        methods.add_method("SetScale", |_, _this, _scale: f32| {
            Ok(())
        });

        // GetAttribute(name) - get a named attribute from the frame
        methods.add_method("GetAttribute", |lua, this, name: String| {
            let state = this.state.borrow();
            if let Some(frame) = state.widgets.get(this.id) {
                if let Some(attr) = frame.attributes.get(&name) {
                    return match attr {
                        AttributeValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
                        AttributeValue::Number(n) => Ok(Value::Number(*n)),
                        AttributeValue::Boolean(b) => Ok(Value::Boolean(*b)),
                        AttributeValue::Nil => Ok(Value::Nil),
                    };
                }
            }
            Ok(Value::Nil)
        });

        // SetAttribute(name, value) - set a named attribute on the frame
        methods.add_method("SetAttribute", |lua, this, (name, value): (String, Value)| {
            // Store attribute (if it's a simple type)
            {
                let mut state = this.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(this.id) {
                    let attr = match &value {
                        Value::Nil => AttributeValue::Nil,
                        Value::Boolean(b) => AttributeValue::Boolean(*b),
                        Value::Integer(i) => AttributeValue::Number(*i as f64),
                        Value::Number(n) => AttributeValue::Number(*n),
                        Value::String(s) => AttributeValue::String(s.to_str().map(|s| s.to_string()).unwrap_or_default()),
                        _ => AttributeValue::Nil, // Tables etc not stored persistently
                    };
                    if matches!(attr, AttributeValue::Nil) && matches!(value, Value::Nil) {
                        frame.attributes.remove(&name);
                    } else if !matches!(value, Value::Table(_)) {
                        // Only store non-table values
                        frame.attributes.insert(name.clone(), attr);
                    }
                }
            }

            // Trigger OnAttributeChanged script if one exists
            let scripts_table: Option<mlua::Table> = lua.globals().get("__scripts").ok();
            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnAttributeChanged", this.id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();
                if let Some(handler) = handler {
                    // Get frame userdata
                    let frame_ref_key = format!("__frame_{}", this.id);
                    let frame_ud: Value = lua.globals().get(frame_ref_key.as_str()).unwrap_or(Value::Nil);
                    // Call handler with (self, name, value)
                    let name_str = lua.create_string(&name)?;
                    let _ = handler.call::<()>((frame_ud, name_str, value));
                }
            }
            Ok(())
        });

        // ===== Button Methods =====

        // SetNormalFontObject(fontObject) - Set font for normal state
        methods.add_method("SetNormalFontObject", |_, _this, _font_object: Value| {
            Ok(())
        });

        // SetHighlightFontObject(fontObject) - Set font for highlight state
        methods.add_method("SetHighlightFontObject", |_, _this, _font_object: Value| {
            Ok(())
        });

        // SetDisabledFontObject(fontObject) - Set font for disabled state
        methods.add_method("SetDisabledFontObject", |_, _this, _font_object: Value| {
            Ok(())
        });

        // GetNormalTexture() - Get the normal state texture
        methods.add_method("GetNormalTexture", |_, _this, ()| {
            Ok(Value::Nil)
        });

        // GetHighlightTexture() - Get the highlight state texture
        methods.add_method("GetHighlightTexture", |_, _this, ()| {
            Ok(Value::Nil)
        });

        // GetPushedTexture() - Get the pushed state texture
        methods.add_method("GetPushedTexture", |_, _this, ()| {
            Ok(Value::Nil)
        });

        // GetDisabledTexture() - Get the disabled state texture
        methods.add_method("GetDisabledTexture", |_, _this, ()| {
            Ok(Value::Nil)
        });

        // SetNormalTexture(texture) - Set texture for normal state
        methods.add_method("SetNormalTexture", |_, _this, _texture: Value| {
            Ok(())
        });

        // SetHighlightTexture(texture) - Set texture for highlight state
        methods.add_method("SetHighlightTexture", |_, _this, _texture: Value| {
            Ok(())
        });

        // SetPushedTexture(texture) - Set texture for pushed state
        methods.add_method("SetPushedTexture", |_, _this, _texture: Value| {
            Ok(())
        });

        // SetDisabledTexture(texture) - Set texture for disabled state
        methods.add_method("SetDisabledTexture", |_, _this, _texture: Value| {
            Ok(())
        });

        // SetEnabled(enabled) - Enable/disable button
        methods.add_method("SetEnabled", |_, _this, _enabled: bool| {
            Ok(())
        });

        // IsEnabled() - Check if button is enabled
        methods.add_method("IsEnabled", |_, _this, ()| {
            Ok(true)
        });

        // Click() - Simulate button click
        methods.add_method("Click", |_, _this, ()| {
            Ok(())
        });

        // RegisterForClicks(...) - Register which mouse buttons trigger clicks
        methods.add_method("RegisterForClicks", |_, _this, _args: mlua::MultiValue| {
            Ok(())
        });

        // SetButtonState(state, locked) - Set button visual state
        methods.add_method("SetButtonState", |_, _this, (_state, _locked): (String, Option<bool>)| {
            Ok(())
        });

        // GetButtonState() - Get button visual state
        methods.add_method("GetButtonState", |_, _this, ()| {
            Ok("NORMAL".to_string())
        });
    }
}
