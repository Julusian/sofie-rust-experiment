use chrono::Duration;

use crate::data_model::{
    part::{Part, PartInTransition},
    part_instance::PartCalculatedTimings,
    piece::{IBlueprintPieceType, Piece, PieceEnableStart},
    rundown_playlist::RundownHoldState,
};

/**
 * Calculate the total pre-roll duration of a PartInstance
 * Note: once the part has been taken this should not be recalculated. Doing so may result in the timings shifting
 */
fn calculate_part_preroll(pieces: &[Piece]) -> Duration {
    let mut longest = Duration::zero();

    for piece in pieces {
        if piece.piece_type != IBlueprintPieceType::Normal {
            // Ignore preroll for transition pieces
            continue;
        }

        match piece.enable.start {
            PieceEnableStart::Now => {
                // A piece starting at now was adlibbed, so does not affect the starting preroll
            }
            PieceEnableStart::Offset(start) => {
                if piece.preroll_duration > Duration::zero() {
                    // How far before the part does the piece protrude
                    let offset = piece.preroll_duration - start;
                    if offset > Duration::zero() && offset > longest {
                        longest = offset
                    }
                }
            }
        };
    }

    longest
}

/**
 * Calculate the total post-roll duration of a PartInstance
 */
fn calculate_part_postroll(pieces: &[Piece]) -> Duration {
    let mut longest = Duration::zero();

    for piece in pieces {
        if piece.enable.duration.is_some() {
            // presume it ends before we do a take
            continue;
        }

        if piece.postroll_duration > longest {
            longest = piece.postroll_duration
        }
    }

    longest
}

/**
 * Calculate the timings of the period where the parts can overlap.
 */
pub fn calculatePartTimings(
    hold_state: RundownHoldState,
    from_part: Option<&Part>,
    from_pieces: Option<&[Piece]>,
    to_part: &Part,
    to_pieces: &[Piece],
) -> PartCalculatedTimings {
    // If in a hold, we cant do the transition
    let is_in_hold =
        hold_state != RundownHoldState::NONE && hold_state != RundownHoldState::COMPLETE;

    let to_part_preroll = calculate_part_preroll(to_pieces);
    let from_part_postroll = match (from_part, from_pieces) {
        (Some(_), Some(from_pieces)) => calculate_part_postroll(from_pieces),
        _ => Duration::zero(),
    };
    let to_part_postroll = calculate_part_postroll(to_pieces);

    let mut in_transition = None;
    let mut allow_transition_piece = false;
    if !is_in_hold {
        if let Some(from_part) = from_part {
            if from_part.autonext {
                // An auto-next with overlap is essentially a simple transition, so we treat it as one
                allow_transition_piece = false;
                in_transition = Some(PartInTransition {
                    block_take_duration: Duration::zero(),
                    part_content_delay_duration: Duration::zero(),
                    previous_part_keepalive_duration: from_part
                        .autonext_overlap
                        .unwrap_or(Duration::zero()),
                });
            } else if !from_part.disable_next_in_transition {
                allow_transition_piece = true;
                in_transition = to_part.in_transition.clone();
            }
        }
    }

    // Try and convert the transition
    match (in_transition, from_part) {
        (None, None) | (None, Some(_)) | (Some(_), None) => {
            // The amount to delay the part 'switch' to, to ensure the outTransition has time to complete as well as any prerolls for part B
            // Duration::max(toPartPreroll, other)
            let out_transition_duration = from_part.map_or(Duration::zero(), |p| {
                p.out_transition
                    .as_ref()
                    .map_or(Duration::zero(), |t| t.duration)
            });
            let take_offset = Duration::max(
                Duration::zero(),
                Duration::max(out_transition_duration, to_part_preroll),
            );

            PartCalculatedTimings {
                in_transition_start: None, // No transition to use
                // delay the new part for a bit
                to_part_delay: take_offset,
                to_part_postroll: to_part_postroll,
                // The old part needs to continue for a while
                from_part_remaining: take_offset + from_part_postroll,
                from_part_postroll: from_part_postroll,
            }
        }
        (Some(in_transition), Some(from_part)) => {
            // The amount of time needed to complete the outTransition before the 'take' point
            let out_transition_time = if let Some(out_transition) = &from_part.out_transition {
                out_transition.duration - in_transition.previous_part_keepalive_duration
            } else {
                Duration::zero()
            };

            // The amount of time needed to preroll Part B before the 'take' point
            let preroll_time = to_part_preroll - in_transition.part_content_delay_duration;

            // The amount to delay the part 'switch' to, to ensure the outTransition has time to complete as well as any prerolls for part B
            let take_offset = Duration::max(
                Duration::zero(),
                Duration::max(out_transition_time, preroll_time),
            );

            PartCalculatedTimings {
                in_transition_start: if allow_transition_piece {
                    Some(take_offset)
                } else {
                    None
                },
                to_part_delay: take_offset + in_transition.part_content_delay_duration,
                to_part_postroll: to_part_postroll,
                from_part_remaining: take_offset
                    + in_transition.previous_part_keepalive_duration
                    + from_part_postroll,
                from_part_postroll: from_part_postroll,
            }
        }
    }
}

// export function getPartTimingsOrDefaults(
// 	partInstance: DBPartInstance,
// 	pieceInstances: PieceInstance[]
// ): PartCalculatedTimings {
// 	if (partInstance.partPlayoutTimings) {
// 		return partInstance.partPlayoutTimings
// 	} else {
// 		return calculatePartTimings(
// 			RundownHoldState.NONE,
// 			undefined,
// 			undefined,
// 			partInstance.part,
// 			pieceInstances.map((p) => p.piece)
// 		)
// 	}
// }

// function calculatePartExpectedDurationWithPrerollInner(rawDuration: number, timings: PartCalculatedTimings): number {
// 	return Math.max(0, rawDuration + timings.toPartDelay - timings.fromPartRemaining)
// }

// export function calculatePartExpectedDurationWithPreroll(
// 	part: DBPart,
// 	pieces: PieceInstancePiece[]
// ): number | undefined {
// 	if (part.expectedDuration === undefined) return undefined

// 	const timings = calculatePartTimings(undefined, {}, [], part, pieces)

// 	return calculatePartExpectedDurationWithPrerollInner(part.expectedDuration, timings)
// }

// export function calculatePartInstanceExpectedDurationWithPreroll(
// 	partInstance: Pick<DBPartInstance, 'part' | 'partPlayoutTimings'>
// ): number | undefined {
// 	if (partInstance.part.expectedDuration === undefined) return undefined

// 	if (partInstance.partPlayoutTimings) {
// 		return calculatePartExpectedDurationWithPrerollInner(
// 			partInstance.part.expectedDuration,
// 			partInstance.partPlayoutTimings
// 		)
// 	} else {
// 		return partInstance.part.expectedDurationWithPreroll ?? partInstance.part.expectedDuration
// 	}
// }
