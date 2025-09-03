<script setup>
import { ref, computed, defineEmits } from 'vue';

const emit = defineEmits(['close']);

const currentPageIndex = ref(0);

// Placeholder tutorial content - replace with your actual content
const tutorialPages = ref([
    {
        title: "Single Target Fire Mage Simulator",
        description: "This tutorial will guide you through the key features of the simulator.  Learn how to set up raids, configure players, and analyze results.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_0.png",
        points: [
            "Create and manage raid configurations",
            "Automatically populate multiple viable rotations from template",
            "Tweak player gear", 
            "Generate stat weights to evaluate upgrades"
        ]
    },
    {
        title: "Use Approximate Gear to Build a Template",
        description: "Start by reviewing the preset gear levels.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_2.png",
        points: [
            "If you find a reasonable prest match to the gear on your mage team, proceed to the next step",
            "Otherwise, manually build out the gear sets for your team",
            "If you are not using a preset, you must also speicify the player config settings for each mage"
        ]
    }, 
    {
        title: "Create Raid from Template",
        description: "Proceed with multiple raid creation.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_3.png",
        points: [
            "If a preset gear level fit your team, use that (shown)",
            "If you built your team manually, select a 'from existing' template", 
            "Select the encounter duration carefully, it will determine the suite of rotations"
        ]
    },
    {
        title: "Running the Comparison",
        description: "Run the simulation, view results, and switch to the Comparison tab.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_4.png",
        points: [
            "There could be many (20+) results",
            "Each curve represents the performance of a rotation given the gear and other specified parameters",
            "Select 2-3 of the top rotations",
            "It may be necessary to zoom in and hover over the highest DPS curves to resolve the name"
        ]
    },
    {
        title: "Prune Losing Rotations",
        description: "Change the raid name for the top rotation(s).",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_5.png",
        points: [
            "Changing the name will flag the simulator to keep the raids",
            "The other raids can be quickly pruned by deleting one and selecting group removal"
        ]
    },
    {
        title: "Review Rotation Logic",
        description: "Each mage will likely be assigned a different rotation.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_7.png",
        points: [
            "The opening sequence will always run to completion unless an action is not available (on cooldown)",
            "Priority rotations are conditional, some mages may not have any priority rotations",
            "The default action is almost always Fireball"
        ]
    },
    {
        title: "Tweak Gear Sets and Buffs",
        description: "Modify the mage team gear and buffs.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_8.png",
        points: [
            "If a preset was used, modify the gear sets for major upgrades",
            "Buffs should also be fine-tuned to what is expected during raid"
        ]
    },
    {
        title: "Generate Stat Weights",
        description: "Rerun the fine-tuned raid to generate stat weights.",
        image: "https://ronkuby-mage.github.io/vanilla-firemage/img/tutorial/page_9.png",
        points: [
            "The number of iterations should be increased to 300k-1M",
            "Spell power per hit can be ignored unless hit is below 98%",
            "Individual mage stat weights can be controlled with 'Stat Weight Target' and 'Stat Weight Differential' selections under Player Config"
        ]
    }
]);

const currentPage = computed(() => tutorialPages.value[currentPageIndex.value]);

const nextPage = () => {
    if (currentPageIndex.value < tutorialPages.value.length - 1) {
        currentPageIndex.value++;
    }
};

const previousPage = () => {
    if (currentPageIndex.value > 0) {
        currentPageIndex.value--;
    }
};

const exitTutorial = () => {
    emit('close');
};
</script>

