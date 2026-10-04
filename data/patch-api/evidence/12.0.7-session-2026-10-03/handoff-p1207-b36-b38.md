# Retail 12.0.7 B36–B38 authoring handoff

Active goal: author pending B36–B38 row evidence, exact master-anchored proposed edits, staged public-Lua behavioral tests and spec. Completion: static anchor/evidence review and deliverable inventory. Exclusions: integration, repo/git mutation, cargo/tests/builds/simulator, agents/model CLIs. Writes confined to this audit cache. Runtime proof not run, per request.

Status: evidence collection in progress; no coverage credit claimed.

## Scope checkpoint
All 13 requested row IDs remain audit-pending in the current ledger. B37/B38 require live per-widget origin state and wrapped outputs, not read denial. Existing secret_origin::unwrap_input is gated on forbidden-aspects and therefore cannot be reused unchanged for a strict retail-12-0-7 gate. B36 has two producers: frame formatting::set_font and CreateFont's fonts::font_set_font. Neither validates assets or heights. Cached docs are later-cache contract evidence, not native historical proof.

Planned separation: state fields/defaults, independently useful all-argument authentication helper and SetFont authentication prerequisites; withhold eight widget producers for RED. No compilation or execution evidence will be claimed.

## Anchored edits (13)

Base master: `54799ac3e17eef168011e6692d2200c9b512a02a`. Every OLD matches once. OLD blocks are nonoverlapping.

RED: apply E01–E03 plus staged tests/spec, withhold E04–E11 widget producers. E12/E13 are independent B36 authentication prerequisites; include them only when exercising that prerequisite. No row credit until actual GREEN and independent acceptance.

### E01 — `state` — `src/widget/frame.rs`

Explicit live per-widget origin state.

OLD:
```rust
    pub secret_texture: bool,
```

NEW:
```rust
    pub secret_texture: bool,
    /// INFERRED: per-input origins keep shared aspects secret until both clear.
    pub secret_button_state: bool,
    pub secret_button_enabled: bool,
    pub secret_scroll_horizontal: bool,
    pub secret_scroll_vertical: bool,
```

### E02 — `state` — `src/widget/frame_defaults.rs`

Default origins are public.

OLD:
```rust
            secret_texture: false,
```

NEW:
```rust
            secret_texture: false,
            secret_button_state: false,
            secret_button_enabled: false,
            secret_scroll_horizontal: false,
            secret_scroll_vertical: false,
```

### E03 — `test-support` — `src/lua_api/frame/methods/secret_origin.rs`

Independently useful authentication prerequisite; direct retail-12-0-7 gate, not forbidden-aspects.

OLD:
```rust
pub(crate) fn unwrap_input(state: &LuaState, value: Val) -> LuaResult<(Val, bool)> {
    if !cfg!(feature = "forbidden-aspects") {
        return Ok((value, false));
    }
    let secret = rilua::table_security::is_secret_value(state, value);
    let value = rilua::table_security::unwrap_secret(state, value)?;
    Ok((value, secret))
}
```

NEW:
```rust
pub(crate) fn unwrap_input(state: &LuaState, value: Val) -> LuaResult<(Val, bool)> {
    if !cfg!(feature = "forbidden-aspects") {
        return Ok((value, false));
    }
    let secret = rilua::table_security::is_secret_value(state, value);
    let value = rilua::table_security::unwrap_secret(state, value)?;
    Ok((value, secret))
}

/// Authenticate receiver, named arguments and extras before any validation.
/// Replace call-stack inputs only after the entire list authenticates.
pub(crate) fn authenticate_retail_arguments(state: &mut LuaState) -> LuaResult<bool> {
    if !cfg!(feature = "retail-12-0-7") {
        return Ok(false);
    }
    let values = state.stack[state.base..state.top]
        .iter()
        .map(|value| {
            let secret = rilua::table_security::is_secret_value(state, *value);
            let plain = rilua::table_security::unwrap_secret(state, *value)?;
            Ok((plain, secret))
        })
        .collect::<LuaResult<Vec<_>>>()?;
    // INFERRED: accepted extra secret arguments also contribute to the aspect.
    let secret = values.iter().skip(1).any(|(_, secret)| *secret);
    for (index, (value, _)) in values.into_iter().enumerate() {
        state.stack_set(state.base + index, value);
    }
    Ok(secret)
}
```

### E04 — `producer` — `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`

Wrap live enabled state for secure and tainted reads.

