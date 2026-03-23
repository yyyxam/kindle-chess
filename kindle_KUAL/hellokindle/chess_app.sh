#!/bin/sh

# ─── Chess App Launcher ───────────────────────────────────────────────────────
# Deploys from /mnt/us/hellokindle/
# Binary:  bin/kindle-hello
# Logs:    log/app.log (written by the binary itself via log4rs)
# Secrets: secrets/token.json

APP_DIR="/mnt/us/hellokindle"
BINARY="$APP_DIR/bin/kindle-hello"
SCRIPT_LOG="$APP_DIR/log/launch.log"
PID_FILE="/tmp/chess_app.pid"

# Ensure log dir exists
mkdir -p "$APP_DIR/log"
mkdir -p "$APP_DIR/secrets"

echo "=== chess_app.sh started ===" > "$SCRIPT_LOG"
echo "Time: $(date)"               >> "$SCRIPT_LOG"

# ─── Sanity checks ────────────────────────────────────────────────────────────

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found at $BINARY" >> "$SCRIPT_LOG"
    eips -c
    eips "ERROR: kindle-hello binary not found"
    sleep 5
    exit 1
fi

if [ ! -x "$BINARY" ]; then
    echo "WARNING: Binary not executable — fixing..." >> "$SCRIPT_LOG"
    chmod +x "$BINARY"
fi

# ─── Kill any stale instance ──────────────────────────────────────────────────

if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo "Killing stale instance (PID $OLD_PID)" >> "$SCRIPT_LOG"
        kill "$OLD_PID"
        sleep 1
    fi
    rm -f "$PID_FILE"
fi

# ─── X11 / display setup ──────────────────────────────────────────────────────

# The Kindle's Xorg instance is always on :0
export DISPLAY=:0

# Hand the screen over from the Kindle UI to X11.
# eips -c clears the eInk framebuffer so X11 draws on a clean slate.
eips -c

# Tell the Kindle framework to stop managing the display so X11 can own it.
# This is the same lipc call used by KOReader and similar X11 apps on Kindle.
lipc-set-prop com.lab126.pillow disableEnablePillow disable 2>/dev/null || true

echo "Display setup done, launching binary..." >> "$SCRIPT_LOG"

# ─── Launch ───────────────────────────────────────────────────────────────────

cd "$APP_DIR"
"$BINARY" >> "$SCRIPT_LOG" 2>&1 &
echo $! > "$PID_FILE"
echo "Launched with PID $(cat $PID_FILE)" >> "$SCRIPT_LOG"

# Wait for the binary to finish
wait $(cat "$PID_FILE")
EXIT_CODE=$?
rm -f "$PID_FILE"

echo "Binary exited with code $EXIT_CODE" >> "$SCRIPT_LOG"

# ─── Cleanup ──────────────────────────────────────────────────────────────────

# Hand display back to the Kindle UI
lipc-set-prop com.lab126.pillow disableEnablePillow enable 2>/dev/null || true
eips -c
eips -f

echo "=== chess_app.sh done ===" >> "$SCRIPT_LOG"
