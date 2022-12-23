use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{cache::doc::DocWithId, object_with_overrides::ObjectWithOverrides};

use super::ids::ShowStyleBaseId;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLayer {
    pub exclusive_group: Option<String>,
}
pub type SourceLayers = HashMap<String, SourceLayer>;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DBShowStyleBase {
    #[serde(rename = "_id")]
    pub id: ShowStyleBaseId,

    pub source_layers_with_overrides: ObjectWithOverrides<SourceLayers>,
}
impl<'a> DocWithId<'a, ShowStyleBaseId> for DBShowStyleBase {
    fn doc_id(&'a self) -> &'a ShowStyleBaseId {
        &self.id
    }
}
