//! Host-driven URL texture notifications using real texture receiver identity.

use super::WowLuaEnv;
use crate::lua_api::methods::frame_ref;
use crate::widget::WidgetType;
use rilua::{LuaApiMut, Val};

impl WowLuaEnv {
    /// INFERRED: consume one host notification before synchronous listener dispatch.
    /// No request, network, cancellation, texture pixels or native restrictions inferred.
    pub fn publish_next_url_texture_result(&self) -> crate::Result<bool> {
        let notification = self
            .state
            .borrow_mut()
            .url_texture_inputs
            .pending
            .pop_front();
        let Some(notification) = notification else {
            return Ok(false);
        };
        self.publish_url_texture_notification(&notification)?;
        Ok(true)
    }

    fn publish_url_texture_notification(
        &self,
        notification: &crate::c_api::url_texture_inputs::UrlTextureNotification,
    ) -> crate::Result<()> {
        let is_texture = self
            .state
            .borrow()
            .widgets
            .get(notification.texture_id)
            .is_some_and(|frame| frame.widget_type == WidgetType::Texture);
        if !is_texture {
            return Err(rilua::runtime_error(
                "URL texture notification requires an existing texture",
            )
            .into());
        }
        let (saved_top, texture) = {
            let mut lua = self.lua.borrow_mut();
            let state = lua.state_mut();
            let saved_top = state.top;
            let texture = frame_ref(state, notification.texture_id)?;
            state.push(texture);
            (saved_top, texture)
        };
        let result = self.fire_event_with_args(
            "URL_TEXTURE_REQUEST_RESULT",
            &[texture, Val::Num(notification.result as u8 as f64)],
        );
        self.lua.borrow_mut().state_mut().top = saved_top;
        result
    }
}
