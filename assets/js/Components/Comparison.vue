<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import Chart from 'chart.js/auto';

const props = defineProps({
    result: Object,
    activeRaid: Object
});

const chartCanvas = ref(null);
const showAll = ref(true);
const selectedRaid = ref(null);
const dpsMode = ref('total'); // Add this new ref
let chartInstance = null;
const dpsModeOptions = computed(() => [
    { value: 'total', title: 'Total DPS' },
    { value: 'per-mage', title: 'Per Mage DPS' }
]);

const hasComparisonData = computed(() => {
    return props.result?.comparison_data?.length > 0;
});

const raidOptions = computed(() => {
    if (!props.result?.comparison_data) return [];
    return props.result.comparison_data.map(raid => ({
        value: raid.raid_id,
        title: raid.raid_name
    }));
});

const visibleData = computed(() => {
    if (!props.result?.comparison_data) return [];
    if (showAll.value) return props.result.comparison_data;
    return props.result.comparison_data.filter(r => r.raid_id === selectedRaid.value);
});

const getFinalDps = (raid) => {
    if (!raid.dps_over_time || raid.dps_over_time.length === 0) 
        return raid.avg_dps;
    return raid.dps_over_time[raid.dps_over_time.length - 1];
};

const renderChart = () => {
    if (!chartCanvas.value || !visibleData.value.length) return;
    
    const ctx = chartCanvas.value.getContext('2d');
    const data = visibleData.value;
    
    // Generate time labels
    const maxPoints = Math.max(...data.map(r => r.dps_over_time?.length || 0));
    const duration = (props.activeRaid?.config?.duration - props.activeRaid?.config?.duration_variance) || 60;
    const timeInterval = duration / maxPoints;
    const labels = Array.from({length: maxPoints}, (_, i) => (i * timeInterval).toFixed(0));
    
    const colors = [
        'rgb(255, 99, 132)',
        'rgb(54, 162, 235)',
        'rgb(255, 205, 86)',
        'rgb(75, 192, 192)',
        'rgb(153, 102, 255)',
        'rgb(255, 159, 64)'
    ];
    
    console.log("I have a data", data)

    const datasets = data.map((raid, index) => ({
        label: raid.raid_name,
        data: raid.dps_over_time || [],
        borderColor: colors[index % colors.length],
        backgroundColor: colors[index % colors.length] + '20',
        tension: 0.1,
        fill: false,
        pointRadius: 0,
        pointHoverRadius: 5,
        borderWidth: raid.raid_id === props.activeRaid?.id ? 3 : 2
    }));
    
    const chartConfig = {
        type: 'line',
        data: {
            labels: labels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'top',
                },
                title: {
                    display: true,
                    text: 'DPS Over Time Comparison'
                },
                tooltip: {
                    mode: 'index',
                    intersect: false,
                }
            },
            scales: {
                x: {
                    display: true,
                    title: {
                        display: true,
                        text: 'Time (seconds)'
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: 'DPS'
                    },
                    beginAtZero: false
                }
            },
            interaction: {
                mode: 'nearest',
                axis: 'x',
                intersect: false
            }
        }
    };
    
    if (chartInstance) {
        chartInstance.data = chartConfig.data;
        chartInstance.update();
    } else {
        chartInstance = new Chart(ctx, chartConfig);
    }
};

// Watchers
watch([showAll, selectedRaid, () => props.result], () => {
    if (hasComparisonData.value) {
        nextTick(() => renderChart());
    }
});

watch(() => props.result, (newVal) => {
    if (newVal?.comparison_data?.length > 0 && !selectedRaid.value) {
        selectedRaid.value = newVal.comparison_data[0].raid_id;
    }
});

onMounted(() => {
    if (hasComparisonData.value) {
        renderChart();
    }
});

onUnmounted(() => {
    if (chartInstance) {
        chartInstance.destroy();
        chartInstance = null;
    }
});
</script>

<template>
    <div class="comparison">
        <div class="comparison-content" v-if="hasComparisonData">
            <div class="comparison-controls">
                <div class="control-group">
                    <!--
                    <label class="checkbox-label">
                        <input type="checkbox" v-model="showAll">
                        <span>Show all raids</span>
                    </label>
                    <select-simple 
                        v-if="!showAll"
                        v-model="selectedRaid" 
                        :options="raidOptions"
                        placeholder="Select raid..."
                    />
                    -->

                    <select-simple 
                        v-model="dpsMode"
                        :options="dpsModeOptions"
                        placeholder="DPS Mode..."
                    />
                    <select-simple 
                        v-if="!showAll"
                        v-model="selectedRaid" 
                        :options="raidOptions"
                        placeholder="Select raid..."
                    />

                </div>
            </div>
            
            <div class="chart-wrapper">
                <canvas ref="chartCanvas" class="dps-chart"></canvas>
            </div>
            <!--
            <div class="comparison-table">
                <table>
                    <thead>
                        <tr>
                            <th>Raid</th>
                            <th>Average DPS</th>
                            <th>Peak DPS</th>
                            <th>Time to Peak</th>
                            <th>Final DPS</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="raid in visibleData" 
                            :key="raid.raid_id"
                            :class="{ 'active-raid': raid.raid_id === activeRaid?.id }">
                            <td class="raid-name">{{ raid.raid_name }}</td>
                            <td>{{ raid.avg_dps.toFixed(1) }}</td>
                            <td>{{ raid.peak_dps.toFixed(1) }}</td>
                            <td>{{ raid.time_to_peak.toFixed(1) }}s</td>
                            <td>{{ getFinalDps(raid).toFixed(1) }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
            -->
        </div>
        <div class="no-data" v-else>
            <p>No comparison data available.</p>
            <p>Enable "Include in Comparison" for raids you want to compare.</p>
        </div>
    </div>
</template>
