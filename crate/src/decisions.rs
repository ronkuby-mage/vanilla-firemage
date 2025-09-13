//! decisions.rs — rotation logic
use crate::constants as C;
use crate::constants::{Action, Buff};
use crate::state::State;
use crate::apl::{AplItem, AplConditionType, AplConditionOp, AplCondition, AplValue, AplValueType};

fn action_to_buff(action: Action) -> Option<Buff> {
    match action {
        Action::Sapp => Some(Buff::Sapp),
        Action::Toep => Some(Buff::Toep),
        Action::Zhc => Some(Buff::Zhc),
        Action::Mqg => Some(Buff::Mqg),
        _ => None,
    }
}

fn action_ready_for_action(st: &State, lane: usize, action: Action) -> bool {
    if action_to_buff(action).is_some() {
        if let Some(buff) = action_to_buff(action) {
            return st.lanes[lane].buff_cooldown[buff as usize] <= 0.0
        } else {
            return false
        }
    } else if action == Action::FireBlast {
        return st.lanes[lane].fb_cooldown <= 0.0;
    } else if action == Action::PowerInfusion {
        return st.lanes[lane].pi_cooldown.iter().cloned().reduce(f64::min).unwrap() <= 0.0;
    } else if action == Action::Combustion {
        return st.lanes[lane].comb_cooldown <= 0.0;
    } else if action == Action::ArcanePower {
        return st.lanes[lane].ap_cooldown <= 0.0;
    } else if action == Action::PresenceOfMind {
        return st.lanes[lane].pom_cooldown <= 0.0;
    } else if action == Action::Berserking {
        return st.lanes[lane].berserk_cooldown <= 0.0;
    }

    true
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
    recast_delay: f64,
}

impl ScriptedMage {
    pub fn new(
        initial_sequence: Vec<Action>,
        default_action: Action,
        recast_delay: f64,
    ) -> Self {
        Self {
            stage: 0,
            initial_sequence,
            default_action,
            recast_delay,
        }
    }
}

impl MageDecider for ScriptedMage {
    fn decide(&mut self, st: &State, lane: usize) -> Option<(Action, f64)> {
        // walk opener (optionally skipping unready buff casts)
        let mut s = self.stage;
        while s < self.initial_sequence.len() {
            let action = self.initial_sequence[s];
            if action_ready_for_action(st, lane, action) {
                self.stage = s + 1;
                return Some((action, self.recast_delay)); // no double dip on reaction time
            }
            s += 1;
        }
        self.stage = self.initial_sequence.len();        
        Some((self.default_action, self.recast_delay))
    }
}

pub struct TeamDecider {
    mages: Vec<Box<dyn MageDecider>>,
}

impl TeamDecider {
    pub fn new(mages: Vec<Box<dyn MageDecider>>) -> Self { 
        Self { mages } 
    }
}

impl Decider for TeamDecider {
    fn next_action(&mut self, st: &State) -> Option<(usize, Action, f64)> {
        let lane = st.next_cast_lane()?;
        let (act, sig) = self.mages[lane].decide(st, lane)?;
        Some((lane, act, sig))
    }
}

// Updated enum with Auto context
#[derive(Debug, Clone, Copy)]
enum ValueContext {
    Int,   // For exists checks and stacks
    Float, // For durations, times, percentages  
    Auto,  // For constants when both constants are being compared
}

pub struct AdaptiveMage {
    stage: usize,            // per-lane progress in opener
    initial_sequence: Vec<Action>, // opener shared by all lanes
    items: Vec<AplItem>,     // priority rotation rules
    default_action: Action,
    recast_delay: f64,
    reaction_time: f64,
}

impl AdaptiveMage {
    pub fn new(
        initial_sequence: Vec<Action>,
        items: Vec<AplItem>,
        default_action: Action,
        recast_delay: f64,
        reaction_time: f64,
    ) -> Self {
        Self {
            stage: 0,
            initial_sequence,
            items,
            default_action,
            recast_delay,
            reaction_time,
        }
    }

    fn conditional_action(&self, items: &[AplItem], st: &State, lane: usize) -> Option<Action> {
        // Iterate through items in priority order
        for item in items {
            if self.evaluate_condition(&item.condition, st, lane) {
                if action_ready_for_action(st, lane, item.action) {
                    return Some(item.action);
                }
            }
        }
        None
    }

