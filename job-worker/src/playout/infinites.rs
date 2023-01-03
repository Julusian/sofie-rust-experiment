use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use chrono::Duration;
use itertools::Itertools;
use mongodb::bson::{self, doc};
use sofie_rust_experiment::{get_random_id, normalizeArrayToMapOfRefs};

use crate::data_model::{
    extra::get_piece_control_object_id,
    ids::{
        unprotect_array, PartId, PartInstanceId, PieceInstanceId, PieceInstanceInfiniteId,
        ProtectedId, RundownId, RundownPlaylistActivationId, SegmentId, ShowStyleBaseId,
    },
    part::Part,
    part_instance::PartInstance,
    piece::{Piece, PieceEnable, PieceEnableStart, PieceLifespan},
    piece_instance::{rewrapPieceToInstance, PieceInstance, PieceInstanceInfinite},
    rundown::Rundown,
    show_style_base::SourceLayers,
};

pub fn buildPastInfinitePiecesForThisPartQuery(
    part: &Part,
    partsIdsBeforeThisInSegment: &[PartId],
    segmentsIdsBeforeThisInRundown: &[SegmentId],
    rundownIdsBeforeThisInPlaylist: &[RundownId],
) -> Option<bson::Document> {
    let mut fragments = Vec::with_capacity(3);

    if partsIdsBeforeThisInSegment.len() > 0 {
        fragments.push(doc! {
            // same segment, and previous part
            "lifespan": {
                "$in": vec![
                    PieceLifespan::OutOnSegmentEnd,
                    PieceLifespan::OutOnSegmentChange,
                    PieceLifespan::OutOnRundownEnd,
                    PieceLifespan::OutOnRundownChange,
                    PieceLifespan::OutOnShowStyleEnd,
                ],
            },
            "startRundownId": part.rundown_id.unprotect(),
            "startSegmentId": part.segment_id.unprotect(),
            "startPartId": { "$in": unprotect_array(partsIdsBeforeThisInSegment) },
        });
    }

    if segmentsIdsBeforeThisInRundown.len() > 0 {
        fragments.push(doc! {
            // same rundown, and previous segment
            "lifespan": {
                "$in": vec![
                    PieceLifespan::OutOnRundownEnd,
                    PieceLifespan::OutOnRundownChange,
                    PieceLifespan::OutOnShowStyleEnd,
                ],
            },
            "startRundownId": part.rundown_id.unprotect(),
            "startSegmentId": { "$in": unprotect_array(segmentsIdsBeforeThisInRundown) },
        });
    }

    if rundownIdsBeforeThisInPlaylist.len() > 0 {
        fragments.push(doc! {
            // previous rundown
            "lifespan": {
                "$in": vec![
                    PieceLifespan::OutOnShowStyleEnd,
                ],
            },
            "startRundownId": { "$in":unprotect_array( rundownIdsBeforeThisInPlaylist) },
        })
    }

    if fragments.len() == 0 {
        None
    } else if fragments.len() == 1 {
        let mut res = doc! {
            "invalid": { "$ne": true },
            "startPartId": { "$ne": part.id.unprotect() },
        };

        res.extend(fragments.pop().unwrap());
        Some(res)
    } else {
        Some(doc! {
            "invalid": { "$ne": true },
            "startPartId": { "$ne": part.id.unprotect() },
            "$or": fragments,
        })
    }
}

