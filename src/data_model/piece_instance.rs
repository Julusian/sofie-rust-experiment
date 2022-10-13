use crate::cache::doc::DocWithId;

use super::piece::Piece;

#[derive(Clone)]
pub struct PieceInstanceInfinite {
    pub infinite_instance_id: String,
    pub infinite_instance_index: usize,
    pub infinite_piece_id: String,
    pub from_previous_part: bool,
    pub from_hold: bool,
}

#[derive(Clone)]
pub struct PieceInstance {
    pub id: String,

    pub part_instance_id: String,

    pub piece: Piece,

    pub reset: bool,
    pub disabled: bool,

    pub dynamically_inserted: Option<u64>,
    pub infinite: Option<PieceInstanceInfinite>,
}
impl<'a> DocWithId<'a> for PieceInstance {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
