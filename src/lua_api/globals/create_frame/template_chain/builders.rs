use super::{FastHandlerRef, FastLiteralValue, FastScriptInstall};
#[path = "assignment_builders.rs"]
mod assignment_builders;
#[path = "builder_function_family/mod.rs"]
mod builder_function_family;
#[path = "builder_global_family/mod.rs"]
mod builder_global_family;
#[path = "builder_method_family.rs"]
mod builder_method_family;
#[path = "register_handler_builders.rs"]
mod register_handler_builders;

pub(super) use self::assignment_builders::build_assignment_handler;
use self::assignment_builders::{
    build_ancestor_assignment_handler, build_global_assignment_handler,
    build_nested_assignment_handler, build_nested_global_pair_table_assignment_handler,
    build_parent_assignment_handler, build_set_frame_level_from_parent_handler,
};
use self::builder_function_family::build_function_family_handler;
use self::builder_global_family::build_global_family_handler;
use self::builder_method_family::build_method_family_handler;
use self::register_handler_builders::{
    build_register_for_clicks_handler, build_register_for_drag_handler,
};
use crate::lua_api::methods::create_string;
use crate::lua_api::script_helpers::{
    ScriptBinding, get_script, remove_script, set_script, set_script_binding,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Load a cached Lua chunk, converting loader errors into runtime errors.
pub(super) fn load_template(
    state: &mut LuaState,
    source: &str,
    tag: &str,
) -> LuaResult<rilua::Function> {
    let saved_slots = state.global_slots.take();
    let cache_tag = format!("{tag}-no-global-slots");
    let result = crate::loader::chunk_cache::load_chunk(state, source, &cache_tag)
        .map_err(|error| rilua::runtime_error(error.to_string()));
    state.global_slots = saved_slots;
    let function = result?;
    apply_template_factory_environment(state, &function)?;
    Ok(function)
}

pub(super) fn load_constant_template(
    state: &mut LuaState,
    source: &'static str,
    tag: &str,
) -> LuaResult<rilua::Function> {
    let saved_slots = state.global_slots.take();
    let cache_tag = format!("{tag}-no-global-slots");
    let result = crate::loader::chunk_cache::load_constant_chunk(state, source, &cache_tag)
        .map_err(|error| rilua::runtime_error(error.to_string()));
    state.global_slots = saved_slots;
    let function = result?;
    apply_template_factory_environment(state, &function)?;
    Ok(function)
}

fn apply_template_factory_environment(
    state: &mut LuaState,
    function: &rilua::Function,
) -> LuaResult<()> {
    let secure = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        sim.loading_addon_index
            .and_then(|index| sim.addons.get(index as usize))
            .is_some_and(|addon| addon.use_secure_env)
    };
    if secure {
        crate::lua_api::globals::security::mark_secure_state(state, function)?;
    }
    crate::lua_api::loader_env::apply_loading_scoped_fenv_state(state, function)
        .map_err(|error| rilua::runtime_error(error.to_string()))
}

fn build_set_alpha_handler(state: &mut LuaState, alpha: f64) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local alpha = ...
            return function(self, ...)
                return self:SetAlpha(alpha)
            end
        "#,
        "template-set-alpha-handler",
    )?;
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[Val::Num(alpha)],
    )
}

pub(super) fn build_fast_handler(
    state: &mut LuaState,
    handler_ref: FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    if let FastHandlerRef::InlineMethod { handler_name, body } = handler_ref {
        let params = crate::loader::helpers::handler_params(handler_name);
        let locals = crate::loader::helpers::inline_argument_locals(handler_name);
        let source = format!("return function({params}) {locals}\n{body}\nend");
        let builder = load_template(state, &source, "template-inline-method")?;
        return crate::lua_api::methods::call_function_state(
            state,
            Val::Function(builder.gc_ref()),
            &[],
        )
        .map(Some);
    }
    if let Some(result) = try_family_dispatchers(state, &handler_ref)? {
        return Ok(result);
    }
    build_terminal_fast_handler(state, handler_ref)
}

/// Try each family dispatcher in turn; return `Some(outer)` on first
/// match (where the inner option encodes NoOp-style non-results).
fn try_family_dispatchers(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    if let Some(result) = build_sequence_fast_handler(state, handler_ref)? {
        return Ok(Some(result));
    }
    if let Some(result) = build_method_family_handler(state, handler_ref)? {
        return Ok(Some(Some(result)));
    }
    if let Some(result) = build_global_family_handler(state, handler_ref)? {
        return Ok(Some(Some(result)));
    }
    if let Some(result) = build_function_family_handler(state, handler_ref)? {
        return Ok(Some(Some(result)));
    }
    Ok(None)
}

