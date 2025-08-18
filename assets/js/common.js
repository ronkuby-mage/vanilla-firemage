export default {
    uuid() {
        return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function(c) {
            let r = Math.random() * 16 | 0, v = c == "x" ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    },
    stats() {
        return {
            int: 0,
            spi: 0,
            mp5: 0,
            crit: 0,
            hit: 0,
            sp: 0,
            sp_arcane: 0,
            sp_fire: 0,
            sp_frost: 0,
            sp_nature: 0,
            sp_shadow: 0,
            spell_penetration: 0,
            mana: 0,
        }
    },
    baseTalents() {
        return new Array(49).fill(0);
    },
    parseTalents(url) {
        let m;
        if (m = url.match(/\/talent-calc\/mage\/([0-9\-]+)/))
            return this.parseWowheadTalents(m[1]);
        return null;
    },
    parseWowheadTalents(str) {
        let talents = this.baseTalents();
        let trees = [0, 16, 32];
        let tree = 0;
        let index = 0;
        let arr = str.split("");
        for (let value of arr) {
            if (value == "-") {
                tree++;
                index = trees[tree];
            }
            else {
                talents[index] = parseInt(value);
                index++;
            }
        }
        return talents;
    },
    foods: {
        NONE: 0,
        WELL_FED: 17222,
        NIGHTFIN: 13931,
        RUNN_TUM: 18254,
    },
    flasks: {
        NONE: 0,
        SUPREME_POWER: 13512,
        DISTILLED_WISDOM: 13511,
    },
    weapon_oils: {
        NONE: 0,
        BLESSED_WIZARD: 23123,
        BRILLIANT_WIZARD: 20749,
        WIZARD: 25128,
        BRILLIANT_MANA: 20748,
    },
    auras: {
        ARCANE_POWER: 12042,
        BERSERKING: 20554,
        CLEARCAST: 12536,
        COMBUSTION: 29977,
        EVOCATION: 12051,
        FIRE_VULNERABILITY: 22959,
        INNERVATE: 29166,
        IGNITE: 12654,
        PRESENCE_OF_MIND: 12043,
        POWER_INFUSION: 10060,
        WINTERS_CHILL: 12579,
        ARCANE_POTENCY: 24544,
        BLUE_DRAGON: 23688,
        BURST_OF_KNOWLEDGE: 15646,
        CHAOS_FIRE: 24389,
        CHROMATIC_INFUSION: 27675,
        EPHEMERAL_POWER: 23271,
        ESSENCE_OF_SAPPHIRON: 28779,
        MIND_QUICKENING: 23723,
        NAT_PAGLE: 24610,
        OBSIDIAN_INSIGHT: 26166,
        UNSTABLE_POWER: 24658,
        ENIGMAS_ANSWER: 26129,
        NETHERWIND_FOCUS: 22007,
    },
    cooldowns: {
        ARCANE_POWER: 12042,
        BERSERKING: 20554,
        COLD_SNAP: 11958,
        COMBUSTION: 29977,
        EVOCATION: 12051,
        FIRE_BLAST: 10199,
        PRESENCE_OF_MIND: 12043,
        // Trinkets
        ARCANE_POTENCY: 24544,
        BURST_OF_KNOWLEDGE: 15646,
        CHAOS_FIRE: 24389,
        CHROMATIC_INFUSION: 27675,
        EPHEMERAL_POWER: 23271,
        ESSENCE_OF_SAPPHIRON: 28779,
        MANA_INFUSION: 28760,
        MIND_QUICKENING: 23723,
        NAT_PAGLE: 24610,
        OBSIDIAN_INSIGHT: 26166,
        UNSTABLE_POWER: 24658,
        // Items
        CELESTIAL_ORB: 9253,
        ENGULFING_SHADOWS: 27860,
        MANA_GEM: 10058,
        MANA_POTION: 17531,
        ROBE_ARCHMAGE: 18385,
    }
};