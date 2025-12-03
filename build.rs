use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Load .env file
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let env_path = Path::new(&manifest_dir).join(".env");

    if env_path.exists() {
        let content = fs::read_to_string(&env_path).expect("Failed to read .env file");

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Set environment variable for compile time
                println!("cargo:rustc-env={}={}", key, value);
            }
        }

        // Rerun build if .env changes
        println!("cargo:rerun-if-changed=.env");
    }
}