OLD:
```rust
pub(super) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(button_enabled).unwrap_or(true)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}
```

NEW:
```rust
pub(super) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (enabled, secret) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let enabled = frame.map(button_enabled).unwrap_or(true);
        let secret = cfg!(feature = "retail-12-0-7")
            && frame.is_some_and(|frame| frame.secret_button_state || frame.secret_button_enabled);
        (enabled, secret)
    };
    let value = if secret {
        rilua::table_security::wrap_host_secret_bool(state, enabled)
    } else {
        Val::Bool(enabled)
    };
    state.push(value);
    Ok(1)
}
```

### E05 — `producer` — `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`

Authenticate before lookup; origin commits before callbacks, including equal enabled state.

OLD:
```rust
pub(super) fn set_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = bool::from_stack(state, 2).ok().unwrap_or(true);
    let changed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| button_enabled(f) != enabled)
            .unwrap_or(false)
    };
    set_button_enabled_value(state, id, enabled)?;
    if changed {
        fire_enable_disable_script(state, id, enabled)?;
    }
    Ok(0)
}
```

NEW:
```rust
pub(super) fn set_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let secret = super::super::secret_origin::authenticate_retail_arguments(state)?;
    let id = frame_id_from_stack(state, 1)?;
    let enabled = bool::from_stack(state, 2).ok().unwrap_or(true);
    let changed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| button_enabled(f) != enabled)
            .unwrap_or(false)
    };
    if cfg!(feature = "retail-12-0-7") {
        if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id) {
            // INFERRED: public overwrite clears only this input's origin.
            frame.secret_button_enabled = secret;
        }
    }
    set_button_enabled_value(state, id, enabled)?;
    if changed {
        fire_enable_disable_script(state, id, enabled)?;
    }
    Ok(0)
}
```

### E06 — `producer` — `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`

Authenticate state, lock and extras before token decoding. Lock remains ignored as before (excluded behavior).

OLD:
```rust
pub(super) fn set_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let state_name = String::from_stack(state, 2)?;
    let pushed = state_name.eq_ignore_ascii_case("PUSHED");
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.button_state = if pushed { 1 } else { 0 };
        }
        sync_button_slot_visibility(&mut sim, id);
    }
    Ok(0)
}
```

NEW:
```rust
pub(super) fn set_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let secret = super::super::secret_origin::authenticate_retail_arguments(state)?;
    let id = frame_id_from_stack(state, 1)?;
    let state_name = String::from_stack(state, 2)?;
    let pushed = state_name.eq_ignore_ascii_case("PUSHED");
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.button_state = if pushed { 1 } else { 0 };
            if cfg!(feature = "retail-12-0-7") {
                // INFERRED: public overwrite clears only this input's origin.
                frame.secret_button_state = secret;
            }
        }
        sync_button_slot_visibility(&mut sim, id);
    }
    Ok(0)
}
```

### E07 — `producer` — `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`

Wrap live token under either ButtonState input origin.

OLD:
```rust
pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let pushed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| frame.button_state == 1)
            .unwrap_or(false)
    };
    let name = if pushed { "PUSHED" } else { "NORMAL" };
    let name_val = create_string(state, name);
    state.push(name_val);
    Ok(1)
}
```

NEW:
```rust
pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (pushed, secret) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let pushed = frame.is_some_and(|frame| frame.button_state == 1);
        let secret = cfg!(feature = "retail-12-0-7")
            && frame.is_some_and(|frame| frame.secret_button_state || frame.secret_button_enabled);
        (pushed, secret)
    };
    let name = if pushed { "PUSHED" } else { "NORMAL" };
    let name_val = if secret {
        rilua::table_security::wrap_host_secret_string(state, name)
    } else {
        create_string(state, name)
    };
    state.push(name_val);
    Ok(1)
}
```

### E08 — `producer` — `src/lua_api/frame/methods/widgets/slider.rs`

Wrap live horizontal offset under either ScrollOffset axis origin.

OLD:
```rust
pub(super) fn get_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_horizontal)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
}
```

NEW:
```rust
pub(super) fn get_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (offset, secret) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let offset = frame.map(|frame| frame.scroll_horizontal).unwrap_or(0.0);
        let secret = cfg!(feature = "retail-12-0-7")
            && frame.is_some_and(|frame| {
                frame.secret_scroll_horizontal || frame.secret_scroll_vertical
            });
        (offset, secret)
    };
    let value = if secret {
        rilua::table_security::wrap_host_secret_number(state, offset)
    } else {
        Val::Num(offset)
    };
    state.push(value);
    Ok(1)
}
```

