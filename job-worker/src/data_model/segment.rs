use serde::{Deserialize, Serialize};

use crate::cache::doc::DocWithId;

use super::ids::{RundownId, SegmentId};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
pub enum SegmentOrphaned {
    #[default]
    No,
    /** Segment is deleted from the NRCS but we still need it */
    DELETED, // = 'deleted',
    /** Segment should be hidden, but it is still playing */
    HIDDEN, // = 'hidden',
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct Segment {
    #[serde(rename = "_id")]
    pub id: SegmentId,
    #[serde(rename = "_rank")]
    pub rank: f32,

    pub rundown_id: RundownId,

    #[serde(default)]
    pub orphaned: SegmentOrphaned,
}
impl<'a> DocWithId<'a, SegmentId> for Segment {
    fn doc_id(&'a self) -> &'a SegmentId {
        &self.id
    }
}
