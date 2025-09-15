//! constants.rs — first pass port from Python `constants.py`
//!
//! Notes:
//! - Keeps spell ordering: [Scorch, Pyroblast, Fireball, FireBlast, Frostbolt]
//! - Replaces string/idx maps with enums where possible.
//! - Values that depended on runtime toggles in Python (ranks, talents, incinerate, etc.)
//!   are computed in `Constants::new(cfg)`.
//! - This file intentionally avoids any sim state. That lives in `state.rs`.

use std::fmt;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Talent {
    // Arcane Tree (0-15)
    ArcaneSubtlety = 0,
    ArcaneFocus = 1,
    ImprovedArcaneMissiles = 2,
    WandSpecialization = 3,
    MagicAbsorbtion = 4,
    ArcaneConcentration = 5,
    MagicAttunement = 6,
    ImprovedArcaneExplosion = 7,
    ArcaneResiliance = 8,
    ImprovedManaShield = 9,
    ImprovedCounterspell = 10,
    ArcaneMeditation = 11,
    PresenceOfMind = 12,
    ArcaneMind = 13,
    ArcaneInstability = 14,
    ArcanePower = 15,
    
    // Fire Tree (16-31)
    ImprovedFireball = 16,
    Impact = 17,
    Ignite = 18,
    FlameThrowing = 19,
    ImprovedFireBlast = 20,
    Incinerate = 21,
    ImprovedFlamestrike = 22,
    Pryoblast = 23,
    BurningSoul = 24,
    ImprovedScorch = 25,
    ImprovedFireWard = 26,
    MasterOfElements = 27,
    CriticalMass = 28,
    BlastWave = 29,
    FirePower = 30,
    Combustion = 31,
    
    // Frost Tree (32-48)
    FrostWarding = 32,
    ImprovedFrostbolt = 33,
    ElementalPrecision = 34,
    IceShards = 35,
    Frostbite = 36,
    ImprovedFrostNova = 37,
    Permafrost = 38,
    PiercingIce = 39,
    ColdSnap = 40,
    ImprovedBlizard = 41,
    ArcticReach = 42,
    FrostChanneling = 43,
    Shatter = 44,
    IceBlock = 45,
    ImprovedConeOfCold = 46,
    WintersChill = 47,
    IceBarrier = 48,
}

#[derive(Debug, Clone)]
pub struct TalentPoints {
    points: [u8; 49],
}

impl TalentPoints {
    pub fn new() -> Self {
        let talents_data = [
            2, 3, 0, 0, 0, 5, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            5, 0, 5, 2, 1, 2, 2, 1, 2, 3, 0, 3, 3, 1, 5, 1,
            0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
        
        Self { points: talents_data }
    }
    
    pub fn get(&self, talent: Talent) -> u8 {
        self.points[talent as usize]
    }
    
    pub fn set(&mut self, talent: Talent, points: u8) {
        self.points[talent as usize] = points;
    }
}

impl TalentPoints {
    pub fn from_vec(talents_data: Vec<u8>) -> Result<Self, String> {
        if talents_data.len() != 49 {
            return Err(format!("Expected 49 talents, got {}", talents_data.len()));
        }
        
        let points: [u8; 49] = talents_data.try_into()
            .map_err(|_| "Failed to convert Vec to array".to_string())?;
        
        Ok(Self { points })
    }
}

#[derive(Debug, Clone, Default)]
pub struct TeamTalentPoints {
    pub team: Vec<TalentPoints>,
}

impl TeamTalentPoints {
    // Create team with n mages (initialized with empty talent points)
    pub fn new(n: usize) -> Self {
        Self {
            team: vec![TalentPoints::new(); n],
        }
    }
    
    // Copy Vec<u8> to the ith mage
    pub fn set_mage_talents(&mut self, i: usize, talents_data: Vec<u8>) -> Result<(), String> {
        if i >= self.team.len() {
            return Err(format!("Index {} out of bounds", i));
        }
        
        self.team[i] = TalentPoints::from_vec(talents_data)?;
        Ok(())
    }
    
