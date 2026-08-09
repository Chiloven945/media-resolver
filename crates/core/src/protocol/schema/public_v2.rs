use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct PublicResponse {
    pub code: Option<u16>,
    pub status: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PublicStatus {
    pub media: Option<PublicMedia>,
    pub quote: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PublicMedia {
    #[serde(default)]
    pub all: Vec<Value>,
    #[serde(default)]
    pub photos: Vec<PublicPhoto>,
    #[serde(default)]
    pub videos: Vec<PublicVideo>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PublicPhoto {
    pub id: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub transcode_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PublicVideo {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub thumbnail_url: Option<String>,
    pub duration: Option<f64>,
    pub filesize: Option<u64>,
    #[serde(default)]
    pub formats: Vec<PublicVideoFormat>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PublicVideoFormat {
    pub container: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u64>,
    pub url: String,
    pub size: Option<u64>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublicTombstone {
    pub reason: String,
}
