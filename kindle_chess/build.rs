use dotenv::from_filename;
use std::env;

fn main() {
    get_env_variables();
}

fn get_env_variables() -> () {
    let profile = std::env::var("PROFILE").unwrap();
    let env_file = match profile.as_str() {
        "release" => ".env.release",
        _ => ".env.debug",
    };

    from_filename(env_file).expect(&format!("Failed to load {}", env_file));

    let root_dir = env::var("ROOT_DIR").expect("ROOT_DIR must be set");

    // Pass to compiler
    println!("cargo:rustc-env=ROOT_DIR={}", root_dir);
    println!("cargo:rustc-env=LOG_FILE_DIR={}{}", root_dir, "log/");
    println!("cargo:rustc-env=ASSETS_DIR={}{}", root_dir, "assets/");
    println!(
        "cargo:rustc-env=AUTH_TOKEN={}{}",
        root_dir, "secrets/token.json"
    );
    println!(
        "cargo:rustc-env=LICHESS_API_BASE={}",
        "https://lichess.org/api"
    );
}
