use chrono::Duration;

use crate::cache::doc::DocWithId;

#[derive(Clone)]
pub struct PartInTransition {
    /** Duration this transition block a take for. After this time, another take is allowed which may cut this transition off early */
    pub block_take_duration: Duration,
    /** Duration the previous part be kept playing once the transition is started. Typically the duration of it remaining in-vision */
    pub previous_part_keepalive_duration: Duration,
    /** Duration the pieces of the part should be delayed for once the transition starts. Typically the duration until the new part is in-vision */
    pub part_content_delay_duration: Duration,
}
#[derive(Clone)]
pub struct PartOutTransition {
    /** How long to keep this part alive after taken out  */
    pub duration: Duration,
}

#[derive(Clone)]
pub struct Part {
    pub id: String,
    pub rank: usize,

    pub segment_id: String,

    // pub autonext: bool, Implied by autonext_overlap being defined
    pub autonext_overlap: Option<Duration>,

    pub disable_next_in_transition: bool,
    pub in_transition: Option<PartInTransition>,
    pub out_transition: Option<PartOutTransition>,
    pub untimed: bool,

    pub expected_duration: Option<Duration>,

    pub invalid: bool,
    pub floated: bool,
}
impl<'a> DocWithId<'a> for Part {
    fn doc_id(&'a self) -> &'a str {
        &self.id
    }
}
impl Part {
    pub fn is_playable(&self) -> bool {
        !self.invalid && !self.floated
    }
}
