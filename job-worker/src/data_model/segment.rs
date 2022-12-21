use crate::cache::doc::DocWithId;

use super::ids::{RundownId, SegmentId};

#[derive(Clone, Copy, PartialEq)]
pub enum SegmentOrphaned {
    No,
    /** Segment is deleted from the NRCS but we still need it */
    DELETED, // = 'deleted',
    /** Segment should be hidden, but it is still playing */
    HIDDEN, // = 'hidden',
}

#[derive(Clone)]
pub struct Segment {
    pub id: SegmentId,
    pub rank: usize,

    pub rundown_id: RundownId,

    pub orphaned: SegmentOrphaned,
}
impl<'a> DocWithId<'a, SegmentId> for Segment {
    fn doc_id(&'a self) -> &'a SegmentId {
        &self.id
    }
}
