use chrono::{DateTime, Utc};

use crate::cache::doc::DocWithId;

use super::{
    ids::{
        PartInstanceId, PieceId, PieceInstanceId, PieceInstanceInfiniteId, RundownId,
        RundownPlaylistActivationId,
    },
    piece::Piece,
};

#[derive(Clone)]
pub struct PieceInstanceInfinite {
    pub infinite_instance_id: PieceInstanceInfiniteId,
    pub infinite_instance_index: usize,
    pub infinite_piece_id: PieceId,
    pub from_previous_part: bool,
    pub from_previous_playhead: bool,
    pub from_hold: bool,
}

#[derive(Clone)]
pub struct PieceInstanceUserDuration {
    //
}

#[derive(Clone)]
pub struct PieceInstance {
    pub id: PieceInstanceId,

    pub rundown_id: RundownId,
    pub part_instance_id: PartInstanceId,

    pub piece: Piece,

    pub playlist_activation_id: RundownPlaylistActivationId,
    pub reset: bool,
    pub disabled: bool,

    pub dynamically_inserted: Option<DateTime<Utc>>,
    pub adlib_source_id: Option<String>,
    pub infinite: Option<PieceInstanceInfinite>,

    pub planned_started_playback: Option<DateTime<Utc>>,
    pub planned_stopped_playback: Option<DateTime<Utc>>,
    pub reported_started_playback: Option<DateTime<Utc>>,
    pub reported_stopped_playback: Option<DateTime<Utc>>,

    pub user_duration: Option<PieceInstanceUserDuration>,
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
    is_temporary: bool,
) -> PieceInstance {
    PieceInstance {
        id: PieceInstanceId::new_from(format!(
            "{}_{}",
            part_instance_id.unprotect(),
            piece.id.unprotect()
        )),

        rundown_id: rundown_id.clone(),
        part_instance_id,

        piece: piece,

        playlist_activation_id,
        reset: false,
        disabled: false,
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
