use crate::data_model::{
    ids::{ShowStyleBaseId, ShowStyleVariantId},
    show_style_base::ShowStyleBase,
};

pub struct JobContext {
    //
}
impl JobContext {
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
