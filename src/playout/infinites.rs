use std::collections::{HashMap, HashSet};

use chrono::Duration;
use itertools::Itertools;

use crate::data_model::{
    extra::get_piece_control_object_id,
    piece::{PieceEnableStart, PieceLifespan},
    piece_instance::PieceInstance,
    show_style_base::SourceLayers,
};

// export function buildPiecesStartingInThisPartQuery(part: DBPart): MongoQuery<Piece> {
// 	return { startPartId: part._id }
// }

// export function buildPastInfinitePiecesForThisPartQuery(
// 	part: DBPart,
// 	partsIdsBeforeThisInSegment: PartId[],
// 	segmentsIdsBeforeThisInRundown: SegmentId[],
// 	rundownIdsBeforeThisInPlaylist: RundownId[]
// ): MongoQuery<Piece> | null {
// 	const fragments = _.compact([
// 		partsIdsBeforeThisInSegment.length > 0
// 			? {
// 					// same segment, and previous part
// 					lifespan: {
// 						$in: [
// 							PieceLifespan.OutOnSegmentEnd,
// 							PieceLifespan.OutOnSegmentChange,
// 							PieceLifespan.OutOnRundownEnd,
// 							PieceLifespan.OutOnRundownChange,
// 							PieceLifespan.OutOnShowStyleEnd,
// 						],
// 					},
// 					startRundownId: part.rundownId,
// 					startSegmentId: part.segmentId,
// 					startPartId: { $in: partsIdsBeforeThisInSegment },
// 			  }
// 			: undefined,
// 		segmentsIdsBeforeThisInRundown.length > 0
// 			? {
// 					// same rundown, and previous segment
// 					lifespan: {
// 						$in: [
// 							PieceLifespan.OutOnRundownEnd,
// 							PieceLifespan.OutOnRundownChange,
// 							PieceLifespan.OutOnShowStyleEnd,
// 						],
// 					},
// 					startRundownId: part.rundownId,
// 					startSegmentId: { $in: segmentsIdsBeforeThisInRundown },
// 			  }
// 			: undefined,
// 		rundownIdsBeforeThisInPlaylist.length > 0
// 			? {
// 					// previous rundown
// 					lifespan: {
// 						$in: [PieceLifespan.OutOnShowStyleEnd],
// 					},
// 					startRundownId: { $in: rundownIdsBeforeThisInPlaylist },
// 			  }
// 			: undefined,
// 	])

// 	if (fragments.length === 0) {
// 		return null
// 	} else if (fragments.length === 1) {
// 		return {
// 			invalid: { $ne: true },
// 			startPartId: { $ne: part._id },
// 			...fragments[0],
// 		}
// 	} else {
// 		return {
// 			invalid: { $ne: true },
// 			startPartId: { $ne: part._id },
// 			$or: fragments,
// 		}
// 	}
// }

