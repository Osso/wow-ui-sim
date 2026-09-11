//! XML animation type definitions.

use serde::Deserialize;

use super::types_support::{KeyValuesXml, ScriptsXml};

/// Animation group definition.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnimationGroupXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@parentArray")]
    pub parent_array: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@mixin")]
    pub mixin: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@setToFinalAlpha")]
    pub set_to_final_alpha: Option<bool>,
    #[serde(rename = "@looping")]
    pub looping: Option<String>,
    #[serde(rename = "$value", default)]
    pub elements: Vec<AnimationElement>,
}

/// Elements that can appear inside an AnimationGroup.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum AnimationElement {
    Animation(AnimationXml),
    Alpha(AnimationXml),
    Translation(AnimationXml),
    LineTranslation(AnimationXml),
    Rotation(AnimationXml),
    Scale(AnimationXml),
    LineScale(AnimationXml),
    Path(AnimationXml),
    FlipBook(AnimationXml),
    VertexColor(AnimationXml),
    TextureCoordTranslation(AnimationXml),
    Scripts(Box<ScriptsXml>),
    KeyValues(KeyValuesXml),
    #[serde(other)]
    Unknown,
}

/// Common and type-specific animation attributes retained for shared XML application.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AnimationXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "Scripts")]
    pub scripts: Option<Box<ScriptsXml>>,
    #[serde(rename = "KeyValues")]
    pub key_values: Option<KeyValuesXml>,
    #[serde(rename = "@parentKey", alias = "@parentkey")]
    pub parent_key: Option<String>,
    #[serde(rename = "@childKey")]
    pub child_key: Option<String>,
    #[serde(rename = "@target")]
    pub target: Option<String>,
    #[serde(rename = "@targetKey")]
    pub target_key: Option<String>,
    #[serde(rename = "@duration")]
    pub duration: Option<f32>,
    #[serde(rename = "@order")]
    pub order: Option<u32>,
    #[serde(rename = "@startDelay")]
    pub start_delay: Option<f32>,
    #[serde(rename = "@endDelay")]
    pub end_delay: Option<f32>,
    #[serde(rename = "@smoothing")]
    pub smoothing: Option<String>,
    // Alpha
    #[serde(rename = "@fromAlpha")]
    pub from_alpha: Option<f32>,
    #[serde(rename = "@toAlpha")]
    pub to_alpha: Option<f32>,
    // Translation
    #[serde(rename = "@offsetX")]
    pub offset_x: Option<f32>,
    #[serde(rename = "@offsetY")]
    pub offset_y: Option<f32>,
    // Scale
    #[serde(rename = "@scaleX")]
    pub scale_x: Option<f32>,
    #[serde(rename = "@scaleY")]
    pub scale_y: Option<f32>,
    #[serde(rename = "@fromScaleX")]
    pub from_scale_x: Option<f32>,
    #[serde(rename = "@fromScaleY")]
    pub from_scale_y: Option<f32>,
    #[serde(rename = "@toScaleX")]
    pub to_scale_x: Option<f32>,
    #[serde(rename = "@toScaleY")]
    pub to_scale_y: Option<f32>,
    // Rotation
    #[serde(rename = "@degrees")]
    pub degrees: Option<f32>,
    #[serde(rename = "@radians")]
    pub radians: Option<f32>,
    // FlipBook
    #[serde(rename = "@flipBookRows")]
    pub flip_book_rows: Option<u32>,
    #[serde(rename = "@flipBookColumns")]
    pub flip_book_columns: Option<u32>,
    #[serde(rename = "@flipBookFrames")]
    pub flip_book_frames: Option<u32>,
    // TextureCoordTranslation
    #[serde(rename = "@offsetU")]
    pub offset_u: Option<f32>,
    #[serde(rename = "@offsetV")]
    pub offset_v: Option<f32>,
    // Path
    #[serde(rename = "@curve")]
    pub curve: Option<String>,
    // Child elements; support depends on the corresponding simulator animation methods.
    #[serde(rename = "Origin")]
    pub origin: Option<OriginXml>,
    #[serde(rename = "ControlPoints")]
    pub control_points: Option<ControlPointsXml>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct OriginXml {
    #[serde(rename = "@point")]
    pub point: Option<String>,
    #[serde(rename = "@x")]
    pub x: Option<f32>,
    #[serde(rename = "@y")]
    pub y: Option<f32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ControlPointsXml {
    #[serde(rename = "ControlPoint", default)]
    pub points: Vec<ControlPointXml>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ControlPointXml {
    #[serde(rename = "@offsetX")]
    pub offset_x: Option<f32>,
    #[serde(rename = "@offsetY")]
    pub offset_y: Option<f32>,
}
