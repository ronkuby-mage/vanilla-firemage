//! state.rs — dynamic per-simulation state + mechanics (faithful to Python)

use core::f64;

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use crate::constants::{self as C, Buff};
use crate::constants::{Action, Spell, Constants};
use crate::orchestration::{DamageAccumulator, LogEntry, LogType, SpellResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct Totals {
    pub total_damage: f64,
    pub player_damage: f64,
    pub ignite_damage: f64,
    pub crit_damage: f64,
}

#[derive(Debug, Clone)]
pub struct Global {
    pub running_time: f64,
    pub duration: f64,
    pub decision_gate: bool,
}
impl Global { pub fn new(duration: f64) -> Self { Self { running_time: 0.0, duration, decision_gate: false } } }

#[derive(Debug, Clone)]
pub struct Boss {
    pub ignite_timer: f64,
    pub ignite_count: u8,
    pub ignite_value: f64,
    pub ignite_multiplier: f64,
    pub tick_timer: f64,
    pub scorch_timer: f64,
    pub scorch_count: u8,
    pub spell_vulnerability: f64,
    pub dragonling_start: f64,
    pub nightfall: Vec<f64>,

    pub scorch_refresh_history: Vec<f64>,  // Times when scorch was refreshed
    pub ignite_refresh_history: Vec<f64>,  // Times when ignite was refreshed    
}
impl Default for Boss {
    fn default() -> Self {
        Self {
            ignite_timer: 0.0,
            ignite_count: 0,
            ignite_value: 0.0,
            ignite_multiplier: 1.0,
            tick_timer: f64::INFINITY,
            scorch_timer: 0.0,
            scorch_count: 0,
            spell_vulnerability: 0.0,
            dragonling_start: -C::DRAGONLING_DURATION,
            nightfall: vec![],
            scorch_refresh_history: Vec::new(),
            ignite_refresh_history: Vec::new(),
        }
    }
}

// dynamic values
#[derive(Debug, Clone)]
pub struct MageLane {
    pub cast_type: Action,
    pub spell_type: [Spell; C::MAX_QUEUED_SPELLS], // Changed to fixed-size array
    pub cast_timer: f64,
    pub spell_timer: [f64; C::MAX_QUEUED_SPELLS],  // Changed to fixed-size array
    pub gcd_timer: f64,
    pub fb_cooldown: f64,
    pub comb_stack: u8,
    pub comb_left: u8,
    pub comb_cooldown: f64,
    pub buff_timer: [f64; C::NUM_BUFFS],
    pub buff_cooldown: [f64; C::NUM_BUFFS],
    pub buff_ticks: [u32; C::NUM_DAMAGE_BUFFS],
    pub pi_timer: [f64; C::MAX_PI],
    pub pi_cooldown: [f64; C::MAX_PI],
    pub berserk_timer: f64,
    pub berserk_cooldown: f64,
    pub pyro_timer: f64,
    pub pyro_count: u8,
    pub pyro_value: f64,
    pub crit_too_late: bool,
    pub hit_chance: f64,
    pub crit_chance: f64,
    pub spell_power: f64,
    pub cast_number: i32,
    pub damage: f64,
}

impl Default for MageLane {
    fn default() -> Self {
        Self {
            cast_type: Action::Gcd,
            spell_type: [Spell::Scorch; C::MAX_QUEUED_SPELLS], // Initialize array with default
            cast_timer: f64::INFINITY,
            spell_timer: [f64::INFINITY; C::MAX_QUEUED_SPELLS], // Initialize array with INFINITY
            gcd_timer: 0.0,
            fb_cooldown: 0.0,
            comb_stack: 0,
            comb_left: 0,
            comb_cooldown: 0.0,
            buff_timer: [0.0; C::NUM_BUFFS],
            buff_cooldown: [f64::INFINITY; C::NUM_BUFFS],
            buff_ticks: [0; C::NUM_DAMAGE_BUFFS],
            pi_timer: [0.0; C::MAX_PI],
            pi_cooldown: [f64::INFINITY; C::MAX_PI],
            berserk_timer: 0.0,
            berserk_cooldown: f64::INFINITY,
            pyro_timer: f64::INFINITY,
            pyro_count: 0,
            pyro_value: 0.0,
            crit_too_late: false,
            hit_chance: 0.99,
            crit_chance: 0.062,
            spell_power: 0.0,
            cast_number: -1,
            damage: 0.0,
        }
    }
}

// static values
#[derive(Debug, Clone, Default)]
pub struct PlayerMeta {
    pub pi_count: Vec<usize>,
    pub dmf_slots: Vec<usize>,
    pub sr_slots: Vec<usize>,
    pub ts_slots: Vec<usize>,
    pub cleaner_slots: Vec<usize>,
    pub berserk_slots: Vec<f64>,
    pub target_slots: Vec<usize>,
    pub nightfall_period: Vec<f64>,
    pub vulnerability: f64,
    pub coe: f64,
    pub no_debuff_limit: bool,
    pub name: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub global: Global,
    pub boss: Boss,
    pub lanes: Vec<MageLane>,
    pub meta: PlayerMeta,
    pub totals: Totals,
    pub log_enabled: bool,
    pub log: Vec<LogEntry>,
    pub damage_log: Vec<DamageAccumulator>,
}

fn combustion_string(mage_line: &mut MageLane) -> String {
    if mage_line.comb_left > 0 {
        return format!("stack: {}  remain: {}", mage_line.comb_stack, mage_line.comb_left);
    } else if mage_line.comb_cooldown <= 0.0 {
        return format!("off cooldown")
    }
    format!("cooldown: {:.2}", mage_line.comb_cooldown)
}

fn buff_string(mage_lane: &mut MageLane) -> String {
    let mut buffs: String = "".to_owned();
    if mage_lane.buff_timer[Buff::Sapp as usize] > 0.0 {
        buffs.push_str("Sapp");
    } else {
        buffs.push_str("    ");
    }
    if mage_lane.buff_timer[Buff::Toep as usize] > 0.0 {
        buffs.push_str("Toep");
    } else {
        buffs.push_str("    ");
    }
    if mage_lane.buff_timer[Buff::Zhc as usize] > 0.0 {
        buffs.push_str("ZHC");
    } else {
        buffs.push_str("   ");
    }
    if mage_lane.buff_timer[Buff::Mqg as usize] > 0.0 {
        buffs.push_str("MQG");
    } else {
        buffs.push_str("   ");
    }
    if mage_lane.pi_timer.iter().cloned().reduce(f64::max).unwrap() > 0.0 {
        buffs.push_str("PI");
    } else {
        buffs.push_str("  ");
    }
    if mage_lane.berserk_timer > 0.0 {
        buffs.push_str("BSK");
    } else {
        buffs.push_str("  ");
    }
    buffs
}

fn debuff_string(boss: &mut Boss, dragonling_active: bool) -> String {
    let mut debuffs: String = "".to_owned();
    
    if boss.scorch_timer > 0.0 {
        debuffs.push_str(format!("scorch:{}({:.2}) ", boss.scorch_count, boss.scorch_timer).as_str());
    } else {
        debuffs.push_str("scorch:0 ");
    }
    if boss.ignite_timer > 0.0 {
        debuffs.push_str(format!("ignite:{}({:.2})[{:.0}|{:.2}] ", boss.ignite_count, boss.ignite_timer, boss.ignite_value, boss.ignite_multiplier).as_str());
    } else {
        debuffs.push_str("ignite:0 ");
    }
    if dragonling_active {
        debuffs.push_str("dragonling ");
    }
    if boss.spell_vulnerability > 0.0 {
        debuffs.push_str(format!("nightfall:{:.2}", boss.spell_vulnerability).as_str());
    }
    debuffs
}

impl State {
    pub fn new(duration: f64, num_mages: usize) -> Self {
        Self {
            global: Global::new(duration),
            boss: Boss::default(),
            lanes: vec![MageLane::default(); num_mages],
            meta: PlayerMeta::default(),
            totals: Totals::default(),
            log_enabled: false,
            log: vec![],
            damage_log: vec![],
        }
    }

    pub fn log_cast(&mut self, log_type: LogType, unit_id: i32, spell: Action) {
        let l = &mut self.lanes[unit_id as usize];
        let dragonling_active = (self.global.running_time >= self.boss.dragonling_start) && (self.global.running_time < self.boss.dragonling_start + C::DRAGONLING_DURATION);
        let b = &mut self.boss;
        self.log.push(LogEntry {
            log_type: log_type,
            text: format!("s[{}]", spell),
            unit_name: self.meta.name[unit_id as usize].clone(),
            t: self.global.running_time,
            dps: 0.0,
            total_dps: if self.global.running_time > 0.0 { self.totals.total_damage / self.global.running_time } else { 0.0 },
            ignite_dps: if self.global.running_time > 0.0 { self.totals.ignite_damage / self.global.running_time } else { 0.0 },
            value: 0.0,
            value2: 0.0,
            spell_result: SpellResult::None,
            combustion: combustion_string(l),
            buffs: buff_string(l),
            debuffs: debuff_string(b, dragonling_active),
        });
    }

    pub fn log_tick(&mut self, value: f64, partial: f64) {
        let dragonling_active = (self.global.running_time >= self.boss.dragonling_start) && (self.global.running_time < self.boss.dragonling_start + C::DRAGONLING_DURATION);
        let b = &mut self.boss;
        self.log.push(LogEntry {
            log_type: LogType::IgniteTick,
            text: format!("a[Ignite] -> t[{:.0}]", value),
            unit_name: format!(""),
            t: self.global.running_time,
            dps: 0.0,
            total_dps: if self.global.running_time > 0.0 { self.totals.total_damage / self.global.running_time } else { 0.0 },
            ignite_dps: if self.global.running_time > 0.0 { self.totals.ignite_damage / self.global.running_time } else { 0.0 },
            value: value,
            value2: (1.0 - partial)*value,
            spell_result: SpellResult::Hit,
            combustion: String::new(),
            buffs: String::new(),
            debuffs: debuff_string(b, dragonling_active),
        });
    }

    pub fn log_spell_impact(&mut self, unit_id: i32, spell: Spell, value: f64, partial: f64, result: SpellResult) {

        let l = &mut self.lanes[unit_id as usize];
        let dragonling_active = (self.global.running_time >= self.boss.dragonling_start) && (self.global.running_time < self.boss.dragonling_start + C::DRAGONLING_DURATION);
        let b = &mut self.boss;
        self.log.push(LogEntry {

            log_type: LogType::SpellImpact,
            text: format!("s[{}] -> t[{:.0}]", spell, value),
            unit_name: self.meta.name[unit_id as usize].clone(),
            t: self.global.running_time,
            dps: 0.0,
            total_dps: if self.global.running_time > 0.0 { self.totals.total_damage / self.global.running_time } else { 0.0 },
            ignite_dps: if self.global.running_time > 0.0 { self.totals.ignite_damage / self.global.running_time } else { 0.0 },
            value: value,
            value2: (1.0 - partial)*value,
            spell_result: result,
            combustion: combustion_string(l),
            buffs: buff_string(l),
            debuffs: debuff_string(b, dragonling_active),
        });
    }

    // ---------- time & scheduling ----------
    pub fn subtime(&mut self, dt: f64) {
        self.global.running_time += dt;
        self.boss.ignite_timer -= dt;
        self.boss.tick_timer -= dt;
        self.boss.scorch_timer -= dt;
        self.boss.spell_vulnerability -= dt;
        for l in &mut self.lanes {
            l.cast_timer -= dt;
            for timer in &mut l.spell_timer {
                *timer -= dt;
            }
            l.gcd_timer -= dt;
            l.comb_cooldown -= dt;
            l.fb_cooldown -= dt;
            for t in &mut l.buff_timer { *t -= dt; }
            for c in &mut l.buff_cooldown { *c -= dt; }
            for t in &mut l.pi_timer { *t -= dt; }
            for c in &mut l.pi_cooldown { *c -= dt; }
            l.berserk_cooldown -= dt;
            l.berserk_timer -= dt;
        }
        if self.meta.no_debuff_limit {
            for l in &mut self.lanes { l.pyro_timer -= dt; }
        }
        for t in &mut self.boss.nightfall {
            *t -= dt;
        }
    }

    pub fn next_cast_lane(&self) -> Option<usize> {
        self.lanes
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.cast_timer.total_cmp(&b.1.cast_timer))
            .map(|(i, _)| i)
    }

    pub fn next_pyro_lane(&self) -> Option<usize> {
        self.lanes
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.pyro_timer.total_cmp(&b.1.pyro_timer))
            .map(|(i, _)| i)
    }

    pub fn next_spell_lane(&self) -> Option<usize> {
        // Find the lane with the minimum spell_timer value across all queued spells
        self.lanes
            .iter()
            .enumerate()
            .min_by(|a, b| {
                let min_a = a.1.spell_timer.iter().copied().fold(f64::INFINITY, f64::min);
                let min_b = b.1.spell_timer.iter().copied().fold(f64::INFINITY, f64::min);
                min_a.total_cmp(&min_b)
            })
            .map(|(i, _)| i)
    }

    pub fn set_decision_gate(&mut self, on: bool) { self.global.decision_gate = on; }
    pub fn decision_gate(&self) -> bool { self.global.decision_gate }
    pub fn in_progress(&self) -> bool { self.global.running_time < self.global.duration }

    /// Called by the decider mapping of _apply_decisions → start_action
    pub fn start_action(&mut self, lane: usize, action: Action, continuing_delay: f64, k: &Constants) {
        use crate::constants::{Action as A, Buff as B, Spell as S};

        let l = &mut self.lanes[lane];

        // schedule start
        l.cast_timer = continuing_delay;
        l.cast_type = action;

        // GCD spells add cast time and compute leftover gcd
        if action == A::Gcd {
            l.cast_timer = C::GLOBAL_COOLDOWN;
        }
        else if action.triggers_gcd() {
            let base_cast = match action {
                A::Scorch      => k.cast_time[S::Scorch as usize],
                A::Pyroblast   => k.cast_time[S::Pyroblast as usize],
                A::Fireball    => k.cast_time[S::Fireball as usize],
                A::FireBlast   => k.cast_time[S::FireBlast as usize], // 0.0 in constants; still on GCD
                A::Frostbolt   => k.cast_time[S::Frostbolt as usize],
                _ => 0.0,
            };

            // MQG haste (Python divides cast portion by 1+MQG)
            let mut cast_time: f64 = base_cast;
            if action != A::Scorch {
                let mut haste: f64 = 1.0;
                haste *= if l.buff_timer[B::Mqg as usize] > 0.0 { 1.0 + C::MQG_HASTE } else { 1.0 };
                haste *= if l.berserk_timer > 0.0 { 1.0 + self.meta.berserk_slots[lane] } else { 1.0 };
                cast_time /= haste;
                cast_time = cast_time.max(C::GLOBAL_COOLDOWN);
            }

            let total = continuing_delay + cast_time;
            l.cast_timer = total;

            // leftover GCD stored to be pushed after cast ends
            let gcd_len = C::GLOBAL_COOLDOWN;
            l.gcd_timer = (gcd_len + continuing_delay - total).max(0.0);
        } else {
            l.gcd_timer = 0.0;
        }
        if self.log_enabled {
            self.log_cast(LogType::CastStart, lane as i32, action);    
        }

        // Block decisions until the event is handled (Python clears global decision flag)
        self.set_decision_gate(false);
    }

    pub fn finish_cast(&mut self, k: &Constants) {
        use crate::constants::{Action as A, Buff as B, Spell as S};

        // Which lane just finished its cast?
        let Some(lane) = self.next_cast_lane() else { return };
        let dt = self.lanes[lane].cast_timer;
        self.subtime(dt); // advance global & subtract dt from all timers

        // Snapshot lane and cast type
        let l = &mut self.lanes[lane];
        let action = l.cast_type;

        // 1) transfer to spell stage if it's a non-instant 
        let is_instant = matches!(action, A::Combustion | A::Sapp | A::Toep | A::Zhc | A::Mqg | A::PowerInfusion | A::Berserking | A::Gcd);
        if !is_instant {
            // map Action → Spell index
            let spell = match action {
                A::Scorch    => S::Scorch,
                A::Pyroblast => S::Pyroblast,
                A::Fireball  => S::Fireball,
                A::FireBlast => S::FireBlast, // its SPELL_TIME will be 0, handled as instant projectile
                A::Frostbolt => S::Frostbolt,
                _ => S::Scorch, // safe default; you can refine
            };
            // Find the first available slot in the spell queue (timer == f64::INFINITY)
            if let Some(slot) = l.spell_timer.iter().position(|&t| t == f64::INFINITY) {
                l.spell_type[slot] = spell;
                l.spell_timer[slot] = k.spell_travel[spell as usize];
            }
            // If no slots available, the spell is dropped (queue full)

            // Special: fire blast starts its own cooldown on *cast end* in Python
            if matches!(action, A::FireBlast) {
                l.fb_cooldown = C::FIRE_BLAST_COOLDOWN;
            }
        } else {

            // 2) apply instant casts’ effects immediately (Python “apply instant spells” block)
            match action {
                A::Combustion => {
                    l.comb_left = C::COMBUSTIONS as u8;
                    l.comb_stack = 1;
                    l.comb_cooldown = f64::INFINITY; // temp hold until last charge is used
                }
                A::Sapp | A::Toep | A::Zhc | A::Mqg  => {
                    let b = match action {
                        A::Sapp => B::Sapp,
                        A::Toep => B::Toep,
                        A::Zhc => B::Zhc,
                        A::Mqg => B::Mqg,
                        _ => unreachable!(),
                    } as usize;

                    // start active duration and cooldown
                    l.buff_timer[b] = C::BUFF_DURATION[b];
                    l.buff_cooldown[b] = C::BUFF_COOLDOWN[b];

                    // reset damage-buff tick counters
                    if b < C::NUM_DAMAGE_BUFFS {
                        l.buff_ticks[b] = 0;
                    }
                    // Internal cooldown on all *other* dmg trinkets + MQG:
                    // set their cooldown to at least this buff’s duration
                    let lock_icd = C::NUM_DAMAGE_BUFFS + 1; // include MQG
                    for bb in 0..lock_icd {
                        if bb == b { continue; }
                        l.buff_cooldown[bb] = l.buff_cooldown[bb].max(C::BUFF_DURATION[b]);
                    }
                }
                A::PowerInfusion => {
                    let slot = l.pi_cooldown.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap();

                    // start active duration and cooldown
                    l.pi_timer[slot] = C::PI_DURATION;
                    l.pi_cooldown[slot] = C::PI_COOLDOWN;
                }
                A::Berserking => {
                    // start active duration and cooldown
                    l.berserk_timer = C::BERSERK_DURATION;
                    l.berserk_cooldown = C::BERSERK_COOLDOWN;
                }
                _ => {}
            }
        }

        let push_gcd: bool;
        {
            let l = &mut self.lanes[lane];

            if l.gcd_timer > 0.0 {
                // schedule a GCD placeholder cast immediately
                l.cast_type  = Action::Gcd;
                l.cast_timer = l.gcd_timer;
                l.gcd_timer  = 0.0;
                push_gcd = true;
            } else {
                // we’ll open the decision gate after we drop `l`
                push_gcd = false;
            }
            // `l` goes out of scope here — mutable borrow ends.
        }

        // ---- now we can borrow &mut self again safely ----
        if !push_gcd {
            self.set_decision_gate(true);
            let cn = self.lanes[lane].cast_number;
            self.lanes[lane].cast_number = cn.saturating_add(1);
        }
        if self.log_enabled {
            self.log_cast(LogType::CastSuccess, lane as i32, action); 
        }

    }

    // ---------- mechanics: landing & effects (faithful to Python) ----------
    pub fn land_spell(&mut self, k: &Constants, rng: &mut ChaCha8Rng) {
        let Some(lane) = self.next_spell_lane() else { return };
        
        // Find the spell slot with the minimum timer in this lane
        let l = &self.lanes[lane];
        let (slot, &min_timer) = l.spell_timer
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .unwrap(); // We know there's at least one element
        
        let dt = min_timer;
        self.subtime(dt);

        if !self.in_progress() { return }
        let lanes_len = self.lanes.len();

        // grab lane fields you need for early checks
        let lane_hit = self.lanes[lane].hit_chance;
        let spell_string = self.lanes[lane].spell_type[slot];
        let spell_type = self.lanes[lane].spell_type[slot] as usize;
        let l = &mut self.lanes[lane];
        
        // Clear the processed spell slot
        l.spell_timer[slot] = f64::INFINITY;

        // use the stashed values instead of reading through `l` where possible
        if rng.r#gen::<f64>() >= lane_hit {
            if self.log_enabled {
                self.log_spell_impact(lane as i32, spell_string, 0.0, 0.0, SpellResult::Miss);
            }
            return;
        }

        // ---- read-only stuff from &self (no &mut borrow yet) ----
        let targeted_all = self.meta.target_slots.len() == lanes_len;
        let is_cleaner = self.meta.cleaner_slots.iter().any(|&i| i == lane);
        let is_target = targeted_all || self.meta.target_slots.iter().any(|&i| i == lane);
        let is_dmf = self.meta.dmf_slots.iter().any(|&i| i == lane);
        let is_sr = self.meta.sr_slots.iter().any(|&i| i == lane);
        let is_ts = self.meta.ts_slots.iter().any(|&i| i == lane);

        let dragonling_active = (self.global.running_time >= self.boss.dragonling_start) && (self.global.running_time < self.boss.dragonling_start + C::DRAGONLING_DURATION);
        let mut buff_damage = if dragonling_active { C::DRAGONLING_BUFF } else { 0.0 };
        for b in 0..C::NUM_DAMAGE_BUFFS { if l.buff_timer[b] > 0.0 { buff_damage += C::BUFF_DAMAGE[b] + (l.buff_ticks[b] as f64)*C::BUFF_PER_TICK[b]; l.buff_ticks[b]=l.buff_ticks[b].saturating_add(1); } }

        let base_roll: f64 = rng.r#gen();
        let mut spell_damage = k.spell_base[spell_type] + base_roll*k.spell_range[spell_type] + k.sp_multiplier[spell_type]*(l.spell_power + buff_damage);
        let mut partial: f64 = 1.0;
        if k.is_fire[spell_type] {
            let r: f64 = rng.r#gen();
            partial = if r < C::RES_THRESH[1] { C::RES_AMOUNT[0] } else if r < C::RES_THRESH[2] { C::RES_AMOUNT[1] } else if r < C::RES_THRESH[3] { C::RES_AMOUNT[2] } else { C::RES_AMOUNT[3] };
            spell_damage *= partial;
        }

        // all damage multipliers
        spell_damage *= self.meta.coe * k.damage_multiplier[spell_type]; // COE + fire power
        if k.is_fire[spell_type] && self.boss.scorch_timer > 0.0 { spell_damage *= 1.0 + C::SCORCH_MULTIPLIER*(self.boss.scorch_count as f64); }
        if l.pi_timer.iter().any(|&x| x > 0.0) { spell_damage *= 1.0 + C::POWER_INFUSION; }
        if self.boss.spell_vulnerability > 0.0 { spell_damage *= 1.0 + C::NIGHTFALL_VULN; }
        if is_dmf { spell_damage *= 1.0 + C::DMF_BUFF; }
        if is_sr { spell_damage *= 1.0 + C::SR_BUFF; }
        if is_ts { spell_damage *= 1.0 + C::TS_BUFF; }
        spell_damage *= self.meta.vulnerability; // Thaddius
        if is_cleaner { spell_damage *= 1.0 + C::UDC_MOD }

        // add to total
        self.totals.total_damage += spell_damage;
        l.damage += spell_damage;
        //if is_target { self.totals.player_damage += spell_damage; }

        let is_fire = k.is_fire[spell_type];
        // getting rid of buffer bonus for now
        //let comb_bonus = if is_fire && !is_scorch && l.comb_left > 0 { C::PER_COMBUSTION * (l.comb_stack as f64) } else { 0.0 };
        let comb_bonus = if is_fire && l.comb_left > 0 { C::PER_COMBUSTION * (l.comb_stack as f64) } else { 0.0 };
        let crit_chance = (l.crit_chance + comb_bonus + k.incin_bonus[spell_type]).clamp(0.0, 1.0);
        let is_crit = rng.r#gen::<f64>() < crit_chance;

        if is_crit {
            if is_fire {
                // ignite timer checks
                if self.boss.ignite_timer <= 0.0 {
                    self.boss.ignite_count = 0;
                    self.boss.ignite_value = 0.0;
                }
                if self.boss.ignite_timer < C::DECISION_POINT { l.crit_too_late = true; }
                if self.boss.tick_timer > C::IGNITE_TICK && self.boss.ignite_count > 0 { self.boss.tick_timer = C::IGNITE_TICK; }
                if self.boss.ignite_timer > 0.0 {
                    self.boss.ignite_refresh_history.push(self.global.running_time);
                    // Optional: Keep history size manageable (e.g., last 10 refreshes)
                    if self.boss.ignite_refresh_history.len() > C::MAX_DEBUFF_HISTORY {
                        self.boss.ignite_refresh_history.remove(0);
                    }
                }                
                self.boss.ignite_timer = C::IGNITE_TIME + 1e-6;

                if self.boss.ignite_count == 0 {
                    self.boss.tick_timer = C::IGNITE_TICK;
                    let pi_mult = if l.pi_timer.iter().any(|&x| x > 0.0) { 1.0 + C::POWER_INFUSION } else { 1.0 };
                    let dmf_mult = if is_dmf {1.0 + C::DMF_BUFF} else { 1.0 };
                    let sr_mult = if is_sr {1.0 + C::SR_BUFF} else { 1.0 };
                    let ts_mult = if is_ts {1.0 + C::TS_BUFF} else { 1.0 };
                    // snap shot value
                    self.boss.ignite_multiplier = if is_cleaner { 1.0 + C::UDC_MOD } else { 1.0 } * pi_mult * dmf_mult * sr_mult * ts_mult * self.meta.vulnerability;
                }
                if self.boss.ignite_count < C::IGNITE_STACK {
                    let crit_mult = 1.0 + k.icrit_damage; // 1.5
                    let ignite_add = crit_mult * k.ignite_damage * spell_damage;
                    if is_cleaner { self.boss.ignite_value += (1.0 + C::UDC_MOD) * ignite_add; } else { self.boss.ignite_value += ignite_add; }
                    self.boss.ignite_count = self.boss.ignite_count.saturating_add(1).min(C::IGNITE_STACK);
                }

                // subtract previous damage from totals
                self.totals.total_damage -= spell_damage;
                l.damage -= spell_damage;
                //if is_target { self.totals.player_damage -= spell_damage; }

                // calculate crit damage
                let crit_line = (1.0 + k.icrit_damage) * spell_damage; // 1.5x for fire crit ledger
                let crit_mult_line = if is_cleaner { 1.0 + C::UDC_MOD } else { 1.0 };

                // add crit damage
                self.totals.total_damage += crit_line * crit_mult_line;
                l.damage += crit_line * crit_mult_line;
                //if is_target { self.totals.player_damage += crit_line * crit_mult_line; }

                // reset spell damage
                spell_damage = crit_line * crit_mult_line;

                if l.comb_left == 1 { l.comb_cooldown = C::COMBUSTION_COOLDOWN; }
                if l.comb_left > 0 { l.comb_left -= 1; }
            } else {
                let extra = k.crit_damage * spell_damage; // +0.5 or +1.0
                self.totals.total_damage += extra;
                if is_target { self.totals.player_damage += extra; }
            }
        }
        if self.boss.scorch_timer <= 0.0 {
            self.boss.scorch_count = 0
        }
        if k.is_scorch[spell_type] {
            if rng.r#gen::<f64>() < lane_hit {
                if self.boss.scorch_timer > 0.0 {
                    self.boss.scorch_refresh_history.push(self.global.running_time);
                    if self.boss.scorch_refresh_history.len() > C::MAX_DEBUFF_HISTORY {
                        self.boss.scorch_refresh_history.remove(0);
                    }
                }

                self.boss.scorch_timer = C::SCORCH_TIME;
                self.boss.scorch_count = (self.boss.scorch_count + 1).min(C::SCORCH_STACK);
            }
        }

        if self.meta.no_debuff_limit && k.is_pyro[spell_type] {
            let mut tick_damage = k.spell_base[Spell::PyroDot as usize] + k.sp_multiplier[Spell::PyroDot as usize]*(l.spell_power + buff_damage);
            tick_damage *= k.damage_multiplier[Spell::PyroDot as usize]; // fire power
            if l.pi_timer.iter().any(|&x| x > 0.0) { tick_damage *= 1.0 + C::POWER_INFUSION; }
            if is_dmf { tick_damage *= 1.0 + C::DMF_BUFF; }
            if is_sr { tick_damage *= 1.0 + C::SR_BUFF; }
            if is_ts { tick_damage *= 1.0 + C::TS_BUFF; }
            if is_cleaner { tick_damage *= 1.0 + C::UDC_MOD }
            l.pyro_count = C::PYRO_COUNT;
            l.pyro_timer = C::PYRO_TIMER;
            l.pyro_value = tick_damage/(C::PYRO_COUNT as f64);
        }

        if is_fire { l.comb_stack = l.comb_stack.saturating_add(1); }

        if self.log_enabled {
            if is_crit {
                self.log_spell_impact(lane as i32, spell_string, spell_damage, partial, SpellResult::Crit);
                //println!("  {:6.2} mage {} spell {} CRIT for {}", self.global.running_time, lane, spell_string, spell_damage)
            } else {
                self.log_spell_impact(lane as i32, spell_string, spell_damage, partial, SpellResult::Hit);
                //println!("  {:6.2} mage {} spell {} hit  for {}", self.global.running_time, lane, spell_string, spell_damage)
            }
        } else {
            self.damage_log.push(DamageAccumulator { time: self.global.running_time, damage: spell_damage});
        }

    }

    pub fn tick_ignite(&mut self, rng: &mut ChaCha8Rng) {
        // subtime
        let dt = self.boss.tick_timer;
        self.subtime(dt);

        if !self.in_progress() { return }

        if self.boss.ignite_timer >= C::IGNITE_TICK {
            self.boss.tick_timer = C::IGNITE_TICK;
        } else {
            self.boss.tick_timer = f64::INFINITY;
        }
        if self.boss.ignite_timer <= 0.0 {
            return
        }

        let mut mult = C::COE_MULTIPLIER * self.boss.ignite_multiplier;
        if self.boss.scorch_timer > 0.0 { mult *= 1.0 + C::SCORCH_MULTIPLIER*(self.boss.scorch_count as f64); }
        if self.boss.spell_vulnerability > 0.0 { mult *= 1.0 + C::NIGHTFALL_VULN; }
        let r: f64 = rng.r#gen();
        let partial: f64 = if r < C::RES_THRESH[1] { C::RES_AMOUNT[0] } else if r < C::RES_THRESH[2] { C::RES_AMOUNT[1] } else if r < C::RES_THRESH[3] { C::RES_AMOUNT[2] } else { C::RES_AMOUNT[3] };
        mult *= partial;
        let ignite_damage = mult * self.boss.ignite_value;
        self.totals.ignite_damage += ignite_damage;
        if self.log_enabled {
            self.log_tick(ignite_damage, partial);
        } else {
            self.damage_log.push(DamageAccumulator { time: self.global.running_time, damage: ignite_damage});
        }

    }

    pub fn proc_nightfall(&mut self, _k: &Constants, rng: &mut ChaCha8Rng) {
        // 1) find soonest Nightfall check
        let (idx, dt) = match self.boss.nightfall
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
        {
            Some((i, &t)) if t.is_finite() => (i, t),
            _ => return, // nothing scheduled
        };

        // 2) advance time by dt (this also subtracts dt from all boss.nightfall_timers)
        self.subtime(dt);

        // 3) reset this source's swing timer to its period
        if let Some(period) = self.meta.nightfall_period.get(idx).copied() {
            self.boss.nightfall[idx] = period;
        }

        // 4) roll Nightfall proc; if it hits, apply vulnerability window
        if rng.r#gen::<f64>() < C::NIGHTFALL_PROC_PROB {
            self.boss.spell_vulnerability = C::NIGHTFALL_DURATION;
        }
    }

    pub fn tick_pyro(&mut self) {
        let Some(lane) = self.next_pyro_lane() else { return };
        let dt = self.lanes[lane].pyro_timer;
        self.subtime(dt); // advance global & subtract dt from all timers

        if !self.in_progress() { return }

        // Snapshot lane and cast type
        let lanes_len = self.lanes.len();
        let l = &mut self.lanes[lane];
        let targeted_all = self.meta.target_slots.len() == lanes_len;
        let is_target = targeted_all || self.meta.target_slots.iter().any(|&i| i == lane);

        l.pyro_count -= 1;
        if l.pyro_count > 0 { l.pyro_timer = C::PYRO_TIMER; } else { l.pyro_timer = f64::INFINITY }

        let mut mult = C::COE_MULTIPLIER;
        if self.boss.scorch_timer > 0.0 { mult *= 1.0 + C::SCORCH_MULTIPLIER*(self.boss.scorch_count as f64); }
        if self.boss.spell_vulnerability > 0.0 { mult *= 1.0 + C::NIGHTFALL_VULN; }

        let damage = mult * l.pyro_value;
        self.totals.total_damage += damage;
        if is_target { self.totals.player_damage += damage; }

        if self.log_enabled {
            self.log_spell_impact(lane as i32, Spell::PyroDot, damage, 1.0, SpellResult::Hit);
        } else {
            self.damage_log.push(DamageAccumulator { time: self.global.running_time, damage: damage});
        }
    }

    /// One discrete simulation step (faithful to mechanics._advance):
    /// choose the nearest event among: cast finish, spell land, ignite tick, nightfall proc
    /// Priority on ties: cast < spell < tick < proc
    pub fn step_one(&mut self, k: &Constants, rng: &mut ChaCha8Rng) {
        // Gather next event times
        let cast_t  = self.lanes.iter().map(|l| l.cast_timer).fold(f64::INFINITY, f64::min);
        // Find minimum spell_timer across all lanes and all queued spells
        let spell_t = self.lanes.iter()
            .flat_map(|l| l.spell_timer.iter().copied())
            .fold(f64::INFINITY, f64::min);
        let tick_t  = self.boss.tick_timer;
        let proc_t  = self.boss.nightfall.iter().copied().fold(f64::INFINITY, f64::min);
        let pyro_t  = self.lanes.iter().map(|l| l.pyro_timer).fold(f64::INFINITY, f64::min);

        // Short-circuit if nothing scheduled
        if !cast_t.is_finite() && !spell_t.is_finite() && !tick_t.is_finite() && !proc_t.is_finite() && !pyro_t.is_finite() {
            return;
        }

        // Exact Python priority: cast < spell < tick < proc
        if cast_t <= spell_t && cast_t <= tick_t && cast_t <= proc_t && cast_t <= pyro_t {
            self.finish_cast(k);
            return;
        }
        if spell_t <= tick_t && spell_t <= proc_t && spell_t <= pyro_t {
            self.land_spell(k, rng);
            return;
        }
        if tick_t <= proc_t && tick_t <= pyro_t {
            self.tick_ignite(rng);
            return;
        }
        if proc_t <= pyro_t {
            self.proc_nightfall(k, rng);
        } else {
            self.tick_pyro();            
        }
        
    }

}
 