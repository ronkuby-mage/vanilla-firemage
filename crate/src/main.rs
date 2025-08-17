// in src/main.rs

use std::collections::HashMap;
use crate::constants::{Action, ConstantsConfig, Buff, ConsumeBuff, RaidBuff, WorldBuff};
use crate::decisions::{TeamDecider, MageDecider, ScriptedMage};
use crate::orchestration::{run_many_with, Configuration, SimParams, Stats, Buffs, Timing};

fn main() {
    let num_mages = 3usize;

    // Base stats for each mage
    let stats = Stats {
        spell_power: vec![600.0; num_mages],
        crit_chance: vec![0.18; num_mages], // additional to base 0.062 added later
        hit_chance: vec![0.08; num_mages],  // additional to base 0.89 capped at 0.99
        intellect: vec![300.0; num_mages],
    };

    let mut consumes = HashMap::new();
    consumes.insert(ConsumeBuff::GreaterArcaneElixir, vec![0,1,2]);
    consumes.insert(ConsumeBuff::ElixirOfGreaterFirepower, vec![0,1,2]);
    consumes.insert(ConsumeBuff::FlaskOfSupremePower, vec![]);
    consumes.insert(ConsumeBuff::BlessedWizardOil, vec![]);
    consumes.insert(ConsumeBuff::BrilliantWizardOil, vec![]);
    consumes.insert(ConsumeBuff::VeryBerryCream, vec![]);
    consumes.insert(ConsumeBuff::StormwindGiftOfFriendship, vec![]);
    consumes.insert(ConsumeBuff::InfallibleMind, vec![]);
    consumes.insert(ConsumeBuff::RunnTumTuberSurprise, vec![]);

    let mut raid = HashMap::new();
    raid.insert(RaidBuff::ArcaneIntellect, (0..num_mages).collect());
    raid.insert(RaidBuff::ImprovedMark, vec![]);
    raid.insert(RaidBuff::BlessingOfKings, vec![]);

    let mut world = HashMap::new();
    world.insert(WorldBuff::RallyingCryOfTheDragonslayer, (0..num_mages).collect());
    world.insert(WorldBuff::SongflowerSerenade, vec![]);
    world.insert(WorldBuff::DireMaulTribute, vec![]);
    world.insert(WorldBuff::SpiritOfZandalar, vec![]);
    world.insert(WorldBuff::SaygesDarkFortuneOfDamage, (0..num_mages).collect());

    // Buffs/consumes (trimmed set — mirrors a subset of ArrayGenerator adjustments)
    let buffs = Buffs {
        consumes, raid, world,
        boss: "",
        auras_mage_atiesh: vec![0.0; num_mages],
        auras_lock_atiesh: vec![0.0; num_mages],
        auras_boomkin: vec![0.0; num_mages],
        racial: vec!["human"; num_mages],
    };

    let timing = Timing {
        duration_mean: 45.0,
        duration_sigma: 0.0,
        initial_delay: 0.2,
        recast_delay: 0.05,
    };

    let mut buff_assignments = HashMap::new();
    buff_assignments.insert(Buff::Sapp, vec![]);
    buff_assignments.insert(Buff::Toep, vec![]);
    buff_assignments.insert(Buff::Zhc, vec![]);
    buff_assignments.insert(Buff::Mqg, vec![0, 1, 2]);
    buff_assignments.insert(Buff::PowerInfusion, vec![]);

    let config = Configuration {
        num_mages,
        target: vec![0, 1, 2], // treat all as target for player_damage
        buff_assignments,
        udc: vec![0, 2],
        nightfall: vec![1.77, 3.55],
        dragonling: 20.0,
    };

    let consts_cfg = ConstantsConfig { ..Default::default() };

    let params = SimParams { stats, buffs, timing, config, consts_cfg };

    // Build per-mage rotations:
    let sequences: Vec<Vec<Action>> = vec![
        vec![Action::Scorch, Action::Scorch, Action::Mqg], // mage 0
        vec![Action::Scorch, Action::Scorch, Action::Mqg], // mage 1
        vec![Action::Scorch, Action::Scorch, Action::Mqg], // mage 2
    ];
    let defaults  = vec![Action::Fireball, Action::Fireball, Action::Fireball];
    let init_rct  = vec![0.20, 0.20, 0.20];
    let cont_rct  = vec![0.05, 0.05, 0.05];

    let make_decider = || {
        let mages: Vec<Box<dyn MageDecider>> = (0..params.config.num_mages)
            .map(|i| {
                Box::new(ScriptedMage::new(
                    sequences[i].clone(),
                    defaults[i],
                    init_rct[i],
                    cont_rct[i],
                )) as Box<dyn MageDecider>
            })
            .collect();
        TeamDecider::new(mages)
    };

    let sims = 50000;
    let seed = 42;
    let results = run_many_with::<TeamDecider, _>(&params, make_decider, sims, seed);


    // Per-mage / per-target summary (as we discussed earlier)
    let m = params.config.num_mages as f64;
    let target_count = params.config.target.len() as f64;

    let (mut total, mut ignite, mut player) = (0.0, 0.0, 0.0);
    for r in &results { total += r.total_dps; ignite += r.ignite_dps; player += r.player_dps; }
    let n = results.len() as f64;

    println!("Ran {} sims:", results.len());
    println!("  mean total dps per mage:    {:.1}", total / (n * m));
    println!("  mean ignite dps per mage:   {:.1}", ignite / (n * m));
    println!("  mean player dps per target: {:.1}", player / (n * target_count) + ignite / (n * m));
}
