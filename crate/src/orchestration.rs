//! orchestration.rs — high-level driver and initialization
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Normal, Distribution};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::constants::{Buff, Constants, ConstantsConfig, ConsumeBuff, RaidBuff, WorldBuff, Racial, BossType};
use crate::state::{State};
use crate::decisions::Decider;


// ---- Parameters mirrored from Python inputs (trimmed first pass) ----
#[derive(Debug, Clone)]
pub struct Stats { pub spell_power: Vec<f64>, pub crit_chance: Vec<f64>, pub hit_chance: Vec<f64>, pub intellect: Vec<f64> }

fn has_idx<K: Eq + std::hash::Hash>(map: &std::collections::HashMap<K, Vec<usize>>, key: K, idx: usize) -> bool {
    map.get(&key).map(|v| v.contains(&idx)).unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct Buffs {
    // NEW: per-mage buff assignment by index
    pub consumes: HashMap<ConsumeBuff, Vec<usize>>,
    pub raid:     HashMap<RaidBuff,     Vec<usize>>,
    pub world:    HashMap<WorldBuff,    Vec<usize>>,
    pub boss: BossType,
    pub auras_mage_atiesh: Vec<usize>,
    pub auras_lock_atiesh: Vec<usize>,
    pub auras_boomkin: Vec<usize>,
    pub racial: Vec<Racial>,
    pub berserk: Vec<f64>,
}
#[derive(Debug, Clone, Copy)]
pub struct Timing { pub duration_mean: f64, pub duration_sigma: f64, pub initial_delay: f64, pub recast_delay: f64, pub reaction_time: f64}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub num_mages: usize,
    pub target: Vec<usize>,
    pub vary: Vec<usize>,
    pub do_stat_weights: bool,
    pub no_debuff_limit: bool,
    pub buff_assignments: HashMap<Buff, Vec<usize>>,
    pub pi_count: Vec<usize>,
    pub udc: Vec<usize>,
    pub t3_6p: Vec<usize>,
    pub nightfall: Vec<f64>,
    pub dragonling: f64,
    pub boss: BossType,
    pub coe: bool,
    pub name: Vec<String>,
}
impl Configuration {
    pub fn new() -> Self {
        let mut buff_assignments = HashMap::new();
        buff_assignments.insert(Buff::Sapp, vec![]);
        buff_assignments.insert(Buff::Toep, vec![]);
        buff_assignments.insert(Buff::Zhc, vec![]);
        buff_assignments.insert(Buff::Mqg, vec![]);
        Self {
            num_mages: 0,
            target: vec![],
            vary: vec![],
            do_stat_weights: true,
            no_debuff_limit: true,
            buff_assignments,
            pi_count: vec![],
            udc: vec![],
            t3_6p: vec![],
            nightfall: vec![],
            dragonling: f64::INFINITY,
            boss: BossType::None,
            coe: true,
            name: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimParams {
    pub stats: Stats,
    pub buffs: Buffs,
    pub timing: Timing,
    pub config: Configuration,
    pub consts_cfg: ConstantsConfig,
}

#[derive(Debug, Clone, Default)]
pub struct SimResult { pub total_dps: f64, pub ignite_dps: f64, pub player_dps: f64 }

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellResult {
    #[default]
    None,
    Hit,
    Crit,
    Miss,
    Pending,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum LogType {
    #[default]
    None,
    CastStart,
    CastSuccess,
    SpellImpact,
    IgniteTick,
    Debug,
    Wait,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub log_type: LogType,
    pub unit_name: String,
    pub text: String,
    pub t: f64,
    pub dps: f64,
    pub total_dps: f64,
    pub ignite_dps: f64,
    pub value: f64,
    pub value2: f64,
    pub spell_result: SpellResult,
    pub combustion: String,
    pub buffs: String,
    pub debuffs: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DamageAccumulator {
    pub time: f64,
    pub damage: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PlayerResult {
    pub dmg: u64,
    pub dps: f64,
    pub ninetieth: f64,
    pub name: String,
}

// Result from one run
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SimulationResult {
    pub iterations: i32,    
    pub t: f64,
    pub dmg: u64,
    pub dps: f64,
    pub ignite_dmg: u64,
    pub ignite_dps: f64,
    pub players: Vec<PlayerResult>,
    pub log: Vec<LogEntry>,
    pub damage_log: Vec<f64>,
}

// Result from multiple runs
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SimulationsResult {
    pub iterations: i32,
    pub dps: f64,
    pub min_dps: f64,
    pub max_dps: f64,
    pub ignite_dps: f64,
    pub players: Vec<PlayerResult>,
    pub histogram: HashMap<u32, u32>,
    pub damage_log: Vec<f64>,
    pub dps_sp: f64,
    pub dps_crit: f64,
    pub dps_hit: f64,
    pub dps_select: f64,
    pub dps90_sp: f64,
    pub dps90_crit: f64,
    pub dps90_hit: f64,
    pub dps90_select: f64,
}

// ---- Helpers ----
fn sample_duration(tim: &Timing, rng: &mut ChaCha8Rng) -> f64 {
    let normal = Normal::new(tim.duration_mean, tim.duration_sigma).unwrap();
    normal.sample(rng).max(tim.duration_mean - tim.duration_sigma)
}

fn first_action_offsets(num_mages: usize, initial_delay: f64, rng: &mut ChaCha8Rng) -> Vec<f64> {
    //let mut rng = Pcg64Mcg::seed_from_u64(9);
    let normal = Normal::new(0.0, initial_delay).unwrap();
    (0..num_mages).map(|_| normal.sample(rng).abs()).collect()
}

fn apply_buffs(stats: &mut Stats, buffs: &Buffs) {
 
    // 1) Intellect pipeline 
    for i in 0..stats.intellect.len() {
        let mut intel = stats.intellect[i];

        // base
        intel += buffs.racial[i].base_intellect();

        // flat adds
        if has_idx(&buffs.raid, RaidBuff::ArcaneIntellect, i) { intel += 31.0; }
        if has_idx(&buffs.raid, RaidBuff::ImprovedMark, i) { intel += 1.35 * 12.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::StormwindGiftOfFriendship, i) { intel += 30.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::InfallibleMind, i) { intel += 25.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::RunnTumTuberSurprise, i) { intel += 10.0; }
        if has_idx(&buffs.world, WorldBuff::SongflowerSerenade, i) { intel += 15.0; }

        // multiplicative
        let kings = if has_idx(&buffs.raid, RaidBuff::BlessingOfKings, i) { 1.10 } else { 1.0 };
        let soz   = if has_idx(&buffs.world, WorldBuff::SpiritOfZandalar, i) { 1.15 } else { 1.0 };
        let racial: f64 = buffs.racial[i].intellect_multiplier();
        intel = intel * kings * soz * racial;
        stats.intellect[i] = intel;
    }

    // 2) Spell power buffs
    for (i, sp) in stats.spell_power.iter_mut().enumerate() {

        if has_idx(&buffs.consumes, ConsumeBuff::GreaterArcaneElixir, i) { *sp += 35.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::ElixirOfGreaterFirepower, i) { *sp += 40.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::FlaskOfSupremePower, i) { *sp += 150.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::BlessedWizardOil, i) { *sp += 60.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::BrilliantWizardOil, i) { *sp += 36.0; }
        if has_idx(&buffs.consumes, ConsumeBuff::VeryBerryCream, i) { *sp += 23.0; }
        *sp += 33.0 * buffs.auras_lock_atiesh.get(i).copied().unwrap_or(0) as f64;
    }

    // 3) Crit chance buffs (uses UPDATED intellect)
    for (i, cc) in stats.crit_chance.iter_mut().enumerate() {
        *cc += 0.062; // base + talents 
        if has_idx(&buffs.consumes, ConsumeBuff::BrilliantWizardOil, i) { *cc += 0.01; }
        if has_idx(&buffs.world, WorldBuff::RallyingCryOfTheDragonslayer, i) { *cc += 0.10; }
        if has_idx(&buffs.world, WorldBuff::SongflowerSerenade, i) { *cc += 0.05; }
        if has_idx(&buffs.world, WorldBuff::DireMaulTribute, i) { *cc += 0.03; }
        *cc += stats.intellect[i] / 5950.0; // intellect → crit
        *cc += 0.60 * (buffs.boss == BossType::Loatheb) as i32 as f64;
        *cc += 0.02 * buffs.auras_mage_atiesh.get(i).copied().unwrap_or(0) as f64;
        *cc += 0.03 * buffs.auras_boomkin.get(i).copied().unwrap_or(0) as f64;
        if *cc > 1.0 { *cc = 1.0; }
    }

    // 4) Hit chance floor/cap
    for hc in &mut stats.hit_chance { *hc = (*hc + 0.89).min(0.99); }
}

fn init_state(p: &SimParams, rng: &mut ChaCha8Rng, idx: u64) -> State {
    use crate::constants as C;

    let num = p.config.num_mages;
    let mut st = State::new(sample_duration(&p.timing, rng), num);

    st.log_enabled = idx == 0;

    st.meta.cleaner_slots = p.config.udc.clone();
    st.meta.t3_6p_slots = p.config.t3_6p.clone();
    st.meta.target_slots = p.config.target.clone();
    st.meta.dmf_slots = p.buffs.world.get(&WorldBuff::SaygesDarkFortuneOfDamage).unwrap().clone().to_vec();
    st.meta.sr_slots = p.buffs.world.get(&WorldBuff::SoulRevival).unwrap().clone().to_vec();
    st.meta.ts_slots = p.buffs.world.get(&WorldBuff::TracesOfSilithyst).unwrap().clone().to_vec();
    st.meta.pi_count = p.config.pi_count.clone();
    st.meta.no_debuff_limit = p.config.no_debuff_limit.clone();
    st.meta.vulnerability = if p.buffs.boss == BossType::Thaddius { 1.0 + C::THADDIUS_BUFF } else { 1.0 };
    st.meta.nightfall_period = p.config.nightfall.clone();
    st.meta.coe = if p.config.coe { C::COE_MULTIPLIER } else { 1.0 };
    st.meta.name = p.config.name.clone();
    st.meta.berserk_slots = p.buffs.berserk.clone();
    st.boss.nightfall = p.config.nightfall.clone(); // start the swing timers
    st.boss.dragonling_start = p.config.dragonling;

    // Per-lane stats
    let offsets = first_action_offsets(num, p.timing.initial_delay, rng);
    let overall_delay = offsets.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    for i in 0..num {
        let l = &mut st.lanes[i];
        l.cast_timer = offsets[i];
        l.hit_chance = p.stats.hit_chance[i];
        l.crit_chance = p.stats.crit_chance[i];
        l.spell_power = p.stats.spell_power[i];
        // Buff availability: PI, trinkets that are assigned get 0 cooldown to open
        for cooldown in l.buff_cooldown.iter_mut() { *cooldown = f64::INFINITY; }
        // Others could come from config similarly
    }

    for lane_idx in 0..st.lanes.len() {
        for (buff, indices) in &p.config.buff_assignments {
            if indices.contains(&lane_idx) {
                if let Some(lane) = st.lanes.get_mut(lane_idx) {
                    lane.buff_cooldown[*buff as usize] = 0.0;
                }
            }
        }
        let pis: usize = st.meta.pi_count[lane_idx];
        for slot in 0..pis.min(C::MAX_PI) {
            st.lanes[lane_idx].pi_cooldown[slot] = 0.0;
        }
        if st.meta.berserk_slots[lane_idx] > 0.0 { st.lanes[lane_idx].berserk_cooldown = 0.0 }
    }
    st.subtime(overall_delay); // set delay after all time initializations

    st
}

/// Print SP / Hit / Crit / Int for each mage, plus which buffs are currently ready (cooldown <= 0).
/// Call this right after `init_state(...)` inside `run_single`.
pub fn display_party_stats(st: &State, intellect: Option<&[f64]>) {
    // If you add/remove buffs, update this list to match Buff order/variants.
    let known_buffs: &[(Buff, &str)] = &[
        (Buff::Sapp, "sapp"),
        (Buff::Toep, "toep"),
        (Buff::Zhc,  "zhc"),
        (Buff::Mqg,  "mqg"),
    ];

    log::debug!("\n=== Player Stats ===");
    for (i, lane) in st.lanes.iter().enumerate() {
        // gather ready buffs
        let mut ready: Vec<&str> = Vec::new();
        for (b, label) in known_buffs {
            let idx = *b as usize;
            if lane.buff_cooldown.get(idx).map(|&cd| cd <= 0.0).unwrap_or(false) {
                ready.push(*label);
            }
        }
        let ready_str = if ready.is_empty() { "-".to_string() } else { ready.join(",") };

        // intellect if provided; otherwise show "-"
        let int_str = intellect
            .and_then(|ints| ints.get(i).copied())
            .map(|v| format!("{:.0}", v))
            .unwrap_or_else(|| "-".to_string());

        log::debug!(
            "Mage {:>2}: SP={:>4.0}  Hit={:>5.2}%  Crit={:>5.2}%  Int={} Ready=[{}]",
            i,
            lane.spell_power,
            100.0 * lane.hit_chance,
            100.0 * lane.crit_chance,
            int_str,
            ready_str
        );
    }
}


fn create_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

pub fn run_single<D: Decider>(params: &SimParams, decider: &mut D, seed: u64, idx: u64) -> SimulationResult {

    let mut rng = if idx == 0 {
        // Deterministic for reproducibility of the first run
        create_rng(seed)
    } else {
        // Non-deterministic for parallel workers
        ChaCha8Rng::from_entropy()
    };

    // Build constants and bake stats
    let k = Constants::new(&params.consts_cfg);
    let mut baked_params = params.clone();

    apply_buffs(&mut baked_params.stats, &params.buffs);

    // Init state
    let mut st = init_state(&baked_params, &mut rng, idx);

    if st.log_enabled {
    // show effective stats & ready buffs
        display_party_stats(&st, Some(&baked_params.stats.intellect));
    }

    while st.in_progress() {
        
        if let Some((lane, action, delay_sigma)) = decider.next_action(&st) {
            // sample continuing delay
            let normal = Normal::new(0.0, delay_sigma).unwrap();
            let continuing_delay: f64 = normal.sample(&mut rng).abs();
            
            st.start_action(lane, action, continuing_delay, &k);
        }

        // step one event
        while !st.decision_gate() && st.in_progress() {
            st.step_one(&k, &mut rng);
        }
    }

    // Aggregate DPS
    let dur = st.global.duration.max(1e-9);
    let mut players = Vec::<PlayerResult>::new();
    for i in 0..st.lanes.len() {
        let dmg = st.lanes[i].damage;
        let total_dmg = dmg + st.totals.ignite_damage / (st.lanes.len() as f64);
        //log::debug!("{:3} player {} amount {:4.}", idx, i, (st.lanes.len() as f64) * dmg/dur);
        players.push(PlayerResult {
            name: params.config.name[i].clone(),
            dmg: dmg as u64,
            dps: total_dmg /dur,
            ninetieth: 0.0,
        });
    }

    // build damage over time
    const DELTA_T: f64 = 0.25;
    let num_intervals = if !st.log_enabled { ((params.timing.duration_mean - params.timing.duration_sigma) / DELTA_T).ceil() as usize } else { 0 };
    let mut total_damage = Vec::with_capacity(num_intervals);
    if !st.log_enabled {
        let mut cumulative = 0.0;
        let mut log_index = 0;
        total_damage.push(0.0);  // starting at t= 0.0
        for tdx in 0..num_intervals {
            let time_point = (tdx + 1) as f64 * DELTA_T;
            // Add all damage that occurred in this interval
            while log_index < st.damage_log.len() && st.damage_log[log_index].time <= time_point {
                cumulative += st.damage_log[log_index].damage;
                log_index += 1;
            }
            total_damage.push(cumulative / time_point);
        }
    }

    let result = SimulationResult {
        iterations: 1,
        t: dur,
        dmg: (st.totals.total_damage + st.totals.ignite_damage) as u64,
        dps: (st.totals.total_damage + st.totals.ignite_damage) /dur,
        ignite_dmg: st.totals.ignite_damage as u64,
        ignite_dps: st.totals.ignite_damage /dur,
        players: players.clone(),
        log: st.log.clone(),
        damage_log: total_damage.clone(),
    };

    result.clone()
}

pub fn run_many_with<D, F>(params: &SimParams, make_decider: F, iterations: i32, seed: u64) -> SimulationsResult
where
    D: Decider,
    F: Fn() -> D,
{
    let mut result: SimulationsResult = SimulationsResult { iterations, ..Default::default() };
    const BIN_SIZE: f64 = 50.0;
    let mut dps_values = vec![Vec::with_capacity(iterations as usize); params.config.num_mages];

    for idx in 1..=iterations {
        // Fresh decider for each iteration
        let mut decider = make_decider();

        // If you want a per-iter seed, keep passing i as u64 like before
        let sim_result = run_single(params, &mut decider, seed, idx as u64);

        for jdx in 0..params.config.num_mages {
            dps_values[jdx].push(sim_result.players[jdx].dps);
        }

        result.dps += sim_result.dps;
        result.ignite_dps += sim_result.ignite_dps as f64;

        if idx == 1 || sim_result.dps < result.min_dps {
            result.min_dps = sim_result.dps;
        }
        if idx == 1 || sim_result.dps > result.max_dps {
            result.max_dps = sim_result.dps;
        }

        let bin = ((sim_result.dps / BIN_SIZE).floor() * BIN_SIZE) as u32;
        if let Some(num) = result.histogram.get_mut(&bin) {
            *num += 1;
        } else {
            result.histogram.insert(bin, 1);
        }

        if idx == 1 {
            result.players.clone_from(&sim_result.players);
        } else {
            for (jdx, pr) in sim_result.players.iter().enumerate() {
                result.players[jdx].dps += pr.dps;
            }
        }
        if idx == 1 {
            result.damage_log = sim_result.damage_log.clone();
        } else {
            for (x, y) in result.damage_log.iter_mut().zip(sim_result.damage_log.iter()) {
                *x += y;
            }
        }
    }

    // Calculate 90th percentile
    for jdx in 0..params.config.num_mages {
        dps_values[jdx].sort_by(|a, b| a.partial_cmp(b).unwrap());
        let percentile_90_index = ((iterations as f64) * 0.9).ceil() as usize - 1;
        result.players[jdx].ninetieth = dps_values[jdx][percentile_90_index.min(iterations as usize - 1)];
    }

    result.dps /= iterations as f64;
    result.ignite_dps /= iterations as f64;
    for jdx in 0..result.players.len() {
        result.players[jdx].dps /= iterations as f64;
    }
    for d in result.damage_log.iter_mut() { *d /= iterations as f64; }

    result  
}