    fn evaluate_condition(&self, condition: &AplCondition, st: &State, lane: usize) -> bool {

        match condition.condition_type {
            AplConditionType::None => true,
            AplConditionType::True => {
                if condition.values.len() >= 1 {
                    let context = self.infer_value_context(&condition.values[0]);
                    self.evaluate_value(&condition.values[0], st, lane, context) != 0.0
                } else {
                    false
                }
            }
            AplConditionType::False => {
                if condition.values.len() >= 1 {
                    let context = self.infer_value_context(&condition.values[0]);
                    self.evaluate_value(&condition.values[0], st, lane, context) == 0.0
                } else {
                    false
                }
            }
            AplConditionType::Not => {
                if condition.conditions.len() >= 1 {
                    !self.evaluate_condition(&condition.conditions[0], st, lane)
                } else {
                    false
                }
            }
            AplConditionType::And => {
                condition.conditions.iter().all(|c| self.evaluate_condition(c, st, lane))
            }
            AplConditionType::Or => {
                condition.conditions.iter().any(|c| self.evaluate_condition(c, st, lane))
            }
            AplConditionType::Cmp => {
                if condition.values.len() >= 2 {
                    self.compare_two_values(&condition.values[0], &condition.values[1], &condition.op, st, lane)
                } else {
                    false
                }
            }
        }
    }

    fn compare_two_values(&self, left_val: &AplValue, right_val: &AplValue, op: &AplConditionOp, st: &State, lane: usize) -> bool {

        // Determine comparison context based on non-constant types
        let context = match (left_val.value_type, right_val.value_type) {
            (AplValueType::Const, other) | (other, AplValueType::Const) => {
                self.infer_value_context_from_type(other)
            }
            (left_type, _) => {
                // Both non-constant, use left side to determine context
                self.infer_value_context_from_type(left_type)
            }
        };
        // special case: Ignite or scorch debuff
        if left_val.value_type == AplValueType::TargetAuraDuration || right_val.value_type == AplValueType::TargetAuraDuration {
            return self.compare_debuff_duration_with_reaction_time(left_val, right_val, op, st, lane, context);
        }

        let left = self.evaluate_value(left_val, st, lane, context);
        let right = self.evaluate_value(right_val, st, lane, context);

        self.compare_values(left, right, op)
    }

    fn infer_value_context(&self, value: &AplValue) -> ValueContext {
        self.infer_value_context_from_type(value.value_type)
    }

    fn infer_value_context_from_type(&self, value_type: AplValueType) -> ValueContext {
        
        match value_type {
            AplValueType::SimTime | AplValueType::SimTimePercent | AplValueType::SimDuration |
            AplValueType::PlayerCooldownDuration | AplValueType::PlayerAuraDuration | AplValueType::TargetAuraDuration => {
                ValueContext::Float
            }
            AplValueType::PlayerCooldownExists | AplValueType::PlayerAuraExists | AplValueType::TargetAuraExists |
            AplValueType::TargetAuraStacks => {
                ValueContext::Int
            }
            AplValueType::Const => ValueContext::Auto, // Will be determined by context
            _ => ValueContext::Float,
        }
    }

    fn get_const_value(&self, value: &AplValue, context: ValueContext) -> f64 {
        // Step 2: Check both int and float, use non-zero value
        let has_float = value.vfloat != 0.0;
        let has_int = value.vint != 0;

        match (has_int, has_float) {
            (false, false) => 0.0, // Both empty/zero
            (true, false) => value.vint as f64, // Only int is non-zero
            (false, true) => value.vfloat, // Only float is non-zero
            (true, true) => {
                // Both non-zero, choose based on context
                match context {
                    ValueContext::Int => value.vint as f64,
                    ValueContext::Float => value.vfloat,
                    ValueContext::Auto => {
                        // If both constants, use int if no floating point component
                        if value.vfloat.fract() == 0.0 {
                            value.vint as f64
                        } else {
                            value.vfloat
                        }
                    }
                }
            }
        }
    }

    fn js_constant_to_buff(&self, constant: i32) -> Option<Buff> {
        match constant {
            28779 => Some(Buff::Sapp),    // ESSENCE_OF_SAPPHIRON
            23723 => Some(Buff::Mqg),     // MIND_QUICKENING  
            23271 => Some(Buff::Toep),    // EPHEMERAL_POWER
            24658 => Some(Buff::Zhc),     // UNSTABLE_POWER
            _ => None,
        }
    }

