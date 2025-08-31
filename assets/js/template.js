import _, { isLength } from "lodash";
import apl from "./apl";
import common from "./common";
import ids from "./items";
import ignite from "./ignite";

const shortFightThreshold = 25.0;
const mediumFightThreshold = 45.0;
const longFightThreshold = 60.0;
const durationCriteria = 1.5;
const staticCondition = {
    2: 20.0,
    3: 15.0,
    4: 12.0,
    5: 10.0
};
const preScorches = {
    1: 6,
    2: 3,
    3: 2,
    4: 2,
    5: 2,
    6: 1,
}
const PreScorch = Object.freeze({
    PRESCORCH_YES: "",
    PRESCORCH_NO: "no-scorch"
});
const Buffer = Object.freeze({
    BUFFER_FIREBALL: "gcd",
    BUFFER_PYROBLAST: "pyro",
    BUFFER_NOTHING: "",
});
const OpeningPermutation = Object.freeze({
    MQG: "mqg",
    TWO_TRINKETS: "mqg-dmg",
    ACTIVE_DAMAGE: "dmg",
    TWO_DAMAGE: "dmg-dmg",
    BY_PLAYER: "" // also placeholder for no active
});
const auraTriggers = new Set([
    "mqg",
    "mqg-dmg",
    "dmg",
    "dmg-dmg"
]);
const SustainPermutation = Object.freeze({
    NO_SUSTAIN: "",
    ONE_SUSTAIN: "cob",
    TWO_SUSTAIN_COBCOB: "cob-cob",
    TWO_SUSTAIN_COBCD: "cob-cd",
    TWO_SUSTAIN_COBWEP: "cob-wep",
    THREE_SUSTAIN_COBCOBCOB: "3cob-cob-cob",
    THREE_SUSTAIN_COBCDCD: "cob-2cd",
    THREE_SUSTAIN_COBCDWEP: "cob-cd-wep",
    FOUR_SUSTAIN_COBCOBCDCD: "2cob-2cd",
    FOUR_SUSTAIN_COBCOBWEPWEP: "2cob-2wep",
});
const rotationsBySustains = Object.freeze({
    0: [SustainPermutation.NO_SUSTAIN],
    1: [SustainPermutation.ONE_SUSTAIN],
    2: [SustainPermutation.TWO_SUSTAIN_COBCOB, SustainPermutation.TWO_SUSTAIN_COBCD, SustainPermutation.TWO_SUSTAIN_COBWEP],
    3: [SustainPermutation.THREE_SUSTAIN_COBCOBCOB, SustainPermutation.THREE_SUSTAIN_COBCDCD, SustainPermutation.THREE_SUSTAIN_COBCDWEP],
    4: [SustainPermutation.FOUR_SUSTAIN_COBCOBCDCD, SustainPermutation.FOUR_SUSTAIN_COBCOBWEPWEP],
});
const Sustain = Object.freeze({
    NO: "",
    WEP: "wep",
    CD: "cd",
    COB: "cob",
});
const knownDamageTrinkets = new Set([
    ids.ids.TRINKET_RESTRAINED_ESSENCE,
    ids.ids.TRINKET_TOEP,
    ids.ids.TRINKET_ZHC,
]);
const trinketToCast = Object.freeze({
    [ids.ids.TRINKET_RESTRAINED_ESSENCE]: "EssenceOfSapphiron",
    [ids.ids.TRINKET_TOEP]: "EphemeralPower",
    [ids.ids.TRINKET_ZHC]: "UnstablePower",
    [ids.ids.TRINKET_MQG]: "MindQuickening",
});
const trinketToCooldown = Object.freeze({
    [ids.ids.TRINKET_RESTRAINED_ESSENCE]: common.cooldowns.ESSENCE_OF_SAPPHIRON,
    [ids.ids.TRINKET_TOEP]: common.cooldowns.EPHEMERAL_POWER,
    [ids.ids.TRINKET_ZHC]: common.cooldowns.UNSTABLE_POWER,
    [ids.ids.TRINKET_MQG]: common.cooldowns.MIND_QUICKENING,
});
const trinketToAura = Object.freeze({
    [ids.ids.TRINKET_RESTRAINED_ESSENCE]: common.auras.ESSENCE_OF_SAPPHIRON,
    [ids.ids.TRINKET_TOEP]: common.auras.EPHEMERAL_POWER,
    [ids.ids.TRINKET_ZHC]: common.auras.UNSTABLE_POWER,
    [ids.ids.TRINKET_MQG]: common.auras.MIND_QUICKENING,
});
/* see charts on page 6 of https://github.com/ronkuby-mage/vanilla-firemage/ignite.pdf */
const getSustainPermutations = (staticTime, numMages, averageCrit) => {
    let sustainPermutations = [];
    const regularMages = Math.min(numMages, 6);
    const critLookup = Math.max(Math.round(averageCrit) - ignite.minimum, 0);
    let ranked = [];
    
    if (regularMages == 1) {
        sustainPermutations.push(SustainPermutation.ONE_SUSTAIN);
    } else if (regularMages == 2) {
        sustainPermutations.push(SustainPermutation.TWO_SUSTAIN_COBCOB);
        sustainPermutations.push(SustainPermutation.TWO_SUSTAIN_COBCD);
        sustainPermutations.push(SustainPermutation.TWO_SUSTAIN_COBWEP);
    } else if (regularMages >= 3) {
        let table = ignite[regularMages];
        for (const [key, value] of Object.entries(table)) {
            if (value.length > critLookup) {
                ranked.push([value[critLookup], key]);
            } else {
                ranked.push([Infinity, key]);
            }
        }
        let sorted = ranked.sort((a, b) => a[0] - b[0]);
        if (sorted.length == 0 || sorted[0].length == 0 || sorted[0][0] == Infinity) {
            sustainPermutations.push(SustainPermutation.NO_SUSTAIN);
        } else {
            let factor = durationCriteria;
            while (sustainPermutations.length == 0 && factor < 999) {
                sorted.forEach(values => {
                    if (values[0] >= staticTime/factor && values[0] < staticTime*factor) {
                        sustainPermutations.push(...rotationsBySustains[values[1]]);
                    }
                });
                factor *= durationCriteria;
            }
        }
    }

    return sustainPermutations;
};

