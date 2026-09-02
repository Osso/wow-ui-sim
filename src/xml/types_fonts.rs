use serde::Deserialize;

/// Font definition.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@inherits")]
    pub inherits: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "@font")]
    pub font: Option<String>,
    #[serde(rename = "@height")]
    pub height: Option<f32>,
    #[serde(rename = "@outline")]
    pub outline: Option<String>,
    #[serde(rename = "@justifyH")]
    pub justify_h: Option<String>,
    #[serde(rename = "@justifyV")]
    pub justify_v: Option<String>,
    /// `<Shadow>` child: offset and colour. Blizzard's shadowed system fonts
    /// (`SystemFont_Shadow_*`) carry it here, and every `GameFont*` inherits
    /// from one of them, so dropping it flattens nearly all UI text.
    #[serde(rename = "Shadow")]
    pub shadow: Option<crate::xml::ShadowXml>,
    /// `<Color>` child: the text colour every FontString inheriting the font
    /// starts with. `GameFontNormal` and its small/large siblings carry
    /// `color="NORMAL_FONT_COLOR"`, the number fonts explicit r/g/b.
    #[serde(rename = "Color")]
    pub color: Option<crate::xml::ColorXml>,
}

/// FontFamily definition - collection of fonts for different alphabets.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontFamilyXml {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "Member", default)]
    pub members: Vec<FontFamilyMemberXml>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FontFamilyMemberXml {
    #[serde(rename = "@alphabet")]
    pub alphabet: Option<String>,
    #[serde(rename = "Font")]
    pub font: Option<FontXml>,
}
