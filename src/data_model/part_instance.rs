use chrono::{DateTime, Duration, Utc};

use crate::cache::doc::DocWithId;

use super::part::Part;

#[derive(Clone)]
pub struct PartInstanceTimings {
    pub planned_started_playback: Option<DateTime<Utc>>,
    pub planned_stopped_playback: Option<DateTime<Utc>>,

    pub take: Option<DateTime<Utc>>,
    pub play_offset: Option<Duration>,
}

#[derive(Clone)]
pub struct PartInstance {
    pub id: String,

    pub rundown_id: String,
    pub segment_id: String,

    pub part: Part,

    pub timings: PartInstanceTimings,
    pub is_taken: bool,
    pub reset: bool,

    pub consumes_next_segment_id: bool,

    pub block_take_until: Option<DateTime<Utc>>,
}
impl<'a> DocWithId<'a> for PartInstance {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
