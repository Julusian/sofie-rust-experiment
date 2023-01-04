use std::rc::Rc;

use crate::data_model::{
    ids::{ShowStyleBaseId, ShowStyleVariantId},
    show_style_base::SourceLayers,
};

use super::direct_collections::{DirectCollections, MongoReadOnlyCollection};

pub struct JobContext {
    //
    collections: Rc<DirectCollections>,
}
impl JobContext {
    pub fn create(collections: Rc<DirectCollections>) -> JobContext {
        JobContext { collections }
    }

    pub fn direct_collections(&self) -> &DirectCollections {
        &self.collections
    }

    pub async fn get_show_style_compound(
        &self,
        _variant_id: &ShowStyleVariantId,
        base_id: &ShowStyleBaseId,
    ) -> Result<Option<ShowStyleBase>, String> {
        // TODO - properly
        self.get_show_style_base(base_id).await
    }
    pub async fn get_show_style_base(
        &self,
        base_id: &ShowStyleBaseId,
    ) -> Result<Option<ShowStyleBase>, String> {
        let db_show_style = self
            .collections
            .show_style_bases
            .find_one_by_id(base_id, None)
            .await?;

        Ok(db_show_style.map(|show_style| ShowStyleBase {
                id: show_style.id,
                source_layers: show_style.source_layers_with_overrides.defaults, // TODO - respect overrides
            }))
    }
}

pub struct ShowStyleBase {
    pub id: ShowStyleBaseId,

    pub source_layers: SourceLayers,
}