pub fn getPlayheadTrackingInfinitesForPart(
    playlistActivationId: String,
    partsBeforeThisInSegmentSet: &HashSet<String>,
    segmentsBeforeThisInRundownSet: &HashSet<String>,
    // rundownsBeforeThisInPlaylist: RundownId[],
    // rundownsToShowstyles: Map<RundownId, ShowStyleBaseId>,
    // currentPartInstance: DBPartInstance,
    // currentPartPieceInstances: PieceInstance[],
    // rundown: ReadonlyDeep<Pick<DBRundown, '_id' | 'showStyleBaseId'>>,
    // part: DBPart,
    // newInstanceId: PartInstanceId,
    // nextPartIsAfterCurrentPart: boolean,
    // isTemporary: boolean
) -> Vec<PieceInstance> {
    // 	const canContinueAdlibOnEnds = nextPartIsAfterCurrentPart
    // 	interface InfinitePieceSet {
    // 		[PieceLifespan.OutOnShowStyleEnd]?: PieceInstance
    // 		[PieceLifespan.OutOnRundownEnd]?: PieceInstance
    // 		[PieceLifespan.OutOnSegmentEnd]?: PieceInstance
    // 		onChange?: PieceInstance
    // 	}
    // 	const piecesOnSourceLayers = new Map<string, InfinitePieceSet>()

    // 	const canContinueShowStyleEndInfinites = continueShowStyleEndInfinites(
    // 		rundownsBeforeThisInPlaylist,
    // 		rundownsToShowstyles,
    // 		currentPartInstance.rundownId,
    // 		rundown
    // 	)

    // 	const groupedPlayingPieceInstances = _.groupBy(currentPartPieceInstances, (p) => p.piece.sourceLayerId)
    // 	for (const [sourceLayerId, pieceInstances] of Object.entries(groupedPlayingPieceInstances)) {
    // 		// Find the ones that starts last. Note: any piece will stop an onChange
    // 		const lastPiecesByStart = _.groupBy(pieceInstances, (p) => p.piece.enable.start)
    // 		let lastPieceInstances = lastPiecesByStart['now'] || []
    // 		if (lastPieceInstances.length === 0) {
    // 			const target = max(Object.keys(lastPiecesByStart), (k) => Number(k))
    // 			if (target !== undefined) {
    // 				lastPieceInstances = lastPiecesByStart[target] || []
    // 			}
    // 		}

    // 		// Some basic resolving, to figure out which is our candidate
    // 		let lastPieceInstance: PieceInstance | undefined
    // 		for (const candidate of lastPieceInstances) {
    // 			if (lastPieceInstance === undefined || isCandidateBetterToBeContinued(lastPieceInstance, candidate)) {
    // 				lastPieceInstance = candidate
    // 			}
    // 		}

    // 		if (lastPieceInstance && !lastPieceInstance.plannedStoppedPlayback && !lastPieceInstance.userDuration) {
    // 			// If it is an onChange, then it may want to continue
    // 			let isUsed = false
    // 			switch (lastPieceInstance.piece.lifespan) {
    // 				case PieceLifespan.OutOnSegmentChange:
    // 					if (currentPartInstance.segmentId === part.segmentId) {
    // 						// Still in the same segment
    // 						isUsed = true
    // 					}
    // 					break
    // 				case PieceLifespan.OutOnRundownChange:
    // 					if (lastPieceInstance.rundownId === part.rundownId) {
    // 						// Still in the same rundown
    // 						isUsed = true
    // 					}
    // 					break
    // 			}

    // 			if (isUsed) {
    // 				const pieceSet = piecesOnSourceLayers.get(sourceLayerId) ?? {}
    // 				pieceSet.onChange = lastPieceInstance
    // 				piecesOnSourceLayers.set(sourceLayerId, pieceSet)
    // 				// This may get pruned later, if somethng else has a start of 0
    // 			}
    // 		}

    // 		// Check if we should persist any adlib onEnd infinites
    // 		if (canContinueAdlibOnEnds) {
    // 			const piecesByInfiniteMode = _.groupBy(
    // 				pieceInstances.filter((p) => p.dynamicallyInserted),
    // 				(p) => p.piece.lifespan
    // 			)
    // 			for (const mode0 of [
    // 				PieceLifespan.OutOnRundownEnd,
    // 				PieceLifespan.OutOnSegmentEnd,
    // 				PieceLifespan.OutOnShowStyleEnd,
    // 			]) {
    // 				const mode = mode0 as
    // 					| PieceLifespan.OutOnRundownEnd
    // 					| PieceLifespan.OutOnSegmentEnd
    // 					| PieceLifespan.OutOnShowStyleEnd
    // 				const pieces = (piecesByInfiniteMode[mode] || []).filter(
    // 					(p) => p.infinite && (p.infinite.fromPreviousPlayhead || p.dynamicallyInserted)
    // 				)
    // 				// This is the piece we may copy across
    // 				const candidatePiece =
    // 					pieces.find((p) => p.piece.enable.start === 'now') ?? max(pieces, (p) => p.piece.enable.start)
    // 				if (candidatePiece && !candidatePiece.plannedStoppedPlayback && !candidatePiece.userDuration) {
    // 					// Check this infinite is allowed to continue to this part
    // 					let isValid = false
    // 					switch (mode) {
    // 						case PieceLifespan.OutOnSegmentEnd:
    // 							isValid =
    // 								currentPartInstance.segmentId === part.segmentId &&
    // 								partsBeforeThisInSegmentSet.has(candidatePiece.piece.startPartId)
    // 							break
    // 						case PieceLifespan.OutOnRundownEnd:
    // 							isValid =
    // 								candidatePiece.rundownId === part.rundownId &&
    // 								(segmentsBeforeThisInRundownSet.has(currentPartInstance.segmentId) ||
    // 									currentPartInstance.segmentId === part.segmentId)
    // 							break
    // 						case PieceLifespan.OutOnShowStyleEnd:
    // 							isValid = canContinueShowStyleEndInfinites
    // 					}

    // 					if (isValid) {
    // 						const pieceSet = piecesOnSourceLayers.get(sourceLayerId) ?? {}
    // 						pieceSet[mode] = candidatePiece
    // 						piecesOnSourceLayers.set(sourceLayerId, pieceSet)
    // 					}
    // 				}
    // 			}
    // 		}
    // 	}

    // 	const rewrapInstance = (p: PieceInstance | undefined): PieceInstance | undefined => {
    // 		if (p) {
    // 			const instance = rewrapPieceToInstance(
    // 				p.piece,
    // 				playlistActivationId,
    // 				part.rundownId,
    // 				newInstanceId,
    // 				isTemporary
    // 			)
    // 			markPieceInstanceAsContinuation(p, instance)

    // 			if (p.infinite) {
    // 				// This was copied from before, so we know we can force the time to 0
    // 				instance.piece = {
    // 					...instance.piece,
    // 					enable: {
    // 						start: 0,
    // 					},
    // 				}
    // 				instance.infinite = {
    // 					...p.infinite,
    // 					infiniteInstanceIndex: p.infinite.infiniteInstanceIndex + 1,
    // 					fromPreviousPart: true,
    // 					fromPreviousPlayhead: true,
    // 				}

    // 				return instance
    // 			}
    // 		}
    // 		return undefined
    // 	}

    // 	return flatten(
    // 		Array.from(piecesOnSourceLayers.values()).map((ps) => {
    // 			return _.compact(Object.values(ps).map(rewrapInstance))
    // 		})
    // 	)
}

