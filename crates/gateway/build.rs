use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .arg("install")
        .arg("starknet-sierra-compile")
        .output()
        .expect("Failed to install starknet-sierra-compile");

    if !output.status.success() {
        eprintln!("Cargo install failed: {:?}", output);
        std::process::exit(1);
    }
}
