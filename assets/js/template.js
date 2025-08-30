import _ from "lodash";
import apl from "./apl";
import common from "./common";
import ids from "./items";

const shortFightThreshold = 25.0;
const mediumFightThreshold = 45.0;
const longFightThreshold = 60.0;
const staticCondition = {
    2: 20.0,
    3: 15.0,
    4: 12.0,
    5: 10.0
};
const PreScorch = Object.freeze({
    PRESCORCH_YES: "prescorch",
    PRESCORCH_NO: ""
});
const OpeningPermutation = Object.freeze({
    MQG: "mqg",
    TWO_TRINKETS: "mqg-dmg",
    ACTIVE_DAMAGE: "dmg",
    TWO_DAMAGE: "dmg-dmg",
    BY_PLAYER: "" // also placeholder for no active
});
const sustainPermutation = Object.freeze({
    NO_SUSTAIN: "",
    ONE_SUSTAIN: "",
    TWO_SUSTAIN_COBCOB: "cob-cob",
    TWO_SUSTAIN_COBCD: "cob-cd",
    TWO_SUSTAIN_COBWEP: "cob-wep",
    THREE_SUSTAIN_COBCOBCOB: "3cob-cob-cob",
    THREE_SUSTAIN_COBCDCD: "cob-2cd",
    THREE_SUSTAIN_COBCDWEP: "cob-cd-wep",
    FOUR_SUSTAIN_COBCOBCCDCD: "2cob-2cd",
    FOUR_SUSTAIN_COBCBOWEPWEP: "2cob-2wep",
});
const knownDamageTrinkets = new Set([
    ids.TRINKET_RESTRAINED_ESSENCE,
    ids.TOEP,
    ids.TRINKET_ZHC,
]);

/* see charts on page 6 of https://github.com/ronkuby-mage/vanilla-firemage/ignite.pdf */
const getSustainPermutations = (staticTime, numMages, averageCrit) => {
    let sustainPermutations = [];
    const regularMages = Math.min(numMages, 6);

    if (numMages == 1) {
        sustainPermutations.push()

    }


    return sustainPermutations;
};


/**
 * Generates multiple raids based on a template raid
 * @param {Object} templateRaid - The source raid to use as a template
 * @param {Object} options - Configuration options for generation
 * @returns {Array} Array of generated raid objects
 */