// function markPieceInstanceAsContinuation(previousInstance: PieceInstance, instance: PieceInstance) {
// 	instance._id = protectString(`${instance._id}_continue`)
// 	instance.dynamicallyInserted = previousInstance.dynamicallyInserted
// 	instance.adLibSourceId = previousInstance.adLibSourceId
// 	instance.reportedStartedPlayback = previousInstance.reportedStartedPlayback
// }

// export function isPiecePotentiallyActiveInPart(
// 	previousPartInstance: DBPartInstance | undefined,
// 	partsBeforeThisInSegment: Set<PartId>,
// 	segmentsBeforeThisInRundown: Set<SegmentId>,
// 	rundownsBeforeThisInPlaylist: RundownId[],
// 	rundownsToShowstyles: Map<RundownId, ShowStyleBaseId>,
// 	rundown: ReadonlyDeep<Pick<DBRundown, '_id' | 'showStyleBaseId'>>,
// 	part: DBPart,
// 	pieceToCheck: Piece
// ): boolean {
// 	// If its from the current part
// 	if (pieceToCheck.startPartId === part._id) {
// 		return true
// 	}

// 	switch (pieceToCheck.lifespan) {
// 		case PieceLifespan.WithinPart:
// 			// This must be from another part
// 			return false
// 		case PieceLifespan.OutOnSegmentEnd:
// 			return (
// 				pieceToCheck.startSegmentId === part.segmentId && partsBeforeThisInSegment.has(pieceToCheck.startPartId)
// 			)
// 		case PieceLifespan.OutOnRundownEnd:
// 			if (pieceToCheck.startRundownId === part.rundownId) {
// 				if (pieceToCheck.startSegmentId === part.segmentId) {
// 					return partsBeforeThisInSegment.has(pieceToCheck.startPartId)
// 				} else {
// 					return segmentsBeforeThisInRundown.has(pieceToCheck.startSegmentId)
// 				}
// 			} else {
// 				return false
// 			}
// 		case PieceLifespan.OutOnSegmentChange:
// 			if (previousPartInstance !== undefined) {
// 				// This gets handled by getPlayheadTrackingInfinitesForPart
// 				// We will only copy the pieceInstance from the previous, never using the original piece
// 				return false
// 			} else {
// 				// Predicting what will happen at arbitrary point in the future
// 				return (
// 					pieceToCheck.startSegmentId === part.segmentId &&
// 					partsBeforeThisInSegment.has(pieceToCheck.startPartId)
// 				)
// 			}
// 		case PieceLifespan.OutOnRundownChange:
// 			if (previousPartInstance !== undefined) {
// 				// This gets handled by getPlayheadTrackingInfinitesForPart
// 				// We will only copy the pieceInstance from the previous, never using the original piece
// 				return false
// 			} else {
// 				// Predicting what will happen at arbitrary point in the future
// 				return (
// 					pieceToCheck.startRundownId === part.rundownId &&
// 					segmentsBeforeThisInRundown.has(pieceToCheck.startSegmentId)
// 				)
// 			}
// 		case PieceLifespan.OutOnShowStyleEnd:
// 			return previousPartInstance && pieceToCheck.lifespan === PieceLifespan.OutOnShowStyleEnd
// 				? continueShowStyleEndInfinites(
// 						rundownsBeforeThisInPlaylist,
// 						rundownsToShowstyles,
// 						previousPartInstance.rundownId,
// 						rundown
// 				  )
// 				: false
// 		default:
// 			assertNever(pieceToCheck.lifespan)
// 			return false
// 	}
// }

