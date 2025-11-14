#!/bin/sh

cd kindle_chess_ui

echo "Building Binaries"
podman run --rm -v "$PWD":/workspace:Z -w /workspace --env CARGO_NET_OFFLINE=false docker.io/messense/rust-musl-cross:arm-musleabi cargo build --release --target arm-unknown-linux-musleabi

echo "Copying Binaries to local KUAL-App"
yes | cp -r ./target/arm-unknown-linux-musleabi/release/kindle-x11-test ../kindle_KUAL/hellokindle/bin/

cd ..

echo "Deleting Logs before copying.."
yes | rm ./kindle_KUAL/hellokindle/log/*

echo "Temporarly moving token before copying.."
mv ./kindle_KUAL/hellokindle/secrets/token.json ./

echo "Flashing local KUAL-App to Kindle"
scp -r ./kindle_KUAL/* root@kindle:/mnt/us

echo "Moving token back.."
mv ./token.json ./kindle_KUAL/hellokindle/secrets/
echo "Done flashing!"