### E09 — `producer` — `src/lua_api/frame/methods/widgets/slider.rs`

Authenticate before offset decoding, update origin before equal-value early return; existing callbacks/presentation ordering retained.

OLD:
```rust
pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_horizontal == offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_horizontal = offset;
    }
    crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
    drop(sim);
    fire_scroll_frame_event(state, id, "OnHorizontalScroll", &[Val::Num(offset)])?;
    Ok(0)
}
```

NEW:
```rust
pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let secret = crate::lua_api::frame::methods::secret_origin::authenticate_retail_arguments(state)?;
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if cfg!(feature = "retail-12-0-7") {
        if let Some(frame) = sim.widgets.get_mut(id) {
            // INFERRED: public overwrite clears only this axis's origin.
            frame.secret_scroll_horizontal = secret;
        }
    }
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_horizontal == offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_horizontal = offset;
    }
    crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
    drop(sim);
    fire_scroll_frame_event(state, id, "OnHorizontalScroll", &[Val::Num(offset)])?;
    Ok(0)
}
```

### E10 — `producer` — `src/lua_api/frame/methods/widgets/slider.rs`

Wrap live vertical offset under either ScrollOffset axis origin.

OLD:
```rust
pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_vertical)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
}
```

NEW:
```rust
pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (offset, secret) = {
        let sim = borrow_state(state)?;
        let frame = sim.widgets.get(id);
        let offset = frame.map(|frame| frame.scroll_vertical).unwrap_or(0.0);
        let secret = cfg!(feature = "retail-12-0-7")
            && frame.is_some_and(|frame| {
                frame.secret_scroll_horizontal || frame.secret_scroll_vertical
            });
        (offset, secret)
    };
    let value = if secret {
        rilua::table_security::wrap_host_secret_number(state, offset)
    } else {
        Val::Num(offset)
    };
    state.push(value);
    Ok(1)
}
```

### E11 — `producer` — `src/lua_api/frame/methods/widgets/slider.rs`

Authenticate before offset decoding, update origin before equal-value early return; existing callbacks/presentation ordering retained.

OLD:
```rust
pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_vertical == offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_vertical = offset;
    }
    crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
    drop(sim);
    fire_scroll_frame_event(state, id, "OnVerticalScroll", &[Val::Num(offset)])?;
    Ok(0)
}
```

NEW:
```rust
pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let secret = crate::lua_api::frame::methods::secret_origin::authenticate_retail_arguments(state)?;
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if cfg!(feature = "retail-12-0-7") {
        if let Some(frame) = sim.widgets.get_mut(id) {
            // INFERRED: public overwrite clears only this axis's origin.
            frame.secret_scroll_vertical = secret;
        }
    }
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_vertical == offset)
    {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scroll_vertical = offset;
    }
    crate::lua_api::frame::methods::widget_scroll::invalidate_scroll_presentation(&mut sim, id);
    drop(sim);
    fire_scroll_frame_event(state, id, "OnVerticalScroll", &[Val::Num(offset)])?;
    Ok(0)
}
```

### E12 — `producer` — `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs`

B36 authentication prerequisite only; no font asset/height validation or credit.

OLD:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
        let flags = val_to_string(state, stack_val(state, 5));
        set_simple_html_font(state, id, text_type, font, size, flags);
        return Ok(0);
    }
    let font = val_to_string(state, stack_val(state, 2));
    if font.is_none() {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let size = match stack_val(state, 3) {
        Val::Num(n) => Some(n as f32),
        _ => None,
    };
    let flags = val_to_string(state, stack_val(state, 4));
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get_mut(id)
        .is_some_and(|frame| apply_font_args(frame, font, size, flags));
    // Re-applying the current font leaves the text as drawn.
    if changed {
        sim.widgets.mark_visual_dirty(id);
    }
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}
```

NEW:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::frame::methods::secret_origin::authenticate_retail_arguments(state)?;
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
        let flags = val_to_string(state, stack_val(state, 5));
        set_simple_html_font(state, id, text_type, font, size, flags);
        return Ok(0);
    }
    let font = val_to_string(state, stack_val(state, 2));
    if font.is_none() {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let size = match stack_val(state, 3) {
        Val::Num(n) => Some(n as f32),
        _ => None,
    };
    let flags = val_to_string(state, stack_val(state, 4));
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get_mut(id)
        .is_some_and(|frame| apply_font_args(frame, font, size, flags));
    // Re-applying the current font leaves the text as drawn.
    if changed {
        sim.widgets.mark_visual_dirty(id);
    }
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}
```

