<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, watch } from "vue";
import _ from "lodash";
import apl from "../apl";
import common from "../common";
import items from "../items";

const props = defineProps(["modelValue", "player"]);
const emits = defineEmits(["update:modelValue", "save"]);

// DEFINE the migration function
const ensureAplStructure = () => {
    if (!props.modelValue.fixedSequence) {
        props.modelValue.fixedSequence = {
            id: "fixed-sequence",
            status: true,
            condition: apl.condition(),
            action: {
                id: "fixed-sequence-action",
                key: "Sequence",
                target_id: 1,
                sequence: [apl.action()],
            }
        };
    }

    if (!props.modelValue.defaultAction) {
        props.modelValue.defaultAction = {
            id: "default-action",
            status: true,
            action: apl.action()
        };
    }
};

// CALL it immediately after definition - THIS IS WHERE TO PUT IT
ensureAplStructure();

/*
 * Collapse
 */
const collapsed = ref([]);
const isCollapsed = (id) => {
    return collapsed.value.includes(id);
};
const collapseToggle = (id) => {
    if (isCollapsed(id))
        collapsed.value = collapsed.value.filter(i => i !== id);
    else
        collapsed.value.push(id);
};
const collapseAll = () => {
    const allIds = [
        props.modelValue.fixedSequence.id, 
        ...props.modelValue.items.map(item => item.id),
        props.modelValue.defaultAction.id  // ADD THIS LINE
    ];
    if (collapsed.value.length === 0)
        collapsed.value = allIds;
    else
        collapsed.value = [];
};

/*
 * Item
 */
const createItem = () => {
    let item = apl.item();
    props.modelValue.items.push(item);
    changed();
};
const deleteItem = (id) => {
    props.modelValue.items = props.modelValue.items.filter(item => item.id !== id);
    changed();
};
const copyItem = (item) => {
    let copy = _.cloneDeep(item);
    copy.id = common.uuid();
    let index = props.modelValue.items.findIndex(i => i.id === item.id);
    props.modelValue.items.splice(index, 0, copy);
    changed();
};
const statusToggle = (item) => {
    item.status = !item.status;
    changed();
};
const itemTitle = (item) => {
    let arr = [];

    let action = apl.actions().find(a => a.key == item.action.key);
    if (action) {
        if (action.item) {
            let item = items.gear[action.title].find(i => i.id == action.item);
            if (item)
                arr.push(_.capitalize(action.title)+": "+item.title);
            else
                arr.push("Use: "+_.capitalize(action.title));
        }
        else {
            arr.push(action.title);
        }
    }

    if (item.action.key == "Sequence")
        arr.push(item.action.sequence.length+" actions");

    let numCond = (cond) => {
        if (cond.condition_type == apl.condition_type.NONE)
            return 0;
        let n = 1;
        for (let c of cond.conditions)
            n+= numCond(c);
        return n;
    };

    let n = numCond(item.condition);
    if (n == 1)
        arr.push(n+" condition");
    else if (n > 1)
        arr.push(n+" conditions");

    return arr.join(", ");
};

/*
 * Drag move
 */
const el = ref();
const dragging = reactive({
    id: null,
    index: 0,
    start: 0,
    x: 0,
    y: 0,
})
const onDragStart = (e, id) => {
    if (props.modelValue.items.length < 2)
        return;
    dragging.id = id;
    dragging.index = props.modelValue.items.findIndex(item => item.id === id);
    dragging.start = dragging.index;
};
const onDragEnd = () => {
    if (!dragging.id)
        return;

    if (dragging.start !== dragging.index) {
        let item = props.modelValue.items.splice(dragging.start, 1)[0];
        props.modelValue.items.splice(dragging.index > dragging.start ? dragging.index-1 : dragging.index, 0, item);
        changed();
    }

    dragging.id = null;
};
const onDragMove = (e) => {
    if (!dragging.id)
        return;
    dragging.x = e.clientX;
    dragging.y = e.clientY;
    if (el.value) {
        let wrapper = el.value.querySelector(".apl-items");
        for (var i=0; i<wrapper.children.length; i++) {
            let item = wrapper.children[i];
            let rect = item.getBoundingClientRect();
            if (e.clientY < rect.top + rect.height/2) {
                dragging.index = i;
                break;
            }
        }
        if (i == wrapper.children.length)
            dragging.index = props.modelValue.items.length;
    }
};
const isDragTo = (index) => {
    return dragging.id && dragging.index == index && dragging.start != index && dragging.start != index-1;
};
const isDragEnd = computed(() => {
    return dragging.id && dragging.index == props.modelValue.items.length && dragging.start != dragging.index-1;
});