    fn evaluate_value(&self, value: &AplValue, st: &State, lane: usize, context: ValueContext) -> f64 {

        match value.value_type {
            AplValueType::Const => {
                self.get_const_value(value, context)
            }
            AplValueType::SimTime => st.global.running_time,
            AplValueType::SimTimePercent => (st.global.running_time / st.global.duration) * 100.0,
            AplValueType::SimDuration => st.global.duration - st.global.running_time,
            
            AplValueType::PlayerCooldownExists => {
                match value.vint {
                    29977 => if st.lanes[lane].comb_cooldown > 0.0 { 1.0 } else { 0.0 }, // COMBUSTION
                    10199 => if st.lanes[lane].fb_cooldown > 0.0 { 1.0 } else { 0.0 },   // FIRE_BLAST
                    10060 => if st.lanes[lane].pi_cooldown.iter().cloned().reduce(f64::min).unwrap() > 0.0 { 1.0 } else { 0.0 },   // PI
                    12042 => if st.lanes[lane].ap_cooldown > 0.0 { 1.0 } else { 0.0 },
                    12043 => if st.lanes[lane].pom_cooldown > 0.0 { 1.0 } else { 0.0 },
                    20554 => if st.lanes[lane].berserk_cooldown > 0.0 { 1.0 } else { 0.0 },
                    _ => {
                        if let Some(buff) = self.js_constant_to_buff(value.vint) {
                            if st.lanes[lane].buff_cooldown[buff as usize] > 0.0 { 1.0 } else { 0.0 }
                        } else {
                            0.0
                        }
                    }
                }
            }
            
            AplValueType::PlayerCooldownDuration => {
                match value.vint {
                    29977 => st.lanes[lane].comb_cooldown.max(0.0), // COMBUSTION
                    10199 => st.lanes[lane].fb_cooldown.max(0.0),   // FIRE_BLAST
                    10060 => st.lanes[lane].pi_cooldown.iter().cloned().reduce(f64::min).unwrap().max(0.0),
                    12042 => st.lanes[lane].ap_cooldown.max(0.0),
                    12043 => st.lanes[lane].pom_cooldown.max(0.0),
                    20554 => st.lanes[lane].berserk_cooldown.max(0.0),
                    _ => {
                        if let Some(buff) = self.js_constant_to_buff(value.vint) {
                            st.lanes[lane].buff_cooldown[buff as usize].max(0.0)
                        } else {
                            0.0
                        }
                    }
                }
            }
            
            AplValueType::PlayerAuraExists => {
                match value.vint {
                    29977 => st.lanes[lane].comb_left as f64, // COMBUSTION - use comb_left
                    10060 => if st.lanes[lane].pi_timer.iter().cloned().reduce(f64::max).unwrap() > 0.0 { 1.0 } else { 0.0 },
                    12042 => if st.lanes[lane].ap_timer > 0.0 {1.0} else { 0.0 },
                    12043 => if st.lanes[lane].pom_active {1.0} else { 0.0 },
                    20554 => if st.lanes[lane].berserk_timer > 0.0 {1.0} else { 0.0 },
                    _ => {
                        if let Some(buff) = self.js_constant_to_buff(value.vint) {
                            if st.lanes[lane].buff_timer[buff as usize] > 0.0 { 1.0 } else { 0.0 }
                        } else {
                            0.0
                        }
                    }
                }
            }
            
            AplValueType::PlayerAuraDuration => {
                match value.vint {
                    29977 => 0.0, // COMBUSTION - no duration for combustion aura
                    10060 => st.lanes[lane].pi_timer.iter().cloned().reduce(f64::max).unwrap().max(0.0),
                    12042 => st.lanes[lane].ap_timer.max(0.0),
                    20554 => st.lanes[lane].berserk_timer.max(0.0),
                    _ => {
                        if let Some(buff) = self.js_constant_to_buff(value.vint) {
                            st.lanes[lane].buff_timer[buff as usize].max(0.0)
                        } else {
                            0.0
                        }
                    }
                }
            }
            
            AplValueType::TargetAuraExists => {
                match value.vint {
                    22959 => if st.boss.scorch_timer > 0.0 { 1.0 } else { 0.0 }, // FIRE_VULNERABILITY
                    12654 => if st.boss.ignite_timer > 0.0 { 1.0 } else { 0.0 }, // Ignite
                    _ => 0.0,
                }
            }
            
            AplValueType::TargetAuraStacks => {
                match value.vint {
                    22959 => if st.boss.scorch_timer > 0.0 { st.boss.scorch_count as f64 } else { 0.0 }, // FIRE_VULNERABILITY (scorch stacks)
                    12654 => if st.boss.ignite_timer > 0.0 { st.boss.ignite_count as f64 } else { 0.0 },     // Ignite stacks
                    _ => 0.0,
                }
            }
            
            AplValueType::TargetAuraDuration => {
                match value.vint {
                    22959 => st.boss.scorch_timer.max(0.0), // FIRE_VULNERABILITY
                    12654 => st.boss.ignite_timer.max(0.0),     // Ignite
                    _ => 0.0,
                }
            }
            
            _ => 0.0,
        }
    }