/// Final match for variants that don't belong to a family dispatcher
/// (click/drag registration, alpha/frame-level setters, assignments).
fn build_terminal_fast_handler(
    state: &mut LuaState,
    handler_ref: FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    if let Some(result) = try_terminal_non_assignment_handler(state, &handler_ref)? {
        return Ok(result);
    }
    if let Some(result) = try_terminal_direct_assignment_handler(state, &handler_ref)? {
        return Ok(result);
    }
    if let Some(result) = try_terminal_assignment_handler(state, &handler_ref)? {
        return Ok(result);
    }
    match handler_ref {
        FastHandlerRef::NoOp => Ok(None),
        _ => unreachable!(
            "FastHandlerRef variant not dispatched by any family handler or the terminal match; \
             this is a bug — every variant must be handled"
        ),
    }
}

fn try_terminal_non_assignment_handler(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::RegisterForClicks {
            first,
            second,
            third,
        } => build_register_for_clicks_handler(state, first, *second, *third)
            .map(Some)
            .map(Some),
        FastHandlerRef::RegisterForDrag(button) => build_register_for_drag_handler(state, button)
            .map(Some)
            .map(Some),
        FastHandlerRef::SetAlpha(alpha) => {
            build_set_alpha_handler(state, *alpha).map(Some).map(Some)
        }
        FastHandlerRef::SetFrameLevelFromParent(delta) => {
            build_set_frame_level_from_parent_handler(state, *delta)
                .map(Some)
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn try_terminal_direct_assignment_handler(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::AssignAncestorRef { field, depth } => {
            build_ancestor_assignment_handler(state, field, *depth)
                .map(Some)
                .map(Some)
        }
        FastHandlerRef::AssignLiteral { field, value } => {
            build_assignment_handler(state, field, *value)
                .map(Some)
                .map(Some)
        }
        FastHandlerRef::AssignGlobalFieldLiteral {
            target_path,
            field,
            value,
        } => build_global_assignment_handler(state, target_path, field, *value)
            .map(Some)
            .map(Some),
        _ => Ok(None),
    }
}

fn try_terminal_assignment_handler(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::AssignNestedLiteral {
            parent_field,
            field,
            value,
        } => build_nested_assignment_handler(state, parent_field, field, *value)
            .map(Some)
            .map(Some),
        FastHandlerRef::AssignNestedGlobalPairTable {
            parent_field,
            field,
            first_path,
            second_path,
        } => build_nested_global_pair_table_assignment_handler(
            state,
            parent_field,
            field,
            first_path,
            second_path,
        )
        .map(Some)
        .map(Some),
        FastHandlerRef::AssignParentField { field, value } => {
            build_parent_assignment_handler(state, field, *value)
                .map(Some)
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn build_sequence_fast_handler(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    if let Some(result) = try_sequence_pair_variant(state, handler_ref)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_sequence_triple_variant(state, handler_ref)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_sequence_quad_variant(state, handler_ref)? {
        return Ok(Some(result));
    }
    try_sequence_conditional_handler(state, handler_ref)
}

fn try_sequence_pair_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::Sequence2(parts) => {
            let (first_ref, second_ref) = &**parts;
            let first = build_fast_handler(state, first_ref.clone())?;
            let second = build_fast_handler(state, second_ref.clone())?;
            chain_optional_handlers(state, first, second).map(Some)
        }
        _ => Ok(None),
    }
}

fn try_sequence_triple_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::Sequence3(parts) => {
            let (first_ref, second_ref, third_ref) = &**parts;
            let first = build_fast_handler(state, first_ref.clone())?;
            let second = build_fast_handler(state, second_ref.clone())?;
            let third = build_fast_handler(state, third_ref.clone())?;
            let first_pair = chain_optional_handlers(state, first, second)?;
            chain_optional_handlers(state, first_pair, third).map(Some)
        }
        _ => Ok(None),
    }
}

fn try_sequence_quad_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    match handler_ref {
        FastHandlerRef::Sequence4(parts) => {
            let (first_ref, second_ref, third_ref, fourth_ref) = &**parts;
            let first = build_fast_handler(state, first_ref.clone())?;
            let second = build_fast_handler(state, second_ref.clone())?;
            let third = build_fast_handler(state, third_ref.clone())?;
            let fourth = build_fast_handler(state, fourth_ref.clone())?;
            let first_pair = chain_optional_handlers(state, first, second)?;
            let first_triplet = chain_optional_handlers(state, first_pair, third)?;
            chain_optional_handlers(state, first_triplet, fourth).map(Some)
        }
        _ => Ok(None),
    }
}

