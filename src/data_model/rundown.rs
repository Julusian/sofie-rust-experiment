use chrono::{DateTime, Duration, Utc};

use crate::cache::doc::DocWithId;

use super::ids::{RundownId, ShowStyleBaseId, ShowStyleVariantId};

#[derive(Clone)]
pub struct Rundown {
    pub id: RundownId,

    pub show_style_base_id: ShowStyleBaseId,
    pub show_style_variant_id: ShowStyleVariantId,

    pub restored_from_snapshot_id: Option<String>,
}
impl<'a> DocWithId<'a, RundownId> for Rundown {
    fn doc_id(&'a self) -> &'a RundownId {
        &self.id
    }
}