pub fn getPlayheadTrackingInfinitesForPart(
    playlistActivationId: &RundownPlaylistActivationId,
    partsBeforeThisInSegmentSet: &HashSet<PartId>,
    segmentsBeforeThisInRundownSet: &HashSet<SegmentId>,
    rundownsBeforeThisInPlaylist: &[RundownId],
    rundownsToShowstyles: &HashMap<RundownId, ShowStyleBaseId>,
    currentPartInstance: &PartInstance,
    currentPartPieceInstances: &[PieceInstance],
    rundown: &Rundown,
    part: &Part,
    newInstanceId: &PartInstanceId,
    nextPartIsAfterCurrentPart: bool,
    isTemporary: bool,
) -> Vec<PieceInstance> {
    let canContinueAdlibOnEnds = nextPartIsAfterCurrentPart;

    let mut result = Vec::new();

    let canContinueShowStyleEndInfinites = continueShowStyleEndInfinites(
        rundownsBeforeThisInPlaylist,
        rundownsToShowstyles,
        &currentPartInstance.rundown_id,
        rundown,
    );

    let groupedPlayingPieceInstances = {
        let mut grouped: HashMap<String, Vec<&PieceInstance>> = HashMap::new();

        for piece in currentPartPieceInstances {
            if let Some(group_vec) = grouped.get_mut(&piece.piece.source_layer_id) {
                group_vec.push(piece);
            } else {
                grouped.insert(piece.piece.source_layer_id.clone(), vec![piece]);
            }
        }

        grouped
    };

    for (source_layer_id, pieceInstances) in groupedPlayingPieceInstances {
        // Find the ones that starts last. Note: any piece will stop an onChange
        let lastPiecesByStart = {
            let mut grouped: HashMap<PieceEnableStart, Vec<&PieceInstance>> = HashMap::new();

            for piece in &pieceInstances {
                if let Some(group_vec) = grouped.get_mut(&piece.piece.enable.start) {
                    group_vec.push(piece);
                } else {
                    grouped.insert(piece.piece.enable.start.clone(), vec![piece]);
                }
            }

            grouped
        };
        let lastPieceInstances = {
            let now_entry = lastPiecesByStart.get(&PieceEnableStart::Now);

            // TODO - this is pretty horrible

            if let Some(now_entry) = now_entry {
                now_entry.clone()
            } else {
                let mut largest = None;
                for key in lastPiecesByStart.keys() {
                    match (largest, key) {
                        (None, PieceEnableStart::Offset(o)) => largest = Some(o),
                        (Some(l), PieceEnableStart::Offset(o)) => {
                            if o > l {
                                largest = Some(o)
                            }
                        }
                        (Some(_), PieceEnableStart::Now) | (None, PieceEnableStart::Now) => {
                            // Not useful
                        }
                    }
                }

                if let Some(largest) = largest {
                    if let Some(pieces) =
                        lastPiecesByStart.get(&PieceEnableStart::Offset(largest.clone()))
                    {
                        pieces.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
        };

        // Some basic resolving, to figure out which is our candidate
        let lastPieceInstance = lastPieceInstances.into_iter().reduce(|best, candidate| {
            if isCandidateBetterToBeContinued(best, candidate) {
                candidate
            } else {
                best
            }
        });

        if let Some(lastPieceInstance) = lastPieceInstance {
            if lastPieceInstance.planned_stopped_playback.is_none()
                && lastPieceInstance.user_duration.is_none()
            {
                // If it is an onChange, then it may want to continue
                let is_used = match lastPieceInstance.piece.lifespan {
                    PieceLifespan::OutOnSegmentChange => {
                        currentPartInstance.segment_id == part.segment_id
                    }
                    PieceLifespan::OutOnRundownChange => {
                        currentPartInstance.rundown_id == part.rundown_id
                    }
                    PieceLifespan::WithinPart
                    | PieceLifespan::OutOnSegmentEnd
                    | PieceLifespan::OutOnRundownEnd
                    | PieceLifespan::OutOnShowStyleEnd => false,
                };

                if is_used {
                    if let Some(piece) = rewrapPlayheadTrackingPiece(
                        playlistActivationId,
                        part,
                        newInstanceId,
                        lastPieceInstance,
                        isTemporary,
                    ) {
                        result.push(piece);
                    }
                    // This may get pruned later, if somethng else has a start of 0
                }
            }
        }

        // Check if we should persist any adlib onEnd infinites
        if canContinueAdlibOnEnds {
            let piecesByInfiniteMode = {
                let mut grouped: HashMap<PieceLifespan, Vec<&PieceInstance>> = HashMap::new();

                for piece in &pieceInstances {
                    if piece.dynamically_inserted.is_some() {
                        if let Some(group_vec) = grouped.get_mut(&piece.piece.lifespan) {
                            group_vec.push(piece);
                        } else {
                            grouped.insert(piece.piece.lifespan.clone(), vec![piece]);
                        }
                    }
                }

                grouped
            };

            if let Some(candidatePiece) =
                getCandidateForMode(&piecesByInfiniteMode, &PieceLifespan::OutOnRundownEnd)
            {
                if candidatePiece.rundown_id == part.rundown_id
                    && (segmentsBeforeThisInRundownSet.contains(&currentPartInstance.segment_id)
                        || currentPartInstance.segment_id == part.segment_id)
                {
                    if let Some(piece) = rewrapPlayheadTrackingPiece(
                        playlistActivationId,
                        part,
                        newInstanceId,
                        candidatePiece,
                        isTemporary,
                    ) {
                        result.push(piece);
                    }
                }
            }

            if let Some(candidatePiece) =
                getCandidateForMode(&piecesByInfiniteMode, &PieceLifespan::OutOnSegmentEnd)
            {
                if currentPartInstance.segment_id == part.segment_id
                    && partsBeforeThisInSegmentSet.contains(&candidatePiece.piece.start_part_id)
                {
                    if let Some(piece) = rewrapPlayheadTrackingPiece(
                        playlistActivationId,
                        part,
                        newInstanceId,
                        candidatePiece,
                        isTemporary,
                    ) {
                        result.push(piece);
                    }
                }
            }

            if let Some(candidatePiece) =
                getCandidateForMode(&piecesByInfiniteMode, &PieceLifespan::OutOnShowStyleEnd)
            {
                if canContinueShowStyleEndInfinites {
                    if let Some(piece) = rewrapPlayheadTrackingPiece(
                        playlistActivationId,
                        part,
                        newInstanceId,
                        candidatePiece,
                        isTemporary,
                    ) {
                        result.push(piece);
                    }
                }
            }
        }
    }

    result
}

fn rewrapPlayheadTrackingPiece(
    playlist_activation_id: &RundownPlaylistActivationId,
    part: &Part,
    newInstanceId: &PartInstanceId,
    input: &PieceInstance,
    is_temporary: bool,
) -> Option<PieceInstance> {
    if let Some(input_infinite) = &input.infinite {
        let mut instance = rewrapPieceToInstance(
            input.piece.clone(),
            playlist_activation_id.clone(),
            part.rundown_id.clone(),
            newInstanceId.clone(),
            is_temporary,
        );

        markPieceInstanceAsContinuation(input, &mut instance);

        // This was copied from before, so we know we can force the time to 0
        instance.piece.enable.start = PieceEnableStart::Offset(Duration::zero());
        instance.infinite = {
            let mut inf = input_infinite.clone();

            inf.infinite_instance_index = input_infinite.infinite_instance_index + 1;
            inf.from_previous_part = true;
            inf.from_previous_playhead = true;

            Some(inf)
        };

        Some(instance)
    } else {
        None
    }
}

fn getCandidateForMode<'a>(
    piecesByInfiniteMode: &'a HashMap<PieceLifespan, Vec<&PieceInstance>>,
    mode: &PieceLifespan,
) -> Option<&'a PieceInstance> {
    let pieces = piecesByInfiniteMode.get(mode).map_or(Vec::new(), |pieces| {
        pieces
            .iter()
            .filter(|piece| {
                if let Some(inf) = &piece.infinite {
                    inf.from_previous_playhead || piece.dynamically_inserted.is_some()
                } else {
                    false
                }
            })
            .collect_vec()
    });
    // This is the piece we may copy across
    let candidatePiece = pieces.iter().reduce(|best, new| {
        match (&best.piece.enable.start, &new.piece.enable.start) {
            (PieceEnableStart::Now, PieceEnableStart::Now) => best,
            (PieceEnableStart::Offset(_), PieceEnableStart::Now) => new,
            (PieceEnableStart::Now, PieceEnableStart::Offset(_)) => best,
            (PieceEnableStart::Offset(best_offset), PieceEnableStart::Offset(new_offset)) => {
                // TODO - by id?
                if best_offset >= new_offset {
                    best
                } else {
                    new
                }
            }
        }
    });

    if let Some(candidatePiece) = candidatePiece {
        if candidatePiece.planned_stopped_playback.is_none()
            && candidatePiece.user_duration.is_none()
        {
            Some(candidatePiece)
        } else {
            None
        }
    } else {
        None
    }
}

fn markPieceInstanceAsContinuation(previousInstance: &PieceInstance, instance: &mut PieceInstance) {
    instance.id = PieceInstanceId::new_from(format!("{}_continue", instance.id.unprotect()));
    instance.dynamically_inserted = previousInstance.dynamically_inserted;
    instance.adlib_source_id = previousInstance.adlib_source_id.clone();
    instance.reported_started_playback = previousInstance.reported_started_playback;
}

pub fn isPiecePotentiallyActiveInPart(
    previousPartInstance: Option<&PartInstance>,
    partsBeforeThisInSegment: &HashSet<PartId>,
    segmentsBeforeThisInRundown: &HashSet<SegmentId>,
    rundownsBeforeThisInPlaylist: &[RundownId],
    rundownsToShowstyles: &HashMap<RundownId, ShowStyleBaseId>,
    rundown: &Rundown,
    part: &Part,
    pieceToCheck: &Piece,
) -> bool {
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
    todo!();
}

fn doesPieceAStartBeforePieceB(orderedPartIds: &[PartId], pieceA: &Piece, pieceB: &Piece) -> bool {
    if pieceA.start_part_id == pieceB.start_part_id {
        // TODO - is this equal to before?
        match (&pieceA.enable.start, &pieceB.enable.start) {
            (PieceEnableStart::Offset(a), PieceEnableStart::Offset(b)) => a < b,
            (PieceEnableStart::Offset(_), PieceEnableStart::Now) => false,
            (PieceEnableStart::Now, PieceEnableStart::Offset(_)) => false,
            (PieceEnableStart::Now, PieceEnableStart::Now) => false,
        }
    } else {
        let a_index = orderedPartIds
            .iter()
            .position(|id| id == &pieceA.start_part_id);
        let b_index = orderedPartIds
            .iter()
            .position(|id| id == &pieceB.start_part_id);

        match (a_index, b_index) {
            (None, _) => false,
            (_, None) => true,
            (Some(a), Some(b)) => a < b,
        }
    }
}

#[derive(Default)]
struct InfinitePieceSet {
    // TODO - could this be refs?
    on_show_style_end: Option<Piece>,
    on_rundown_end: Option<Piece>,
    on_segment_end: Option<Piece>,
}
impl InfinitePieceSet {
    fn set_piece(&mut self, lifespan: PieceLifespan, piece: Piece) {
        match lifespan {
            PieceLifespan::WithinPart => todo!(),
            PieceLifespan::OutOnSegmentChange => todo!(),
            PieceLifespan::OutOnSegmentEnd => {
                self.on_segment_end = Some(piece);
            }
            PieceLifespan::OutOnRundownChange => todo!(),
            PieceLifespan::OutOnRundownEnd => {
                self.on_rundown_end = Some(piece);
            }
            PieceLifespan::OutOnShowStyleEnd => {
                self.on_show_style_end = Some(piece);
            }
        }
    }

    fn get_piece(&self, lifespan: PieceLifespan) -> Option<&Piece> {
        match lifespan {
            PieceLifespan::WithinPart => None,
            PieceLifespan::OutOnSegmentChange => None,
            PieceLifespan::OutOnSegmentEnd => self.on_segment_end.as_ref(),
            PieceLifespan::OutOnRundownChange => None,
            PieceLifespan::OutOnRundownEnd => self.on_rundown_end.as_ref(),
            PieceLifespan::OutOnShowStyleEnd => self.on_show_style_end.as_ref(),
        }
    }
}

pub fn getPieceInstancesForPart2(
    playlistActivationId: RundownPlaylistActivationId,
    playingPartInstance: Option<&PartInstance>,
    playingPieceInstances: &[PieceInstance], //| undefined,
    rundown: &Rundown,
    part: &Part,
    partsBeforeThisInSegmentSet: &HashSet<PartId>,
    segmentsBeforeThisInRundownSet: &HashSet<SegmentId>,
    rundownsBeforeThisInPlaylist: &[RundownId],
    rundownsToShowstyles: &HashMap<RundownId, ShowStyleBaseId>,
    possiblePieces: &[Piece],
    orderedPartIds: &[PartId],
    newInstanceId: PartInstanceId,
    nextPartIsAfterCurrentPart: bool,
    isTemporary: bool,
) -> Vec<PieceInstance> {
    let mut piecesOnSourceLayers = HashMap::new(); //Map<string, InfinitePieceSet>()

    // Filter down to the last starting onEnd infinite per layer
    for candidatePiece in possiblePieces {
        if candidatePiece.start_part_id != part.id
            && (candidatePiece.lifespan == PieceLifespan::OutOnShowStyleEnd
                || candidatePiece.lifespan == PieceLifespan::OutOnRundownEnd
                || candidatePiece.lifespan == PieceLifespan::OutOnSegmentEnd)
        {
            let useIt = isPiecePotentiallyActiveInPart(
                playingPartInstance,
                partsBeforeThisInSegmentSet,
                segmentsBeforeThisInRundownSet,
                rundownsBeforeThisInPlaylist,
                rundownsToShowstyles,
                rundown,
                part,
                candidatePiece,
            );

            if useIt {
                let pieceSetEntry =
                    piecesOnSourceLayers.entry(candidatePiece.source_layer_id.clone());

                let pieceSet = match pieceSetEntry {
                    std::collections::hash_map::Entry::Occupied(mut entry) => entry.into_mut(),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(InfinitePieceSet::default())
                    }
                };

                if pieceSet
                    .get_piece(candidatePiece.lifespan)
                    .and_then(|existingPiece| {
                        Some(doesPieceAStartBeforePieceB(
                            orderedPartIds,
                            existingPiece,
                            candidatePiece,
                        ))
                    })
                    .unwrap_or(true)
                {
                    pieceSet.set_piece(candidatePiece.lifespan, candidatePiece.clone());
                }
            }
        }
    }

    // OnChange infinites take priority over onEnd, as they travel with the playhead
    let infinitesFromPrevious = if let Some(playingPartInstance) = playingPartInstance {
        getPlayheadTrackingInfinitesForPart(
            &playlistActivationId,
            partsBeforeThisInSegmentSet,
            segmentsBeforeThisInRundownSet,
            rundownsBeforeThisInPlaylist,
            rundownsToShowstyles,
            playingPartInstance,
            playingPieceInstances,
            rundown,
            part,
            &newInstanceId,
            nextPartIsAfterCurrentPart,
            isTemporary,
        )
    } else {
        vec![]
    };

    // Compile the resulting list

    let playingPieceInstancesMap = normalizeArrayToMapOfRefs(playingPieceInstances, |p| {
        p.infinite
            .as_ref()
            .and_then(|inf| Some(inf.infinite_piece_id.clone()))
    });

    let wrap_piece = |p: Piece| {
        let mut instance = rewrapPieceToInstance(
            p,
            playlistActivationId.clone(),
            part.rundown_id.clone(),
            newInstanceId.clone(),
            isTemporary,
        );

        if instance.piece.lifespan != PieceLifespan::WithinPart {
            let existingPiece = if nextPartIsAfterCurrentPart {
                playingPieceInstancesMap.get(&instance.piece.id)
            } else {
                None
            };

            let mut instance_infinite = PieceInstanceInfinite {
                infinite_instance_id: existingPiece
                    .and_then(|p| p.infinite.as_ref())
                    .and_then(|inf| Some(inf.infinite_instance_id.clone()))
                    .unwrap_or_else(|| PieceInstanceInfiniteId::new_from(get_random_id())),
                infinite_instance_index: existingPiece
                    .and_then(|p| p.infinite.as_ref())
                    .and_then(|inf| Some(inf.infinite_instance_index + 1))
                    .unwrap_or(0),
                infinite_piece_id: instance.piece.id.clone(),
                from_previous_part: false, // Set below

                // Not relevant?
                from_hold: false,
                from_previous_playhead: false,
            };

            instance_infinite.from_previous_part = instance.piece.start_part_id != part.id;

            if let Some(existingPiece) = existingPiece {
                if instance.piece.start_part_id != part.id
                    || instance.dynamically_inserted.is_some()
                {
                    // If it doesnt start in this part, then mark it as a continuation
                    markPieceInstanceAsContinuation(existingPiece, &mut instance);
                }
            }

            if instance_infinite.from_previous_part {
                // If this is not the start point, it should start at 0
                // Note: this should not be setitng fromPreviousPlayhead, as it is not from the playhead
                instance.piece.enable = PieceEnable {
                    start: PieceEnableStart::Offset(Duration::zero()),
                    duration: None,
                };
            }

            instance.infinite = Some(instance_infinite);
        }

        instance
    };

    let normalPieces = possiblePieces.iter().filter(|p| p.start_part_id == part.id);

    let mut result = normalPieces
        .map(|piece| wrap_piece(piece.clone()))
        .chain(infinitesFromPrevious)
        .collect::<Vec<_>>();
    for pieceSet in piecesOnSourceLayers.into_values() {
        if let Some(piece) = pieceSet.on_show_style_end {
            result.push(wrap_piece(piece));
        }
        if let Some(piece) = pieceSet.on_rundown_end {
            result.push(wrap_piece(piece));
        }
        if let Some(piece) = pieceSet.on_segment_end {
            result.push(wrap_piece(piece));
        }

        // if (pieceSet.onChange) {
        // 	result.push(rewrapInstance(pieceSet.onChange))
        // }
    }

    result
}

