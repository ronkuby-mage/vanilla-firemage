import apl from "./apl";
import common from "./common";
import _ from "lodash";

const defaultApls = () => {
    let data = [];
    let item, cond, initial_actions, items;

    let fire = apl.apl();
    fire.id = "no-scorch";
    fire.name = "Fire (no Scorch)";

    // default fire
    let ns_initial_actions = apl.action();
    ns_initial_actions.id = "no-scorch-sequence";
    ns_initial_actions.key = "Sequence";
    ns_initial_actions.sequence = [
        apl.getAction("Combustion"),
        apl.getAction("MindQuickening"),
        apl.getAction("Fireball"),
        apl.getAction("PowerInfusion"),
    ];

    fire.fixedSequence.action = _.cloneDeep(ns_initial_actions);
    fire.items = [];
    fire.defaultAction.action = apl.getAction("Fireball")
    data.push(fire);

    let two_trink = apl.apl();
    two_trink.id = "two-trink";
    two_trink.name = "Fire (two trinkets)";
    items = [];

    let tt_initial_actions = apl.action();
    tt_initial_actions.id = "scorch-sequence";
    tt_initial_actions.key = "Sequence";
    tt_initial_actions.sequence = [
        apl.getAction("EssenceOfSapphiron"),
        apl.getAction("Scorch"),
        apl.getAction("Scorch"),
        apl.getAction("Combustion"),
        apl.getAction("Pyroblast"),
        apl.getAction("PowerInfusion"),
    ];   

    item = apl.item();
    item.condition.condition_type = apl.condition_type.TRUE;
    item.condition.values = [apl.value()];
    item.condition.values[0].value_type = apl.value_type.PLAYER_COOLDOWN_EXISTS;
    item.condition.values[0].vint = common.cooldowns.MIND_QUICKENING;
    item.action = apl.getAction("MindQuickening");
    items.push(item);


    two_trink.fixedSequence.action = _.cloneDeep(tt_initial_actions);
    two_trink.items = _.cloneDeep(items);
    two_trink.defaultAction.action = apl.getAction("Fireball")
    data.push(two_trink);

    let new_fire = apl.apl();
    new_fire.id = "yes-scorch";
    new_fire.name = "Fire (Scorch first)";

    initial_actions = apl.action();
    initial_actions.id = "scorch-sequence";
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
    new_fire.items = [];
    new_fire.defaultAction.action = apl.getAction("Fireball")
    data.push(new_fire);

    // maintain scorch
    let maintain_scorch = apl.apl();
    maintain_scorch.id = "maintain-scorch";
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

    maintain_scorch.fixedSequence.action = _.cloneDeep(initial_actions);
    maintain_scorch.items = _.cloneDeep(items);
    maintain_scorch.defaultAction.action = apl.getAction("Fireball");
    data.push(maintain_scorch);

    // spam scorch
    let spam_scorch = apl.apl();
    spam_scorch.id = "scorch";
    spam_scorch.name = "Spam Scorch";
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

    spam_scorch.fixedSequence.action = _.cloneDeep(initial_actions);
    spam_scorch.items = _.cloneDeep(items);
    spam_scorch.defaultAction.action = apl.getAction("Scorch");
    data.push(spam_scorch);
    
    // scorch wip
    let scorch_wip = apl.apl();
    scorch_wip.id = "scorch-WIP";
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

    scorch_wip.fixedSequence.action = _.cloneDeep(initial_actions);
    scorch_wip.items = _.cloneDeep(items);
    scorch_wip.defaultAction.action = apl.getAction("Fireball");
    data.push(scorch_wip);

    // cobimf
    let cob_imf = apl.apl();
    cob_imf.id = "cob-imf";
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
 
    cob_imf.fixedSequence.action = _.cloneDeep(initial_actions);
    cob_imf.items = _.cloneDeep(items);
    cob_imf.defaultAction.action = apl.getAction("Fireball");
    data.push(cob_imf);

    // cdimf
    let cd_imf = apl.apl();
    cd_imf.id = "cd-imf";
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
 
    cd_imf.fixedSequence.action = _.cloneDeep(initial_actions);
    cd_imf.items = _.cloneDeep(items);
    cd_imf.defaultAction.action = apl.getAction("Fireball");

    data.push(cd_imf);

    // amber
    let amber = apl.apl();
    amber.id = "amber";
    amber.name = "Amber Special";

    items = [];    

    item = apl.item();
    item.condition.condition_type = apl.condition_type.FALSE;
    item.condition.values = [apl.value()];
    item.condition.values[0].value_type = apl.value_type.PLAYER_AURA_EXISTS;
    item.condition.values[0].vint = common.auras.POWER_INFUSION;
    item.action = apl.getAction("PowerInfusion");
    items.push(item);

    amber.fixedSequence.action = _.cloneDeep(initial_actions);
    amber.items = _.cloneDeep(items);
    amber.defaultAction.action = apl.getAction("Fireball");
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
        name: "Phase 5 Enter",
        loadout: {
            head: { item_id: 19375, enchant_id: 24164 },
            neck: { item_id: 18814, enchant_id: null },
            shoulder: { item_id: 19370, enchant_id: 24421 },
            back: { item_id: 19857, enchant_id: null },
            chest: { item_id: 19682, enchant_id: 20025 },
            wrist: { item_id: 19374, enchant_id: 20008 },
            hands: { item_id: 18808, enchant_id: null },
            waist: { item_id: 19136, enchant_id: null },
            legs: { item_id: 19683, enchant_id: 24164 },
            feet: { item_id: 19684, enchant_id: 13890 },
            finger1: { item_id: 19147, enchant_id: null },
            finger2: { item_id: 19147, enchant_id: null },
            trinket1: { item_id: 19379, enchant_id: null },
            trinket2: { item_id: 19339, enchant_id: null },
            main_hand: { item_id: 19356, enchant_id: 22749 },
            off_hand: { item_id: null, enchant_id: null },
            ranged: { item_id: 19861, enchant_id: null }
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
        name: "Phase 6 Enter (w/ UDC)",
        loadout: {            
            head: { item_id: 21347, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 19370, enchant_id: 24421 },
            back: { item_id: 22731, enchant_id: null },
            chest: { item_id: 23085, enchant_id: 20025 },
            wrist: { item_id: 23091, enchant_id: 20008 },
            hands: { item_id: 23084, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 21676, enchant_id: 24164 },
            feet: { item_id: 21344, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 19403, enchant_id: null },
            trinket1: { item_id: 19379, enchant_id: null },
            trinket2: { item_id: 19339, enchant_id: null },
            main_hand: { item_id: 21622, enchant_id: 22749 },
            off_hand: { item_id: 21597, enchant_id: null },
            ranged: { item_id: 21603, enchant_id: null }
        }
    }, {
        name: "Phase 6 Exit",
        loadout: {            
            head: { item_id: 22498, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 22983, enchant_id: 29467 },
            back: { item_id: 23050, enchant_id: null },
            chest: { item_id: 22496, enchant_id: 20025 },
            wrist: { item_id: 22503, enchant_id: 20008 },
            hands: { item_id: 21585, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 22497, enchant_id: 24164 },
            feet: { item_id: 22500, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 23062, enchant_id: null },
            trinket1: { item_id: 19339, enchant_id: null },
            trinket2: { item_id: 23046, enchant_id: null },
            main_hand: { item_id: 22807, enchant_id: 22749 },
            off_hand: { item_id: 23049, enchant_id: null },
            ranged: { item_id: 22821, enchant_id: null }
        }
    }, {
        name: "Phase 6 Exit (w/ UDC)",
        loadout: {
            head: { item_id: 22498, enchant_id: 24164 },
            neck: { item_id: 21608, enchant_id: null },
            shoulder: { item_id: 22499, enchant_id: 29467 },
            back: { item_id: 23050, enchant_id: null },
            chest: { item_id: 23085, enchant_id: 20025 },
            wrist: { item_id: 23091, enchant_id: 20008 },
            hands: { item_id: 23084, enchant_id: 25078 },
            waist: { item_id: 22502, enchant_id: null },
            legs: { item_id: 22497, enchant_id: 24164 },
            feet: { item_id: 22500, enchant_id: 13890 },
            finger1: { item_id: 21709, enchant_id: null },
            finger2: { item_id: 23062, enchant_id: null },
            trinket1: { item_id: 23207, enchant_id: null },
            trinket2: { item_id: 23046, enchant_id: null },
            main_hand: { item_id: 22807, enchant_id: 22749 },
            off_hand: { item_id: 23049, enchant_id: null },
            ranged: { item_id: 22820, enchant_id: null }
        }
    }, {
        name: "Classic Era",
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
        name: "Classic Era (Two Mages)",
        loadout: {            
            head: { item_id: 22498, enchant_id: 24164 },
            neck: { item_id: 23057, enchant_id: null },
            shoulder: { item_id: 22983, enchant_id: 29467 },
            back: { item_id: 23050, enchant_id: null },
            chest: { item_id: 22496, enchant_id: 20025 },
            wrist: { item_id: 21186, enchant_id: 20008 },
            hands: { item_id: 21585, enchant_id: 25078 },
            waist: { item_id: 22730, enchant_id: null },
            legs: { item_id: 23070, enchant_id: 24164 },
            feet: { item_id: 22500, enchant_id: 13890 },
            finger1: { item_id: 23237, enchant_id: null },
            finger2: { item_id: 23025, enchant_id: null },
            trinket1: { item_id: 23207, enchant_id: null },
            trinket2: { item_id: 23046, enchant_id: null },
            main_hand: { item_id: 22589, enchant_id: 22749 },
            off_hand: { item_id: null, enchant_id: null },
            ranged: { item_id: 22821, enchant_id: null }
        }
    }],
};