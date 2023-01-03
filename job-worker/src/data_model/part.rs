use chrono::Duration;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::{
    extra::{ITranslatableMessage, NoteBase, NoteSeverity},
    ids::{PartId, PieceId, RundownId, SegmentId},
};

#[derive(Clone, Copy, PartialEq, Deserialize_repr, Serialize_repr, Default, Debug)]
#[repr(u8)]
pub enum PartHoldMode {
    #[default]
    NONE = 0,
    FROM = 1,
    TO = 2,
}

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartInTransition {
    /** Duration this transition block a take for. After this time, another take is allowed which may cut this transition off early */
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    pub block_take_duration: Duration,
    /** Duration the previous part be kept playing once the transition is started. Typically the duration of it remaining in-vision */
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    pub previous_part_keepalive_duration: Duration,
    /** Duration the pieces of the part should be delayed for once the transition starts. Typically the duration until the new part is in-vision */
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    pub part_content_delay_duration: Duration,
}
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartOutTransition {
    /** How long to keep this part alive after taken out  */
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    pub duration: Duration,
}
#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartInvalidReason {
    pub message: ITranslatableMessage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<NoteSeverity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartNoteOrigin {
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    piece_id: Option<PieceId>,
}

pub type PartNote = NoteBase<PartNoteOrigin>;

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Part {
    #[serde(rename = "_id")]
    pub id: PartId,
    #[serde(rename = "_rank")]
    pub rank: f32,

    pub rundown_id: RundownId,
    pub segment_id: SegmentId,

    pub external_id: String,

    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metaData: Option<serde_json::Value>,

    #[serde(default)]
    pub hold_mode: PartHoldMode,

    #[serde(default)]
    pub autonext: bool,
    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonext_overlap: Option<Duration>,

    #[serde(default)]
    pub disable_next_in_transition: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transition: Option<PartInTransition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_transition: Option<PartOutTransition>,
    #[serde(default)]
    pub untimed: bool,

    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_duration: Option<Duration>,
    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_duration_with_preroll: Option<Duration>,

    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_duration: Option<Duration>,

    #[serde(default)]
    pub invalid: bool,
    #[serde(default)]
    pub floated: bool,

    #[serde(default)]
    pub gap: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<PartInvalidReason>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<PartNote>>,

    #[serde(default)]
    pub should_notify_current_playing_part: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes_for_next: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_duration_group: Option<String>,
    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_duration: Option<Duration>,
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
