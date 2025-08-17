// lib.rs — module exports
// Ensure we import the Rng trait in each file that needs `gen()` to avoid reserved keyword issues.
pub mod constants;
pub mod state;
pub mod decisions;
pub mod orchestration;
pub mod legacy_config; // if you added it

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use crate::orchestration::{SimParams, run_single, run_many_with, SimulationResult, SimulationsResult};
use crate::legacy_config::{LegacyConfig, convert_legacy_to_simparams};
use crate::constants::Action;
use crate::decisions::{TeamDecider, MageDecider, ScriptedMage};
use console_error_panic_hook;
use log::{Level};


/// Build a default per-mage TeamDecider (same shape as your factory)
fn build_team_decider(params: &SimParams) -> TeamDecider {

    let m = params.config.num_mages;
    let mages: Vec<Box<dyn MageDecider>> = (0..m)
        .map(|_| {
            // If your ScriptedMage::new has (sequence, default, init_sigma, cont_sigma)
            Box::new(ScriptedMage::new(vec![], Action::Fireball, 0.05, 0.05))
                as Box<dyn MageDecider>
        })
        .collect();
    TeamDecider::new(mages)
}

#[wasm_bindgen]
pub fn run_simulations(cfg_js: JsValue, iterations: i32) -> JsValue {
    //console_error_panic_hook::set_once();
    //console_log::init_with_level(Level::Debug).expect("error initializing log");
    let legacy: LegacyConfig = from_value(cfg_js).expect("bad config from JS");
    let params: SimParams = convert_legacy_to_simparams(legacy);

    let make_decider = || build_team_decider(&params);

    let seed = 42u64; // or take from legacy.rng_seed.unwrap_or(42)
    let results: SimulationsResult = run_many_with::<_, _>(&params, make_decider, iterations, seed);


    serde_wasm_bindgen::to_value(&results).unwrap()
}

#[wasm_bindgen]
pub fn run_simulation(cfg_js: JsValue) -> JsValue {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Debug).expect("error initializing log");
    log::debug!("Engine made it this far.");

    let legacy: LegacyConfig = from_value(cfg_js).expect("bad config from JS");
    let params: SimParams = convert_legacy_to_simparams(legacy);

    let mut decider: TeamDecider = build_team_decider(&params);

    let seed = 42u64; // or take from legacy.rng_seed.unwrap_or(42)
    let result: SimulationResult = run_single(&params, &mut decider, seed, 0);


    // // Aggregate like the old UI expects
    // let m = params.config.num_mages as f64;
    // let targets = params.config.target.len().max(1) as f64;


    // let out = serde_json::json!({
    //     "iterations": 1,
    //     "dps": result.total_dps / m,
    //     "ignite_dps": result.ignite_dps / m,
    //     "player_dps_per_target": result.player_dps / targets,
    //     // add min/max/hist if the UI expects them
    // });
    log::debug!("Engine made it THIS!!!!! far.");

    to_value(&result).unwrap()
}