use std::collections::HashMap;

use crate::data_model::{
    ids::{PartId, SegmentId},
    part_instance::PartInstance,
    rundown_playlist::RundownPlaylist,
};

use super::cache::SegmentsAndParts;

pub struct SelectNextPartResult {
    pub part_id: PartId,
    pub segment_id: SegmentId,
    pub index: usize,
    pub consumes_next_segment_id: bool,
}

pub fn select_next_part(
    playlist: &RundownPlaylist,
    previous_part_instance: Option<&PartInstance>,
    currently_selected_part_instance: Option<&PartInstance>,
    parts_and_segments: SegmentsAndParts,
    ignore_unplayable: bool,
) -> Option<SelectNextPartResult> {
    let mut parts = parts_and_segments.parts;
    let segments = parts_and_segments.segments;

    // In the parts array, insert currentlySelectedPartInstance over its part, as it is already nexted, so wont change unless necessary
    if let Some(currently_selected_part_instance) = currently_selected_part_instance {
        let current_id = &currently_selected_part_instance.part.id;
        let index = parts.iter().position(|doc| &doc.id == current_id);

        if let Some(index) = index {
            parts[index] = currently_selected_part_instance.part.clone();
        }
    }

    /*
     * Iterates over all the parts and searches for the first one to be playable
     * @param offset the index from where to start the search
     * @param condition whether the part will be returned
     * @param length the maximum index or where to stop the search
     */
    let find_first_playable_part = |offset: usize,
                                    only_segment_id: Option<&SegmentId>,
                                    length: Option<usize>|
     -> Option<SelectNextPartResult> {
        // Filter to after and find the first playabale
        let end = length.unwrap_or(parts.len());
        for index in offset..end {
            let part = &parts[index];

            if (!ignore_unplayable || part.is_playable())
                && (only_segment_id.map_or(true, |segment_id| &part.segment_id == segment_id))
            {
                return Some(SelectNextPartResult {
                    part_id: part.id.clone(),
                    segment_id: part.segment_id.clone(),
                    index,
                    consumes_next_segment_id: false,
                });
            }
        }

        None
    };

    let mut search_from_index = 0;
    if let Some(previous_part_instance) = previous_part_instance {
        let current_index = parts
            .iter()
            .position(|doc| &doc.id == &previous_part_instance.part.id);

        if let Some(current_index) = current_index {
            search_from_index = current_index + 1;
        } else {
            let mut segment_starts = HashMap::new();
            for (i, part) in parts.iter().enumerate() {
                if !segment_starts.contains_key(&part.segment_id) {
                    segment_starts.insert(part.segment_id.clone(), i);
                }
            }

            // Look for other parts in the segment to reference
            let segment_start_index = segment_starts.get(&previous_part_instance.segment_id);
            if let Some(segment_start_index) = segment_start_index {
                let mut next_in_segment_index = None;

                for i in *segment_start_index..parts.len() {
                    let part = &parts[i];
                    if part.segment_id != previous_part_instance.segment_id {
                        break;
                    }

                    if part.rank < previous_part_instance.part.rank {
                        next_in_segment_index = Some(i + 1);
                    }
                }

                search_from_index = next_in_segment_index.unwrap_or(*segment_start_index);
            } else {
                // If we didn't find the segment in the list of parts, then look for segments after this one.
                let segment_index = segments
                    .iter()
                    .position(|doc| doc.id == previous_part_instance.segment_id);

                if let Some(segment_index) = segment_index {
                    let mut following_segment_start = None;

                    // Find the first segment with parts that lies after this
                    for i in (segment_index + 1)..segments.len() {
                        let segment_start = segment_starts.get(&segments[i].id);
                        if segment_start.is_some() {
                            following_segment_start = segment_start.cloned();
                            break;
                        }
                    }

                    // Either there is a segment after, or we are at the end of the rundown
                    search_from_index = following_segment_start.unwrap_or(parts.len() + 1);
                } else {
                    // Somehow we cannot place the segment, so the start of the playlist is better than nothing
                }
            }
        }
    }

    // Filter to after and find the first playabale
    let mut next_part = find_first_playable_part(search_from_index, None, None);

    if playlist.next_segment_id.is_some() {
        // No previous part, or segment has changed
        let do_it = if let Some(previous_part_instance) = previous_part_instance {
            next_part.as_ref().map(|p| &p.segment_id) != Some(&previous_part_instance.segment_id)
        } else {
            true
        };

        if do_it {
            // Find first in segment
            let new_segment_part =
                find_first_playable_part(0, playlist.next_segment_id.as_ref(), None);
            if let Some(mut new_segment_part) = new_segment_part {
                // If matched matched, otherwise leave on auto
                new_segment_part.consumes_next_segment_id = true;
                next_part = Some(new_segment_part);
            }
        }
    }

    // if playlist should loop, check from 0 to currentPart
    if playlist.loop_ && next_part.is_none() && previous_part_instance.is_some() {
        // Search up until the current part
        next_part = find_first_playable_part(0, None, Some(search_from_index - 1));
    }

    next_part
}
