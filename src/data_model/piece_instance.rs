use crate::cache::doc::DocWithId;

use super::{
    ids::{PartInstanceId, PieceId, PieceInstanceId, PieceInstanceInfiniteId},
    piece::Piece,
};

#[derive(Clone)]
pub struct PieceInstanceInfinite {
    pub infinite_instance_id: PieceInstanceInfiniteId,
    pub infinite_instance_index: usize,
    pub infinite_piece_id: PieceId,
    pub from_previous_part: bool,
    pub from_hold: bool,
}

#[derive(Clone)]
pub struct PieceInstance {
    pub id: PieceInstanceId,

    pub part_instance_id: PartInstanceId,

    pub piece: Piece,

    pub reset: bool,
    pub disabled: bool,

    pub dynamically_inserted: Option<u64>,
    pub infinite: Option<PieceInstanceInfinite>,
}
impl<'a> DocWithId<'a, PieceInstanceId> for PieceInstance {
    fn doc_id(&'a self) -> &'a PieceInstanceId {
        &self.id
    }
}