fn try_sequence_conditional_handler(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    if let Some(result) = try_conditional_global_field_equals_variant(state, handler_ref)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_conditional_self_noargs_variant(state, handler_ref)? {
        return Ok(Some(result));
    }
    try_conditional_self_field_variant(state, handler_ref)
}

fn try_conditional_global_field_equals_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    let FastHandlerRef::ConditionalGlobalFieldEqualsStringThen {
        target_path,
        field,
        value,
        then_ref,
    } = handler_ref
    else {
        return Ok(None);
    };
    build_conditional_global_field_equals_handler(state, target_path, field, value, then_ref)
        .map(Some)
}

fn try_conditional_self_noargs_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    let FastHandlerRef::ConditionalSelfNoArgsMethod {
        method_name,
        then_ref,
        else_ref,
    } = handler_ref
    else {
        return Ok(None);
    };
    build_conditional_self_noargs_handler(state, method_name, then_ref, else_ref).map(Some)
}

fn try_conditional_self_field_variant(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Option<Val>>> {
    let FastHandlerRef::ConditionalSelfFieldTruthy {
        field,
        then_ref,
        else_ref,
    } = handler_ref
    else {
        return Ok(None);
    };
    build_conditional_self_field_handler(state, field, then_ref, else_ref).map(Some)
}

fn build_conditional_global_field_equals_handler(
    state: &mut LuaState,
    target_path: &str,
    field: &str,
    value: &str,
    then_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    let then_handler = build_fast_handler(state, then_ref.clone())?;
    let target =
        crate::lua_api::globals::create_frame::helpers::resolve_global_path(state, target_path);
    let field = create_string(state, field);
    let value = create_string(state, value);
    let builder = load_template(
        state,
        r#"
            local target, field, value, then_handler = ...
            return function(self, ...)
                if not target or target[field] ~= value then
                    return
                end
                if then_handler then
                    return then_handler(self, ...)
                end
            end
        "#,
        "template-inline-conditional-global-field-equals-string-then",
    )?;
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[target, field, value, then_handler.unwrap_or(Val::Nil)],
    )
    .map(Some)
}

fn build_conditional_self_noargs_handler(
    state: &mut LuaState,
    method_name: &str,
    then_ref: &FastHandlerRef<'_>,
    else_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    let then_handler = build_fast_handler(state, then_ref.clone())?;
    let else_handler = build_fast_handler(state, else_ref.clone())?;
    let method_name = create_string(state, method_name);
    let builder = load_template(
        state,
        TEMPLATE_CONDITIONAL_SELF_NOARGS,
        "template-inline-conditional-self-noargs",
    )?;
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[
            method_name,
            then_handler.unwrap_or(Val::Nil),
            else_handler.unwrap_or(Val::Nil),
        ],
    )
    .map(Some)
}

const TEMPLATE_CONDITIONAL_SELF_NOARGS: &str = r#"
    local method_name, then_handler, else_handler = ...
    return function(self, ...)
        if self[method_name](self) then
            if then_handler then
                return then_handler(self, ...)
            end
            return
        end
        if else_handler then
            return else_handler(self, ...)
        end
    end
"#;

fn build_conditional_self_field_handler(
    state: &mut LuaState,
    field: &str,
    then_ref: &FastHandlerRef<'_>,
    else_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    let then_handler = build_fast_handler(state, then_ref.clone())?;
    let else_handler = build_fast_handler(state, else_ref.clone())?;
    let field = create_string(state, field);
    let builder = load_template(
        state,
        TEMPLATE_CONDITIONAL_SELF_FIELD,
        "template-inline-conditional-self-field",
    )?;
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[
            field,
            then_handler.unwrap_or(Val::Nil),
            else_handler.unwrap_or(Val::Nil),
        ],
    )
    .map(Some)
}

const TEMPLATE_CONDITIONAL_SELF_FIELD: &str = r#"
    local field, then_handler, else_handler = ...
    return function(self, ...)
        if self[field] then
            if then_handler then
                return then_handler(self, ...)
            end
            return
        end
        if else_handler then
            return else_handler(self, ...)
        end
    end
"#;

fn chain_optional_handlers(
    state: &mut LuaState,
    first: Option<Val>,
    second: Option<Val>,
) -> LuaResult<Option<Val>> {
    match (first, second) {
        (Some(first), Some(second)) => {
            build_chained_handler(state, first, second, "inline-sequence", false).map(Some)
        }
        (Some(first), None) => Ok(Some(first)),
        (None, Some(second)) => Ok(Some(second)),
        (None, None) => Ok(None),
    }
}

