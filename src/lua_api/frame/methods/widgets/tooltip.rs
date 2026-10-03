//! GameTooltip widget methods.

mod content;
mod line_data;
mod line_frames;
mod owner;
mod sizing;

use self::content::*;
use self::line_data::*;
use self::line_frames::*;
use self::owner::*;
use crate::lua_bridge::table_set_rust_fn;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

const TOOLTIP_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Lines
    ("ClearLines", clear_lines),
    ("AddLine", add_line),
    ("AddDoubleLine", add_double_line),
    ("AddTexture", add_texture),
    ("AddAtlas", add_atlas),
    ("NumLines", num_lines),
    ("GetNumLines", num_lines),
    ("GetLeftLine", get_left_line),
    ("GetRightLine", get_right_line),
    // Layout (spacing, width, padding)
    ("SetCustomLineSpacing", set_custom_line_spacing),
    ("GetCustomLineSpacing", get_custom_line_spacing),
    ("SetMinimumWidth", set_minimum_width),
    ("GetMinimumWidth", get_minimum_width),
    ("SetAllowShowWithNoLines", set_allow_show_with_no_lines),
    ("SetCustomWordWrapMinWidth", set_custom_word_wrap_min_width),
    ("SetShrinkToFitWrapped", set_shrink_to_fit_wrapped),
    ("SetPadding", set_padding),
    ("GetPadding", get_padding),
    ("ClearPadding", clear_padding),
    ("AppendText", append_text),
    // Content — spell/unit/item getters + setters
    ("GetSpell", get_spell),
    ("GetUnit", get_unit),
    ("GetItem", get_item),
    ("SetSpellByID", set_spell_by_id),
    ("SetSpellBookItem", set_spell_book_item),
    ("SetShapeshift", set_shapeshift),
    ("SetItemByID", set_item_by_id),
    ("SetMountBySpellID", set_mount_by_spell_id),
    ("SetCompanionPet", set_companion_pet),
    ("SetTalent", set_talent),
    ("SetGlyph", set_glyph),
    ("SetToyByItemID", set_toy_by_item_id),
    ("SetHeirloomByItemID", set_heirloom_by_item_id),
    ("SetHyperlink", set_hyperlink),
    ("SetAction", set_action),
    ("SetBagItem", set_bag_item),
    ("SetBackpackToken", set_backpack_token),
    ("SetCurrencyToken", set_currency_token),
    ("SetInventoryItem", set_inventory_item),
    ("SetSocketedItem", set_socketed_item),
    ("SetSocketGem", set_socket_gem),
    ("SetExistingSocketGem", set_existing_socket_gem),
    ("SetTradePlayerItem", set_trade_player_item),
    ("SetTradeTargetItem", set_trade_target_item),
    ("SetInboxItem", set_inbox_item),
    ("SetSendMailItem", set_send_mail_item),
    ("SetTradeSkillItem", set_trade_skill_item),
    ("SetUnitBuff", set_unit_buff),
    (
        "SetUnitBuffByAuraInstanceID",
        set_unit_buff_by_aura_instance_id,
    ),
    ("SetUnitDebuff", set_unit_debuff),
    (
        "SetUnitDebuffByAuraInstanceID",
        set_unit_debuff_by_aura_instance_id,
    ),
    ("SetUnitAura", set_unit_aura),
    (
        "SetUnitAuraByAuraInstanceID",
        set_unit_aura_by_aura_instance_id,
    ),
    // Ownership + anchoring
    ("SetOwner", set_owner),
    ("SetObjectTooltipPosition", set_object_tooltip_position),
    ("GetOwner", get_owner),
    ("IsOwned", is_owned),
    ("FadeOut", fade_out),
    ("GetAnchorType", get_anchor_type),
    ("SetAnchorType", set_anchor_type),
    // Misc
    ("CopyTooltip", copy_tooltip),
    ("SetFrameStack", set_frame_stack),
    ("AddFontStrings", add_font_strings),
    ("IsEquippedItem", is_equipped_item),
    ("ResetSecondaryCompareItem", reset_secondary_compare_item),
    (
        "AdvanceSecondaryCompareItem",
        advance_secondary_compare_item,
    ),
    ("SetCompareItem", set_compare_item),
];

/// Tooltip side of the shared `SetUnit` name; dispatched by `widgets::set_unit`.
pub(super) fn set_tooltip_unit(state: &mut LuaState) -> LuaResult<u32> {
    set_unit(state)
}

pub(super) fn register_tooltip(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in TOOLTIP_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}
