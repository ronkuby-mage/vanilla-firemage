<script setup>
import SimContainer from "./sim_container";
import Tutorial from "./Components/Tutorial.vue";
import { computed, ref, reactive, watch, onMounted, nextTick } from "vue";
import common from "./common";
import presets from "./presets";
import items from "./items";
import aplData from "./apl";
import { mage as talentTree } from "./talents";
import {_, debounce} from "lodash";
import { compressToEncodedURIComponent, decompressFromEncodedURIComponent } from "lz-string";
import { generateRaidsFromTemplate, getDerivedTrinkets, getSustainPermutationsWrapper, getSustain, getPlayerApl } from "./template";
import { preScorchOptions, bufferSpellOptions, derivedOpeningOptions, sustainOptions, scorchPerCrit, scorchPerSP } from "./template";

/*
 * Helpers
 */

const css = (str) => {
    return _.kebabCase(str);
};

const spellUrl = (id) => {
    return "https://www.wowhead.com/classic/spell="+id;
};
const itemTitle = (id) => {
    let item = common.getItem(id);
    return item ? item.title : null;
};
const convertRace = (from) => {
    if (from == "Gnome")
        return "Troll";
    if (from == "Human")
        return "Undead";
    if (from == "Troll")
        return "Gnome";
    if (from == "Undead")
        return "Human";
    return from;
};
const otherSlot = (slot) => {
    var n = slot.substr(-1);
    n = parseInt(n);
    if (isNaN(n))
        return null;
    n = n == 1 ? 2 : 1;
    return slot.substr(0, slot.length-1)+n;
};
const copyToClipboard = (str) => {
    var el = document.createElement("textarea");
    el.value = str;
    el.style.opacity = 0;
    el.style.position = "absolute";
    el.style.top = 0;
    document.body.appendChild(el);
    el.select();
    document.execCommand("copy")
    document.body.removeChild(el);
};
const tooltipRefreshTrigger = computed(() => ({
    tab: activeTab.value,
    gearType: activeGearType.value,
    slot: activeSlot.value,
    playerId: activePlayer.value?.id,
    itemSearch: activeTab.value === 'loadout' ? itemSearch.value : null
}));
const refreshTooltips = debounce(() => {
    if (window.$WowheadPower && 
        (activeTab.value === 'loadout' || activeTab.value === 'config')) {
        window.$WowheadPower.refreshLinks();
    }
}, 300);
/*
 * Stats
 */

/*
 * Gear / loadout
 */
const isItemSpecial = (id) => {
    for (let key in items.ids) {
        if (items.ids[key] == id)
            return true;
    }
    return false;
};
const unequipFromAllPlayers = (id) => {
    let modified = false;
    for (let raid of raids.value) {
        for (let player of raid.players) {
            for (let slot in player.loadout) {
                if (player.loadout[slot].item_id == id) {
                    player.loadout[slot].item_id = null;
                    modified = true;
                }
            }
        }
    }

    if (modified)
        saveRaids(raids.value);
};

/*
 * Config
 */
const defaultConfig = () => {
    return {
        rng_seed: 0,
        duration: 60,
        duration_variance: 0,
        avg_spell_dmg: false,
        target_level: 63,
        target_resistance: 0,
        targets: 1,
        distance: 30,
        reaction_time: 0.5,
        initial_delay: 1.0,
        continuing_delay: 0.01,
        curse_of_elements: true,
        curse_of_shadows: true,
        judgement_of_wisdom: false,
        arcanite_dragonling: "",
        nightfall1: "",
        nightfall2: "",
        nightfall3: "",
        boss: "None",
        in_comparison: true,
        no_debuff_limit: true,
    };
};

const defaultBuffs = () => {
    return {
        // RAID
        arcane_intellect: false,
        imp_mark_of_the_wild: false,
        blessing_of_kings: false,

        // WORLD
        songflower: false,
        rallying_cry: false,
        dire_maul_tribute: false,
        spirit_of_zandalar: false,
        // World snaphot
        dmf_dmg: false,
        soul_revival: false,
        traces_of_silithyst: false,

        // CONSUMES
        flask_of_supreme_power: false,
        infallible_mind: false,
        gift_of_stormwind: false,
        elixir_greater_arcane: false,
        elixir_greater_firepower: false,
        elixir_frost_power: false,
        brilliant_wizard_oil: false,       // "none" | "brilliant" | "blessed"
        blessed_wizard_oil: false,       // "none" | "brilliant" | "blessed"
        very_berry_cream: false,
        runn_tum_tuber: false,             // "none" | "runn_tum_tuber" | ...

        // AURAS (per-player toggles if your UI exposes them)
        atiesh_mage: 0,
        atiesh_warlock: 0,
        moonkin_aura: false,
    };
};

const presetBuffs = () => {
    return {
        // RAID
        arcane_intellect: true,
        imp_mark_of_the_wild: true,
        blessing_of_kings: true,

        // WORLD
        songflower: true,
        rallying_cry: true,
        dire_maul_tribute: true,
        spirit_of_zandalar: true,
        // World snaphot
        dmf_dmg: true,
        soul_revival: false,
        traces_of_silithyst: false,

        // CONSUMES
        flask_of_supreme_power: true,
        infallible_mind: true,
        gift_of_stormwind: false,
        elixir_greater_arcane: true,
        elixir_greater_firepower: true,
        elixir_frost_power: false,
        brilliant_wizard_oil: true,       // "none" | "brilliant" | "blessed"
        blessed_wizard_oil: false,       // "none" | "brilliant" | "blessed"
        very_berry_cream: true,
        runn_tum_tuber: false,             // "none" | "runn_tum_tuber" | ...

        // AURAS (per-player toggles if your UI exposes them)
        atiesh_mage: 0,
        atiesh_warlock: 0,
        moonkin_aura: false,
    };
};

const defaultItems = () => {
    return {
        t2_8p: false,
        t3_6p: false,
        udc: false,
        sapp: false,
        toep: false,
        zhc: false,
        mqg: false,
    };
};


/*
 * Player
 */
const defaultPlayer = () => {
    return {
        name: "Player",
        race: "Undead",
        level: 60,
        berserk: 10,
        id: common.uuid(),        
        stats: common.baseStats("Undead"),
        loadout: common.baseLoadout(),
        buffs: defaultBuffs(),
        pi_count: 1,
        is_target: true,
        is_vary: true,
        // new: one nested buffs object, booleans/strings only
        talents: common.parseWowheadTalents("23000502-5052122123033151-003"),
        items: [],
        apl: presets.apls?.[0] || aplData.getDefaultApl(),
        bonus_stats: common.stats(),        
    };
};
const createPlayer = (name) => {
    let player = defaultPlayer();
    player.name = name;
    activeRaid.value.players.push(player);
};
const visualStats = (player) => {
    let stats = common.displayStats(player);
    stats.mana+= 1213 + stats.int*15 - 280;
    return stats;
};
const activePlayerId = ref(null);
const activePlayer = computed(() => {
    if (!activeRaid.value)
        return null;
    return activeRaid.value.players.find(player => player.id == activePlayerId.value);
});

/*
 * Raid
 */

const defaultRaid = (name) => {
    return {
        id: common.uuid(),
        groupId: "",
        name: "My raid",
        faction: "Horde",
        config: defaultConfig(),
        players: [defaultPlayer()],
        _sync_buffs: false,
    }
};
const loadRaids = () => {
    let raids = window.localStorage.getItem("raids");
    if (raids)
        raids = JSON.parse(raids);

    if (_.isEmpty(raids)) {
        raids = [defaultRaid()];
    }
    else {
        let defRaid = defaultRaid();
        let defPlayer = defaultPlayer();
        // Convert data
        for (let raid of raids) {
            for (let player of raid.players) {
                player.talents.splice(49);
                if (player.hasOwnProperty("extra_stats")) {
                    player.bonus_stats = player.extra_stats;
                    delete player.extra_stats;
                }
                for (let key in defPlayer) {
                    if (!player.hasOwnProperty(key))
                        player[key] = defPlayer[key];
                }
                if (player.apl && player.apl.id) {
                    if (aplData.isPreset(player.apl.id)) {
                        let ap = presets.apls.find(a => a.id == player.apl.id);
                        if (ap)
                            player.apl = _.cloneDeep(ap);
                    }
                } else {
                    // If APL is missing or invalid, set to default
                    player.apl = _.cloneDeep(presets.apls[0]);
                }
            }
            for (let key in defRaid) {
                if (!raid.hasOwnProperty(key))
                    raid[key] = defRaid[key];
            }
            for (let key in defRaid.config) {
                if (!raid.config.hasOwnProperty(key))
                    raid.config[key] = defRaid.config[key];
            }
        }
    }

    return raids;
};
const saveRaids = (raids) => {
    window.localStorage.setItem("raids", JSON.stringify(raids));
};
const raids = ref(loadRaids());
const deleteRaid = (id) => {
    raids.value = raids.value.filter(raid => raid.id != id);
    if (!raids.value.length)
        raids.value.push(defaultRaid());
    if (settings.raid_id == id)
        settings.raid_id = raids.value[0].id;
    saveRaids(raids.value);
};
const activeRaid = computed(() => {
    return raids.value.find(raid => raid.id == settings.raid_id);
});
const getRaidsDeletableInGroup = (groupId) => {
    if (groupId == "") return [];
    
    return raids.value.filter(raid => {
        // Must match the prefix pattern
        if (raid.groupId == groupId) {
            return true;
        }
        
        return false;
    });
};
const deleteRaidGroup = (groupId) => {
    const raidsToDelete = getRaidsDeletableInGroup(groupId);
    const idsToDelete = raidsToDelete.map(raid => raid.id);
    
    raids.value = raids.value.filter(raid => !idsToDelete.includes(raid.id));
    
    if (!raids.value.length)
        raids.value.push(defaultRaid());
    
    if (idsToDelete.includes(settings.raid_id))
        settings.raid_id = raids.value[0].id;
    
    saveRaids(raids.value);
};
const confirmationGroupContinue = () => {
    confirmation.value.groupContinue();
    confirmSpotlight.value.close();
};
/*
 * Settings
 */
const detectedCores = () => {
    return navigator.hardwareConcurrency;
};
const defaultSettings = () => {
    return {
        iterations: 10000,
        threads: detectedCores(),
        raid_id: null,
        normalize_ignite: false,
    }
};
const loadSettings = () => {
    let settings = window.localStorage.getItem("settings");
    if (settings) {
        settings = JSON.parse(settings);
        settings = _.merge(defaultSettings(), settings);
    }
    else {
        settings = defaultSettings();
    }

    if (!settings.raid_id || !raids.value.find(raid => raid.id == settings.raid_id))
        settings.raid_id = raids.value[0].id;

    return settings;
};
const saveSettings = () => {
    window.localStorage.setItem("settings", JSON.stringify(settings));
};
const settings = reactive(loadSettings());

/* Raid Templates */

const templateRaidEdit = ref();
const templateRaidModel = ref({
    templateType: 'preset', // 'preset' or 'existing'
    numMages: 3,
    gearLevel: null,
    sourceRaid: null,
    faction: 'Horde',
    prefix: '',
    encounterDuration: 60
});

const templateRaidNumMageOptions = computed(() => [
    { value: 2, title: '2 Mages' },
    { value: 3, title: '3 Mages' },
    { value: 4, title: '4 Mages' },
    { value: 5, title: '5 Mages' },
    { value: 6, title: '6 Mages' }
]);

const templateRaidGearOptions = computed(() => {
    return presets.loadouts.map(l => ({ value: l.name, title: l.name }));
});

const templateRaidSourceOptions = computed(() => {
    return raids.value.map(r => ({ value: r.id, title: r.name }));
});

const createRaidsFromTemplateOpen = () => {
    templateRaidModel.value = {
        templateType: 'preset',
        numMages: 3,
        gearLevel: presets.loadouts.length > 0 ? presets.loadouts[0].name : null,
        sourceRaid: raids.value.length > 0 ? raids.value[0].id : null,
        faction: 'Horde',
        prefix: '',
        encounterDuration: 60        
    };
    templateRaidEdit.value.open(true);
};

const updateTemplateRaids = () => {
    // Validate encounter duration
    const duration = templateRaidModel.value.encounterDuration;
    if (!duration || duration < 10 || duration > 300) {
        alert("Encounter duration must be between 10 and 300 seconds");
        return;
    }
    
    templateRaidEdit.value.close();
    
    if (templateRaidModel.value.templateType === 'preset') {
        createRaidsFromPreset();
    } else {
        createRaidsFromExisting();
    }
};

const createRaidsFromPreset = () => {
    const numMages = templateRaidModel.value.numMages;
    const gearLevel = templateRaidModel.value.gearLevel;
    const prefix = templateRaidModel.value.prefix;
    const faction = templateRaidModel.value.faction;
    const encounterDuration = templateRaidModel.value.encounterDuration;    
    
    if (!gearLevel) {
        alert("Please select a gear level");
        return;
    }
    
    // Find the preset loadout
    const presetLoadout = presets.loadouts.find(l => l.name === gearLevel);
    if (!presetLoadout) {
        alert("Could not find preset loadout");
        return;
    }
    // take all raids off comparison
    raids.value.forEach(raid => {
        raid.config.in_comparison = false;
    });
   
    const baseRaid = defaultRaid();

    baseRaid.name = `${prefix}`;
    baseRaid.faction = faction;
    baseRaid.config = _.cloneDeep(activeRaid.value.config);
    baseRaid.config.duration = encounterDuration;
    baseRaid.config.duration_variance = 0;
    baseRaid.config.in_comparison = true;
    baseRaid.players = [];
    
    // Create the specified number of mage players
    for (let i = 1; i <= numMages; i++) {
        const player = defaultPlayer();
        player.name = `Mage${i}`;
        player.race = faction === 'Alliance' ? 'Gnome' : 'Undead';
        player.loadout = _.cloneDeep(presetLoadout.loadout);
        player.buffs = _.cloneDeep(presetBuffs());
        if (gearLevel.includes('Era')) {
            player.buffs.atiesh_mage = Math.min(numMages - 1, 4);
            player.buffs.atiesh_warlock = Math.max(Math.min(5 - numMages, 2), 0);
        }
        const stats = common.displayStats(player);
        player.id = common.uuid();
        baseRaid.players.push(player);
    }

    // Generate variations
    const newRaids = generateRaidsFromTemplate(baseRaid, {
        isPreset: true,
        namePrefix: prefix,
        encounterDuration: encounterDuration,
        naxxTrinketAvailability: !gearLevel.includes('Phase 5') && !gearLevel.includes('Phase 6 Enter')
    });
    
    raids.value.push(...newRaids);
    raids.value = _.sortBy(raids.value, "name");
    saveRaids(raids.value);

    // Set the first newly created raid as active (after sorting)
    if (newRaids.length > 0) {
        const firstNewRaid = raids.value.find(raid => 
            newRaids.some(newRaid => newRaid.id === raid.id)
        );
        if (firstNewRaid) {
            settings.raid_id = firstNewRaid.id;
        }
    }    

    notify({
        title: "Success!",
        text: `Created ${newRaids.length} raids with ${numMages} mages each`,
        timer: 3000,
        class: "success"
    });
};