// export function getPieceInstancesForPart(
// 	playlistActivationId: RundownPlaylistActivationId,
// 	playingPartInstance: DBPartInstance | undefined,
// 	playingPieceInstances: PieceInstance[] | undefined,
// 	rundown: ReadonlyDeep<Pick<DBRundown, '_id' | 'showStyleBaseId'>>,
// 	part: DBPart,
// 	partsBeforeThisInSegmentSet: Set<PartId>,
// 	segmentsBeforeThisInRundownSet: Set<SegmentId>,
// 	rundownsBeforeThisInPlaylist: RundownId[],
// 	rundownsToShowstyles: Map<RundownId, ShowStyleBaseId>,
// 	possiblePieces: Piece[],
// 	orderedPartIds: PartId[],
// 	newInstanceId: PartInstanceId,
// 	nextPartIsAfterCurrentPart: boolean,
// 	isTemporary: boolean
// ): PieceInstance[] {
// 	const doesPieceAStartBeforePieceB = (pieceA: PieceInstancePiece, pieceB: PieceInstancePiece): boolean => {
// 		if (pieceA.startPartId === pieceB.startPartId) {
// 			return pieceA.enable.start < pieceB.enable.start
// 		}
// 		const pieceAIndex = orderedPartIds.indexOf(pieceA.startPartId)
// 		const pieceBIndex = orderedPartIds.indexOf(pieceB.startPartId)

// 		if (pieceAIndex === -1) {
// 			return false
// 		} else if (pieceBIndex === -1) {
// 			return true
// 		} else if (pieceAIndex < pieceBIndex) {
// 			return true
// 		} else {
// 			return false
// 		}
// 	}

// 	interface InfinitePieceSet {
// 		[PieceLifespan.OutOnShowStyleEnd]?: Piece
// 		[PieceLifespan.OutOnRundownEnd]?: Piece
// 		[PieceLifespan.OutOnSegmentEnd]?: Piece
// 		// onChange?: PieceInstance
// 	}
// 	const piecesOnSourceLayers = new Map<string, InfinitePieceSet>()

// 	// Filter down to the last starting onEnd infinite per layer
// 	for (const candidatePiece of possiblePieces) {
// 		if (
// 			candidatePiece.startPartId !== part._id &&
// 			(candidatePiece.lifespan === PieceLifespan.OutOnShowStyleEnd ||
// 				candidatePiece.lifespan === PieceLifespan.OutOnRundownEnd ||
// 				candidatePiece.lifespan === PieceLifespan.OutOnSegmentEnd)
// 		) {
// 			const useIt = isPiecePotentiallyActiveInPart(
// 				playingPartInstance,
// 				partsBeforeThisInSegmentSet,
// 				segmentsBeforeThisInRundownSet,
// 				rundownsBeforeThisInPlaylist,
// 				rundownsToShowstyles,
// 				rundown,
// 				part,
// 				candidatePiece
// 			)

// 			if (useIt) {
// 				const pieceSet = piecesOnSourceLayers.get(candidatePiece.sourceLayerId) ?? {}
// 				const existingPiece = pieceSet[candidatePiece.lifespan]
// 				if (!existingPiece || doesPieceAStartBeforePieceB(existingPiece, candidatePiece)) {
// 					pieceSet[candidatePiece.lifespan] = candidatePiece
// 					piecesOnSourceLayers.set(candidatePiece.sourceLayerId, pieceSet)
// 				}
// 			}
// 		}
// 	}

// 	// OnChange infinites take priority over onEnd, as they travel with the playhead
// 	const infinitesFromPrevious = playingPartInstance
// 		? getPlayheadTrackingInfinitesForPart(
// 				playlistActivationId,
// 				partsBeforeThisInSegmentSet,
// 				segmentsBeforeThisInRundownSet,
// 				rundownsBeforeThisInPlaylist,
// 				rundownsToShowstyles,
// 				playingPartInstance,
// 				playingPieceInstances || [],
// 				rundown,
// 				part,
// 				newInstanceId,
// 				nextPartIsAfterCurrentPart,
// 				isTemporary
// 		  )
// 		: []