pub(super) fn install_fast_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &'static str,
    install: FastScriptInstall<'_>,
) -> LuaResult<()> {
    match install {
        FastScriptInstall::Set(handler_ref) => {
            install_set_handler(state, frame_id, handler_name, handler_ref)?
        }
        FastScriptInstall::Intrinsic { handler, new_first } => {
            install_intrinsic_handler(state, frame_id, handler_name, handler, new_first)?
        }
        FastScriptInstall::Chain { handler, new_first } => {
            install_chained_handler(state, frame_id, handler_name, handler, new_first)?
        }
    }
    Ok(())
}

fn install_set_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &'static str,
    handler_ref: FastHandlerRef<'_>,
) -> LuaResult<()> {
    if matches!(handler_ref, FastHandlerRef::NoOp) {
        remove_script(state, frame_id, handler_name);
        return Ok(());
    }
    if let Some(handler) = build_fast_handler(state, handler_ref)? {
        set_script(state, frame_id, handler_name, handler);
    }
    Ok(())
}

fn install_intrinsic_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &'static str,
    handler_ref: FastHandlerRef<'_>,
    new_first: bool,
) -> LuaResult<()> {
    let binding = if new_first {
        ScriptBinding::Precall
    } else {
        ScriptBinding::Postcall
    };
    if matches!(handler_ref, FastHandlerRef::NoOp) {
        crate::lua_api::script_helpers::remove_script_binding(
            state,
            frame_id,
            handler_name,
            binding,
        );
        return Ok(());
    }
    if let Some(handler) = build_fast_handler(state, handler_ref)? {
        set_script_binding(state, frame_id, handler_name, binding, handler);
    }
    Ok(())
}

fn install_chained_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler_name: &'static str,
    handler_ref: FastHandlerRef<'_>,
    new_first: bool,
) -> LuaResult<()> {
    if matches!(handler_ref, FastHandlerRef::NoOp) {
        remove_script(state, frame_id, handler_name);
        return Ok(());
    }
    let Some(new_handler) = build_fast_handler(state, handler_ref)? else {
        return Ok(());
    };
    let Some(old_handler) = get_script(state, frame_id, handler_name) else {
        set_script(state, frame_id, handler_name, new_handler);
        return Ok(());
    };
    let chained = build_chained_handler(state, old_handler, new_handler, handler_name, new_first)?;
    set_script(state, frame_id, handler_name, chained);
    Ok(())
}

pub(super) fn build_chained_handler(
    state: &mut LuaState,
    old_handler: Val,
    new_handler: Val,
    handler_name: &str,
    new_first: bool,
) -> LuaResult<Val> {
    let (first, second) = if new_first {
        (new_handler, old_handler)
    } else {
        (old_handler, new_handler)
    };
    // Root both handlers before any allocation so the returned closure cannot
    // capture a dangling function ref in its upvalues.
    let stack_slot = root_vals_on_stack(state, first, second);
    let result = invoke_chained_template(state, handler_name, first, second);
    state.top = stack_slot;
    result
}

/// Push two values onto the Lua stack to root them for GC safety.
/// Returns the original stack top so the caller can restore it.
fn root_vals_on_stack(state: &mut LuaState, first: Val, second: Val) -> usize {
    let stack_slot = state.top;
    state.ensure_stack(stack_slot + 2);
    state.stack_set(stack_slot, first);
    state.stack_set(stack_slot + 1, second);
    state.top = stack_slot + 2;
    stack_slot
}

/// Load the chained-handler template and call it with `(handler_name, first, second)`.
fn invoke_chained_template(
    state: &mut LuaState,
    handler_name: &str,
    first: Val,
    second: Val,
) -> LuaResult<Val> {
    let (source, tag) = chained_handler_template();
    let builder = load_template(state, source, tag)?;
    let handler_name = create_string(state, handler_name);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[handler_name, first, second],
    )
}

fn chained_handler_template() -> (&'static str, &'static str) {
    (
        r#"
            local handler_name, first, second = ...
            local report = debug.getregistry()["__report_script_error"]
            return function(self, ...)
                if securecall then
                    securecall(first, self, ...)
                    securecall(second, self, ...)
                else
                    local ok1, err1 = pcall(first, self, ...)
                    local ok2, err2 = pcall(second, self, ...)
                    if not ok1 then
                        local name = self.GetName and self:GetName() or "?"
                        report("[script:" .. handler_name .. "] " .. name .. ": " .. tostring(err1))
                    end
                    if not ok2 then
                        local name = self.GetName and self:GetName() or "?"
                        report("[script:" .. handler_name .. "] " .. name .. ": " .. tostring(err2))
                    end
                end
            end
        "#,
        "template-chained-handler",
    )
}
