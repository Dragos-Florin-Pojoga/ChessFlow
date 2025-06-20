console.log("importing WASM module in worker");
importScripts("/public/pkg/wasm_engine.js");

(async function () {
    console.log("initializing WASM module in worker");
    await wasm_bindgen({ module_or_path: "/public/pkg/wasm_engine_bg.wasm" });

    self.onmessage = async function ({ data }) {
        // Handle messages that are requests to call WASM methods
        if (data.callId) { // Check if it's a method call from engine.js
            const { callId, method, args } = data;
            if (typeof wasm_bindgen[method] === 'function') {
                try {
                    const result = await wasm_bindgen[method](...args);
                    self.postMessage({ callId, result });
                } catch (err) {
                    self.postMessage({ callId, error: err.toString() });
                }
            } else {
                self.postMessage({ callId, error: `Unknown method: ${method}` });
            }
        }
        // No other message types handled here directly now
    };

    self.postMessage({ type: 'worker_ready' });
})();