### E13 — `producer` — `src/lua_api/globals/font_strings_collection/fonts.rs`

B36 authentication prerequisite only; no font asset/height validation or credit.

OLD:
```rust
fn font_set_font(state: &mut LuaState) -> LuaResult<u32> {
    let font = stack_val(state, 1);
    let path = Option::<String>::from_stack(state, 2)?;
    let height = Option::<f64>::from_stack(state, 3)?;
    let flags = Option::<String>::from_stack(state, 4)?;
    let Some(path) = path else { return Ok(0) };
    let path_val = create_string(state, &path);
    table_set_static(state, font, "__fontPath", path_val);
    if let Some(h) = height {
        table_set_static(state, font, "__fontHeight", Val::Num(h));
    }
    let flags_val = create_string(state, flags.as_deref().unwrap_or(""));
    table_set_static(state, font, "__fontFlags", flags_val);
    Ok(0)
}
```

NEW:
```rust
fn font_set_font(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::frame::methods::secret_origin::authenticate_retail_arguments(state)?;
    let font = stack_val(state, 1);
    let path = Option::<String>::from_stack(state, 2)?;
    let height = Option::<f64>::from_stack(state, 3)?;
    let flags = Option::<String>::from_stack(state, 4)?;
    let Some(path) = path else { return Ok(0) };
    let path_val = create_string(state, &path);
    table_set_static(state, font, "__fontPath", path_val);
    if let Some(h) = height {
        table_set_static(state, font, "__fontHeight", Val::Num(h));
    }
    let flags_val = create_string(state, flags.as_deref().unwrap_or(""));
    table_set_static(state, font, "__fontFlags", flags_val);
    Ok(0)
}
```

## Per-row evidence — 2026-10-04

All 13 IDs still `audit-pending`, capabilities `[]`, inspected without modifying the ledger. Source quotes are from the retained local excerpt, not a fresh/full Wiki capture. Cached declaration root below is `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`; cache may postdate build 68182 and cannot establish historical native policy. Current-provider line anchors below were read from working files identical to the edit base.

### `widgets-Button-GetButtonState-129`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:129`):
> Button:GetButtonState + SecretReturnsForAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:72–84`):
```lua
			Name = "GetButtonState",
			Type = "Function",
			SecretReturnsForAspect = { Enum.SecretAspect.ButtonState },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "buttonState", Type = "SimpleButtonStateToken", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:187`:
```rust
pub(super) fn get_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let pushed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|frame| frame.button_state == 1)
            .unwrap_or(false)
    };
```

Must change: E07: wrap live frame.button_state when either ButtonState origin is secret.

### `widgets-Button-IsEnabled-130`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:130`):
> Button:IsEnabled + SecretReturnsForAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:257–269`):
```lua
			Name = "IsEnabled",
			Type = "Function",
			SecretReturnsForAspect = { Enum.SecretAspect.ButtonState },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "isEnabled", Type = "bool", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:72`:
```rust
pub(super) fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).map(button_enabled).unwrap_or(true)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}
```

Must change: E04: wrap live __enabled attribute when either ButtonState origin is secret.

### `widgets-Button-SetButtonState-131`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:131`):
> Button:SetButtonState + SecretArgumentsAddAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:293–303`):
```lua
			Name = "SetButtonState",
			Type = "Function",
			SecretArgumentsAddAspect = { Enum.SecretAspect.ButtonState },
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "buttonState", Type = "SimpleButtonStateToken", Nilable = false },
				{ Name = "lock", Type = "bool", Nilable = false, Default = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:173`:
```rust
pub(super) fn set_button_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let state_name = String::from_stack(state, 2)?;
    let pushed = state_name.eq_ignore_ascii_case("PUSHED");
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.button_state = if pushed { 1 } else { 0 };
        }
```

Must change: E06: authenticate receiver, token, lock and extras before token decoding; record accepted origin.

### `widgets-Button-SetEnabled-132`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:132`):
> Button:SetEnabled + SecretArgumentsAddAspect, SecretArguments NotAllowed -> AllowedWhenUntainted

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:336–346`):
```lua
			Name = "SetEnabled",
			Type = "Function",
			IsProtectedFunction = true,
			SecretArgumentsAddAspect = { Enum.SecretAspect.ButtonState },
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "enabled", Type = "bool", Nilable = false, Default = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:98`:
```rust
pub(super) fn set_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = bool::from_stack(state, 2).ok().unwrap_or(true);
    let changed = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .map(|f| button_enabled(f) != enabled)
            .unwrap_or(false)
