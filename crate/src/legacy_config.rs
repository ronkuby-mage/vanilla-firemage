use std::collections::HashMap;
use crate::constants::{
    ConstantsConfig, Racial, Buff, BossType,
    ConsumeBuff as Cn, RaidBuff as Rd, WorldBuff as Wb,
};
use crate::orchestration::Buffs; // <- your Buffs struct
use crate::orchestration::{SimParams, Stats, Timing, Configuration};
use strum::IntoEnumIterator;
use serde::Deserialize;
use serde_json::Value;

// ---- JS -> Rust legacy shapes (from App.vue / simConfig) ----
#[derive(Debug, Deserialize)]
pub struct LegacyConfig {
    pub duration: Option<f64>,
    pub duration_variance: Option<f64>,
    pub curse_of_elements: Option<bool>,
    pub arcanite_dragonling: Option<Value>,
    pub nightfall1: Option<Value>,
    pub nightfall2: Option<Value>,
    pub nightfall3: Option<Value>,
    pub reaction_time: Option<f64>,
    pub player_delay: Option<f64>,
    pub boss: Option<String>,
    pub players: Vec<LegacyPlayer>,
    // Optional RNG seed if UI sends it; fallback to host seed
    pub rng_seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyBuffs {
    // UI toggles (all optional booleans)
    pub arcane_intellect: Option<bool>,
    pub imp_mark_of_the_wild: Option<bool>,
    pub blessing_of_kings: Option<bool>,

    pub rallying_cry: Option<bool>,
    pub songflower: Option<bool>,
    pub dire_maul_tribute: Option<bool>,
    pub spirit_of_zandalar: Option<bool>,

    pub flask_of_supreme_power: Option<bool>,
    pub infallible_mind: Option<bool>, // +25 Int (consume)
    pub gift_of_stormwind: Option<bool>,
    pub elixir_greater_arcane: Option<bool>,
    pub elixir_greater_firepower: Option<bool>, // +40 fire sp
    pub brilliant_wizard_oil: Option<bool>,    // "brilliant", "blessed", "none"
    pub blessed_wizard_oil: Option<bool>,    // "brilliant", "blessed", "none"
    pub very_berry_cream: Option<bool>,          // 
    pub runn_tum_tuber: Option<bool>,          // "runn_tum_tuber", etc.

    pub moonkin_aura: Option<bool>,
    pub atiesh_mage: Option<usize>,
    pub atiesh_warlock: Option<usize>,

    // DMF and other world-ish toggles sometimes appear
    pub dmf_dmg: Option<bool>,
    pub soul_revival: Option<bool>,
    pub traces_of_silithyst: Option<bool>,
   
}

#[derive(Debug, Deserialize)]
pub struct LegacyPlayer {
    pub name: Option<String>,
    pub race: Option<String>,   // e.g., "Gnome", "Undead", etc.
    pub talents: Vec<u8>, // not used in backend yet

    pub stats: LegacyStats,
    pub buffs: LegacyBuffs,
    pub has_pi: Option<bool>,
    pub is_target: Option<bool>,
    pub items: LegacyItems,

