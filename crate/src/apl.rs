use serde::{Serialize, Deserialize};
use crate::constants::Action;
use crate::orchestration::Timing;
use crate::decisions::{TeamDecider, ScriptedMage, MageDecider};
use crate::legacy_config::LegacyPlayer;
use serde_json::Value;

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AplConditionType {
    #[default]
    None,
    And,
    Or,
    Cmp,
    Not,
    False,
    True,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AplConditionOp {
    #[default]
    None,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum AplValueType {
    #[default]
    None,
    Const,
    PlayerCooldownExists,
    PlayerCooldownDuration,
    PlayerAuraExists,
    PlayerAuraDuration,
    TargetAuraExists,
    TargetAuraStacks,
    TargetAuraDuration,
    SimTime,
    SimTimePercent,
    SimDuration,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Apl {
    pub items: Vec<AplItem>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AplItem {
    pub condition: AplCondition,
    pub action: AplAction,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AplCondition {
    pub condition_type: AplConditionType,
    pub op: AplConditionOp,
    pub conditions: Vec<AplCondition>,
    pub values: Vec<AplValue>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AplAction {
    pub key: Action,
    pub target_id: i32,
    pub sequence: Vec<AplAction>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AplValue {
    pub value_type: AplValueType,
    pub vstr: String,
    pub vfloat: f64,
    pub vint: i32,
}

// Helper trait for parsing enums from strings
trait FromJsonString {
    fn from_json_string(s: &str) -> Self;
}

impl FromJsonString for AplConditionType {
    fn from_json_string(s: &str) -> Self {
        match s {
            "And" => AplConditionType::And,
            "Or" => AplConditionType::Or,
            "Cmp" => AplConditionType::Cmp,
            "Not" => AplConditionType::Not,
            "False" => AplConditionType::False,
            "True" => AplConditionType::True,
            _ => AplConditionType::None,
        }
    }
}

impl FromJsonString for AplConditionOp {
    fn from_json_string(s: &str) -> Self {
        match s {
            "Eq" => AplConditionOp::Eq,
            "Neq" => AplConditionOp::Neq,
            "Gt" => AplConditionOp::Gt,
            "Gte" => AplConditionOp::Gte,
            "Lt" => AplConditionOp::Lt,
            "Lte" => AplConditionOp::Lte,
            _ => AplConditionOp::None,
        }
    }
}

impl FromJsonString for AplValueType {
    fn from_json_string(s: &str) -> Self {
        match s {
            "Const" => AplValueType::Const,
            "PlayerCooldownExists" => AplValueType::PlayerCooldownExists,
            "PlayerCooldownDuration" => AplValueType::PlayerCooldownDuration,
            "PlayerAuraExists" => AplValueType::PlayerAuraExists,
            "PlayerAuraDuration" => AplValueType::PlayerAuraDuration,
            "TargetAuraExists" => AplValueType::TargetAuraExists,
            "TargetAuraStacks" => AplValueType::TargetAuraStacks,
            "TargetAuraDuration" => AplValueType::TargetAuraDuration,
            "SimTime" => AplValueType::SimTime,
            "SimTimePercent" => AplValueType::SimTimePercent,
            "SimDuration" => AplValueType::SimDuration,
            _ => AplValueType::None,
        }
    }
}

// Helper function to convert APL action key string to Action enum
fn apl_key_to_action(key: &str) -> Action {
    match key {
        "Combustion" => Action::Combustion,
        "EphemeralPower" => Action::Toep,
        "EssenceOfSapphiron" => Action::Sapp,
        "Fireball" => Action::Fireball,
        "FireBlast" => Action::FireBlast,
        "Frostbolt" => Action::Frostbolt,
        "MindQuickening" => Action::Mqg,
        "PowerInfusion" => Action::PowerInfusion,
        "Pyroblast" => Action::Pyroblast,
        "Scorch" => Action::Scorch,
        "UnstablePower" => Action::Zhc,
        "Wait" => Action::Gcd,
        _ => Action::Gcd,
    }
}

// Helper function to safely get f64 from JSON value
fn get_f64_from_value(value: &Value, default: f64) -> f64 {
    match value {
        Value::Number(n) => n.as_f64().unwrap_or(default),
        Value::String(s) => {
            if s.is_empty() {
                default
            } else {
                s.parse().unwrap_or(default)
            }
        }
        _ => default,
    }
}

// Helper function to safely get i32 from JSON value
fn get_i32_from_value(value: &Value, default: i32) -> i32 {
    match value {
        Value::Number(n) => n.as_i64().unwrap_or(default as i64) as i32,
        Value::String(s) => {
            if s.is_empty() {
                default
            } else {
                s.parse().unwrap_or(default)
            }
        }
        _ => default,
    }
}

impl From<&Value> for AplValue {
    fn from(json: &Value) -> Self {
        let mut value = AplValue::default();
        
        if let Some(value_type_str) = json.get("value_type").and_then(|v| v.as_str()) {
            value.value_type = AplValueType::from_json_string(value_type_str);
        }
        
        if let Some(vstr) = json.get("vstr") {
            value.vstr = match vstr {
                Value::String(s) => s.clone(),
                _ => String::new(),
            };
        }
        
        if let Some(vfloat) = json.get("vfloat") {
            value.vfloat = get_f64_from_value(vfloat, 0.0);
        }
        
        if let Some(vint) = json.get("vint") {
            value.vint = get_i32_from_value(vint, 0);
        }
        
        value
    }
}



impl From<&Value> for AplCondition {
    fn from(json: &Value) -> Self {
        let mut condition = AplCondition::default();
        
        if let Some(condition_type_str) = json.get("condition_type").and_then(|v| v.as_str()) {
            condition.condition_type = AplConditionType::from_json_string(condition_type_str);
        }
        
        if let Some(op_str) = json.get("op").and_then(|v| v.as_str()) {
            condition.op = AplConditionOp::from_json_string(op_str);
        }
        
        // Parse nested conditions array (recursive)
        if let Some(conditions_array) = json.get("conditions").and_then(|v| v.as_array()) {
            condition.conditions = conditions_array.iter()
                .map(|cond_item| AplCondition::from(cond_item))
                .collect();
        }
        
        // Parse values array
        if let Some(values_array) = json.get("values").and_then(|v| v.as_array()) {
            condition.values = values_array.iter()
                .map(|val_item| AplValue::from(val_item))
                .collect();
        }
        
        condition
    }
}

// Extract default action from APL JSON
pub fn extract_default_action(apl_value: &Value) -> Action {
    apl_value
        .get("defaultAction")
        .and_then(|da| da.get("action"))
        .and_then(|action| action.get("key"))
        .and_then(|key| key.as_str())
        .map(apl_key_to_action)
        .unwrap_or(Action::Fireball) // Default fallback
}

// Extract fixed sequence from APL JSON
pub fn extract_fixed_sequence(apl_value: &Value) -> Vec<Action> {
    apl_value
        .get("fixedSequence")
        .and_then(|fs| fs.get("action"))
        .and_then(|action| action.get("sequence"))
        .and_then(|seq| seq.as_array())
        .map(|sequence_array| {
            sequence_array
                .iter()
                .filter_map(|item| {
                    item.get("key")
                        .and_then(|key| key.as_str())
                        .map(apl_key_to_action)
                })
                .collect()
        })
        .unwrap_or_else(Vec::new) // Empty sequence if not found
}


// Create a ScriptedMage from APL data
fn create_scripted_mage_from_apl(apl_value: &Value, timing: &Timing) -> Box<dyn MageDecider> {
    let default_action = extract_default_action(apl_value);
    let fixed_sequence = extract_fixed_sequence(apl_value);
    
    // Use timing values for reaction times
    let initial_react = timing.initial_delay;
    let continuing_react = timing.recast_delay;
    
    Box::new(ScriptedMage::new(
        fixed_sequence,
        default_action,
        initial_react,
        continuing_react,
    ))
}

// Function to add to your convert_legacy_to_simparams function
pub fn create_team_decider_from_players(players: &[LegacyPlayer], timing: &Timing) -> TeamDecider {
    let mut mages: Vec<Box<dyn MageDecider>> = Vec::new();
    
    for player in players {
        let mage_decider = if let Some(apl_value) = &player.apl {
            create_scripted_mage_from_apl(apl_value, timing)
        } else {
            // Fallback for players without APL data
            Box::new(ScriptedMage::new(
                vec![], // No opener sequence
                Action::Fireball, // Default action
                timing.initial_delay,
                timing.recast_delay,
            ))
        };
        
        mages.push(mage_decider);
    }
    
    TeamDecider::new(mages)
}
