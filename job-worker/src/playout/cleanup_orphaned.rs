use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    cache::{collection::DbCacheReadCollection, object::DbCacheReadObject},
    constants::PRESERVE_UNSYNCED_PLAYING_SEGMENT_CONTENTS,
    context::context::JobContext,
    data_model::{
        ids::{RundownId, SegmentId},
        part_instance::PartInstanceOrphaned,
        segment::SegmentOrphaned,
    },
};

use super::cache::PlayoutCache;

/**
 * Cleanup any orphaned (deleted) segments and partinstances once they are no longer being played
 * @param cache
 */
pub async fn cleanupOrphanedItems(context: &JobContext, cache: &mut PlayoutCache) {
    let selectedPartInstancesSegmentIds = {
        let mut res = HashSet::new();

        if let Some(instance) = cache.get_current_part_instance() {
            res.insert(instance.segment_id);
        }
        if let Some(instance) = cache.get_next_part_instance() {
            res.insert(instance.segment_id);
        }

        res
    };

    // Cleanup any orphaned segments once they are no longer being played. This also cleans up any adlib-parts, that have been marked as deleted as a deferred cleanup operation
    let segments = cache
        .segments
        .find_some(|s| s.orphaned == SegmentOrphaned::No);
    let orphanedSegmentIds = segments.iter().map(|s| s.id.clone()).collect_vec();

    let mut alterSegmentsFromRundowns: HashMap<RundownId, AlterOrphanedSegmentIds> = HashMap::new();
    //= new Map<RundownId, { deleted: SegmentId[]; hidden: SegmentId[] }>()
    for segment in segments {
        // If the segment is orphaned and not the segment for the next or current partinstance
        if !selectedPartInstancesSegmentIds.contains(&segment.id) {
            // todo!()
            let rundown_segments_entry =
                alterSegmentsFromRundowns.entry(segment.rundown_id.clone());
            let rundown_segments = match rundown_segments_entry {
                std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                std::collections::hash_map::Entry::Vacant(e) => e.insert(AlterOrphanedSegmentIds {
                    deleted: vec![],
                    hidden: vec![],
                }),
            };

            // The segment is finished with. Queue it for attempted removal or reingest
            match segment.orphaned {
                SegmentOrphaned::DELETED => {
                    rundown_segments.deleted.push(segment.id.clone());
                }
                SegmentOrphaned::HIDDEN => {
                    // The segment is finished with. Queue it for attempted resync
                    rundown_segments.hidden.push(segment.id.clone());
                }
                SegmentOrphaned::No => {
                    // Do nothing
                }
            }
        }
    }

    // We need to run this outside of the current lock, and within an ingest lock, so defer to the work queue
    for (rundownId, candidateSegmentIds) in alterSegmentsFromRundowns {
        let rundown = cache.rundowns.find_one_by_id(&rundownId);
        if let Some(rundown) = rundown {
            if rundown.restored_from_snapshot_id.is_some() {
                // This is not valid as the rundownId won't match the externalId, so ingest will fail
                // For now do nothing
            } else {
                todo!()
                // await context.queueIngestJob(IngestJobs.RemoveOrphanedSegments, {
                // 	rundownExternalId: rundown.externalId,
                // 	peripheralDeviceId: null,
                // 	orphanedHiddenSegmentIds: candidateSegmentIds.hidden,
                // 	orphanedDeletedSegmentIds: candidateSegmentIds.deleted,
                // })
            }
        }
    }

    let playlist = cache.playlist.doc();

    let mut removePartInstanceIds = Vec::new(); //: PartInstanceId[] = []
                                                // Cleanup any orphaned partinstances once they are no longer being played (and the segment isnt orphaned)
    let orphanedInstances = cache
        .part_instances
        .find_some(|p| p.orphaned == PartInstanceOrphaned::Deleted && !p.reset);
    for partInstance in orphanedInstances {
        if (PRESERVE_UNSYNCED_PLAYING_SEGMENT_CONTENTS
            && orphanedSegmentIds.contains(&partInstance.segment_id))
        {
            // If the segment is also orphaned, then don't delete it until it is clear
            continue;
        }

        if Some(&partInstance.id) != playlist.current_part_instance_id.as_ref()
            && Some(&partInstance.id) != playlist.next_part_instance_id.as_ref()
        {
            removePartInstanceIds.push(partInstance.id);
        }
    }

    // Cleanup any instances from above
    if removePartInstanceIds.len() > 0 {
        resetPartInstancesWithPieceInstances(context, cache, "AA".to_string()); //{ _id: { $in: removePartInstanceIds } })
    }
}

struct AlterOrphanedSegmentIds {
    deleted: Vec<SegmentId>,
    hidden: Vec<SegmentId>,
}

/**
 * Reset selected or all partInstances with their pieceInstances
 * @param cache
 * @param selector if not provided, all partInstances will be reset
 */
pub fn resetPartInstancesWithPieceInstances(
    context: &JobContext,
    cache: &mut PlayoutCache,
    selector: String, // MongoQuery<DBPartInstance>
) {
    todo!()
    // const partInstancesToReset = cache.PartInstances.updateAll((p) => {
    // 	if (!p.reset && (!selector || mongoWhere(p, selector))) {
    // 		p.reset = true
    // 		return p
    // 	} else {
    // 		return false
    // 	}
    // })

    // // Reset any in the cache now
    // if (partInstancesToReset.length) {
    // 	cache.PieceInstances.updateAll((p) => {
    // 		if (!p.reset && partInstancesToReset.includes(p.partInstanceId)) {
    // 			p.reset = true
    // 			return p
    // 		} else {
    // 			return false
    // 		}
    // 	})
    // }

    // // Defer ones which arent loaded
    // cache.deferAfterSave(async (cache) => {
    // 	const partInstanceIdsInCache = cache.PartInstances.findAll(null).map((p) => p._id)

    // 	// Find all the partInstances which are not loaded, but should be reset
    // 	const resetInDb = await context.directCollections.PartInstances.findFetch(
    // 		{
    // 			$and: [
    // 				selector ?? {},
    // 				{
    // 					// Not any which are in the cache, as they have already been done if needed
    // 					_id: { $nin: partInstanceIdsInCache },
    // 					reset: { $ne: true },
    // 				},
    // 			],
    // 		},
    // 		{ projection: { _id: 1 } }
    // 	).then((ps) => ps.map((p) => p._id))

    // 	// Do the reset
    // 	const allToReset = [...resetInDb, ...partInstancesToReset]
    // 	await Promise.all([
    // 		resetInDb.length
    // 			? context.directCollections.PartInstances.update(
    // 					{
    // 						_id: { $in: resetInDb },
    // 						reset: { $ne: true },
    // 					},
    // 					{
    // 						$set: {
    // 							reset: true,
    // 						},
    // 					}
    // 			  )
    // 			: undefined,
    // 		allToReset.length
    // 			? context.directCollections.PieceInstances.update(
    // 					{
    // 						partInstanceId: { $in: allToReset },
    // 						reset: { $ne: true },
    // 					},
    // 					{
    // 						$set: {
    // 							reset: true,
    // 						},
    // 					}
    // 			  )
    // 			: undefined,
    // 	])
    // })
}
