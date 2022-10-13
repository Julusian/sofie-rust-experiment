use std::collections::HashMap;

use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct SourceLayer {
    pub exclusive_group: Option<String>,
}
pub type SourceLayers = HashMap<String, SourceLayer>;

#[derive(Clone)]
pub struct ShowStyleBase {
    pub id: String,

    pub source_layers: SourceLayers,
}
impl<'a> DocWithId<'a> for ShowStyleBase {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