// 	// Compile the resulting list

// 	const playingPieceInstancesMap = normalizeArrayToMapFunc(
// 		playingPieceInstances ?? [],
// 		(p) => p.infinite?.infinitePieceId
// 	)

// 	const wrapPiece = (p: PieceInstancePiece) => {
// 		const instance = rewrapPieceToInstance(p, playlistActivationId, part.rundownId, newInstanceId, isTemporary)

// 		if (instance.piece.lifespan !== PieceLifespan.WithinPart) {
// 			const existingPiece = nextPartIsAfterCurrentPart
// 				? playingPieceInstancesMap.get(instance.piece._id)
// 				: undefined
// 			instance.infinite = {
// 				infiniteInstanceId: existingPiece?.infinite?.infiniteInstanceId ?? getRandomId(),
// 				infiniteInstanceIndex: (existingPiece?.infinite?.infiniteInstanceIndex ?? -1) + 1,
// 				infinitePieceId: instance.piece._id,
// 				fromPreviousPart: false, // Set below
// 			}

// 			instance.infinite.fromPreviousPart = instance.piece.startPartId !== part._id
// 			if (existingPiece && (instance.piece.startPartId !== part._id || instance.dynamicallyInserted)) {
// 				// If it doesnt start in this part, then mark it as a continuation
// 				markPieceInstanceAsContinuation(existingPiece, instance)
// 			}

// 			if (instance.infinite.fromPreviousPart) {
// 				// If this is not the start point, it should start at 0
// 				// Note: this should not be setitng fromPreviousPlayhead, as it is not from the playhead
// 				instance.piece = {
// 					...instance.piece,
// 					enable: {
// 						start: 0,
// 					},
// 				}
// 			}
// 		}

// 		return instance
// 	}

// 	const normalPieces = possiblePieces.filter((p) => p.startPartId === part._id)
// 	const result = normalPieces.map(wrapPiece).concat(infinitesFromPrevious)
// 	for (const pieceSet of Array.from(piecesOnSourceLayers.values())) {
// 		const onEndPieces = _.compact([
// 			pieceSet[PieceLifespan.OutOnShowStyleEnd],
// 			pieceSet[PieceLifespan.OutOnRundownEnd],
// 			pieceSet[PieceLifespan.OutOnSegmentEnd],
// 		])
// 		result.push(...onEndPieces.map(wrapPiece))

// 		// if (pieceSet.onChange) {
// 		// 	result.push(rewrapInstance(pieceSet.onChange))
// 		// }
// 	}

// 	return result
// }

#[derive(Clone, PartialEq)]
pub enum ResolvedEndCap {
    None,
    Absolute(Duration),
    Relative(String),
}

#[derive(Clone)]
pub struct PieceInstanceWithTimings {
    pub piece: PieceInstance,
    /**
     * This is a maximum end point of the pieceInstance.
     * If the pieceInstance also has a enable.duration or userDuration set then the shortest one will need to be used
     * This can be:
     *  - 'now', if it was stopped by something that does not need a preroll (or is virtual)
     *  - '#something.start + 100', if it was stopped by something that needs a preroll
     *  - '100', if not relative to now at all
     */
    resolved_end_cap: ResolvedEndCap,
    priority: i64,
}

/**
 * Get the `enable: { start: ?? }` for the new piece in terms that can be used as an `end` for another object
 */
fn getPieceStartTime(
    new_piece_start: &PieceEnableStart,
    new_piece: &PieceInstance,
) -> ResolvedEndCap {
    match new_piece_start {
        PieceEnableStart::Offset(val) => ResolvedEndCap::Absolute(*val),
        PieceEnableStart::Now => ResolvedEndCap::Relative(format!(
            "#{}.start",
            get_piece_control_object_id(&new_piece.id)
        )),
    }
}

fn is_clear(piece: &PieceInstance) -> bool {
    piece.piece.virtual_
}

