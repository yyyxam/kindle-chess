# kindle-chess
KUAL-app written in Rust, implementing the free and open lichess APIs to enable (online) chess games on amazon's (jailbroken) kindle fire 7

# Compile to Kindle binary:
'RUSTFLAGS="-C target-feature=+crt-static" cross build --target arm-unknown-linux-musleabi --release'
