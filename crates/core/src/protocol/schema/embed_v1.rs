use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct EmbedDocument {
    #[serde(rename = "__typename")]
    pub type_name: Option<String>,
    #[serde(rename = "mediaDetails", default)]
    pub media_details: Vec<EmbedMedia>,
    pub quoted_tweet: Option<Box<EmbedDocument>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmbedMedia {
    #[serde(rename = "media_url_https")]
    pub media_url_https: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub original_info: Option<EmbedOriginalInfo>,
    pub video_info: Option<EmbedVideoInfo>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmbedOriginalInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmbedVideoInfo {
    pub duration_millis: Option<u64>,
    #[serde(default)]
    pub variants: Vec<EmbedVariant>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EmbedVariant {
    pub bitrate: Option<u64>,
    pub content_type: Option<String>,
    pub url: String,
}
