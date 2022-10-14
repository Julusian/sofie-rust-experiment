use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct Segment {
    pub id: String,
    pub rank: usize,

    pub rundown_id: String,
}
impl<'a> DocWithId<'a> for Segment {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
