<script setup>
import { ref, onMounted, onUpdated, onBeforeUnmount, nextTick } from "vue";

const props = defineProps({
    position: {
        type: String,
        default: "top"
    },
});

const el = ref(null);
const pos = ref(props.position);

const checkCollision = () => {
    if (!el.value?.parentNode) return;
    
    const rect = el.value.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    
    // Reset to original position first
    pos.value = props.position;
    
    // Handle horizontal collision
    if (pos.value.includes("right") && rect.right > viewportWidth) {
        pos.value = pos.value.replace("right", "left");
    } else if (pos.value.includes("left") && rect.left < 0) {
        pos.value = pos.value.replace("left", "right");
    }
    
    // Handle vertical collision  
    if (pos.value.includes("top") && rect.top < 0) {
        pos.value = pos.value.replace("top", "bottom");
    } else if (pos.value.includes("bottom") && rect.bottom > viewportHeight) {
        pos.value = pos.value.replace("bottom", "top");
    }
};

const setupTooltip = () => {
    if (!el.value?.parentNode) return;
    
    if (!el.value.parentNode.classList.contains("tooltip-anchor")) {
        el.value.parentNode.classList.add("tooltip-anchor");
        el.value.parentNode.addEventListener("mouseenter", checkCollision);
    }
};

onMounted(setupTooltip);
onUpdated(setupTooltip);

onBeforeUnmount(() => {
    if (el.value?.parentNode) {
        el.value.parentNode.classList.remove("tooltip-anchor");
        el.value.parentNode.removeEventListener("mouseenter", checkCollision);
    }
});
</script>

<template>
    <div ref="el" class="tooltip" :class="['position-'+pos]">
        <slot></slot>
    </div>
</template>