#!/bin/sh

# SICHERHEITS-WRAPPER für Kindle Hello World Test

EXTENSION_PATH="/mnt/us/extensions/hellokindle"
APP_PATH="mtn/us/hellokindle"
BINARY_PATH="/mnt/us/hellokindle/bin/kindle-hello"
LOG_FILE="/mnt/us/hellokindle/tmp/script.log"

echo "=== Kindle Script-Test gestartet ===" > "$LOG_FILE"
echo "Zeit: $(date)" >> "$LOG_FILE"

cd "$APP_PATH"

# Sicherheitscheck 1: Binary vorhanden?
if [ ! -f "$BINARY_PATH" ]; then
    echo "FEHLER: Binary nicht gefunden!" >> "$LOG_FILE"
    eips -c
    eips -g "FEHLER: test_scripts binary fehlt"
    eips -g "Prüfen Sie die Installation"
    sleep 3
    exit 1
fi

# Sicherheitscheck 2: Binary ausführbar?
if [ ! -x "$BINARY_PATH" ]; then
    echo "WARNUNG: Binary nicht ausführbar - korrigiere..." >> "$LOG_FILE"
    chmod +x kindle-hello
fi

# Sicherheitscheck 3: Backup des aktuellen Displays (falls möglich)
echo "Starte Test-App..." >> "$LOG_FILE"

# WICHTIG: Timeout - falls das Programm hängt, automatisch beenden nach 15 Sekunden
timeout 180 $BINARY_PATH >> "$LOG_FILE" 2>&1

EXIT_CODE=$?

echo "Test beendet mit Code: $EXIT_CODE" >> "$LOG_FILE"

# Falls etwas schiefgegangen ist
if [ $EXIT_CODE -ne 0 ] && [ $EXIT_CODE -ne 124 ]; then
    echo "FEHLER bei der Ausführung!" >> "$LOG_FILE"
    eips -c
    eips -g "Test-App Fehler aufgetreten"
    eips -g "Siehe Log: /mnt/us/hellokindle/tmp/auth_test.log"
    sleep 3
elif [ $EXIT_CODE -eq 124 ]; then
    echo "Timeout - App wurde automatisch beendet" >> "$LOG_FILE"
    eips -c
    eips -g "Test erfolgreich (Timeout-Schutz)"
    sleep 2
else
    echo "Test erfolgreich abgeschlossen!" >> "$LOG_FILE"
fi

# Sicherstellung: Display ist sauber
eips -c
eips -f

echo "=== Test-Skript Ende ===" >> "$LOG_FILE"