const getPlayerApl = (preScorch, bufferSpell, derivedOpening, sustain, playerTrinket, havePI, isLastPlayer, numMages) => {
    let playerApl = apl.apl();
    playerApl.id = common.uuid();
    playerApl.name = "";

    let fixedSequence = apl.action();
    fixedSequence.id = "fixed-sequence";
    fixedSequence.key = "Sequence";
    if (derivedOpening == OpeningPermutation.TWO_TRINKETS) { // damage + MQG, doing damage
        fixedSequence.sequence.push(apl.getAction(trinketToCast[playerTrinket[0]]));
    } else if (derivedOpening == OpeningPermutation.MQG && preScorch == PreScorch.PRESCORCH_NO) {
        fixedSequence.sequence.push(apl.getAction("MindQuickening"));
    }
    if (preScorch == PreScorch.PRESCORCH_YES) {
        const scorches = preScorches[Math.min(Math.max(numMages, 1), 6)];
        for (let i = 0; i < scorches; i++) {
            fixedSequence.sequence.push(apl.getAction("Scorch"));
        }
    }
    if (derivedOpening == OpeningPermutation.TWO_DAMAGE || derivedOpening == OpeningPermutation.ACTIVE_DAMAGE) {
        // slot 2 priority
        fixedSequence.sequence.push(apl.getAction(trinketToCast[playerTrinket[1]]));
    }
    fixedSequence.sequence.push(apl.getAction("Combustion"));
    if (bufferSpell == Buffer.BUFFER_PYROBLAST) {
        fixedSequence.sequence.push(apl.getAction("Pyroblast"));
        if (havePI) {
            fixedSequence.sequence.push(apl.getAction("PowerInfusion"));
        }
    } else {
        fixedSequence.sequence.push(apl.getAction("Fireball"));
        if (havePI) {
            fixedSequence.sequence.push(apl.getAction("PowerInfusion"));
        }
        if (bufferSpell == Buffer.BUFFER_FIREBALL) {
            fixedSequence.sequence.push(apl.getAction("None"));
        }
    }
    if (derivedOpening == OpeningPermutation.MQG && preScorch == PreScorch.PRESCORCH_YES) {
        fixedSequence.sequence.push(apl.getAction("MindQuickening"));
    }

    let cond, item, items = [];

    // check for trinket cooldown
    if (derivedOpening == OpeningPermutation.TWO_DAMAGE || derivedOpening == OpeningPermutation.TWO_TRINKETS) {
        let spell = "MindQuickening";
        let cooldown = common.cooldowns.MIND_QUICKENING;
        if (derivedOpening == OpeningPermutation.TWO_DAMAGE) {
            spell = trinketToCast[playerTrinket[0]];
            cooldown = trinketToCooldown[playerTrinket[0]];
        }

        item = apl.item();
        item.condition.condition_type = apl.condition_type.TRUE;
        item.condition.values = [apl.value()];
        item.condition.values[0].value_type = apl.value_type.PLAYER_COOLDOWN_EXISTS;
        item.condition.values[0].vint = cooldown;
        item.action = apl.getAction(spell);
        items.push(item);
    }

    // active priority
    if (sustain == Sustain.CD) {
        item = apl.item();
        if (!auraTriggers.has(derivedOpening)) {
            item.condition.condition_type = apl.condition_type.TRUE;
            item.condition.values = [apl.value()];
            item.condition.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
            item.condition.values[0].vint = common.auras.COMBUSTION;
        } else {
            item.condition.condition_type = apl.condition_type.TRUE;
            item.condition.condition_type = apl.condition_type.OR;
            cond = apl.condition();
            cond.condition_type = apl.condition_type.TRUE;
            cond.values = [apl.value()];
            cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
            cond.values[0].vint = common.auras.COMBUSTION;
            item.condition.conditions.push(cond);
            if (derivedOpening == OpeningPermutation.MQG) {
                cond = apl.condition();
                cond.condition_type = apl.condition_type.TRUE;
                cond.values = [apl.value()];
                cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
                cond.values[0].vint = common.auras.MIND_QUICKENING;
                item.condition.conditions.push(cond);
            }
            if (derivedOpening == OpeningPermutation.ACTIVE_DAMAGE || derivedOpening == OpeningPermutation.TWO_DAMAGE) {
                cond = apl.condition();
                cond.condition_type = apl.condition_type.TRUE;
                cond.values = [apl.value()];
                cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
                cond.values[0].vint = trinketToCooldown[playerTrinket[1]];
                item.condition.conditions.push(cond);
            }
            if (derivedOpening == OpeningPermutation.TWO_TRINKETS || derivedOpening == OpeningPermutation.TWO_DAMAGE) {
                cond = apl.condition();
                cond.condition_type = apl.condition_type.TRUE;
                cond.values = [apl.value()];
                cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
                cond.values[0].vint = trinketToCooldown[playerTrinket[0]];
                item.condition.conditions.push(cond);
            }
        }
        item.action = apl.getAction("Fireball");
        items.push(item);
    }

    if (sustain == Sustain.CD || sustain == Sustain.COB) {
        item = apl.item();
        item.condition.condition_type = apl.condition_type.TRUE;
        item.condition.condition_type = apl.condition_type.AND;
        cond = apl.condition();
        cond.condition_type = apl.condition_type.CMP;
        cond.op = apl.condition_op.EQ;    
        cond.values = [apl.value(), apl.value()];
        cond.values[0].value_type = apl.value_type.TARGET_AURA_STACKS;
        cond.values[0].vint = common.auras.IGNITE;
        cond.values[1].value_type = apl.value_type.CONST;
        cond.values[1].vfloat = 5;
        item.condition.conditions.push(cond);
        cond = apl.condition();
        cond.condition_type = apl.condition_type.CMP;
        cond.op = apl.condition_op.LT;
        cond.values = [apl.value(), apl.value()];
        cond.values[0].value_type = apl.value_type.TARGET_AURA_DURATION;
        cond.values[0].vint = common.auras.IGNITE;
        cond.values[1].value_type = apl.value_type.CONST;
        cond.values[1].vfloat = 1.5;
        item.condition.conditions.push(cond);
        cond = apl.condition();
        cond.condition_type = apl.condition_type.FALSE;
        cond.values = [apl.value()];
        cond.values[0].value_type = apl.value_type.PLAYER_COOLDOWN_EXISTS;
        cond.values[0].vint = common.cooldowns.FIRE_BLAST;
        item.condition.conditions.push(cond);
        item.action = apl.getAction("FireBlast");
        items.push(item);
    }

    if (sustain == Sustain.CD || sustain == Sustain.COB || sustain == Sustain.WEP) {
        item = apl.item();
        item.condition.condition_type = apl.condition_type.TRUE;
        item.condition.condition_type = apl.condition_type.OR;
        cond = apl.condition();
        cond.condition_type = apl.condition_type.CMP;
        cond.op = apl.condition_op.EQ;    
        cond.values = [apl.value(), apl.value()];
        cond.values[0].value_type = apl.value_type.TARGET_AURA_STACKS;
        cond.values[0].vint = common.auras.IGNITE;
        cond.values[1].value_type = apl.value_type.CONST;
        cond.values[1].vfloat = 5;
        item.condition.conditions.push(cond);
        cond = apl.condition();
        cond.condition_type = apl.condition_type.CMP;
        cond.op = apl.condition_op.LT;    
        cond.values = [apl.value(), apl.value()];
        cond.values[0].value_type = apl.value_type.TARGET_AURA_DURATION;
        cond.values[0].vint = common.auras.FIRE_VULNERABILITY;
        cond.values[1].value_type = apl.value_type.CONST;
        cond.values[1].vfloat = 5;
        item.condition.conditions.push(cond);
        item.action = apl.getAction("Scorch");
        items.push(item);       
    }

    // one player needs to hold scorch
    if (sustain == Sustain.NO && isLastPlayer && preScorch == PreScorch.PRESCORCH_YES) {
        item = apl.item();
        item.condition.condition_type = apl.condition_type.CMP;
        item.condition.op = apl.condition_op.LT;
        item.condition.values = [apl.value(), apl.value()];
        item.condition.values[0].value_type = apl.value_type.TARGET_AURA_DURATION;
        item.condition.values[0].vint = common.auras.FIRE_VULNERABILITY;
        item.condition.values[1].value_type = apl.value_type.CONST;
        item.condition.values[1].vfloat = 5;
        item.action = apl.getAction("Scorch");
        items.push(item);        
    }
    
    let defaultAction = apl.getAction("Fireball");

    playerApl.fixedSequence.action = _.cloneDeep(fixedSequence);
    playerApl.items = _.cloneDeep(items);
    playerApl.defaultAction.action = _.cloneDeep(defaultAction);

    return playerApl;
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
        const bufferSpells = [];
        if (preScorch == PreScorch.PRESCORCH_NO) {
            bufferSpells.push(Buffer.BUFFER_NOTHING);
        } else {
            if (encounterDuration >= shortFightThreshold) {
                bufferSpells.push(Buffer.BUFFER_PYROBLAST);
            }
            if (encounterDuration < mediumFightThreshold) {
                bufferSpells.push(Buffer.BUFFER_NOTHING);
            }
            if (encounterDuration < longFightThreshold) {
                bufferSpells.push(Buffer.BUFFER_FIREBALL);
            }

        }
        bufferSpells.forEach(bufferSpell => {
            let openingPermutations = [];
            if (isPreset) {
                if (encounterDuration < longFightThreshold) {
                    openingPermutations.push(OpeningPermutation.MQG);
                }
                if (encounterDuration >= shortFightThreshold && encounterDuration < longFightThreshold) {
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
                const buildTime = staticCondition[buildMages] + (50.0 - averageCrit)/5.0;
                const staticTime = Math.max(encounterDuration - buildTime, 0.0)
                let sustainPermutations = [];

                if (preScorch == PreScorch.PRESCORCH_NO) {
                    // if we are not prescorching, it is unlikely sustaining ignite is a focus
                    sustainPermutations = [SustainPermutation.NO_SUSTAIN];
                } else {
                    sustainPermutations = getSustainPermutations(staticTime, templateRaid.players.length, averageCrit);
                }

                sustainPermutations.forEach(sustainPermutation => {
                    let desc = [];
                    if (preScorchPermutations.length > 1) {
                        desc.push(preScorch);
                    }
                    if (bufferSpells.length > 1) {
                        desc.push(bufferSpell);
                    }
                    if (openingPermutations.length > 1) {
                        desc.push(openingPermutation);
                    }
                    if (sustainPermutations.length > 1) {
                        desc.push(sustainPermutation);
                    }
                    const description = desc.map(str => str !== "" ? str + " " : str);
                    let count = 0;
                    const newRaid = _.cloneDeep(templateRaid);
                    newRaid.id = common.uuid();
                    newRaid.name = `${namePrefix} ${description.join("")}`;
                    newRaid.players.forEach(player => {
                        player.id = common.uuid();
                        let derivedOpening = openingPermutation;                
                        let playerTrinket = [];
                        // check if we need trinket swap
                        if (isPreset) {
                            if (openingPermutation == OpeningPermutation.ACTIVE_DAMAGE) {
                                if (naxxTrinketAvailability) {
                                    playerTrinket = [ids.ids.TRINKET_MARK_OF_THE_CHAMPION, ids.ids.TRINKET_RESTRAINED_ESSENCE];
                                } else {
                                    playerTrinket = [ids.ids.TRINKET_TEAR, ids.ids.TRINKET_ZHC];
                                }
                            } else if (openingPermutation == OpeningPermutation.MQG) {
                                if (naxxTrinketAvailability) {
                                    playerTrinket = [ids.ids.TRINKET_MARK_OF_THE_CHAMPION, ids.ids.TRINKET_MQG];
                                } else {
                                    playerTrinket = [ids.ids.TRINKET_TEAR, ids.ids.TRINKET_MQG];
                                }
                            } else if (openingPermutation == OpeningPermutation.TWO_TRINKETS) {
                                if (naxxTrinketAvailability) {
                                    playerTrinket = [ids.ids.TRINKET_RESTRAINED_ESSENCE, ids.ids.TRINKET_MQG];
                                } else {
                                    playerTrinket = [ids.ids.TRINKET_ZHC, ids.ids.TRINKET_MQG];
                                }
                            }
                            player.loadout.trinket1 = _.cloneDeep({ item_id: playerTrinket[0], enchant_id: null });
                            player.loadout.trinket2 = _.cloneDeep({ item_id: playerTrinket[1], enchant_id: null });
                        } else {
                            if (player.loadout.trinket1.item_id == ids.ids.TRINKET_MQG || player.loadout.trinket2.item_id == ids.ids.TRINKET_MQG) {
                                if (knownDamageTrinkets.has(player.loadout.trinket1.item_id) || knownDamageTrinkets.has(player.loadout.trinket2.item_id)) {
                                    derivedOpening = OpeningPermutation.TWO_TRINKETS;
                                    if (player.loadout.trinket1.item_id == ids.ids.TRINKET_MQG) {
                                        playerTrinket = [player.loadout.trinket2.item_id, ids.ids.TRINKET_MQG];
                                    } else {
                                        playerTrinket = [player.loadout.trinket1.item_id, ids.ids.TRINKET_MQG];
                                    }
                                } else {
                                    derivedOpening = OpeningPermutation.MQG;
                                    playerTrinket = [0, ids.ids.TRINKET_MQG];
                                }
                            } else if (knownDamageTrinkets.has(player.loadout.trinket1.item_id) && knownDamageTrinkets.has(player.loadout.trinket2.item_id)) {
                                derivedOpening = OpeningPermutation.TWO_DAMAGE;
                                playerTrinket = [player.loadout.trinket1.item_id, player.loadout.trinket2.item_id];
                            } else if (knownDamageTrinkets.has(player.loadout.trinket1.item_id)) {
                                derivedOpening = OpeningPermutation.ACTIVE_DAMAGE;
                                playerTrinket = [0, player.loadout.trinket1.item_id];
                            } else if (knownDamageTrinkets.has(player.loadout.trinket2.item_id)) {
                                derivedOpening = OpeningPermutation.ACTIVE_DAMAGE;
                                playerTrinket = [0, player.loadout.trinket2.item_id];
                            }
                        }
                        // done collecting/setting player trinkets
                        // time to build the player apl
                        count += 1;
                        const isLastPlayer = templateRaid.players.length == count;
                        const isSecondLastPlayer = templateRaid.players.length - 1 == count;
                        const isThirdLastPlayer = templateRaid.players.length - 2 == count;
                        const isFourthLastPlayer = templateRaid.players.length - 3 == count;
                        let sustain = Sustain.NO;
                        if ((isLastPlayer && ((sustainPermutation == SustainPermutation.ONE_SUSTAIN) ||
                                            (sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBCOB) ||
                                            (sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBCD) ||
                                            (sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBWEP) ||
                                            (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCOBCOB) ||
                                            (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDCD) ||
                                            (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDWEP) ||
                                            (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBCDCD) ||
                                            (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBWEPWEP))) ||
                            (isSecondLastPlayer && ((sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBCOB) ||
                                            (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCOBCOB) ||
                                            (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBCDCD) ||
                                            (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBWEPWEP))) ||
                            (isThirdLastPlayer && (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCOBCOB))) {
                            sustain = Sustain.COB;
                        }
                        if ((isSecondLastPlayer && ((sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBCD) ||
                                                   (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDCD) ||
                                                   (sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDWEP))) ||
                            (isThirdLastPlayer && ((sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDCD) ||
                                                  (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBCDCD))) ||
                            (isFourthLastPlayer && (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBCDCD))) {
                            sustain = Sustain.CD;
                        }
                        if ((isSecondLastPlayer && sustainPermutation == SustainPermutation.TWO_SUSTAIN_COBWEP) ||
                            (isThirdLastPlayer && ((sustainPermutation == SustainPermutation.THREE_SUSTAIN_COBCDWEP) ||
                                                  (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBWEPWEP))) ||
                            (isFourthLastPlayer && (sustainPermutation == SustainPermutation.FOUR_SUSTAIN_COBCOBWEPWEP))) {
                            sustain = Sustain.WEP;
                        }
                        const playerApl = getPlayerApl(preScorch, bufferSpell, derivedOpening, sustain, playerTrinket, player.pi_count > 0, isLastPlayer, templateRaid.players.length);
                        player.apl = _.cloneDeep(playerApl);
                    });
                    generatedRaids.push(newRaid);
                });
            });
        });
    });

    return generatedRaids;
};

// Export additional utility functions if needed
export const createVariationOptions = (baseOptions) => {
    // Helper to create different variation configurations
    return baseOptions;
};