/*
 * Update
 */
// Replace the changed function (around line 163-170) with:
const changed = () => {
    // Ensure modelValue has an id before checking if it's a preset
    if (!props.modelValue.id) {
        props.modelValue.id = common.uuid();
        props.modelValue.name = "";
    } else if (apl.isPreset(props.modelValue.id)) {
        props.modelValue.id = common.uuid();
        props.modelValue.name = "";
    }
    emits("update:modelValue", props.modelValue);
};

/*
 * Events
 */
watch(() => props.modelValue.id, (value) => {
    if (collapsed.value.length) {
        const allIds = [props.modelValue.fixedSequence.id, ...props.modelValue.items.map(i => i.id)];
        collapsed.value = allIds;
    }
});
watch(() => props.modelValue.id, (value) => {
    if (collapsed.value.length) {
        const allIds = [props.modelValue.fixedSequence.id, ...props.modelValue.items.map(i => i.id)];
        collapsed.value = allIds;
    }
});
onMounted(() => {
    
    // Existing event listeners
    window.addEventListener("mousemove", onDragMove);
    window.addEventListener("mouseup", onDragEnd);
});
onUnmounted(() => {
    window.removeEventListener("mousemove", onDragMove);
    window.removeEventListener("mouseup", onDragEnd);
});

/* fixed sequence funcitons */
if (!props.modelValue.fixedSequence) {
    props.modelValue.fixedSequence = {
        id: "fixed-sequence",
        status: true,
        condition: apl.condition(),
        action: {
            id: "fixed-sequence-action",
            key: "Sequence",
            target_id: 1,
            sequence: [apl.action()],
        }
    };
}
// Add methods for managing fixed sequence actions:
const createFixedSequenceAction = () => {
    props.modelValue.fixedSequence.action.sequence.push(apl.action());
    changed();
};
const deleteFixedSequenceAction = (index) => {
    // Don't allow deleting the last action - always keep at least one
    if (props.modelValue.fixedSequence.action.sequence.length > 1) {
        props.modelValue.fixedSequence.action.sequence.splice(index, 1);
    } else {
        // Replace with a new empty action instead of deleting
        props.modelValue.fixedSequence.action.sequence[0] = apl.action();
    }
    changed();
};
/* default action */
if (!props.modelValue.defaultAction) {
    props.modelValue.defaultAction = {
        id: "default-action",
        status: true,
        action: apl.action()
    };
}
const defaultActionTitle = () => {
    let action = apl.defaultActions().find(a => a.key == props.modelValue.defaultAction.action.key);
    if (action) {
        if (action.item) {
            let item = items.gear[action.title].find(i => i.id == action.item);
            if (item)
                return _.capitalize(action.title) + ": " + item.title;
            else
                return "Use: " + _.capitalize(action.title);
        }
        return action.title;
    }
    return "Default Action";
};
</script>

