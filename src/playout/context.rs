use crate::data_model::show_style_base::ShowStyleBase;

pub struct JobContext {
    //
}
impl JobContext {
    pub async fn get_show_style_compound(
        &self,
        variant_id: &str,
        base_id: &str,
    ) -> Result<ShowStyleBase, String> {
        todo!()
    }
}
