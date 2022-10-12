use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct PieceInstance {
    pub id: String,

    pub part_instance_id: String,

    pub reset: bool,
}
impl<'a> DocWithId<'a> for PieceInstance {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
