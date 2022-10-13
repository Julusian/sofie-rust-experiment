use std::{collections::HashSet, ops::Add};

use chrono::{DateTime, Duration, Utc};
use sofie_rust_experiment::get_random_id;

use super::{
    cache::{FakeDoc, PlayoutCache},
    lib::is_too_close_to_autonext,
    select_next_part::select_next_part,
};
use crate::{
    cache::{
        collection::{DbCacheReadCollection, DbCacheWriteCollection},
        object::{DbCacheReadObject, DbCacheWriteObject},
    },
    data_model::{
        part_instance::PartInstance,
        piece::Piece,
        piece_instance::{PieceInstance, PieceInstanceInfinite},
        rundown_playlist::{progress_hold_state, RundownHoldState},
    },
};

pub async fn take_next_part_inner(
    mut cache: PlayoutCache,
    now: DateTime<Utc>,
) -> Result<(), String> {
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
        let show_style_compound = FakeDoc {
            id: get_random_id(),
        };

        complete_hold(cache, show_style_compound).await?;

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

    reset_previous_segment(&mut cache)?;

    // Once everything is synced, we can choose the next part
    // 	await setNextPart(context, cache, nextPart) TODO

    // Setup the parts for the HOLD we are starting
    if cache.playlist.doc().previous_part_instance_id.is_some()
        && cache.playlist.doc().hold_state == RundownHoldState::ACTIVE
    {
        let hold_from_part_instance =
            &current_part_instance.ok_or_else(|| format!("previousPart not found!"))?;

        start_hold(
            &mut cache,
            &playlist_activation_id,
            hold_from_part_instance,
            &take_part_instance,
        )?;
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

pub fn reset_previous_segment(cache: &mut PlayoutCache) -> Result<(), String> {
    let current_part_instance = cache.get_current_part_instance();
    let previous_part_instance = cache.get_previous_part_instance();

    // If the playlist is looping and
    // If the previous and current part are not in the same segment, then we have just left a segment
    if let Some(previous_part_instance) = previous_part_instance {
        if cache.playlist.doc().loop_
            && Some(&previous_part_instance.segment_id)
                != current_part_instance.as_ref().map(|part| &part.segment_id)
        {
            // Reset the old segment
            let segment_id = &previous_part_instance.segment_id;

            let updated_ids = cache
                .part_instances
                .update_all(|doc| {
                    if !doc.reset && &doc.segment_id == segment_id {
                        let mut res = doc.clone();

                        res.reset = true;

                        Some(res)
                    } else {
                        None
                    }
                })
                .map_err(|_| "Failed to reset PartInstances")?;

            let updated_ids_set: HashSet<String> = HashSet::from_iter(updated_ids.into_iter());

            cache
                .piece_instances
                .update_all(|doc| {
                    if updated_ids_set.contains(&doc.part_instance_id) {
                        let mut res = doc.clone();

                        res.reset = true;

                        Some(res)
                    } else {
                        None
                    }
                })
                .map_err(|_| "Failed to reset PieceInstances")?;
        }
    }

    Ok(())
}

// async function afterTakeUpdateTimingsAndEvents(
// 	context: JobContext,
// 	cache: CacheForPlayout,
// 	showStyle: ReadonlyDeep<ProcessedShowStyleCompound>,
// 	blueprint: ReadonlyDeep<WrappedShowStyleBlueprint>,
// 	isFirstTake: boolean,
// 	takeDoneTime: number
// ): Promise<void> {
// 	const { currentPartInstance: takePartInstance, previousPartInstance } = getSelectedPartInstancesFromCache(cache)

// 	if (takePartInstance) {
// 		// Simulate playout, if no gateway
// 		const playoutDevices = cache.PeripheralDevices.findAll((d) => d.type === PeripheralDeviceType.PLAYOUT)
// 		if (playoutDevices.length === 0) {
// 			logger.info(
// 				`No Playout gateway attached to studio, reporting PartInstance "${
// 					takePartInstance._id
// 				}" to have started playback on timestamp ${new Date(takeDoneTime).toISOString()}`
// 			)
// 			reportPartInstanceHasStarted(context, cache, takePartInstance, takeDoneTime)

// 			if (previousPartInstance) {
// 				logger.info(
// 					`Also reporting PartInstance "${
// 						previousPartInstance._id
// 					}" to have stopped playback on timestamp ${new Date(takeDoneTime).toISOString()}`
// 				)
// 				reportPartInstanceHasStopped(context, cache, previousPartInstance, takeDoneTime)
// 			}

// 			// Future: is there anything we can do for simulating autoNext?
// 		}

// 		const takeRundown = takePartInstance ? cache.Rundowns.findOne(takePartInstance.rundownId) : undefined

// 		if (isFirstTake && takeRundown) {
// 			if (blueprint.blueprint.onRundownFirstTake) {
// 				const span = context.startSpan('blueprint.onRundownFirstTake')
// 				try {
// 					await blueprint.blueprint.onRundownFirstTake(
// 						new PartEventContext(
// 							'onRundownFirstTake',
// 							context.studio,
// 							context.getStudioBlueprintConfig(),
// 							showStyle,
// 							context.getShowStyleBlueprintConfig(showStyle),
// 							takeRundown,
// 							takePartInstance
// 						)
// 					)
// 				} catch (err) {
// 					logger.error(`Error in showStyleBlueprint.onRundownFirstTake: ${stringifyError(err)}`)
// 				}
// 				if (span) span.end()
// 			}
// 		}

// 		if (blueprint.blueprint.onPostTake && takeRundown) {
// 			const span = context.startSpan('blueprint.onPostTake')
// 			try {
// 				await blueprint.blueprint.onPostTake(
// 					new PartEventContext(
// 						'onPostTake',
// 						context.studio,
// 						context.getStudioBlueprintConfig(),
// 						showStyle,
// 						context.getShowStyleBlueprintConfig(showStyle),
// 						takeRundown,
// 						takePartInstance
// 					)
// 				)
// 			} catch (err) {
// 				logger.error(`Error in showStyleBlueprint.onPostTake: ${stringifyError(err)}`)
// 			}
// 			if (span) span.end()
// 		}
// 	}
// }

// export function updatePartInstanceOnTake(
// 	context: JobContext,
// 	cache: CacheForPlayout,
// 	showStyle: ReadonlyDeep<ProcessedShowStyleCompound>,
// 	blueprint: ReadonlyDeep<WrappedShowStyleBlueprint>,
// 	takeRundown: DBRundown,
// 	takePartInstance: DBPartInstance,
// 	currentPartInstance: DBPartInstance | undefined
// ): void {
// 	const playlist = cache.Playlist.doc

// 	// TODO - the state could change after this sampling point. This should be handled properly
// 	let previousPartEndState: PartEndState | undefined = undefined
// 	if (blueprint.blueprint.getEndStateForPart && currentPartInstance) {
// 		try {
// 			const time = getCurrentTime()

// 			const resolvedPieces = getResolvedPieces(context, cache, showStyle.sourceLayers, currentPartInstance)

// 			const span = context.startSpan('blueprint.getEndStateForPart')
// 			const context2 = new RundownContext(
// 				{
// 					name: `${playlist.name}`,
// 					identifier: `playlist=${playlist._id},currentPartInstance=${
// 						currentPartInstance._id
// 					},execution=${getRandomId()}`,
// 				},
// 				context.studio,
// 				context.getStudioBlueprintConfig(),
// 				showStyle,
// 				context.getShowStyleBlueprintConfig(showStyle),
// 				takeRundown
// 			)
// 			previousPartEndState = blueprint.blueprint.getEndStateForPart(
// 				context2,
// 				playlist.previousPersistentState,
// 				convertPartInstanceToBlueprints(currentPartInstance),
// 				resolvedPieces.map(convertResolvedPieceInstanceToBlueprints),
// 				time
// 			)
// 			if (span) span.end()
// 			logger.info(`Calculated end state in ${getCurrentTime() - time}ms`)
// 		} catch (err) {
// 			logger.error(`Error in showStyleBlueprint.getEndStateForPart: ${stringifyError(err)}`)
// 			previousPartEndState = undefined
// 		}
// 	}

// 	// calculate and cache playout timing properties, so that we don't depend on the previousPartInstance:
// 	const tmpTakePieces = processAndPrunePieceInstanceTimings(
// 		showStyle.sourceLayers,
// 		cache.PieceInstances.findAll((p) => p.partInstanceId === takePartInstance._id),
// 		0
// 	)
// 	const partPlayoutTimings = calculatePartTimings(
// 		cache.Playlist.doc.holdState,
// 		currentPartInstance?.part,
// 		cache.PieceInstances.findAll((p) => p.partInstanceId === currentPartInstance?._id).map((p) => p.piece),
// 		takePartInstance.part,
// 		tmpTakePieces.filter((p) => !p.infinite || p.infinite.infiniteInstanceIndex === 0).map((p) => p.piece)
// 	)

// 	cache.PartInstances.updateOne(takePartInstance._id, (p) => {
// 		p.isTaken = true
// 		p.partPlayoutTimings = partPlayoutTimings

// 		if (previousPartEndState) {
// 			p.previousPartEndState = previousPartEndState
// 		}

// 		return p
// 	})
// }

// export async function afterTake(
// 	context: JobContext,
// 	cache: CacheForPlayout,
// 	takePartInstance: DBPartInstance,
// 	timeOffsetIntoPart: number | null = null
// ): Promise<void> {
// 	const span = context.startSpan('afterTake')
// 	// This function should be called at the end of a "take" event (when the Parts have been updated)
// 	// or after a new part has started playing

// 	await updateTimeline(context, cache, timeOffsetIntoPart || undefined)

// 	cache.deferAfterSave(async () => {
// 		// This is low-prio, defer so that it's executed well after publications has been updated,
// 		// so that the playout gateway has haf the chance to learn about the timeline changes
// 		if (takePartInstance.part.shouldNotifyCurrentPlayingPart) {
// 			context
// 				.queueEventJob(EventsJobs.NotifyCurrentlyPlayingPart, {
// 					rundownId: takePartInstance.rundownId,
// 					isRehearsal: !!cache.Playlist.doc.rehearsal,
// 					partExternalId: takePartInstance.part.externalId,
// 				})
// 				.catch((e) => {
// 					logger.warn(`Failed to queue NotifyCurrentlyPlayingPart job: ${e}`)
// 				})
// 		}
// 	})

// 	if (span) span.end()
// }

/**
 * A Hold starts by extending the "extendOnHold"-able pieces in the previous Part.
 */
fn start_hold(
    cache: &mut PlayoutCache,
    activation_id: &String,
    hold_from_part_instance: &PartInstance,
    hold_to_part_instance: &PartInstance,
) -> Result<(), String> {
    let items_to_copy = cache.piece_instances.find_some(|doc| {
        doc.part_instance_id == hold_from_part_instance.id && doc.piece.extend_on_hold
    });

    for instance in items_to_copy {
        if instance.infinite.is_none() {
            let infinite_instance_id = get_random_id();

            // mark current one as infinite
            cache
                .piece_instances
                .update_one(&instance.id, |doc| {
                    let mut res = doc.clone();

                    res.infinite = Some(PieceInstanceInfinite {
                        infinite_instance_id: infinite_instance_id.clone(),
                        infinite_instance_index: 0,
                        infinite_piece_id: instance.piece.id.clone(),
                        from_previous_part: false,
                        from_hold: false,
                    });

                    Some(res)
                })
                .map_err(|_| format!("Failed to make held piece infinite"))?;

            // make the extension
            let mut new_instance_piece = instance.piece.clone();
            // new_instance_piece.enable = { start: 0};
            new_instance_piece.extend_on_hold = false;

            let new_instance = PieceInstance {
                id: format!("{}_hold", &instance.id),
                // playlistActivationId: activationId,
                // rundownId: instance.rundownId,
                part_instance_id: hold_to_part_instance.id.clone(),
                // dynamicallyInserted: getCurrentTime(),
                piece: new_instance_piece,
                reset: false,
                infinite: Some(PieceInstanceInfinite {
                    infinite_instance_id: infinite_instance_id,
                    infinite_instance_index: 1,
                    infinite_piece_id: instance.piece.id.clone(),
                    from_previous_part: true,
                    from_hold: true,
                }),
                // Preserve the timings from the playing instance
                // reportedStartedPlayback: instance.reportedStartedPlayback,
                // reportedStoppedPlayback: instance.reportedStoppedPlayback,
            };

            // TODO
            // const content = newInstance.piece.content as VTContent | undefined
            // if (content && content.fileName && content.sourceDuration && instance.plannedStartedPlayback) {
            // 	content.seek = Math.min(content.sourceDuration, getCurrentTime() - instance.plannedStartedPlayback)
            // }

            // This gets deleted once the nextpart is activated, so it doesnt linger for long
            cache
                .piece_instances
                .replace_one(new_instance)
                .map_err(|_| format!("Failed to insert held piece"))?;
        }
    }
    Ok(())
}

async fn complete_hold(
    mut cache: PlayoutCache,
    show_style_compound: FakeDoc,
) -> Result<(), String> {
    cache
        .playlist
        .update(|doc| {
            let mut res = doc.clone();

            res.hold_state = RundownHoldState::COMPLETE;

            Some(res)
        })
        .map_err(|_| format!("Failed to mark hold completed"))?;

    if cache.playlist.doc().current_part_instance_id.is_some() {
        let current_part_instance = cache
            .get_current_part_instance()
            .ok_or_else(|| format!("currentPart not found!"))?;

        // Clear the current extension line
        // TODO
        // innerStopPieces(
        // 	context,
        // 	cache,
        // 	showStyleCompound.sourceLayers,
        // 	currentPartInstance,
        // 	(p) => !!p.infinite?.fromHold,
        // 	undefined
        // )
    }

    // TODO
    // await updateTimeline(context, cache)

    Ok(())
}