#[derive(Clone, PartialEq)]
pub enum ResolvedEndCap {
    None,
    Absolute(Duration),
    Relative(String),
}

#[derive(Clone)]
pub struct PieceInstanceWithTimings {
    pub piece: Rc<PieceInstance>,
    /**
     * This is a maximum end point of the pieceInstance.
     * If the pieceInstance also has a enable.duration or userDuration set then the shortest one will need to be used
     * This can be:
     *  - 'now', if it was stopped by something that does not need a preroll (or is virtual)
     *  - '#something.start + 100', if it was stopped by something that needs a preroll
     *  - '100', if not relative to now at all
     */
    pub resolved_end_cap: ResolvedEndCap,
    pub priority: i64,
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

        // track the new piece
        let new_piece_tmp = new_piece.clone();
        active_pieces.insert(key.clone(), new_piece);

        // We don't want to include virtual pieces in the output (most of the time)
        // TODO - do we want to always output virtual pieces from the 'other' group?
        if include_virtual
            || ((!is_clear(&new_piece_tmp.piece)
                || key == PieceInstanceOnInfiniteLayersKeys::other)
                && !is_capped_by_avirtual(active_pieces, &key, &new_piece_tmp.piece))
        {
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
                        || isCandidateBetterToBeContinued(&active_other.piece, &new_piece_tmp.piece)
                    {
                        // These modes should stop the 'other' when they start if not hidden behind a higher priority onEnd
                        active_other.resolved_end_cap =
                            getPieceStartTime(new_pieces_start, &new_piece_tmp.piece);
                        active_pieces.remove(&PieceInstanceOnInfiniteLayersKeys::other);
                    }
                }
            }

            results.push(new_piece_tmp);
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
                            piece: Rc::new((*piece).clone()),
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
                            piece: Rc::new((*piece).clone()),
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
                            piece: Rc::new((*piece).clone()),
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
                            piece: Rc::new((*piece).clone()),
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

fn continueShowStyleEndInfinites(
    rundowns_before_this_in_playlist: &[RundownId],
    rundowns_to_showstyles: &HashMap<RundownId, ShowStyleBaseId>,
    previous_rundown_id: &RundownId,
    rundown: &Rundown,
) -> bool {
    if Some(&rundown.show_style_base_id) != rundowns_to_showstyles.get(previous_rundown_id) {
        false
    } else {
        let target_show_style = &rundown.show_style_base_id;
        let from_index = rundowns_before_this_in_playlist
            .iter()
            .position(|rd| rd == previous_rundown_id)
            .unwrap_or(0);

        rundowns_before_this_in_playlist
            .iter()
            .skip(from_index)
            .all(|rd| rundowns_to_showstyles.get(rd) == Some(target_show_style))
    }
}
