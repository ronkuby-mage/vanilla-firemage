//! decisions.rs — rotation logic

use crate::constants::{Action, Buff};
use crate::state::State;

fn action_to_buff(action: Action) -> Option<Buff> {
    match action {
        Action::Sapp => Some(Buff::Sapp),
        Action::Toep => Some(Buff::Toep),
        Action::Zhc => Some(Buff::Zhc),
        Action::Mqg => Some(Buff::Mqg),
        Action::PowerInfusion => Some(Buff::PowerInfusion),
        _ => None,
    }
}

fn buff_ready_for_action(st: &State, lane: usize, action: Action) -> bool {
    if let Some(buff) = action_to_buff(action) {
        st.lanes[lane].buff_cooldown[buff as usize] <= 0.0
    } else {
        false
    }
}

pub trait Decider {
    fn next_action(&mut self, st: &State) -> Option<(usize, Action, f64)>;
}

pub trait MageDecider {
    /// Decide next action for *this lane only*.
    /// Return (action, reaction_sigma) or None to skip.
    fn decide(&mut self, state: &State, lane: usize) -> Option<(Action, f64)>;
}

pub struct ScriptedMage {
    stage: usize,            // per-lane progress
    initial_sequence: Vec<Action>, // opener shared by all lanes
    default_action: Action,
    initial_react: f64,
    continuing_react: f64,
}

impl ScriptedMage {
    pub fn new(
        initial_sequence: Vec<Action>,
        default_action: Action,
        initial_react: f64,
        continuing_react: f64,
    ) -> Self {
        Self {
            stage: 0,
            initial_sequence,
            default_action,
            initial_react,
            continuing_react,
        }
    }
}

impl MageDecider for ScriptedMage {
    fn decide(&mut self, st: &State, lane: usize) -> Option<(Action, f64)> {
        // walk opener (optionally skipping unready buff casts)
        let mut s = self.stage;
        while s < self.initial_sequence.len() {
            let action = self.initial_sequence[s];
            let react: f64 = if s == 0 { self.initial_react } else { self.continuing_react };
            if !action_to_buff(action).is_some() || buff_ready_for_action(st, lane, action) {
                self.stage = s + 1;
                return Some((action, react));
            }
            s += 1;
        }
        Some((self.default_action, self.continuing_react))
    }
}

pub struct TeamDecider {
    mages: Vec<Box<dyn MageDecider>>,
}

impl TeamDecider {
    pub fn new(mages: Vec<Box<dyn MageDecider>>) -> Self { Self { mages } }
}

impl Decider for TeamDecider {
    fn next_action(&mut self, st: &State) -> Option<(usize, Action, f64)> {
        let lane = st.next_cast_lane()?;
        let (act, sig) = self.mages[lane].decide(st, lane)?;
        Some((lane, act, sig))
    }
}