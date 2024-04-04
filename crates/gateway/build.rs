use std::process::Command;

fn main() {
    install_starknet_sierra_compile();
}

fn install_starknet_sierra_compile() {
    println!("Installing starknet-sierra-compile");
    let mut command = Command::new("cargo");
    command.arg("install");
    command.arg("--root");
    command.arg("tmp/cargo"); // TODO: DOn't dup the path.
    command.arg("starknet-sierra-compile");
    let compile_output = command
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute command: {}", e));

    if !compile_output.status.success() {
        let stderr_output = String::from_utf8(compile_output.stderr).unwrap(); // TODO: handle error
        panic!("Failed to compile Sierra code: {}", stderr_output);
    };
}
