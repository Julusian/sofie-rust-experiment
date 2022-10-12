use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollectionImpl},
        doc::DocWithId,
        object::{DbCacheReadObject, DbCacheWriteObjectImpl},
    },
    data_model::{part_instance::PartInstance, rundown_playlist::RundownPlaylist},
};

#[derive(Clone)]
pub struct FakeDoc {
    id: String,
}
impl<'a> DocWithId<'a> for FakeDoc {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}

pub struct PlayoutCache {
    pub playlist_id: String,
    pub playlist: DbCacheWriteObjectImpl<RundownPlaylist>,

    // pub playlist_lock: Rc<>
    pub rundowns: DbCacheWriteCollectionImpl<FakeDoc>,
    pub segments: DbCacheWriteCollectionImpl<FakeDoc>,
    pub parts: DbCacheWriteCollectionImpl<FakeDoc>,
    pub part_instances: DbCacheWriteCollectionImpl<PartInstance>,
    pub piece_instances: DbCacheWriteCollectionImpl<FakeDoc>,

    pub baseline_objects: DbCacheWriteCollectionImpl<FakeDoc>,
    pub timeline: DbCacheWriteObjectImpl<FakeDoc>,

    pub peripheral_devices: DbCacheWriteCollectionImpl<FakeDoc>,
}
impl PlayoutCache {
    pub fn get_current_part_instance(&self) -> Option<PartInstance> {
        let playlist = self.playlist.doc();

        if let Some(instance_id) = &playlist.current_part_instance_id {
            self.part_instances.find_one_by_id(instance_id)
        } else {
            None
        }
    }

    pub fn get_next_part_instance(&self) -> Option<PartInstance> {
        let playlist = self.playlist.doc();

        if let Some(instance_id) = &playlist.next_part_instance_id {
            self.part_instances.find_one_by_id(instance_id)
        } else {
            None
        }
    }
    pub fn get_previous_part_instance(&self) -> Option<PartInstance> {
        let playlist = self.playlist.doc();

        if let Some(instance_id) = &playlist.previous_part_instance_id {
            self.part_instances.find_one_by_id(instance_id)
        } else {
            None
        }
    }
}