```

Must change: E05: authenticate all args before lookup/conversion; record origin before callbacks and on same-value calls.

### `widgets-EditBox-SetFont-133`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:133`):
> EditBox:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleEditBoxAPIDocumentation.lua:677–694`):
```lua
			Name = "SetFont",
			Type = "Function",
			RequiresValidFontAsset = true,
			RequiresValidFontHeight = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "fontFile", Type = "cstring", Nilable = false },
				{ Name = "height", Type = "uiFontHeight", Nilable = false },
				{ Name = "flags", Type = "TBFFlags", Nilable = false },
			},

			Returns =
			{
				{ Name = "success", Type = "bool", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
```

Must change: E12 authentication prerequisite only; asset and height validators BLOCKED.

### `widgets-Font-SetFont-134`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:134`):
> Font:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontAPIDocumentation.lua:199–211`):
```lua
			Name = "SetFont",
			Type = "Function",
			RequiresValidFontAsset = true,
			RequiresValidFontHeight = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "fontFile", Type = "cstring", Nilable = false },
				{ Name = "height", Type = "uiFontHeight", Nilable = false },
				{ Name = "flags", Type = "TBFFlags", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/globals/font_strings_collection/fonts.rs:102`:
```rust
fn font_set_font(state: &mut LuaState) -> LuaResult<u32> {
    let font = stack_val(state, 1);
    let path = Option::<String>::from_stack(state, 2)?;
    let height = Option::<f64>::from_stack(state, 3)?;
    let flags = Option::<String>::from_stack(state, 4)?;
    let Some(path) = path else { return Ok(0) };
    let path_val = create_string(state, &path);
    table_set_static(state, font, "__fontPath", path_val);
    if let Some(h) = height {
```

Must change: E13 authentication prerequisite only; asset and height validators BLOCKED.

### `widgets-FontString-SetFont-135`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:135`):
> FontString:SetFont + RequiresValidFontHeight + RequiresValidFontAsset, arg2.Type number -> uiFontHeight

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua:500–517`):
```lua
			Name = "SetFont",
			Type = "Function",
			RequiresValidFontAsset = true,
			RequiresValidFontHeight = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "fontFile", Type = "FontAsset", Nilable = false },
				{ Name = "fontHeight", Type = "uiFontHeight", Nilable = false },
				{ Name = "flags", Type = "TBFFlags", Nilable = true },
			},

			Returns =
			{
				{ Name = "success", Type = "bool", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
```

Must change: E12 authentication prerequisite only; FontAsset and height validators BLOCKED.

### `widgets-MessageFrame-SetFont-136`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:136`):
> MessageFrame:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleMessageFrameAPIDocumentation.lua:295–307`):
```lua
			Name = "SetFont",
			Type = "Function",
			RequiresValidFontAsset = true,
			RequiresValidFontHeight = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "fontFile", Type = "cstring", Nilable = false },
				{ Name = "height", Type = "uiFontHeight", Nilable = false },
				{ Name = "flags", Type = "TBFFlags", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
```

Must change: E12 authentication prerequisite only; asset and height validators BLOCKED.

### `widgets-ScrollFrame-GetHorizontalScroll-138`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:138`):
> ScrollFrame:GetHorizontalScroll + SecretReturnsForAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:10–22`):
```lua
			Name = "GetHorizontalScroll",
			Type = "Function",
			SecretReturnsForAspect = { Enum.SecretAspect.ScrollOffset },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "offset", Type = "uiUnit", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/widgets/slider.rs:469`:
```rust
pub(super) fn get_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_horizontal)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
```

Must change: E08: wrap live scroll_horizontal when either ScrollOffset origin is secret.

### `widgets-ScrollFrame-GetVerticalScroll-139`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:139`):
> ScrollFrame:GetVerticalScroll + SecretReturnsForAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:51–63`):
```lua
			Name = "GetVerticalScroll",
			Type = "Function",
			SecretReturnsForAspect = { Enum.SecretAspect.ScrollOffset },

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "offset", Type = "uiUnit", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/widgets/slider.rs:511`:
```rust
pub(super) fn get_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.scroll_vertical)
        .unwrap_or(0.0);
    state.push(Val::Num(offset));
    Ok(1)