<template>
    <div class="apl" ref="el">
        <button class="btn btn-secondary small collapse-all" @click="collapseAll">
            <template v-if="collapsed.length == 0">Collapse all</template>
            <template v-else>Expand all</template>
        </button>

       <!-- Fixed Sequence Section -->
        <div class="apl-fixed-sequence">
            <div class="fixed-sequence-header">
                <span class="header-title">Opening Sequence</span>
                <span class="header-note">Always executes first</span>
            </div>
            <div
                class="apl-item fixed-sequence"
                :class="[
                    isCollapsed(modelValue.fixedSequence.id) ? 'collapsed' : 'expanded',
                    'status-'+modelValue.fixedSequence.status,
                ]"
            >
                <div class="header">
                    <button class="toggle" @click="collapseToggle(modelValue.fixedSequence.id)">
                        <span v-if="isCollapsed(modelValue.fixedSequence.id)">
                            <micon icon="add" />
                            <tooltip>Expand</tooltip>
                        </span>
                        <span v-else>
                            <micon icon="remove" />
                            <tooltip>Collapse</tooltip>
                        </span>
                    </button>
                    <!-- No delete button for fixed sequence -->
                </div>

                <div class="title" v-if="isCollapsed(modelValue.fixedSequence.id)" @click="collapseToggle(modelValue.fixedSequence.id)">
                    Opening Sequence: {{ modelValue.fixedSequence.action.sequence.length }} actions
                </div>
                <div class="body" v-else>
                    <!-- Fixed sequence body - show sequence management directly -->
                    <div class="fixed-sequence-body">
                        <div class="apl-sequence">
                            <template v-for="(action, index) in modelValue.fixedSequence.action.sequence" :key="action.id">
                                <apl-action
                                    v-model="modelValue.fixedSequence.action.sequence[index]"
                                    :player="props.player"
                                    :deletable="true"
                                    @delete="deleteFixedSequenceAction(index)"
                                    @update:modelValue="changed"
                                />
                            </template>
                            <button class="btn btn-secondary small" @click="createFixedSequenceAction">Add action</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="apl-items" :class="{dragend: isDragEnd}">
            <div class="rotation-header">
                <span class="header-title">Priority Rotation</span>
                <span class="header-note">Evaluated in order after opening sequence</span>
            </div>          
            <div
                class="apl-item"
                :class="[
                    isCollapsed(item.id) ? 'collapsed' : 'expanded',
                    isDragTo(index) ? 'dragto' : '',
                    dragging.id == item.id ? 'dragfrom' : '',
                    'status-'+item.status,
                ]"
                v-for="(item, index) in modelValue.items"
                :key="item.id"
            >
                <div class="header">
                    <button class="toggle" @click="collapseToggle(item.id)">
                        <span v-if="isCollapsed(item.id)">
                            <micon icon="add" />
                            <tooltip>Expand</tooltip>
                        </span>
                        <span v-else>
                            <micon icon="remove" />
                            <tooltip>Collapse</tooltip>
                        </span>
                    </button>
                    <button class="copy" @click="copyItem(item)">
                        <micon icon="content_copy" />
                        <tooltip>Clone</tooltip>
                    </button>
                    <button class="status" @click="statusToggle(item)">
                        <template v-if="item.status">
                            <micon icon="visibility" />
                            <tooltip>Disable</tooltip>
                        </template>
                        <template v-else>
                            <micon icon="visibility_off" />
                            <tooltip>Enable</tooltip>
                        </template>
                    </button>
                    <button class="delete" @click="deleteItem(item.id)">
                        <micon icon="delete" />
                        <tooltip>Delete</tooltip>
                    </button>
                </div>

                <div class="drag" @mousedown="onDragStart($event, item.id)">
                    <micon icon="drag_indicator" />
                </div>

                <div class="title" v-if="isCollapsed(item.id)" @click="collapseToggle(item.id)">
                    {{ itemTitle(item) }}
                </div>
                <div class="body" v-else>
                    <apl-condition v-model="item.condition" :player="props.player" @update:modelValue="changed" />
                    <apl-action v-model="item.action" :player="props.player" @update:modelValue="changed" />
                </div>
            </div>
        </div>
        <div class="apl-buttons">
            <button class="btn btn-primary" @click="createItem">New action</button>
        </div>
        <!-- Default Action Section -->
        <div class="apl-default-action">
            <div class="default-action-header">
                <span class="header-title">Default Action</span>
                <span class="header-note">Executed when no priority conditions are met</span>
            </div>
            <div class="apl-item default-action" :class="[
                    isCollapsed(modelValue.defaultAction.id) ? 'collapsed' : 'expanded',
                    'status-'+modelValue.defaultAction.status,
                ]">
                <div class="header">
                    <button class="toggle" @click="collapseToggle(modelValue.defaultAction.id)">
                        <span v-if="isCollapsed(modelValue.defaultAction.id)">
                            <micon icon="add" /><tooltip>Expand</tooltip>
                        </span>
                        <span v-else>
                            <micon icon="remove" /><tooltip>Collapse</tooltip>
                        </span>
                    </button>
                </div>
                <div class="title" v-if="isCollapsed(modelValue.defaultAction.id)" @click="collapseToggle(modelValue.defaultAction.id)">
                    {{ defaultActionTitle() }}
                </div>
                <div class="body" v-else>
                    <apl-action v-model="modelValue.defaultAction.action" :player="props.player" :isDefault="true" @update:modelValue="changed" />
                </div>
            </div>
        </div>
        <div class="apl-buttons">
            <button class="btn btn-secondary right" @click="emits('save')">Save rotation</button>
        </div>
        <div class="dragger" v-if="dragging.id" :style="{transform: 'translate3d('+dragging.x+'px,'+dragging.y+'px,0)'}"></div>
    </div>
</template>