<template>
    <div class="tutorial-content">
        <div class="tutorial-main">
            <div class="tutorial-image">
                <img 
                    :src="currentPage.image" 
                    :alt="currentPage.title"
                    class="tutorial-img"
                />
            </div>
            <div class="tutorial-text">
                <transition name="wipe-in" mode="out-in">
                    <div :key="currentPageIndex" class="text-content">
                        <h2 class="tutorial-title">{{ currentPage.title }}</h2>
                        <p class="tutorial-description">{{ currentPage.description }}</p>
                        <ul v-if="currentPage.points" class="tutorial-points">
                            <li v-for="point in currentPage.points" :key="point">{{ point }}</li>
                        </ul>
                    </div>
                </transition>
            </div>
        </div>
        
        <div class="tutorial-controls">
            <button 
                class="btn btn-secondary large tutorial-btn" 
                @click="previousPage" 
                :disabled="currentPageIndex === 0"
            >
                Previous
            </button>
            
            <div class="tutorial-progress">
                <span class="page-indicator">
                    {{ currentPageIndex + 1 }} / {{ tutorialPages.length }}
                </span>
                <div class="progress-dots">
                    <div 
                        v-for="(page, index) in tutorialPages" 
                        :key="index"
                        class="progress-dot"
                        :class="{ active: index === currentPageIndex }"
                    ></div>
                </div>
            </div>
            
            <button 
                class="btn btn-secondary large tutorial-btn" 
                @click="nextPage" 
                :disabled="currentPageIndex === tutorialPages.length - 1"
            >
                Next
            </button>
            
            <button 
                class="btn btn-primary large tutorial-btn tutorial-exit" 
                @click="exitTutorial"
            >
                Exit Tutorial
            </button>
        </div>
    </div>
</template>

<style scoped>
.tutorial-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    box-sizing: border-box;
}

.tutorial-main {
    flex: 1;
    display: flex;
    gap: 3rem;
    align-items: center;
    margin-bottom: 2rem;
}

.tutorial-image {
    flex: 0 0 45%;
}

.tutorial-img {
    width: 100%;
    height: auto;
    border-radius: 20px;
    border: 3px solid #333;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.tutorial-text {
    flex: 1;
    padding-left: 2rem;
}

.text-content {
    min-height: 300px;
}

.tutorial-title {
    font-size: 2.5rem;
    font-weight: bold;
    color: #fff;
    margin-bottom: 1.5rem;
    line-height: 1.2;
}

.tutorial-description {
    font-size: 1.25rem;
    color: #ccc;
    line-height: 1.6;
    margin-bottom: 2rem;
}

.tutorial-points {
    list-style: none;
    padding: 0;
    margin: 0;
}

.tutorial-points li {
    font-size: 1.1rem;
    color: #aaa;
    padding: 0.75rem 0;
    padding-left: 1.5rem;
    position: relative;
    line-height: 1.4;
}

.tutorial-points li::before {
    content: "→";
    position: absolute;
    left: 0;
    color: #4CAF50;
    font-weight: bold;
    font-size: 1.2rem;
}

.tutorial-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 2rem;
    padding-top: 2rem;
    border-top: 1px solid #444;
}

.tutorial-btn {
    min-width: 120px;
    padding: 1rem 2rem;
    font-size: 1.1rem;
    font-weight: 600;
}

.tutorial-exit {
    margin-left: auto;
}

.tutorial-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
}

.page-indicator {
    font-size: 1rem;
    color: #ccc;
    font-weight: 500;
}

.progress-dots {
    display: flex;
    gap: 0.5rem;
}

.progress-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #444;
    transition: background 0.3s ease;
}

.progress-dot.active {
    background: #4CAF50;
}

/* Wipe-in transition */
.wipe-in-enter-active,
.wipe-in-leave-active {
    transition: all 0.5s ease;
}

.wipe-in-enter-from {
    opacity: 0;
    transform: translateX(30px);
}

.wipe-in-leave-to {
    opacity: 0;
    transform: translateX(-30px);
}

.wipe-in-enter-to,
.wipe-in-leave-from {
    opacity: 1;
    transform: translateX(0);
}

/* Responsive design */
@media (max-width: 1200px) {
    .tutorial-main {
        flex-direction: column;
        text-align: center;
    }
    
    .tutorial-image {
        flex: none;
        max-width: 500px;
    }
    
    .tutorial-text {
        padding-left: 0;
        padding-top: 2rem;
    }
    
    .tutorial-title {
        font-size: 2rem;
    }
    
    .tutorial-description {
        font-size: 1.1rem;
    }
}

@media (max-width: 768px) {
    .tutorial-content {
        padding: 1rem;
    }
    
    .tutorial-controls {
        flex-wrap: wrap;
        gap: 1rem;
    }
    
    .tutorial-btn {
        min-width: 100px;
        padding: 0.75rem 1.5rem;
        font-size: 1rem;
    }
    
    .tutorial-title {
        font-size: 1.75rem;
    }
    
    .tutorial-description {
        font-size: 1rem;
    }
}
</style>