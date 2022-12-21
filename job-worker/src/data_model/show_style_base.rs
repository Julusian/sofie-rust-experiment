use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cache::doc::DocWithId;

use super::ids::ShowStyleBaseId;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLayer {
    pub exclusive_group: Option<String>,
}
pub type SourceLayers = HashMap<String, SourceLayer>;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowStyleBase {
    #[serde(rename = "_id")]
    pub id: ShowStyleBaseId,

    pub source_layers: SourceLayers,
}
impl<'a> DocWithId<'a, ShowStyleBaseId> for ShowStyleBase {
    fn doc_id(&'a self) -> &'a ShowStyleBaseId {
        &self.id
    }
}