fn is_capped_by_avirtual(
    active_pieces: &PieceInstanceOnInfiniteLayers,
    key: &PieceInstanceOnInfiniteLayersKeys,
    new_piece: &PieceInstance,
) -> bool {
    if (key == &PieceInstanceOnInfiniteLayersKeys::onRundownEnd
        || key == &PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd)
        && active_pieces
            .get(&PieceInstanceOnInfiniteLayersKeys::onSegmentEnd)
            .map_or(false, |piece| {
                is_candidate_more_important(new_piece, &piece.piece).unwrap_or(false)
            })
    {
        true
    } else if key == &PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd
        && active_pieces
            .get(&PieceInstanceOnInfiniteLayersKeys::onRundownEnd)
            .map_or(false, |piece| {
                is_candidate_more_important(new_piece, &piece.piece).unwrap_or(false)
            })
    {
        true
    } else {
        false
    }
}

/**
 * Process the infinite pieces to determine the start time and a maximum end time for each.
 * Any pieces which have no chance of being shown (duplicate start times) are pruned
 * The stacking order of infinites is considered, to define the stop times
 */
pub fn processAndPrunePieceInstanceTimings(
    source_layers: &SourceLayers,
    pieces: &[PieceInstance],
    now_in_part: Duration,
    keep_disabled_pieces: bool,
    include_virtual: bool,
) -> Vec<PieceInstanceWithTimings> {
    // We want to group by exclusive groups, to let them be resolved
    let exclusive_group_map: HashMap<String, String> = source_layers
        .iter()
        .filter_map(|(id, sl)| {
            if let Some(exclusive_group) = &sl.exclusive_group {
                Some((id.clone(), exclusive_group.clone()))
            } else {
                None
            }
        })
        .collect();

    let grouped_pieces = {
        let mut grouped: HashMap<String, Vec<&PieceInstance>> = HashMap::new();

        for piece in pieces {
            // At this stage, if a Piece is disabled, the `keepDisabledPieces` must be turned on. If that's the case
            // we split out the disabled Pieces onto the sourceLayerId they actually exist on, instead of putting them
            // onto the shared "exclusivityGroup" layer. This may cause it to not display "exactly" accurately
            // while in the disabled state, but it should keep it from affecting any not-disabled Pieces.
            let group_id = if !piece.disabled {
                exclusive_group_map
                    .get(&piece.piece.source_layer_id)
                    .unwrap_or(&piece.piece.source_layer_id)
            } else if keep_disabled_pieces {
                &piece.piece.source_layer_id
            } else {
                continue;
            };

            if let Some(group_vec) = grouped.get_mut(group_id) {
                group_vec.push(piece);
            } else {
                grouped.insert(group_id.clone(), vec![piece]);
            }
        }

        grouped
    };

    let mut results = Vec::new();

    for pieces in grouped_pieces.into_values() {
        // Group and sort the pieces so that we can step through each point in time
        let pieces_by_start = {
            let mut grouped: HashMap<PieceEnableStart, Vec<&PieceInstance>> = HashMap::new();

            for piece in pieces {
                if let Some(group_vec) = grouped.get_mut(&piece.piece.enable.start) {
                    group_vec.push(piece);
                } else {
                    grouped.insert(piece.piece.enable.start.clone(), vec![piece]);
                }
            }

            grouped
                .into_iter()
                .sorted_by_key(|grp| match grp.0 {
                    PieceEnableStart::Offset(offset) => offset,
                    PieceEnableStart::Now => now_in_part,
                })
                .collect::<Vec<_>>()
        };

        // Step through time
        let mut active_pieces = PieceInstanceOnInfiniteLayers::default();

        for (newPiecesStart, pieces) in pieces_by_start {
            let mut new_pieces = find_piece_instances_on_infinite_layers(&pieces);

            // Apply the updates
            // Note: order is important, the higher layers must be done first
            update_with_new_pieces(
                &mut results,
                &mut active_pieces,
                new_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::other),
                &newPiecesStart,
                include_virtual,
                PieceInstanceOnInfiniteLayersKeys::other,
            );
            update_with_new_pieces(
                &mut results,
                &mut active_pieces,
                new_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::onSegmentEnd),
                &newPiecesStart,
                include_virtual,
                PieceInstanceOnInfiniteLayersKeys::onSegmentEnd,
            );
            update_with_new_pieces(
                &mut results,
                &mut active_pieces,
                new_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::onRundownEnd),
                &newPiecesStart,
                include_virtual,
                PieceInstanceOnInfiniteLayersKeys::onRundownEnd,
            );
            update_with_new_pieces(
                &mut results,
                &mut active_pieces,
                new_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd),
                &newPiecesStart,
                include_virtual,
                PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd,
            );
        }
    }

    // Strip out any pieces that start and end at the same point
    results
        .into_iter()
        .filter(|doc| {
            doc.resolved_end_cap == ResolvedEndCap::None
                || !is_end_cap_equal_to_piece_start(
                    &doc.resolved_end_cap,
                    &doc.piece.piece.enable.start,
                )
        })
        .collect()
}

