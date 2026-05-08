# kindle-chess
KUAL-app written in Rust, implementing the free and open lichess APIs to enable (online) chess games on amazon's (jailbroken) kindle fire 7

# Features
- Retrieve ongoing games via lichessapi
- Continue ongoing chess game
- Authenticate via phone through QR
- Update directly via github release possible

# Compile to Kindle binary:
`RUSTFLAGS="-C target-feature=+crt-static" cross build --target arm-unknown-linux-musleabi --release`
