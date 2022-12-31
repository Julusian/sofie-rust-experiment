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
    Offset(
        #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
        Duration,
    ),
    // TODO - this is broken..
    Now,
}

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct PieceEnable {
    pub start: PieceEnableStart,

    #[serde_as(
        as = "Option<serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>>"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PieceLifespan {
    /** The Piece will only exist in it's designated Part. As soon as the playhead leaves the Part, the Piece will stop */
    #[serde(rename = "part-only")]
    WithinPart, // = 'part-only',
    /** The Piece will only exist in it's designated Segment. It will begin playing when taken and will stop when the
     * playhead leaves the Segment */
    #[serde(rename = "segment-change")]
    OutOnSegmentChange, // = 'segment-change',
    /** The Piece will only exist in it's designated Segment. It will begin playing when taken and will stop when the
     * playhead leaves the Segment or the playhead moves before the beginning of the Piece */
    #[serde(rename = "segment-end")]
    OutOnSegmentEnd, //= 'segment-end',
    /** The Piece will only exist in it's designated Rundown. It will begin playing when taken and will stop when the
     * playhead leaves the Rundown */
    #[serde(rename = "rundown-change")]
    OutOnRundownChange, // = 'rundown-change',
    /** The Piece will only exist in it's designated Rundown. It will begin playing when taken and will stop when the
     * playhead leaves the Rundown or the playhead moves before the beginning of the Piece */
    #[serde(rename = "rundown-end")]
    OutOnRundownEnd, //= 'rundown-end',
    /** The Piece will only exist while the ShowStyle doesn't change. It will begin playing when taken and will stop
     * when the playhead leaves the Rundown into a new Rundown with a different ShowStyle */
    #[serde(rename = "showstyle-end")]
    OutOnShowStyleEnd, //= 'showstyle-end',
}
impl Into<bson::Bson> for PieceLifespan {
    fn into(self) -> bson::Bson {
        bson::Bson::Int32(self as i32)
    }
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum IBlueprintPieceType {
    #[serde(rename = "normal")]
    Normal, // = 'normal',
    #[serde(rename = "in-transition")]
    InTransition, // = 'in-transition',
    #[serde(rename = "out-transition")]
    OutTransition, // = 'out-transition',
}

#[serde_as]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Deserialize, Serialize)]
pub struct Piece {
    #[serde(rename = "_id")]
    pub id: PieceId,

    pub start_part_id: PartId,
    pub start_segment_id: SegmentId,
    pub start_rundown_id: RundownId,

    pub external_id: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<serde_json::Value>,

    pub enable: PieceEnable,
    pub lifespan: PieceLifespan,
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    #[serde(default = "Duration::zero")]
    pub preroll_duration: Duration,
    #[serde_as(as = "serde_with::DurationMilliSeconds<i64, serde_with::formats::Flexible>")]
    #[serde(default = "Duration::zero")]
    pub postroll_duration: Duration,

    pub source_layer_id: String,
    pub output_layer_id: String,

    #[serde(default)]
    pub virtual_: bool,
    pub piece_type: IBlueprintPieceType,

    #[serde(default)]
    pub extend_on_hold: bool,

    #[serde(default)]
    pub invalid: bool,

    pub content: serde_json::Value,

    pub status: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continues_ref_id: Option<PieceId>,

    pub timeline_objects_string: String,

    #[serde(default)]
    pub to_be_queued: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_playout_items: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_packages: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_direct_play: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(default)]
    pub has_side_effects: bool,
    #[serde(default)]
    pub not_in_vision: bool,
}
impl<'a> DocWithId<'a, PieceId> for Piece {
    fn doc_id(&'a self) -> &'a PieceId {
        &self.id
    }
}
