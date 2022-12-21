use std::{collections::HashMap, rc::Rc};

use futures::TryFutureExt;
use itertools::Itertools;
use mongodb::bson::doc;
use tokio::join;

use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollectionImpl},
        doc::DocWithId,
        object::{DbCacheReadObject, DbCacheWriteObjectImpl},
    },
    context::direct_collections::{DirectCollections, MongoReadOnlyCollection},
    data_model::{
        ids::{
            unprotect_array, unprotect_optional, PartId, PartInstanceId, PieceInstanceId,
            ProtectedId, RundownId, RundownPlaylistActivationId, RundownPlaylistId, SegmentId,
            ShowStyleBaseId,
        },
        part::Part,
        part_instance::PartInstance,
        piece_instance::PieceInstance,
        rundown::Rundown,
        rundown_playlist::RundownPlaylist,
        segment::Segment,
    },
    playout::playlist::sort_segments_in_rundowns,
};

use super::playlist::{self, sort_parts_in_sorted_segments};

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
    pub playlist: DbCacheWriteObjectImpl<RundownPlaylist, RundownPlaylistId>,

    // pub playlist_lock: Rc<>
    pub rundowns: DbCacheWriteCollectionImpl<Rundown, RundownId>,
    pub segments: DbCacheWriteCollectionImpl<Segment, SegmentId>,
    pub parts: DbCacheWriteCollectionImpl<Part, PartId>,
    pub part_instances: DbCacheWriteCollectionImpl<PartInstance, PartInstanceId>,
    pub piece_instances: DbCacheWriteCollectionImpl<PieceInstance, PieceInstanceId>,
    // pub baseline_objects: DbCacheWriteCollectionImpl<FakeDoc, RundownPlaylistActivationId>,
    // pub timeline: DbCacheWriteObjectImpl<FakeDoc, RundownPlaylistActivationId>,
    // pub peripheral_devices: DbCacheWriteCollectionImpl<FakeDoc, RundownPlaylistActivationId>,
}
impl PlayoutCache {
    pub async fn create(
        collections: &Rc<DirectCollections>,
        playlist_id: &RundownPlaylistId,
    ) -> Result<PlayoutCache, String> {
        let playlist = collections
            .rundown_playlists
            .find_one_by_id(playlist_id, None)
            .await?;

        if let Some(playlist) = playlist {
            let rundowns = collections
                .rundowns
                .find_fetch(
                    doc! {
                        "playlistId": playlist_id.unprotect()
                    },
                    None,
                )
                .await?;

            let selectedPartInstanceIds = [
                playlist.current_part_instance_id.clone(),
                playlist.next_part_instance_id.clone(),
                playlist.previous_part_instance_id.clone(),
            ]
            .into_iter()
            .filter_map(|v| unprotect_optional(v))
            .collect::<Vec<_>>();

            let rundown_ids = rundowns
                .iter()
                .map(|rd| rd.id.unprotect())
                .collect::<Vec<_>>();

            let partInstancesCollection = {
                // Future: We could optimise away this query if we tracked the segmentIds of these PartInstances on the playlist
                // TODO - projection
                collections
                    .part_instances
                    .find_fetch(doc! { "_id": {"$in": &selectedPartInstanceIds}}, None)
                    .and_then(|aa| {
                        let segmentIds = aa
                            .into_iter()
                            .map(|instance| instance.segment_id.unprotect_move())
                            .unique()
                            .collect::<Vec<_>>();

                        let mut partInstancesSelector = doc! {
                            "rundownId": { "$in": &rundown_ids },
                            "$or": [
                                {
                                    "segmentId": { "$in": &segmentIds },
                                    "reset": { "$ne": true },
                                },
                                {
                                    "_id": { "$in": &selectedPartInstanceIds },
                                },
                            ],
                        };

                        if let Some(activation_id) = &playlist.activation_id {
                            partInstancesSelector
                                .insert("playlistActivationId", activation_id.unprotect());
                        }

                        collections
                            .part_instances
                            .find_fetch(partInstancesSelector, None)
                    })
            };

            // // If there is an ingestCache, then avoid loading some bits from the db for that rundown
            // const loadRundownIds = ingestCache ? rundownIds.filter((id) => id !== ingestCache.RundownId) : rundownIds
            let load_rundown_ids = &rundown_ids;
            // const baselineFromIngest = ingestCache && ingestCache.RundownBaselineObjs.getIfLoaded()
            // const loadBaselineIds = baselineFromIngest ? loadRundownIds : rundownIds

            let mut pieceInstancesSelector = doc! {
                "rundownId": { "$in": &rundown_ids },
                "partInstanceId": { "$in": &selectedPartInstanceIds },
            };
            // TODO - is this correct? If the playlist isnt active do we want any of these?
            if let Some(activation_id) = &playlist.activation_id {
                pieceInstancesSelector.insert("playlistActivationId", activation_id.unprotect());
            }

            let (segments, parts, part_instances, piece_instances) = join!(
                collections
                    .segments
                    .find_fetch(doc! { "rundownId": {"$in":load_rundown_ids }}, None),
                collections
                    .parts
                    .find_fetch(doc! { "rundownId": {"$in":load_rundown_ids }}, None),
                // TODO RundownBaselineObjects
                partInstancesCollection,
                collections
                    .piece_instances
                    .find_fetch(pieceInstancesSelector, None),
                // 	// Future: This could be defered until we get to updateTimeline. It could be a small performance boost
                // 	DbCacheWriteOptionalObject.createOptionalFromDatabase(
                // 		context,
                // 		context.directCollections.Timelines,
                // 		context.studioId
                // 	),
            );

            let segments = segments?;
            let parts = parts?;
            let part_instances = part_instances?;
            let piece_instances = piece_instances?;

            // if (ingestCache) {
            // 	// Populate the collections with the cached data instead
            // 	segments.fillWithDataFromArray(ingestCache.Segments.findAll(null), true)
            // 	parts.fillWithDataFromArray(ingestCache.Parts.findAll(null), true)
            // 	if (baselineFromIngest) {
            // 		baselineObjects.fillWithDataFromArray(baselineFromIngest.findAll(null), true)
            // 	}
            // }

            // return [segments, parts, ...collections, baselineObjects]

            Ok(PlayoutCache {
                playlist: DbCacheWriteObjectImpl::from_document(
                    "rundownPlaylist".to_string(),
                    playlist,
                ),

                rundowns: DbCacheWriteCollectionImpl::from_documents(
                    "rundowns".to_string(),
                    &rundowns,
                ),
                segments: DbCacheWriteCollectionImpl::from_documents(
                    "segments".to_string(),
                    &segments,
                ),
                parts: DbCacheWriteCollectionImpl::from_documents("parts".to_string(), &parts),
                part_instances: DbCacheWriteCollectionImpl::from_documents(
                    "partInstances".to_string(),
                    &part_instances,
                ),
                piece_instances: DbCacheWriteCollectionImpl::from_documents(
                    "pieceInstances".to_string(),
                    &piece_instances,
                ),
            })
        } else {
            Err(format!("RundownPlaylist \"{}\" was not found", playlist_id))
        }
    }

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
    let segments = sort_segments_in_rundowns(segments_cache.find_all(), rundown_ids_in_order);

    let parts = sort_parts_in_sorted_segments(parts_cache.find_all(), &segments);

    SegmentsAndParts { segments, parts }
}
