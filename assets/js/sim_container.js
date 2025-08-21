import Worker from "./sim_worker.js?worker";

class SimContainer {
    constructor(threads, iterations, configs, onSuccess, onError, onProgress) {
        // Handle both single config (backward compatibility) and multiple configs
        this.configs = Array.isArray(configs) ? _.cloneDeep(configs) : [_.cloneDeep(configs)];
        this.threads = threads;
        this.iterations = parseInt(iterations);
        this.workers = [];
        this.runs = [];
        this.start_time = null;
        this.results_by_raid = new Map(); // Store results for each raid


        console.log('[SimContainer] Constructor:', {
            iterations: this.iterations,
            configCount: this.configs.length,
            threads: this.threads
        });

        if (!this.threads || isNaN(this.threads))
            throw "Invalid threads";
        if (!this.iterations || isNaN(this.iterations))
            throw "Invalid iterations";

        let run_itr = Math.max(50, Math.min(200, Math.ceil(this.iterations/this.threads)));
        let num_workers = Math.min(this.threads, Math.ceil(this.iterations/run_itr));

        console.log('[SimContainer] Run info:', {
            1: run_itr,
            2: num_workers
        });


        // Create runs for each config
        for (let config of this.configs) {
            for (let total = 0; total < this.iterations; total += run_itr) {
                let itr = Math.min(run_itr, this.iterations - total);
                this.runs.push({
                    iterations: itr,
                    config: config,
                    raid_id: config.raid_id,
                    raid_name: config.raid_name,
                    is_active_raid: config.is_active_raid,
                    started: false,
                });
            }
        }

        console.log('[SimContainer] Created runs:', this.runs.length);        

        for (let i = 0; i < num_workers; i++) {
            this.workers.push(new Worker());

            this.workers[i].onmessage = (event) => {
                let data = event.data;

                if (data.type == "error") {
                    this.workers[i].terminate();
                    onError(data);
                }

                // Thread done
                if (data.type == "success") {
                    const raid_id = data.raid_id;
                    
                    // Initialize or merge results for this raid
                    if (!this.results_by_raid.has(raid_id)) {
                        console.log('[SimContainer] New raid result:', raid_id, data.is_active_raid);                        
                        this.results_by_raid.set(raid_id, {
                            ...data.result,
                            raid_id: raid_id,
                            raid_name: data.raid_name,
                            is_active_raid: data.is_active_raid,
                            dps_timeline: data.result.dps_timeline || []
                        });
                    } else {
                        console.log('[SimContainer] Merging raid result:', raid_id, data.is_active_raid);                        
                        this.mergeResults(raid_id, data.result);
                    }

                    // Check if all runs are complete
                    const completedIterations = this.getTotalCompletedIterations();
                    const totalExpectedIterations = this.iterations * this.configs.length;

                    console.log('[SimContainer] Progress:', {
                        completed: completedIterations,
                        expected: totalExpectedIterations,
                        raidsCompleted: this.results_by_raid.size,
                        configsCount: this.configs.length
                    });                    

                    if (onProgress) {
                        const progress = {
                            iterations: Math.floor(completedIterations / this.configs.length),
                            dps: this.calculateAverageDps(),
                        };
                        onProgress(progress);
                    }

                    // Special handling for single iteration
                    if (this.iterations === 1) {
                        // For single iteration, check if this specific raid is complete
                        const raidResult = this.results_by_raid.get(raid_id);
                        if (raidResult && raidResult.iterations === 1) {
                            // Check if all raids are complete
                            if (this.results_by_raid.size === this.configs.length) {
                                this.workers[i].terminate();
                                const finalResult = this.compileFinalResults();
                                finalResult.time = (Date.now() - this.start_time) / 1000;
                                onSuccess(finalResult);
                            } else if (!this.startNextRun(i)) {
                                this.workers[i].terminate();
                            }
                        }
                    } else {
                        // Multiple iterations handling
                        if (completedIterations >= totalExpectedIterations) {
                            this.workers[i].terminate();
                            const finalResult = this.compileFinalResults();
                            finalResult.time = (Date.now() - this.start_time) / 1000;
                            onSuccess(finalResult);
                        } else {
                            if (!this.startNextRun(i))
                                this.workers[i].terminate();
                        }
                    }
                }
            };

            this.workers[i].onerror = (...args) => {
                onError(...args);
                this.workers[i].terminate();
            };
        }

    }

