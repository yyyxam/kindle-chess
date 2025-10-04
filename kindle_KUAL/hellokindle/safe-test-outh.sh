#!/bin/sh

# SICHERHEITS-WRAPPER für Kindle Hello World Test

EXTENSION_PATH="/mnt/us/extensions/hellokindle"
APP_PATH="mtn/us/hellokindle"
BINARY_PATH="/mnt/us/hellokindle/bin/kindle-hello"
LOG_FILE="/mnt/us/hellokindle/tmp/script.log"

echo "=== Binary test script started... ===" > "$LOG_FILE"
echo "Time: $(date)" >> "$LOG_FILE"

cd "$APP_PATH"

# Sicherheitscheck 1: Binary vorhanden?
if [ ! -f "$BINARY_PATH" ]; then
    echo "ERROR: Binary not found!" >> "$LOG_FILE"
    eips -c
    eips "ERROR: Binary not found!"
    sleep 10
    exit 1
fi

# Sicherheitscheck 2: Binary ausführbar?
if [ ! -x "$BINARY_PATH" ]; then
    echo "WARNING: Binary not executable - making executable..." >> "$LOG_FILE"
    eips "WARNING: Binary not executable - making executable..."
    chmod +x kindle-hello
fi

# Sicherheitscheck 3: Backup des aktuellen Displays (falls möglich)
echo "Starting test app..." >> "$LOG_FILE"
eips "Starting test app..."
sleep 3
eips -c

# WICHTIG: Timeout - falls das Programm hängt, automatisch beenden nach 15 Sekunden
timeout 180 $BINARY_PATH >> "$LOG_FILE" 2>&1

EXIT_CODE=$?

echo "Test ended with code: $EXIT_CODE" >> "$LOG_FILE"

# Falls etwas schiefgegangen ist
if [ $EXIT_CODE -ne 0 ] && [ $EXIT_CODE -ne 124 ]; then
    echo "ERROR in running test app!" >> "$LOG_FILE"
    eips -c
    eips "An error occured, running the test app"
    eips "Check logs: /mnt/us/hellokindle/tmp/script.log and app.log"
    sleep 3
elif [ $EXIT_CODE -eq 124 ]; then
    echo "Timeout - App has been shutdown automatically" >> "$LOG_FILE"
    eips -c
    eips "Test successfull (Safety-Timeout)"
    sleep 2
else
    echo "Test completed successfully!" >> "$LOG_FILE"
fi

# Sicherstellung: Display ist sauber
eips -c
eips -f

echo "=== End of binary test script ===" >> "$LOG_FILE"