```

Must change: E10: wrap live scroll_vertical when either ScrollOffset origin is secret.

### `widgets-ScrollFrame-SetHorizontalScroll-140`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:140`):
> ScrollFrame:SetHorizontalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:79–89`):
```lua
			Name = "SetHorizontalScroll",
			Type = "Function",
			IsProtectedFunction = true,
			SecretArgumentsAddAspect = { Enum.SecretAspect.ScrollOffset },
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "offset", Type = "uiUnit", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/widgets/slider.rs:480`:
```rust
pub(super) fn set_horizontal_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_horizontal == offset)
    {
```

Must change: E09: authenticate all args, commit horizontal origin before same-offset early return, retain notification ordering.

### `widgets-ScrollFrame-SetVerticalScroll-141`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:141`):
> ScrollFrame:SetVerticalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:103–113`):
```lua
			Name = "SetVerticalScroll",
			Type = "Function",
			IsProtectedFunction = true,
			SecretArgumentsAddAspect = { Enum.SecretAspect.ScrollOffset },
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "offset", Type = "uiUnit", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/widgets/slider.rs:522`:
```rust
pub(super) fn set_vertical_scroll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let offset = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if sim
        .widgets
        .get(id)
        .is_some_and(|frame| frame.scroll_vertical == offset)
    {
```

Must change: E11: authenticate all args, commit vertical origin before same-offset early return, retain notification ordering.

### `widgets-SimpleHTML-SetFont-142`

Source (`data/patch-api/sources/12.0.7-api-changes.txt:142`):
> SimpleHTML:SetFont + RequiresValidFontHeight + RequiresValidFontAsset

Cached declaration (`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleHTMLAPIDocumentation.lua:203–216`):
```lua
			Name = "SetFont",
			Type = "Function",
			RequiresValidFontAsset = true,
			RequiresValidFontHeight = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "textType", Type = "HTMLTextType", Nilable = false },
				{ Name = "fontFile", Type = "cstring", Nilable = false },
				{ Name = "height", Type = "uiFontHeight", Nilable = false },
				{ Name = "flags", Type = "TBFFlags", Nilable = false },
			},
		},
```

Exists today: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`:
```rust
pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
```

Must change: E12 authentication prerequisite only; textType overload preserved, asset and height validators BLOCKED.

## B36 re-verification: five rows remain blocked

Two distinct producer paths, not five separate setters. Frame-backed EditBox/FontString/MessageFrame/SimpleHTML use formatting.rs:330; CreateFont uses fonts.rs:102, Lua table storage with __fontPath/__fontHeight/__fontFlags. Frame setters cast arbitrary numeric heights to f32; Font stores f64. SimpleHTML uses an extra textType argument and returns no results, while the common frame branch returns one bool; preserve that distinction.

Renderer `src/render/font.rs:44–163` has a limited built-in font list, `:203–232` privately reads optional CASC/cache bytes, and `:444–450` family_name substitutes default family on misses. These are not an authoritative per-environment FontAsset validator: missing bytes/install, headless or no-CASC configuration, an unknown name, and invalid font content cannot safely be collapsed into the same invalid-asset result. A renderer-family fallback is specifically unacceptable as API validity proof. No explicit host asset-validity catalog or native height/failure policy was found.

Cached consumers that a naive validator would threaten:
- `Blizzard_Console/Blizzard_Console.lua:177–184`: `self.EditBox:SetFont(fontFile, fontHeight, fontFlags);` and `self.MessageFrame:SetFont(fontFile, fontHeight - 2, fontFlags);` use dynamic sizes, not a validated universal positive bound.
- `Blizzard_SharedXML/FontableFrameMixin.lua:20–23`: `self.fontObject:SetFont(font, fontHeight, fontFlags);` forwards caller-supplied font data. A three-font allowlist would reject otherwise legitimate custom/locale assets.
- `Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.lua:914`: `chatFrame:SetFont(fontFile, fontSize, fontFlags);` depends on inherited font data.

These do not prove native acceptance of zero, negative or arbitrary assets. They establish why guessed clamping, renderer-only availability and hard-coded allowlists cannot be introduced safely. E12/E13 authenticate all args, including ignored extras and SimpleHTML textType, before any early return/validation and preserve public behavior. They are independently correct authentication prerequisites, not satisfaction of either source annotation.

Exact per-row blocker-note text for the ledger (retain audit-pending, capabilities []; do NOT close from authentication-only tests):

