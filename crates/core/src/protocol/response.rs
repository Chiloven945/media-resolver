use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RemoteDocument {
    #[serde(rename = "__typename")]
    pub type_name: Option<String>,
    #[serde(rename = "mediaDetails", default)]
    pub media_details: Vec<RemoteMedia>,
    pub quoted_tweet: Option<Box<RemoteDocument>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteMedia {
    #[serde(rename = "media_url_https")]
    pub media_url_https: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub original_info: Option<RemoteOriginalInfo>,
    pub video_info: Option<RemoteVideoInfo>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteOriginalInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteVideoInfo {
    pub duration_millis: Option<u64>,
    #[serde(default)]
    pub variants: Vec<RemoteVariant>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteVariant {
    pub bitrate: Option<u64>,
    pub content_type: Option<String>,
    pub url: String,
}
