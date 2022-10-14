use chrono::{DateTime, Duration, Utc};

use crate::cache::doc::DocWithId;

use super::ids::{
    PartInstanceId, RundownId, RundownPlaylistActivationId, RundownPlaylistId, SegmentId,
};

#[derive(Clone, Copy, PartialEq)]
pub enum RundownHoldState {
    NONE = 0,
    PENDING = 1,  // During STK
    ACTIVE = 2,   // During full, STK is played
    COMPLETE = 3, // During full, full is played
}
pub fn progress_hold_state(input: &RundownHoldState) -> RundownHoldState {
    match input {
        RundownHoldState::NONE => RundownHoldState::NONE,
        RundownHoldState::PENDING => RundownHoldState::ACTIVE,
        RundownHoldState::ACTIVE => RundownHoldState::COMPLETE,
        RundownHoldState::COMPLETE => RundownHoldState::NONE,
    }
}

#[derive(Clone)]
pub struct RundownPlaylist {
    pub id: RundownPlaylistId,

    pub activation_id: Option<RundownPlaylistActivationId>,
    pub rehearsal: bool,
    pub hold_state: RundownHoldState,

    pub current_part_instance_id: Option<PartInstanceId>,
    pub next_part_instance_id: Option<PartInstanceId>,
    pub previous_part_instance_id: Option<PartInstanceId>,
    pub next_segment_id: Option<SegmentId>,
    pub next_time_offset: Option<Duration>,
    pub next_part_manual: bool,

    pub started_playback: Option<DateTime<Utc>>,

    pub rundown_ids_in_order: Vec<RundownId>,
    pub loop_: bool,
}
impl<'a> DocWithId<'a, RundownPlaylistId> for RundownPlaylist {
    fn doc_id(&'a self) -> &'a RundownPlaylistId {
        &self.id
    }
}