export const generateRaidsFromTemplate = (templateRaid, options = {}) => {
    const {
        isPreset = true,
        namePrefix = '',
        encounterDuration = 60,
        averageCrit = 25.0,
        naxxTrinketAvailability = true
        // Add other options as needed
    } = options;
    const generatedRaids = [];

    // first determine whether no prescorch should be an option
    let preScorchPermutations = [PreScorch.PRESCORCH_YES];
    if (encounterDuration < mediumFightThreshold) {
        preScorchPermutations.push(PreScorch.PRESCORCH_NO)
    }
    // loop over opening permutation
    preScorchPermutations.forEach(preScorch => {
        let openingPermutations = [];
        if (isPreset) {
            if (encounterDuration < mediumFightThreshold) {
                openingPermutations.push(OpeningPermutation.MQG);
            }
            if (encounterDuration > shortFightThreshold && encounterDuration < longFightThreshold) {
                openingPermutations.push(OpeningPermutation.TWO_TRINKETS);
            }
            if (encounterDuration >= mediumFightThreshold) {
                openingPermutations.push(OpeningPermutation.ACTIVE_DAMAGE);
            }
        } else {
            // placeholder so that we get one iteration/and for no active
            openingPermutations.push(OpeningPermutation.BY_PLAYER);
        }

        openingPermutations.forEach(openingPermutation => {
            // calculate time to static conditions
            const buildMages = Math.min(Math.max(templateRaid.players.length, 2), 5);
            const buildTime = staticCondition.buildMages + (50.0 - averageCrit)/5.0;
            const staticTime = Math.max(encounterDuration - buildTime, 0.0)
            let sustainPermutations = [];

            if (preScorch == PreScorch.PRESCORCH_NO) {
                // if we are not prescorching, it is unlikely sustaining ignite is a focus
                sustainPermutations = [sustainPermutation.NO_SUSTAIN];
            } else {
                sustainPermutations = getSustainPermutations(staticTime, templateRaid.players.length, averageCrit);
            }

            sustainPermutations.forEach(sustainPermutation => {
                const newRaid = _.cloneDeep(templateRaid);
                newRaid.id = common.uuid();
                newRaid.players.forEach(player => {
                    player.id = common.uuid();
                    let derivedOpening = openingPermutation;                
                    let player_trinket = [];
                    // check if we need trinket swap
                    if (isPreset) {
                        if (openingPermutation == OpeningPermutation.ACTIVE_DAMAGE) {
                            if (naxxTrinketAvailability) {
                                player_trinket = [ids.TRINKET_MARK_OF_THE_CHAMPION, ids.TRINKET_RESTRAINED_ESSENCE];
                            } else {
                                player_trinket = [ids.TRINKET_TEAR, ids.TRINKET_ZHC];
                            }
                        } else if (openingPermutation == OpeningPermutation.MQG) {
                            if (naxxTrinketAvailability) {
                                player_trinket = [ids.TRINKET_MARK_OF_THE_CHAMPION, ids.TRINKET_MQG];
                            } else {
                                player_trinket = [ids.TRINKET_TEAR, ids.TRINKET_MQG];
                            }
                        } else if (openingPermutation == OpeningPermutation.TWO_TRINKETS) {
                            if (naxxTrinketAvailability) {
                                player_trinket = [ids.TRINKET_RESTRAINED_ESSENCE, ids.TRINKET_MQG];
                            } else {
                                player_trinket = [ids.TRINKET_ZHC, ids.TRINKET_MQG];
                            }
                        }
                        player.loadout.trinket1 = { item_id: player_trinket[0], enchant_id: null };
                        player.loadout.trinket2 = { item_id: player_trinket[1], enchant_id: null };
                    } else {
                        if (player.loadout.trinket1.item_id == ids.TRINKET_MQG || player.loadout.trinket2.item_id == ids.TRINKET_MQG) {
                            if (knownDamageTrinkets.has(player.loadout.trinket1.item_id) || knownDamageTrinkets.has(player.loadout.trinket1.item_id)) {
                                derivedOpening = OpeningPermutation.TWO_TRINKETS;
                                if (player.loadout.trinket1.item_id == ids.TRINKET_MQG) {
                                    player_trinket = [player.loadout.trinket2.item_id, ids.TRINKET_MQG];
                                } else {
                                    player_trinket = [player.loadout.trinket1.item_id, ids.TRINKET_MQG];
                                }
                            } else {
                                derivedOpening = OpeningPermutation.MQG;
                                player_trinket = [0, ids.TRINKET_MQG];
                            }
                        } else if (knownDamageTrinkets.has(player.loadout.trinket1.item_id) && knownDamageTrinkets.has(player.loadout.trinket2.item_id)) {
                            derivedOpening = OpeningPermutation.TWO_DAMAGE;
                            player_trinket = [player.loadout.trinket1.item_id, player.loadout.trinket2.item_id];
                        } else if (knownDamageTrinkets.has(player.loadout.trinket1.item_id)) {
                            derivedOpening = OpeningPermutation.ACTIVE_DAMAGE;
                            player_trinket = [0, player.loadout.trinket1.item_id];
                        } else if (knownDamageTrinkets.has(player.loadout.trinket2.item_id)) {
                            derivedOpening = OpeningPermutation.ACTIVE_DAMAGE;
                            player_trinket = [0, player.loadout.trinket2.item_id];
                        }
                    }
                });
                // done collecting/setting player trinkets



            });
        });
    });




    variations.forEach(variation => {
        const newRaid = _.cloneDeep(templateRaid);
        
        // Modify the raid based on variation
        newRaid.id = common.uuid();
        newRaid.name = namePrefix ? 
            `${namePrefix} ${templateRaid.name} - ${variation}` : 
            `${templateRaid.name} - ${variation}`;
        newRaid.faction = variation;
        newRaid.config.duration = encounterDuration;
        
        // Convert player races if faction changed
        if (newRaid.faction !== templateRaid.faction) {
            newRaid.players.forEach(player => {
                player.id = common.uuid();
                player.race = convertRaceForFaction(player.race, variation);
            });
        } else {
            // Still need new player IDs even if same faction
            newRaid.players.forEach(player => {
                player.id = common.uuid();
            });
        }
        
        generatedRaids.push(newRaid);
    });

    return generatedRaids;
};

/**
 * Helper function to convert race based on faction
 */
const convertRaceForFaction = (currentRace, targetFaction) => {
    const raceConversions = {
        'Alliance': {
            'Troll': 'Gnome',
            'Undead': 'Human',
            'Gnome': 'Gnome',
            'Human': 'Human'
        },
        'Horde': {
            'Gnome': 'Troll',
            'Human': 'Undead', 
            'Troll': 'Troll',
            'Undead': 'Undead'
        }
    };
    
    return raceConversions[targetFaction][currentRace] || currentRace;
};

// Export additional utility functions if needed
export const createVariationOptions = (baseOptions) => {
    // Helper to create different variation configurations
    return baseOptions;
};