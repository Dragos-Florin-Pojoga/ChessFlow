#!/bin/bash

nix-shell './tracy.nix' &
TRACY_PID=$!

if [ -z "$TRACY_PID" ]; then
    echo "Error: Failed to start Tracy"
    exit 1
fi

echo "Tracy started with PID: $TRACY_PID"

cargo run --bin native_engine --release --features tracy

APP_EXIT_STATUS=$?

echo "Application finished with exit status: $APP_EXIT_STATUS"

echo "Waiting for Tracy (PID $TRACY_PID) to close..."
wait $TRACY_PID

exit $APP_EXIT_STATUS
