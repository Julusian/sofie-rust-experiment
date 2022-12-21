use chrono::Duration;
use mongodb::bson::{self, Document};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::cache::doc::DocWithId;

use super::ids::{PartId, PieceId, RundownId, SegmentId};

#[serde_as]
#[derive(Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PieceEnableStart {
    Offset(#[serde_as(as = "serde_with::DurationSeconds<i64>")] Duration),
    // TODO - this is broken..
    Now,
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub struct PieceEnable {
    pub start: PieceEnableStart,

    #[serde_as(as = "Option<serde_with::DurationSeconds<i64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PieceLifespan {
    /** The Piece will only exist in it's designated Part. As soon as the playhead leaves the Part, the Piece will stop */
    WithinPart, // = 'part-only',
    /** The Piece will only exist in it's designated Segment. It will begin playing when taken and will stop when the
     * playhead leaves the Segment */
    OutOnSegmentChange, // = 'segment-change',
    /** The Piece will only exist in it's designated Segment. It will begin playing when taken and will stop when the
     * playhead leaves the Segment or the playhead moves before the beginning of the Piece */
    OutOnSegmentEnd, //= 'segment-end',
    /** The Piece will only exist in it's designated Rundown. It will begin playing when taken and will stop when the
     * playhead leaves the Rundown */
    OutOnRundownChange, // = 'rundown-change',
    /** The Piece will only exist in it's designated Rundown. It will begin playing when taken and will stop when the
     * playhead leaves the Rundown or the playhead moves before the beginning of the Piece */
    OutOnRundownEnd, //= 'rundown-end',
    /** The Piece will only exist while the ShowStyle doesn't change. It will begin playing when taken and will stop
     * when the playhead leaves the Rundown into a new Rundown with a different ShowStyle */
    OutOnShowStyleEnd, //= 'showstyle-end',
}
impl Into<bson::Bson> for PieceLifespan {
    fn into(self) -> bson::Bson {
        bson::Bson::Int32(self as i32)
    }
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum IBlueprintPieceType {
    Normal,        // = 'normal',
    InTransition,  // = 'in-transition',
    OutTransition, // = 'out-transition',
}

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub struct Piece {
    pub id: PieceId,

    pub start_part_id: PartId,
    pub start_segment_id: SegmentId,
    pub start_rundown_id: RundownId,

    pub enable: PieceEnable,
    pub lifespan: PieceLifespan,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub preroll_duration: Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub postroll_duration: Duration,

    pub source_layer_id: String,
    pub virtual_: bool,
    pub piece_type: IBlueprintPieceType,

    pub extend_on_hold: bool,
}
impl<'a> DocWithId<'a, PieceId> for Piece {
    fn doc_id(&'a self) -> &'a PieceId {
        &self.id
    }
}