const createRaidsFromExisting = () => {
    const sourceRaidId = templateRaidModel.value.sourceRaid;
    const encounterDuration = templateRaidModel.value.encounterDuration; 

    if (!sourceRaidId) {
        alert("Please select a source raid");
        return;
    }
    
    const sourceRaid = raids.value.find(r => r.id === sourceRaidId);
    if (!sourceRaid) {
        alert("Could not find source raid");
        return;
    }
    // take all raids off comparison
    raids.value.forEach(raid => {
        raid.config.in_comparison = false;
    });
  
    const baseRaid = _.cloneDeep(sourceRaid);
    baseRaid.id = common.uuid();
    baseRaid.name = `${sourceRaid.name}`;
    baseRaid.config.duration = encounterDuration;
    baseRaid.config.duration_variance = 0;
    baseRaid.config.in_comparison = true;
    
    // Give players unique ids and calculate average crit
    baseRaid.players.forEach(player => {
        const stats = common.displayStats(player);
        player.id = common.uuid();
    });

    // Generate variations (e.g., opposite faction, different durations, etc.)
    const newRaids = generateRaidsFromTemplate(baseRaid, {
        isPreset: false,
        namePrefix: sourceRaid.name,
        encounterDuration: encounterDuration
    });    
    
    raids.value.push(...newRaids);
    raids.value = _.sortBy(raids.value, "name");
    saveRaids(raids.value);

    // Set the first newly created raid as active (after sorting)
    if (newRaids.length > 0) {
        const firstNewRaid = raids.value.find(raid => 
            newRaids.some(newRaid => newRaid.id === raid.id)
        );
        if (firstNewRaid) {
            settings.raid_id = firstNewRaid.id;
        }
    }    

    notify({
        title: "Success!",
        text: `Created ${newRaids.length} candidate rotations for ${sourceRaid.name}`,
        timer: 3000,
        class: "success"
    });
};

/*
 * Run simulation
 */
const result = ref(null);
const isRunning = ref(false);
// stats for the simulator -- gear only
const simStats = (player) => {
    let stats = common.stats();
    stats = common.addStats(stats, common.loadoutStats(player.loadout));
    stats = common.addStats(stats, player.bonus_stats);

    return stats;
};
const simLoadoutItems = (loadout) => {
    let simItems = defaultItems();
    if (loadout.trinket1.item_id == items.ids.TRINKET_RESTRAINED_ESSENCE || loadout.trinket2.item_id == items.ids.TRINKET_RESTRAINED_ESSENCE)
        simItems.sapp = true;
    if (loadout.trinket1.item_id == items.ids.TRINKET_TOEP || loadout.trinket2.item_id == items.ids.TRINKET_TOEP)
        simItems.toep = true;
    if (loadout.trinket1.item_id == items.ids.TRINKET_ZHC || loadout.trinket2.item_id == items.ids.TRINKET_ZHC)
        simItems.zhc = true;
    if (loadout.trinket1.item_id == items.ids.TRINKET_MQG || loadout.trinket2.item_id == items.ids.TRINKET_MQG)
        simItems.mqg = true;
    let simSets = common.loadoutSets(loadout);
    if (simSets.hasOwnProperty(items.ids.SET_UDC)) {
        if (simSets[items.ids.SET_UDC] == 3)
            simItems.udc = true;
    }
    if (simSets.hasOwnProperty(items.ids.SET_T3)) {
        if (simSets[items.ids.SET_T3] >= 6)
            simItems.t3_6p = true;
    }
    if (simSets.hasOwnProperty(items.ids.SET_T2)) {
        if (simSets[items.ids.SET_T2] == 8)
            simItems.t2_8p = true;
    }
    return simItems;
};
const simBuffs = (player) => {
    //let buffs = _.cloneDeep(defaultBuffs);
    let buffs = defaultBuffs();
    let faction = common.raceFaction(player.race);

    // check for 21 total
    // raid buffs
    if (player.buffs.arcane_intellect)
        buffs.arcane_intellect = true;
    if (player.buffs.imp_mark_of_the_wild)
        buffs.imp_mark_of_the_wild = true;
    if (player.buffs.blessing_of_kings && faction == "Alliance")
        buffs.blessing_of_kings = true;

    // world buffs
    if (player.buffs.rallying_cry)
        buffs.rallying_cry = true;
    if (player.buffs.spirit_of_zandalar)
        buffs.spirit_of_zandalar = true;
    if (player.buffs.songflower)
        buffs.songflower = true;
    if (player.buffs.dire_maul_tribute)
        buffs.dire_maul_tribute = true;
    // world snapshots
    if (player.buffs.dmf_dmg)
        buffs.dmf_dmg = true;
    if (player.buffs.soul_revival)
        buffs.soul_revival = true;
    if (player.buffs.traces_of_silithyst)
        buffs.traces_of_silithyst = true;

    // consumes
    if (player.buffs.flask_of_supreme_power)
        buffs.flask_of_supreme_power = true;
    if (player.buffs.elixir_greater_firepower)
        buffs.elixir_greater_firepower = true;
    if (player.buffs.elixir_frost_power)
        buffs.elixir_frost_power = true;
    if (player.buffs.elixir_greater_arcane)
        buffs.elixir_greater_arcane = true;
    if (player.buffs.brilliant_wizard_oil)
        buffs.brilliant_wizard_oil = true;
    if (player.buffs.blessed_wizard_oil)
        buffs.blessed_wizard_oil = true;
    if (player.buffs.infallible_mind)
        buffs.infallible_mind = true;
    if (player.buffs.gift_of_stormwind)
        buffs.gift_of_stormwind = true;
    if (player.buffs.very_berry_cream)
        buffs.very_berry_cream = true;
    if (player.buffs.runn_tum_tuber)
        buffs.runn_tum_tuber = true;

    // auras
    if (player.buffs.moonkin_aura)
        buffs.moonkin_aura = true;
    buffs.atiesh_mage = player.buffs.atiesh_mage
    buffs.atiesh_warlock = player.buffs.atiesh_warlock

    return buffs;
};

const simApl = (apl) => {
    apl = _.cloneDeep(apl);
    for (let i=0; i<apl.items.length; i++) {
        if (!apl.items[i].status) {
            apl.items.splice(i, 1);
            i--;
        }
    }
    return apl;
};
const simConfig = (raid = null) => {
    if (!raid) raid = activeRaid.value;
    
    let config = _.cloneDeep(raid.config);
    for (let key in config) {
        if (key.substr(0, 1) == "_")
            delete config[key];
    }

    if (config.targets < 1)
        config.targets = 1;

    if (activeRaid.value.faction == "Horde" && config.judgement_of_wisdom)
        config.judgement_of_wisdom = false;

    config.players = [];
    for (let p of raid.players) {
        let player = _.cloneDeep(defaultPlayer());
        for (var key in player)
            player[key] = _.cloneDeep(p[key]);
        player.stats = simStats(p);
        player.items = simLoadoutItems(p.loadout);
        player.buffs = simBuffs(p);
        player.apl = simApl(p.apl);
        config.players.push(player);
    }

    return config;
};
const runSingle = () => {
    // Single iteration only runs the active raid
    const config = simConfig();
    config.raid_id = activeRaid.value.id;
    config.raid_name = activeRaid.value.name;
    config.is_active_raid = true;
    
    console.log('[App] Running config single iteration with config:', config);

    const sc = new SimContainer(settings.threads, 1, config, r => {
        isRunning.value = false;
        result.value = r;
    }, e => {
        console.error("Error", e);
    });

    isRunning.value = true;
    result.value = null;
    sc.start();
};
const simProgress = reactive({
    dps: 0,
    progress: 0,
});
const runMultiple = () => {
    let iterations = settings.iterations;
    let configs = [];
    const config = simConfig();
    const duration = config.duration;
    const variance = config.duration_variance;
    
    // Multiple iterations run all raids with in_comparison checked
    for (let raid of raids.value) {
        if (raid.config.in_comparison || raid.id == activeRaid.value.id) {
            let config = _.cloneDeep(simConfig(raid));
            config.raid_id = raid.id;
            config.raid_name = raid.name;
            config.duration = duration
            config.duration_variance = variance
            config.is_active_raid = (raid.id === activeRaid.value.id);
            configs.push(config);
        }
    }
    
    if (configs.length === 0) {
        alert("No raids selected for comparison");
        return;
    }
    
    const sc = new SimContainer(settings.threads, settings.iterations, configs, r => {
        isRunning.value = false;
        result.value = r;
    }, e => {
        console.error("Error", e);
    }, p => {
        simProgress.dps = p.dps;
        simProgress.progress = p.iterations / iterations;
    });

    isRunning.value = true;
    result.value = null;
    simProgress.dps = 0;
    simProgress.progress = 0;
    sc.start();
};
/*
 * Combat log
 */
const formatter = (text, key, cl) => {
    let regex = new RegExp(key+"\\[([^\\]]+)\\]", "g");
    return text.replace(regex, "<span class='format-"+cl+"'>$1</span>");
};
const formatTime = (s) => {
    let str = s.toFixed(2);
    return str.length == 4 ? "0"+str : str;
};
const formatLogText = (log) => {
    let text = log.text;
    text = formatter(text, "s", "spell");
    text = formatter(text, "t", "target");
    text = formatter(text, "a", "aura");
    text = formatter(text, "c", "cooldown");
    text = formatter(text, "m", "mana");
    text = text.replace("->", "&#8594;");
    text = text.replace("!=", "&ne;");
    // Remove unknown formatters
    text = text.replace(/[a-z0-9]+\[([^\]]+)\]/g, "$1");
    return text;
};
const formatLogType = (type) => {
    return _.capitalize(_.kebabCase(type).replace(/-/g, " "));
};
const filterPlayer = ref(null);
const filterPlayerOptions = computed(() => {
    if (!result.value)
        return [];
    let options = [];
    for (let player of result.value.players)
        options.push({value: player.name, title: player.name});
    return options;
});
const filteredLog = computed(() => {
    if (!result.value)
        return [];
    let log = result.value.log.filter(log => log.log_type != "Debug");
    if (filterPlayer.value)
        log = log.filter(log => log.unit_name == filterPlayer.value);
    return log;
});


/*
 * Confirmation
 */
const confirmSpotlight = ref();
const confirmation = ref({});
const confirm = (options) => {
    let defaults = {
        text: "Are you sure?",
        confirm: "Yes",
        abort: "No",
        continue: () => {},
        showGroupDelete: false,
        groupDelete: "Delete Group",
        groupContinue: () => {},
    };
    confirmation.value = _.merge(defaults, options);
    confirmSpotlight.value.open();

    return new Promise((resolve, reject) => {
        confirmation.value.continue = resolve;
        confirmation.value.groupContinue = options.groupContinue || (() => {});
    });
};
const confirmationContinue = () => {
    confirmation.value.continue();
    confirmSpotlight.value.close();
};
const confirmationCancel = () => {
    confirmSpotlight.value.close();
};

/*
 * Alert
 */
const alertSpotlight = ref();
const alerter = reactive({
    text: "",
});
const alert = (text) => {
    alerter.text = text;
    alertSpotlight.value.open();
};
const alertClose = () => {
    alertSpotlight.value.close();
};

/*
 * Notification
 */
const notifications = ref([]);
const notify = (obj) => {
    if (!obj.id)
        obj.id = common.uuid();
    notifications.value.push(obj);
    if (obj.timer && obj.timer > 0) {
        setTimeout(() => {
            notifications.value = notifications.value.filter(n => n.id != obj.id);
        }, obj.timer);
    }
};

/*
 * Tutorial
 */
const tutorialSpotlight = ref();
const tutorialHasBeenSeen = ref(false);
const openTutorial = () => {
    if (tutorialSpotlight.value) {
        tutorialHasBeenSeen.value = true;
        tutorialSpotlight.value.open();
    }
};
const closeTutorial = () => {
    if (tutorialSpotlight.value) {
        tutorialSpotlight.value.close();
    }
};

/*
 * Raid UI
 */
const raidSelectOpen = ref(false);
const confirmDeleteRaid = (raid) => {
    raidSelectOpen.value = false;
    
    const deletableRaids = getRaidsDeletableInGroup(raid.groupId);
    const isPartOfGroup = raid.groupId != "" && deletableRaids.length > 1;
    
    confirm({
        text: isPartOfGroup 
            ? `Are you sure you want to delete "${raid.name}"?`
            : `Are you sure you want to delete "${raid.name}"?`,
        confirm: "Delete",
        abort: "Cancel",
        showGroupDelete: isPartOfGroup,
        groupDelete: deletableRaids.length 
            ? `Delete ${deletableRaids.length} raids`
            : `Delete ${deletableRaids.length} raids`,
        groupContinue: () => {
            deleteRaidGroup(raid.groupId);
        }
    }).then(() => {
        deleteRaid(raid.id);
    });
};
const selectRaid = (id) => {
    settings.raid_id = id;
    raidSelectOpen.value = false;
};
const raidEdit = ref();
const raidModel = ref(defaultRaid());
const raidModelCopy = ref(null);
const raidIsImport = ref(false);
const raidCopyOptions = computed(() => {
    if (!activeRaid.value)
        return [];
    let options = [];
    for (let raid of raids.value) {
        if (raid.id == raidModel.value.id)
            continue;
        options.push({value: raid.id, title: raid.name});
    }
    return options;
});
const createRaidOpen = () => {
    raidModel.value = defaultRaid();
    raidModel.value.name = "";
    raidSelectOpen.value = false;
    raidIsImport.value = false;
    raidEdit.value.open(true);
};
const updateRaid = () => {
    raidEdit.value.close();
    let raid = _.cloneDeep(raidModel.value);

    if (raidModelCopy.value) {
        let copy = raids.value.find(r => r.id == raidModelCopy.value);
        if (copy) {
            raid = _.cloneDeep(copy);
            raid.id = raidModel.value.id;
            raid.name = raidModel.value.name;
            raid.faction = raidModel.value.faction;
            if (raid.faction != copy.faction) {
                for (let player of raid.players)
                    player.race = convertRace(player.race);
            }
        }
    }

    let index = raids.value.findIndex(r => r.id == raid.id);
    if (index != -1) {
        if (raids.value[index].faction != raid.faction) {
            for (let player of raid.players)
                player.race = convertRace(player.race);
        }
        raids.value[index] = raid;
    }
    else {
        raids.value.push(raid);
        settings.raid_id = raid.id;
    }

    for (let player of raid.players) {
        if (common.raceFaction(player.race) != raidModel.value.faction)
            player.race = convertRace(player.race);
        if (player.custom_items && player.custom_items.length) {
            for (let item of player.custom_items) {
                let index = customItems.value.findIndex(ci => ci.id == item.id);
                if (index == -1)
                    customItems.value.push(item);
                else
                    customItems.value.splice(index, 1, item);
            }
            saveCustomItems(customItems.value);
        }
    }

    raids.value = _.sortBy(raids.value, "name");
};
const onRaidNameChange = () => {
    if (activeRaid.value) {
        activeRaid.value.groupId = "";
    }
};
const factionOptions = [
    { value: "Alliance", title: "Alliance" },
    { value: "Horde", title: "Horde" },
];
const bossOptions = [
    { value: "None", title: "None" },
    { value: "Loatheb", title: "Loatheb" },
    { value: "Thaddius", title: "Thaddius" },
];

