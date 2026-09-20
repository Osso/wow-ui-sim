use super::types::{FrameXml, XmlElement};
use super::types_elements::FrameElement;

macro_rules! frame_variant_data {
    ($self:expr, $($variant:ident),+ $(,)?) => {
        match $self {
            $(Self::$variant(f) => Some((f, stringify!($variant))),)+
            _ => None,
        }
    };
}

macro_rules! impl_frame_data_for {
    ($type:ty, $non_frame_note:literal) => {
        impl $type {
            /// Extract the inner `FrameXml` and variant tag name for frame-like elements.
            #[doc = $non_frame_note]
            pub fn as_frame_data(&self) -> Option<(&FrameXml, &'static str)> {
                self.as_primary_frame_data()
                    .or_else(|| self.as_secondary_frame_data())
            }

            fn as_primary_frame_data(&self) -> Option<(&FrameXml, &'static str)> {
                #[cfg(feature = "aura-xml-widgets")]
                if let Some(frame_data) =
                    frame_variant_data!(self, AuraContainer, ManagedAuraContainer, AuraButton,)
                {
                    return Some(frame_data);
                }

                frame_variant_data!(
                    self,
                    Frame,
                    Button,
                    ItemButton,
                    CheckButton,
                    EditBox,
                    ScrollFrame,
                    Slider,
                    StatusBar,
                    GameTooltip,
                    ColorSelect,
                    Model,
                    ModelScene,
                    EventFrame,
                    CinematicModel,
                    PlayerModel,
                    DressUpModel
                )
            }

            fn as_secondary_frame_data(&self) -> Option<(&FrameXml, &'static str)> {
                frame_variant_data!(
                    self,
                    Browser,
                    Minimap,
                    MessageFrame,
                    MovieFrame,
                    ScrollingMessageFrame,
                    SimpleHTML,
                    WorldFrame,
                    DropDownToggleButton,
                    DropdownButton,
                    EventButton,
                    EventEditBox,
                    Cooldown,
                    TaxiRouteFrame,
                    ModelFFX,
                    TabardModel,
                    UiCamera,
                    UnitPositionFrame,
                    OffScreenFrame,
                    Checkout,
                    FogOfWarFrame,
                    QuestPOIFrame,
                    ArchaeologyDigSiteFrame,
                    ScenarioPOIFrame,
                    UIThemeContainerFrame,
                    EventScrollFrame,
                    ContainedAlertFrame,
                    MapScene,
                    Line
                )
            }
        }
    };
}

impl_frame_data_for!(
    XmlElement,
    "Returns `None` for non-frame elements (Script, Include, Font, etc.)."
);
impl_frame_data_for!(
    FrameElement,
    "Returns `None` for `ScopedModifier` which has no `FrameXml`."
);

#[cfg(test)]
mod tests {
    use super::super::types::ScopedModifierXml;
    use super::*;

    #[test]
    fn xml_element_frame_data_returns_frame_and_tag() {
        let frame = FrameXml {
            name: Some("MapSceneFrame".to_string()),
            ..FrameXml::default()
        };

        let element = XmlElement::MapScene(frame);
        let (frame_data, tag) = element
            .as_frame_data()
            .expect("MapScene should be frame-like");

        assert_eq!(tag, "MapScene");
        assert_eq!(frame_data.name.as_deref(), Some("MapSceneFrame"));
    }

    #[test]
    fn xml_element_frame_data_ignores_non_frame_variants() {
        assert!(XmlElement::Unknown.as_frame_data().is_none());
    }

    #[test]
    fn frame_element_frame_data_returns_frame_and_tag() {
        let frame = FrameXml {
            parent_key: Some("ScrollChild".to_string()),
            ..FrameXml::default()
        };

        let element = FrameElement::EventScrollFrame(frame);
        let (frame_data, tag) = element
            .as_frame_data()
            .expect("EventScrollFrame should be frame-like");

        assert_eq!(tag, "EventScrollFrame");
        assert_eq!(frame_data.parent_key.as_deref(), Some("ScrollChild"));
    }

    #[test]
    fn frame_element_frame_data_returns_primary_widget_tag() {
        let frame = FrameXml {
            name: Some("ActionButton".to_string()),
            ..FrameXml::default()
        };

        let element = FrameElement::Button(frame);
        let (frame_data, tag) = element
            .as_frame_data()
            .expect("Button should be frame-like");

        assert_eq!(tag, "Button");
        assert_eq!(frame_data.name.as_deref(), Some("ActionButton"));
    }

    #[test]
    fn frame_element_frame_data_ignores_scoped_modifier() {
        let scoped = ScopedModifierXml {
            forbidden: Some(true),
            scripts_use_given_env: None,
            add_to_secure_env: None,
            hide_from_global_env: None,
            full_lockdown: None,
            use_forbidden_object_table: None,
            elements: Vec::new(),
        };

        assert!(
            FrameElement::ScopedModifier(scoped)
                .as_frame_data()
                .is_none()
        );
    }
}
