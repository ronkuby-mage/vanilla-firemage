import apl from "./apl";
import common from "./common";
import _ from "lodash";

const defaultApls = () => {
    let data = [];

    let blank = apl.apl();
    blank.id = "preset-blank";
    blank.name = "Blank";
    blank.defaultAction.action = apl.getAction("Fireball");
    data.push(blank);

    return data;
};


export default {
    apls: defaultApls(),
    talents: [
        { name: "Deep Fire", talents: common.parseWowheadTalents("23000502-5052122123033151-003") },
        { name: "AP Fire", talents: common.parseWowheadTalents("2300250310231531-505202012-003") },
        { name: "AP Frost", talents: common.parseWowheadTalents("2300250310231531--053500030022") },
        { name: "WC Frost", talents: common.parseWowheadTalents("2300250300231--0535000300230135") },
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
            hands: { item_id: 18808, enchant_id: 25078 },
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