`widgets-EditBox-SetFont-133`:
> BLOCKED: EditBox:SetFont RequiresValidFontAsset + RequiresValidFontHeight lack an authoritative host validator for cstring font asset and authenticated native height bounds/failure policy; the optional renderer/CASC cache and default-family fallback cannot prove asset validity without risking valid cached Blizzard consumers. All-argument AllowedWhenUntainted authentication is a prerequisite only and earns no asset/height coverage.

`widgets-Font-SetFont-134`:
> BLOCKED: Font:SetFont RequiresValidFontAsset + RequiresValidFontHeight lack an authoritative host validator for cstring font asset and authenticated native height bounds/failure policy; the optional renderer/CASC cache and default-family fallback cannot prove asset validity without risking valid cached Blizzard consumers. Font table has a zero-result contract, not the frame branch success bool. All-argument AllowedWhenUntainted authentication is a prerequisite only and earns no asset/height coverage.

`widgets-FontString-SetFont-135`:
> BLOCKED: FontString:SetFont RequiresValidFontAsset + RequiresValidFontHeight lack an authoritative host validator for FontAsset (domain unresolved; later cache differs from cstring receivers) and authenticated native height bounds/failure policy; the optional renderer/CASC cache and default-family fallback cannot prove asset validity without risking valid cached Blizzard consumers. All-argument AllowedWhenUntainted authentication is a prerequisite only and earns no asset/height coverage.

`widgets-MessageFrame-SetFont-136`:
> BLOCKED: MessageFrame:SetFont RequiresValidFontAsset + RequiresValidFontHeight lack an authoritative host validator for cstring font asset and authenticated native height bounds/failure policy; the optional renderer/CASC cache and default-family fallback cannot prove asset validity without risking valid cached Blizzard consumers. Later-cache declaration has no returns, unlike current shared frame success bool; native failure/return policy is unresolved. All-argument AllowedWhenUntainted authentication is a prerequisite only and earns no asset/height coverage.

`widgets-SimpleHTML-SetFont-142`:
> BLOCKED: SimpleHTML:SetFont RequiresValidFontAsset + RequiresValidFontHeight lack an authoritative host validator for cstring font asset and authenticated native height bounds/failure policy; the optional renderer/CASC cache and default-family fallback cannot prove asset validity without risking valid cached Blizzard consumers. SimpleHTML textType overload and zero-result contract must be preserved. All-argument AllowedWhenUntainted authentication is a prerequisite only and earns no asset/height coverage.

## Authored test inventory and integration boundaries

New full files (tag `test-support` for integration):
- `staging/p1207-b36-b38/tests/patch_12_0_7_widget_secret_aspects.rs` — nine test functions: eight enabled by retail-12-0-7, one feature-off control. One retail case includes the five-receiver B36 authentication matrix; no font-validation credit. All Lua assertions enter through WowLuaEnv::eval, use no u32 return, and mint actual secrets securely before tainted calls. Host tests mutate explicit state then assert public Lua outputs.
- `staging/p1207-b36-b38/docs/specs/widget-secret-aspects-12-0-7.md` — unverified requirements remain unchecked.

Existing tests to change: **none required by these edits**; ordinary behavior and return shapes remain unchanged. Integration must preserve these concrete regression entry points:
- `tests/methods_button.rs:435–511,617` public enabled/token round trips; `tests/methods_button/enabled_bindings.rs:32` committed-state binding order.
- `tests/scroll_widgets.rs:446,514,534` out-of-range offsets, zero ranges and same-offset dirty suppression; `tests/scroll_widgets/script_bindings.rs:47,230` payload/order and EventScrollFrame callbacks.
- `tests/font_api.rs:31,113,736,763,785` CreateFont round trip, nil path atomicity, one-bool FontString return, nilable flags and plain height. `Fonts\\Arial.ttf` at :37 is an existing noncanonical fixture, not proof of a valid native asset. Do not rewrite it to make an arbitrary font allowlist pass.
- `src/loader/tests/wow_api_globals/startup_globals.rs:396` ordinary 12.0.7 widget compatibility.

