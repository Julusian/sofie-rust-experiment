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

/**
 * Numbers are relative to the start of toPartGroup. Nothing should ever be negative, the pieces of toPartGroup will be delayed to allow for other things to complete.
 * Note: once the part has been taken this should not be recalculated. Doing so may result in the timings shifting if the preroll required for the part is found to have changed
 */
#[derive(Clone)]
pub struct PartCalculatedTimings {
    pub in_transition_start: Option<Duration>, // The start time within the toPartGroup of the inTransition
    pub to_part_delay: Duration, // How long after the start of toPartGroup should piece time 0 be
    pub to_part_postroll: Duration,
    pub from_part_remaining: Duration, // How long after the start of toPartGroup should fromPartGroup continue?
    pub from_part_postroll: Duration,
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
    pub part_playout_timings: Option<PartCalculatedTimings>,

    pub consumes_next_segment_id: bool,

    pub block_take_until: Option<DateTime<Utc>>,
}
impl<'a> DocWithId<'a> for PartInstance {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
