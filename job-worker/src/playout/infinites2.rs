use std::{collections::HashSet, future::ready};

use chrono::{Duration, Utc};
use futures::future::LocalBoxFuture;
use itertools::Itertools;
use mongodb::bson::doc;
use ordered_float::OrderedFloat;
use tokio::join;

use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollection},
        object::DbCacheReadObject,
    },
    context::{context::JobContext, direct_collections::MongoReadOnlyCollection},
    data_model::{
        ids::{PartId, PartInstanceId, ProtectedId, RundownId, SegmentId},
        part::Part,
        part_instance::{PartInstance, PartInstanceOrphaned},
        piece::Piece,
        piece_instance::PieceInstance,
        rundown::Rundown,
        rundown_playlist::RundownPlaylist,
        segment::Segment,
    },
    ingest::cache::IngestCache,
};

use super::{
    cache::{FakeDoc, PlayoutCache},
    infinites::{
        buildPastInfinitePiecesForThisPartQuery, getPieceInstancesForPart2,
        getPlayheadTrackingInfinitesForPart, processAndPrunePieceInstanceTimings,
    },
    playlist::sortRundownIDsInPlaylist,
};

/** When we crop a piece, set the piece as "it has definitely ended" this far into the future. */
// export const DEFINITELY_ENDED_FUTURE_DURATION = 1 * 1000

/**
 * We can only continue adlib onEnd infinites if we go forwards in the rundown. Any distance backwards will clear them.
 * */
pub fn canContinueAdlibOnEndInfinites(
    _context: &JobContext,
    _playlist: &RundownPlaylist,
    ordered_segments: &[Segment],
    previous_part_instance: Option<&PartInstance>,
    candidate_instance: &Part,
) -> bool {
    if let Some(previous_part_instance) = previous_part_instance {
        // When in the same segment, we can rely on the ranks to be in order. This is to handle orphaned parts, but is also valid for normal parts
        if candidate_instance.segment_id == previous_part_instance.segment_id {
            candidate_instance.rank > previous_part_instance.part.rank
        } else {
            // Check if the segment is after the other
            let previous_segment_index = ordered_segments
                .iter()
                .position(|s| s.id == previous_part_instance.segment_id);
            let candidate_segment_index = ordered_segments
                .iter()
                .position(|s| s.id == candidate_instance.segment_id);

            match (previous_segment_index, candidate_segment_index) {
                (Some(previous_segment_index), Some(candidate_segment_index)) => {
                    candidate_segment_index >= previous_segment_index
                }
                _ => {
                    // Should never happen, as orphaned segments are kept around
                    false
                }
            }
        }
    } else {
        // There won't be anything to continue anyway..
        false
    }
}

struct IdsBeforeThisPart {
    parts_before_this_in_segment: Vec<PartId>,
    segments_before_this_in_rundown: Vec<SegmentId>,
    rundowns_before_this_in_playlist: Vec<RundownId>,
}

