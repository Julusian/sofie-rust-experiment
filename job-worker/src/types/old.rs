pub type Time = i64;

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct PartId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct RundownId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct SegmentId {
    #[protected_value]
    id: String,
}

pub struct Piece {
    // pub startPartId: PartId, // pub lifespan:
}

pub struct PartInstance {
    //
    pub timings: Option<PartInstanceTimings>,
}

pub struct PartInstanceTimings {
    /** Point in time the Part was taken, (ie the time of the user action) */
    pub take: Option<Time>,
    /** Point in time the "take" action has finished executing */
    pub takeDone: Option<Time>,
    /** Point in time the Part started playing (ie the time of the playout) */
    pub startedPlayback: Option<Time>,
    /** Point in time the Part stopped playing (ie the time of the user action) */
    pub takeOut: Option<Time>,
    /** Point in time the Part stopped playing (ie the time of the playout) */
    pub stoppedPlayback: Option<Time>,
    /** Point in time the Part was set as Next (ie the time of the user action) */
    pub next: Option<Time>,

    /** The playback offset that was set for the last take */
    pub playOffset: Option<Time>,
    /**
     * The duration this part was playing for.
     * This is set when the next part has started playback
     */
    pub duration: Option<Time>,
}
impl Default for PartInstanceTimings {
    fn default() -> Self {
        Self {
            take: None,
            takeDone: None,
            startedPlayback: None,
            takeOut: None,
            stoppedPlayback: None,
            next: None,
            playOffset: None,
            duration: None,
        }
    }
}

pub struct PieceInstance {
    //
}

pub struct Part {
    pub _id: PartId,
    pub segmentId: SegmentId,

    pub expectedDuration: Option<Time>,
    pub autoNext: bool,
    pub budgetDuration: Option<Time>,

    pub displayDurationGroup: Option<String>,
}
impl Part {
    pub fn canAutoNext(&self) -> bool {
        if self.autoNext {
            if let Some(dur) = self.expectedDuration {
                return dur > 0;
            } else {
                false
            }
        } else {
            false
        }
    }
}

pub struct Rundown {
    pub _id: RundownId,

    pub endOfRundownIsShowBreak: bool,
}

pub struct RundownPlaylist {
    //
}
