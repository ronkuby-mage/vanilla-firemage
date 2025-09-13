import items from "./items";

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
    },
    raceFaction(race) {
        return ["Gnome", "Human"].indexOf(race) != -1 ? "Alliance" : "Horde";
    },
    baseStats(race) {
        let stats = this.stats();
        stats.crit = 0.2;

        if (race == "Gnome") {
            stats.int = 132;
            stats.spi = 120;
        }
        else if (race == "Human") {
            stats.int = 125;
            stats.spi = 126;
        }
        else if (race == "Troll") {
            stats.int = 121;
            stats.spi = 121;
        }
        else if (race == "Undead") {
            stats.int = 123;
            stats.spi = 125;
        }

        return stats;
    },
    addStats(a, b) {
        let stats = this.stats();
        let val = (v) => {
            v = parseFloat(v);
            return isNaN(v) ? 0 : v;
        };
        for (let key in stats)
            stats[key] = val(_.get(a, key, 0)) + val(_.get(b, key, 0));
        return stats;
    },
    loadoutStats(loadout) {

        let stats = this.stats();
        let sets = {};

        for (let slot in loadout) {
            
            let item = this.getItem(loadout[slot].item_id);
            if (item) {
                stats = this.addStats(stats, item);
                let enchant = this.getEnchant(loadout[slot].enchant_id);
                if (enchant)
                    stats = this.addStats(stats, enchant);

                if (item.set) {
                    if (!sets.hasOwnProperty(item.set)) {
                        let set = _.find(items.sets, {id: item.set});
                        sets[item.set] = {
                            set: set,
                            n: 1,
                        };
                    }
                    else {
                        sets[item.set].n++;
                        if (sets[item.set].set) {
                            let setbonus = _.get(sets[item.set].set, "set"+sets[item.set].n);
                            if (setbonus)
                                stats = this.addStats(stats, setbonus);
                        }
                    }
                }
            }
        }
        return stats;
    },
    loadoutSlotToItemSlot(slot) {
        return slot.replace(/[0-9]+/g, "");
    },
    isCustomItem(id) {
        if (typeof(id) == "object")
            id = id.id;
        return typeof(id) == "string" && id.indexOf("custom_") === 0;
    },
    itemUrl(id) {
        if (this.isCustomItem(id))
            return null;
        if (typeof(id) == "object")
            id = id.id;
        if (typeof(id) == "string")
            id = id.replace(":", "&rand=");
        return "https://www.wowhead.com/classic/item="+id;
    },
    getItem(slot, id) {
        if (id === undefined) {
            if (!slot)
                return null;
            id = slot;
            for (let key in items.gear) {
                let item = items.gear[key].find(i => i.id == id);
                if (item)
                    return item;
            }
        }
        else {
            if (!id)
                return null;
            slot = this.loadoutSlotToItemSlot(slot);
            let item = items.gear[slot].find(i => i.id == id);
            if (item)
                return item;
        }
        let item = customItems.value.find(i => i.id == id);
        if (item)
            return item;

        return null;
    },
    getEnchant(slot, id) {
        if (id === undefined) {
            id = slot;
            for (let key in items.enchants) {
                let item = items.enchants[key].find(i => i.id == id || i.enchantment_id == id);
                if (item)
                    return item;
            }
            return null;
        }
        else {
            slot = loadoutSlotToItemSlot(slot);
            return items.enchants[slot].find(i => i.id == id || i.enchantment_id == id);
        }
    },
    gearUrl(player, slot) {
        let itemSlot = player.loadout[slot];
        if (!itemSlot.item_id)
            return null;
        let item = this.getItem(slot, itemSlot.item_id);
        if (!item)
            return null;
        let url = this.itemUrl(item.id);

        if (itemSlot.enchant_id) {
            let enchant = this.getEnchant(itemSlot.enchant_id);
            if (enchant)
                url+= "&ench="+enchant.enchantment_id;
        }

        if (item.set) {
            let pcs = [];
            for (let key in player.loadout) {
                let itm = this.getItem(key, player.loadout[key].item_id);
                if (_.get(itm, "set") == item.set)
                    pcs.push(itm.id);
            }
            if (pcs.length)
                url+= "&pcs="+pcs.join(":");
        }

        return url;
    },
    loadoutSlots() {
        return [
            "head", "neck", "shoulder", "back", "chest", "wrist",
            "hands", "waist", "legs", "feet",
            "finger1", "finger2", "trinket1", "trinket2",
            "main_hand", "off_hand", "ranged",
        ];
    },
    baseLoadout() {
        let loadout = {};
        for (let slot of this.loadoutSlots()) {
            loadout[slot] = {
                item_id: null,
                enchant_id: null,
            };
        }
        return loadout;
    },
    loadoutSets(loadout) {
        let sets = {};

        for (let slot in loadout) {
            
            let item = this.getItem(loadout[slot].item_id);
            if (item) {
                if (item.set) {
                    if (!sets.hasOwnProperty(item.set)) {
                        let set = _.find(items.sets, {id: item.set});
                        sets[item.set] = 1;
                    }
                    else {
                        sets[item.set]++;
                    }
                }
            }
        }
        return sets;
    },
    displayStats(player) {
        let x;
        let faction = this.raceFaction(player.race);
        let stats = this.baseStats(player.race);
        stats = this.addStats(stats, this.loadoutStats(player.loadout));
        stats = this.addStats(stats, player.bonus_stats);
        stats.crit += player.talents[14]; // arcane instability
        stats.crit += 2*player.talents[28]; // critical mass

        if (player.buffs.arcane_intellect)
            stats.int+= 31;
        if (player.buffs.imp_mark_of_the_wild) {
            x = 12;
            x = x * 1.35;
            stats.int += x;
        }
        if (player.buffs.gift_of_stormwind)
            stats.int+= 30;
        if (player.buffs.infallible_mind)
            stats.int+= 25;
        if (player.buffs.runn_tum_tuber)
            stats.int+= 10;
        if (player.buffs.songflower)
            stats.int+= 15;
        if (player.buffs.blessing_of_kings && faction == "Alliance")
            stats.int*= 1.1;
        if (player.buffs.spirit_of_zandalar)
            stats.int*= 1.15;
        if (player.race == "Gnome")
            stats.int*= 1.05;

        if (player.buffs.elixir_greater_arcane)
            stats.sp+= 35;
        if (player.buffs.elixir_greater_firepower)
            stats.sp_fire+= 40;
        if (player.buffs.elixir_frost_power)
            stats.sp_frost+= 15;
        if (player.buffs.flask_of_supreme_power)
            stats.sp+= 150;
        if (player.buffs.blessed_wizard_oil)
            stats.sp+= 60;
        else if (player.buffs.brilliant_wizard_oil)
            stats.sp+= 36;
        if (player.buffs.very_berry_cream)
            stats.sp+= 23;
        stats.sp += 33*player.buffs.atiesh_warlock;

        if (player.buffs.brilliant_wizard_oil)
            stats.crit+= 1;
        if (player.buffs.rallying_cry)
            stats.crit+= 10;
        if (player.buffs.songflower)
            stats.crit+= 5;
        if (player.buffs.dire_maul_tribute)
            stats.crit+=3;
        stats.crit += stats.int / 59.5;
        stats.crit += 2*player.buffs.atiesh_mage;
        if (player.buffs.moonkin_aura)
            stats.crit += 3;

        stats.int = Math.round(stats.int);

        return stats;
    }  
};