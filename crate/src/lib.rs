// lib.rs — module exports
// Ensure we import the Rng trait in each file that needs `gen()` to avoid reserved keyword issues.
pub mod constants;
pub mod state;
pub mod decisions;
pub mod orchestration;
pub mod legacy_config; // if you added it
pub mod apl;

use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{from_value, to_value};
use crate::orchestration::{run_single, run_many_with, SimulationResult, SimulationsResult};
use crate::legacy_config::{LegacyConfig, convert_legacy_to_simparams_and_players_data};
use crate::decisions::TeamDecider;
use crate::apl::create_team_decider_from_apls;
use console_error_panic_hook;
use log::{Level};

#[wasm_bindgen]
pub fn run_simulations(cfg_js: JsValue, iterations: i32) -> JsValue {
    //console_error_panic_hook::set_once();
    //console_log::init_with_level(Level::Debug).expect("error initializing log");
    let legacy: LegacyConfig = from_value(cfg_js).expect("bad config from JS");


    let (params, players_data) = convert_legacy_to_simparams_and_players_data(legacy);
    //let mut decider: TeamDecider = create_team_decider_from_apls(&players_data, &params.timing);    
    let make_decider = move || create_team_decider_from_apls(&players_data, &params.timing);

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



    let (params, players_data) = convert_legacy_to_simparams_and_players_data(legacy);
    let mut decider: TeamDecider = create_team_decider_from_apls(&players_data, &params.timing);    
    //let make_decider = move || create_decider_from_players(players_data);    

    let seed = 9u64; // or take from legacy.rng_seed.unwrap_or(42)
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
    log::debug!("Engine made it THIS!!!!! far. {:?}", result);

    to_value(&result).unwrap()
}