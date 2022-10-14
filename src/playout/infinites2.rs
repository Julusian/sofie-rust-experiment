// /** When we crop a piece, set the piece as "it has definitely ended" this far into the future. */
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
        .map(|currentSegment| {
            cache
                .segments
                .find_some(|s| s.rundown_id == next_part.rundown_id && s.rank < currentSegment.rank)
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
        .sorted_by_key(|p| p.rank)
        .map(|p| p.id)
        .collect();

    IdsBeforeThisPart {
        parts_before_this_in_segment: parts_before_this_in_segment_sorted,
        segments_before_this_in_rundown,
        rundowns_before_this_in_playlist,
    }
}

use std::collections::HashSet;

use chrono::{Duration, Utc};
use itertools::Itertools;

use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollection},
        object::DbCacheReadObject,
    },
    data_model::{
        ids::{PartId, PartInstanceId, RundownId, SegmentId},
        part::Part,
        part_instance::{PartInstance, PartInstanceOrphaned},
        piece::Piece,
        piece_instance::PieceInstance,
        rundown::Rundown,
        rundown_playlist::RundownPlaylist,
        segment::Segment,
    },
};

use super::{
    cache::{FakeDoc, PlayoutCache},
    context::JobContext,
    infinites::{
        getPieceInstancesForPart2, getPlayheadTrackingInfinitesForPart,
        processAndPrunePieceInstanceTimings,
    },
    playlist::sortRundownIDsInPlaylist,
};

pub async fn fetchPiecesThatMayBeActiveForPart(
    context: &JobContext,
    cache: &PlayoutCache,
    unsavedIngestCache: Option<FakeDoc>, // Omit<ReadOnlyCache<CacheForIngest>, 'Rundown'> | undefined,
    part: &Part,
) -> Result<Vec<Piece>, String> {
    todo!()
    // 	const span = context.startSpan('fetchPiecesThatMayBeActiveForPart')

    // 	const piecePromises: Array<Promise<Array<Piece>> | Array<Piece>> = []

    // 	// Find all the pieces starting in the part
    // 	const thisPiecesQuery = buildPiecesStartingInThisPartQuery(part)
    // 	piecePromises.push(
    // 		unsavedIngestCache?.RundownId === part.rundownId
    // 			? unsavedIngestCache.Pieces.findAll((p) => mongoWhere(p, thisPiecesQuery))
    // 			: context.directCollections.Pieces.findFetch(thisPiecesQuery)
    // 	)

    // 	// Figure out the ids of everything else we will have to search through
    // 	const { partsBeforeThisInSegment, segmentsBeforeThisInRundown, rundownsBeforeThisInPlaylist } =
    // 		getIdsBeforeThisPart(context, cache, part)

    // 	if (unsavedIngestCache?.RundownId === part.rundownId) {
    // 		// Find pieces for the current rundown
    // 		const thisRundownPieceQuery = buildPastInfinitePiecesForThisPartQuery(
    // 			part,
    // 			partsBeforeThisInSegment,
    // 			segmentsBeforeThisInRundown,
    // 			[] // other rundowns don't exist in the ingestCache
    // 		)
    // 		if (thisRundownPieceQuery) {
    // 			piecePromises.push(unsavedIngestCache.Pieces.findAll((p) => mongoWhere(p, thisRundownPieceQuery)))
    // 		}

    // 		// Find pieces for the previous rundowns
    // 		const previousRundownPieceQuery = buildPastInfinitePiecesForThisPartQuery(
    // 			part,
    // 			[], // Only applies to the current rundown
    // 			[], // Only applies to the current rundown
    // 			rundownsBeforeThisInPlaylist
    // 		)
    // 		if (previousRundownPieceQuery) {
    // 			piecePromises.push(context.directCollections.Pieces.findFetch(previousRundownPieceQuery))
    // 		}
    // 	} else {
    // 		// No cache, so we can do a single query to the db for it all
    // 		const infinitePiecesQuery = buildPastInfinitePiecesForThisPartQuery(
    // 			part,
    // 			partsBeforeThisInSegment,
    // 			segmentsBeforeThisInRundown,
    // 			rundownsBeforeThisInPlaylist
    // 		)
    // 		if (infinitePiecesQuery) {
    // 			piecePromises.push(context.directCollections.Pieces.findFetch(infinitePiecesQuery))
    // 		}
    // 	}

    // 	const pieces = flatten(await Promise.all(piecePromises))
    // 	if (span) span.end()
    // 	return pieces
}

pub async fn syncPlayheadInfinitesForNextPartInstance(
    context: &JobContext,
    cache: &mut PlayoutCache,
) -> Result<(), String> {
    let nextPartInstance = cache.get_next_part_instance();
    let currentPartInstance = cache.get_current_part_instance();

    match (nextPartInstance, currentPartInstance) {
        (Some(nextPartInstance), Some(currentPartInstance)) => {
            let activation_id = cache.playlist.doc().activation_id.clone().ok_or_else(|| {
                format!("RundownPlaylist \"{}\" is not active", cache.playlist_id)
            })?;

            let rundown = cache
                .rundowns
                .find_one_by_id(&currentPartInstance.rundown_id)
                .ok_or_else(|| {
                    format!(
                        "Rundown \"{}\" is not active",
                        currentPartInstance.rundown_id.unprotect()
                    )
                })?;

            let ids_before_next_part = getIdsBeforeThisPart(context, cache, &nextPartInstance.part);

            let showStyleBase = context
                .get_show_style_base(&rundown.show_style_base_id)
                .await?;

            let orderedPartsAndSegments = cache.get_ordered_segments_and_parts();

            let canContinueAdlibOnEnds = canContinueAdlibOnEndInfinites(
                context,
                cache.playlist.doc(),
                &orderedPartsAndSegments.segments,
                Some(&currentPartInstance),
                &nextPartInstance.part,
            );
            let playingPieceInstances = cache
                .piece_instances
                .find_some(|p| p.part_instance_id == currentPartInstance.id);

            let nowInPart = currentPartInstance
                .timings
                .planned_started_playback
                .map_or(Duration::zero(), |start| Utc::now() - start);
            let prunedPieceInstances = processAndPrunePieceInstanceTimings(
                &showStyleBase.source_layers,
                &playingPieceInstances,
                nowInPart,
                false,
                true,
            );

            let rundownIdsToShowstyleIds = cache.get_show_style_ids_rundown_mapping_from_cache();

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
                &rundownIdsToShowstyleIds,
                &currentPartInstance,
                &prunedPieceInstances
                    .into_iter()
                    .map(|p| p.piece)
                    .collect_vec(),
                &rundown,
                &nextPartInstance.part,
                &nextPartInstance.id,
                canContinueAdlibOnEnds,
                false,
            );

            cache
                .piece_instances
                .save_into(
                    |p| {
                        p.part_instance_id == nextPartInstance.id
                            && p.infinite
                                .as_ref()
                                .map_or(false, |inf| inf.from_previous_playhead)
                    },
                    infinites,
                )
                .map_err(|e| format!("Failed to save new infinites"))?;

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

    let activation_id = cache
        .playlist
        .doc()
        .activation_id
        .clone()
        .ok_or_else(|| format!("RundownPlaylist \"{}\" is not active", cache.playlist_id))?;

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
