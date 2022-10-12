use std::ops::Add;

use chrono::{DateTime, Duration, Utc};

use super::{
    cache::PlayoutCache, lib::is_too_close_to_autonext, select_next_part::select_next_part,
};
use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollection},
        object::{DbCacheReadObject, DbCacheWriteObject},
    },
    data_model::{
        part_instance::PartInstance,
        rundown_playlist::{progress_hold_state, RundownHoldState},
    },
};

pub fn take_next_part_inner(mut cache: PlayoutCache, now: DateTime<Utc>) -> Result<(), String> {
    let playlist_activation_id = {
        let playlist = cache.playlist.doc();

        if let Some(activation_id) = &playlist.activation_id {
            Ok(activation_id.clone())
        } else {
            Err(format!("Rundown Playlist {} is not active!", playlist.id))
        }
    }?;

    let time_offset = cache.playlist.doc().next_time_offset;

    let current_part_instance = cache.get_current_part_instance();
    let next_part_instance = cache.get_next_part_instance();
    let previous_part_instance = cache.get_previous_part_instance();

    let current_rundown = {
        let current_or_next = current_part_instance
            .as_ref()
            .or(next_part_instance.as_ref());

        if let Some(part_instance) = current_or_next {
            cache
                .rundowns
                .find_one_by_id(&part_instance.rundown_id)
                .ok_or_else(|| {
                    format!(
                        "Rundown \"{}\" could not be found!",
                        &part_instance.rundown_id
                    )
                })
        } else {
            Err("No PartInstance could be found!".to_string())
        }
    }?;

    // 	const pShowStyle = context.getShowStyleCompound(currentRundown.showStyleVariantId, currentRundown.showStyleBaseId)

    if let Some(current_part_instance) = &current_part_instance {
        let now = Utc::now(); // TODO - this replaces a now above?

        if let Some(block_take_until) = current_part_instance.block_take_until {
            let remaining_time = block_take_until.signed_duration_since(now);
            if remaining_time > Duration::zero() {
                println!(
                    "Take is blocked until {}. Which is in: {}ms",
                    block_take_until,
                    remaining_time.num_milliseconds()
                );
                return Err("TakeBlockedDuration".to_string()); // TODO - UserError
            }
        }

        // If there was a transition from the previous Part, then ensure that has finished before another take is permitted
        let allow_transition = previous_part_instance
            .map_or(true, |instance| !instance.part.disable_next_in_transition);
        if allow_transition {
            if let Some(start) = current_part_instance.timings.planned_started_playback {
                if let Some(in_transition) = &current_part_instance.part.in_transition {
                    if now < start.add(in_transition.block_take_duration) {
                        return Err("TakeDuringTransition".to_string());
                    }
                }
            }
        }

        if is_too_close_to_autonext(&current_part_instance, true) {
            return Err("TakeCloseToAutonext".to_string()); // TODO - UserError
        }
    }

    if cache.playlist.doc().hold_state == RundownHoldState::COMPLETE {
        let err = cache.playlist.update(|doc| {
            let mut res = doc.clone();
            res.hold_state = RundownHoldState::NONE;
            Some(res)
        });
        if let Err(err) = err {
            println!("Failed to update PartInstance")
        }

        // If hold is active, then this take is to clear it
    } else if cache.playlist.doc().hold_state == RundownHoldState::ACTIVE {
        // TODO
        // await completeHold(context, cache, await pShowStyle, currentPartInstance)

        return Ok(());
    }

    let take_part_instance = next_part_instance.ok_or_else(|| "takePart not found!".to_string())?;
    let take_rundown = cache
        .rundowns
        .find_one_by_id(&take_part_instance.rundown_id)
        .ok_or_else(|| "takeRundown: takeRundown not found!".to_string())?;

    // Autonext may have setup the plannedStartedPlayback. Clear it so that a new value is generated
    cache
        .part_instances
        .update_one(&take_part_instance.id, |doc| {
            if doc.timings.planned_started_playback.is_some() {
                let mut res = doc.clone();
                res.timings.planned_started_playback = None;
                res.timings.planned_stopped_playback = None;
                Some(res)
            } else {
                None
            }
        })
        .map_err(|_| format!("Failed to clear plannedStartedPlayback"))?;

    // it is only a first take if the Playlist has no startedPlayback and the taken PartInstance is not untimed
    let is_first_time =
        !cache.playlist.doc().started_playback.is_some() && !take_part_instance.part.untimed;

    clear_next_segment_id(&mut cache, &take_part_instance)?;

    let next_part = select_next_part(
        cache.playlist.doc(),
        Some(&take_part_instance),
        None,
        cache.get_ordered_segments_and_parts(),
        true,
    );

    // TODO
    // 	const showStyle = await pShowStyle
    // 	const blueprint = await context.getShowStyleBlueprint(showStyle._id)
    // 	if (blueprint.blueprint.onPreTake) {
    // 		const span = context.startSpan('blueprint.onPreTake')
    // 		try {
    // 			await blueprint.blueprint.onPreTake(
    // 				new PartEventContext(
    // 					'onPreTake',
    // 					context.studio,
    // 					context.getStudioBlueprintConfig(),
    // 					showStyle,
    // 					context.getShowStyleBlueprintConfig(showStyle),
    // 					takeRundown,
    // 					takePartInstance
    // 				)
    // 			)
    // 		} catch (err) {
    // 			logger.error(`Error in showStyleBlueprint.onPreTake: ${stringifyError(err)}`)
    // 		}
    // 		if (span) span.end()
    // 	}

    // 	updatePartInstanceOnTake(context, cache, showStyle, blueprint, takeRundown, takePartInstance, currentPartInstance)

    cache
        .playlist
        .update(|doc| {
            let mut res = doc.clone();

            res.previous_part_instance_id = res.current_part_instance_id;
            res.current_part_instance_id = Some(take_part_instance.id.clone());

            res.hold_state = progress_hold_state(&doc.hold_state);

            Some(res)
        })
        .map_err(|_| "Failed to update selected instance ids".to_string())?;

    cache
        .part_instances
        .update_one(&take_part_instance.id, |doc| {
            let mut res = doc.clone();

            res.is_taken = true;

            res.timings.take = Some(now);
            res.timings.play_offset = time_offset;

            Some(res)
        })
        .map_err(|_| "Failed to update taken partinstance".to_string())?;

    // 	resetPreviousSegment(cache) TODO

    // Once everything is synced, we can choose the next part
    // 	await setNextPart(context, cache, nextPart) TODO

    // Setup the parts for the HOLD we are starting
    if cache.playlist.doc().previous_part_instance_id.is_some()
        && cache.playlist.doc().hold_state == RundownHoldState::ACTIVE
    {
        // 		startHold(context, cache, playlistActivationId, currentPartInstance, nextPartInstance) TODO
    }

    // 	await afterTake(context, cache, takePartInstance, timeOffset) TODO

    // Last: TODO
    // 	const takeDoneTime = getCurrentTime()
    // 	cache.defer(async (cache2) => {
    // 		await afterTakeUpdateTimingsAndEvents(context, cache2, showStyle, blueprint, isFirstTake, takeDoneTime)
    // 	})
    Ok(())
}

pub fn clear_next_segment_id(
    cache: &mut PlayoutCache,
    take_or_current_part_instance: &PartInstance,
) -> Result<(), String> {
    if take_or_current_part_instance.consumes_next_segment_id
        && cache.playlist.doc().next_segment_id.as_ref()
            == Some(&take_or_current_part_instance.segment_id)
    {
        // clear the nextSegmentId if the newly taken partInstance says it was selected because of it
        cache
            .playlist
            .update(|doc| {
                let mut res = doc.clone();

                res.next_segment_id = None;

                Some(res)
            })
            .map_err(|_| "Failed to clear nextSegmentId".to_string())?;
    }

    Ok(())
}
