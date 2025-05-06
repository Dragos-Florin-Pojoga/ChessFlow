// TODO: fix init race condition (having to sleep after creating the worker)

export function createEngineWorker() {
    const worker = new Worker('/public/worker.js');

    const engine = new Proxy({}, {
        get(_, method) {
            return (...args) => {
                return new Promise((resolve, reject) => {
                    const callId = Math.random().toString(36).substr(2, 9);

                    const handleMessage = ({ data }) => {
                        if (data.callId === callId) {
                            worker.removeEventListener('message', handleMessage);
                            if (data.error) {
                                reject(new Error(data.error));
                            } else {
                                resolve(data.result);
                            }
                        }
                    };

                    worker.addEventListener('message', handleMessage);
                    worker.postMessage({ callId, method, args });
                });
            };
        }
    });

    return engine;
}
