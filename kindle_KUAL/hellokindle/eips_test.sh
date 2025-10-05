#!/bin/sh


EXTENSION_PATH="/mnt/us/extensions/hellokindle"
LOG_FILE="/mnt/us/hellokindle/tmp/eips-test.log"

echo "=== Kindle Hello World Test gestartet ===" > "$LOG_FILE"
echo "Zeit: $(date)" >> "$LOG_FILE"

cd "$EXTENSION_PATH"

echo "Starte Test-App..." >> "$LOG_FILE"

echo "First screen clear..." >> "$LOG_FILE"
eips -c
eips ""


# DRAW CHESSBOARD
for j in 0 1 2 3; do

    for i in 0 1 2 3; do
        x=$((i * 268))
        y=$((j * 268))
        eips -d l=10,w=134,h=134 -x $x -y $y
    done

    for i in 0 1 2 3; do
        x=$((134 + i * 268))
        y=$(( 134 + j * 268))
        eips -d l=10,w=134,h=134 -x $x -y $y
        sleep 0.5
    done

done
sleep 20
eips -c



# echo "String-Print" >> "$LOG_FILE"
# eips "Teststring"
# sleep 2
# eips ''
# sleep 2
# eips -c

# echo "Highlighted String-Print" >> "$LOG_FILE"
# eips "Highlighted Teststring" -h
# sleep 2
# eips ''
# sleep 2
# eips -c

# echo "String-Print" >> "$LOG_FILE"
# eips 2 2 "Laaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaanger Teststring"
# sleep 2
# eips ''
# sleep 2


# echo "Black-Horse-Transp-BG" >> "$LOG_FILE"
# eips -g "/mnt/us/hellokindle/assets/black_horse.png" # not working bc transparent background
# sleep 2
# eips ''
# sleep 2
# eips -c

# echo "Black-Horse-white-BG" >> "$LOG_FILE"
# eips -g "/mnt/us/hellokindle/assets/black_horse_w_bg.png" # works, but see conversion command to 8-bit greyscale in wiki
# sleep 2
# eips ''
# sleep 2

# Sicherstellung: Display ist sauber
echo "Clearing display" >> "$LOG_FILE"
eips -c
eips -f

echo "=== Test-Skript Ende ===" >> "$LOG_FILE"
