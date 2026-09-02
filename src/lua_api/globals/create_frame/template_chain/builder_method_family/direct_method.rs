use super::super::{FastHandlerRef, load_template};
use crate::lua_api::methods::create_string;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// `self[method](self, ...)` shapes.
pub(super) fn build_direct_method_variants(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    if let Some(result) = build_direct_conditional_variants(state, handler_ref)? {
        return Ok(Some(result));
    }
    build_direct_method_call_variants(state, handler_ref)
}

fn build_direct_conditional_variants(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    match handler_ref {
        FastHandlerRef::ConditionalSelfTextEmptyShowTextChild => {
            build_conditional_self_text_empty_show_text_child_handler(state).map(Some)
        }
        FastHandlerRef::MethodThenUncheckedParentFieldClearAndShowText { method_name, field } => {
            build_method_then_unchecked_parent_field_clear_and_show_text_handler(
                state,
                method_name,
                field,
            )
            .map(Some)
        }
        FastHandlerRef::ConditionalSelfGetTextNonEmptyThenParentMethodWithSelfGetTextAndClear {
            method_name,
        } => build_conditional_self_get_text_non_empty_then_parent_method_with_self_get_text_and_clear_handler(state, method_name)
            .map(Some),
        FastHandlerRef::ConditionalNotSelfNoArgsMethodThen {
            method_name,
            then_ref,
        } => {
            let then_handler = super::super::build_fast_handler(state, (**then_ref).clone())?;
            build_conditional_not_self_noargs_method_then_handler(state, method_name, then_handler)
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn build_direct_method_call_variants(
    state: &mut LuaState,
    handler_ref: &FastHandlerRef<'_>,
) -> LuaResult<Option<Val>> {
    match handler_ref {
        FastHandlerRef::Method(method_name) => build_method_handler(state, method_name).map(Some),
        FastHandlerRef::MethodWithBoolArg { method_name, value } => {
            build_method_with_value_arg_handler(state, method_name, Val::Bool(*value)).map(Some)
        }
        FastHandlerRef::MethodWithNumberArg { method_name, value } => {
            build_method_with_value_arg_handler(state, method_name, Val::Num(*value)).map(Some)
        }
        FastHandlerRef::MethodWithTwoNumberArgs {
            method_name,
            first,
            second,
        } => {
            build_method_with_two_number_args_handler(state, method_name, *first, *second).map(Some)
        }
        FastHandlerRef::MethodWithStringArg { method_name, arg } => {
            build_method_with_string_arg_handler(state, method_name, arg).map(Some)
        }
        _ => Ok(None),
    }
}

fn build_conditional_self_text_empty_show_text_child_handler(
    state: &mut LuaState,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            return function(self, ...)
                if self:GetText() == "" and self.Text then
                    return self.Text:Show()
                end
            end
        "#,
        "template-conditional-self-text-empty-show-text-child",
    )?;
    crate::lua_api::methods::call_function_state(state, Val::Function(builder.gc_ref()), &[])
}

fn build_conditional_self_get_text_non_empty_then_parent_method_with_self_get_text_and_clear_handler(
    state: &mut LuaState,
    method_name: &str,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name = ...
            return function(self, ...)
                local text = self:GetText()
                if text and #text > 0 then
                    local parent = self:GetParent()
                    __wow_bind_xml_method(parent, method_name)(parent, self:GetText())
                    return self:SetText("")
                end
            end
        "#,
        "template-conditional-self-get-text-non-empty-then-parent-method-with-self-get-text-and-clear",
    )?;
    let method_name = create_string(state, method_name);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name],
    )
}

fn build_conditional_not_self_noargs_method_then_handler(
    state: &mut LuaState,
    method_name: &str,
    then_handler: Option<Val>,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name, then_handler = ...
            return function(self, ...)
                if not __wow_bind_xml_method(self, method_name)(self) then
                    if then_handler then
                        return then_handler(self, ...)
                    end
                end
            end
        "#,
        "template-conditional-not-self-noargs-method-then",
    )?;
    let method_name = create_string(state, method_name);
    let then_handler = then_handler.unwrap_or(Val::Nil);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name, then_handler],
    )
}

fn build_method_then_unchecked_parent_field_clear_and_show_text_handler(
    state: &mut LuaState,
    method_name: &str,
    field: &str,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        TEMPLATE_METHOD_UNCHECKED_PARENT_FIELD_CLEAR_SHOW_TEXT,
        "template-method-unchecked-parent-field-clear-show-text",
    )?;
    let method_name = create_string(state, method_name);
    let field_name = create_string(state, field);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name, field_name],
    )
}

const TEMPLATE_METHOD_UNCHECKED_PARENT_FIELD_CLEAR_SHOW_TEXT: &str = r#"
    local method_name, field_name = ...
    return function(self, ...)
        __wow_bind_xml_method(self, method_name)(self, ...)
        if self:GetChecked() then
            return
        end
        local parent = self:GetParent()
        if not parent then
            return
        end
        local target = parent[field_name]
        if not target then
            return
        end
        target:SetText("")
        if target.Text then
            target.Text:Show()
        end
    end
"#;

fn build_method_with_value_arg_handler(
    state: &mut LuaState,
    method_name: &str,
    value: Val,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name, value = ...
            return function(self, ...)
                local method = self[method_name]
                if method ~= nil then
                    return method(self, value)
                end
                return __wow_call_xml_method_fallback(self, method_name, value)
            end
        "#,
        "template-method-value-arg",
    )?;
    let method_name = create_string(state, method_name);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name, value],
    )
}

fn build_method_with_two_number_args_handler(
    state: &mut LuaState,
    method_name: &str,
    first: f64,
    second: f64,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name, first, second = ...
            return function(self, ...)
                local method = self[method_name]
                if method ~= nil then
                    return method(self, first, second)
                end
                return __wow_call_xml_method_fallback(self, method_name, first, second)
            end
        "#,
        "template-method-two-number-args",
    )?;
    let method_name = create_string(state, method_name);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name, Val::Num(first), Val::Num(second)],
    )
}

/// `<OnLoad method="X"/>` and friends. The public object is tried first
/// (no global lookup on the hot path); a miss goes through the shared
/// bootstrap's fallback, which finds methods a secure mixin installed on the
/// frame's forbidden partition (the unit-frame aura containers keep their
/// OnLoad / OnUpdate there) and otherwise calls the public method as it
/// stands now, a no-op when neither surface has it.
fn build_method_handler(state: &mut LuaState, method_name: &str) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name = ...
            return function(self, ...)
                local method = self[method_name]
                if method ~= nil then
                    return method(self, ...)
                end
                return __wow_call_xml_method_fallback(self, method_name, ...)
            end
        "#,
        "template-method-handler",
    )?;
    let method_name = create_string(state, method_name);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name],
    )
}

fn build_method_with_string_arg_handler(
    state: &mut LuaState,
    method_name: &str,
    arg: &str,
) -> LuaResult<Val> {
    let builder = load_template(
        state,
        r#"
            local method_name, literal_arg = ...
            return function(self, ...)
                local method = self[method_name]
                if method ~= nil then
                    return method(self, literal_arg)
                end
                return __wow_call_xml_method_fallback(self, method_name, literal_arg)
            end
        "#,
        "template-method-string-handler",
    )?;
    let method_name = create_string(state, method_name);
    let literal_arg = create_string(state, arg);
    crate::lua_api::methods::call_function_state(
        state,
        Val::Function(builder.gc_ref()),
        &[method_name, literal_arg],
    )
}
