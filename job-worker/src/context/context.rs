use crate::data_model::{
    ids::{ShowStyleBaseId, ShowStyleVariantId},
    show_style_base::ShowStyleBase,
};

use super::direct_collections::DirectCollections;

pub struct JobContext {
    //
    collections: Box<DirectCollections>,
}
impl JobContext {
    pub fn direct_collections(&self) -> &DirectCollections {
        &self.collections
    }

    pub async fn get_show_style_compound(
        &self,
        variant_id: &ShowStyleVariantId,
        base_id: &ShowStyleBaseId,
    ) -> Result<ShowStyleBase, String> {
        todo!()
    }
    pub async fn get_show_style_base(
        &self,
        base_id: &ShowStyleBaseId,
    ) -> Result<ShowStyleBase, String> {
        todo!()
    }
}
