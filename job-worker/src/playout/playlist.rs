use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::data_model::{ids::RundownId, part::Part, segment::Segment};

pub fn sort_segments_in_rundowns(
    mut segments: Vec<Segment>,
    rundown_ids_in_order: &Vec<RundownId>,
) -> Vec<Segment> {
    let rundown_rank_lookup = rundown_ids_in_order
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect::<HashMap<_, _>>();

    segments.sort_by(|a, b| {
        if a.rundown_id == b.rundown_id {
            a.rank.cmp(&b.rank)
        } else {
            let rd_a = rundown_rank_lookup
                .get(&a.rundown_id)
                .unwrap_or(&usize::MAX);
            let rd_b = rundown_rank_lookup
                .get(&b.rundown_id)
                .unwrap_or(&usize::MAX);
            rd_a.cmp(rd_b)
        }
    });

    segments
}

// export function sortPartsInSegments(
// 	parts: DBPart[],
// 	playlist: Pick<DBRundownPlaylist, 'rundownIdsInOrder'>,
// 	segments: Array<Pick<DBSegment, '_id' | 'rundownId' | '_rank'>>
// ): DBPart[] {
// 	return sort_parts_in_sorted_segments(parts, sort_segments_in_rundowns(segments, playlist))
// }
pub fn sort_parts_in_sorted_segments(
    mut parts: Vec<Part>,
    sorted_segments: &Vec<Segment>,
) -> Vec<Part> {
    let segment_rank_lookup = sorted_segments
        .iter()
        .enumerate()
        .map(|(i, seg)| (seg.id.clone(), i))
        .collect::<HashMap<_, _>>();

    parts.sort_by(|a, b| {
        if a.segment_id == b.segment_id {
            a.rank.cmp(&b.rank)
        } else {
            let seg_a = segment_rank_lookup
                .get(&a.segment_id)
                .unwrap_or(&usize::MAX);
            let seg_b = segment_rank_lookup
                .get(&b.segment_id)
                .unwrap_or(&usize::MAX);
            seg_a.cmp(seg_b)
        }
    });

    parts
}

/**
 * Sort an array of RundownIds based on a reference list
 * @param sortedPossibleIds The already sorted ids. This may be missing some of the unsorted ones
 * @param unsortedRundownIds The ids to sort
 */
pub fn sortRundownIDsInPlaylist(
    sorted_possible_ids: &[RundownId],
    unsorted_rundown_ids: Vec<RundownId>,
) -> Vec<RundownId> {
    let unsorted_rundown_ids_set = unsorted_rundown_ids.into_iter().collect::<HashSet<_>>();

    let mut sorted_verified_existing = sorted_possible_ids
        .iter()
        .filter(|id| unsorted_rundown_ids_set.contains(*id))
        .cloned()
        .collect::<Vec<_>>();

    let sorted_verified_existing_set = sorted_verified_existing.iter().collect::<HashSet<_>>();

    // Find the ids which are missing from the playlist (just in case)
    sorted_verified_existing.extend(
        unsorted_rundown_ids_set
            .into_iter()
            .filter(|id| !sorted_verified_existing_set.contains(id))
            .sorted(),
    );

    sorted_verified_existing
}
