use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct Piece {
    pub id: String,

    pub extend_on_hold: bool,
}
impl<'a> DocWithId<'a> for Piece {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
