#!/bin/sh

# echo "Flashing KUAL-extension"
# scp -r ./kindle_KUAL/extensions/* root@kindle:/mnt/us/extensions/

cd kindle_chess

echo "Building Binaries"
RUSTFLAGS="-C target-feature=+crt-static" cross build --target armv7-unknown-linux-musleabi --release

echo "Copy Binaries to local KUAL-App"
yes | cp -r ./target/armv7-unknown-linux-musleabi/release/kindle-hello ../kindle_KUAL/hellokindle/bin/

cd ..

echo "Flash local KUAL-App to Kindle"
scp -r ./kindle_KUAL/* root@kindle:/mnt/us
