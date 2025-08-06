use std::{env, path::PathBuf};

fn main() {
    let network_dir = env::var("EVALFILE").unwrap_or_else(|_| "./networks/cherry_736-256-v2".to_string());
    let network_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("network.bin");
    let network_bytes = std::fs::read(&network_dir).unwrap();

    std::fs::write(&network_path, &network_bytes).unwrap();

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={}", network_dir);
}