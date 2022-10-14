use chrono::Duration;

use crate::cache::doc::DocWithId;

use super::ids::PieceId;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PieceEnableStart {
    Offset(Duration),
    Now,
}

#[derive(Clone)]
pub struct PieceEnable {
    pub start: PieceEnableStart,

    pub duration: Option<Duration>,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum IBlueprintPieceType {
    Normal,        // = 'normal',
    InTransition,  // = 'in-transition',
    OutTransition, // = 'out-transition',
}

#[derive(Clone)]
pub struct Piece {
    pub id: PieceId,

    pub enable: PieceEnable,
    pub lifespan: PieceLifespan,
    pub preroll_duration: Duration,
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
