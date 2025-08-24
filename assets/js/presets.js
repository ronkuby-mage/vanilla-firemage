import apl from "./apl";
import common from "./common";
import _ from "lodash";

const defaultApls = () => {
    let data = [];
    let item, cond, initial_actions, items;

    let new_fire = apl.apl();
    new_fire.id = "fire-naxx";
    new_fire.name = "Fire (Naxx)";

    // default fire
    initial_actions = apl.action();
    initial_actions.id = "fixed-sequence-action";
    initial_actions.key = "Sequence";
    initial_actions.sequence = [
        apl.getAction("Scorch"),
        apl.getAction("Scorch"),
        apl.getAction("EssenceOfSapphiron"),
        apl.getAction("Combustion"),
        apl.getAction("Pyroblast"),
        apl.getAction("PowerInfusion"),
    ];   

    new_fire.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(new_fire);

    // maintain scorch
    let maintain_scorch = apl.apl();
    maintain_scorch.id = "fire-naxx-maintain-scorch";
    maintain_scorch.name = "Maintain Scorch";
    items = [];    

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

    maintain_scorch.items = _.cloneDeep(items);
    maintain_scorch.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(maintain_scorch);

    // spam scorch
    let spam_scorch = apl.apl();
    spam_scorch.id = "fire-naxx-scorch";
    spam_scorch.name = "Spam Scorch";
    spam_scorch.defaultAction.action = apl.getAction("Scorch");
    items = [];

    item = apl.item();
    item.condition.condition_type = apl.condition_type.TRUE;
    item.condition.condition_type = apl.condition_type.OR;
    cond = apl.condition();
    cond.condition_type = apl.condition_type.TRUE;
    cond.values = [apl.value()];
    cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    cond.values[0].vint = common.auras.COMBUSTION;
    item.condition.conditions.push(cond);
    cond = apl.condition();
    cond.condition_type = apl.condition_type.TRUE;
    cond.values = [apl.value()];
    cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    cond.values[0].vint = common.auras.ESSENCE_OF_SAPPHIRON;
    item.condition.conditions.push(cond);
    item.action = apl.getAction("Fireball");
    items.push(item);

    spam_scorch.items = _.cloneDeep(items);
    spam_scorch.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(spam_scorch);
    
    // scorch wip
    let scorch_wip = apl.apl();
    scorch_wip.id = "fire-naxx-scorch-WIP";
    scorch_wip.name = "Scorch with Extreme Predujice";
    items = [];    

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

    scorch_wip.items = _.cloneDeep(items);
    scorch_wip.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(scorch_wip);

    // cobimf
    let cob_imf = apl.apl();
    cob_imf.id = "fire-naxx-cob-imf";
    cob_imf.name = "COBIMF";
    items = [];    

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
 
    cob_imf.items = _.cloneDeep(items);
    cob_imf.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(cob_imf);

    // cdimf
    let cd_imf = apl.apl();
    cd_imf.id = "fire-naxx-cd-imf";
    cd_imf.name = "CDIMF";
    items = [];    

    item = apl.item();
    item.condition.condition_type = apl.condition_type.TRUE;
    item.condition.condition_type = apl.condition_type.OR;
    cond = apl.condition();
    cond.condition_type = apl.condition_type.TRUE;
    cond.values = [apl.value()];
    cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    cond.values[0].vint = common.auras.COMBUSTION;
    item.condition.conditions.push(cond);
    cond = apl.condition();
    cond.condition_type = apl.condition_type.TRUE;
    cond.values = [apl.value()];
    cond.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    cond.values[0].vint = common.auras.ESSENCE_OF_SAPPHIRON;
    item.condition.conditions.push(cond);
    item.action = apl.getAction("Fireball");
    items.push(item);

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
 
    cd_imf.items = _.cloneDeep(items);
    cd_imf.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(cd_imf);

    // amber
    let amber = apl.apl();
    amber.id = "fire-naxx-amber";
    amber.name = "Amber Special";
    items = [];    

    item = apl.item();
    item.condition.condition_type = apl.condition_type.FALSE;
    item.condition.values = [apl.value()];
    item.condition.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    item.condition.values[0].vint = common.auras.POWER_INFUSION;
    item.action = apl.getAction("PowerInfusion");
    items.push(item);

    amber.items = _.cloneDeep(items);
    amber.fixedSequence.action = _.cloneDeep(initial_actions);
    data.push(amber);


    let blank = apl.apl();
    blank.id = "preset-blank";
    blank.name = "Blank";
    data.push(blank);

    return data;
};