/*
 * Player UI
 */
const playerStats = ref(common.stats());
const playerEdit = ref();
const playerImport = ref();
const playerModel = ref(defaultPlayer());
const playerModelCopy = ref(null);
const playerImportConfig = ref(true);
const playerImportLoadout = ref(true);
const selectPlayer = (id) => {
    activePlayerId.value = id;
};
const otherPlayerOptions = computed(() => {
    if (!activeRaid.value)
        return {};
    let options = [];
    for (let player of activeRaid.value.players) {
        if (player.id == activePlayerId.value)
            continue;
        options.push({value: player.id, title: player.name});
    }
    return options;
});
const confirmDeletePlayer = (player) => {
    raidSelectOpen.value = false;
    confirm({
        text: "Are you sure you want to delete "+player.name+"?",
        confirm: "Delete",
        abort: "Cancel",
    }).then(() => {
        activeRaid.value.players = activeRaid.value.players.filter(p => p.id != player.id);
        if (activePlayerId.value == player.id) {
            if (activeRaid.value.players.length)
                activePlayerId.value = activeRaid.value.players[0].id;
            else
                activePlayerId.value = null;
        }
    });
};
const playerCopyOptions = computed(() => {
    if (!activeRaid.value)
        return [];
    let options = [];
    for (let player of activeRaid.value.players) {
        if (player.id == playerModel.value.id)
            continue;
        options.push({value: player.id, title: player.name});
    }
    return options;
});
const createPlayerOpen = () => {
    if (!activeRaid.value)
        return;
    playerModel.value = defaultPlayer();
    playerModel.value.name = "";
    playerModel.value.race = activeRaid.value.faction == "Alliance" ? "Gnome" : "Undead";
    playerEdit.value.open(true);
};
const updatePlayer = () => {
    let isImport = playerImport.value.isOpen;
    playerEdit.value.close();
    playerImport.value.close();
    if (!activeRaid.value)
        return;

    let player = null;
    if (isImport) {
        if (playerModelCopy.value) {
            let copy = activeRaid.value.players.find(p => p.id == playerModelCopy.value);
            if (copy) {
                if (playerImportConfig.value) {
                    player = _.cloneDeep(playerModel.value);
                    player.id = copy.id;
                    player.name = copy.name;
                    if (!playerImportLoadout.value) {
                        player.loadout = _.cloneDeep(copy.loadout);
                        delete player.custom_items;
                    }
                }
                else {
                    player = _.cloneDeep(copy);
                    if (playerImportLoadout.value) {
                        player.loadout = _.cloneDeep(playerModel.value.loadout);
                        player.custom_items = _.cloneDeep(playerModel.value.custom_items);
                    }
                }
            }
        }
    }
    else {
        if (playerModelCopy.value) {
            let copy = activeRaid.value.players.find(p => p.id == playerModelCopy.value);
            if (copy) {
                player = _.cloneDeep(copy);
                player.id = playerModel.value.id;
                player.name = playerModel.value.name;
                player.race = playerModel.value.race;
            }
            playerModelCopy.value = null;
        }
    }

    if (!player)
        player = _.cloneDeep(playerModel.value);

    if (player.custom_items && player.custom_items.length) {
        for (let item of player.custom_items) {
            let index = customItems.value.findIndex(ci => ci.id == item.id);
            if (index == -1)
                customItems.value.push(item);
            else
                customItems.value.splice(index, 1, item);
        }
        saveCustomItems(customItems.value);
        delete player.custom_items;
    }

    let index = activeRaid.value.players.findIndex(r => r.id == player.id);
    if (index != -1)
        activeRaid.value.players[index] = player;
    else
        activeRaid.value.players.push(player);

    syncBuffs();

    activePlayerId.value = player.id;
};
const specFromTalents = (talents) => {
    let count = [0, 0, 0];
    for (let i = 0; i < talents.length; i++) {
        if (talents[i] > 0)
            count[Math.floor(i / 16)]++;
    }

    let max = 0, tree = 0;
    for (let i = 0; i < count.length; i++) {
        if (count[i] > max) {
            max = count[i];
            tree = i;
        }
    }

    let trees = ["arcane", "fire", "frost"];

    return trees[tree];
};
const playerSpecIcon = (player) => {
    return "spec_"+specFromTalents(player.talents);
};
const raceOptions = computed(() => {
    if (activeRaid.value && activeRaid.value.faction == "Alliance") {
        return [
            { value: "Gnome", title: "Gnome" },
            { value: "Human", title: "Human" },
        ];
    }

    return [
        { value: "Troll", title: "Troll" },
        { value: "Undead", title: "Undead" },
    ];
});
const berserkOptions = computed(() => {
    const options = [];
    for (let i = 10; i <= 30; i += 2) {
        options.push({ value: i, title: `${i}%` });
    }
    return options;
});
const talentPreset = ref(null);
const talentPresetOptions = computed(() => {
    return presets.talents.map(t => { return {title: t.name, value: t.talents}; });
});
const setTalentPreset = () => {
    if (!activePlayer.value || !talentPreset.value)
        return;
    activePlayer.value.talents = talentPreset.value;
    nextTick(() => { talentPreset.value = null; });
};

/*
 * Main panel UI
 */
const activeTab = ref("config");
const activeSlot = ref("head");
const activeGearType = ref("gear");
const activeResultTab = ref("overview");

/*
 * Config UI
 */
const playerConfigExclusive = (e, key, others) => {
    if (!_.isArray(others))
        others = [others];

    if (e.target.checked) {
        for (let other of others) {
            if (other == key)
                continue;
            activePlayer.value.buffs[other] = false;
        }
    }
};
const cycleCount = (field) => {
    if (activePlayer.value?.buffs?.atiesh_mage === undefined) { return;}
    const max = 4;
    const total = activePlayer.value.buffs.atiesh_mage + activePlayer.value.buffs.atiesh_warlock + Number(activePlayer.value.buffs.moonkin_aura);
    const cur  = Number(activePlayer.value.buffs[field] ?? 0);
    if (field === "power_infusion") {
        const max_pi = 4;
        if (activePlayer.value.pi_count < max_pi)
            activePlayer.value.pi_count += 1;
        else
            activePlayer.value.pi_count = 0;
   }
   else if (field != "moonkin_aura") {
        if (total < max) {
            activePlayer.value.buffs[field] = cur + 1;
        }
        else {
            activePlayer.value.buffs[field] = 0;
        }
    }
    else {
        if (cur > 0) {
            activePlayer.value.buffs[field] = false;
        }
        else {
            if (total < max) {
                activePlayer.value.buffs[field] = true;
            }
        }
    }
}
const showTip = () => {
    return false;
}
const talentImport = ref("");
const importTalents = () => {
    let talents = common.parseTalents(talentImport.value);
    if (talents)
        activePlayer.value.talents = talents;
    else
        alert("Could not parse talent URL");
    talentImport.value = "";
};

const onSyncBuffs = () => {
    syncBuffs();
};
const syncBuffs = () => {
    if (!activeRaid.value._sync_buffs)
        return;
    let skip = [
        "moonkin_aura", "atiesh_mage", "atiesh_warlock"
    ];
    for (let player of activeRaid.value.players) {
        if (player.id == activePlayer.value.id)
            continue;
        for (let key in activePlayer.value.buffs) {
            if (skip.includes(key))
                continue;
            player.buffs[key] = activePlayer.value.buffs[key];
        }
    }
};

/*
 * Item UI
 */
const paperdollSlots = (pos) => {
    if (pos == "left") {
        return [
            "head", "neck", "shoulder",
            "back", "chest", "wrist", "hands",
        ];
    }
    if (pos == "right") {
        return [
            "waist", "legs", "feet",
            "finger1", "finger2", "trinket1", "trinket2",
        ];
    }
    if (pos == "bottom") {
        return [
            "main_hand", "off_hand", "ranged",
        ];
    }
};
const paperdollClick = (slot, type) => {
    activeSlot.value = slot;
    activeGearType.value = type ? type : "gear";
    nextTick(() => {
        if (itemSearchInput.value)
            itemSearchInput.value.focus();
    });
};
const loadoutOptions = computed(() => {
    return [...otherPlayerOptions.value, ...presets.loadouts.map(l => { return {value: l.name, title: "Preset: "+l.name}})];
});
const itemSearch = ref("");
const itemSearchInput = ref();
const itemSorting = ref({
    name: "ilvl",
    order: "desc",
});
const itemSort = (items, sorting) => {
    if (!sorting || !sorting.name)
        return items;

    let type = null;
    for (let i=0; i<items.length; i++) {
        let value = _.get(items[i], sorting.name, null);
        if (value !== null) {
            type = typeof(value);
            if (type == "object") {
                if (_.isArray(value))
                    type = "array";
                else
                    continue;
            }
            break;
        }
    }

    if (type === null)
        return items;

    return items.sort((a, b) => {
        let av = _.get(a, sorting.name, null);
        let bv = _.get(b, sorting.name, null);
        let ac = common.isCustomItem(a);
        let bc = common.isCustomItem(b);
        let result = 0;

        if (ac && !bc) return -1;
        if (!ac && bc) return 1;

        if (sorting.name == "phase") {
            if (!av) av = 1;
            if (!bv) bv = 1;
        }

        if (type == "string") {
            try { av = av.toString(); } catch(e) { av = ""; };
            try { bv = bv.toString(); } catch(e) { bv = ""; };
            result = av.localeCompare(bv);
        }
        else if (type == "number") {
            av = parseFloat(av);
            bv = parseFloat(bv);
            if (isNaN(av)) av = 0;
            if (isNaN(bv)) bv = 0;
            result = av - bv;
        }
        else if (type == "array") {
            av = _.get(av, "length", 0);
            bv = _.get(bv, "length", 0);
            result = av - bv;
        }

        if (sorting.order == "desc" && result != 0)
            result = result < 0 ? 1 : -1;

        // if (result == 0)
        //     result = a.title.localeCompare(b.title);

        return result;
    });
};
const itemList = computed(() => {
    let data = {
        type: activeGearType.value,
        slot: common.loadoutSlotToItemSlot(activeSlot.value),
        list: [],
    };

    if (data.type == "enchant") {
        data.list = items.enchants[data.slot];
    }
    else {
        data.list = items.gear[data.slot];
        let custom = customItems.value.filter(ci => ci.slot == data.slot);
        if (custom.length)
            data.list = [...custom, ...data.list];
    }

    let faction_str = activeRaid.value.faction.toLowerCase().substr(0, 1);
    data.list = data.list.filter(item => {
        if (itemSearch.value.length) {
            if (item.title.toLowerCase().indexOf(itemSearch.value.toLowerCase()) == -1)
                return false;
        }
        if (item.hasOwnProperty("faction") && item.faction != faction_str)
            return false;
        return true;
    });

    data.list = itemSort(data.list, itemSorting.value);

    return data;
});
const itemSearchEnter = () => {
    if (itemList.value.list.length)
        itemClick(itemList.value.list[0]);
};
const itemSearchUp = () => {
    if (!itemList.value.list.length || !activePlayer.value || !activeSlot.value)
        return;
    let current = activePlayer.value.loadout[activeSlot.value].item_id;
    if (!current)
        return;
    let index = itemList.value.list.findIndex(i => i.id == current);
    activePlayer.value.loadout[activeSlot.value].item_id = itemList.value.list[(index+itemList.value.list.length-1)%itemList.value.list.length].id;
};
const itemSearchDown = () => {
    if (!itemList.value.list.length || !activePlayer.value || !activeSlot.value)
        return;
    let current = activePlayer.value.loadout[activeSlot.value].item_id;
    if (!current)
        return itemSearchEnter();
    let index = itemList.value.list.findIndex(i => i.id == current);
    activePlayer.value.loadout[activeSlot.value].item_id = itemList.value.list[(index+1)%itemList.value.list.length].id;
};
const itemClick = (item) => {
    if (!activePlayer.value || !activeSlot.value)
        return;

    let loadout = activePlayer.value.loadout[activeSlot.value];
    let key = activeGearType.value == "enchant" ? "enchant_id" : "item_id";
    if (loadout[key] == item.id) {
        loadout[key] = null;
    }
    else {
        if (key == "item_id" && activeSlot.value == "off_hand") {
            let mh = common.getItem("main_hand", activePlayer.value.loadout["main_hand"].item_id);
            if (mh.twohand)
                return;
        }
        else if (key == "item_id" && activeSlot.value == "main_hand" && item.twohand) {
            activePlayer.value.loadout["off_hand"].item_id = null;
        }

        if (item.unique) {
            let other = otherSlot(activeSlot.value);
            if (other && activePlayer.value.loadout[other].item_id) {
                if (item.unique === true) {
                    if (activePlayer.value.loadout[other].item_id == item.id)
                        return;
                }
                // Unique category
                else {
                    let otherItem = common.getItem(other, activePlayer.value.loadout[other].item_id);
                    if (otherItem && otherItem.unique && otherItem.unique === item.unique)
                        return;
                }
            }
        }

        loadout[key] = item.id;
    }

    refreshTooltips();
};
const copyLoadoutPlayer = ref(null);
const copyLoadout = (source) => {
    if (!activePlayer.value)
        return;
    let loadout = null;
    let preset = presets.loadouts.find(l => l.name == source);
    if (preset) {
        loadout = _.cloneDeep(preset.loadout);
    }
    else {
        let player = activeRaid.value.players.find(p => p.id == source);
        if (player)
            loadout = _.cloneDeep(player.loadout);
    }
    if (loadout) {
        activePlayer.value.loadout = loadout;
        refreshTooltips();
    }
    nextTick(() => { copyLoadoutPlayer.value = null });
};

