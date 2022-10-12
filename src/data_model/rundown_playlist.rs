use chrono::{DateTime, Duration, Utc};

use crate::cache::doc::DocWithId;

#[derive(Clone, PartialEq)]
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
    pub id: String,

    pub activation_id: Option<String>,
    pub hold_state: RundownHoldState,

    pub current_part_instance_id: Option<String>,
    pub next_part_instance_id: Option<String>,
    pub previous_part_instance_id: Option<String>,
    pub next_segment_id: Option<String>,
    pub next_time_offset: Option<Duration>,

    pub started_playback: Option<DateTime<Utc>>,

    pub rundown_ids_in_order: Vec<String>,
    pub loop_: bool,
}
impl<'a> DocWithId<'a> for RundownPlaylist {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
