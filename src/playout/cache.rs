use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollectionImpl},
        doc::DocWithId,
        object::{DbCacheReadObject, DbCacheWriteObjectImpl},
    },
    data_model::{
        part::Part, part_instance::PartInstance, piece_instance::PieceInstance,
        rundown_playlist::RundownPlaylist, segment::Segment,
    },
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
    pub segments: DbCacheWriteCollectionImpl<Segment>,
    pub parts: DbCacheWriteCollectionImpl<Part>,
    pub part_instances: DbCacheWriteCollectionImpl<PartInstance>,
    pub piece_instances: DbCacheWriteCollectionImpl<PieceInstance>,

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

    pub fn get_ordered_segments_and_parts(&self) -> SegmentsAndParts {
        get_rundowns_segments_and_parts_from_caches(
            &self.parts,
            &self.segments,
            &self.playlist.doc().rundown_ids_in_order,
        )
    }
}

pub struct SegmentsAndParts {
    pub segments: Vec<Segment>,
    pub parts: Vec<Part>,
}

fn get_rundowns_segments_and_parts_from_caches(
    parts_cache: &DbCacheWriteCollectionImpl<Part>,
    segments_cache: &DbCacheWriteCollectionImpl<Segment>,
    rundown_ids_in_order: &Vec<String>,
) -> SegmentsAndParts {
    todo!()
    // const segments = sortSegmentsInRundowns(
    // 	segmentsCache.findAll(null, {
    // 		sort: {
    // 			rundownId: 1,
    // 			_rank: 1,
    // 		},
    // 	}),
    // 	playlist
    // )

    // const parts = sortPartsInSortedSegments(
    // 	partsCache.findAll(null, {
    // 		sort: {
    // 			rundownId: 1,
    // 			_rank: 1,
    // 		},
    // 	}),
    // 	segments
    // )

    // return {
    // 	segments: segments,
    // 	parts: parts,
    // }
}