fn is_end_cap_equal_to_piece_start(cap: &ResolvedEndCap, start: &PieceEnableStart) -> bool {
    match (cap, start) {
        (ResolvedEndCap::Absolute(end), PieceEnableStart::Offset(start)) => end == start,
        _ => false,
    }
}

// #[derive(Clone)]
// pub enum PieceEnableStartExt {
//     Offset(u64),
//     Now(u64),
// }

fn update_with_new_pieces(
    results: &mut Vec<PieceInstanceWithTimings>,
    active_pieces: &mut PieceInstanceOnInfiniteLayers,
    new_piece: Option<PieceInstanceWithTimings>,
    new_pieces_start: &PieceEnableStart,
    include_virtual: bool,
    key: PieceInstanceOnInfiniteLayersKeys,
) {
    if let Some(new_piece) = new_piece {
        if let Some(active_piece) = active_pieces.get_mut(&key) {
            active_piece.resolved_end_cap = getPieceStartTime(new_pieces_start, &new_piece.piece);
        }

        // TODO - we can't clone new_piece. doing that makes this logic super broken

        // track the new piece
        active_pieces.insert(key.clone(), new_piece.clone());

        // We don't want to include virtual pieces in the output (most of the time)
        // TODO - do we want to always output virtual pieces from the 'other' group?
        if include_virtual
            || ((!is_clear(&new_piece.piece) || key == PieceInstanceOnInfiniteLayersKeys::other)
                && !is_capped_by_avirtual(active_pieces, &key, &new_piece.piece))
        {
            results.push(new_piece.clone());

            if key == PieceInstanceOnInfiniteLayersKeys::onSegmentEnd
                || (key == PieceInstanceOnInfiniteLayersKeys::onRundownEnd
                    && !active_pieces
                        .contains_key(&PieceInstanceOnInfiniteLayersKeys::onSegmentEnd))
                || key == PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd
                    && !active_pieces.contains_key(&PieceInstanceOnInfiniteLayersKeys::onSegmentEnd)
                    && !active_pieces.contains_key(&PieceInstanceOnInfiniteLayersKeys::onRundownEnd)
            {
                // when start === 0, we are likely to have multiple infinite continuations. Only stop the 'other' if it should not be considered for being on air
                if let Some(active_other) =
                    active_pieces.get_mut(&PieceInstanceOnInfiniteLayersKeys::other)
                {
                    if new_pieces_start != &PieceEnableStart::Offset(Duration::zero())
                        || isCandidateBetterToBeContinued(&active_other.piece, &new_piece.piece)
                    {
                        // These modes should stop the 'other' when they start if not hidden behind a higher priority onEnd
                        active_other.resolved_end_cap =
                            getPieceStartTime(new_pieces_start, &new_piece.piece);
                        active_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::other);
                    }
                }
            }
        }
    }
}

fn is_candidate_more_important(best: &PieceInstance, candidate: &PieceInstance) -> Option<bool> {
    // Prioritise the one from this part over previous part
    let best_from_previous_part = best
        .infinite
        .as_ref()
        .map_or(false, |inf| inf.from_previous_part);
    let candidate_from_previous_part = candidate
        .infinite
        .as_ref()
        .map_or(false, |inf| inf.from_previous_part);
    if best_from_previous_part && !candidate_from_previous_part {
        // Prefer the candidate as it is not from previous
        return Some(true);
    } else if !best_from_previous_part && candidate_from_previous_part {
        // Prefer the best as it is not from previous
        return Some(false);
    }

    match (best.dynamically_inserted, candidate.dynamically_inserted) {
        (Some(best_inserted), Some(candidate_inserted)) => {
            // prefer the one which starts later
            return Some(best_inserted < candidate_inserted);
        }
        (Some(_), None) => {
            // Prefer the adlib
            return Some(false);
        }
        (None, Some(_)) => {
            // Prefer the adlib
            return Some(true);
        }
        (None, None) => {
            // Neither are adlibs, try other things
        }
    };

    // If one is virtual, prefer that
    if best.piece.virtual_ && !candidate.piece.virtual_ {
        // Prefer the virtual best
        return Some(false);
    } else if !best.piece.virtual_ && candidate.piece.virtual_ {
        // Prefer the virtual candidate
        return Some(true);
    }

    None
}

