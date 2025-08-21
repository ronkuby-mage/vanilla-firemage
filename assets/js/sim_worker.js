// sim_worker.js
import init, { run_simulation, run_simulations } from "simulator";

onmessage = (event) => {

    if (event.data.type == "start") {
        init().then(r => {

            let result;
            if (event.data.iterations == 1) {
                result = run_simulation(event.data.config);
            }
            else {
                result = run_simulations(event.data.config, event.data.iterations);
            }

            postMessage({
                type: "success",
                raid_id: event.data.config.raid_id,
                is_active_raid: event.data.is_active_raid,
                result: result,
            });
        })
        .catch(e => {
            // sim_worker.js (just before postMessage)
            console.warn(e);
        });
    }
}