export default {
    apls: defaultApls(),
    talents: [
        { name: "Fire", talents: common.parseWowheadTalents("23000502-5052122123033151-003") },
    ],
    loadouts: [{
        name: "Molten Core",
        loadout: {
            head: { item_id: 16795, enchant_id: 22844 },
            neck: { item_id: 18814, enchant_id: null },
            shoulder: { item_id: 11782, enchant_id: null },
            back: { item_id: 17078, enchant_id: null },
            chest: { item_id: 14152, enchant_id: 20025 },
            wrist: { item_id: 16799, enchant_id: 20008 },
            hands: { item_id: 16801, enchant_id: null },
            waist: { item_id: 19136, enchant_id: null },
            legs: { item_id: 16915, enchant_id: 22844 },
            feet: { item_id: 16800, enchant_id: 13890 },
            finger1: { item_id: 19147, enchant_id: null },
            finger2: { item_id: 19147, enchant_id: null },
            trinket1: { item_id: 18820, enchant_id: null },
            trinket2: { item_id: 12930, enchant_id: null },
            main_hand: { item_id: 17103, enchant_id: 22749 },
            off_hand: { item_id: "10796:1965", enchant_id: null },
            ranged: { item_id: "15283:1959", enchant_id: null }
        }
    }, {
        name: "Phase 5 Exit",
        loadout: {
            head: { item_id: 19375, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 19370, enchant_id: 24421 },
            back: { item_id: 22731, enchant_id: null },
            chest: { item_id: 21343, enchant_id: 20025 },
            wrist: { item_id: 21186, enchant_id: 20008 },
            hands: { item_id: 21585, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 21676, enchant_id: 24164 },
            feet: { item_id: 21344, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 21836, enchant_id: null },
            trinket1: { item_id: 19379, enchant_id: null },
            trinket2: { item_id: 19339, enchant_id: null },
            main_hand: { item_id: 21622, enchant_id: 22749 },
            off_hand: { item_id: 21597, enchant_id: null },
            ranged: { item_id: 21603, enchant_id: null }
        }
    }, {
        name: "Naxxramas",
        loadout: {            
            head: { item_id: 22498, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 22983, enchant_id: 29467 },
            back: { item_id: 23050, enchant_id: null },
            chest: { item_id: 22496, enchant_id: 20025 },
            wrist: { item_id: 21186, enchant_id: 20008 },
            hands: { item_id: 21585, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 23070, enchant_id: 24164 },
            feet: { item_id: 22500, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 23237, enchant_id: null },
            trinket1: { item_id: 19339, enchant_id: null },
            trinket2: { item_id: 23046, enchant_id: null },
            main_hand: { item_id: 22589, enchant_id: 22749 },
            off_hand: { item_id: null, enchant_id: null },
            ranged: { item_id: 22821, enchant_id: null }
        }
    }, {
        name: "Naxxramas (Undead)",
        loadout: {
            head: { item_id: 22498, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 22983, enchant_id: 29467 },
            back: { item_id: 23050, enchant_id: null },
            chest: { item_id: 23085, enchant_id: 20025 },
            wrist: { item_id: 23091, enchant_id: 20008 },
            hands: { item_id: 23084, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 23070, enchant_id: 24164 },
            feet: { item_id: 21344, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 23031, enchant_id: null },
            trinket1: { item_id: 23207, enchant_id: null },
            trinket2: { item_id: 23046, enchant_id: null },
            main_hand: { item_id: 22589, enchant_id: 22749 },
            off_hand: { item_id: null, enchant_id: null },
            ranged: { item_id: 22820, enchant_id: null }
        }
    }],
};