const loadCustomItems = () => {
    let data = window.localStorage.getItem("custom_items");
    if (data)
        data = JSON.parse(data);
    return data ? data : [];
};
const saveCustomItems = (data) => {
    window.localStorage.setItem("custom_items", JSON.stringify(data));
};
const customItems = ref(loadCustomItems());
const customItemModel = ref(null);
const editCustomItem = ref();
const missingItem = ref();
const missingItemOpen = () => {
    if (!missingItem.value)
        return;
    missingItem.value.open(true);
};
const createCustomItemOpen = () => {
    if (!editCustomItem.value || !activeSlot.value)
        return;
    if (missingItem.value)
        missingItem.value.close();
    customItemModel.value = {
        is_new: true,
        id: "custom_"+common.uuid(),
        slot: activeSlot.value,
        title: "",
        ilvl: 1,
        sp: null,
        sp_arcane: null,
        sp_fire: null,
        sp_frost: null,
        int: null,
        spi: null,
        mp5: null,
        hit: null,
        crit: null,
        q: "epic",
        twohand: false,
    };
    editCustomItem.value.open(true);
};
const editCustomItemOpen = (item) => {
    if (!editCustomItem.value)
        return;
    customItemModel.value = _.cloneDeep(item);
    editCustomItem.value.open(true);
};
const updateCustomItem = () => {
    if (!customItemModel.value)
        return;
    delete customItemModel.value.is_new;
    let index = customItems.value.findIndex(ci => ci.id == customItemModel.value.id);
    if (index === -1)
        customItems.value.push(customItemModel.value);
    else
        customItems.value.splice(index, 1, customItemModel.value);
    saveCustomItems(customItems.value);
    if (editCustomItem.value)
        editCustomItem.value.close();
};
const deleteCustomItem = (item) => {
    confirm({
        text: "Do you want to delete "+item.title+"?",
    }).then(() => {
        customItems.value = customItems.value.filter(ci => ci.id != item.id);
        unequipFromAllPlayers(item.id);
    });
};

/*
 * APL UI
 */
const loadApls = () => {
    let apls = window.localStorage.getItem("apls");
    if (apls)
        apls = JSON.parse(apls);
    if (_.isEmpty(apls))
        apls = [];
    return apls;
};
const saveApls = (data) => {
    // Filter out any APLs with null/undefined IDs or preset IDs
    const validApls = data.filter(a => {
        if (!a || !a.id) return false;
        // Check if it's a preset
        return !aplData.isPreset(a.id);
    });
    window.localStorage.setItem("apls", JSON.stringify(validApls));
};
const selectApl = (data) => {
    if (!activePlayer.value || !data)
        return;
    // Deep clone to prevent reference sharing between players
    activePlayer.value.apl = _.cloneDeep(data);
};
const apls = ref(loadApls());
const editApl = ref();
const aplModel = ref(null);
const aplTargetOptions = computed(() => {
    return apls.value.map(a => { return {value: a.id, title: a.name}; });
});
const copyPlayerApl = () => {
    if (!playerModelCopy.value || !activePlayer.value)
        return;
    let player = activeRaid.value.players.find(p => p.id == playerModelCopy.value);
    if (!player)
        return;
    activePlayer.value.apl = _.cloneDeep(player.apl);
    nextTick(() => { playerModelCopy.value = null; });
};
const editAplOpen = () => {
    if (!activePlayer.value || !activePlayer.value.apl) {
        console.error('No active player or APL');
        return;
    }
    
    aplModel.value = _.cloneDeep(activePlayer.value.apl);
    
    // Ensure the model has an ID
    if (!aplModel.value.id) {
        aplModel.value.id = common.uuid();
    }
    
    if (aplData.isPreset(aplModel.value.id)) {
        if (aplModel.value.name == "Blank")
            aplModel.value.name = "";
        else
            aplModel.value.name += " copy";
        aplModel.value.id = common.uuid();
    }
    
    if (editApl.value)
        editApl.value.open(true);
};
const deleteApl = (id) => {
    if (aplData.isPreset(id))
        return;
    apls.value = apls.value.filter(a => a.id != id);
    saveApls(apls.value);
};
const updateApl = () => {
    if (!aplModel.value)
        return;

    // Ensure the APL has a valid ID
    if (!aplModel.value.id) {
        aplModel.value.id = common.uuid();
    }
    
    // If it's a preset being saved as new, give it a new ID
    if (aplData.isPreset(aplModel.value.id)) {
        aplModel.value.id = common.uuid();
    }

    let data = _.cloneDeep(aplModel.value);
    let index = _.findIndex(apls.value, {id: data.id});
    if (index == -1)
        apls.value.push(data);
    else
        apls.value.splice(index, 1, data);

    if (activePlayer.value && activePlayer.value.apl.id == data.id)
        activePlayer.value.apl.name = data.name;

    saveApls(apls.value);
    if (editApl.value)
        editApl.value.close();
};

// Auto-build rotation options
const autoBuildOptions = computed({
    get() {
        if (!activePlayer.value) {
            return {
                preScorch: '',
                bufferSpell: '',
                derivedOpening: '',
                sustain: '',
                playerTrinkets: [0, 0]
            };
        }
        
        // Initialize auto-build options if they don't exist for this player
        if (!activePlayer.value.autoBuildOptions) {
            activePlayer.value.autoBuildOptions = {
                preScorch: '',
                bufferSpell: '',
                derivedOpening: '',
                sustain: '',
                playerTrinkets: [0, 0]
            };
        }
        
        return activePlayer.value.autoBuildOptions;
    },
    set(value) {
        if (activePlayer.value) {
            activePlayer.value.autoBuildOptions = value;
        }
    }
});
const talentNames = talentTree.trees.reduce((a, b) => { return [...a, ...b.talents.rows.flat()]; }, []).map(t => t.name);
const autofillRotation = () => {
    const is_arcane = activePlayer.value.talents[talentNames.indexOf("arcane_power")] > 0;
    const duration = activeRaid.value.config.duration;
    
    // pre scorch
    let preScorch = "";
    if (is_arcane) preScorch = "ap-fire";
    else if (duration < 25.0) preScorch = "no-scorch"

    // buffer spell
    let bufferSpell = "";
    if (preScorch == "") {
        if (duration < 35.0) bufferSpell = "gcd";
        else bufferSpell = "pyro";
    }

    // trinket business
    const trinketResult = getDerivedTrinkets(activePlayer.value.loadout.trinket1.item_id, activePlayer.value.loadout.trinket2.item_id);
    let derivedOpening = trinketResult.derived;
    let playerTrinkets = trinketResult.trinkets;

    // sustain
    let crits = [];
    let scorchRanks = [];
    activeRaid.value.players.forEach(player => {
        const stats = common.displayStats(player);
        const effectiveCrit = (Math.min(stats.hit, 10.0) + 89.0)*stats.crit/99.0;
        crits.push(effectiveCrit);
        scorchRanks.push(stats.sp*scorchPerSP + effectiveCrit*scorchPerCrit);
    });
    const averageCrit = crits.reduce((sum, num) => sum + num, 0) / crits.length;
    const scorchRank = scorchRanks.map((value, index) => 
        scorchRanks.filter((v, i) => v > value || (v === value && i < index)).length
    );
    let sustainPermutation = getSustainPermutationsWrapper(averageCrit, activeRaid.value.players.length, duration, preScorch)[0];
    const playerIndex = activeRaid.value.players.findIndex(player => player.id === activePlayer.value.id);
    let sustain = getSustain(scorchRank[playerIndex], sustainPermutation);
    if (sustainPermutation == "" && scorchRank[playerIndex] == 0 && preScorch == "") {
        sustain = "maintain";
    }

    // For now, just set some default values as an example
    const newOptions = {
        preScorch: preScorch,
        bufferSpell: bufferSpell,
        derivedOpening: derivedOpening,
        sustain: sustain,
        playerTrinkets: playerTrinkets
    };
    activePlayer.value.autoBuildOptions = newOptions;
};

const setRotationFromAutoBuild = () => {
    // TODO: Call getPlayerApl with the selected options and set activePlayer.apl
    const autoBuildOptions = activePlayer.value.autoBuildOptions;
    let sustain = autoBuildOptions.sustain;
    let rank0 = false;
    
    if (sustain == "maintain") {
        rank0 = true;
        sustain = "";
    }
    const pi_count = activePlayer.value.pi_count;

    let derivedOpening = autoBuildOptions.derivedOpening;
    let playerTrinkets = autoBuildOptions.playerTrinkets;
    if (autoBuildOptions.derivedOpening == "") {
        // trinket business (again)
        const trinketResult = getDerivedTrinkets(activePlayer.value.loadout.trinket1.item_id, activePlayer.value.loadout.trinket2.item_id);
        derivedOpening = trinketResult.derived;
        playerTrinkets = trinketResult.trinkets;
    }

    const playerApl = getPlayerApl(autoBuildOptions.preScorch,
                                autoBuildOptions.bufferSpell,
                                derivedOpening,
                                sustain,
                                playerTrinkets,
                                pi_count,
                                rank0,
                                activeRaid.value.players.length);    
    activePlayer.value.apl = _.cloneDeep(playerApl);
};

/*
 * Export keys
 * We use these to make exports smaller by removing the keys and storing the values as arrays
 *
 * DO NOT CHANGE ORDER, REMOVE KEYS, OR ADD KEYS IN THE MIDDLE OF THE ARRAYS
 * ONLY ADD KEYS TO THE END
 * OTHERWISE IMPORTS WILL BREAK
 */
const raidExportKeys = () => {
    return [
        "name", "faction", "config", "players", "_sync_buffs",
    ];
};
const configExportKeys = () => {
    return [
        "rng_seed", "duration", "duration_variance", "avg_spell_dmg",
        "target_level", "target_resistance", "targets", "distance",
        "reaction_time", "initial_delay", "continuing_delay", "boss",
        "curse_of_elements", "curse_of_shadows", "judgement_of_wisdom",
    ];
};
const playerExportKeys = () => {
    return [
        "name", "race", "level", "apl", "buffs",
        "bonus_stats", "talents", "loadout",
    ];
};
const BuffExportKeys = () => {
    return [
        "dmf_dmg", "soul_revival", "traces_of_silithyst",
        "arcane_intellect", "imp_mark_of_the_wild", "moonkin_aura",
        "flask_of_supreme_power", "very_berry_cream",
        "blessing_of_kings", "atiesh_mage", "atiesh_warlock", "infallible_mind",
        "gift_of_stormwind", "songflower", "rallying_cry", "spirit_of_zandalar",
        "dire_maul_tribute", "elixir_greater_firepower", "elixir_greater_arcane",
        "brilliant_wizard_oil", "blessed_wizard_oil", "runn_tum_tuber", "elixir_frost_power"
    ];
};
const aplExportKeys = () => {
    return ["type", "version", "items"];
};
const aplItemExportKeys = () => {
    return ["condition", "action", "status"];
};
const aplConditionExportKeys = () => {
    return ["condition_type", "op", "conditions", "values"];
};
const aplActionExportKeys = () => {
    // Add target id if we add interface for it
    // return ["key", "sequence", "target_id"];
    return ["key", "sequence"];
};
const aplValueExportKeys = () => {
    return ["value_type", "vstr", "vfloat", "vint"];
};
const exportSerialize = (keys, data, strict) => {
    if (!data.hasOwnProperty("x"))
        data.x = [];
    for (let key of keys) {
        let value = _.get(data, key, 0);
        if (value === true)
            value = 1;
        else if (value === false)
            value = 0;
        data.x.push(value);
        delete data[key];
    }
    if (strict) {
        for (let key in data) {
            if (key != "x")
                delete data[key];
        }
    }
};
const importDeserialize = (keys, data, ref) => {
    const objValue = (k, v) => {
        if (ref.hasOwnProperty(k)) {
            if (v === undefined)
                return ref[k];
            if (ref[k] === false || ref[k] === true)
                return v === 1 || v === true;
        }
        return v;
    };

    if (data.hasOwnProperty("x")) {
        for (let i in data.x) {
            let k = keys[i];
            data[k] = objValue(k, data.x[i]);
        }
        delete data.x;
    }

    for (let key in ref) {
        if (!data.hasOwnProperty(key))
            data[key] = ref[key];
    }

    return data;
};

/*
 * Export UI
 */
