use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::{
    extra::NoteBase,
    ids::{RundownId, SegmentId},
};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
pub enum SegmentOrphaned {
    /** Segment is deleted from the NRCS but we still need it */
    #[serde(rename = "deleted")]
    DELETED, // = 'deleted',
    /** Segment should be hidden, but it is still playing */
    #[serde(rename = "hidden")]
    HIDDEN, // = 'hidden',
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SegmentNoteOrigin {
    name: String,
}

pub type SegmentNote = NoteBase<SegmentNoteOrigin>;

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    #[serde(rename = "_id")]
    pub id: SegmentId,
    #[serde(rename = "_rank")]
    pub rank: f32,

    pub rundown_id: RundownId,

    pub external_id: String,

    #[serde_as(as = "serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>")]
    pub external_modified: DateTime<Utc>,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(default)]
    pub show_shelf: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_as: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub orphaned: Option<SegmentOrphaned>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<SegmentNote>>,
}
impl<'a> DocWithId<'a, SegmentId> for Segment {
    fn doc_id(&'a self) -> &'a SegmentId {
        &self.id
    }
}
