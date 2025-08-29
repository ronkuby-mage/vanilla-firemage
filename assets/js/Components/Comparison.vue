<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import Chart from 'chart.js/auto';
import zoomPlugin from 'chartjs-plugin-zoom';

// Register the plugin
Chart.register(zoomPlugin);

const props = defineProps({
    result: Object,
    activeRaid: Object
});


const generateColor = (index) => {
    // Predefined colors for the first 6 raids
    const predefinedColors = [
        'rgb(255, 99, 132)',
        'rgb(54, 162, 235)', 
        'rgb(255, 205, 86)',
        'rgb(75, 192, 192)',
        'rgb(153, 102, 255)',
        'rgb(255, 159, 64)'
    ];
    
    if (index < predefinedColors.length) {
        return predefinedColors[index];
    }
    
    // Generate colors using HSL for better distribution
    const hue = (index * 137.508) % 360; // Golden angle approximation for good distribution
    const saturation = 65 + (index % 3) * 10; // Vary saturation slightly (65-85%)
    const lightness = 45 + (index % 4) * 5;   // Vary lightness slightly (45-60%)
    
    return `hsl(${hue}, ${saturation}%, ${lightness}%)`;
};

const chartCanvas = ref(null);
const showAll = ref(true);
const selectedRaid = ref(null);
const dpsMode = ref('total'); // Add this new ref
const runningAverage = ref(true); // Default enabled
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

const resetZoom = () => {
    if (chartInstance) {
        chartInstance.resetZoom();
    }
};

const getFinalDps = (raid) => {
    if (!raid.dps_over_time || raid.dps_over_time.length === 0) 
        return raid.avg_dps;
    return raid.dps_over_time[raid.dps_over_time.length - 1];
};

// Function to apply running average filter
const applyRunningAverage = (data, timeInterval, windowSeconds = 3) => {
    if (!runningAverage.value || data.length === 0) return data;
    
    const windowSize = Math.max(1, Math.round(windowSeconds / timeInterval));
    const smoothedData = [];
    
    for (let i = 0; i < data.length; i++) {
        const start = Math.max(0, i - Math.floor(windowSize / 2));
        const end = Math.min(data.length, i + Math.floor(windowSize / 2) + 1);
        
        let sum = 0;
        let count = 0;
        
        for (let j = start; j < end; j++) {
            sum += data[j];
            count++;
        }
        
        smoothedData[i] = sum / count;
    }
    
    return smoothedData;
};

const renderChart = () => {
    if (!chartCanvas.value || !visibleData.value.length) return;
    
    const ctx = chartCanvas.value.getContext('2d');
    const data = visibleData.value;
    
    // Generate time labels
    const maxPoints = Math.max(...data.map(r => r.dps_over_time?.length || 0));
    const duration = (props.activeRaid?.config?.duration - props.activeRaid?.config?.duration_variance) || 60;
    const timeInterval = duration / (maxPoints - 1);
    const chartColor = '#dfdfdf';

    // Create labels that match the data points
    const labels = Array.from({length: maxPoints}, (_, i) => (i * timeInterval).toFixed(1));
  
    const datasets = data.map((raid, index) => {
        let chartData = raid.dps_over_time || [];
        
        // If per-mage mode is selected, divide by number of players
        if (dpsMode.value === 'per-mage' && raid?.players) {
            const playerCount = raid.players.length;
            chartData = chartData.map(dpsValue => dpsValue / playerCount);
        }

        // Apply running average filter if enabled
        chartData = applyRunningAverage(chartData, timeInterval);        
        
        return {
            label: raid.raid_name,
            data: chartData,
            borderColor: generateColor(index),
            backgroundColor: generateColor(index) + '20',
            tension: 0.1,
            fill: false,
            pointRadius: 0,
            pointHoverRadius: 5,
            borderWidth: raid.raid_id === props.activeRaid?.id ? 3 : 2
        };
    });
    
    const chartConfig = {
        type: 'line',
        data: {
            labels: labels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            color: chartColor,
            plugins: {
                legend: {
                    position: 'top',
                    color: chartColor
                },
                title: {
                    display: true,
                    text: dpsMode.value === 'per-mage' ? 'DPS Per Mage Over Time Comparison' : 'DPS Over Time Comparison',
                    color: chartColor
                },
                tooltip: {
                    enabled: false
                },
                zoom: {
                    pan: {
                        enabled: false,
                    },
                    zoom: {
                        drag: {
                            enabled: true,
                            backgroundColor: 'rgba(54,162,235,0.3)'
                        },
                        mode: 'x'
                    }
                }                
            },
            scales: {
                x: {
                    display: true,
                    title: {
                        display: true,
                        text: 'Time (seconds)',
                        color: chartColor
                    },
                    ticks: {
                        callback: function(value, index, values) {
                            // Convert tick value to actual time
                            const timeValue = value * timeInterval;
                            
                            // Only show labels that align with our step size
                            const remainder = 10*(timeValue - Math.floor(timeValue));
                            const decimals = remainder >= 1 ? 1 : 0;
                            
                            return timeValue.toFixed(decimals);
                        },
                        color: chartColor,
                    },
                    grid: {
                       color: '#7f7f7f',
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: dpsMode.value === 'per-mage' ? 'DPS per Mage' : 'DPS',
                        color: chartColor,
                    },
                    beginAtZero: false,
                    ticks: {
                        color: chartColor
                    },
                    grid: {
                       color: '#7f7f7f',
                    }
                }
            },
            interaction: {
                mode: 'nearest',
                intersect: false
            }
        }
    };
    
    if (chartInstance) {
        chartInstance.data = chartConfig.data;
        chartInstance.options = chartConfig.options;        
        chartInstance.update();
    } else {
        chartInstance = new Chart(ctx, chartConfig);
    }
};

watch([showAll, selectedRaid, dpsMode, runningAverage, () => props.result], () => {
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
                <label class="checkbox-label">
                    <input type="checkbox" v-model="runningAverage">
                    <span>Running Average (3s)</span>
                </label>                
                <button @click="resetZoom" class="btn btn-secondary small">Reset Zoom</button>
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
</template>
