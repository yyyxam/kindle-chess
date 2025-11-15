#!/bin/sh

echo "=== X11 Chess Test Script ==="

# Check current processes
echo "Current X11 windows:"
xwininfo -tree -root | grep -E "^\s+0x" | wc -l

# Kill any previous test instance
if [ -f /tmp/chess_test.pid ]; then
    kill $(cat /tmp/chess_test.pid) 2>/dev/null
    rm /tmp/chess_test.pid
fi

# Set display
export DISPLAY=${DISPLAY:-:0.0}
echo "Using DISPLAY=$DISPLAY"

# Run the test
echo "Starting chess test..."
./bin/kindle-x11-test > /tmp/chess_test.log 2>&1 &
echo $! > /tmp/chess_test.pid

echo "Started with PID: $(cat /tmp/chess_test.pid)"
echo "Log: /tmp/chess_test.log"
echo ""
echo "Commands:"
echo "  tail -f /tmp/chess_test.log  # Watch log"
echo "  kill \$(cat /tmp/chess_test.pid)  # Stop"
