//! CreateFrame implementation for creating WoW frames from Lua.

use super::super::frame::FrameHandle;
use super::super::SimState;
use crate::widget::{Anchor, AnchorPoint, AttributeValue, Frame, WidgetType};
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Create the CreateFrame Lua function.
pub fn create_frame_function(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    let state_clone = Rc::clone(&state);
    let create_frame = lua.create_function(move |lua, args: mlua::MultiValue| {
        let mut args_iter = args.into_iter();

        let frame_type: String = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Frame".to_string());

        let name_raw: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        let parent_arg = args_iter.next();
        let parent_id: Option<u64> = parent_arg.as_ref().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });

        // Get template parameter (4th argument)
        let template: Option<String> = args_iter
            .next()
            .and_then(|v| lua.coerce_string(v).ok().flatten())
            .map(|s| s.to_string_lossy().to_string());

        // Get parent ID (default to UIParent)
        let parent_id = parent_id.or_else(|| {
            let state = state_clone.borrow();
            state.widgets.get_id_by_name("UIParent")
        });

        // Handle $parent/$Parent name substitution
        let name: Option<String> = name_raw.map(|n| {
            if n.contains("$parent") || n.contains("$Parent") {
                if let Some(pid) = parent_id {
                    let state = state_clone.borrow();
                    if let Some(parent_name) = state.widgets.get(pid).and_then(|f| f.name.clone())
                    {
                        n.replace("$parent", &parent_name)
                            .replace("$Parent", &parent_name)
                    } else {
                        n.replace("$parent", "").replace("$Parent", "")
                    }
                } else {
                    n.replace("$parent", "").replace("$Parent", "")
                }
            } else {
                n
            }
        });

        let widget_type = WidgetType::from_str(&frame_type).unwrap_or(WidgetType::Frame);
        let frame = Frame::new(widget_type, name.clone(), parent_id);
        let frame_id = frame.id;

        // Track if we need to create tooltip FontStrings after userdata creation
        let mut needs_tooltip_fontstrings = false;
        // Track if we need to register HybridScrollBarTemplate children as globals
        let mut needs_hybrid_scroll_globals = false;

        let mut state = state_clone.borrow_mut();
        state.widgets.register(frame);

        if let Some(pid) = parent_id {
            state.widgets.add_child(pid, frame_id);

            // Inherit strata and level from parent (like wowless does)
            let parent_props = state.widgets.get(pid).map(|p| (p.frame_strata, p.frame_level));
            if let Some((parent_strata, parent_level)) = parent_props {
                if let Some(f) = state.widgets.get_mut(frame_id) {
                    f.frame_strata = parent_strata;
                    f.frame_level = parent_level + 1;
                }
            }
        }

        // Handle template-based child elements
        // UICheckButtonTemplate and similar templates create a Text FontString
        if let Some(ref tmpl) = template {
            if tmpl.contains("CheckButton") || tmpl.contains("CheckBox") {
                // Create Text FontString child for checkbox templates
                let text_name = name.as_ref().map(|n| format!("{}Text", n));
                let text_frame = Frame::new(WidgetType::FontString, text_name.clone(), Some(frame_id));
                let text_id = text_frame.id;
                state.widgets.register(text_frame);
                state.widgets.add_child(frame_id, text_id);

                // Store the text child reference on the parent frame
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                }
            }

            // HybridScrollBarTemplate creates track textures directly on the scrollbar frame
            if tmpl.contains("HybridScroll") {
                // Create ThumbTexture
                let thumb_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let thumb_id = thumb_tex.id;
                state.widgets.register(thumb_tex);
                state.widgets.add_child(frame_id, thumb_id);

                // Create ScrollBarTop texture
                let top_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let top_id = top_tex.id;
                state.widgets.register(top_tex);
                state.widgets.add_child(frame_id, top_id);

                // Create ScrollBarMiddle texture
                let mid_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let mid_id = mid_tex.id;
                state.widgets.register(mid_tex);
                state.widgets.add_child(frame_id, mid_id);

                // Create ScrollBarBottom texture
                let bot_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bot_id = bot_tex.id;
                state.widgets.register(bot_tex);
                state.widgets.add_child(frame_id, bot_id);

                // Create ScrollUpButton
                let up_name = name.as_ref().map(|n| format!("{}ScrollUpButton", n));
                let up_button = Frame::new(WidgetType::Button, up_name.clone(), Some(frame_id));
                let up_id = up_button.id;
                state.widgets.register(up_button);
                state.widgets.add_child(frame_id, up_id);

                // Create ScrollDownButton
                let down_name = name.as_ref().map(|n| format!("{}ScrollDownButton", n));
                let down_button = Frame::new(WidgetType::Button, down_name.clone(), Some(frame_id));
                let down_id = down_button.id;
                state.widgets.register(down_button);
                state.widgets.add_child(frame_id, down_id);

                if let Some(sb) = state.widgets.get_mut(frame_id) {
                    sb.children_keys.insert("ThumbTexture".to_string(), thumb_id);
                    sb.children_keys.insert("ScrollBarTop".to_string(), top_id);
                    sb.children_keys.insert("ScrollBarMiddle".to_string(), mid_id);
                    sb.children_keys.insert("ScrollBarBottom".to_string(), bot_id);
                    sb.children_keys.insert("ScrollUpButton".to_string(), up_id);
                    sb.children_keys.insert("ScrollDownButton".to_string(), down_id);
                }
                // Register children as globals after state is dropped (if frame has a name)
                if name.is_some() {
                    needs_hybrid_scroll_globals = true;
                }
            }

            // FauxScrollFrameTemplate and ScrollFrameTemplate create ScrollBar with buttons as child
            if tmpl.contains("ScrollFrame") || (tmpl.contains("ScrollBar") && !tmpl.contains("HybridScroll")) {
                // Create ScrollBar slider child
                let scrollbar_name = name.as_ref().map(|n| format!("{}ScrollBar", n));
                let mut scrollbar = Frame::new(WidgetType::Slider, scrollbar_name.clone(), Some(frame_id));
                scrollbar.visible = false; // Usually hidden by default
                let scrollbar_id = scrollbar.id;
                state.widgets.register(scrollbar);
                state.widgets.add_child(frame_id, scrollbar_id);

                // Create ThumbTexture for ScrollBar
                let thumb_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let thumb_id = thumb_tex.id;
                state.widgets.register(thumb_tex);
                state.widgets.add_child(scrollbar_id, thumb_id);

                // Create ScrollBarTop texture (scrollbar track top)
                let top_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let top_id = top_tex.id;
                state.widgets.register(top_tex);
                state.widgets.add_child(scrollbar_id, top_id);

                // Create ScrollBarMiddle texture (scrollbar track middle)
                let mid_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let mid_id = mid_tex.id;
                state.widgets.register(mid_tex);
                state.widgets.add_child(scrollbar_id, mid_id);

                // Create ScrollBarBottom texture (scrollbar track bottom)
                let bot_tex = Frame::new(WidgetType::Texture, None, Some(scrollbar_id));
                let bot_id = bot_tex.id;
                state.widgets.register(bot_tex);
                state.widgets.add_child(scrollbar_id, bot_id);

                if let Some(sb) = state.widgets.get_mut(scrollbar_id) {
                    sb.children_keys.insert("ThumbTexture".to_string(), thumb_id);
                    sb.children_keys.insert("ScrollBarTop".to_string(), top_id);
                    sb.children_keys.insert("ScrollBarMiddle".to_string(), mid_id);
                    sb.children_keys.insert("ScrollBarBottom".to_string(), bot_id);
                }

                // Create ScrollUpButton child of ScrollBar
                let up_name = name.as_ref().map(|n| format!("{}ScrollBarScrollUpButton", n));
                let up_button = Frame::new(WidgetType::Button, up_name.clone(), Some(scrollbar_id));
                let up_id = up_button.id;
                state.widgets.register(up_button);
                state.widgets.add_child(scrollbar_id, up_id);

                // Create Normal texture for ScrollUpButton
                let up_normal = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_normal_id = up_normal.id;
                state.widgets.register(up_normal);
                state.widgets.add_child(up_id, up_normal_id);

                // Create Pushed texture for ScrollUpButton
                let up_pushed = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_pushed_id = up_pushed.id;
                state.widgets.register(up_pushed);
                state.widgets.add_child(up_id, up_pushed_id);

                // Create Disabled texture for ScrollUpButton
                let up_disabled = Frame::new(WidgetType::Texture, None, Some(up_id));
                let up_disabled_id = up_disabled.id;
                state.widgets.register(up_disabled);
                state.widgets.add_child(up_id, up_disabled_id);

                // Store texture references on ScrollUpButton
                if let Some(up_btn) = state.widgets.get_mut(up_id) {
                    up_btn.children_keys.insert("Normal".to_string(), up_normal_id);
                    up_btn.children_keys.insert("Pushed".to_string(), up_pushed_id);
                    up_btn.children_keys.insert("Disabled".to_string(), up_disabled_id);
                }

                // Create ScrollDownButton child of ScrollBar
                let down_name = name.as_ref().map(|n| format!("{}ScrollBarScrollDownButton", n));
                let down_button = Frame::new(WidgetType::Button, down_name.clone(), Some(scrollbar_id));
                let down_id = down_button.id;
                state.widgets.register(down_button);
                state.widgets.add_child(scrollbar_id, down_id);

                // Create Normal texture for ScrollDownButton
                let down_normal = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_normal_id = down_normal.id;
                state.widgets.register(down_normal);
                state.widgets.add_child(down_id, down_normal_id);

                // Create Pushed texture for ScrollDownButton
                let down_pushed = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_pushed_id = down_pushed.id;
                state.widgets.register(down_pushed);
                state.widgets.add_child(down_id, down_pushed_id);

                // Create Disabled texture for ScrollDownButton
                let down_disabled = Frame::new(WidgetType::Texture, None, Some(down_id));
                let down_disabled_id = down_disabled.id;
                state.widgets.register(down_disabled);
                state.widgets.add_child(down_id, down_disabled_id);

                // Store texture references on ScrollDownButton
                if let Some(down_btn) = state.widgets.get_mut(down_id) {
                    down_btn.children_keys.insert("Normal".to_string(), down_normal_id);
                    down_btn.children_keys.insert("Pushed".to_string(), down_pushed_id);
                    down_btn.children_keys.insert("Disabled".to_string(), down_disabled_id);
                }

                // Store child references on frames
                if let Some(scrollbar_frame) = state.widgets.get_mut(scrollbar_id) {
                    scrollbar_frame.children_keys.insert("ScrollUpButton".to_string(), up_id);
                    scrollbar_frame.children_keys.insert("ScrollDownButton".to_string(), down_id);
                }
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("ScrollBar".to_string(), scrollbar_id);
                }
            }

            // PanelTabButtonTemplate creates Text FontString
            if tmpl.contains("PanelTabButton") || tmpl.contains("TabButtonTemplate") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                }
            }

            // DefaultPanelTemplate creates Bg texture and TitleText FontString
            if tmpl.contains("DefaultPanelTemplate") || tmpl.contains("BasicPanel") {
                // Create Bg texture child
                let bg_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bg_id = bg_tex.id;
                state.widgets.register(bg_tex);
                state.widgets.add_child(frame_id, bg_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Bg".to_string(), bg_id);
                }

                // Create TitleText FontString child
                let title_text = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let title_id = title_text.id;
                state.widgets.register(title_text);
                state.widgets.add_child(frame_id, title_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("TitleText".to_string(), title_id);
                }
            }

            // ObjectiveTrackerContainerHeaderTemplate creates Text FontString, MinimizeButton, and Background
            if tmpl.contains("ObjectiveTrackerContainerHeader") || tmpl.contains("ObjectiveTrackerHeader") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);

                // Create MinimizeButton child
                let min_button = Frame::new(WidgetType::Button, None, Some(frame_id));
                let min_id = min_button.id;
                state.widgets.register(min_button);
                state.widgets.add_child(frame_id, min_id);

                // Create Background texture child (used by WorldQuestTracker)
                let bg_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
                let bg_id = bg_tex.id;
                state.widgets.register(bg_tex);
                state.widgets.add_child(frame_id, bg_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                    parent_frame.children_keys.insert("MinimizeButton".to_string(), min_id);
                    parent_frame.children_keys.insert("Background".to_string(), bg_id);
                }
            }

            // Tooltip templates create NineSlice child frame (required by SharedTooltipTemplates)
            if tmpl.contains("TooltipTemplate") || tmpl.contains("Tooltip") {
                let nine_slice = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let nine_slice_id = nine_slice.id;
                state.widgets.register(nine_slice);
                state.widgets.add_child(frame_id, nine_slice_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame
                        .children_keys
                        .insert("NineSlice".to_string(), nine_slice_id);
                }
            }

            // GameTooltipTemplate - mark that we need to create TextLeftN/TextRightN FontStrings
            // This is handled after the main frame userdata is created
            if tmpl.contains("GameTooltipTemplate") {
                needs_tooltip_fontstrings = true;
            }

            // NOTE: PortraitFrameTemplate and ButtonFrameTemplate children (TitleContainer, NineSlice,
            // CloseButton, PortraitContainer) are now created by the XML template system via
            // instantiate_template_children in loader.rs. The parentKey handling there also
            // updates children_keys for SetTitle and other lookups.

            // NOTE: ButtonFrameTemplate handling is now done via proper template inheritance
            // in loader.rs (instantiate_template_children). The template registry stores
            // virtual frames and their children are automatically instantiated when a frame
            // inherits from them.

            // EditModeSystemSelectionTemplate creates a Label FontString (used by LibEditMode)
            // Also sets parent.Selection = self for addon access patterns
            if tmpl.contains("EditModeSystemSelection") {
                let label_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let label_id = label_fs.id;
                state.widgets.register(label_fs);
                state.widgets.add_child(frame_id, label_id);
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Label".to_string(), label_id);
                }

                // Set the Selection key on the parent frame so addon code can access via parent.Selection
                if let Some(pid) = parent_id {
                    if let Some(actual_parent) = state.widgets.get_mut(pid) {
                        actual_parent.children_keys.insert("Selection".to_string(), frame_id);
                    }
                }
            }

            // SettingsCheckBoxControlTemplate creates Text and Checkbox children
            if tmpl.contains("SettingsCheckBoxControl") || tmpl.contains("SettingsCheckbox") {
                // Create Text FontString child
                let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                let text_id = text_fs.id;
                state.widgets.register(text_fs);
                state.widgets.add_child(frame_id, text_id);

                // Create Checkbox child
                let checkbox = Frame::new(WidgetType::CheckButton, None, Some(frame_id));
                let checkbox_id = checkbox.id;
                state.widgets.register(checkbox);
                state.widgets.add_child(frame_id, checkbox_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Text".to_string(), text_id);
                    parent_frame.children_keys.insert("Checkbox".to_string(), checkbox_id);
                }
            }

            // SettingsListTemplate creates Header and ScrollBox children (used by Settings UI)
            if tmpl.contains("SettingsListTemplate") || tmpl.contains("SettingsList") {
                // Create Header frame
                let header = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let header_id = header.id;
                state.widgets.register(header);
                state.widgets.add_child(frame_id, header_id);

                // Create Header.DefaultsButton
                let defaults_btn = Frame::new(WidgetType::Button, None, Some(header_id));
                let defaults_btn_id = defaults_btn.id;
                state.widgets.register(defaults_btn);
                state.widgets.add_child(header_id, defaults_btn_id);

                // Create Header.Title FontString
                let title = Frame::new(WidgetType::FontString, None, Some(header_id));
                let title_id = title.id;
                state.widgets.register(title);
                state.widgets.add_child(header_id, title_id);

                // Store Header children keys
                if let Some(header_frame) = state.widgets.get_mut(header_id) {
                    header_frame.children_keys.insert("DefaultsButton".to_string(), defaults_btn_id);
                    header_frame.children_keys.insert("Title".to_string(), title_id);
                }

                // Create ScrollBox frame
                let scrollbox = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let scrollbox_id = scrollbox.id;
                state.widgets.register(scrollbox);
                state.widgets.add_child(frame_id, scrollbox_id);

                // Create ScrollBox.ScrollTarget
                let scroll_target = Frame::new(WidgetType::Frame, None, Some(scrollbox_id));
                let scroll_target_id = scroll_target.id;
                state.widgets.register(scroll_target);
                state.widgets.add_child(scrollbox_id, scroll_target_id);

                if let Some(sb) = state.widgets.get_mut(scrollbox_id) {
                    sb.children_keys.insert("ScrollTarget".to_string(), scroll_target_id);
                }

                // Store children keys on main frame
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Header".to_string(), header_id);
                    parent_frame.children_keys.insert("ScrollBox".to_string(), scrollbox_id);
                }
            }

            // SecureGroupHeaderTemplate and SecureGroupPetHeaderTemplate create indexed child frames
            if tmpl == "SecureGroupHeaderTemplate" || tmpl == "SecureGroupPetHeaderTemplate" {
                // Create at least 5 child frames indexed numerically [1], [2], etc.
                for _i in 1..=5 {
                    let child = Frame::new(WidgetType::Button, None, Some(frame_id));
                    let child_id = child.id;
                    state.widgets.register(child);
                    state.widgets.add_child(frame_id, child_id);
                }
            }

            // PlumberSettingsPanelLayoutTemplate creates FrameContainer with nested child frames
            if tmpl == "PlumberSettingsPanelLayoutTemplate" {
                // Create FrameContainer
                let frame_container = Frame::new(WidgetType::Frame, None, Some(frame_id));
                let frame_container_id = frame_container.id;
                state.widgets.register(frame_container);
                state.widgets.add_child(frame_id, frame_container_id);

                // Create child frames inside FrameContainer
                let child_keys = ["LeftSection", "RightSection", "CentralSection", "SideTab",
                                  "TabButtonContainer", "ModuleTab", "ChangelogTab"];
                for key in child_keys {
                    let child = Frame::new(WidgetType::Frame, None, Some(frame_container_id));
                    let child_id = child.id;
                    state.widgets.register(child);
                    state.widgets.add_child(frame_container_id, child_id);
                    if let Some(fc) = state.widgets.get_mut(frame_container_id) {
                        fc.children_keys.insert(key.to_string(), child_id);
                    }
                }

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("FrameContainer".to_string(), frame_container_id);
                }
            }

            // AddonListEntryTemplate creates Title, Status, Reload FontStrings, Enabled CheckButton, LoadAddonButton
            // Used by Blizzard_AddOnList for addon list entries
            if tmpl.contains("AddonListEntry") {
                // Create Title FontString (300x12)
                let mut title_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                title_fs.width = 300.0;
                title_fs.height = 16.0; // Taller for better visibility
                title_fs.font_size = 14.0;
                title_fs.text = Some("TestAddon".to_string()); // Placeholder text
                title_fs.text_color = crate::widget::Color::new(1.0, 0.78, 0.0, 1.0); // Gold color
                // Anchor Title at LEFT + 32px (leaving room for checkbox)
                title_fs.anchors.push(Anchor {
                    point: AnchorPoint::Left,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Left,
                    x_offset: 32.0,
                    y_offset: 0.0,
                });
                let title_id = title_fs.id;
                state.widgets.register(title_fs);
                state.widgets.add_child(frame_id, title_id);

                // Create Status FontString (anchored RIGHT)
                let mut status_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                status_fs.width = 100.0;
                status_fs.height = 12.0;
                status_fs.anchors.push(Anchor {
                    point: AnchorPoint::Right,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Right,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
                let status_id = status_fs.id;
                state.widgets.register(status_fs);
                state.widgets.add_child(frame_id, status_id);

                // Create Reload FontString (220x12, anchored RIGHT)
                let mut reload_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                reload_fs.width = 220.0;
                reload_fs.height = 12.0;
                reload_fs.visible = false; // Hidden by default
                reload_fs.anchors.push(Anchor {
                    point: AnchorPoint::Right,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Right,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
                let reload_id = reload_fs.id;
                state.widgets.register(reload_fs);
                state.widgets.add_child(frame_id, reload_id);

                // Create Enabled CheckButton (24x24, anchored LEFT)
                let mut enabled_cb = Frame::new(WidgetType::CheckButton, None, Some(frame_id));
                enabled_cb.width = 24.0;
                enabled_cb.height = 24.0;
                enabled_cb.anchors.push(Anchor {
                    point: AnchorPoint::Left,
                    relative_to: None,
                    relative_to_id: Some(frame_id as usize),
                    relative_point: AnchorPoint::Left,
                    x_offset: 4.0,
                    y_offset: 0.0,
                });
                let enabled_id = enabled_cb.id;
                state.widgets.register(enabled_cb);
                state.widgets.add_child(frame_id, enabled_id);

                // Create CheckedTexture for the CheckButton
                let mut checked_tex = Frame::new(WidgetType::Texture, None, Some(enabled_id));
                checked_tex.width = 24.0;
                checked_tex.height = 24.0;
                checked_tex.visible = false; // Hidden until SetChecked(true) is called
                // Set atlas texture to checkmark-minimal
                checked_tex.attributes.insert("__atlas".to_string(), AttributeValue::String("checkmark-minimal".to_string()));
                let checked_tex_id = checked_tex.id;
                state.widgets.register(checked_tex);
                state.widgets.add_child(enabled_id, checked_tex_id);
                if let Some(cb) = state.widgets.get_mut(enabled_id) {
                    cb.children_keys.insert("CheckedTexture".to_string(), checked_tex_id);
                }

                // Create LoadAddonButton (100x22, hidden by default)
                let mut load_btn = Frame::new(WidgetType::Button, None, Some(frame_id));
                load_btn.width = 100.0;
                load_btn.height = 22.0;
                load_btn.visible = false;
                let load_btn_id = load_btn.id;
                state.widgets.register(load_btn);
                state.widgets.add_child(frame_id, load_btn_id);

                // Store children keys
                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Title".to_string(), title_id);
                    parent_frame.children_keys.insert("Status".to_string(), status_id);
                    parent_frame.children_keys.insert("Reload".to_string(), reload_id);
                    parent_frame.children_keys.insert("Enabled".to_string(), enabled_id);
                    parent_frame.children_keys.insert("LoadAddonButton".to_string(), load_btn_id);
                }
            }

            // AddonListCategoryTemplate creates Title FontString
            // Used by Blizzard_AddOnList for category headers
            if tmpl.contains("AddonListCategory") && !tmpl.contains("Entry") {
                // Create Title FontString
                let mut title_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
                title_fs.width = 300.0;
                title_fs.height = 12.0;
                let title_id = title_fs.id;
                state.widgets.register(title_fs);
                state.widgets.add_child(frame_id, title_id);

                if let Some(parent_frame) = state.widgets.get_mut(frame_id) {
                    parent_frame.children_keys.insert("Title".to_string(), title_id);
                }
            }
        }

        // Button widget types get NormalTexture, PushedTexture, HighlightTexture, DisabledTexture children
        if widget_type == WidgetType::Button || widget_type == WidgetType::CheckButton {
            // Create NormalTexture
            let normal_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let normal_id = normal_tex.id;
            state.widgets.register(normal_tex);
            state.widgets.add_child(frame_id, normal_id);

            // Create PushedTexture
            let pushed_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let pushed_id = pushed_tex.id;
            state.widgets.register(pushed_tex);
            state.widgets.add_child(frame_id, pushed_id);

            // Create HighlightTexture
            let highlight_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let highlight_id = highlight_tex.id;
            state.widgets.register(highlight_tex);
            state.widgets.add_child(frame_id, highlight_id);

            // Create DisabledTexture
            let disabled_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let disabled_id = disabled_tex.id;
            state.widgets.register(disabled_tex);
            state.widgets.add_child(frame_id, disabled_id);

            // Create Icon texture (for action buttons)
            let icon_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let icon_id = icon_tex.id;
            state.widgets.register(icon_tex);
            state.widgets.add_child(frame_id, icon_id);

            // Create IconOverlay texture (for action buttons)
            let icon_overlay_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let icon_overlay_id = icon_overlay_tex.id;
            state.widgets.register(icon_overlay_tex);
            state.widgets.add_child(frame_id, icon_overlay_id);

            // Create Border texture (for action buttons)
            let border_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let border_id = border_tex.id;
            state.widgets.register(border_tex);
            state.widgets.add_child(frame_id, border_id);

            // Create Text fontstring for button label
            let mut text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            // Center the text on the button
            text_fs.anchors.push(crate::widget::Anchor {
                point: crate::widget::AnchorPoint::Center,
                relative_to: None,
                relative_to_id: Some(frame_id as usize),
                relative_point: crate::widget::AnchorPoint::Center,
                x_offset: 0.0,
                y_offset: 0.0,
            });
            text_fs.draw_layer = crate::widget::DrawLayer::Overlay;
            let text_id = text_fs.id;
            state.widgets.register(text_fs);
            state.widgets.add_child(frame_id, text_id);

            // Store texture references as children_keys
            if let Some(btn) = state.widgets.get_mut(frame_id) {
                btn.children_keys.insert("NormalTexture".to_string(), normal_id);
                btn.children_keys.insert("PushedTexture".to_string(), pushed_id);
                btn.children_keys.insert("HighlightTexture".to_string(), highlight_id);
                btn.children_keys.insert("DisabledTexture".to_string(), disabled_id);
                btn.children_keys.insert("Icon".to_string(), icon_id);
                btn.children_keys.insert("IconOverlay".to_string(), icon_overlay_id);
                btn.children_keys.insert("Border".to_string(), border_id);
                btn.children_keys.insert("Text".to_string(), text_id);
            }
        }

        // Slider widget types get Low, High, Text fontstrings and ThumbTexture
        if widget_type == WidgetType::Slider {
            // Create Low fontstring (min value label)
            let low_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let low_id = low_fs.id;
            state.widgets.register(low_fs);
            state.widgets.add_child(frame_id, low_id);

            // Create High fontstring (max value label)
            let high_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let high_id = high_fs.id;
            state.widgets.register(high_fs);
            state.widgets.add_child(frame_id, high_id);

            // Create Text fontstring (current value label)
            let text_fs = Frame::new(WidgetType::FontString, None, Some(frame_id));
            let text_id = text_fs.id;
            state.widgets.register(text_fs);
            state.widgets.add_child(frame_id, text_id);

            // Create ThumbTexture
            let thumb_tex = Frame::new(WidgetType::Texture, None, Some(frame_id));
            let thumb_id = thumb_tex.id;
            state.widgets.register(thumb_tex);
            state.widgets.add_child(frame_id, thumb_id);

            // Store references as children_keys
            if let Some(slider) = state.widgets.get_mut(frame_id) {
                slider.children_keys.insert("Low".to_string(), low_id);
                slider.children_keys.insert("High".to_string(), high_id);
                slider.children_keys.insert("Text".to_string(), text_id);
                slider.children_keys.insert("ThumbTexture".to_string(), thumb_id);
            }
        }

        drop(state); // Release borrow before creating userdata

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

        // Create TextLeftN/TextRightN FontStrings for GameTooltipTemplate
        if needs_tooltip_fontstrings {
            if let Some(ref frame_name) = name {
                // Create TextLeft1..4 and TextRight1..4 as globals using Lua
                let lua_code = format!(
                    r#"
                    local tooltip = {}
                    for i = 1, 4 do
                        local leftName = "{}TextLeft" .. i
                        local rightName = "{}TextRight" .. i
                        _G[leftName] = tooltip:CreateFontString(leftName, "ARTWORK", "GameTooltipText")
                        _G[rightName] = tooltip:CreateFontString(rightName, "ARTWORK", "GameTooltipText")
                    end
                    "#,
                    frame_name, frame_name, frame_name
                );
                let _ = lua.load(&lua_code).exec();
            }
        }

        // Register HybridScrollBarTemplate children as globals
        if needs_hybrid_scroll_globals {
            if let Some(ref frame_name) = name {
                // Register scroll buttons and thumb texture as globals using Lua
                let lua_code = format!(
                    r#"
                    local frame = {}
                    local frameName = "{}"
                    _G[frameName .. "ScrollUpButton"] = frame.ScrollUpButton
                    _G[frameName .. "ScrollDownButton"] = frame.ScrollDownButton
                    _G[frameName .. "ThumbTexture"] = frame.ThumbTexture
                    "#,
                    frame_name, frame_name
                );
                let _ = lua.load(&lua_code).exec();
            }
        }

        Ok(ud)
    })?;
    Ok(create_frame)
}
