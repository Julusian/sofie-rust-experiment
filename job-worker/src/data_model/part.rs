use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::ids::{PartId, RundownId, SegmentId};

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartInTransition {
    /** Duration this transition block a take for. After this time, another take is allowed which may cut this transition off early */
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub block_take_duration: Duration,
    /** Duration the previous part be kept playing once the transition is started. Typically the duration of it remaining in-vision */
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub previous_part_keepalive_duration: Duration,
    /** Duration the pieces of the part should be delayed for once the transition starts. Typically the duration until the new part is in-vision */
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub part_content_delay_duration: Duration,
}
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartOutTransition {
    /** How long to keep this part alive after taken out  */
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub duration: Duration,
}

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize)]
pub struct Part {
    #[serde(rename = "_id")]
    pub id: PartId,
    #[serde(rename = "_rank")]
    pub rank: usize,

    pub rundown_id: RundownId,
    pub segment_id: SegmentId,

    // pub autonext: bool, Implied by autonext_overlap being defined
    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonext_overlap: Option<Duration>,

    pub disable_next_in_transition: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transition: Option<PartInTransition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_transition: Option<PartOutTransition>,
    #[serde(default)]
    pub untimed: bool,

    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_duration: Option<Duration>,

    #[serde(default)]
    pub invalid: bool,
    #[serde(default)]
    pub floated: bool,
}
impl<'a> DocWithId<'a, PartId> for Part {
    fn doc_id(&'a self) -> &'a PartId {
        &self.id
    }
}
impl Part {
    pub fn is_playable(&self) -> bool {
        !self.invalid && !self.floated
    }
}
