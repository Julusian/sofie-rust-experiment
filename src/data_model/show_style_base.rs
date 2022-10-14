use std::collections::HashMap;

use crate::cache::doc::DocWithId;

use super::ids::ShowStyleBaseId;

#[derive(Clone)]
pub struct SourceLayer {
    pub exclusive_group: Option<String>,
}
pub type SourceLayers = HashMap<String, SourceLayer>;

#[derive(Clone)]
pub struct ShowStyleBase {
    pub id: ShowStyleBaseId,

    pub source_layers: SourceLayers,
}
impl<'a> DocWithId<'a, ShowStyleBaseId> for ShowStyleBase {
    fn doc_id(&'a self) -> &'a ShowStyleBaseId {
        &self.id
    }
}
