use crate::{
    cache::{collection::DbCacheWriteCollectionImpl, object::DbCacheWriteObjectImpl},
    data_model::{
        ids::{PieceId, RundownId},
        piece::Piece,
        rundown::Rundown,
    },
};

pub struct IngestCache {
    pub rundown_external_id: String,
    pub rundown: DbCacheWriteObjectImpl<Rundown, RundownId>, // TODO - should be Optional

    pub pieces: DbCacheWriteCollectionImpl<Piece, PieceId>,
}
impl IngestCache {
    // pub fn get_current_part_instance(&self) -> Option<PartInstance> {
    //     let playlist = self.playlist.doc();

    //     if let Some(instance_id) = &playlist.current_part_instance_id {
    //         self.part_instances.find_one_by_id(instance_id)
    //     } else {
    //         None
    //     }
    // }
}
