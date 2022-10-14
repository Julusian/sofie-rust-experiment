use std::collections::HashSet;

use chrono::{Duration, Utc};
use sofie_rust_experiment::get_random_id;

use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollection},
        object::{DbCacheReadObject, DbCacheWriteObject},
    },
    data_model::{
        ids::{PartInstanceId, SegmentPlayoutId},
        part_instance::{PartInstance, PartInstanceTimings},
    },
};

use super::{
    cache::PlayoutCache,
    context::JobContext,
    infinites2::{
        fetchPiecesThatMayBeActiveForPart, getPieceInstancesForPart,
        syncPlayheadInfinitesForNextPartInstance,
    },
    select_next_part::SelectNextPartResult,
};

pub enum SetNextPartTarget {
    Part(SelectNextPartResult),
    PartInstance(PartInstance),
}

pub async fn setNextPart(
    context: &JobContext,
    cache: &mut PlayoutCache,
    rawNextPart: Option<SetNextPartTarget>,
    // 	rawNextPart: Omit<SelectNextPartResult, 'index'> | DBPartInstance | null,
    setManually: bool,
    nextTimeOffset: Option<Duration>,
) -> Result<(), String> {
    let rundownIds = cache
        .get_rundown_ids_from_cache()
        .into_iter()
        .collect::<HashSet<_>>();
    let currentPartInstance = cache.get_current_part_instance();
    let nextPartInstance = cache.get_next_part_instance();

    if let Some(rawNextPart) = rawNextPart {
        let activation_id = cache.playlist.doc().activation_id.clone().ok_or_else(|| {
            format!(
                "RundownPlaylist \"{}\" is not active",
                cache.playlist.doc_id().unprotect()
            )
        })?;

        // create new instance
        let new_instance_id = match &rawNextPart {
            SetNextPartTarget::PartInstance(instance) => {
                if instance.part.invalid {
                    return Err(format!("Part is marked as invalid, cannot set as next."));
                } else if !rundownIds.contains(&instance.rundown_id) {
                    return Err(format!(
                        "PartInstance \"{}\" of rundown \"{}\" not part of RundownPlaylist \"{}\"",
                        instance.id.unprotect(),
                        instance.rundown_id.unprotect(),
                        cache.playlist.doc_id().unprotect()
                    ));
                }

                cache
                    .part_instances
                    .update_one(&instance.id, |doc| {
                        let mut res = doc.clone();

                        res.consumes_next_segment_id = false;

                        Some(res)
                    })
                    .map_err(|_| format!("Failed to reuse part instance"))?;

                syncPlayheadInfinitesForNextPartInstance(context, cache).await?;

                instance.id.clone()
            }
            SetNextPartTarget::Part(selected_part) => {
                let matched_next_part_instance = nextPartInstance.as_ref().and_then(|inst| {
                    if inst.part.id == selected_part.part_id {
                        Some(inst)
                    } else {
                        None
                    }
                });

                if let Some(next_part_instance) = matched_next_part_instance {
                    // Re-use existing

                    cache
                        .part_instances
                        .update_one(&next_part_instance.id, |doc| {
                            let mut res = doc.clone();

                            res.consumes_next_segment_id = false;

                            Some(res)
                        })
                        .map_err(|_| format!("Failed to reuse part instance"))?;

                    syncPlayheadInfinitesForNextPartInstance(context, cache).await?;

                    next_part_instance.id.clone()
                } else {
                    // Create new isntance
                    let part = cache
                        .parts
                        .find_one_by_id(&selected_part.part_id)
                        .ok_or_else(|| format!("Failed to find part to set as next"))?;

                    if part.invalid {
                        return Err(format!("Part is marked as invalid, cannot set as next."));
                    } else if !rundownIds.contains(&part.rundown_id) {
                        return Err(format!(
                            "Part \"{}\" of rundown \"{}\" not part of RundownPlaylist \"{}\"",
                            part.id.unprotect(),
                            part.rundown_id.unprotect(),
                            cache.playlist.doc_id().unprotect()
                        ));
                    }

                    let id = PartInstanceId::new_from(format!(
                        "{}_{}",
                        part.id.unprotect(),
                        get_random_id()
                    ));
                    let new_take_count = currentPartInstance
                        .as_ref()
                        .map(|inst| inst.take_count + 1)
                        .unwrap_or(0); // Increment

                    let segment_playout_id = currentPartInstance
                        .as_ref()
                        .and_then(|inst| {
                            if inst.segment_id == part.segment_id {
                                Some(inst.segment_playout_id.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| SegmentPlayoutId::new_from(get_random_id()));

                    cache
                        .part_instances
                        .insert(PartInstance {
                            id: id.clone(),
                            take_count: new_take_count,
                            rehearsal: cache.playlist.doc().rehearsal,
                            playlist_activation_id: activation_id,
                            rundown_id: part.rundown_id.clone(),
                            segment_id: part.segment_id.clone(),
                            segment_playout_id,
                            part: part.clone(),
                            consumes_next_segment_id: selected_part.consumes_next_segment_id,
                            timings: PartInstanceTimings {
                                set_as_next: Utc::now(),

                                planned_started_playback: None,
                                planned_stopped_playback: None,

                                take: None,
                                play_offset: None,
                            },

                            is_taken: false,
                            block_take_until: None,
                            part_playout_timings: None,
                            reset: false,
                            orphaned: crate::data_model::part_instance::PartInstanceOrphaned::No,
                        })
                        .map_err(|_| format!("Failed to create part instance"))?;

                    let rundown =
                        cache
                            .rundowns
                            .find_one_by_id(&part.rundown_id)
                            .ok_or_else(|| {
                                format!("Could not find rundown {}", part.rundown_id.unprotect())
                            })?;

                    let possible_pieces =
                        fetchPiecesThatMayBeActiveForPart(context, cache, None, &part).await?;
                    let new_piece_instances = getPieceInstancesForPart(
                        context,
                        cache,
                        currentPartInstance.as_ref(),
                        &rundown,
                        &part,
                        &possible_pieces,
                        &id,
                        false,
                    )?;
                    for piece_instance in new_piece_instances {
                        cache
                            .piece_instances
                            .insert(piece_instance)
                            .map_err(|_| format!("Failed to insert piece instance"))?;
                    }

                    id
                }
            }
        };

        // TODO
        // let selected_part_instance_ids = Vec::with_capacity(3);
        // 		const selectedPartInstanceIds = _.compact([
        // 			newInstanceId,
        // 			cache.Playlist.doc.currentPartInstanceId,
        // 			cache.Playlist.doc.previousPartInstanceId,
        // 		])

        // 		// reset any previous instances of this part
        // 		resetPartInstancesWithPieceInstances(context, cache, {
        // 			_id: { $nin: selectedPartInstanceIds },
        // 			rundownId: nextPart.rundownId,
        // 			'part._id': nextPart._id,
        // 		})

        // 		const nextPartInstanceTmp = nextPartInfo.type === 'partinstance' ? nextPartInfo.instance : null
        // 		cache.Playlist.update((p) => {
        // 			p.nextPartInstanceId = newInstanceId
        // 			p.nextPartManual = !!(setManually || nextPartInstanceTmp?.orphaned)
        // 			p.nextTimeOffset = nextTimeOffset || null
        // 			return p
        // 		})
    } else {
        // Set to null

        cache
            .playlist
            .update(|doc| {
                let mut res = doc.clone();

                res.next_part_instance_id = None;
                res.next_part_manual = setManually;
                res.next_time_offset = None;

                Some(res)
            })
            .map_err(|_| format!("failed to clear next part instance"))?;
    }

    {
        // Remove any instances which havent been taken
        let instances_ids_to_remove = cache
            .part_instances
            .remove_by_filter(|p| {
                !p.is_taken
                    && Some(&p.id) != cache.playlist.doc().next_part_instance_id.as_ref()
                    && Some(&p.id) != cache.playlist.doc().current_part_instance_id.as_ref()
            })
            .map_err(|_| format!("failed to find part instances to cleanup"))?;
        let instances_ids_to_remove_set =
            instances_ids_to_remove.into_iter().collect::<HashSet<_>>();

        cache
            .piece_instances
            .remove_by_filter(|p| instances_ids_to_remove_set.contains(&p.part_instance_id))
            .map_err(|_| format!("failed to cleanup piece instances"))?;
    }

    {
        let next_part_instance = cache.get_next_part_instance();
        // When entering a segment, or moving backwards in a segment, reset any partInstances in that window
        // In theory the new segment should already be reset, as we do that upon leaving, but it wont be if jumping to earlier in the same segment or maybe if the rundown wasnt reset
        if let Some(nextPartInstance) = next_part_instance {
            let current_part_instance = cache.get_current_part_instance();

            let mut resetPartInstanceIds = HashSet::new();

            if let Some(current_part_instance) = &current_part_instance {
                // Always clean the current segment, anything after the current part (except the next part)
                let trailing_in_old_segment = cache.part_instances.find_some(|p| {
                    !p.reset
                        && p.id != current_part_instance.id
                        && p.id != nextPartInstance.id
                        && p.segment_id == current_part_instance.segment_id
                        && p.part.rank > current_part_instance.part.rank
                });

                for part in trailing_in_old_segment {
                    resetPartInstanceIds.insert(part.id);
                }
            }

            let clean_next_segment = if let Some(current_part_instance) = &current_part_instance {
                nextPartInstance.segment_id != current_part_instance.segment_id
                    || (nextPartInstance.segment_id == current_part_instance.segment_id
                        && nextPartInstance.part.rank < current_part_instance.part.rank)
            } else {
                true
            };

            if clean_next_segment {
                // clean the whole segment if new, or jumping backwards

                let current_part_instance_id = current_part_instance.as_ref().map(|inst| &inst.id);
                let new_segment_parts = cache.part_instances.find_some(|p| {
                    !p.reset
                        && p.id != nextPartInstance.id
                        && Some(&p.id) != current_part_instance_id
                        && p.segment_id == nextPartInstance.segment_id
                });
                for part in new_segment_parts {
                    resetPartInstanceIds.insert(part.id);
                }
            }

            if resetPartInstanceIds.len() > 0 {
                // TODO
                // resetPartInstancesWithPieceInstances(context, cache, {
                // 	_id: { $in: Array.from(resetPartInstanceIds) },
                // })
            }
        }
    }

    // TODO
    // 	await cleanupOrphanedItems(context, cache)

    Ok(())
}