const importMessage = ref(null);
const exportType = ref("raid");
const exportTypeOptions = computed(() => {
    let options = [{value: "raid", title: "Raid"}];
    if (activePlayer.value)
        options.push({value: "player", title: "Player"});
    return options;
});
const statsExportData = (stats) => {
    let data = [];
    let keys = _.keys(common.stats());
    for (let key of keys)
        data.push(_.get(stats, key, 0));
    return data;
};
const statsImportData = (data) => {
    let stats = common.stats();
    let keys = _.keys(common.stats());
    for (let i of data)
        stats[keys[i]] = data[i];
    return stats;
};
const loadoutExportData = (loadout) => {
    loadout = _.cloneDeep(loadout);
    for (let key in loadout) {
        if (loadout[key].item_id) {
            loadout[key] = [loadout[key].item_id, loadout[key].enchant_id ? loadout[key].enchant_id : 0];
            if (common.isCustomItem(loadout[key][0]))
                loadout[key].push(customItems.value.find(ci => ci.id == loadout[key][0]));
        }
        else {
            loadout[key] = [0,0];
        }
    }
    let data = [];
    for (let key of common.loadoutSlots())
        data.push(loadout[key]);
    return data;
};
const loadoutImportData = (data) => {
    let loadout = common.baseLoadout();
    let slots = common.loadoutSlots();
    for (let i in slots) {
        loadout[slots[i]] = {
            item_id: (data[i][0] ? data[i][0] : null),
            enchant_id: (data[i][1] ? data[i][1] : null),
        };
    }
    return loadout;
};
const talentExportData = (talents) => {
    let str = "";
    for (let i in talents) {
        if (i%16 == 0 && i != 0 && i < 33)
            str+= "-";
        str+= talents[i];
    }
    str = str.replace(/[0]+-/, "-");
    str = str.replace(/[0]+$/, "");
    str = str.replace(/[-]+$/, "");
    return str;
};
const talentImportData = (talents) => {
    return common.parseWowheadTalents(talents);
};
const aplExportData = (data) => {
    let minimize = (obj) => {
        delete obj.id;
        delete obj.title;
        if (obj.condition) {
            minimize(obj.condition);
            exportSerialize(aplConditionExportKeys(), obj.condition);
        }
        if (obj.action) {
            minimize(obj.action);
            exportSerialize(aplActionExportKeys(), obj.action, true);
        }
        if (obj.items)
            obj.items.forEach((o) => { minimize(o); exportSerialize(aplItemExportKeys(), o); });
        if (obj.conditions)
            obj.conditions.forEach((o) => { minimize(o); exportSerialize(aplConditionExportKeys(), o); });
        if (obj.actions)
            obj.actions.forEach((o) => { minimize(o); exportSerialize(aplActionExportKeys(), o, true); });
        if (obj.sequence)
            obj.sequence.forEach((o) => { minimize(o); exportSerialize(aplActionExportKeys(), o, true); });
        if (obj.values)
            obj.values.forEach((o) => { minimize(o); exportSerialize(aplValueExportKeys(), o); });
    };

    if (aplData.isPreset(data.id)) {
        return {id: data.id};
    }
    else {
        data = _.cloneDeep(data);
        delete data.name;
        minimize(data);
        exportSerialize(aplExportKeys(), data);
        return data;
    }
};
const aplImportData = (data) => {
    let restore = (obj) => {
        if (!obj.id)
            obj.id = common.uuid();
        if (obj.condition) {
            importDeserialize(aplConditionExportKeys(), obj.condition, aplData.condition());
            restore(obj.condition);
        }
        if (obj.action) {
            importDeserialize(aplActionExportKeys(), obj.action, aplData.action());
            restore(obj.action);
        }
        if (obj.items)
            obj.items.forEach((o) => { importDeserialize(aplItemExportKeys(), o, aplData.item()); restore(o); });
        if (obj.conditions)
            obj.conditions.forEach((o) => { importDeserialize(aplConditionExportKeys(), o, aplData.condition()); restore(o); });
        if (obj.actions)
            obj.actions.forEach((o) => { importDeserialize(aplActionExportKeys(), o, aplData.getAction(o[0])); restore(o); });
        if (obj.sequence)
            obj.sequence.forEach((o) => { importDeserialize(aplActionExportKeys(), o, aplData.getAction(o[0])); restore(o); });
        if (obj.values)
            obj.values.forEach((o) => { importDeserialize(aplValueExportKeys(), o, aplData.value()); restore(o); });
    };

    data = _.cloneDeep(data);

    if (data.id) {
        let ap = presets.apls.find(a => a.id == data.id);
        if (ap)
            return ap;
    }

    importDeserialize(aplExportKeys(), data, aplData.apl());
    restore(data);

    return data;
};
const exportPlayerData = (player) => {
    player = _.cloneDeep(player);
    player.bonus_stats = statsExportData(player.bonus_stats);
    player.talents = talentExportData(player.talents);
    player.loadout = loadoutExportData(player.loadout);
    player.apl = aplExportData(player.apl);
    delete player.id;
    delete player.stats;
    delete player.items;
    exportSerialize(playerExportKeys(), player);
    return player;
};
const importPlayerData = (original) => {
    let data = _.cloneDeep(original);
    importDeserialize(playerExportKeys(), data, defaultPlayer());
    data.loadout = loadoutImportData(data.loadout);
    data.bonus_stats = statsImportData(data.bonus_stats);
    data.talents = talentImportData(data.talents);
    data.apl = aplImportData(data.apl);

    let custom = [];
    let key = playerExportKeys().findIndex(k => k == "loadout");
    for (let slot in original.x[key]) {
        if (original.x[key][slot].length >= 3)
            custom.push(original.x[key][slot][2]);
    }
    if (custom.length)
        data.custom_items = custom;

    return data;
};
const exportRaidData = (raid) => {
    raid = _.cloneDeep(raid);
    for (let p in raid.players)
        raid.players[p] = exportPlayerData(raid.players[p]);
    exportSerialize(configExportKeys(), raid.config);
    exportSerialize(raidExportKeys(), raid);
    delete raid.id;
    return raid;
};
const importRaidData = (data) => {
    data = _.cloneDeep(data);
    importDeserialize(raidExportKeys(), data, defaultRaid());
    data.config = importDeserialize(configExportKeys(), data.config, defaultConfig());

    for (let p in data.players)
        data.players[p] = importPlayerData(data.players[p]);

    return data;
};
const exportSuccess = ref(false);
const exportSubmit = () => {
    exportSuccess.value = false;

    let data = {
        exp: exportType.value,
        v: "1.0",
        data: null,
    };

    if (exportType.value == "raid") {
        data.data = exportRaidData(activeRaid.value);
    }
    else if (exportType.value == "player" && activePlayer.value) {
        data.data = exportPlayerData(activePlayer.value);
    }
    else {
        alert("Invalid export type");
        return;
    }

    data = compressToEncodedURIComponent(JSON.stringify(data));
    data = window.location.origin+"#mse="+data;
    copyToClipboard(data);
    nextTick(() => { exportSuccess.value = true; });
};
const importData = ref("");
const importSubmit = () => {
    if (importString(importData.value)) {
        importData.value = "";
    }
};
const importString = (str) => {
    if (!str.length)
        return;

    let type = null;
    let data = null;
    try {
        data = JSON.parse(str);
        if (data.phase)
            type = "60up";
        else if (data.gear && data.gear.items)
            type = "wse";
    }
    catch (e) {
        let m = str.match(/https\:\/\/(vanilla|sod)\.warcraftlogs\.com\/reports\/([a-z0-9]+)/i);
        if (m) {
            type = "wcl";
            data = m[2];
        }
        else {
            type = "native";
            data = str;
        }
    }

    // TODO: External import sources
    try {
        if (type == "60up") {
            importSixtyUpgrades(data);
            return true;
        }
        else if (type == "wse") {
            importWSE(data);
            return true;
        }
        else if (type == "wcl") {

        }
        else if (type == "native") {
            importNativeString(str);
            return true;
        }
    }
    catch(e) {
        console.error(e);
    }

    // No matching imports
    alert("Unrecognized format");
    return false;
};
const importNativeString = (str) => {
    let index = str.indexOf("#mse=");
    if (index != -1)
        str = str.substr(index+5);
    let data = JSON.parse(decompressFromEncodedURIComponent(str));
    importNative(data);
};
const importNative = (data) => {
    if (!data.exp)
        throw "Invalid export type";
    if (data.exp == "raid") {
        let raid = importRaidData(data.data);
        raidModel.value = raid;
        raidIsImport.value = true;
        raidModelCopy.value = null;
        if (raids.value.find(r => r.name == raidModel.value.name))
            raidModel.value.name+= " copy";
        raidEdit.value.open(true);
    }
    else if (data.exp == "player") {
        let player = importPlayerData(data.data);
        playerModel.value = player;
        playerImport.value.open(true);
    }
    else {
        throw "Invalid export type";
    }
};
const importSixtyUpgrades = (data) => {
    if (!data.items)
        throw("No items found");

    let player = defaultPlayer();
    player.name = _.get(data, "character.name");
    player.race = _.capitalize(_.get(data, "character.race", "Undead"));

    let convertSlot = (slot) => {
        slot = slot.toLowerCase();
        slot = slot.replace("finger_", "finger");
        slot = slot.replace("trinket_", "trinket");
        slot = slot.replace("shoulders", "shoulder");
        slot = slot.replace("wrists", "wrist");
        return player.loadout.hasOwnProperty(slot) ? slot : null;
    };

    let errors = [];

    for (let _item of data.items) {
        let slot = convertSlot(_item.slot);
        let id = _item.id;
        if (_item.hasOwnProperty("suffixId"))
            id+= ":"+_item.suffixId;
        let item = common.getItem(slot, id);
        if (!item)
            item = items.gear[common.loadoutSlotToItemSlot(slot)].find(i => i.title == _item.name);
        if (!item) {
            player.loadout[slot].item_id = null;
            errors.push("Could not find item: "+_item.name);
            continue;
        }

        let enchant = null;
        if (_item.enchant) {
            if (_item.enchant.id)
                enchant = items.enchants[slot].find(e => e.enchantment_id == _item.enchant.id);
            if (!enchant && _item.enchant.spellId)
                enchant = items.enchants[slot].find(e => e.id == _item.enchant.spellId);
            if (!enchant)
                errors.push("Could not find enchant: "+_item.enchant.name);
        }

        player.loadout[slot].item_id = item.id;
        player.loadout[slot].enchant_id = enchant ? enchant.id : null;
    }

    if (data.talents && data.talents.length) {
        let flatTree = talentTree.trees.reduce((acc, t) => [...acc, ...t.talents.rows.flat()], []);
        player.talents = common.baseTalents();
        for (let _talent of data.talents) {
            let index = flatTree.findIndex(t => t.spellIds.indexOf(_talent.spellId) != -1);
            if (index == -1)
                errors.push("Could not find talent: "+_talent.name);
            else
                player.talents[index] = _talent.rank;
        }
    }

    playerModel.value = player;
    importMessage.value = errors.join("<br>");
    playerImport.value.open(true);
};
const importWSE = (data) => {
    if (!data.gear || !data.gear.items)
        throw("No items found");

    let player = defaultPlayer();
    let slots = common.loadoutSlots();
    let errors = [];

    for (let i in data.gear.items) {
        let _item = data.gear.items[i];
        if (!_item)
            continue;
        let slot = slots[i];
        let item = common.getItem(slot, _item.id);
        if (!item) {
            player.loadout[slot].item_id = null;
            errors.push("Could not find item: "+_item.id);
            continue;
        }

        let enchant = null;
        if (_item.enchant) {
            if (_item.enchant.id)
                enchant = items.enchants[slot].find(e => e.enchantment_id == _item.enchant);
            if (!enchant)
                errors.push("Could not find enchant: "+_item.enchant);
        }

        player.loadout[slot].item_id = item.id;
        player.loadout[slot].enchant_id = enchant ? enchant.id : null;
    }

    if (data.race)
        player.race = _.capitalize(data.race);
    if (common.raceFaction(player.race) != activeRaid.value.faction)
        player.race = convertRace(player.race);

    if (data.talents)
        player.talents = common.parseWowheadTalents(data.talents);

    playerModel.value = player;
    importMessage.value = errors.join("<br>");
    playerImport.value.open(true);
};

/*
 * Result UI
 */
const resultHidden = ref(false);
const resultOpen = ref(false);
const closeResult = () => {
    resultOpen.value = false;
};
const openResult = () => {
    resultOpen.value = true;
};
const playerDps = (player) => {
    if (result.value) {
        return player.dps;
    }

    return player.dps;
};
const statWeight = (wtype) => {
    if (wtype == "sp") {
        return (result.value.dps_sp - result.value.dps_select)/15.0/result.value.iterations;
    } else if (wtype == "crit") {
        return 10.0 * (result.value.dps_crit - result.value.dps_select) / (result.value.dps_sp - result.value.dps_select);
    } else if (wtype == "hit") {
        return 10.0 * (result.value.dps_hit - result.value.dps_select) / (result.value.dps_sp - result.value.dps_select);
    } else if (wtype == "sp90") {
        return (result.value.dps90_sp - result.value.dps90_select)/15.0/result.value.iterations;
    } else if (wtype == "crit90") {
        return 10.0 * (result.value.dps90_crit - result.value.dps90_select) / (result.value.dps90_sp - result.value.dps90_select);
    } else if (wtype == "hit90") {
        return 10.0 * (result.value.dps90_hit - result.value.dps90_select) / (result.value.dps90_sp - result.value.dps90_select);
    }
    return 0.0;
}
const histogramData = computed(() => {
    return result.value.histogram;
});
const comparisonShowAll = ref(true);
const selectedComparison = ref(null);

/*
 * Watchers
 */
watch(settings, saveSettings, {deep : true});
watch(raids, saveRaids, {deep : true});
watch(() => settings.raid_id, (value) => {
    let raid = raids.value.find(raid => raid.id == value);
    if (raid && raid.players.length) {
        activePlayerId.value = raid.players[0].id;
    }
    else {
        activePlayerId.value = null;
        activeTab.value = "config";
    }
});
watch(tooltipRefreshTrigger, refreshTooltips, { deep: true });
watch(() => activePlayerId.value, (value) => {
    if (activeTab.value == "loadout") {
        nextTick(() => {
            if (itemSearchInput.value)
                itemSearchInput.value.focus();
        });
    }
});
watch(() => activeTab.value, (value) => {
    if (value == "loadout") {
        nextTick(() => {
            if (itemSearchInput.value)
                itemSearchInput.value.focus();
        });
    }
    exportSuccess.value = false;
});
watch(() => activePlayer.value, () => {
    if (activePlayer.value) {
        playerStats.value = visualStats(activePlayer.value);
        syncBuffs();
    }
}, {deep: true});
watch(() => result.value, () => {
    activeResultTab.value = "overview";
    resultHidden.value = false;
});
watch(() => customItems.value, (value) => {
    saveCustomItems(value);
});
watch(() => activeRaid.value.faction, () => {
    for (let player of activeRaid.value.players) {
        if (common.raceFaction(player.race) != activeRaid.value.faction)
            player.race = convertRace(player.race);
    }
});
/*
 * Events
 */
onMounted(() => {
    if (activeRaid.value && activeRaid.value.players.length) {
        activePlayerId.value = activeRaid.value.players[0].id;
        activeTab.value = "loadout";
        nextTick(() => {
            playerStats.value = visualStats(activePlayer.value);
        });
    }

    if (window.location.hash) {
        let hash = window.location.hash.substr(1);
        if (hash.substr(0, 4) == "mse=") {
            try {
                importNativeString(hash.substr(4));
            }
            catch(e) {
                alert("Could not import data");
                console.log(e);
            }
            window.location.hash = "";
        }
    }
});
</script>

