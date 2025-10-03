#!/bin/sh

# SICHERHEITS-WRAPPER für Kindle Hello World Test

EXTENSION_PATH="/mnt/us/extensions/hellokindle"
LOG_FILE="/mnt/us/hellokindle/tmp/hello-test.log"

echo "=== Kindle Hello World Test gestartet ===" > "$LOG_FILE"
echo "Zeit: $(date)" >> "$LOG_FILE"

cd "$EXTENSION_PATH"

# Sicherheitscheck 3: Backup des aktuellen Displays (falls möglich)
echo "Starte Test-App..." >> "$LOG_FILE"

echo "First screen clear..." >> "$LOG_FILE"
eips -c
sleep 2

echo "String-Print" >> "$LOG_FILE"
eips "Teststring"
sleep 2
eips ''
sleep 2
eips -c

echo "Highlighted String-Print" >> "$LOG_FILE"
eips "Highlighted Teststring" -h
sleep 2
eips ''
sleep 2
eips -c

echo "String-Print" >> "$LOG_FILE"
eips 2 2 "Laaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaanger Teststring"
sleep 2
eips ''
sleep 2

echo "Black-Horse-Transp-BG" >> "$LOG_FILE"
eips -g "/mnt/us/hellokindle/assets/black_horse.png" # not working bc transparent background
sleep 2
eips ''
sleep 2
eips -c


echo "Black-Horse-white-BG" >> "$LOG_FILE"
eips -g "/mnt/us/hellokindle/assets/black_horse_w_bg.png" # works, but see conversion command to 8-bit greyscale in wiki
sleep 2
eips ''
sleep 2

# Sicherstellung: Display ist sauber
echo "Clearing display" >> "$LOG_FILE"
eips -c
eips -f

echo "=== Test-Skript Ende ===" >> "$LOG_FILE"