    fn compare_values(&self, left: f64, right: f64, op: &AplConditionOp) -> bool {
        use crate::apl::AplConditionOp;
        
        match op {
            AplConditionOp::Eq => (left - right).abs() < f64::EPSILON,
            AplConditionOp::Neq => (left - right).abs() >= f64::EPSILON,
            AplConditionOp::Gt => left > right,
            AplConditionOp::Gte => left >= right,
            AplConditionOp::Lt => left < right,
            AplConditionOp::Lte => left <= right,
            AplConditionOp::None => false,
        }
    }

    fn compare_debuff_duration_with_reaction_time(
        &self, 
        left_val: &AplValue, 
        right_val: &AplValue, 
        op: &AplConditionOp, 
        st: &State, 
        lane: usize,
        context: ValueContext
    ) -> bool {
        // Identify which is the debuff duration and which is the threshold
        let (duration_val, threshold_val, reversed) = if left_val.value_type == AplValueType::TargetAuraDuration {
            (left_val, right_val, false)
        } else {
            (right_val, left_val, true)
        };
        
        // Get current values
        let current_duration = self.evaluate_value(duration_val, st, lane, context);
        let threshold = self.evaluate_value(threshold_val, st, lane, context);
        
        // Determine which debuff we're checking
        let (refresh_history, reaction_time) = match duration_val.vint {
            22959 => (&st.boss.scorch_refresh_history, self.reaction_time), // FIRE_VULNERABILITY (scorch)
            12654 => (&st.boss.ignite_refresh_history, self.reaction_time), // Ignite
            _ => {
                // Unknown debuff, fall back to simple comparison
                return if reversed {
                    self.compare_values(threshold, current_duration, op)
                } else {
                    self.compare_values(current_duration, threshold, op)
                };
            }
        };
        
        // Check if condition would have been true within reaction time window
        let current_time = st.global.running_time;
        let reaction_window_start = current_time - reaction_time;
        
        // Find if there was a refresh within our reaction window
        let refreshed_recently = refresh_history.iter()
            .any(|&refresh_time| refresh_time > reaction_window_start && refresh_time <= current_time);
        
        if refreshed_recently {
            // Calculate what the timer would have been at reaction_window_start
            // This assumes we know the debuff duration constants
            let debuff_duration = match duration_val.vint {
                22959 => C::SCORCH_TIME,
                12654 => C::IGNITE_TIME,
                _ => 0.0,
            };
            
            // Find the most recent refresh before reaction_window_start
            let last_refresh_before = refresh_history.iter()
                .filter(|&&t| t <= reaction_window_start)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .copied();

            if let Some(refresh_time) = last_refresh_before {
                // Calculate what the timer would have been when we started reacting
                let timer_at_reaction_start = debuff_duration - (reaction_window_start - refresh_time);
                
                if timer_at_reaction_start > 0.0 {
                    // The debuff was active when we started reacting
                    // Check if the condition would have been true then
                    let condition_was_true = if reversed {
                        self.compare_values(threshold, timer_at_reaction_start, op)
                    } else {
                        self.compare_values(timer_at_reaction_start, threshold, op)
                    };
                    
                    if condition_was_true {
                        // Condition was true when we started reacting, honor it
                        return true;
                    }
                }
            }
        }
        
        // No recent refresh or condition wasn't true before, use current values
        if reversed {
            self.compare_values(threshold, current_duration, op)
        } else {
            self.compare_values(current_duration, threshold, op)
        }
    }    

}

pub struct AdaptiveTeamDecider {
    mages: Vec<Box<dyn MageDecider>>,
}

impl AdaptiveTeamDecider {
    pub fn new(mages: Vec<Box<dyn MageDecider>>) -> Self { 
        Self { mages } 
    }
}

impl Decider for AdaptiveTeamDecider {
    fn next_action(&mut self, st: &State) -> Option<(usize, Action, f64)> {
        let lane = st.next_cast_lane()?;
        let (act, sig) = self.mages[lane].decide(st, lane)?;
        Some((lane, act, sig))
    }
}

impl MageDecider for AdaptiveMage {
    fn decide(&mut self, st: &State, lane: usize) -> Option<(Action, f64)> {
        // First, try to execute the opener sequence
        let mut s = self.stage;
        while s < self.initial_sequence.len() {
            let action = self.initial_sequence[s];

            // For buff actions, check if they're ready; for non-buff actions, always proceed
            if action_ready_for_action(st, lane, action) {
                self.stage = s + 1;
                return Some((action, self.recast_delay));
            }
            s += 1;
        }
        self.stage = self.initial_sequence.len();
        
        // Opener is complete, now use the adaptive priority list
        if let Some(action) = self.conditional_action(&self.items, st, lane) {
            // Check if this action is ready (for buff actions)
            return Some((action, self.recast_delay));
        }
        
        // Fall back to default action
        Some((self.default_action, self.recast_delay))
    }
}