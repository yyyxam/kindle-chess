// build.rs
use std::env;
use std::fs;

fn main() {
    // Read .env file manually to avoid dotenv dependency
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let env_file = match profile.as_str() {
        "release" => ".env.release",
        _ => ".env.debug",
    };

    if let Ok(contents) = fs::read_to_string(env_file) {
        for line in contents.lines() {
            if let Some(eq_pos) = line.find('=') {
                let key = &line[..eq_pos];
                let value = &line[eq_pos + 1..];
                if key == "ROOT_DIR" {
                    println!("cargo:rustc-env=ROOT_DIR={}", value);
                    println!("cargo:rustc-env=LOG_FILE_DIR={}log/", value);
                    println!("cargo:rustc-env=ASSETS_DIR={}assets/", value);
                    println!("cargo:rustc-env=AUTH_TOKEN={}secrets/token.json", value);
                }
            }
        }
    }

<<<<<<< HEAD
    let root_dir = env::var("ROOT_DIR").expect("ROOT_DIR must be set");

    // Pass to compiler
    println!("cargo:rustc-env=ROOT_DIR={}", root_dir);
    println!("cargo:rustc-env=LOG_FILE_DIR={}{}", root_dir, "log/");
    println!("cargo:rustc-env=TARGET=arm-unknown-linux-musleabihf");
    println!("cargo:rustc-env=LICHESS_API_BASE=https://lichess.org/api");
}