    // rotation/APL placeholder (if you plan to parse later)
    pub apl: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct LegacyStats {
    pub int: f64,
    pub sp: f64,
    pub crit: f64, // percent in UI (e.g., 10 for 10%)
    pub hit: f64,  // percent in UI (e.g., 9 for 9%)
}

#[derive(Debug, Deserialize)]
pub struct LegacyItems {
    pub sapp: Option<bool>,
    pub toep: Option<bool>,
    pub zhc: Option<bool>,
    pub mqg: Option<bool>,
    pub udc: Option<bool>,
}


pub fn racial_from_str<S: AsRef<str>>(s: S) -> Racial {
    let t = s.as_ref().trim().to_ascii_lowercase();
    match t.as_str() {
        // Alliance
        "human" | "hum" => Racial::Human,
        "gnome" | "gno" => Racial::Gnome,
        "night elf" | "night-elf" | "nightelf" | "ne" => Racial::NightElf,
        "dwarf" | "dwa" => Racial::Dwarf,
        // Horde
        "orc" => Racial::Orc,
        "troll" => Racial::Troll,
        "tauren" => Racial::Tauren,
        "undead" | "forsaken" | "ud" => Racial::Undead,
        // Fallback
        _ => Racial::Other,
    }
}
fn parse_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

// Rename your existing function to this:
fn convert_legacy_to_simparams_internal(cfg: LegacyConfig, timing: Timing) -> SimParams {
    log::debug!("LegacyConfig (Debug): {:#?}", cfg);
    let nm = cfg.players.len();

    // --- Stats vectors (per mage) ---
    let mut stats = Stats {
        spell_power: vec![0.0; nm],
        crit_chance: vec![0.0; nm],
        hit_chance:  vec![0.0; nm],
        intellect:   vec![0.0; nm],
    };
    let mut racials: Vec<Racial> = vec![Racial::Other; nm];

    for (i, p) in cfg.players.iter().enumerate() {
        stats.spell_power[i] = p.stats.sp;
        stats.crit_chance[i] = p.stats.crit / 100.0; // UI sends percent
        stats.hit_chance[i]  = p.stats.hit / 100.0;
        stats.intellect[i]   = p.stats.int;

        // Map race string → Racial enum
        racials[i] = p.race.as_deref().map(racial_from_str).unwrap_or(Racial::Other);
    }

    // --- Per-mage buff assignments (index lists) ---
    let mut consumes: HashMap<Cn, Vec<usize>> = HashMap::new();
    let mut raid:     HashMap<Rd, Vec<usize>> = HashMap::new();
    let mut world:    HashMap<Wb, Vec<usize>> = HashMap::new();

    // ensure every key exists
    for k in Cn::iter() { consumes.entry(k).or_default(); }
    for k in Rd::iter() { raid.entry(k).or_default(); }
    for k in Wb::iter() { world.entry(k).or_default(); }

    // helper to push an index
    fn push_idx<K: std::hash::Hash + Eq>(map: &mut HashMap<K, Vec<usize>>, k: K, idx: usize) {
        map.entry(k).or_default().push(idx);
    }

    let mut auras_mage_atiesh: Vec<usize> = vec![0; nm];
    let mut auras_lock_atiesh: Vec<usize> = vec![0; nm];
    let mut auras_boomkin: Vec<usize> = vec![0; nm];
    for (i, p) in cfg.players.iter().enumerate() {
        // ---- RAID ----
        if p.buffs.arcane_intellect.unwrap_or(false)      { push_idx(&mut raid, Rd::ArcaneIntellect, i); }
        if p.buffs.imp_mark_of_the_wild.unwrap_or(false)  { push_idx(&mut raid, Rd::ImprovedMark, i); }
        if p.buffs.blessing_of_kings.unwrap_or(false)     { push_idx(&mut raid, Rd::BlessingOfKings, i); }
        auras_mage_atiesh[i] = p.buffs.atiesh_mage.unwrap_or(0);
        auras_lock_atiesh[i] = p.buffs.atiesh_warlock.unwrap_or(0);
        if p.buffs.moonkin_aura.unwrap_or(false)          { auras_boomkin[i] = 1; }

        // ---- WORLD ----
        if p.buffs.rallying_cry.unwrap_or(false)          { push_idx(&mut world, Wb::RallyingCryOfTheDragonslayer, i); }
        if p.buffs.songflower.unwrap_or(false)            { push_idx(&mut world, Wb::SongflowerSerenade, i); }
        if p.buffs.dire_maul_tribute.unwrap_or(false)     { push_idx(&mut world, Wb::DireMaulTribute, i); }
        if p.buffs.spirit_of_zandalar.unwrap_or(false)    { push_idx(&mut world, Wb::SpiritOfZandalar, i); }
        // DMF damage (Sayge) if present:
        if p.buffs.dmf_dmg.unwrap_or(false)               { push_idx(&mut world, Wb::SaygesDarkFortuneOfDamage, i); }
        if p.buffs.soul_revival.unwrap_or(false)          { push_idx(&mut world, Wb::SoulRevival, i); }
        if p.buffs.traces_of_silithyst.unwrap_or(false)          { push_idx(&mut world, Wb::TracesOfSilithyst, i); }

        // ---- CONSUMES ----
        // Intellect consumes:
        if p.buffs.infallible_mind.unwrap_or(false)       { push_idx(&mut consumes, Cn::InfallibleMind, i); }
        if p.buffs.gift_of_stormwind.unwrap_or(false)       { push_idx(&mut consumes, Cn::StormwindGiftOfFriendship, i); }
        if p.buffs.very_berry_cream.unwrap_or(false)        { push_idx(&mut consumes, Cn::VeryBerryCream, i); }
        if p.buffs.runn_tum_tuber.unwrap_or(false)        { push_idx(&mut consumes, Cn::RunnTumTuberSurprise, i); }
        // Spell power elixirs:
        if p.buffs.flask_of_supreme_power.unwrap_or(false) { push_idx(&mut consumes, Cn::FlaskOfSupremePower, i); }
        if p.buffs.elixir_greater_firepower.unwrap_or(false) { push_idx(&mut consumes, Cn::ElixirOfGreaterFirepower, i); }
        if p.buffs.elixir_greater_arcane.unwrap_or(false) { push_idx(&mut consumes, Cn::GreaterArcaneElixir, i); }
        // Wizard oils:
        if p.buffs.brilliant_wizard_oil.unwrap_or(false) { push_idx(&mut consumes, Cn::BrilliantWizardOil, i); }
        if p.buffs.blessed_wizard_oil.unwrap_or(false) { push_idx(&mut consumes, Cn::BlessedWizardOil, i); }

        // Atiesh aura flags (if you model them as per-mage scalar auras, handle elsewhere)
        // p.atiesh_mage / p.atiesh_warlock could feed your auras_mage_atiesh / auras_lock_atiesh
    }
    // log::debug!("World buffs assignments:");
    // for (buff, lanes) in world.clone().into_iter() {
    //     debug!("{:?} -> {:?}", buff, lanes);
    // }
    // log::debug!("Done");

    let boss: BossType = match cfg.boss.as_deref() {
        Some("Loatheb") => BossType::Loatheb,
        Some("Thaddius") => BossType::Thaddius,
        _ => BossType::None,
    };
    let buffs = Buffs {
        // NEW fields we introduced earlier:
        consumes: consumes,
        raid: raid,
        world: world,
        boss: boss,
        auras_mage_atiesh: auras_mage_atiesh,
        auras_lock_atiesh: auras_lock_atiesh,
        auras_boomkin: auras_boomkin,
        racial: racials,
    };
    
    let mut buff_assignments: HashMap<Buff, Vec<usize>> = HashMap::new();
    for k in Buff::iter() { buff_assignments.entry(k).or_default(); }

    let mut target = vec![];
    let mut udc = vec![];
    for (i, p) in cfg.players.iter().enumerate() {
        if p.items.sapp.unwrap_or(false)      { push_idx(&mut buff_assignments, Buff::Sapp, i); }
        if p.items.toep.unwrap_or(false)      { push_idx(&mut buff_assignments, Buff::Toep, i); }
        if p.items.zhc.unwrap_or(false)      { push_idx(&mut buff_assignments, Buff::Zhc, i); }
        if p.items.mqg.unwrap_or(false)      { push_idx(&mut buff_assignments, Buff::Mqg, i); }
        if p.items.udc.unwrap_or(false) { udc.push(i); }
        if p.is_target.unwrap_or(false) { target.push(i); }
        if p.has_pi.unwrap_or(false)      { push_idx(&mut buff_assignments, Buff::PowerInfusion, i); }
    }
    // log::debug!("Buffs assignments:");
    // for (buff, lanes) in buff_assignments.clone().into_iter() {
    //     debug!("{:?} -> {:?}", buff, lanes);
    // }
    // log::debug!("Done");
    let dragonling: f64 = cfg.arcanite_dragonling.as_ref().and_then(parse_f64).unwrap_or(f64::INFINITY);
    let nightfall: Vec<f64> = [cfg.nightfall1, cfg.nightfall2, cfg.nightfall3].into_iter().filter_map(|opt| opt.as_ref().and_then(parse_f64)).map(|f| f.max(1.0)).collect();
    let coe:bool = if cfg.curse_of_elements.unwrap_or(false) { true } else {false};

    let config = Configuration {
        num_mages: nm,
        target: target,
        buff_assignments: buff_assignments,
        udc: udc,
        nightfall: nightfall,
        dragonling: dragonling,
        boss: boss,
        coe: coe,
    };
    // Constants config: carry defaults unless you expose knobs in UI
    let consts_cfg = ConstantsConfig::default();

    SimParams { stats, buffs, timing, config, consts_cfg }
}

fn extract_players_apls(players: &Vec<LegacyPlayer>) -> Vec<Option<serde_json::Value>> {
    players.iter().map(|player| player.apl.clone()).collect()
}

pub fn convert_legacy_to_simparams_and_players_data(cfg: LegacyConfig) -> (SimParams, Vec<Option<serde_json::Value>>) {
    // ... existing logic to create params ...
    // Extract/convert players data before it gets consumed
    let players_data = extract_players_apls(&cfg.players);

        // --- Timing ---
    let timing = Timing {
        duration_mean: cfg.duration.unwrap_or(0.0),                   // keep your default or derive from UI
        duration_sigma: cfg.duration_variance.unwrap_or(0.0),
        recast_delay: cfg.reaction_time.unwrap_or(0.0),
        initial_delay: cfg.player_delay.unwrap_or(0.0),
    };
    let sim_params = convert_legacy_to_simparams_internal(cfg, timing);

    (sim_params, players_data)
}