<template>
    <div class="app">
        <div id="main">
            <div class="left">
                <div class="template-raids">
                    <div class="create"><button class="btn btn-primary block large" @click="createRaidsFromTemplateOpen">Create Raids from Template</button></div>
                </div>

                <div class="raid">
                    <div class="current-raid" @click="raidSelectOpen = !raidSelectOpen">
                        <template v-if="activeRaid">
                            <wowicon class="faction" :icon="activeRaid.faction" />
                            <div class="name">{{ activeRaid.name }}</div>
                            <micon class="caret" icon="keyboard_arrow_down" />
                        </template>
                        <template v-else>
                            <div class="name"><i>No raid</i></div>
                        </template>
                    </div>
                    <div class="raid-select" v-if="raidSelectOpen">
                        <div class="raids">
                            <div
                                class="raid"
                                :class="{active: activeRaid.value == raid.id}"
                                v-for="raid in raids"
                                :key="raid.id"
                                @click="selectRaid(raid.id)"
                            >
                                <wowicon class="faction" :icon="raid.faction" />
                                <span class="middle info">
                                    <div class="name">{{ raid.name }}</div>
                                    <div class="players">{{ raid.players.length }} players</div>
                                </span>
                                <micon class="delete" icon="delete" @click.stop="confirmDeleteRaid(raid)" />
                            </div>
                        </div>
                        <button class="create" @click="createRaidOpen">
                            <micon icon="add" />
                            <span class="middle">Create raid</span>
                        </button>
                    </div>
                </div>

                <div class="team" v-if="activeRaid">
                    <div class="players">
                        <div
                            class="player"
                            :class="{active: activePlayerId == player.id}"
                            v-for="player in activeRaid.players"
                            :key="player.id"
                            @click="selectPlayer(player.id)"
                        >
                            <wowicon class="race" :icon="player.race" />
                            <wowicon class="spec" :icon="playerSpecIcon(player)" />
                            <div class="middle name">{{ player.name }}</div>
                            <micon class="delete" icon="delete" @click.stop="confirmDeletePlayer(player)" />
                        </div>
                    </div>
                    <button class="create" @click="createPlayerOpen">
                        <micon icon="add" />
                        <span class="middle">Add player</span>
                    </button>
                </div>

                <div class="sim">
                    <template v-if="isRunning">
                        <div class="progress">
                            <div class="circle middle">
                                <progress-circle :value="simProgress.progress" />
                                <div class="center">{{ (simProgress.progress * 100).toFixed() }}%</div>
                            </div>
                            <div class="dps middle">
                                <div class="title">DPS</div>
                                <div class="value">{{ simProgress.dps.toFixed(1) }}</div>
                            </div>
                        </div>
                    </template>
                    <template v-else>
                        <div class="run">
                            <div><button class="btn btn-primary block large" @click="runMultiple">Run</button></div>
                            <div><button class="btn btn-text" @click="runSingle">Single iteration</button></div>
                        </div>
                        <div class="result" v-if="result && !resultHidden">
                            <div class="close" @click="resultHidden = true">
                                <micon icon="close" />
                            </div>
                            <div class="dps">
                                <span class="value">{{ playerDps(result.players[0]).toFixed(1) }}</span>
                                <span class="title">{{ result.players[0].name }}</span>
                            </div>
                            <div class="dps">
                                <span class="value">{{ result.dps.toFixed(1) }}</span>
                                <span class="title">Total</span>
                            </div>
                            <button class="link" @click.stop="openResult">
                                <span class="middle">See result</span>
                                <micon icon="keyboard_double_arrow_right" />
                            </button>
                        </div>
                    </template>
                </div>
            </div>

            <div class="right" v-if="activeRaid">
                <div class="tabs">
                    <button class="tab" :class="{active: activeTab == 'config'}" @click="activeTab = 'config'">
                        Config
                    </button>
                    <template v-if="activePlayer">
                        <button class="tab" :class="{active: activeTab == 'loadout'}" @click="activeTab = 'loadout'">
                            Gear
                        </button>
                        <button class="tab" :class="{active: activeTab == 'talents'}" @click="activeTab = 'talents'">
                            Talents
                        </button>
                        <button class="tab" :class="{active: activeTab == 'rotation'}" @click="activeTab = 'rotation'">
                            Rotation
                        </button>
                    </template>
                    <button class="tab" :class="{active: activeTab == 'export'}" @click="activeTab = 'export'">
                        Export
                    </button>
                    <button class="tab" :class="{active: activeTab == 'import'}" @click="activeTab = 'import'">
                        Import
                    </button>
                    <div class="github">
                        <button 
                            class="tutorial-button" 
                            :class="{ 'tutorial-seen': tutorialHasBeenSeen }"
                            @click="openTutorial" 
                            title="Tutorial"
                        >
                            <img src="/img/help.svg" />
                        </button>
                        <a href="https://github.com/ronkuby-mage/vanilla-firemage" target="_blank">
                            <img src="/img/github-mark.svg" alt="Github">
                        </a>
                    </div>
                </div>
                <div class="config" v-if="activeTab == 'config'">
                    <div class="form-box config-raid">
                        <div class="title">Raid config</div>
                        <div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>Name</label>
                                    <input type="text" v-model="activeRaid.name" @input="onRaidNameChange">
                                </div>
                                <div class="form-item">
                                    <label>Faction</label>
                                    <select-simple v-model="activeRaid.faction" :options="factionOptions" />
                                </div>
                            </div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>Fight duration</label>
                                    <input type="text" v-model.number="activeRaid.config.duration">
                                </div>
                                <div class="form-item">
                                    <label>Variance (+/-)</label>
                                    <input type="text" v-model.number="activeRaid.config.duration_variance">
                                </div>
                            </div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>
                                        <span class="middle">Start Delay</span>
                                        <help>Seconds between the encounter start and average player casting start</help>
                                    </label>
                                    <input type="text" v-model.number="activeRaid.config.initial_delay">
                                </div>
                                <div class="form-item">
                                    <label>
                                        <span class="middle">Cast Delay</span>
                                        <help>Delay is minimized by spell queue</help>
                                    </label>
                                    <input type="text" v-model.number="activeRaid.config.continuing_delay">
                                </div>
                                <div class="form-item">
                                    <label>
                                        <span class="middle">React Time</span>
                                        <help>Response delay to debuff refreshes</help>
                                    </label>
                                    <input type="text" v-model.number="activeRaid.config.reaction_time">
                                </div>
                            </div>
                            <div class="form-cols">
                                <label style="display:block; margin-bottom:2px; line-height:1;">Debuffs</label>
                                <div style="display:flex; align-items:center; gap:8px;">
                                    <div class="icon-checkboxes" style="margin:0;">
                                        <label>
                                            <input type="checkbox" v-model="activeRaid.config.curse_of_elements" >
                                            <wowicon icon="curse_of_elements" />
                                            <tooltip position="topright">Curse of the Elements</tooltip>
                                        </label>
                                    </div>

                                    <div class="form-item" style="margin:0;">
                                        <input type="text" v-model.number="activeRaid.config.arcanite_dragonling"
                                         style="width:48px; height:30px; line-height:30px; padding:0 8px; box-sizing:border-box; text-align:center; display:inline-block;">
                                    </div>

                                    <div class="icon-checkboxes"  style="margin:0;">
                                        <label>
                                            <input type="checkbox" checked disabled>
                                            <wowicon icon="arcanite_dragonling" />
                                            <tooltip position="topright">Time to full stack of flame buffet</tooltip>
                                        </label>
                                    </div>
                                    <div class="form-item" style="margin:0;">
                                        <input type="text" v-model.number="activeRaid.config.nightfall1"
                                         style="width:48px; height:30px; line-height:30px; padding:0 8px; box-sizing:border-box; text-align:center; display:inline-block;">
                                    </div>
                                    <div class="form-item" style="margin:0;">
                                        <input type="text" v-model.number="activeRaid.config.nightfall2"
                                         style="width:48px; height:30px; line-height:30px; padding:0 8px; box-sizing:border-box; text-align:center; display:inline-block;">
                                    </div>
                                    <div class="form-item" style="margin:0;">
                                        <input type="text" v-model.number="activeRaid.config.nightfall3"
                                         style="width:48px; height:30px; line-height:30px; padding:0 8px; box-sizing:border-box; text-align:center; display:inline-block;">
                                    </div>
                                    <div class="icon-checkboxes"  style="margin:0;">
                                        <label>
                                            <input type="checkbox" checked disabled>
                                            <wowicon icon="nightfall" />
                                            <tooltip position="topright">Nightfall swing timers</tooltip>
                                        </label>
                                    </div>

                                </div>
                            </div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>Special Boss</label>
                                    <select-simple v-model="activeRaid.config.boss" :options="bossOptions" />
                                </div>
                            </div>
                        </div>
                        <div class="form-item">
                            <checkbox label="Include in Comparison">
                                <input type="checkbox" v-model="activeRaid.config.in_comparison">
                            </checkbox>
                        </div>
                        <div class="form-item">
                            <checkbox label="No Debuff Limit">
                                <input type="checkbox" v-model="activeRaid.config.no_debuff_limit">
                            </checkbox>
                        </div>
                    </div>

                    <div class="form-box config-player" v-if="activePlayer">
                        <div class="title">Player config</div>
                        <div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>Name</label>
                                    <input type="text" v-model="activePlayer.name">
                                </div>
                                <div class="form-item">
                                    <label>Race</label>
                                    <select-simple v-model="activePlayer.race" :options="raceOptions" />
                                </div>
                                    <div class="form-item" v-if="activePlayer.race === 'Troll'">
                                        <label>Berserk</label>
                                        <select-simple v-model="activePlayer.berserk" :options="berserkOptions" />
                                    </div>
                            </div>
                            <div class="form-item">
                                <label>Raid Buffs</label>
                                <div class="icon-checkboxes">
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.arcane_intellect">
                                        <wowicon icon="arcane_intellect" />
                                        <tooltip>Arcane Intellect</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.imp_mark_of_the_wild">
                                        <wowicon icon="imp_mark_of_the_wild" />
                                        <tooltip>Improved Mark of the Wild</tooltip>
                                    </label>
                                    <template v-if="activeRaid.faction == 'Alliance'">
                                        <label>
                                            <input type="checkbox" v-model="activePlayer.buffs.blessing_of_kings">
                                            <wowicon icon="blessing_of_kings" />
                                            <tooltip>Blessing of Kings</tooltip>
                                        </label>
                                    </template>
                                </div>
                            </div>
                            <div class="form-item">
                                <label>World Buffs</label>
                                <div class="icon-checkboxes">
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.rallying_cry">
                                        <wowicon icon="rallying_cry" />
                                        <tooltip>Rallying Cry of the Dragonslayer</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.spirit_of_zandalar">
                                        <wowicon icon="spirit_of_zandalar" />
                                        <tooltip>Spirit of Zandalar</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.dire_maul_tribute">
                                        <wowicon icon="dire_maul_tribute" />
                                        <tooltip>Slip'kik's Savvy</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.songflower">
                                        <wowicon icon="songflower" />
                                        <tooltip>Songflower</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.dmf_dmg">
                                        <wowicon icon="dmf" />
                                        <tooltip>Sayge's Dark Fortune of Damage</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.soul_revival">
                                        <wowicon icon="soul_revival" />
                                        <tooltip>Soul Revival</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.traces_of_silithyst">
                                        <wowicon icon="traces_of_silithyst" />
                                        <tooltip>Traces of Silithyst</tooltip>
                                    </label>
                                </div>
                            </div>
                            <div class="form-item">
                                <label>Consume Buffs</label>
                                <div class="icon-checkboxes">
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.flask_of_supreme_power">
                                        <wowicon icon="flask_supreme_power" />
                                        <tooltip>Flask of Supreme Power</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.elixir_greater_arcane">
                                        <wowicon icon="elixir_greater_arcane" />
                                        <tooltip>Greater Arcane Elixir</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.elixir_greater_firepower">
                                        <wowicon icon="elixir_greater_firepower" />
                                        <tooltip>Elixir of Greater Firepower</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.elixir_frost_power">
                                        <wowicon icon="elixir_frost_power" />
                                        <tooltip>Elixir of Frost Power</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.brilliant_wizard_oil" @click="playerConfigExclusive($event, 'brilliant_wizard_oil', 'blessed_wizard_oil')">
                                        <wowicon icon="weapon_oil_brilliant_wizard" />
                                        <tooltip>Brilliant Wizard Oil</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.blessed_wizard_oil" @click="playerConfigExclusive($event, 'blessed_wizard_oil', 'brilliant_wizard_oil')">
                                        <wowicon icon="weapon_oil_blessed_wizard" />
                                        <tooltip>Blessed Wizard Oil</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.gift_of_stormwind" @click="playerConfigExclusive($event, 'gift_of_stormwind', 'infallible_mind')">
                                        <wowicon icon="gift_of_stormwind" />
                                        <tooltip>Stormwind Gift of Friendship</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.infallible_mind" @click="playerConfigExclusive($event, 'infallible_mind', 'gift_of_stormwind')">
                                        <wowicon icon="infallible_mind" />
                                        <tooltip>Infallible Mind</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.very_berry_cream">
                                        <wowicon icon="very_berry_cream" />
                                        <tooltip>Very Berry Cream</tooltip>
                                    </label>
                                    <label>
                                        <input type="checkbox" v-model="activePlayer.buffs.runn_tum_tuber">
                                        <wowicon icon="runn_tum_tuber" />
                                        <tooltip>Runn Tum Tuber Surprise</tooltip>
                                    </label>
                                </div>
                            </div>
                            <div class="form-item">
                                <checkbox label="Sync Buffs" tip="Sync buffs between all players">
                                    <input type="checkbox" v-model="activeRaid._sync_buffs" @change="onSyncBuffs">
                                </checkbox>
                                <checkbox label="Stat Weight Target" tip="Player output is counted in stat weights">
                                    <input type="checkbox" v-model="activePlayer.is_target">
                                </checkbox>
                                <checkbox label="Stat Weight Differential" tip="Player stats are varied in stat weights">
                                    <input type="checkbox" v-model="activePlayer.is_vary">
                                </checkbox>
                            </div>
                            <div class="form-item">
                                <label>External Buffs</label>
                                <div class="icon-multiboxes">
                                    <label @click="cycleCount('atiesh_mage')" :class="{ active: activePlayer.buffs.atiesh_mage > 0 }" @mouseenter="showTip = true" @mouseleave="showTip = false">
                                        <wowicon icon="atiesh" />
                                        <span v-if="Number(activePlayer.buffs.atiesh_mage) > 1" class="counter-badge">
                                            {{ activePlayer.buffs.atiesh_mage }}
                                        </span>
                                        <tooltip v-show="showTip" class="tip">Atiesh aura from mage(s) in your party</tooltip>
                                    </label>
                                    <label @click="cycleCount('atiesh_warlock')" :class="{ active: activePlayer.buffs.atiesh_warlock > 0 }" @mouseenter="showTip = true" @mouseleave="showTip = false">
                                        <wowicon icon="atiesh" />
                                        <span v-if="Number(activePlayer.buffs.atiesh_warlock) > 1" class="counter-badge">
                                            {{ activePlayer.buffs.atiesh_warlock }}
                                        </span>
                                        <tooltip v-show="showTip" class="tip">Atiesh aura from warlock(s) in your party</tooltip>
                                    </label>
                                    <label @click="cycleCount('moonkin_aura')" :class="{ active: activePlayer.buffs.moonkin_aura }" @mouseenter="showTip = true" @mouseleave="showTip = false">
                                        <wowicon icon="moonkin_aura" />
                                        <tooltip v-show="showTip" class="tip">Moonkin aura</tooltip>
                                    </label>
                                    <label @click="cycleCount('power_infusion')" :class="{ active: activePlayer.pi_count }" @mouseenter="showTip = true" @mouseleave="showTip = false">
                                        <wowicon icon="power_infusion" />
                                        <span v-if="Number(activePlayer.pi_count) > 1" class="counter-badge">
                                            {{ activePlayer.pi_count }}
                                        </span>
                                        <tooltip v-show="showTip" class="tip">Power Infusion</tooltip>
                                    </label>
                                </div>
                            </div>
                            <div class="form-title">Bonus stats</div>
                            <div class="form-cols">
                                <div class="form-item">
                                    <label>Spell power</label>
                                    <input type="text" v-model.number="activePlayer.bonus_stats.sp">
                                </div>
                                <div class="form-item">
                                    <label>Crit %</label>
                                    <input type="text" v-model.number="activePlayer.bonus_stats.crit">
                                </div>
                                <div class="form-item">
                                    <label>Hit %</label>
                                    <input type="text" v-model.number="activePlayer.bonus_stats.hit">
                                </div>
                                <div class="form-item">
                                    <label>Int</label>
                                    <input type="text" v-model.number="activePlayer.bonus_stats.int">
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="form-box small config-settings">
                        <div class="title">Sim config</div>
                        <div>
                            <div class="form-item">
                                <label>Iterations</label>
                                <input type="text" v-model.number="settings.iterations">
                            </div>
                            <div class="form-item">
                                <label>
                                    <span class="middle">Threads</span>
                                    <help>
                                        Number of parallel simulations to run,<br> using multiple CPU cores.<br><br>
                                        Detected cores: <b>{{ detectedCores() }}</b>
                                    </help>
                                </label>
                                <input type="text" v-model.number="settings.threads">
                            </div>
                        </div>
                    </div>
                </div>

                <div class="loadout" v-if="activeTab == 'loadout' && activePlayer">
                    <div class="overview">
                        <div class="copy">
                            <select-simple
                                v-model="copyLoadoutPlayer"
                                :options="loadoutOptions"
                                @input="copyLoadout"
                                placeholder="Copy gear from..."
                             />
                        </div>
                        <div class="paperdoll">
                            <div :class="pos" v-for="pos in ['left', 'right', 'bottom']">
                                <div class="paperslot" :class="css(slot)" v-for="slot in paperdollSlots(pos)">
                                    <div
                                        class="paperv paperitem"
                                        :class="{active: activeSlot == slot && activeGearType == 'gear'}"
                                        @click="paperdollClick(slot)"
                                    >
                                       <template v-if="activePlayer.loadout[slot].item_id">
                                            <span
                                                class="custom-icon"
                                                v-if="common.isCustomItem(activePlayer.loadout[slot].item_id)"
                                            >
                                                <wowicon icon="question_mark" />
                                                <tooltip>Custom: {{ itemTitle(activePlayer.loadout[slot].item_id) }}</tooltip>
                                            </span>
                                            <a
                                                :href="common.gearUrl(activePlayer, slot)"
                                                data-wh-icon-size="large"
                                                data-whtticon="false"
                                                @click.prevent
                                                v-else
                                            ></a>
                                        </template>
                                    </div>
                                    <div class="papers">
                                        <div
                                            class="paperv paperenchant"
                                            :class="{active: activeSlot == slot && activeGearType == 'enchant'}"
                                            v-if="items.enchants.hasOwnProperty(common.loadoutSlotToItemSlot(slot))"
                                            @click="paperdollClick(slot, 'enchant')"
                                        >
                                            <a
                                                v-if="activePlayer.loadout[slot].enchant_id"
                                                :href="spellUrl(activePlayer.loadout[slot].enchant_id)"
                                                data-wh-icon-size="large"
                                                data-whtticon="false"
                                                @click.prevent
                                            ></a>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="stats">
                            <table>
                                <tbody>
                                    <tr>
                                        <td>Spell power (fire)</td>
                                        <td>
                                            <span>
                                                {{ playerStats.sp + playerStats.sp_fire}}
                                                <tooltip position="left"><spell-power :value="playerStats" /></tooltip>
                                            </span>
                                        </td>
                                    </tr>
                                    <tr>
                                        <td>Crit Chance (fire)</td>
                                        <td>{{ playerStats.crit.toFixed(2) }}%</td>
                                    </tr>
                                    <tr>
                                        <td>+Hit Chance</td>
                                        <td>{{ playerStats.hit.toFixed() }}%</td>
                                    </tr>
                                    <tr>
                                        <td>Intellect</td>
                                        <td>{{ playerStats.int }}</td>
                                    </tr>
                                    <tr>
                                        <td>Spell power (frost)</td>
                                        <td>
                                            <span>
                                                {{ playerStats.sp + playerStats.sp_frost}}
                                                <tooltip position="left"><spell-power :value="playerStats" /></tooltip>
                                            </span>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>

                    <div class="itemlist" v-if="activeSlot && activeGearType">
                        <div class="search">
                            <input
                                type="text"
                                class="search-q"
                                v-model="itemSearch"
                                ref="itemSearchInput"
                                placeholder="Search..."
                                @keydown.enter="itemSearchEnter"
                                @keydown.up="itemSearchUp"
                                @keydown.down="itemSearchDown"
                                @keydown.esc="itemSearch = ''"
                                autofocus
                            >
                            <div class="keyboard-help" @click="itemSearchInput.focus()">
                                <div class="title">
                                    <micon icon="keyboard" />
                                </div>
                                <div class="drop">
                                    <div><span>Esc:</span> <span>Clear search</span></div>
                                    <div><span>Enter:</span> <span>Equip first item in the list</span></div>
                                    <div><span>Up/Down:</span> <span>Equip previous/next item in the list</span></div>
                                </div>
                            </div>
                            <div class="custom-item-button">
                                <button class="btn btn-primary" @click="createCustomItemOpen">Create item</button>
                            </div>
                        </div>
                        <div class="items">
                            <table v-if="itemList.list">
                                <thead>
                                    <tr>
                                        <th class="title">
                                            <sort-link v-model="itemSorting" name="title">Name</sort-link>
                                        </th>
                                        <th v-if="itemList.type != 'enchant'">
                                            <sort-link v-model="itemSorting" name="ilvl" order="desc">ilvl</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="sp" order="desc">SP</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="crit" order="desc">Crit</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="hit" order="desc">Hit</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="int" order="desc">Int</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="spi" order="desc">Spi</sort-link>
                                        </th>
                                        <th>
                                            <sort-link v-model="itemSorting" name="mp5" order="desc">Mp5</sort-link>
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr
                                        :class="{active: itemList.type == 'enchant' ? activePlayer.loadout[activeSlot].enchant_id == item.id : activePlayer.loadout[activeSlot].item_id == item.id}"
                                        v-for="item in itemList.list"
                                        :key="item.id"
                                        @click="itemClick(item)"
                                    >
                                        <td class="title" v-if="common.isCustomItem(item)">
                                            <wowicon icon="question_mark" />
                                            <span class="middle" :class="'quality-'+item.q">
                                                {{ item.title }}
                                            </span>
                                            <span class="middle edit" @click.stop="editCustomItemOpen(item)">
                                                <micon icon="edit" />
                                                <tooltip>Edit item</tooltip>
                                            </span>
                                            <span class="middle delete" @click.stop="deleteCustomItem(item)">
                                                <micon icon="delete" />
                                                <tooltip>Delete item</tooltip>
                                            </span>
                                        </td>
                                        <td class="title" v-else>
                                            <a
                                                :href="itemList.type == 'enchant' ? spellUrl(item.id) : common.itemUrl(item.id)"
                                                :class="'quality-'+_.get(item, 'q', itemList.type == 'enchant' ? 'uncommon' : 'epic')"
                                                data-whtticon="false"
                                                target="_blank"
                                                @click.prevent
                                            >
                                                {{ item.title }}
                                            </a>
                                          <span v-if="item.pvp" class="pvp-badge">PVP</span>
                                        </td>
                                        <td v-if="itemList.type != 'enchant'">{{ item.ilvl }}</td>
                                        <td><spell-power :value="item" /></td>
                                        <td>{{ _.get(item, "crit", "") }}</td>
                                        <td>{{ _.get(item, "hit", "") }}</td>
                                        <td>{{ _.get(item, "int", "") }}</td>
                                        <td>{{ _.get(item, "spi", "") }}</td>
                                        <td>{{ _.get(item, "mp5", "") }}</td>
                                    </tr>
                                </tbody>
                            </table>
                            <div class="empty" v-else>
                                No results
                            </div>
                            <div class="missing-item-button">
                                <button class="btn btn-secondary small" @click="missingItemOpen">Can't find an item?</button>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="talents" v-if="activeTab == 'talents' && activePlayer">
                    <div class="import">
                        <select-simple
                            v-model="talentPreset"
                            :options="talentPresetOptions"
                            placeholder="Select preset..."
                            @input="setTalentPreset"
                        />
                        <input type="text" placeholder="Paste URL from wowhead to import" v-model="talentImport" @input="importTalents">
                    </div>
                    <talent-calculator v-model="activePlayer.talents" :level="activePlayer.level" />
                </div>

                <div class="rotation" v-if="activeTab == 'rotation' && activePlayer">
                    <div class="form-box larger">
                        <div class="form-title">Rotation</div>
                        <apl v-model="activePlayer.apl" :player="activePlayer" @save="editAplOpen" />
                    </div>
                    <div class="form-boxes">
                       <div class="form-box medium apl-auto-build">
                            <div class="form-title">Auto-build</div>
                            <div class="auto-build-button">
                                <button class="btn btn-secondary block" @click="autofillRotation" style="margin-top: 16px;">Autofill Options</button>
                            </div>
                            <div class="form-item">
                                <label>Pre-Scorch</label>
                                <select-simple v-model="autoBuildOptions.preScorch" :options="preScorchOptions" />
                            </div>
                            <div class="form-item">
                                <label>Buffer Spell</label>
                                <select-simple v-model="autoBuildOptions.bufferSpell" :options="bufferSpellOptions" />
                            </div>
                            <div class="form-item">
                                <label>Trinket Usage</label>
                                <select-simple v-model="autoBuildOptions.derivedOpening" :options="derivedOpeningOptions" />
                            </div>
                            <div class="form-item">
                                <label>Ignite Sustain Strategy</label>
                                <select-simple v-model="autoBuildOptions.sustain" :options="sustainOptions" />
                            </div>
                            <div class="auto-build-button" style="margin-top: 16px;">
                                <button class="btn btn-secondary block" @click="setRotationFromAutoBuild">Set Rotation from Options</button>
                            </div>
                        </div>
                        <div class="form-box medium apl-list" v-if="apls.length">
                            <div class="form-title">Your rotations</div>
                            <div class="form-item">
                                <select-simple
                                    v-model="playerModelCopy"
                                    :options="otherPlayerOptions"
                                    empty-option="Copy from..."
                                    @input="copyPlayerApl"
                                />
                            </div>
                            <div class="list">
                                <div
                                    class="item default"
                                    v-for="item in apls"
                                    :key="item.id"
                                    @click="selectApl(item)"
                                >
                                    <div class="name">{{ item.name }}</div>
                                    <div class="actions">
                                        <button class="delete" @click.stop="deleteApl(item.id)">
                                            <micon icon="delete" />
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="export" v-if="activeTab == 'export'">
                    <div class="form-box">
                        <div class="title">Export</div>
                        <div class="form-item">
                            <label>What to export</label>
                            <select-simple v-model="exportType" :options="exportTypeOptions" />
                        </div>
                        <div class="buttons">
                            <button class="btn btn-primary" @click="exportSubmit">Export</button>
                            <span class="middle copy-success" v-if="exportSuccess">
                                <micon icon="check" />
                                <span class="middle">Copied to clipboard!</span>
                            </span>
                        </div>
                    </div>
                </div>

                <div class="import" v-if="activeTab == 'import'">
                    <div class="form-box large">
                        <div class="title">Import</div>
                        <div class="form-item">
                            <textarea v-model="importData" placeholder="Supported formats: MageSim, Sixtyupgrades, WowSimsExporter"></textarea>
                        </div>
                        <div class="buttons">
                            <button class="btn btn-primary" @click="importSubmit">Import</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="result-backdrop" @click="closeResult" v-if="resultOpen"></div>
        <div id="result" :class="{active: resultOpen}">
            <div class="result-content">
                <button class="close" @click="closeResult">
                    <micon icon="close" />
                </button>
                <template v-if="result">
                    <div class="tabs">
                        <button class="tab" :class="{active: activeResultTab == 'overview'}" @click="activeResultTab = 'overview'">
                            Overview
                        </button>
                        <template v-if="result.iterations > 1">
                            <button class="tab" :class="{active: activeResultTab == 'comparison'}" @click="activeResultTab = 'comparison'">
                                Comparison
                            </button>
                        </template>
                        <template v-else>
                            <button class="tab" :class="{active: activeResultTab == 'log'}" @click="activeResultTab = 'log'">
                                Combat log
                            </button>
                        </template>
                    </div>

                    <div class="overview" v-if="activeResultTab == 'overview'">
                        <div class="dps-overview" v-if="resultOpen">
                            <div class="players">
                                <div class="player" v-for="player in result.players">
                                    <div class="progress-wrapper">
                                        <progress-circle :value="playerDps(player) / result.dps" :animate="true" />
                                        <div class="center">
                                            <div class="value">
                                                <animate-number :end="playerDps(player) / result.dps * 100" :decimals="0" />%
                                            </div>
                                        </div>
                                    </div>
                                    <div class="info">
                                        <div class="name">{{ player.name }}</div>
                                        <div class="dps">
                                            <animate-number :end="playerDps(player)" />
                                        </div>
                                    </div>
                                </div>

                            </div>
                            <div class="total progress-wrapper">
                                <progress-circle :value="1" :animate="true" />
                                <div class="center">
                                    <div class="title">Total dps</div>
                                    <div class="value">
                                        <animate-number :end="result.dps" />
                                    </div>
                                    <div class="notice" v-if="result.iterations > 1">{{ result.min_dps.toFixed() }} - {{ result.max_dps.toFixed() }}</div>
                                </div>
                            </div>
                            <div class="ignite progress-wrapper" v-if="result.ignite_dps">
                                <progress-circle :value="result.ignite_dps / result.dps" :animate="true" />
                                <div class="center">
                                    <div class="title">Ignite dps</div>
                                    <div class="value">
                                        <animate-number :end="result.ignite_dps" />
                                    </div>
                                </div>
                            </div>
                            <!-- NEW STATS SECTION -->
                            <div class="stats" v-if="result.iterations > 1">
                                <div class="stats-grid">
                                    <div class="stat-item">
                                        <div class="title">SP per Crit %</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('crit') || 0" :decimals="1" />
                                        </div>
                                    </div>
                                    <div class="stat-item">
                                        <div class="title">SP per Hit %</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('hit') || 0" :decimals="1" />
                                        </div>
                                        <tooltip position="bottom">Invalid if player hit is 98% or 99%</tooltip>
                                    </div>
                                    <div class="stat-item">
                                        <div class="title">DPS per SP</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('sp') || 0" :decimals="2" />
                                        </div>
                                    </div>
                                    <div class="stat-item">
                                        <div class="title">SP per Crit % @90</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('crit90') || 0" :decimals="1" />
                                        </div>
                                    </div>
                                    <div class="stat-item">
                                        <div class="title">SP per Hit % @90</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('hit90') || 0" :decimals="1" />
                                        </div>
                                        <tooltip position="topright">Invalid if player hit is 98% or 99%</tooltip>
                                    </div>
                                    <div class="stat-item">
                                        <div class="title">DPS per SP @90</div>
                                        <div class="value">
                                            <animate-number :end="statWeight('sp90') || 0" :decimals="2" />
                                        </div>
                                    </div>
                                </div>
                            </div>                            
                            <div class="info">
                                <table>
                                    <tbody>
                                        <tr><td>Execution time:</td><td>{{ result.time.toFixed(2) }}s</td></tr>
                                        <template v-if="result.iterations > 1">
                                            <tr><td>Iterations:</td><td>{{ result.iterations }}</td></tr>
                                            <tr><td>Time / iteration:</td><td>{{ (result.time / result.iterations * 1000).toFixed(2) }}ms</td></tr>
                                        </template>
                                    </tbody>
                                </table>
                            </div>
                            <div class="histogram-section" v-if="result.iterations > 1">
                                <histogram :data="histogramData" />
                            </div>
                        </div>
                    </div>
                    <div class="comparison" v-if="activeResultTab == 'comparison'">
                        <comparison v-if="activeResultTab == 'comparison'" :result="result" :active-raid="activeRaid"/>
                    </div>
                    <template v-if="result.iterations < 2">
                        <div class="combat-log" v-if="activeResultTab == 'log'">
                            <div class="search">
                                <div class="search-player">
                                    <select-simple v-model="filterPlayer" :options="filterPlayerOptions" empty-option="All players" />
                                </div>
                            </div>
                            <table>
                                <thead>
                                    <tr>
                                        <th>Time</th>
                                        <th>Unit</th>
                                        <th>Type</th>
                                        <th>Text</th>
                                        <th>Result</th>
                                        <th>Combustion</th>
                                        <th>Buffs</th>
                                        <th>Debuffs</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr v-for="log in filteredLog" :class="['log-type-'+css(log.log_type)]">
                                        <td>{{ formatTime(log.t) }}</td>
                                        <td>{{ log.unit_name }}</td>
                                        <td>{{ formatLogType(log.log_type) }}</td>
                                        <td class="text" v-html="formatLogText(log)"></td>
                                        <td>
                                            <template v-if="log.spell_result == 'Hit' || log.spell_result == 'Crit'">
                                                <span class="format-dmg" :class="['spell-result-'+css(log.spell_result)]">
                                                    {{ log.value.toFixed() }}
                                                </span>
                                                <span v-if="log.value2">
                                                    (-{{ log.value2.toFixed() }})
                                                </span>
                                            </template>
                                            <span v-else-if="log.value">
                                                {{ log.value.toPrecision(2) }}
                                            </span>
                                            <span v-if="log.spell_result == 'Miss'">
                                                Miss
                                            </span>
                                        </td>
                                        <td>
                                            {{ log.combustion }}
                                        </td>
                                        <td>
                                            {{ log.buffs }}
                                        </td>
                                        <td>
                                            {{ log.debuffs }}
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </template>
                </template>
            </div>
        </div>

        <div class="notifications">
            <div
                class="notification"
                :class="[_.get(notification, 'class', null)]"
                v-for="notification in notifications"
            >
                <div class="title" v-if="notification.title" v-html="notification.title"></div>
                <div class="text" v-if="notification.text" v-html="notification.text"></div>
            </div>
        </div>

        <spotlight ref="raidEdit" class="small" v-slot="{ close }">
            <div class="default raid-edit">
                <div class="form-title" v-if="raidIsImport">Import raid</div>
                <div class="form-title" v-else>Create raid</div>
                <div class="form-item">
                    <label>Name</label>
                    <input type="text" v-model="raidModel.name" @keydown.enter="updateRaid">
                </div>
                <div class="form-item">
                    <label>Faction</label>
                    <select-simple v-model="raidModel.faction" :options="factionOptions" />
                </div>
                <div class="form-item" v-if="!raidIsImport">
                    <label>Copy from</label>
                    <select-simple v-model="raidModelCopy" :options="raidCopyOptions" empty-option="None" />
                </div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="updateRaid">Save raid</button>
                    <button class="btn btn-secondary" @click="close">Cancel</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="playerEdit" class="small">
            <div class="default player-edit">
                <div class="form-title">Create player</div>
                <div class="form-item">
                    <label>Name</label>
                    <input type="text" v-model="playerModel.name" @keydown.enter="updatePlayer">
                </div>
                <div class="form-item">
                    <label>Race</label>
                    <select-simple v-model="playerModel.race" :options="raceOptions" />
                </div>
                <div class="form-item">
                    <label>Copy from</label>
                    <select-simple v-model="playerModelCopy" :options="playerCopyOptions" empty-option="None" />
                </div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="updatePlayer">Save player</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="playerImport" class="small">
            <div class="default player-edit">
                <div class="form-title">Import player</div>
                <div class="form-item">
                    <label>Import to</label>
                    <select-simple v-model="playerModelCopy" :options="playerCopyOptions" empty-option="New player" />
                </div>
                <template v-if="playerModelCopy">
                    <div class="form-item">
                        <checkbox label="Import config"><input type="checkbox" v-model="playerImportConfig"></checkbox>
                    </div>
                    <div class="form-item">
                        <checkbox label="Import gear"><input type="checkbox" v-model="playerImportLoadout"></checkbox>
                    </div>
                </template>
                <template v-else>
                    <div class="form-item">
                        <label>Name</label>
                        <input type="text" v-model="playerModel.name" @keydown.enter="updatePlayer">
                    </div>
                </template>
                <div class="import-message" v-if="importMessage" v-html="importMessage"></div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="updatePlayer">Save player</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="editApl" class="small">
            <div class="default apl-edit" v-if="aplModel">
                <div class="form-title">Save rotation</div>
                <div class="form-item">
                    <label>Name</label>
                    <input type="text" v-model="aplModel.name" @keydown.enter="updateApl">
                </div>
                <div class="form-item">
                    <label>Save as</label>
                    <select-simple
                        v-model="aplModel.id"
                        :options="aplTargetOptions"
                        empty-option="New rotation"
                    />
                </div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="updateApl">Save rotation</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="missingItem" v-slot="{ close }">
            <div class="default">
                <div class="title">Can't find an item?</div>
                <p>
                    Only a selection of the most common items are present in the sim.<br>
                    Here's what you can do about it:
                </p>
                <p>
                    <span class="link" @click="createCustomItemOpen">Create a custom item</span><br>
                    This allows you to create an item with custom stats. The item is saved in your browser and is only available to you.
                    It will also be present in your exports if it is equipped by a player.
                </p>
                <p>
                    <span class="link" @click="activeTab = 'config'; close();">Add bonus stats</span><br>
                    A quick solution where you can simply add the stats of your missing item directly to the player.<br>
                    Though it can become cumbersome when you want to switch items.
                </p>
                <p>
                    <a href="https://github.com/ronkuby-mage/vanilla-firemage/issues/new?title=Missing%20item:%20" target="_blank">
                        Create an issue
                    </a><br>
                    If you think this item should be part of the list, you can tell me about it.
                </p>
            </div>
        </spotlight>

        <spotlight ref="editCustomItem">
            <div class="default custom-item-edit" v-if="customItemModel">
                <div class="form-title" v-if="customItemModel.is_new">Create item ({{ customItemModel.slot }})</div>
                <div class="form-title" v-else>Edit item ({{ customItemModel.slot }})</div>
                <div class="form-cols">
                    <div class="form-item">
                        <label>Name</label>
                        <input type="text" v-model="customItemModel.title">
                    </div>
                </div>
                <div class="form-cols">
                    <div class="form-item">
                        <label>Spell power</label>
                        <input type="text" v-model.number="customItemModel.sp">
                    </div>
                    <div class="form-item">
                        <label><span class="sp-arcane">Arcane</span></label>
                        <input type="text" v-model.number="customItemModel.sp_arcane">
                    </div>
                    <div class="form-item">
                        <label><span class="sp-fire">Fire</span></label>
                        <input type="text" v-model.number="customItemModel.sp_fire">
                    </div>
                    <div class="form-item">
                        <label><span class="sp-frost">Frost</span></label>
                        <input type="text" v-model.number="customItemModel.sp_frost">
                    </div>
                </div>
                <div class="form-cols">
                    <div class="form-item">
                        <label>Hit %</label>
                        <input type="text" v-model.number="customItemModel.hit">
                    </div>
                    <div class="form-item">
                        <label>Crit %</label>
                        <input type="text" v-model.number="customItemModel.crit">
                    </div>
                    <div class="form-item">
                        <label>Intellect</label>
                        <input type="text" v-model.number="customItemModel.int">
                    </div>
                    <div class="form-item">
                        <label>Spirit</label>
                        <input type="text" v-model.number="customItemModel.spi">
                    </div>
                    <div class="form-item">
                        <label>Mp5</label>
                        <input type="text" v-model.number="customItemModel.mp5">
                    </div>
                </div>
                <div class="form-item" v-if="customItemModel.slot == 'main_hand'">
                    <checkbox label="Two handed"><input type="checkbox" v-model="customItemModel.twohand"></checkbox>
                </div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="updateCustomItem">Save item</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="confirmSpotlight" class="small confirm">
            <div class="default">
                <div class="confirm-text">{{ confirmation.text }}</div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="confirmationContinue">{{ confirmation.confirm }}</button>
                    <button class="btn btn-secondary" @click="confirmationCancel">{{ confirmation.abort }}</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="confirmSpotlight" class="small confirm">
            <div class="default">
                <div class="confirm-text">{{ confirmation.text }}</div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="confirmationContinue">{{ confirmation.confirm }}</button>
                    <button class="btn btn-secondary" @click="confirmationCancel">{{ confirmation.abort }}</button>
                    <button 
                        v-if="confirmation.showGroupDelete"
                        class="btn btn-primary btn-group-delete" 
                        @click="confirmationGroupContinue"
                    >
                        {{ confirmation.groupDelete }}
                    </button>
                </div>
            </div>
        </spotlight>        

        <spotlight ref="alertSpotlight" class="small alert">
            <div class="default">
                <div class="alert-text">{{ alerter.text }}</div>
                <div class="buttons">
                    <button class="btn btn-primary" @click="alertClose">Close</button>
                </div>
            </div>
        </spotlight>

        <spotlight ref="templateRaidEdit" class="small" v-slot="{ close }">
            <div class="default template-raid-edit">
                <div class="form-title">Create raids from template</div>
                
                <div class="form-item">
                    <label>Template</label>
                    <select-simple 
                        v-model="templateRaidModel.templateType" 
                        :options="[
                            { value: 'preset', title: 'From preset (debuffs & external buffs from active)'},
                            { value: 'existing', title: 'From existing raid' }
                        ]" 
                    />
                </div>

                <template v-if="templateRaidModel.templateType === 'preset'">
                    <div class="form-item">
                        <label>Prefix (optional)</label>
                        <input 
                            type="text" 
                            v-model="templateRaidModel.prefix" 
                            placeholder="e.g., MQG-test"
                        />
                    </div>                    
                    <div class="form-item">
                        <label>Number of mages</label>
                        <select-simple 
                            v-model="templateRaidModel.numMages" 
                            :options="templateRaidNumMageOptions" 
                        />
                    </div>
                    <div class="form-item">
                        <label>Gear level</label>
                        <select-simple 
                            v-model="templateRaidModel.gearLevel" 
                            :options="templateRaidGearOptions" 
                        />
                    </div>
                    <div class="form-item">
                        <label>Faction</label>
                        <select-simple 
                            v-model="templateRaidModel.faction" 
                            :options="factionOptions" 
                        />
                    </div>
                </template>

                <template v-if="templateRaidModel.templateType === 'existing'">
                    <div class="form-item">
                        <label>Select raid</label>
                        <select-simple 
                            v-model="templateRaidModel.sourceRaid" 
                            :options="templateRaidSourceOptions" 
                        />
                    </div>
                </template>

                <div class="form-item">
                    <label>Encounter duration (seconds)</label>
                    <input 
                        type="number" 
                        v-model.number="templateRaidModel.encounterDuration" 
                        min="10" 
                        max="300"
                        placeholder="60"
                    />
                </div>                


                <div class="buttons">
                    <button class="btn btn-primary" @click="updateTemplateRaids">Create raids</button>
                    <button class="btn btn-secondary" @click="close">Cancel</button>
                </div>
            </div>
        </spotlight>
        <spotlight ref="tutorialSpotlight" class="large">
            <Tutorial @close="closeTutorial" />
        </spotlight>
    </div>
</template>
