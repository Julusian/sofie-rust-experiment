use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::ids::{
    PartInstanceId, RundownId, RundownPlaylistActivationId, RundownPlaylistId, SegmentId,
};

#[derive(Clone, Copy, PartialEq, Deserialize_repr, Serialize_repr, Default)]
#[repr(u8)]
pub enum RundownHoldState {
    #[default]
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

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct RundownPlaylist {
    #[serde(rename = "_id")]
    pub id: RundownPlaylistId,

    pub external_id: String,

    pub studio_id: String, // TODO - type

    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_from_snapshot_id: Option<RundownPlaylistId>,

    pub name: String,

    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_time: Option<DateTime<Utc>>,

    pub timing: serde_json::Value,

    pub activation_id: Option<RundownPlaylistActivationId>,
    #[serde(default)]
    pub rehearsal: bool,
    #[serde(default)]
    pub hold_state: RundownHoldState,

    pub current_part_instance_id: Option<PartInstanceId>,
    pub next_part_instance_id: Option<PartInstanceId>,
    pub previous_part_instance_id: Option<PartInstanceId>,
    pub next_segment_id: Option<SegmentId>,
    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_time_offset: Option<Duration>,
    #[serde(default)]
    pub next_part_manual: bool,

    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_playback: Option<DateTime<Utc>>,

    #[serde(default, rename = "loop")]
    pub loop_: bool,

    #[serde(default)]
    pub out_of_order_timing: bool,
    #[serde(default)]
    pub time_of_day_countdowns: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_incorrect_part_playback_reported: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rundowns_started_playback: Option<HashMap<RundownId, DateTime<Utc>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_take_time: Option<DateTime<Utc>>,

    #[serde(default)]
    pub rundown_ranks_are_set_in_sofie: bool,
    pub rundown_ids_in_order: Vec<RundownId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_persistent_state: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracked_ab_sessions: Option<serde_json::Value>,
}
impl<'a> DocWithId<'a, RundownPlaylistId> for RundownPlaylist {
    fn doc_id(&'a self) -> &'a RundownPlaylistId {
        &self.id
    }
}