fn getIdsBeforeThisPart(
    _context: &JobContext,
    cache: &PlayoutCache,
    next_part: &Part,
) -> IdsBeforeThisPart {
    // Get the normal parts
    let mut parts_before_this_in_segment = cache
        .parts
        .find_some(|p| p.segment_id == next_part.segment_id && p.rank < next_part.rank);

    // Find any orphaned parts
    let part_instances_before_this_in_segment = cache.part_instances.find_some(|p| {
        p.segment_id == next_part.segment_id
            && p.orphaned == PartInstanceOrphaned::No
            && p.part.rank < next_part.rank
    });
    parts_before_this_in_segment.extend(
        part_instances_before_this_in_segment
            .into_iter()
            .map(|p| p.part),
    );

    let segments_before_this_in_rundown = cache
        .segments
        .find_one_by_id(&next_part.segment_id)
        .map(|current_segment| {
            cache
                .segments
                .find_some(|s| {
                    s.rundown_id == next_part.rundown_id && s.rank < current_segment.rank
                })
                .into_iter()
                .map(|s| s.id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| Vec::new());

    let sorted_rundown_ids = sortRundownIDsInPlaylist(
        &cache.playlist.doc().rundown_ids_in_order,
        cache.get_rundown_ids_from_cache(),
    );
    let current_rundown_index = sorted_rundown_ids
        .iter()
        .position(|r| r == &next_part.rundown_id);

    let rundowns_before_this_in_playlist = current_rundown_index.map_or_else(
        || Vec::new(),
        |index| sorted_rundown_ids.into_iter().take(index).collect(),
    );

    let parts_before_this_in_segment_sorted = parts_before_this_in_segment
        .into_iter()
        .sorted_by_key(|p| OrderedFloat(p.rank))
        .map(|p| p.id)
        .collect();

    IdsBeforeThisPart {
        parts_before_this_in_segment: parts_before_this_in_segment_sorted,
        segments_before_this_in_rundown,
        rundowns_before_this_in_playlist,
    }
}

pub async fn fetchPiecesThatMayBeActiveForPart(
    context: &JobContext,
    cache: &PlayoutCache,
    unsaved_ingest_cache: Option<&IngestCache>,
    part: &Part,
) -> Result<Vec<Piece>, String> {
    let unsaved_ingest_cache = unsaved_ingest_cache.and_then(|cache| {
        if cache.rundown.doc_id() == &part.rundown_id {
            Some(cache)
        } else {
            None
        }
    });

    // Find all the pieces starting in the part
    let pieces_for_part: LocalBoxFuture<Result<Vec<Piece>, String>> =
        if let Some(unsaved_ingest_cache) = unsaved_ingest_cache {
            Box::pin(ready(Ok(unsaved_ingest_cache
                .pieces
                .find_some(|p| p.start_part_id == part.id))))
        } else {
            let this_pieces_query = doc! { "startPartId": part.id.unprotect() };
            context
                .direct_collections()
                .pieces
                .find_fetch(this_pieces_query, None)
        };

    // Figure out the ids of everything else we will have to search through
    let ids_before_part = getIdsBeforeThisPart(context, cache, part);

    if let Some(unsaved_ingest_cache) = unsaved_ingest_cache {
        // Find pieces for the current rundown
        let thisRundownPieceQuery = buildPastInfinitePiecesForThisPartQuery(
            part,
            &ids_before_part.parts_before_this_in_segment,
            &ids_before_part.segments_before_this_in_rundown,
            &Vec::new(), // other rundowns don't exist in the ingestCache
        );
        let mut this_rundown_pieces: Vec<Piece> =
            if let Some(this_rundown_piece_query) = thisRundownPieceQuery {
                unsaved_ingest_cache.pieces.find_some(|p| todo!())
            } else {
                Vec::new()
            };

        // Find pieces for the previous rundowns
        let previousRundownPieceQuery = buildPastInfinitePiecesForThisPartQuery(
            part,
            &Vec::new(), // Only applies to the current rundown
            &Vec::new(), // Only applies to the current rundown
            &ids_before_part.rundowns_before_this_in_playlist,
        );
        let previous_rundown_pieces: LocalBoxFuture<Result<Vec<Piece>, String>> =
            if let Some(previous_rundown_piece_query) = previousRundownPieceQuery {
                context
                    .direct_collections()
                    .pieces
                    .find_fetch(previous_rundown_piece_query, None)
            } else {
                Box::pin(ready(Ok(Vec::new())))
            };

        let results = join!(pieces_for_part, previous_rundown_pieces);

        let mut all_pieces = results.0?;
        let mut previous_rundown_pieces = results.1?;

        if this_rundown_pieces.len() > 0 {
            all_pieces.append(&mut this_rundown_pieces);
        }
        if previous_rundown_pieces.len() > 0 {
            all_pieces.append(&mut previous_rundown_pieces);
        }

        Ok(all_pieces)
    } else {
        // No cache, so we can do a single query to the db for it all
        let infinite_pieces_query = buildPastInfinitePiecesForThisPartQuery(
            part,
            &ids_before_part.parts_before_this_in_segment,
            &ids_before_part.segments_before_this_in_rundown,
            &ids_before_part.rundowns_before_this_in_playlist,
        );

        let infinite_pieces: LocalBoxFuture<Result<Vec<Piece>, String>> =
            if let Some(infinite_pieces_query) = infinite_pieces_query {
                context
                    .direct_collections()
                    .pieces
                    .find_fetch(infinite_pieces_query, None)
            } else {
                Box::pin(ready(Ok(Vec::new())))
            };

        let results = join!(pieces_for_part, infinite_pieces);

        let mut all_pieces = results.0?;
        let mut infinites = results.1?;

        if infinites.len() > 0 {
            all_pieces.append(&mut infinites);
        }

        Ok(all_pieces)
    }
}

pub async fn syncPlayheadInfinitesForNextPartInstance(
    context: &JobContext,
    cache: &mut PlayoutCache,
) -> Result<(), String> {
    let next_part_instance = cache.get_next_part_instance();
    let current_part_instance = cache.get_current_part_instance();

    match (next_part_instance, current_part_instance) {
        (Some(next_part_instance), Some(current_part_instance)) => {
            let activation_id = cache.playlist.doc().activation_id.clone().ok_or_else(|| {
                format!(
                    "RundownPlaylist \"{}\" is not active",
                    cache.playlist.doc_id().unprotect()
                )
            })?;

            let rundown = cache
                .rundowns
                .find_one_by_id(&current_part_instance.rundown_id)
                .ok_or_else(|| {
                    format!(
                        "Rundown \"{}\" is not active",
                        current_part_instance.rundown_id.unprotect()
                    )
                })?;

            let ids_before_next_part =
                getIdsBeforeThisPart(context, cache, &next_part_instance.part);

            let show_style_base = context
                .get_show_style_base(&rundown.show_style_base_id)
                .await?;

            let ordered_parts_and_segments = cache.get_ordered_segments_and_parts();

            let can_continue_adlib_on_ends = canContinueAdlibOnEndInfinites(
                context,
                cache.playlist.doc(),
                &ordered_parts_and_segments.segments,
                Some(&current_part_instance),
                &next_part_instance.part,
            );
            let playing_piece_instances = cache
                .piece_instances
                .find_some(|p| p.part_instance_id == current_part_instance.id);

            let now_in_part = current_part_instance
                .timings
                .planned_started_playback
                .map_or(Duration::zero(), |start| Utc::now() - start);
            let pruned_piece_instances = processAndPrunePieceInstanceTimings(
                &show_style_base.source_layers,
                &playing_piece_instances,
                now_in_part,
                false,
                true,
            );

            let rundown_ids_to_showstyle_ids =
                cache.get_show_style_ids_rundown_mapping_from_cache();

            let infinites = getPlayheadTrackingInfinitesForPart(
                &activation_id,
                &HashSet::from_iter(
                    ids_before_next_part
                        .parts_before_this_in_segment
                        .into_iter(),
                ),
                &HashSet::from_iter(
                    ids_before_next_part
                        .segments_before_this_in_rundown
                        .into_iter(),
                ),
                &ids_before_next_part.rundowns_before_this_in_playlist,
                &rundown_ids_to_showstyle_ids,
                &current_part_instance,
                &pruned_piece_instances
                    .into_iter()
                    .map(|p| p.piece)
                    .collect_vec(),
                &rundown,
                &next_part_instance.part,
                &next_part_instance.id,
                can_continue_adlib_on_ends,
                false,
            );

            cache
                .piece_instances
                .save_into(
                    |p| {
                        p.part_instance_id == next_part_instance.id
                            && p.infinite
                                .as_ref()
                                .map_or(false, |inf| inf.from_previous_playhead)
                    },
                    infinites,
                )
                .map_err(|_e| format!("Failed to save new infinites"))?;

            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn getPieceInstancesForPart(
    context: &JobContext,
    cache: &PlayoutCache,
    playing_part_instance: Option<&PartInstance>,
    rundown: &Rundown,
    part: &Part,
    possible_pieces: &[Piece],
    new_instance_id: &PartInstanceId,
    is_temporary: bool,
) -> Result<Vec<PieceInstance>, String> {
    let ids_before_next_part = getIdsBeforeThisPart(context, cache, part);

    let activation_id = cache.playlist.doc().activation_id.clone().ok_or_else(|| {
        format!(
            "RundownPlaylist \"{}\" is not active",
            cache.playlist.doc_id().unprotect()
        )
    })?;

    let ordered_parts_and_segments = cache.get_ordered_segments_and_parts();
    let playing_piece_instances = playing_part_instance.map_or(Vec::new(), |instance| {
        cache
            .piece_instances
            .find_some(|p| p.part_instance_id == instance.id)
    });

    let can_continue_adlib_on_ends = canContinueAdlibOnEndInfinites(
        context,
        cache.playlist.doc(),
        &ordered_parts_and_segments.segments,
        playing_part_instance,
        part,
    );

    let rundown_ids_to_showstyle_ids = cache.get_show_style_ids_rundown_mapping_from_cache();

    Ok(getPieceInstancesForPart2(
        activation_id,
        playing_part_instance,
        &playing_piece_instances,
        rundown,
        part,
        &HashSet::from_iter(
            ids_before_next_part
                .parts_before_this_in_segment
                .into_iter(),
        ),
        &HashSet::from_iter(
            ids_before_next_part
                .segments_before_this_in_rundown
                .into_iter(),
        ),
        &ids_before_next_part.rundowns_before_this_in_playlist,
        &rundown_ids_to_showstyle_ids,
        &possible_pieces,
        &ordered_parts_and_segments
            .parts
            .into_iter()
            .map(|p| p.id)
            .collect_vec(),
        new_instance_id.clone(),
        can_continue_adlib_on_ends,
        is_temporary,
    ))
}
