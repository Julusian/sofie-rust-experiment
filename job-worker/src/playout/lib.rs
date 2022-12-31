use std::ops::Sub;

use chrono::{Duration, Utc};

use crate::data_model::part_instance::PartInstance;

/**
 * time in ms before an autotake when we don't accept takes/updates
 */

pub fn is_too_close_to_autonext(current_part_instance: &PartInstance, is_take: bool) -> bool {
    if !current_part_instance.part.autonext {
        false
    } else {
        // TODO - make constant
        let AUTOTAKE_UPDATE_DEBOUNCE: Duration = Duration::milliseconds(5000);
        let AUTOTAKE_TAKE_DEBOUNCE: Duration = Duration::milliseconds(1000);

        let debounce = if is_take {
            AUTOTAKE_TAKE_DEBOUNCE
        } else {
            AUTOTAKE_UPDATE_DEBOUNCE
        };

        if let Some(start) = current_part_instance.timings.planned_started_playback {
            if let Some(expected_duration) = current_part_instance.part.expected_duration {
                // date.now - start = playback duration, duration + offset gives position in part
                let playback_duration = start.signed_duration_since(Utc::now());

                // If there is an auto next planned
                expected_duration.sub(playback_duration) < debounce
            } else {
                false
            }
        } else {
            false
        }
    }
}