fn isCandidateBetterToBeContinued(best: &PieceInstance, candidate: &PieceInstance) -> bool {
    // Fallback to id, as we dont have any other criteria and this will be stable.
    // Note: we shouldnt even get here, as it shouldnt be possible for multiple to start at the same time, but it is possible
    is_candidate_more_important(best, candidate).unwrap_or(best.piece.id < candidate.piece.id)
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum PieceInstanceOnInfiniteLayersKeys {
    onShowStyleEnd,
    onRundownEnd,
    onSegmentEnd,
    other,
}
type PieceInstanceOnInfiniteLayers =
    HashMap<PieceInstanceOnInfiniteLayersKeys, PieceInstanceWithTimings>;

fn find_piece_instances_on_infinite_layers(
    pieces: &[&PieceInstance],
) -> PieceInstanceOnInfiniteLayers {
    let mut res = PieceInstanceOnInfiniteLayers::default();

    for piece in pieces {
        match piece.piece.lifespan {
            PieceLifespan::OutOnShowStyleEnd => {
                if res
                    .get(&PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd)
                    .map_or(true, |v| isCandidateBetterToBeContinued(&v.piece, piece))
                {
                    res.insert(
                        PieceInstanceOnInfiniteLayersKeys::onShowStyleEnd,
                        PieceInstanceWithTimings {
                            piece: (*piece).clone(),
                            resolved_end_cap: ResolvedEndCap::None,
                            priority: 0,
                        },
                    );
                }
            }
            PieceLifespan::OutOnRundownEnd => {
                if res
                    .get(&PieceInstanceOnInfiniteLayersKeys::onRundownEnd)
                    .map_or(true, |v| isCandidateBetterToBeContinued(&v.piece, piece))
                {
                    res.insert(
                        PieceInstanceOnInfiniteLayersKeys::onRundownEnd,
                        PieceInstanceWithTimings {
                            piece: (*piece).clone(),
                            resolved_end_cap: ResolvedEndCap::None,
                            priority: 1,
                        },
                    );
                }
            }
            PieceLifespan::OutOnSegmentEnd => {
                if res
                    .get(&PieceInstanceOnInfiniteLayersKeys::onSegmentEnd)
                    .map_or(true, |v| isCandidateBetterToBeContinued(&v.piece, piece))
                {
                    res.insert(
                        PieceInstanceOnInfiniteLayersKeys::onSegmentEnd,
                        PieceInstanceWithTimings {
                            piece: (*piece).clone(),
                            resolved_end_cap: ResolvedEndCap::None,
                            priority: 2,
                        },
                    );
                }
            }
            PieceLifespan::WithinPart
            | PieceLifespan::OutOnSegmentChange
            | PieceLifespan::OutOnRundownChange => {
                if res
                    .get(&PieceInstanceOnInfiniteLayersKeys::other)
                    .map_or(true, |v| isCandidateBetterToBeContinued(&v.piece, piece))
                {
                    res.insert(
                        PieceInstanceOnInfiniteLayersKeys::other,
                        PieceInstanceWithTimings {
                            piece: (*piece).clone(),
                            resolved_end_cap: ResolvedEndCap::None,
                            priority: 5,
                        },
                    );
                }
            }
        };
    }

    res
}

// function continueShowStyleEndInfinites(
// 	rundownsBeforeThisInPlaylist: RundownId[],
// 	rundownsToShowstyles: Map<RundownId, ShowStyleBaseId>,
// 	previousRundownId: RundownId,
// 	rundown: ReadonlyDeep<Pick<DBRundown, '_id' | 'showStyleBaseId'>>
// ): boolean {
// 	let canContinueShowStyleEndInfinites = true
// 	if (rundown.showStyleBaseId !== rundownsToShowstyles.get(previousRundownId)) {
// 		canContinueShowStyleEndInfinites = false
// 	} else {
// 		const targetShowStyle = rundown.showStyleBaseId
// 		canContinueShowStyleEndInfinites = rundownsBeforeThisInPlaylist
// 			.slice(rundownsBeforeThisInPlaylist.indexOf(previousRundownId))
// 			.every((r) => rundownsToShowstyles.get(r) === targetShowStyle)
// 	}

// 	return canContinueShowStyleEndInfinites
// }