    // Copy the ith mage's talents to a TalentPoints
    pub fn get_mage_talents(&self, i: usize) -> Option<TalentPoints> {
        self.team.get(i).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spell { Scorch = 0, Pyroblast = 1, Fireball = 2, FireBlast = 3, Frostbolt = 4, PyroDot = 5 }

impl fmt::Display for Spell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Spell::Fireball => write!(f, "Fireball"),
            Spell::Scorch => write!(f, "Scorch"),
            Spell::Pyroblast => write!(f, "Pyroblast"),
            Spell::FireBlast => write!(f, "Fire Blast"),
            Spell::Frostbolt => write!(f, "Frostbolt"),
            Spell::PyroDot => write!(f, "Pyroblast DoT"),
        }
    }
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    // Castable spells
    Scorch,
    Pyroblast,
    Fireball,
    FireBlast,
    Frostbolt,
    // Pseudo-cast for pushing GCD-only waits
    #[default]
    Gcd,
    // Instants / external sources (non-GCD spells in the Python model)
    Combustion,
    Sapp,
    Toep,
    Zhc,
    Mqg,
    PowerInfusion,
    Berserking,
    ArcanePower,
    PresenceOfMind,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Fireball => write!(f, "Fireball"),
            Action::Scorch => write!(f, "Scorch"),
            Action::Pyroblast => write!(f, "Pyroblast"),
            Action::FireBlast => write!(f, "Fire Blast"),
            Action::Frostbolt => write!(f, "Frostbolt"),
            Action::Gcd => write!(f, "GCD"),
            Action::Combustion => write!(f, "Combustion"),
            Action::Sapp => write!(f, "Sapp"),
            Action::Toep => write!(f, "ToEP"),
            Action::Zhc => write!(f, "ZHC"),
            Action::Mqg => write!(f, "MQG"),
            Action::PowerInfusion => write!(f, "Power Infusion"),
            Action::Berserking => write!(f, "Berserking"),
            Action::ArcanePower => write!(f, "Arcane Power"),
            Action::PresenceOfMind => write!(f, "Presence of Mind"),
        }
    }
}