All tests/*.rs join ONE integration binary via tests/integration.rs and generated integration_tests.rs; no separate [[test]] target or integration.rs edit is proposed.

## Consumer review and exclusions

Ordinary cached consumers keep plain results: default origins are false and ordinary inputs do not add origin. Cached `Blizzard_MailFrame/MailFrame.lua:107` uses `if ( InboxNextPageButton:IsEnabled() ) then`. `Blizzard_SharedXML/Shared/InputBox/InputBoxTemplates.lua:59–70` uses `scroll = scrollFrame:GetVerticalScroll();` then `scroll = (scroll - (height / 2));`; `Blizzard_SharedXML/Shared/Scroll/ScrollUtil.lua:241` feeds the getter into onVerticalScroll. Their secret-origin paths were not executed. Pinned rilua does not implement secret-number arithmetic, so routing opaque secret offsets through InputBox would fail. Do not claim cached secret-input UI parity or patch vendor consumers. Ordinary cached paths have no new origin absent accepted secret input; runtime regressions remain untested.

Pinned rilua revision 6044544, `src/table_security.rs:92–108`, supplies trusted host bool/number/string secret wrappers usable under tainted callers. Generic wrap_secret (:80) requires secure caller and would incorrectly deny tainted getters; it is deliberately not used. unwrap_secret (:232–245) authenticates payload access without clearing caller taint.

Bounded exclusions:
1. All five B36 RequiresValidFontAsset/RequiresValidFontHeight rows remain BLOCKED. Exact per-row ledger notes above; no guessed bounds, validator, shim, fallback or font allowlist.
2. Aspect aggregation/clearing, extras origin and strict historical cache epoch are not native authenticated; inferred lifecycle/extra-origin policy is marked INFERRED in code/spec.
3. Lock behavior, missing/nil SetEnabled coercion, Enable/Disable/Click-origin clearing, protected/forbidden access control and aspect inheritance remain existing behavior, outside these added annotations.
4. Scroll callback **payload secrecy remains unresolved**: slider.rs:498,540 still passes Val::Num(offset) on accepted secret input. Generated script argument declarations were not found. Tainted callbacks can expose plain offsets; do not claim whole-path non-disclosure from getter opacity. Changing payload policy requires native/consumer evidence; proposed edits preserve callback payloads/order.
5. Auth helper implements AllowedWhenUntainted only. No requested cached setter has NeverSecret/NotAllowed argument policy. It must not be reused to permit secure secrets for such an API: those policies reject secret arguments for every caller.

## Static proof ledger and merge risk

| Scope | Read-only operation | Result | Invalidation |
|---|---|---|---|
| Base | git rev-parse master | 54799ac3e17eef168011e6692d2200c9b512a02a | Master drift requires re-anchor |
| E01–E13 | git show <base>:<path>; Python exact-match/nonoverlap inspection | 13/13 OLD blocks match once, none overlap; seven affected working files equal base | Affected-file edits invalidate anchors |
| New files | Static delimiter/count/type inspection | 9 tests, 11 raw Lua literals, no u32 eval; two staged full new files | Staged-file edits invalidate inspection |
| Compilation/runtime | Cargo/tests/builds/simulator | NOT RUN, explicitly prohibited | No passing claim exists |

The first static inspection wrongly expected nine raw literals (one per test); it reported eleven because the host-state case has three Lua assertion blocks. Corrected count only; no runtime command was rerun.

Manual Rust readability audit: named whole-call authentication step, bounded functions, no warning suppression or source-substring behavioral assertions, direct feature gate, no fallback or wrapped-constant producers. Existing full bodies are repeated solely for exact OLD anchoring. Formatting/compiler checks not run.

Merge risk today: uncompiled/unexecuted proposals, inferred lifecycle, untested cached secret-input consumers, unchanged plain scroll callback payloads, five unresolved font-validation rows. **Not merge-ready runtime proof.** Integrator owns RED/GREEN, formatting/type checks, cached consumer regressions and ledger credit.

## Final authoring status

13 rows inspected: eight B37/B38 rows have proposed live-state producers; five B36 rows have exact blocked notes and two independently useful authentication-prerequisite edits. 13 anchored edits (2 state, 1 test-support, 10 producer; final 2 are font authentication only), nine authored tests, one staged spec. Nothing integrated. No repository/git state mutated; no cargo, tests, builds, simulator, agents or model CLIs executed.

Staged `staging/p1207-b36-b38/tests/patch_12_0_7_widget_secret_aspects.rs` SHA256 `e7df3b66d96078e5bbd903cb4e4bf1c47997f5eb4d78aa8392845122e1e0a7a3`.

Staged `staging/p1207-b36-b38/docs/specs/widget-secret-aspects-12-0-7.md` SHA256 `3c47abb3a4b899485ac2579302d83004bc11384b3a231cd9f78269b05fbdaf5c`.
