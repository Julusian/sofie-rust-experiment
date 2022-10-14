use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollectionImpl},
        doc::DocWithId,
        object::{DbCacheReadObject, DbCacheWriteObjectImpl},
    },
    data_model::{
        ids::{
            PartId, PartInstanceId, PieceInstanceId, RundownId, RundownPlaylistActivationId,
            RundownPlaylistId, SegmentId, ShowStyleBaseId,
        },
        part::Part,
        part_instance::PartInstance,
        piece_instance::PieceInstance,
        rundown::Rundown,
        rundown_playlist::RundownPlaylist,
        segment::Segment,
    },
};

#[derive(Clone)]
pub struct FakeDoc {
    pub id: RundownPlaylistActivationId,
}
impl<'a> DocWithId<'a, RundownPlaylistActivationId> for FakeDoc {
    fn doc_id(&'a self) -> &'a RundownPlaylistActivationId {
        &self.id
    }
}

pub struct PlayoutCache {
    pub playlist_id: String,
    pub playlist: DbCacheWriteObjectImpl<RundownPlaylist, RundownPlaylistId>,

    // pub playlist_lock: Rc<>
    pub rundowns: DbCacheWriteCollectionImpl<Rundown, RundownId>,
    pub segments: DbCacheWriteCollectionImpl<Segment, SegmentId>,
    pub parts: DbCacheWriteCollectionImpl<Part, PartId>,
    pub part_instances: DbCacheWriteCollectionImpl<PartInstance, PartInstanceId>,
    pub piece_instances: DbCacheWriteCollectionImpl<PieceInstance, PieceInstanceId>,

    pub baseline_objects: DbCacheWriteCollectionImpl<FakeDoc, RundownPlaylistActivationId>,
    pub timeline: DbCacheWriteObjectImpl<FakeDoc, RundownPlaylistActivationId>,

    pub peripheral_devices: DbCacheWriteCollectionImpl<FakeDoc, RundownPlaylistActivationId>,
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

    pub fn get_rundown_ids_from_cache(&self) -> Vec<RundownId> {
        self.rundowns
            .find_all()
            .into_iter()
            .map(|rd| rd.id)
            .collect_vec()
    }

    pub fn get_show_style_ids_rundown_mapping_from_cache(
        &self,
    ) -> HashMap<RundownId, ShowStyleBaseId> {
        self.rundowns
            .find_all()
            .into_iter()
            .map(|rd| (rd.id, rd.show_style_base_id))
            .collect()
    }
}

pub struct SegmentsAndParts {
    pub segments: Vec<Segment>,
    pub parts: Vec<Part>,
}

fn get_rundowns_segments_and_parts_from_caches(
    parts_cache: &DbCacheWriteCollectionImpl<Part, PartId>,
    segments_cache: &DbCacheWriteCollectionImpl<Segment, SegmentId>,
    rundown_ids_in_order: &Vec<RundownId>,
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
