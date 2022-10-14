use crate::cache::doc::DocWithId;

use super::ids::{RundownId, SegmentId};

#[derive(Clone)]
pub struct Segment {
    pub id: SegmentId,
    pub rank: usize,

    pub rundown_id: RundownId,
}
impl<'a> DocWithId<'a, SegmentId> for Segment {
    fn doc_id(&'a self) -> &'a SegmentId {
        &self.id
    }
}
