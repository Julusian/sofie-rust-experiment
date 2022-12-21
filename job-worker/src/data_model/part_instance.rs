use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::{
    ids::{PartInstanceId, RundownId, RundownPlaylistActivationId, SegmentId, SegmentPlayoutId},
    part::Part,
};

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct PartInstanceTimings {
    pub planned_started_playback: Option<DateTime<Utc>>,
    pub planned_stopped_playback: Option<DateTime<Utc>>,

    pub set_as_next: DateTime<Utc>,

    pub take: Option<DateTime<Utc>>,
    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_offset: Option<Duration>,
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
pub enum PartInstanceOrphaned {
    #[default]
    No,
    Deleted,
    AdlibPart,
    //  'adlib-part' | 'deleted'
}

/**
 * Numbers are relative to the start of toPartGroup. Nothing should ever be negative, the pieces of toPartGroup will be delayed to allow for other things to complete.
 * Note: once the part has been taken this should not be recalculated. Doing so may result in the timings shifting if the preroll required for the part is found to have changed
 */
#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct PartCalculatedTimings {
    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transition_start: Option<Duration>, // The start time within the toPartGroup of the inTransition
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub to_part_delay: Duration, // How long after the start of toPartGroup should piece time 0 be
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub to_part_postroll: Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub from_part_remaining: Duration, // How long after the start of toPartGroup should fromPartGroup continue?
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub from_part_postroll: Duration,
}

#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct PartInstance {
    #[serde(rename = "_id")]
    pub id: PartInstanceId,

    pub rundown_id: RundownId,
    pub segment_id: SegmentId,

    pub playlist_activation_id: RundownPlaylistActivationId,
    pub segment_playout_id: SegmentPlayoutId,

    pub part: Part,
    #[serde(default)]
    pub orphaned: PartInstanceOrphaned,

    pub timings: PartInstanceTimings,
    #[serde(default)]
    pub is_taken: bool,
    pub take_count: u64,
    pub rehearsal: bool,
    #[serde(default)]
    pub reset: bool,
    pub part_playout_timings: Option<PartCalculatedTimings>,

    pub consumes_next_segment_id: bool,

    pub block_take_until: Option<DateTime<Utc>>,
}
impl<'a> DocWithId<'a, PartInstanceId> for PartInstance {
    fn doc_id(&'a self) -> &'a PartInstanceId {
        &self.id
    }
}
