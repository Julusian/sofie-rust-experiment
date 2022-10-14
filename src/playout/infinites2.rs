// /** When we crop a piece, set the piece as "it has definitely ended" this far into the future. */
// export const DEFINITELY_ENDED_FUTURE_DURATION = 1 * 1000

// /**
//  * We can only continue adlib onEnd infinites if we go forwards in the rundown. Any distance backwards will clear them.
//  * */
// export function canContinueAdlibOnEndInfinites(
// 	_context: JobContext,
// 	playlist: ReadonlyDeep<DBRundownPlaylist>,
// 	orderedSegments: DBSegment[],
// 	previousPartInstance: DBPartInstance | undefined,
// 	candidateInstance: DBPart
// ): boolean {
// 	if (previousPartInstance && playlist) {
// 		// When in the same segment, we can rely on the ranks to be in order. This is to handle orphaned parts, but is also valid for normal parts
// 		if (candidateInstance.segmentId === previousPartInstance.segmentId) {
// 			return candidateInstance._rank > previousPartInstance.part._rank
// 		} else {
// 			// Check if the segment is after the other
// 			const previousSegmentIndex = orderedSegments.findIndex((s) => s._id === previousPartInstance.segmentId)
// 			const candidateSegmentIndex = orderedSegments.findIndex((s) => s._id === candidateInstance.segmentId)

// 			if (previousSegmentIndex === -1 || candidateSegmentIndex === -1) {
// 				// Should never happen, as orphaned segments are kept around
// 				return false
// 			}

// 			return candidateSegmentIndex >= previousSegmentIndex
// 		}
// 	} else {
// 		// There won't be anything to continue anyway..
// 		return false
// 	}
// }

struct IdsBeforeThisPart {
    parts_before_this_in_segment: Vec<String>,
    segments_before_this_in_rundown: Vec<String>,
    rundowns_before_this_in_playlist: Vec<String>,
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

use itertools::Itertools;

use crate::{
    cache::{collection::DbCacheReadCollection, object::DbCacheReadObject},
    data_model::{
        part::Part,
        part_instance::{PartInstance, PartInstanceOrphaned},
        piece::Piece,
        piece_instance::PieceInstance,
        rundown::Rundown,
    },
};

use super::{
    cache::{FakeDoc, PlayoutCache},
    context::JobContext,
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
                        currentPartInstance.rundown_id
                    )
                })?;

            let ids_before_next_part = getIdsBeforeThisPart(context, cache, &nextPartInstance.part);

            let showStyleBase = context
                .get_show_style_base(&rundown.show_style_base_id)
                .await?;

            //
            todo!()

            // 		const orderedPartsAndSegments = getOrderedSegmentsAndPartsFromPlayoutCache(cache)

            // 		const canContinueAdlibOnEnds = canContinueAdlibOnEndInfinites(
            // 			context,
            // 			playlist,
            // 			orderedPartsAndSegments.segments,
            // 			currentPartInstance,
            // 			nextPartInstance.part
            // 		)
            // 		const playingPieceInstances = cache.PieceInstances.findAll((p) => p.partInstanceId === currentPartInstance._id)

            // 		const nowInPart = getCurrentTime() - (currentPartInstance.timings?.plannedStartedPlayback ?? 0)
            // 		const prunedPieceInstances = processAndPrunePieceInstanceTimings(
            // 			showStyleBase.sourceLayers,
            // 			playingPieceInstances,
            // 			nowInPart,
            // 			undefined,
            // 			true
            // 		)

            // 		const rundownIdsToShowstyleIds = getShowStyleIdsRundownMappingFromCache(cache)

            // 		const infinites = libgetPlayheadTrackingInfinitesForPart(
            // 			playlist.activationId,
            // 			new Set(partsBeforeThisInSegment),
            // 			new Set(segmentsBeforeThisInRundown),
            // 			rundownsBeforeThisInPlaylist,
            // 			rundownIdsToShowstyleIds,
            // 			currentPartInstance,
            // 			prunedPieceInstances,
            // 			rundown,
            // 			nextPartInstance.part,
            // 			nextPartInstance._id,
            // 			canContinueAdlibOnEnds,
            // 			false
            // 		)

            // 		saveIntoCache(
            // 			context,
            // 			cache.PieceInstances,
            // 			(p) => p.partInstanceId === nextPartInstance._id && !!p.infinite?.fromPreviousPlayhead,
            // 			infinites
            // 		)
        }
        _ => Ok(()),
    }
}

pub fn getPieceInstancesForPart(
    context: &JobContext,
    cache: &PlayoutCache,
    playingPartInstance: Option<&PartInstance>,
    rundown: &Rundown,
    part: &Part,
    possiblePieces: &[Piece],
    newInstanceId: &str,
    isTemporary: bool,
) -> Vec<PieceInstance> {
    todo!()
    // 	const span = context.startSpan('getPieceInstancesForPart')
    // 	const { partsBeforeThisInSegment, segmentsBeforeThisInRundown, rundownsBeforeThisInPlaylist } =
    // 		getIdsBeforeThisPart(context, cache, part)

    // 	const playlist = cache.Playlist.doc
    // 	if (!playlist.activationId) throw new Error(`RundownPlaylist "${playlist._id}" is not active`)

    // 	const orderedPartsAndSegments = getOrderedSegmentsAndPartsFromPlayoutCache(cache)
    // 	const playingPieceInstances = playingPartInstance
    // 		? cache.PieceInstances.findAll((p) => p.partInstanceId === playingPartInstance._id)
    // 		: []

    // 	const canContinueAdlibOnEnds = canContinueAdlibOnEndInfinites(
    // 		context,
    // 		playlist,
    // 		orderedPartsAndSegments.segments,
    // 		playingPartInstance,
    // 		part
    // 	)

    // 	const rundownIdsToShowstyleIds = getShowStyleIdsRundownMappingFromCache(cache)

    // 	const res = libgetPieceInstancesForPart(
    // 		playlist.activationId,
    // 		playingPartInstance,
    // 		playingPieceInstances,
    // 		rundown,
    // 		part,
    // 		new Set(partsBeforeThisInSegment),
    // 		new Set(segmentsBeforeThisInRundown),
    // 		rundownsBeforeThisInPlaylist,
    // 		rundownIdsToShowstyleIds,
    // 		possiblePieces,
    // 		orderedPartsAndSegments.parts.map((p) => p._id),
    // 		newInstanceId,
    // 		canContinueAdlibOnEnds,
    // 		isTemporary
    // 	)
    // 	if (span) span.end()
    // 	return res
}