// constants.rs
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum ConsumeBuff {
    GreaterArcaneElixir,
    ElixirOfGreaterFirepower,
    ElixirOfFrostPower,
    FlaskOfSupremePower,
    BlessedWizardOil,
    BrilliantWizardOil,
    StormwindGiftOfFriendship,
    InfallibleMind,
    VeryBerryCream,
    RunnTumTuberSurprise,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum RaidBuff {
    ArcaneIntellect,
    ImprovedMark,
    BlessingOfKings,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum WorldBuff {
    RallyingCryOfTheDragonslayer,
    SongflowerSerenade,
    DireMaulTribute,
    SpiritOfZandalar,
    SaygesDarkFortuneOfDamage,
    SoulRevival,
    TracesOfSilithyst,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BossType {
    Loatheb,
    Thaddius,
    /// Fallback for anything we don’t recognize
    None,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Racial {
    Human,
    Gnome,
    NightElf,
    Dwarf,
    Orc,
    Troll,
    Tauren,
    Undead,
    /// Fallback for anything we don’t recognize
    Other,
}

impl Default for Racial {
    fn default() -> Self { Racial::Other }
}

impl Racial {
    /// Intellect multiplier used in classic sims (only Gnome gets +5% Int)
    pub fn intellect_multiplier(self) -> f64 {
        match self {
            Racial::Gnome => 1.05,
            _ => 1.0,
        }
    }
    pub fn base_intellect(self) -> f64 {
        match self {
            Racial::Gnome => 132.0,
            Racial::Human => 125.0,
            Racial::Troll => 121.0,
            Racial::Undead => 123.0,
            _ => 0.0,
        }
    }
}

impl Action {
    #[inline]
    pub fn is_instant(self) -> bool { matches!(self, Action::Combustion | Action::Sapp | Action::Toep | Action::Zhc | Action::Mqg | Action::PowerInfusion | Action::Berserking | Action::Gcd) }
}

impl Action {
    pub fn triggers_gcd(&self) -> bool {
        use Action::*;
        matches!(self, Scorch | Pyroblast | Fireball | FireBlast | Frostbolt)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Buff { Sapp = 0, Toep = 1, Zhc = 2, Mqg = 3 }

pub const LOG: bool = true;

pub const NUM_SPELLS: usize = 6;          // Scorch, Pyro, Fireball, FireBlast, Frostbolt, PyroDot
pub const NUM_ACTIONS: usize = 12;         // includes GCD + instants

// --- Global mechanical constants (mostly invariant during a run) ---
pub const GLOBAL_COOLDOWN: f64 = 1.5;

pub const IGNITE_TIME: f64 = 4.0;
pub const IGNITE_TICK: f64 = 2.0;
pub const IGNITE_STACK: u8 = 5;

pub const SCORCH_TIME: f64 = 30.0;
pub const SCORCH_STACK: u8 = 5;

pub const WC_TIME: f64 = 15.0;
pub const WC_STACK: u8 = 5;
pub const PER_WC: f64 = 0.02;

pub const COE_MULTIPLIER: f64 = 1.10;      // Curse of Elements multiplier
pub const SCORCH_MULTIPLIER: f64 = 0.03;   // +3% fire vuln per stack

pub const FIRE_BLAST_COOLDOWN: f64 = 8.0; 

pub const POWER_INFUSION: f64 = 0.20;
pub const ARCANE_POWER: f64 = 0.3;
pub const MQG_HASTE: f64 = 0.33;           // Mind Quickening Gem cast speed bonus

pub const NUM_BUFFS: usize = 4;            // Sapp, TOEP, ZHC, MQG
pub const NUM_DAMAGE_BUFFS: usize = 3;     // Sapp, TOEP, ZHC
pub const BUFF_DURATION: [f64; NUM_BUFFS] = [20.0, 15.0, 20.0, 20.0];
pub const BUFF_COOLDOWN: [f64; NUM_BUFFS] = [120.0, 90.0, 120.0, 300.0];
pub const PI_DURATION: f64 = 15.0;
pub const PI_COOLDOWN: f64 = 180.0;
pub const BERSERK_DURATION: f64 = 10.0;
pub const BERSERK_COOLDOWN: f64 = 180.0;

pub const AP_COOLDOWN: f64 = 180.0;
pub const AP_DURATION: f64 = 15.0;
pub const POM_COOLDOWN: f64 = 180.0;

/// Flat damage added per spell hit while active (Sapp, TOEP, ZHC)
pub const BUFF_DAMAGE: [f64; NUM_DAMAGE_BUFFS] = [130.0, 175.0, 204.0];
/// Damage per-tick added per single cast while the buff is active (ZHC drains -17 per cast)
pub const BUFF_PER_TICK: [f64; NUM_DAMAGE_BUFFS] = [0.0, 0.0, -17.0];

pub const COMBUSTIONS: u8 = 3;             // charges
pub const PER_COMBUSTION: f64 = 0.10;      // +10% crit per stack
pub const COMBUSTION_COOLDOWN: f64 = 180.0;

pub const RES_AMOUNT: [f64; 4] = [1.0, 0.75, 0.5, 0.25];
pub const RES_THRESH: [f64; 4] = [0.0, 0.8303, 0.9415, 0.9905];
pub const RES_THRESH_UL: [f64; 4] = [0.8303, 0.9415, 0.9905, 1.0];
/// Folded resistance modifier (kept from Python; used in damage tuning)
pub const RESISTANCE_MODIFIER: f64 = 0.940_997;

pub const DMF_BUFF: f64 = 0.1;
pub const SR_BUFF: f64 = 0.1;
pub const TS_BUFF: f64 = 0.05;
pub const THADDIUS_BUFF: f64 = 1.9;

pub const DECISION_POINT: f64 = 2.0;       // seconds remaining threshold used in rotation logic

pub const DRAGONLING_DURATION: f64 = 60.0;
pub const DRAGONLING_BUFF: f64 = 300.0;    // flat SP during window

pub const NIGHTFALL_PROC_PROB: f64 = 0.15;
pub const NIGHTFALL_VULN: f64 = 0.15;      // +15% spell vulnerability
pub const NIGHTFALL_DURATION: f64 = 5.0;

pub const UDC_MOD: f64 = 0.02;

pub const MAX_QUEUED_SPELLS: usize = 4;
pub const MAX_DEBUFF_HISTORY: usize = 10;
pub const MAX_PI: usize = 4;

pub const PYRO_COUNT: u8 = 4;
pub const PYRO_TIMER: f64 = 3.0;

pub const T3_6P_CHANCE: f64 = 0.2;
pub const T3_6P_DAMAGE: f64 = 200.0;
pub const T3_6P_TIMER: f64 = 30.0;

pub const T2_8P_CHANCE: f64 = 0.1;

/// How many opening Scorches are required by number of mages (index by num_mages)
pub const SCORCHES_BY_MAGES: [i32; 13] = [9000, 6, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1];

/// Maps a Buff to the Action that casts it.
#[inline]
pub const fn buff_cast_action(buff: Buff) -> Action {
    match buff {
        Buff::Sapp => Action::Sapp,
        Buff::Toep => Action::Toep,
        Buff::Zhc => Action::Zhc,
        Buff::Mqg => Action::Mqg,
    }
}

/// Configuration inputs that influence constant tables.
#[derive(Debug, Clone)]
pub struct ConstantsConfig {
    pub fireball_rank: u8,        // default 12
    pub frostbolt_rank: u8,       // default 11
    pub simple_spell: bool,       // use simplified bases/ranges
}

impl Default for ConstantsConfig {
    fn default() -> Self {
        Self {
            fireball_rank: 12,
            frostbolt_rank: 11,
            simple_spell: false,
        }
    }
}

/// Constants computed from config at sim start (immutable afterward)
#[derive(Debug, Clone)]
pub struct Constants {
    // Multipliers and base/range tables are aligned to NUM_SPELLS order
    pub sp_multiplier: [f64; NUM_SPELLS],
    pub damage_multiplier: [f64; NUM_SPELLS],
    pub spell_base: [f64; NUM_SPELLS],
    pub spell_range: [f64; NUM_SPELLS],
    pub is_pyro: [bool; NUM_SPELLS],
    pub is_scorch: [bool; NUM_SPELLS],
    pub is_fire: [bool; NUM_SPELLS],
    pub incin_bonus: [f64; NUM_SPELLS],
    pub scorch_chance: f64,
    pub wc_chance: f64,
    pub can_pyro: bool,

    /// Cast time contribution (without reaction delay); use GLOBAL_COOLDOWN for Action::Gcd
    pub cast_time: [f64; NUM_SPELLS],
    /// Projectile/travel time until impact for non-instant spells
    pub spell_travel: [f64; NUM_SPELLS],

    pub fb_cooldown: f64,

    // Crit math (variant for talented Frostbolt is handled here)
    pub is_ignite: bool,
    pub ignite_damage: f64,   // 0.2 of crit
    pub icrit_damage: f64,    // +0.5 for fire crits to emulate 150%
    pub crit_damage: f64,     // +0.5 normal or +1.0 for talented frostbolt

    pub spell_trigger_t2_8p: [bool; NUM_SPELLS],
}

impl Constants {
    pub fn new(talents: &TalentPoints) -> Self {

        let cfg = ConstantsConfig::default();

        // Base SP and damage multipliers
        let mut sp_multiplier = [0.428_571_429, 1.0, 1.0, 0.428_571_429, 0.814_285_714, 0.6];
        let fire_mult = (1.0 + 0.02 * talents.get(Talent::FirePower) as f64) * (1.0 + 0.01 * talents.get(Talent::ArcaneInstability) as f64);
        let frost_mult = (1.0 + 0.02 * talents.get(Talent::PiercingIce) as f64) * (1.0 + 0.01 * talents.get(Talent::ArcaneInstability) as f64);
        let damage_multiplier = [fire_mult, fire_mult, fire_mult, fire_mult, frost_mult, fire_mult]; 

        // Base and ranges: simplified or detailed
        let (mut spell_base, mut spell_range) = if cfg.simple_spell {
            ([250.0, 900.0, 750.0, 500.0, 500.0, 268.0], [0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        } else {
            ([237.0, 716.0, 596.0, 446.0, 515.0, 268.0], [43.0, 174.0, 164.0, 78.0, 40.0, 0.0])
        };

        // Rank overrides
        if cfg.fireball_rank == 11 { // Fireball (rank 11)
            spell_base[Spell::Fireball as usize] = 561.0;
            spell_range[Spell::Fireball as usize] = 154.0;
        }
        match cfg.frostbolt_rank {
            10 => {
                spell_base[Spell::Frostbolt as usize] = 440.0;
                spell_range[Spell::Frostbolt as usize] = 75.0;
            }
            1 => {
                spell_base[Spell::Frostbolt as usize] = 20.0;
                spell_range[Spell::Frostbolt as usize] = 2.0;
                // Faster rank with different coeff
                let fb = Spell::Frostbolt as usize;
                sp_multiplier[fb] = 0.407_142_857;
            }
            _ => {}
        }

        // Cast times (base, before MQG/gcd math)
        let mut cast_time = [1.5, 6.0, 3.5 - 0.1 * talents.get(Talent::ImprovedFireball) as f64, 0.0, 3.0 - 0.1 * talents.get(Talent::ImprovedFrostbolt) as f64, 0.0];
        if cfg.frostbolt_rank == 1 {
            cast_time[Spell::Frostbolt as usize] = 1.5;
        }

        // Projectile/travel times (to impact)
        let spell_travel = [0.0, 0.875, 0.875, 0.0, 0.75, 0.0];

        let fb_cooldown = FIRE_BLAST_COOLDOWN - 0.5 * talents.get(Talent::ImprovedFireBlast) as f64;

        let can_pyro: bool = talents.get(Talent::Pryoblast) > 0;

        // Flags for spell schools
        let is_pyro = [false, true, false, false, false, false]; 
        let is_scorch = [true, false, false, false, false, false];
        let is_fire = [true, true, true, true, false, true];

        // Incinerate talent bonus to Scorch/Fire Blast crit chance
        let incin_bonus = [0.02 * talents.get(Talent::Incinerate) as f64, 0.0, 0.0, 0.02 * talents.get(Talent::Incinerate) as f64, 0.0, 0.0];

        // improved scorch
        let scorch_chance: f64 = 0.33 * talents.get(Talent::ImprovedScorch) as f64;

        // winter's chill
        let wc_chance: f64 = 0.2 * talents.get(Talent::WintersChill) as f64;

        // Crit/ignite math
        let is_ignite: bool = talents.get(Talent::Ignite) > 0;
        let ignite_damage = 0.04 * talents.get(Talent::Ignite) as f64;
        let icrit_damage = 0.5; // +50% for fire crits
        let crit_damage = 0.5 + 0.1 * talents.get(Talent::IceShards) as f64;

        let spell_trigger_t2_8p = [false, false, true, false, true, false];

        Self {
            sp_multiplier,
            damage_multiplier,
            spell_base,
            spell_range,
            is_pyro,
            is_scorch,
            is_fire,
            can_pyro,
            fb_cooldown,
            scorch_chance,
            is_ignite,
            incin_bonus,
            wc_chance,
            cast_time,
            spell_travel,
            ignite_damage,
            icrit_damage,
            crit_damage,
            spell_trigger_t2_8p,
        }
    }
}

/// Convenience for printing the same header Python used (optional)
pub fn log_message() {
    println!("\n===== Simulation Log =====");
}
