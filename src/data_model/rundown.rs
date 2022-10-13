use chrono::{DateTime, Duration, Utc};

use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct Rundown {
    pub id: String,

    pub show_style_base_id: String,
    pub show_style_variant_id: String,
}
impl<'a> DocWithId<'a> for Rundown {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
