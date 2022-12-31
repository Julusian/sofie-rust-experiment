use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::cache::doc::DocWithId;

use super::{
    extra::NoteBase,
    ids::{RundownId, RundownPlaylistId, ShowStyleBaseId, ShowStyleVariantId},
};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum RundownOrphaned {
    #[serde(alias = "deleted")]
    Deleted,
    #[serde(alias = "from-snapshot")]
    FromSnapshot,
    #[serde(alias = "manual")]
    Manual,
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RundownNoteOrigin {
    name: String,
}

pub type RundownNote = NoteBase<RundownNoteOrigin>;

#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct Rundown {
    #[serde(rename = "_id")]
    pub id: RundownId,

    pub external_id: String,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub studio_id: String, // TODO - type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peripheral_device_id: Option<String>, // TODO - type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_from_snapshot_id: Option<RundownId>,

    pub show_style_base_id: ShowStyleBaseId,
    pub show_style_variant_id: ShowStyleVariantId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist_external_id: Option<String>,

    pub playlist_id: RundownPlaylistId,
    #[serde(default)]
    pub playlist_id_is_set_in_sofie: bool,

    #[serde(default)]
    pub end_of_rundown_is_show_break: bool,

    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,

    pub import_versions: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub orphaned: Option<RundownOrphaned>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notified_current_playing_part_external_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<RundownNote>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub externalNRCSName: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_modify_hash: Option<String>,

    pub timing: serde_json::Value,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_status: Option<String>,
}
impl<'a> DocWithId<'a, RundownId> for Rundown {
    fn doc_id(&'a self) -> &'a RundownId {
        &self.id
    }
}
