/**
 * @file engine.js
 * @brief Provides a JavaScript interface to a WebAssembly worker,
 * handling communication and managing callbacks.
 *
 * This module sets up a Web Worker to offload WebAssembly computations
 * from the main thread. It uses a Proxy to dynamically create methods
 * that correspond to the Rust functions exported by the WASM module.
 *
 * Key features:
 * - Worker initialization and readiness check.
 * - Asynchronous communication with the worker using `postMessage` and `onmessage`.
 * - A proxy for the WASM engine, allowing direct calls to Rust functions
 * as if they were local JavaScript functions.
 * - Special handling for `register_callback`: the JavaScript callback
 * is stored in the main thread, and the Rust WASM module notifies
 * the main thread when the callback should be invoked. This avoids
 * passing `Function` objects to the worker, which is not allowed.
 */

let nextCallId = 0; // Counter for unique call IDs for messages to the worker

/**
 * Generates a unique call ID for messages sent to the worker.
 * @returns {string} A unique string identifier.
 */
function generateCallId() {
    return (nextCallId++).toString();
}

/**
 * Creates and initializes a Web Worker that loads the WebAssembly engine.
 * This function returns a Promise that resolves with a proxy object
 * representing the WASM engine, allowing direct method calls.
 *
 * @returns {Promise<object>} A promise that resolves with the WASM engine proxy.
 */
export async function createEngineWorker() {
    // Create a new Web Worker instance. The path should be relative to the
    // script that creates the worker, or an absolute path.
    const worker = new Worker('/public/worker.js');

    // This variable will store the JavaScript callback function provided
    // by the main thread (e.g., from main.js). This function is NOT sent
    // to the worker. Instead, the worker will send a message back to the
    // main thread when this callback needs to be triggered.
    let registeredMainThreadCallback = null;

    /**
     * Waits for the worker to signal that it's ready to receive messages.
     * This is important to ensure the WASM module is fully loaded and
     * initialized in the worker before sending any commands.
     */
    await new Promise(resolve => {
        const initListener = ({ data }) => {
            if (data.type === 'worker_ready') {
                worker.removeEventListener('message', initListener); // Remove listener once ready
                console.log("WebAssembly worker is ready.");
                resolve();
            }
        };
        worker.addEventListener('message', initListener);
    });

    /**
     * Sets up a listener for messages coming *from* the worker.
     * This listener handles two main types of messages:
     * 1. Responses to method calls initiated by this `engine.js` (identified by `callId`).
     * 2. Requests from the worker to trigger the main thread's registered callback
     * (identified by `type: 'callback_trigger'`).
     */
    worker.addEventListener('message', ({ data }) => {
        // Handle callback trigger requests from the worker
        if (data.type === 'callback_trigger') {
            if (registeredMainThreadCallback) {
                try {
                    // Invoke the locally stored callback with the message from Rust
                    console.log("Main thread received callback trigger from worker.");
                    registeredMainThreadCallback(data.message);
                } catch (e) {
                    console.error("Error executing main thread callback:", e);
                }
            } else {
                console.warn("Worker requested callback, but no callback is registered in main thread.");
            }
        }
        // Handle responses to method calls (already handled by individual Promise resolvers)
        // No explicit action needed here, as the `handleMessage` in the Proxy's `get`
        // method will catch these based on `callId`.
    });

    /**
     * Creates a Proxy object that intercepts property access (method calls)
     * on the `engine` object. This allows us to dynamically send messages
     * to the Web Worker for each method call.
     */
    const engine = new Proxy({}, {
        /**
         * Intercepts property (method) access on the `engine` object.
         * @param {object} _target - The target object (empty in this case).
         * @param {string|symbol} prop - The name of the property being accessed (method name).
         * @returns {function|undefined} A function to be called, or undefined if not a method.
         */
        get(_target, prop) {
            // Ignore special properties that are not intended to be WASM methods
            if (prop === 'then' || prop === 'catch' || prop === 'finally' || typeof prop === 'symbol') {
                return undefined;
            }

            /**
             * Special handling for the `register_callback` method.
             * This method does NOT send the function to the worker.
             * Instead, it stores the function locally in the main thread.
             * The worker will later send a message back to the main thread
             * to indicate when this stored callback should be executed.
             * @param {function} cb - The JavaScript callback function.
             * @returns {Promise<void>} A promise that resolves when the callback is registered.
             */
            if (prop === 'register_callback') {
                return (cb) => {
                    if (typeof cb === 'function') {
                        registeredMainThreadCallback = cb; // Store the function locally
                        console.log("Main thread: Callback function registered successfully.");
                        return Promise.resolve(); // Return a resolved promise for consistency
                    } else {
                        console.error("Main thread: register_callback expects a function.");
                        return Promise.reject(new Error("register_callback expects a function."));
                    }
                };
            }

            /**
             * For all other methods, create a function that sends a message
             * to the Web Worker and returns a Promise that resolves with the
             * worker's response for that specific call.
             * @param {...any} args - Arguments passed to the method.
             * @returns {Promise<any>} A promise that resolves with the result from the WASM method.
             */
            return (...args) => new Promise((resolve, reject) => {
                const callId = generateCallId(); // Get a unique ID for this call

                /**
                 * Listener for the specific response to this method call.
                 * This listener is added and removed for each individual call.
                 * @param {MessageEvent} event - The message event from the worker.
                 */
                const handleMessage = ({ data }) => {
                    if (data.callId === callId) { // Check if the message is for this specific call
                        worker.removeEventListener('message', handleMessage); // Clean up listener
                        if (data.error) {
                            reject(new Error(data.error)); // Reject if the worker returned an error
                        } else {
                            resolve(data.result); // Resolve with the result from the worker
                        }
                    }
                };

                // Add the temporary listener for this specific call's response
                worker.addEventListener('message', handleMessage);

                try {
                    // Post the message to the worker, including the method name and arguments.
                    // Functions cannot be cloned, so `args` must only contain serializable data.
                    worker.postMessage({ callId, method: prop, args });
                    console.log(`Main thread: Sent method call "${String(prop)}" with ID ${callId} to worker.`);
                } catch (e) {
                    // If postMessage itself fails (e.g., due to non-cloneable data),
                    // remove the listener and reject the promise immediately.
                    worker.removeEventListener('message', handleMessage);
                    console.error(`Main thread: Failed to post message for method "${String(prop)}" to worker:`, e);
                    reject(new Error(`Failed to send message to worker for method "${String(prop)}": ${e.message}`));
                }
            });
        }
    });

    return engine; // Return the proxy object
}
