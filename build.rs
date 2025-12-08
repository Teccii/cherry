use std::{env, fs, io::Write, path::PathBuf};

fn main() {
    let network_dir = env::var("EVALFILE").unwrap_or_else(|_| "./networks/default.bin".to_string());
    let network_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("network.bin");
    let network_bytes = fs::read(&network_dir).unwrap();

    fs::write(&network_path, &network_bytes).unwrap();

    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={}", network_dir);
}