    mergeResults(raid_id, newResult) {
        let sum = this.results_by_raid.get(raid_id);
        
        if (newResult.min_dps < sum.min_dps)
            sum.min_dps = newResult.min_dps;
        if (newResult.max_dps > sum.max_dps)
            sum.max_dps = newResult.max_dps;
            
        sum.dps = (sum.dps * sum.iterations + newResult.dps * newResult.iterations) / 
                  (sum.iterations + newResult.iterations);
        sum.ignite_dps = (sum.ignite_dps * sum.iterations + newResult.ignite_dps * newResult.iterations) / 
                         (sum.iterations + newResult.iterations);

        // Merge histograms
        if (newResult.histogram) {
            if (!sum.histogram) sum.histogram = new Map();
            for (const [key, val] of newResult.histogram) {
                let acc = sum.histogram.get(key);
                sum.histogram.set(key, val + (acc ? acc : 0));
            }
        }

        // Merge player data
        for (let j = 0; j < sum.players.length; j++) {
            sum.players[j].dps = (sum.players[j].dps * sum.iterations + 
                                 newResult.players[j].dps * newResult.iterations) / 
                                 (sum.iterations + newResult.iterations);
            sum.players[j].ignite_dps = (sum.players[j].ignite_dps * sum.iterations + 
                                         newResult.players[j].ignite_dps * newResult.iterations) / 
                                         (sum.iterations + newResult.iterations);
        }

        // Merge stat weights (only for active raid)
        if (sum.is_active_raid && newResult.dps_sp) {
            sum.dps_select = (sum.dps_select || 0) + newResult.dps_select;
            sum.dps_sp = (sum.dps_sp || 0) + newResult.dps_sp;
            sum.dps_crit = (sum.dps_crit || 0) + newResult.dps_crit;
            sum.dps_hit = (sum.dps_hit || 0) + newResult.dps_hit;
            sum.dps90_select = (sum.dps90_select || 0) + newResult.dps90_select;
            sum.dps90_sp = (sum.dps90_sp || 0) + newResult.dps90_sp;
            sum.dps90_crit = (sum.dps90_crit || 0) + newResult.dps90_crit;
            sum.dps90_hit = (sum.dps90_hit || 0) + newResult.dps90_hit;
        }

        // Merge DPS timeline
        if (newResult.dps_timeline) {
            if (!sum.dps_timeline) sum.dps_timeline = [];
            // Average the timelines
            for (let i = 0; i < newResult.dps_timeline.length; i++) {
                if (i < sum.dps_timeline.length) {
                    sum.dps_timeline[i] = (sum.dps_timeline[i] * sum.iterations + 
                                          newResult.dps_timeline[i] * newResult.iterations) / 
                                          (sum.iterations + newResult.iterations);
                } else {
                    sum.dps_timeline[i] = newResult.dps_timeline[i];
                }
            }
        }

        sum.iterations += newResult.iterations;
    }

    getTotalCompletedIterations() {
        let total = 0;
        for (let result of this.results_by_raid.values()) {
            total += result.iterations;
        }
        return total;
    }

    calculateAverageDps() {
        let totalDps = 0;
        let count = 0;
        for (let result of this.results_by_raid.values()) {
            totalDps += result.dps;
            count++;
        }
        return count > 0 ? totalDps / count : 0;
    }

    compileFinalResults() {
        // Find the active raid result
        let activeResult = null;
        const comparison_data = [];
        
        for (let [raid_id, result] of this.results_by_raid) {
            if (result.is_active_raid) {
                activeResult = result;
            }
            
            // Calculate stats for comparison
            const dps_over_time = result.dps_timeline || [];
            const avg_dps = result.dps;
            const peak_dps = dps_over_time.length > 0 ? Math.max(...dps_over_time) : result.dps;
            const time_to_peak = dps_over_time.length > 0 ? 
                dps_over_time.indexOf(peak_dps) : 0;
            
            comparison_data.push({
                raid_id: raid_id,
                raid_name: result.raid_name,
                dps_over_time: dps_over_time,
                avg_dps: avg_dps,
                peak_dps: peak_dps,
                time_to_peak: time_to_peak,
                players: result.players
            });
        }

        // If no active result (shouldn't happen), use first result
        if (!activeResult && this.results_by_raid.size > 0) {
            activeResult = this.results_by_raid.values().next().value;
        }

        // Return result in expected format with comparison data
        return {
            ...activeResult,
            comparison_data: comparison_data,
            iterations: this.iterations
        };
    }

    start() {
        this.start_time = Date.now();
        for (let i = 0; i < this.workers.length; i++) {
            this.startRun(i, i);
        }
    }

    startRun(worker_index, run_index) {
        let run = this.runs[run_index];
        let config = _.cloneDeep(run.config);
        config.rng_seed = config.rng_seed + run_index * run.iterations;

        this.workers[worker_index].postMessage({
            type: "start",
            config: config,
            iterations: run.iterations,
            raid_id: run.raid_id,
            raid_name: run.raid_name,
            is_active_raid: run.is_active_raid,
            track_timeline: true, // Tell worker to track DPS over time
            do_stat_weights: run.is_active_raid // Only do stat weights for active raid
        });
        run.started = true;
    }

    startNextRun(worker_index) {
        for (let i = 0; i < this.runs.length; i++) {
            if (!this.runs[i].started) {
                this.startRun(worker_index, i);
                return true;
            }
        }
        return false;
    }
}

export default SimContainer;