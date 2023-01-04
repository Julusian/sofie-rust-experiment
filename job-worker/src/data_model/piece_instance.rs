use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::{
    ids::{
        PartInstanceId, PieceId, PieceInstanceId, PieceInstanceInfiniteId, ProtectedId, RundownId,
        RundownPlaylistActivationId,
    },
    piece::Piece,
};

#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct PieceInstanceInfinite {
    pub infinite_instance_id: PieceInstanceInfiniteId,
    pub infinite_instance_index: usize,
    pub infinite_piece_id: PieceId,
    #[serde(default)]
    pub from_previous_part: bool,
    #[serde(default)]
    pub from_previous_playhead: bool,
    #[serde(default)]
    pub from_hold: bool,
}

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct PieceInstance {
    #[serde(rename = "_id")]
    pub id: PieceInstanceId,

    pub rundown_id: RundownId,
    pub part_instance_id: PartInstanceId,

    pub piece: Piece, // TODO - is this bad?

    pub playlist_activation_id: RundownPlaylistActivationId,
    #[serde(default)]
    pub reset: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub hidden: bool,

    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamically_inserted: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adlib_source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infinite: Option<PieceInstanceInfinite>,

    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_started_playback: Option<DateTime<Utc>>,
    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_stopped_playback: Option<DateTime<Utc>>,
    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported_started_playback: Option<DateTime<Utc>>,
    #[serde_as(
        as = "Option<serde_with::TimestampMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported_stopped_playback: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_duration: Option<serde_json::Value>,
}
impl<'a> DocWithId<'a, PieceInstanceId> for PieceInstance {
    fn doc_id(&'a self) -> &'a PieceInstanceId {
        &self.id
    }
}

pub fn rewrapPieceToInstance(
    piece: Piece,
    playlist_activation_id: RundownPlaylistActivationId,
    rundown_id: RundownId,
    part_instance_id: PartInstanceId,
    _is_temporary: bool,
) -> PieceInstance {
    PieceInstance {
        id: PieceInstanceId::new_from(format!(
            "{}_{}",
            part_instance_id.unprotect(),
            piece.id.unprotect()
        )),

        rundown_id,
        part_instance_id,

        piece,

        playlist_activation_id,
        reset: false,
        disabled: false,
        hidden: false,
        // is_temporary,
        dynamically_inserted: None,
        adlib_source_id: None,
        infinite: None,

        planned_started_playback: None,
        planned_stopped_playback: None,
        reported_started_playback: None,
        reported_stopped_playback: None,

        user_duration: None,
    }
}
